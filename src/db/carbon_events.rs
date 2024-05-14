use std::str::FromStr;
use std::sync::Arc;
use anyhow::{Context,Result};
use ethers::utils::hex::{decode, encode_prefixed};
use ethers::utils::keccak256;
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use tracing::{error, info};
use crate::conf::Carbon;
use crate::db::{DbAxelarCallContractEvent, DbPendingActionEvent, PayloadType};
use crate::util::cosmos::Event;
use crate::util::strip_quotes;

pub async fn get_axelar_call_contract_event(pg_pool: &Arc<PgPool>, payload_hash: &String) -> Result<Option<DbAxelarCallContractEvent>> {
    sqlx::query_as::<_, DbAxelarCallContractEvent>(
        "SELECT * FROM axelar_call_contract_events WHERE payload_hash = $1",
    )
        .bind(&payload_hash)
        .fetch_optional(pg_pool.as_ref()).await.context("sql query error for axelar_call_contract_events")
}

pub async fn get_pending_action_by_nonce(pg_pool: &Arc<PgPool>, nonce: &BigDecimal) -> Result<Option<DbPendingActionEvent>> {
    let result = sqlx::query_as::<_, DbPendingActionEvent>(
        "SELECT * FROM pending_action_events WHERE nonce = $1",
    )
        .bind(nonce)
        .fetch_optional(pg_pool.as_ref()).await.context("sql query error for pending_action_events").await;
    match result {
        Some(event) => Ok(Some(event)),
        None => Ok(None),
    }
}

pub async fn get_chain_id_for_nonce(pg_pool: &Arc<PgPool>, nonce: &BigDecimal) -> Result<Option<String>> {
    let result = get_pending_action_by_nonce(pg_pool, nonce).await;
    match result {
        Some(event) => Ok(Some(event.chain_id)),
        None => Ok(None),
    }
}

pub async fn save_bridge_pending_action_event(pg_pool: Arc<PgPool>, event: &DbPendingActionEvent) {
    let result = sqlx::query!(
                        "INSERT INTO pending_action_events (connection_id, bridge_id, chain_id, nonce, pending_action_type, relay_details) VALUES ($1, $2, $3, $4, $5, $6)",
                        event.connection_id,
                        event.bridge_id,
                        event.chain_id,
                        event.nonce,
                        event.pending_action_type,
                        event.get_relay_details_value(),
                    )
        .execute(&*pg_pool)
        .await;

    match result {
        Ok(_res) => info!("Saved bridge_pending_action_event with nonce {:?}", event.nonce),
        Err(e) => error!("Failed to insert event data: {}", e)
    }
}

pub async fn delete_bridge_pending_action_event(pg_pool: Arc<PgPool>, nonce: BigDecimal) {
    let result = sqlx::query!(
                        "DELETE FROM pending_action_events where nonce = $1",
                        nonce,
                    )
        .execute(&*pg_pool)
        .await;

    match result {
        Ok(_res) => info!("deleted bridge_pending_action_event with nonce {:?}", nonce),
        Err(e) => error!("Failed to delete bridge_pending_action_event, err: {}", e)
    }
}

pub async fn save_axelar_call_contract_event(pg_pool: Arc<PgPool>, event: &DbAxelarCallContractEvent) {
    let result = sqlx::query!(
                        "INSERT INTO axelar_call_contract_events (nonce, payload_hash, payload, payload_encoding) VALUES ($1, $2, $3, $4)",
                        event.nonce,
                        &event.payload_hash,
                        event.payload,
                        event.payload_encoding,
                    )
        .execute(&*pg_pool)
        .await;

    match result {
        Ok(_res) => info!("Saved axelar_call_contract_event with payload_hash {:?}, nonce {:?}", &event.payload_hash, event.nonce),
        Err(e) => error!("Failed to insert event data: {}", e)
    }
}