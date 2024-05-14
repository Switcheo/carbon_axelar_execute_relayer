use std::str::FromStr;
use std::sync::Arc;
use anyhow::Context;
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use tracing::{error, info, warn};
use crate::conf::Chain;
use crate::db::{DbAxelarCallContractEvent, PayloadType};
use crate::db::carbon_events::get_axelar_call_contract_event;
use crate::util::evm::ContractCallApprovedEvent;

pub async fn save_call_contract_approved_event(chain_config: Chain, pg_pool: Arc<PgPool>, event: ContractCallApprovedEvent) {
    info!("Received ContractCallApprovedEvent for carbon_axelar_gateway ({:?}): {:?}", &chain_config.carbon_axelar_gateway, event);
    let payload_hash = format!("{:?}", event.payload_hash);

    // get the corresponding carbon event
    let axelar_call_contract_event_result = get_axelar_call_contract_event(&pg_pool, &payload_hash).await;
    let axelar_call_contract_event = match axelar_call_contract_event_result {
        Ok(event) => {
            match event {
                Some(event) => {
                    info!("Found matching event axelar_call_contract_event in DB with payload_hash: {:?}", &payload_hash);
                    event
                },
                None => {
                    warn!("Skipping as payload_hash {:?} does not exist in DB on axelar_call_contract_events table", &payload_hash);
                    return
                }
            }
        }
        Err(e) => {
            error!("Error while querying DB for DbPayloadAcknowledgedEvent, error: {:?}", &e);
            return
        }
    };

    // Save event to db
    match sqlx::query!(
                    "INSERT INTO contract_call_approved_events (command_id, blockchain, broadcast_status, source_chain, source_address, contract_address, payload_hash, source_tx_hash, source_event_index, payload) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                    format!("{:?}", event.command_id),
                    chain_config.chain_id,
                    "pending_broadcast",
                    event.source_chain,
                    event.source_address,
                    format!("{:?}", event.contract_address),
                    &payload_hash,
                    format!("{:?}", event.source_tx_hash),
                    BigDecimal::from_str(&event.source_event_index.to_string()).unwrap(),
                    &axelar_call_contract_event.payload
                )
        .execute(&*pg_pool)
        .await {
        Ok(_result) => info!("Inserted event successfully with payload_hash {}", &payload_hash),
        Err(e) => error!("Unable to insert event, err {}:", e),
    };
}
