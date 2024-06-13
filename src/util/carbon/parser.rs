use std::str::FromStr;

use base64::Engine;
use base64::engine::general_purpose;
use ethers::utils::hex::encode_prefixed;
use ethers::utils::keccak256;
use sqlx::types::BigDecimal;

use crate::db::{BridgeRevertedEvent, DbAxelarCallContractEvent, DbPendingActionEvent, ExpiredPendingActionEvent};
use crate::util::cosmos::Event;

pub fn strip_quotes(input: &str) -> &str {
    input.trim_matches('"')
}

pub fn parse_bridge_pending_action_event(event: Event) -> DbPendingActionEvent {
    let connection_id = event.attributes.iter().find(|a| a.key == "connection_id").map(|a| a.value.clone()).unwrap_or_default();
    let connection_id = strip_quotes(&connection_id).to_string();
    let relay_details = event.attributes.iter().find(|a| a.key == "relay_details").map(|a| a.value.clone()).unwrap_or_default();
    let relay_details = serde_json::from_str(&relay_details)
        .map_err(|e| e.to_string()).expect("could not parse relay_details");

    let nonce = event.attributes.iter().find(|a| a.key == "nonce").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = BigDecimal::from_str(strip_quotes(&nonce))
        .expect("Failed to parse nonce into BigDecimal");
    let pending_action_type = event.attributes.iter().find(|a| a.key == "pending_action_type").map(|a| a.value.clone()).unwrap_or_default();
    let pending_action_type = strip_quotes(&pending_action_type).parse::<i32>()
        .expect("Failed to parse pending_action_type into integer");

    let (bridge_id, chain_id, _) = parse_connection_id(&connection_id);
    
    return DbPendingActionEvent {
        id: -1,
        connection_id,
        bridge_id,
        chain_id,
        nonce,
        pending_action_type,
        retry_count: 0,
        relay_details,
    }
}

pub fn parse_expired_pending_action_event(event: Event) -> ExpiredPendingActionEvent {
    let nonce = event.attributes.iter().find(|a| a.key == "nonce").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = BigDecimal::from_str(strip_quotes(&nonce))
        .expect("Failed to parse nonce into BigDecimal");
    let pending_action_type = event.attributes.iter().find(|a| a.key == "pending_action_type").map(|a| a.value.clone()).unwrap_or_default();
    let pending_action_type = strip_quotes(&pending_action_type).parse::<i32>()
        .expect("Failed to parse pending_action_type into integer");
    let connection_id = event.attributes.iter().find(|a| a.key == "connection_id").map(|a| a.value.clone()).unwrap_or_default();
    let connection_id = strip_quotes(&connection_id).to_string();
    let relay_details = event.attributes.iter().find(|a| a.key == "relay_details").map(|a| a.value.clone()).unwrap_or_default();
    let relay_details = serde_json::from_str(&relay_details)
        .map_err(|e| e.to_string()).expect("could not parse relay_details");

    return ExpiredPendingActionEvent {
        nonce,
        pending_action_type,
        connection_id,
        relay_details,
    }
}

pub fn parse_bridge_reverted_event(event: Event) -> BridgeRevertedEvent {
    let bridge_id = event.attributes.iter().find(|a| a.key == "bridge_id").map(|a| a.value.clone()).unwrap_or_default();
    let bridge_id = strip_quotes(&bridge_id).to_string();
    let chain_id = event.attributes.iter().find(|a| a.key == "chain_id").map(|a| a.value.clone()).unwrap_or_default();
    let chain_id = strip_quotes(&chain_id).to_string();
    let gateway_address = event.attributes.iter().find(|a| a.key == "gateway_address").map(|a| a.value.clone()).unwrap_or_default();
    let gateway_address = strip_quotes(&gateway_address).to_string();
    let nonce = event.attributes.iter().find(|a| a.key == "nonce").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = BigDecimal::from_str(strip_quotes(&nonce))
        .expect("Failed to parse nonce into BigDecimal");

    return BridgeRevertedEvent {
        id: -1,
        bridge_id,
        chain_id,
        gateway_address,
        nonce,
    }
}

pub fn parse_axelar_call_contract_event(event: Event) -> DbAxelarCallContractEvent {
    let nonce = event.attributes.iter().find(|a| a.key == "nonce").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = BigDecimal::from_str(strip_quotes(&nonce))
        .expect("Failed to parse nonce into BigDecimal");
    // let payload_encoding = event.attributes.iter().find(|a| a.key == "payload_encoding").map(|a| a.value.clone()).unwrap_or_default();
    // let payload_encoding = strip_quotes(&payload_encoding).to_string();
    let payload = event.attributes.iter().find(|a| a.key == "payload").map(|a| a.value.clone()).unwrap_or_default();
    let payload_base64 = strip_quotes(&payload).to_string();
    let payload_bytes = general_purpose::STANDARD.decode(payload_base64).unwrap();
    let payload_hex = encode_prefixed(&payload_bytes);

    // get payload_hash
    let payload_hash = keccak256(&payload_bytes);
    let payload_hash = encode_prefixed(payload_hash);

    return DbAxelarCallContractEvent {
        id: -1,
        nonce,
        payload_hash,
        payload: payload_hex,
        payload_encoding: "evm_abi".to_string(),
    }
}

// parse connection_id into bridge_id, chain_id, contract_addr
pub fn parse_connection_id(connection_id: &str) -> (String, String, String) {
    let parts: Vec<&str> = connection_id.split('/').collect();
    if parts.len() < 3 {
        panic!("connection_id requires at least three parts separated by '/', got {}", connection_id);
    }
    (
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
    )
}