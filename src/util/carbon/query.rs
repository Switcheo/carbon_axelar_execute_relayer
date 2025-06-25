use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde_json::{from_value, Value};
use tracing::{debug};
use crate::db::{DbPendingActionEvent, PendingActionType, RelayDetails};
use crate::util::carbon::parser::parse_connection_id;

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

pub async fn get_pending_action(rest_url: &str, nonce: i64) -> Result<DbPendingActionEvent> {
    let client = Client::new();
    let url = format!("{}/carbon/bridge/v1/pending_action/{}", rest_url, nonce);
    let resp: Value = client.get(&url).send().await?.json().await?;

    let action_str = resp["action"]
        .as_str()
        .context("Failed to get action as string")?;
    let action: Value = serde_json::from_str(action_str)
        .context("Failed to deserialize action")?;

    // extract connection_id and relay_details from action
    let connection_id = action["connection_id"]
        .as_str()
        .context("Missing connection_id")?
        .to_string();

    let relay_details = action["relay_details"].clone();

    // parse bridge_id + chain_id from connection_id
    let (bridge_id, chain_id, _) = parse_connection_id(&connection_id);

    Ok(DbPendingActionEvent {
        id: -1, // it's not from DB yet
        connection_id,
        bridge_id,
        chain_id,
        nonce,
        // TODO: pending_action_type is currently hardcoded in this function because the endpoint doesn't return the required information
        // TODO: uncomment below after fix to un-hardcode the action type: https://test-api.carbon.network/carbon/bridge/v1/pending_action/42
        // pending_action_type: PendingActionType::from_prefix(action["method"].as_str().unwrap_or_default())?.into(),
        pending_action_type: PendingActionType::PendingWithdrawType.into(),
        retry_count: 0,
        relay_details,
    })
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