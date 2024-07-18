use cosmrs::tx::Msg;
use tracing::info;
use crate::conf::Carbon;
use crate::util::carbon::msg::MsgStartRelay;
use crate::util::carbon::tx::{send_msg_via_tx};

pub async fn start_relay(carbon_config: &Carbon, nonce: u64) {
    info!("Starting relay on {:?} for nonce: {:?} ", &carbon_config.rpc_url, nonce);
    let msg_start_relay = MsgStartRelay {
        relayer: carbon_config.relayer_address.clone(),
        nonce,
    }
        .to_any()
        .unwrap();

    // send msg via a tx
    let response = send_msg_via_tx(carbon_config, msg_start_relay).await;

    match response {
        Ok(value) => {
            info!("Received successful response: {:?}", value);
        }
        Err(e) => {
            eprintln!("Failed to broadcast message: {:?}", e);
            // Handle the error and possibly update the DB to reflect the failure
        }
    }
}