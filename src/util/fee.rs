use anyhow::{Context, Result};
use ethers::prelude::U256;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use crate::conf::{Fee};
use crate::db::{DbPendingActionEvent, PendingActionType};

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
    let connection_id = &pending_action.connection_id;
    info!("relay_details from Carbon {:?}", relay_details);
    info!("fee: {:?}", relay_details.fee);
    // TODO: process relay fee and see if fee makes sense
    let fee = get_hydrogen_fee(&fee_config, connection_id, &relay_details.fee.denom).await;
    match fee {
        Ok(fee) => {
            info!("hydrogen fee: {:?}", fee);
            // Get the correct hydrogen fee based on pending_action_type
            let pending_action_type = pending_action.get_pending_action_type();
            let hydrogen_fee_value = match pending_action_type {
                PendingActionType::PendingRegisterTokenType => fee.register_token,
                PendingActionType::PendingDeregisterTokenType => fee.deregister_token,
                PendingActionType::PendingDeployNativeTokenType => fee.deploy_native_token,
                // PendingActionType::PendingWithdrawAndExecuteType => fee.withdraw_and_execute_xxx,
                PendingActionType::PendingWithdrawType => fee.withdraw,
                // PendingActionType::PendingExecuteType => fee.execute_xxx,
                _ => {
                    error!("Unknown action type: {:?}", pending_action_type);
                    return false;
                }
            };
            let hydrogen_fee = U256::from_dec_str(&hydrogen_fee_value).unwrap();

            // Get the relay's fee
            let relay_fee = U256::from(relay_details.fee.amount);

            // Calculate the acceptable fee range based on tolerance percentage
            let tolerance = U256::from_dec_str(&((fee_config.fee_tolerance_percentage * 100.0) as u64).to_string()).unwrap();
            let min_acceptable_fee = hydrogen_fee * (U256::from(10000) - tolerance) / U256::from(10000);

            if relay_fee >= min_acceptable_fee {
                info!("Sufficient fee: {}", relay_fee);
                true
            } else {
                warn!("Insufficient fee: {}", relay_fee);
                false
            }
        },
        Err(_err) => false
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
