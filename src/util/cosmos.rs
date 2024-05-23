use anyhow::{anyhow, Context, Result};
use base64::Engine;
use base64::engine::general_purpose;
use cosmrs::tx::{AuthInfo, Body};
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


pub async fn get_account_info(rest_url: &str, account_address: &str) -> Result<(u64, u64)> {
    let client = Client::new();
    let url = format!("{}/cosmos/auth/v1beta1/accounts/{}", rest_url, account_address);
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    let account_number = resp["account"]["base_account"]["account_number"]
        .as_str()
        .ok_or(anyhow!("account_number not found"))?
        .parse::<u64>()?;
    let sequence = resp["account"]["base_account"]["sequence"]
        .as_str()
        .ok_or(anyhow!("sequence not found"))?
        .parse::<u64>()?;
    Ok((account_number, sequence))
}

pub async fn estimate_gas(rest_url: &str, tx_body: &Body, auth_info: &AuthInfo) -> Result<u64> {
    let client = Client::new();

    // Convert tx_bytes to base64
    let tx_bytes = tx_body.clone().into_bytes().expect("unable to parse tx_body into bytes");
    let tx_base64 = general_purpose::STANDARD.encode(&tx_bytes);
    // Convert auth_info to base64
    let auth_info_bytes = auth_info.clone().into_bytes().expect("unable to parse auth_info into bytes");
    let auth_info_base64 = general_purpose::STANDARD.encode(&auth_info_bytes);

    let simulate_body = json!({
        "tx": {
            "body": tx_base64,
            "auth_info": auth_info_base64,
            "signatures": []
        }
    });

    let url = format!("{}/cosmos/tx/v1beta1/simulate", rest_url);
    let resp: serde_json::Value = client.post(&url).json(&simulate_body).send().await?.json().await?;
    let gas_estimate = resp["gas_info"]["gas_used"]
        .as_str()
        .ok_or_else(|| anyhow!("gas_used not found"))?
        .parse::<u64>()?;

    Ok(gas_estimate)
}

pub async fn get_latest_block_height(rpc_url: &str) -> Result<u16> {
    let client = Client::new();
    let url = format!("{}/blocks/latest", rpc_url);
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    let block_height = resp["block"]["header"]["height"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("height not found"))?
        .parse::<u16>()?;

    Ok(block_height)
}
pub async fn send_transaction(rest_url: &str, tx_bytes: Vec<u8>) -> Result<()> {
    let client = Client::new();

    // Convert tx_bytes to base64
    let tx_base64 = general_purpose::STANDARD.encode(&tx_bytes);

    // Prepare the JSON payload
    let payload = json!({
        "tx_bytes": tx_base64,
        "mode": "BROADCAST_MODE_BLOCK" // Or use BROADCAST_MODE_SYNC / BROADCAST_MODE_ASYNC
    });

    // Send the transaction
    let response = client
        .post(format!("{}/cosmos/tx/v1beta1/txs", rest_url))
        .json(&payload)
        .send()
        .await?;

    // Handle the response
    let response_text = response.text().await?;
    println!("Response: {}", response_text);

    Ok(())
}

