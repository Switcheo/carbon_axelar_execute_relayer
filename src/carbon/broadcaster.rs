use anyhow::Result;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::Sender;
use tracing::instrument;

use crate::conf::Carbon;
use crate::util::carbon::msg::IntoAny;
use crate::util::carbon::tx::send_msg_via_tx;

pub struct BroadcastRequest {
    pub msg: Box<dyn IntoAny>,
    pub callback: oneshot::Sender<Result<serde_json::Value>>,
}

#[instrument(name = "broadcaster_carbon", skip_all)]
pub async fn init_all(carbon_config: &Carbon) -> Sender<BroadcastRequest> {
    // initialize broadcast channel
    let channel_tx = init_channel(carbon_config).await;

    // return the channel sender for other processes to send their tx
    channel_tx
}

pub async fn init_channel(carbon_config: &Carbon) -> Sender<BroadcastRequest> {
    let (broadcast_tx, mut broadcast_rx) = mpsc::channel::<BroadcastRequest>(100);
    let carbon_config = carbon_config.clone();
    tokio::spawn(async move {
        while let Some(request) = broadcast_rx.recv().await {
            // Simulate broadcasting msg
            println!("Broadcasting: {:?}", request.msg);

            let response = send_msg_via_tx(&carbon_config, request.msg.into_any()).await;

            // Send the response via the callback channel
            let _ = request.callback.send(response);
        }
    });
    broadcast_tx
}