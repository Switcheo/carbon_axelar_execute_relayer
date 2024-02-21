use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use ethers::{
    contract::EthEvent,
    core::types::{Address, Filter, H256, U256},
    prelude::*,
    providers::{Provider, Ws},
};
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use tracing::{error, info, instrument, warn};

use crate::conf::Chain;
use crate::db::{DbPayloadAcknowledgedEvent, DbWithdrawTokenAcknowledgedEvent, PayloadType};

#[derive(Debug, Clone, PartialEq, Eq, Default, EthEvent)]
#[ethevent(name = "ContractCallApproved", abi = "ContractCallApproved(bytes32,string,string,address,bytes32,bytes32,uint256)")]
pub struct ContractCallApprovedEvent {
    #[ethevent(indexed)]
    pub command_id: H256,
    pub source_chain: String,
    pub source_address: String,
    #[ethevent(indexed)]
    pub contract_address: Address,
    #[ethevent(indexed)]
    pub payload_hash: H256,
    pub source_tx_hash: H256,
    pub source_event_index: U256,
}

#[instrument(name = "listener_evm", skip_all)]
pub async fn init_all_ws(evm_chains: Vec<Chain>, pg_pool: Arc<PgPool>) {
    for chain in evm_chains {
        let pg_pool_clone = pg_pool.clone();
        let chain_clone = chain.clone();
        info!("Subscribing to {} on {}", &chain.name, &chain.ws_url);
        tokio::spawn(async move {
            if let Err(e) = init_ws(chain_clone, pg_pool_clone).await {
                error!("Error initializing WebSocket for {}: {}", &chain.ws_url, e);
            }
        });
    }
}

// init_ws connect to the evm network via WebSocket and watch for relevant events
#[instrument(name = "listener_evm", skip_all, fields(chain = chain_config.name))]
async fn init_ws(chain_config: Chain, pg_pool: Arc<PgPool>) -> Result<()> {
    // Connect to the evm node
    let provider = Provider::<Ws>::connect_with_reconnects(&chain_config.ws_url, 1000).await
        .context("Failed to connect to WS")?;

    let provider = Arc::new(provider);

    info!("Connected to {:?}", &chain_config.ws_url);

    let address = chain_config.axelar_gateway_proxy.parse::<Address>()?;
    let address = ValueOrArray::Value(address);
    // filter for contract_address (2nd indexed topic)
    let topic2 = H256::from(chain_config.carbon_axelar_gateway.clone().parse::<Address>()?);

    let event = ContractCallApprovedEvent::new::<_, Provider<Ws>>(
        Filter::new().address(address).topic2(topic2),
        Arc::clone(&provider)
    );
    let mut events = event.subscribe().await?.take(5);

    info!("Starting to watch {:?} for ContractCallApprovedEvent", &chain_config.name);
    while let Some(log) = events.next().await {
        // TODO: extract to separate thread?
        match log {
            Ok(event) => {
                info!("Received ContractCallApprovedEvent for carbon_axelar_gateway ({:?}): {:?}", &chain_config.carbon_axelar_gateway, event);
                let payload_hash = format!("{:?}", event.payload_hash);

                // get the payload event
                let payload_acknowledged_result = get_payload_acknowledged_event(&pg_pool, &payload_hash).await;
                let payload_acknowledged_event = match payload_acknowledged_result {
                    Ok(event) => {
                        match event {
                            Some(event) => {
                                info!("Found matching event payload_acknowledged_events in DB with payload_hash: {:?}", &payload_hash);
                                event
                            },
                            None => {
                                warn!("Skipping as DbPayloadAcknowledgedEvent payload_hash {:?} does not exist in DB", &payload_hash);
                                continue
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error while querying DB for DbPayloadAcknowledgedEvent, error: {:?}", &e);
                        continue
                    }
                };

                // If it is a token withdraw, we need to check withdrawal event to validate the fee
                if payload_acknowledged_event.payload_type == PayloadType::Withdraw.to_i32() {
                    if let Err(e) = validate_withdraw(&pg_pool, &payload_hash).await {
                        error!("Skipping withdrawal payload_hash {:?} due to error: {:?}", &payload_hash, e);
                        continue
                    }
                }

                // Save event to db
                match sqlx::query!(
                    "INSERT INTO contract_call_approved_events (command_id, blockchain, broadcast_status, source_chain, source_address, contract_address, payload_hash, source_tx_hash, source_event_index, payload) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                    format!("{:?}", event.command_id),
                    chain_config.name,
                    "pending_broadcast",
                    event.source_chain,
                    event.source_address,
                    format!("{:?}", event.contract_address),
                    &payload_hash,
                    format!("{:?}", event.source_tx_hash),
                    BigDecimal::from_str(&event.source_event_index.to_string()).unwrap(),
                    &payload_acknowledged_event.payload
                )
                    .execute(&*pg_pool)
                    .await {
                    Ok(_result) => info!("Inserted event successfully with payload_hash {}", &payload_hash),
                    Err(e) => error!("Unable to insert event, err {}:", e),
                };
            }
            Err(e) => error!("Error listening for ContractCallApprovedEvent logs: {:?}", e),
        }
    }

    Ok(())
}


async fn get_payload_acknowledged_event(pg_pool: &Arc<PgPool>, payload_hash: &String) -> Result<Option<DbPayloadAcknowledgedEvent>> {
    // Check if we should broadcast this event by checking the withdraw_token_acknowledged_events
    sqlx::query_as::<_, DbPayloadAcknowledgedEvent>(
        "SELECT * FROM payload_acknowledged_events WHERE payload_hash = $1",
    )
        .bind(&payload_hash)
        .fetch_optional(pg_pool.as_ref()).await.context("sql query error for payload_acknowledged_events")
}

async fn validate_withdraw(pg_pool: &Arc<PgPool>, payload_hash: &String) -> Result<()> {
    // Check if we should broadcast this event by checking the withdraw_token_acknowledged_events
    let result = sqlx::query_as::<_, DbWithdrawTokenAcknowledgedEvent>(
        r#"
                        SELECT * FROM withdraw_token_acknowledged_events
                        WHERE payload_hash = $1
                        AND (coin->>'amount')::numeric > 0
                        AND (relay_fee->>'amount')::numeric > 0
                        "#,
    )
        .bind(&payload_hash)
        .fetch_optional(pg_pool.as_ref()).await?;

    let withdraw_event = match result {
        Some(event) => {
            info!("Found matching withdraw_token_acknowledged_events in DB with payload_hash: {:?}", &payload_hash);
            event
        }
        None => {
            anyhow::bail!("Skipping as DbWithdrawTokenAcknowledgedEvent payload_hash {:?} does not exist in DB or has 0 amounts", &payload_hash);
        }
    };

    // TODO: translate to handle different relay fee denom and amounts
    if withdraw_event.relay_fee.amount < 10 {
        // 10 is just an arbitrary number, we should do custom logic to convert price
        warn!("withdraw_event.relay_fee.amount < 10");
        anyhow::bail!("withdraw_event.relay_fee.amount < 10");
    }
    Ok(())
}
