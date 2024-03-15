use std::str::FromStr;
use std::sync::Arc;

use ethers::utils::hex::{decode, encode_prefixed};
use ethers::utils::keccak256;
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use tracing::{debug, error, info, instrument};
use url::Url;
use crate::conf::Carbon;
use crate::db::PayloadType;

use crate::ws::{JSONWebSocketClient};

#[derive(Serialize, Deserialize, Debug)]
struct WebSocketMessage {
    id: String,
    jsonrpc: String,
    result: WsResult,
}

#[derive(Serialize, Deserialize, Debug)]
struct WsResult {
    query: String,
    data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    #[serde(rename = "type")]
    pub data_type: String,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Value {
    #[serde(rename = "TxResult")]
    pub tx_result: TxResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxResult {
    pub height: String,
    pub tx: String,
    pub result: TxResultInner,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxResultInner {
    pub data: String,
    pub log: String,
    pub gas_wanted: String,
    pub gas_used: String,
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub attributes: Vec<Attribute>,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attribute {
    pub index: bool,
    pub key: String,
    pub value: String,
}

#[instrument(name = "listener_carbon", skip_all)]
pub async fn init_ws(carbon_config: &Carbon, pg_pool: Arc<PgPool>) {
    info!("Initializing WS for Carbon. Watching {:?} on {:?} for events", &carbon_config.relayer_deposit_address, &carbon_config.ws_url);
    let url = Url::parse(&carbon_config.ws_url).expect(&format!("Invalid WS URL {:?}", &carbon_config.ws_url));

    // create new client
    let mut client = JSONWebSocketClient::new(url);

    // add WithdrawTokenConfirmedEvent subscription
    let pool = pg_pool.clone();
    client.add_cosmos_subscription(
        "1".to_string(),
        format!("Switcheo.carbon.bridge.WithdrawTokenConfirmedEvent.relayer_deposit_address CONTAINS '{}'", &carbon_config.relayer_deposit_address),
        Arc::new(Mutex::new(move |msg: String| {
            // Spawn an async task to handle the message
            let pool = pool.clone();
            tokio::spawn(async move {
                process_withdraw_message(msg, pool.clone()).await;
            });
        })));
    // add PayloadAcknowledgedEvent subscription
    let pool = pg_pool.clone();
    let carbon_config = carbon_config.clone();
    client.add_cosmos_subscription(
        "2".to_string(),
        format!("Switcheo.carbon.bridge.PayloadAcknowledgedEvent.bridge_id CONTAINS '{}'", &carbon_config.axelar_bridge_id),
        Arc::new(Mutex::new(move |msg: String| {
            let pool = pool.clone();
            let carbon_config = carbon_config.clone();
            // Spawn an async task to handle the message
            tokio::spawn(async move {
                process_payload_acknowledged_message(msg, carbon_config.clone(), pool.clone()).await;
            });
        })));

    // connect to WS
    if let Err(e) = client.connect().await {
        error!("Error connecting to client: {:?}", e);
    }
}

// process_withdraw_message processes the WithdrawTokenConfirmedEvent
#[instrument(skip_all)]
async fn process_withdraw_message(msg: String, pg_pool: Arc<PgPool>) {
    info!("Processing new WithdrawTokenConfirmedEvent from Carbon");

    // Process the message and interact with the database
    // Attempt to deserialize the string into WebSocketMessage
    match serde_json::from_str::<WebSocketMessage>(&msg) {
        Ok(query_response) => {
            debug!("Parsed query_response: {:?}", query_response);
            // look for Switcheo.carbon.bridge.WithdrawTokenConfirmedEvent
            let events = query_response.result.data.value.tx_result.result.events;
            let events: Vec<Event> = events.iter().filter(|e| e.event_type == "Switcheo.carbon.bridge.WithdrawTokenConfirmedEvent").cloned().collect();
            for event in events {
                save_withdraw_event(pg_pool.clone(), &event.clone()).await
            }
        }
        Err(e) => error!("Error parsing JSON: {:?}, JSON str:{:?}", e, msg)
    }
}

pub async fn save_withdraw_event(pg_pool: Arc<PgPool>, event: &Event) {
    let coin = event.attributes.iter().find(|a| a.key == "coin").map(|a| a.value.clone()).unwrap_or_default();
    let coin = serde_json::from_str::<serde_json::Value>(&coin).unwrap_or_default();
    let connection_id = event.attributes.iter().find(|a| a.key == "connection_id").map(|a| a.value.clone()).unwrap_or_default();
    let receiver = event.attributes.iter().find(|a| a.key == "receiver").map(|a| a.value.clone()).unwrap_or_default();
    let relay_fee = event.attributes.iter().find(|a| a.key == "relay_fee").map(|a| a.value.clone()).unwrap_or_default();
    let relay_fee = serde_json::from_str::<serde_json::Value>(&relay_fee).unwrap_or_default();
    let relayer_deposit_address = event.attributes.iter().find(|a| a.key == "relayer_deposit_address").map(|a| a.value.clone()).unwrap_or_default();
    let sender = event.attributes.iter().find(|a| a.key == "sender").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = event.attributes.iter().find(|a| a.key == "nonce").map(|a| a.value.clone()).unwrap_or_default();
    let nonce = BigDecimal::from_str(strip_quotes(&nonce))
        .expect("Failed to parse nonce into BigDecimal");

    // save event details to db
    let result = sqlx::query!(
                        "INSERT INTO withdraw_token_confirmed_events (coin, connection_id, receiver, relay_fee, relayer_deposit_address, sender, nonce) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                        coin,
                        strip_quotes(&connection_id),
                        strip_quotes(&receiver),
                        relay_fee,
                        strip_quotes(&relayer_deposit_address),
                        strip_quotes(&sender),
                        nonce,
                    )
        .execute(&*pg_pool)
        .await;

    match result {
        Ok(_res) => info!("Saved withdraw_token_acknowledged_event with nonce {:?}", nonce),
        Err(e) => error!("Failed to insert event data: {}", e)
    }
}

// process_payload_acknowledged_message processes the PayloadAcknowledgedEvent
#[instrument(skip_all)]
async fn process_payload_acknowledged_message(msg: String, carbon_config: Carbon, pg_pool: Arc<PgPool>) {
    info!("Processing new PayloadAcknowledgedEvent from Carbon");

    // Process the message and interact with the database
    // Attempt to deserialize the string into WebSocketMessage
    match serde_json::from_str::<WebSocketMessage>(&msg) {
        Ok(query_response) => {
            debug!("Parsed query_response: {:?}", query_response);
            // look for Switcheo.carbon.bridge.PayloadAcknowledgedEvent
            let events = query_response.result.data.value.tx_result.result.events;
            let events: Vec<Event> = events.iter().filter(|e| e.event_type == "Switcheo.carbon.bridge.PayloadAcknowledgedEvent").cloned().collect();
            for event in &events {
                save_payload_event(&carbon_config, pg_pool.clone(), event).await;
            }
        }
        Err(e) => {
            error!("Error parsing JSON: {:?}, JSON str:{:?}", e, msg);
        }
    }
}

pub async fn save_payload_event(carbon_config: &Carbon, pg_pool: Arc<PgPool>, event: &Event) -> bool {
    let payload_type = event.attributes.iter().find(|a| a.key == "payload_type").map(|a| a.value.clone()).unwrap_or_default();
    let payload_type: PayloadType = payload_type.parse().expect("PayloadType::Unknown");

    // check payload type with list of payload types that we want
    if !is_whitelisted_payload(&carbon_config, &payload_type) {
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
        Ok(_res) => info!("Saved PayloadAcknowledgedEvent with payload_hash {:?}", &payload_hash),
        Err(e) => error!("Failed to insert event data: {}", e)
    }
    false
}

fn is_whitelisted_payload(carbon_config: &Carbon, payload_type: &PayloadType) -> bool {
    if carbon_config.relay_admin_payloads && matches!(payload_type,
            PayloadType::RegisterToken |
            PayloadType::DeregisterToken |
            PayloadType::DeployToken |
            PayloadType::RegisterExecutable |
            PayloadType::DeregisterExecutable |
            PayloadType::ExecuteGateway |
            PayloadType::WithdrawAndExecute |
            PayloadType::PauseContract |
            PayloadType::UnpauseContract
        ) {
        return true;
    }
    if carbon_config.relay_user_payloads && matches!(payload_type,
            PayloadType::Withdraw
        ) {
        return true;
    }
    return false;
}

pub(crate) fn strip_quotes(input: &str) -> &str {
    input.trim_matches('"')
}