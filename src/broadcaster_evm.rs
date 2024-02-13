use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::{Context, Result};

use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use hex::decode as hex_decode;
use sqlx::{PgPool};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, interval};

use crate::conf::ChainConfig;
use crate::db::DbContractCallApprovedEvent;

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
        isContractCallApproved(bytes32,string,string,address,bytes32())(bool)
    ]"#
);


pub async fn init_all(evm_chains: Vec<ChainConfig>, pg_pool: Arc<PgPool>) {
    let evm_chains_clone = evm_chains.clone();
    // initialize signature providers for each chain
    let channel_tx_map = init_channels(evm_chains_clone, pg_pool.clone()).await;

    // listen for db events that have not been broadcast
    let pg_pool_clone = pg_pool.clone();
    poll_for_new_events(pg_pool_clone, channel_tx_map).await;
}


async fn poll_for_new_events(pool: Arc<PgPool>, channel_tx_map: HashMap<String, Sender<DbContractCallApprovedEvent>>) {
    println!("Watching for events to broadcast");
    let mut interval = interval(Duration::from_secs(5));
    loop {
        interval.tick().await;
        let _ = queue_new_events_for_broadcast(&pool, channel_tx_map.clone()).await;
    }
}

async fn queue_new_events_for_broadcast(pool: &PgPool, channel_tx_map: HashMap<String,
    Sender<DbContractCallApprovedEvent>>) -> Result<(), sqlx::Error> {
    // Implement the logic to check for new events
    println!("Checking for new events...");
    let events: Vec<DbContractCallApprovedEvent> = sqlx::query_as!(
        DbContractCallApprovedEvent,
        "SELECT * FROM contract_call_approved_events WHERE broadcast_status = $1",
        "not_broadcasted"
    )
        .fetch_all(pool)
        .await?;

    for event in events {
        println!("New event found: {:?}", event);
        if let Some(sender) = channel_tx_map.get(&event.blockchain) {
            if sender.send(event).await.is_err() {
                println!("Failed to queue event for broadcast");
            }
        } else {
            println!("No channel found for blockchain: {}", event.blockchain);
        }
    }
    Ok(())
}

async fn init_channels(evm_chains: Vec<ChainConfig>, pg_pool: Arc<PgPool>) -> HashMap<String, Sender<DbContractCallApprovedEvent>> {
    let mut channels = HashMap::new();
    // Initialize providers and channels for each chain
    for chain in evm_chains {
        // init channel
        let (tx, mut rx) = mpsc::channel::<DbContractCallApprovedEvent>(100); // Adjust the size based on expected load
        channels.insert(chain.name.clone(), tx);
        let pg_pool = pg_pool.clone();

        // spawn receiving logic
        tokio::spawn(async move {
            let provider = match init_provider(chain.clone()).await {
                Ok(provider) => provider,
                Err(e) => {
                    eprintln!("Error initializing provider for {}: {}", chain.name, e);
                    return
                }
            };
            let axelar_gateway = match chain.axelar_gateway_proxy.parse::<Address>() {
                Ok(address) => address,
                Err(e) => {
                    // Handle the error, e.g., log it or return from the block
                    eprintln!("Error parsing address: {}", e);
                    return; // Exit the async block early
                }
            };
            let axelar_gateway = IAxelarGateway::new(axelar_gateway, provider.clone());

            while let Some(event) = rx.recv().await {
                let event_clone = event.clone();
                let command_id = H256::from_str(&event.command_id).expect("Failed to parse command_id");
                let contract_address = Address::from_str(&event.contract_address).expect("Failed to parse contract_address");
                let payload_hash = H256::from_str(&event.payload_hash).expect("Failed to parse payload_hash");

                // Query blockchain to check if the contract call has already been approved
                let is_executed = axelar_gateway.is_contract_call_approved(
                    command_id.0,
                    event.source_chain,
                    event.source_address,
                    contract_address,
                    payload_hash.0,
                )
                    .call()
                    .await
                    .unwrap_or(false);
                if is_executed {
                    // If already executed, mark db event as executed
                    let _ = sqlx::query!(
                        "UPDATE contract_call_approved_events SET broadcast_status = $1 WHERE id = $2",
                        "executed",
                        event.id
                    )
                        .execute(pg_pool.as_ref())
                        .await;
                    println!("Event already executed: {:?}", event.id);
                }

                let mut is_broadcast = false;

                // broadcast
                match broadcast_tx(chain.clone(), event_clone, provider.clone()).await {
                    Ok(_res) => {
                        is_broadcast = true;
                    }
                    Err(e) => {
                        eprintln!("Broadcast failed with error: {:?}", e)
                    }
                };

                if !is_broadcast {
                    return;
                }

                if let Err(e) = sqlx::query!(
                            "UPDATE contract_call_approved_events SET broadcast_status = $1 WHERE id = $2",
                            "broadcasting",
                            event.id
                        )
                    .execute(pg_pool.as_ref())
                    .await {
                    eprintln!("UPDATE failed with error: {:?}", e);
                }
            }
        });
    }
    channels
}

async fn init_provider(chain: ChainConfig) -> Result<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
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

async fn broadcast_tx(chain: ChainConfig, event: DbContractCallApprovedEvent, provider: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>) -> Result<(), Box<dyn std::error::Error>> {
    // call execute(bytes32,string,string,bytes)()
    let executable = chain.carbon_axelar_gateway.parse::<Address>()?;
    let executable = IAxelarExecutable::new(executable, provider.clone());

    let command_id_hex = hex_decode(&event.command_id)
        .expect("Failed to decode hex string");
    let command_id_h256 = H256::from_slice(&command_id_hex);

    let payload_bytes = match hex_decode(&event.payload) {
        Ok(bytes) => Bytes::from(bytes),
        Err(e) => {
            // Handle the error, e.g., log it or return an Err from your function
            eprintln!("Failed to decode payload_hash: {:?}", e);
            return Err(Box::new(e)); // Adjust error handling as needed
        }
    };

    let receipt = executable
        .execute(
            command_id_h256.0,
            event.source_chain,
            event.source_address,
            payload_bytes,
        )
        .send()
        .await?
        .await?
        .expect("no receipt for execute");
    println!("execute successfully!");
    println!("{receipt:?}");
    if receipt.status == Some(U64::from(1)) {
        println!("Transaction successfully executed");
    } else {
        println!("Transaction failed");
    }

    Ok(())
}