use anyhow::{anyhow,Result};
use reqwest::Client;
use serde_json::Value;
use tracing::info;

pub async fn get_pending_action_nonces(rest_url: &str) -> Result<Vec<u64>> {
    let client = Client::new();
    let url = format!("{}/carbon/bridge/v1/pending_action_nonce", rest_url);
    let resp: Value = client.get(&url).send().await?.json().await?;

    // Parse the nonces array
    let nonces = resp["pending_action_nonces"]
        .as_array()
        .ok_or(anyhow!("pending_action_nonces not found or is not an array"))?
        .iter()
        .map(|nonce| nonce.as_str().ok_or(anyhow!("nonce is not a string"))?.parse::<u64>().map_err(|e| anyhow!("Failed to parse nonce: {:?}", e)))
        .collect::<Result<Vec<u64>>>()?;

    info!("found pending action nonces {:?}", nonces);
    Ok(nonces)
}