use tracing::info;
use crate::conf::Carbon;
use crate::util::carbon_tx::send_msg_expire_relays;

pub async fn expire_pending_actions(carbon_config: &Carbon, nonces: Vec<u64>) {
    info!("Expiring relays on {:?} nonces: {:?} ", &carbon_config.rpc_url, nonces);
    let response = send_msg_expire_relays(&carbon_config, nonces).await;

    match response {
        Ok(value) => {
            info!("Received successful response: {:?}", value);
            // TODO: Update the database here
        }
        Err(e) => {
            eprintln!("Failed to broadcast message: {:?}", e);
            // Handle the error and possibly update the DB to reflect the failure
            // TODO: update db back to pending?
        }
    }
}