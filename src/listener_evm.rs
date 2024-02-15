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

use crate::conf::ChainConfig;
use crate::db::DbWithdrawTokenAcknowledgedEvent;

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
pub async fn init_all_ws(evm_chains: Vec<ChainConfig>, pg_pool: Arc<PgPool>) {
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
async fn init_ws(chain_config: ChainConfig, pg_pool: Arc<PgPool>) -> Result<()> {
    // Connect to the evm node
    let provider = Provider::<Ws>::connect(&chain_config.ws_url).await
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
        match log {
            Ok(event) => {
                info!("Received ContractCallApprovedEvent for carbon_axelar_gateway ({:?}): {:?}", &chain_config.carbon_axelar_gateway, event);
                let payload_hash = format!("{:?}", event.payload_hash);

                // check if we should broadcast this event by checking the withdraw_token_acknowledged_events
                let result = sqlx::query_as::<_, DbWithdrawTokenAcknowledgedEvent>(
                    r#"
                        SELECT * FROM withdraw_token_acknowledged_events
                        WHERE payload_hash = $1
                        AND (coin->>'amount')::numeric > 0
                        AND (relay_fee->>'amount')::numeric > 0
                        "#,
                )
                    .bind(&payload_hash)
                    .fetch_optional(&*pg_pool).await?;

                let withdraw_event = match result {
                    Some(event) => {
                        info!("Found matching event in DB with payload_hash: {:?}", &payload_hash);
                        event
                    }
                    None => {
                        warn!("DbWithdrawTokenAcknowledgedEvent payload_hash {:?} does not exist in DB or has 0 amounts", &payload_hash);
                        continue;
                    }
                };

                // TODO: translate to handle different relay fee denom and amounts
                if withdraw_event.relay_fee.amount < 10 {
                    // 10 is just an arbitrary number, we should do custom logic to convert price
                    warn!("withdraw_event.relay_fee.amount < 10");
                    continue;
                }


                // Process the event data as needed
                // save to db
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
                    &withdraw_event.payload
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
