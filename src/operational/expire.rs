use cosmrs::tx::Msg;
use tracing::info;
use crate::conf::Carbon;
use crate::util::carbon::msg::MsgPruneExpiredPendingActions;
use crate::util::carbon::tx::{send_msg_via_tx};

pub async fn expire_pending_actions(carbon_config: &Carbon, nonces: Vec<u64>) {
    info!("Expiring relays on {:?} nonces: {:?} ", &carbon_config.rpc_url, nonces);
    let msg_expire_actions = MsgPruneExpiredPendingActions {
        creator: carbon_config.relayer_address.clone(),
        nonces,
    }
        .to_any()
        .unwrap();

    // send msg via a tx
    let response = send_msg_via_tx(carbon_config, msg_expire_actions).await;

    match response {
        Ok(value) => {
            info!("Received successful response: {:?}", value);
        }
        Err(e) => {
            eprintln!("Failed to broadcast message: {:?}", e);
        }
    }
}