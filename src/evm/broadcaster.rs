use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use ethers::utils::hex::decode;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{Duration, interval, sleep, timeout};
use tracing::{debug, error, info, instrument, warn};

use crate::conf::Chain;
use crate::db::DbContractCallApprovedEvent;
use crate::db::evm_events::{update_broadcast_status};

abigen!(
    IAxelarExecutable,
    r#"[
        execute(bytes32,string,string,bytes)()
        executeWithToken(bytes32,string,string,bytes,string,uint256)()
    ]"#
);

abigen!(
    IAxelarGateway,
    r#"[
        isContractCallApproved(bytes32,string,string,address,bytes32)(bool)
    ]"#
);

#[instrument(name = "broadcaster_evm", skip_all)]
pub async fn init_all(evm_chains: Vec<Chain>, pg_pool: Arc<PgPool>) {
    let evm_chains_clone = evm_chains.clone();
    // initialize signature providers for each chain
    let channel_tx_map = init_channels(evm_chains_clone, pg_pool.clone()).await;

    // listen for db events that have not been broadcast
    let pg_pool_clone = pg_pool.clone();
    poll_for_new_events(pg_pool_clone, channel_tx_map).await;
}

// Polls for new contract_call_approved_events saved in the DB that can be executed and enqueues them into the broadcast channel
#[instrument(name = "poll_for_new_events", skip_all)]
async fn poll_for_new_events(pool: Arc<PgPool>, channel_tx_map: HashMap<String, Sender<DbContractCallApprovedEvent>>) {
    info!("Watching for events to broadcast");
    let mut interval = interval(Duration::from_secs(5));
    loop {
        interval.tick().await;
        if let Err(e) = queue_new_events_for_broadcast(&pool, channel_tx_map.clone()).await {
            error!("Failed to queue new events for broadcast: {}", e);
        }
    }
}

// Checks the DB for events that can be executed and enqueues them into the broadcast channel
async fn queue_new_events_for_broadcast(pool: &PgPool, channel_tx_map: HashMap<String,
    Sender<DbContractCallApprovedEvent>>) -> Result<()> {
    // check for new events
    debug!("Checking for new events...");
    let events: Vec<DbContractCallApprovedEvent> = sqlx::query_as!(
        DbContractCallApprovedEvent,
        "SELECT * FROM contract_call_approved_events WHERE broadcast_status = $1",
        "pending_broadcast"
    )
        .fetch_all(pool)
        .await?;

    for event in events {
        info!("DB event found: {:?}", event);
        match channel_tx_map.get(&event.blockchain) {
            Some(sender) => {
                if let Err(e) = sender.clone().send(event.clone()).await {
                    error!("Failed to send to channel {:?}, err: {}", &event.blockchain, e);
                }
            }
            None => {
                error!("No channel found for blockchain: {:?}", event.blockchain);
            }
        }
    }
    Ok(())
}

async fn init_channels(evm_chains: Vec<Chain>, pg_pool: Arc<PgPool>) -> HashMap<String, Sender<DbContractCallApprovedEvent>> {
    let mut channels = HashMap::new();
    // Initialize providers and channels for each chain
    for chain in evm_chains {
        info!("Initializing receive_and_broadcast for {:?}", &chain.chain_id);
        // init channel
        let (tx, rx) = mpsc::channel::<DbContractCallApprovedEvent>(100); // Adjust the size based on expected load
        channels.insert(chain.chain_id.clone(), tx);
        let pg_pool = pg_pool.clone();

        // spawn receiving logic
        tokio::spawn(async move {
            if let Err(e) = receive_and_broadcast(chain, rx, pg_pool).await {
                // Handle or log the error e
                error!("Error in receive_and_broadcast: {:?}", e);
            }
        });
    }
    channels
}


#[instrument(name = "broadcaster_evm::receive_and_broadcast", skip_all, fields(
    chain = chain.chain_id
))]
pub async fn receive_and_broadcast(chain: Chain, mut rx: Receiver<DbContractCallApprovedEvent>, pg_pool: Arc<PgPool>) -> Result<()> {
    let provider = init_provider(chain.clone()).await?;
    let axelar_gateway = chain.axelar_gateway_proxy.parse::<Address>()?;
    let axelar_gateway = IAxelarGateway::new(axelar_gateway, provider.clone());

    Ok(while let Some(event) = rx.recv().await {
        let command_id = H256::from_str(&event.command_id).expect("Failed to parse command_id");
        let contract_address = Address::from_str(&event.contract_address).expect("Failed to parse contract_address");
        let payload_hash = H256::from_str(&event.payload_hash).expect("Failed to parse payload_hash");

        // Query blockchain to check if the contract call has already been approved
        let is_approved = axelar_gateway.is_contract_call_approved(
            command_id.0,
            event.source_chain.clone(),
            event.source_address.clone(),
            contract_address,
            payload_hash.0,
        )
            .call()
            .await
            .unwrap_or(false);
        if !is_approved {
            // If already executed, mark db event as executed
            info!("Skipping event as blockchain query for is_contract_call_approved is !approved. This can mean it is already executed, payload_hash: {:?}", &event.payload_hash);
            // update executed
            update_broadcast_status(pg_pool.clone(), &event, "executed").await?;
            continue;
        }

        // Double check db to make sure it is still pending_broadcast
        let exists = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM contract_call_approved_events WHERE id = $1 AND broadcast_status = 'pending_broadcast')",
                event.id.clone()
            )
            .fetch_one(pg_pool.as_ref())
            .await?
            .exists.unwrap_or(false);
        if !exists {
            info!("Skipping event as it is not pending: {:?}", &event.id);
            continue;
        }

        // Update to broadcasting
        update_broadcast_status(pg_pool.clone(), &event, "broadcasting").await?;

        // Attempt to broadcast the transaction
        match broadcast_tx(chain.clone(), event.clone(), provider.clone()).await {
            Ok(_) => {
                info!("broadcast success");
                // If broadcast_tx succeeds, update the execution status
                if let Err(e) = update_broadcast_status(pg_pool.clone(), &event, "executed").await {
                    // Handle the error from update_executed if necessary
                    error!("Failed to update executed status: {:?}", e);
                }
            },
            Err(e) => {
                // Handle the error from broadcast_tx
                error!("Failed to broadcast transaction: {:?}", e);
                // If broadcast_tx fails, update the execution status
                if let Err(e) = update_broadcast_status(pg_pool.clone(), &event, "failed").await {
                    // Handle the error from update_executed if necessary
                    error!("Failed to update executed status: {:?}", e);
                }
            }
        }
    })
}

