use std::str::FromStr;
use std::sync::Arc;
use ethers::utils::hex::{decode, encode_prefixed};
use ethers::utils::keccak256;
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use tracing::{error, info};
use crate::conf::Carbon;
use crate::db::PayloadType;
use crate::util::cosmos::Event;
use crate::util::strip_quotes;

pub async fn save_bridge_pending_action_event(pg_pool: Arc<PgPool>, event: &Event) {
    let connection_id = event.attributes.iter().find(|a| a.key == "connection_id").map(|a| a.value.clone()).unwrap_or_default();
    let relay_fee = event.attributes.iter().find(|a| a.key == "relay_fee").map(|a| a.value.clone()).unwrap_or_default();
    let relay_fee = serde_json::from_str::<serde_json::Value>(&relay_fee).unwrap_or_default();
    let relayer_deposit_address = event.attributes.iter().find(|a| a.key == "relayer_deposit_address").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = event.attributes.iter().find(|a| a.key == "nonce").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = BigDecimal::from_str(strip_quotes(&nonce))
        .expect("Failed to parse nonce into BigDecimal");

    // save event details to db
    let result = sqlx::query!(
                        "INSERT INTO withdraw_token_confirmed_events (connection_id, relay_fee, relayer_deposit_address, nonce) VALUES ($1, $2, $3, $4)",
                        strip_quotes(&connection_id),
                        relay_fee,
                        strip_quotes(&relayer_deposit_address),
                        nonce,
                    )
        .execute(&*pg_pool)
        .await;

    match result {
        Ok(_res) => info!("Saved bridge_pending_action_event with nonce {:?}", nonce),
        Err(e) => error!("Failed to insert event data: {}", e)
    }
}

pub async fn save_bridge_acknowledgement_event(pg_pool: Arc<PgPool>, event: &Event) {
    let connection_id = event.attributes.iter().find(|a| a.key == "connection_id").map(|a| a.value.clone()).unwrap_or_default();
    let relay_fee = event.attributes.iter().find(|a| a.key == "relay_fee").map(|a| a.value.clone()).unwrap_or_default();
    let relay_fee = serde_json::from_str::<serde_json::Value>(&relay_fee).unwrap_or_default();
    let relayer_deposit_address = event.attributes.iter().find(|a| a.key == "relayer_deposit_address").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = event.attributes.iter().find(|a| a.key == "nonce").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = BigDecimal::from_str(strip_quotes(&nonce))
        .expect("Failed to parse nonce into BigDecimal");

    // save event details to db
    let result = sqlx::query!(
                        "INSERT INTO withdraw_token_confirmed_events (connection_id, relay_fee, relayer_deposit_address, nonce) VALUES ($1, $2, $3, $4)",
                        strip_quotes(&connection_id),
                        relay_fee,
                        strip_quotes(&relayer_deposit_address),
                        nonce,
                    )
        .execute(&*pg_pool)
        .await;

    match result {
        Ok(_res) => info!("Saved bridge_pending_action_event with nonce {:?}", nonce),
        Err(e) => error!("Failed to insert event data: {}", e)
    }
}
pub async fn save_bridge_revert_event(pg_pool: Arc<PgPool>, event: &Event) {
    let connection_id = event.attributes.iter().find(|a| a.key == "connection_id").map(|a| a.value.clone()).unwrap_or_default();
    let relay_fee = event.attributes.iter().find(|a| a.key == "relay_fee").map(|a| a.value.clone()).unwrap_or_default();
    let relay_fee = serde_json::from_str::<serde_json::Value>(&relay_fee).unwrap_or_default();
    let relayer_deposit_address = event.attributes.iter().find(|a| a.key == "relayer_deposit_address").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = event.attributes.iter().find(|a| a.key == "nonce").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = BigDecimal::from_str(strip_quotes(&nonce))
        .expect("Failed to parse nonce into BigDecimal");

    // save event details to db
    let result = sqlx::query!(
                        "INSERT INTO withdraw_token_confirmed_events (connection_id, relay_fee, relayer_deposit_address, nonce) VALUES ($1, $2, $3, $4)",
                        strip_quotes(&connection_id),
                        relay_fee,
                        strip_quotes(&relayer_deposit_address),
                        nonce,
                    )
        .execute(&*pg_pool)
        .await;

    match result {
        Ok(_res) => info!("Saved bridge_pending_action_event with nonce {:?}", nonce),
        Err(e) => error!("Failed to insert event data: {}", e)
    }
}

pub async fn save_payload_event(carbon_config: &Carbon, pg_pool: Arc<PgPool>, event: &Event) -> bool {
    let payload_type = event.attributes.iter().find(|a| a.key == "payload_type").map(|a| a.value.clone()).unwrap_or_default();
    let payload_type: PayloadType = payload_type.parse().expect("PayloadType::Unknown");

    // check payload type with list of payload types that we want
    if !crate::conf::is_whitelisted_payload(&carbon_config, &payload_type) {
        info!("Payload type not whitelisted for relaying: {:?}", &payload_type);
        return true;
    }

    let bridge_id = event.attributes.iter().find(|a| a.key == "bridge_id").map(|a| a.value.clone()).unwrap_or_default();
    let chain_id = event.attributes.iter().find(|a| a.key == "chain_id").map(|a| a.value.clone()).unwrap_or_default();

    let nonce = event.attributes.iter().find(|a| a.key == "nonce").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = BigDecimal::from_str(strip_quotes(&nonce))
        .expect("Failed to parse nonce into BigDecimal");
    let payload_encoding = event.attributes.iter().find(|a| a.key == "payload_encoding").map(|a| a.value.clone()).unwrap_or_default();
    let payload = event.attributes.iter().find(|a| a.key == "payload").map(|a| a.value.clone()).unwrap_or_default();

    // get payload_hash
    let payload_bytes = decode(strip_quotes(&payload.clone()))
        .expect("Decoding failed");
    let payload_hash = keccak256(&payload_bytes);
    let payload_hash = encode_prefixed(payload_hash);

    // save event details to db
    let result = sqlx::query!(
                        "INSERT INTO payload_acknowledged_events (payload_type, bridge_id, chain_id, nonce, payload_hash, payload, payload_encoding) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                        payload_type as i32,
                        strip_quotes(&bridge_id),
                        strip_quotes(&chain_id),
                        nonce,
                        &payload_hash,
                        encode_prefixed(&payload_bytes),
                        strip_quotes(&payload_encoding),
                    )
        .execute(&*pg_pool)
        .await;

    match result {
        Ok(_res) => info!("Saved PayloadAcknowledgedEvent with payload_hash {:?}, nonce {:?}", &payload_hash, nonce),
        Err(e) => error!("Failed to insert event data: {}", e)
    }
    false
}