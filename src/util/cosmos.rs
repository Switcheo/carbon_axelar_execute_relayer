use anyhow::Context;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct WebSocketMessage {
    pub id: String,
    pub jsonrpc: String,
    pub result: WsResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WsResult {
    pub query: String,
    pub data: Data,
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

// Extracts relevant events from a JSON message
pub fn extract_events(msg: &str, event_name: &str) -> anyhow::Result<Vec<Event>> {
    let query_response = serde_json::from_str::<WebSocketMessage>(msg)
        .with_context(|| format!("Failed to parse JSON, provided string was: {}", msg))?;

    let events = query_response.result.data.value.tx_result.result.events
        .into_iter()
        .filter(|e| e.event_type == event_name)
        .collect();
    Ok(events)
}

async fn send_transaction(tx_bytes: Vec<u8>, node_url: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    // Convert tx_bytes to base64
    let tx_base64 = base64::encode(tx_bytes);

    // Prepare the JSON payload
    let payload = json!({
        "tx_bytes": tx_base64,
        "mode": "BROADCAST_MODE_BLOCK" // Or use BROADCAST_MODE_SYNC / BROADCAST_MODE_ASYNC
    });

    // Send the transaction
    let response = client
        .post(format!("{}/cosmos/tx/v1beta1/txs", node_url))
        .json(&payload)
        .send()
        .await?;

    // Handle the response
    let response_text = response.text().await?;
    println!("Response: {}", response_text);

    Ok(())
}
