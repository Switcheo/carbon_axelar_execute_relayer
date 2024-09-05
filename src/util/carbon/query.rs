use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde_json::{from_value, Value};
use tracing::{debug};
use crate::db::{RelayDetails};

pub async fn get_pending_action_nonces(rest_url: &str) -> Result<Vec<i64>> {
    let client = Client::new();
    let url = format!("{}/carbon/bridge/v1/pending_action_nonce", rest_url);
    let resp: Value = client.get(&url).send().await?.json().await?;

    // Parse the nonces array
    let nonces = resp["pending_action_nonces"]
        .as_array()
        .ok_or(anyhow!("pending_action_nonces not found or is not an array"))?
        .iter()
        .map(|nonce| nonce.as_str().ok_or(anyhow!("nonce is not a string"))?.parse::<i64>().map_err(|e| anyhow!("Failed to parse nonce: {:?}", e)))
        .collect::<Result<Vec<i64>>>()?;

    debug!("found pending action nonces {:?}", nonces);
    Ok(nonces)
}

pub async fn get_pending_action_relay_details(rest_url: &str, nonce: i64) -> Result<RelayDetails> {
    let client = Client::new();
    let url = format!("{}/carbon/bridge/v1/pending_action/{}", rest_url, nonce);
    let resp: Value = client.get(&url).send().await?.json().await?;
    let action_str = resp["action"]
        .as_str()
        .context("Failed to get action as string")?;
    let action: Value = serde_json::from_str(action_str)
        .context("Failed to deserialize action")?;
    let relay_details: RelayDetails = from_value(action["relay_details"].clone()).expect("cannot parse relay_details");
    Ok(relay_details)
}