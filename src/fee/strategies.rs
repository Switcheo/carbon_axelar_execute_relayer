use ethers::prelude::U256;
use tracing::{error, info};
use tracing::log::warn;
use crate::conf::Fee;
use crate::db::{DbPendingActionEvent, PendingActionType, RelayDetails};
use crate::fee::fee::get_hydrogen_fee;

pub async fn check_hydrogen_strategy(fee_config: &Fee, relay_details: &RelayDetails, pending_action: &DbPendingActionEvent) -> bool {
    let connection_id = &pending_action.connection_id;
    let fee = get_hydrogen_fee(fee_config, connection_id, &relay_details.fee.denom).await;
    match fee {
        Ok(fee) => {
            info!("hydrogen fee: {:?}", fee);
            // Get the correct hydrogen fee based on pending_action_type
            let pending_action_type = pending_action.get_pending_action_type();
            let hydrogen_fee_value = match pending_action_type {
                PendingActionType::PendingRegisterTokenType => fee.register_token,
                PendingActionType::PendingDeregisterTokenType => fee.deregister_token,
                // TODO: support other types of withdraw_and_execute
                PendingActionType::PendingWithdrawAndExecuteType => fee.withdraw_and_execute,
                PendingActionType::PendingWithdrawType => fee.withdraw,
                // PendingActionType::PendingExecuteType => fee.execute_xxx,
                _ => {
                    error!("Unknown/unsupported action type: {:?}", pending_action_type);
                    return false;
                }
            };

            // drop the decimals by splitting at .
            let hydrogen_fee_value_without_decimals = hydrogen_fee_value.split_once('.')
                .map(|(before_decimal, _)| before_decimal)
                .unwrap_or(&hydrogen_fee_value);
            let hydrogen_fee = U256::from_dec_str(hydrogen_fee_value_without_decimals).unwrap();

            // Get the relay's fee
            let relay_fee = U256::from(relay_details.fee.amount);

            // Calculate the acceptable fee range based on tolerance percentage
            let tolerance = U256::from_dec_str(&((fee_config.fee_tolerance_percentage * 100.0) as u64).to_string()).unwrap();
            let min_acceptable_fee = hydrogen_fee * (U256::from(10000) - tolerance) / U256::from(10000);

            if relay_fee >= min_acceptable_fee {
                info!("Sufficient fee: {}", relay_fee);
                true
            } else {
                tracing::warn!("Insufficient fee: {}, min_acceptable_fee: {}", relay_fee, min_acceptable_fee);
                false
            }
        },
        Err(e) => {
            error!("Error fetching hydrogen fee: {:?}", e);
            false
        }
    }
}

pub fn check_all_strategy() -> bool {
    info!("Using 'all' strategy, assuming sufficient fee");
    true
}

pub fn check_greater_than_0_strategy(relay_details: &RelayDetails) -> bool {
    let relay_fee = U256::from(relay_details.fee.amount);
    if relay_fee > U256::from(0) {
        info!("Using 'greater_than_0' strategy, sufficient fee: {}", relay_fee);
        true
    } else {
        warn!("Insufficient fee: {}", relay_fee);
        false
    }
}