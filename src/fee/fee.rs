use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug};

use crate::conf::{Fee, RelayStrategy};
use crate::db::{DbPendingActionEvent};
use crate::fee::strategies::{check_all_strategy, check_greater_than_0_strategy, check_hydrogen_strategy};

// carbon
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeeResponse {
    pub withdraw: String,
    pub register_token: String,
    pub deregister_token: String,
    pub deploy_native_token: String,
    pub quoted_at: String,
}

pub async fn has_enough_fees(fee_config: &Fee, pending_action: DbPendingActionEvent) -> bool {
    let relay_details = pending_action.get_relay_details();
    debug!("relay_details from Carbon {:?}", relay_details);
    match fee_config.relay_strategy {
        RelayStrategy::Hydrogen => check_hydrogen_strategy(fee_config, &relay_details, &pending_action).await,
        RelayStrategy::All => check_all_strategy(),
        RelayStrategy::GreaterThan0 => check_greater_than_0_strategy(&relay_details),
    }
}

pub async fn get_hydrogen_fee(fee_conf: &Fee, connection_id: &str, fee_denom: &str) -> Result<FeeResponse> {
    let client = Client::new();
    let url = format!("{}/bridge_fees?connection_id={}&fee_denom={}", fee_conf.hydrogen_url, connection_id, fee_denom);

    let resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to send request to hydrogen")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_else(|_| String::from("Failed to read response body"));
        return Err(anyhow::anyhow!("Request to hydrogen URL failed with status {}: {}", status, text));
    }

    let fee_response: FeeResponse = resp
        .json()
        .await
        .context("Failed to deserialize response from hydrogen")?;

    Ok(fee_response)
}