pub async fn init_provider(chain: Chain) -> Result<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let provider = Provider::<Http>::try_from(chain.rpc_url)
        .context("Failed to connect to the network")?;
    let chain_id = provider.get_chainid().await
        .context("Failed to get chain ID")?;
    let wallet = chain.relayer_private_key.parse::<LocalWallet>()
        .context("Error parsing wallet key")?;
    let wallet = wallet.with_chain_id(chain_id.as_u64());
    let provider = Arc::new(SignerMiddleware::new(provider, wallet));
    Ok(provider)
}

#[instrument(skip_all, fields(payload_hash = event.payload_hash))]
pub async fn broadcast_tx(chain: Chain, event: DbContractCallApprovedEvent, provider: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>) -> Result<()> {
    let executable = chain.carbon_axelar_gateway.parse::<Address>()?;
    let executable = IAxelarExecutable::new(executable, provider.clone());

    let command_id_hex = decode(event.command_id.clone())
        .expect("Failed to decode hex string");
    let command_id_h256 = H256::from_slice(&command_id_hex);

    let payload_bytes = decode(&event.payload)?;


    // get nonce
    let nonce = provider.get_transaction_count(provider.address(), None).await?;

    // get current gas price
    let mut gas_price = provider.get_gas_price().await?;
    gas_price = gas_price / 2;
    let max_retries = 5;
    let mut retries = 0;

    loop {
        // Send the transaction with the current gas price
        let tx = executable
            .execute(
                command_id_h256.0,
                event.source_chain.clone(),
                event.source_address.clone(),
                Bytes::from(payload_bytes.clone()),
            )
            .gas_price(gas_price.clone())
            .nonce(nonce.clone());

        info!("Sending execute tx for command id {}, payload_hash: {}, with evm gas price: {}, evm nonce: {}", event.command_id.clone(), event.payload_hash.clone(), gas_price.clone(), nonce.clone());

        let mut should_retry = false;

        let send_timeout = Duration::from_secs(60);
        match timeout(send_timeout, tx.send()).await {
            Ok(Ok(pending_tx)) => match timeout(send_timeout, pending_tx).await {
                Ok(Ok(Some(receipt))) => {
                    if receipt.status == Some(U64::from(1)) {
                        info!(
                        "Transaction for payload_hash {} successfully executed. tx_hash: {:?}",
                        &event.payload_hash,
                        &receipt.transaction_hash
                    );
                        debug!("Transaction receipt: {receipt:?}");
                        return Ok(());
                    } else {
                        error!("Transaction failed with receipt: {receipt:?}");
                        anyhow::bail!("Transaction failed with receipt: {receipt:?}");
                    }
                }
                Ok(Ok(None)) => {
                    warn!("Transaction receipt not found. Retrying with higher gas price.");
                    should_retry = true;
                }
                Ok(Err(e)) => {
                    error!("Failed to await transaction receipt.");
                    return Err(e).context("Failed to await transaction receipt");
                }
                Err(_) => {
                    error!("Awaiting transaction receipt timed out.");
                    should_retry = true;
                }
            },
            Ok(Err(e)) => {
                if e.to_string().contains("already known") {
                    warn!("Transaction already known. Retrying with higher gas price.");
                    should_retry = true;
                } else {
                    error!("Failed to send transaction.");
                    return Err(e).context("Failed to send transaction");
                }
            }
            Err(_) => {
                error!("Sending transaction timed out.");
                should_retry = true;
            }
        };

        if should_retry {
            if retries < max_retries {
                retries += 1;
                gas_price = gas_price * U256::from(12) / U256::from(10);
                warn!("Retrying transaction with higher gas price: {:?}, in 30s", gas_price);
                sleep(Duration::from_secs(30)).await;
            } else {
                anyhow::bail!("Sending transaction timed out and max retries reached.");
            }
        }
    }
}