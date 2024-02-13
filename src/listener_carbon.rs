use std::collections::HashMap;
use std::sync::Arc;
use ethers::utils::hex::{decode, encode};
use ethers::utils::keccak256;
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
struct WebSocketMessage {
    id: String,
    jsonrpc: String,
    result: Result,
}

#[derive(Serialize, Deserialize, Debug)]
struct Result {
    query: String,
    data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    #[serde(rename = "type")]
    data_type: String,
    value: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct Value {
    #[serde(rename = "TxResult")]
    tx_result: TxResult,
}

#[derive(Serialize, Deserialize, Debug)]
struct TxResult {
    height: String,
    tx: String,
    result: TxResultInner,
}

#[derive(Serialize, Deserialize, Debug)]
struct TxResultInner {
    data: String,
    log: String,
    gas_wanted: String,
    gas_used: String,
    events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    attributes: Vec<Attribute>,
    #[serde(rename = "type")]
    event_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Attribute {
    index: bool,
    key: String,
    value: String,
}


pub async fn init_ws(url: &String, relayer_deposit_address: &String, pg_pool: Arc<PgPool>) {
    let url = Url::parse(url).expect("Invalid URL");

    // Define multiple subscription messages
    let subscriptions = HashMap::from([
        ("1".to_string(), crate::ws::Subscription {
            message: Message::Text(
                json!({
                    "jsonrpc": "2.0",
                    "method": "subscribe",
                    "id": "1",
                    "params": {
                        "query": format!("Switcheo.carbon.bridge.WithdrawTokenAcknowledgedEvent.relayer_deposit_address CONTAINS '{}'", relayer_deposit_address)
                    }
                }).to_string(),
            ),
            handler: Arc::new(Mutex::new(move |msg: String| {
                let pool = pg_pool.clone();
                // Spawn an async task to handle the message
                tokio::spawn(async move {
                    process_message(msg, pool).await;
                });
            })),
        }),
    ]);

    let client = crate::ws::JSONWebSocketClient::new(url, subscriptions);
    if let Err(e) = client.connect().await {
        println!("Error: {:?}", e);
    }
}

// process_message processes the message
async fn process_message(msg: String, pg_pool: Arc<PgPool>) {
    // Process the message and interact with the database
    // Attempt to deserialize the string into WebSocketMessage
    match serde_json::from_str::<WebSocketMessage>(&msg) {
        Ok(query_response) => {
            println!("Parsed query_response: {:?}", query_response);
            // look for Switcheo.carbon.bridge.WithdrawTokenAcknowledgedEvent
            if let Some(event) = query_response.result.data.value.tx_result.result.events.iter().find(|e| e.event_type == "Switcheo.carbon.bridge.WithdrawTokenAcknowledgedEvent") {
                let coin = event.attributes.iter().find(|a| a.key == "coin").map(|a| a.value.clone()).unwrap_or_default();
                let coin = serde_json::from_str::<serde_json::Value>(&coin).unwrap_or_default();
                let connection_id = event.attributes.iter().find(|a| a.key == "connection_id").map(|a| a.value.clone()).unwrap_or_default();
                let receiver = event.attributes.iter().find(|a| a.key == "receiver").map(|a| a.value.clone()).unwrap_or_default();
                let relay_fee = event.attributes.iter().find(|a| a.key == "relay_fee").map(|a| a.value.clone()).unwrap_or_default();
                let relay_fee = serde_json::from_str::<serde_json::Value>(&relay_fee).unwrap_or_default();
                let relayer_deposit_address = event.attributes.iter().find(|a| a.key == "relayer_deposit_address").map(|a| a.value.clone()).unwrap_or_default();
                let sender = event.attributes.iter().find(|a| a.key == "sender").map(|a| a.value.clone()).unwrap_or_default();
                let payload = event.attributes.iter().find(|a| a.key == "payload").map(|a| a.value.clone()).unwrap_or_default();

                // get payload_hash
                let payload_bytes = decode(strip_quotes(&payload.clone()))
                    .expect("Decoding failed");
                let payload_hash = keccak256(&payload_bytes);
                let payload_hash = encode(payload_hash);

                // save event details to db
                let result = sqlx::query!(
                        "INSERT INTO withdraw_token_acknowledged_events (coin, connection_id, receiver, relay_fee, relayer_deposit_address, sender, payload_hash, payload) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                        coin,
                        strip_quotes(&connection_id),
                        strip_quotes(&receiver),
                        relay_fee,
                        strip_quotes(&relayer_deposit_address),
                        strip_quotes(&sender),
                        payload_hash,
                        strip_quotes(&payload),
                    )
                    .execute(&*pg_pool)
                    .await;

                if let Err(e) = result {
                    println!("Failed to insert event data: {}", e);
                }
            } else {
                println!("Could not find Switcheo.carbon.bridge.WithdrawTokenAcknowledgedEvent event from response");
            }
        }
        Err(e) => {
            println!("Error parsing JSON: {:?}, JSON str:{:?}", e, msg);
        }
    }
}

fn strip_quotes(input: &str) -> String {
    input.trim_matches('"').to_string()
}