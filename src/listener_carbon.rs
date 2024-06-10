use std::sync::Arc;
use futures::lock::Mutex;
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use num_traits::ToPrimitive;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tracing::{error, info, instrument};
use url::Url;
use crate::broadcaster_carbon::BroadcastRequest;

use crate::conf::Carbon;
use crate::constants::events::{CARBON_AXELAR_CALL_CONTRACT_EVENT, CARBON_BRIDGE_PENDING_ACTION_EVENT, CARBON_BRIDGE_REVERT_EVENT};
use crate::db::carbon_events::{delete_bridge_pending_action_event, save_axelar_call_contract_event, save_bridge_pending_action_event};
use crate::util::carbon::{parse_axelar_call_contract_event, parse_bridge_pending_action_event, parse_bridge_reverted_event};
use crate::util::carbon_msg::MsgStartRelay;
use crate::util::cosmos::{extract_events};
use crate::util::fee::{should_relay};
use crate::ws::JSONWebSocketClient;

#[instrument(name = "listener_carbon", skip_all)]
pub async fn init_ws(carbon_config: &Carbon, pg_pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>) {
    info!("Initializing WS for Carbon. Watching {:?} on {:?} for events", &carbon_config.relayer_address, &carbon_config.ws_url);
    let url = Url::parse(&carbon_config.ws_url).expect(&format!("Invalid WS URL {:?}", &carbon_config.ws_url));

    // create new client
    let mut client = JSONWebSocketClient::new(url);

    // add WithdrawTokenConfirmedEvent subscription
    let pool = pg_pool.clone();
    let carbon_config = carbon_config.clone();
    let carbon_broadcaster = carbon_broadcaster.clone();
    client.add_cosmos_subscription(
        "1".to_string(),
        &format!("{}.connection_id CONTAINS '{}/'", CARBON_BRIDGE_PENDING_ACTION_EVENT, &carbon_config.axelar_bridge_id),
        Arc::new(Mutex::new(move |msg: String| {
            // Spawn an async task to handle the message
            let pool = pool.clone();
            let carbon_config = carbon_config.clone();
            let carbon_broadcaster = carbon_broadcaster.clone();
            tokio::spawn(async move {
                process_bridge_pending_action(&carbon_config, msg, pool.clone(), carbon_broadcaster.clone()).await;
            });
        })));

    // add BridgeRevertEvent subscription
    let pool = pg_pool.clone();
    client.add_cosmos_subscription(
        "2".to_string(),
        &format!("{} EXISTS", CARBON_BRIDGE_REVERT_EVENT),
        Arc::new(Mutex::new(move |msg: String| {
            let pool = pool.clone();
            // Spawn an async task to handle the message
            tokio::spawn(async move {
                process_bridge_reverted_event(msg, pool.clone()).await;
            });
        })));

    // add AxelarCallContractEvent subscription
    let pool = pg_pool.clone();
    client.add_cosmos_subscription(
        "3".to_string(),
        &format!("{} EXISTS", CARBON_AXELAR_CALL_CONTRACT_EVENT),
        Arc::new(Mutex::new(move |msg: String| {
            let pool = pool.clone();
            // Spawn an async task to handle the message
            tokio::spawn(async move {
                process_axelar_call_contract_event(msg, pool.clone()).await;
            });
        })));

    // connect to WS
    if let Err(e) = client.connect().await {
        error!("Error connecting to client: {:?}", e);
    }
}



// process_bridge_pending_action processes the PendingActionEvent
#[instrument(skip_all)]
async fn process_bridge_pending_action(carbon_config: &Carbon, msg: String, pg_pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>) {
    info!("Processing new PendingActionEvent from Carbon");
    let events = extract_events(&msg, CARBON_BRIDGE_PENDING_ACTION_EVENT).unwrap();
    for event in events {
        let pending_action = parse_bridge_pending_action_event(event);

        // check if event has expired
        if pending_action.get_relay_details().has_expired() {
            info!("Skipping event with nonce {:?} as it has expired", pending_action.nonce.to_u64());
            continue
        }

        // save to DB
        save_bridge_pending_action_event(pg_pool.clone(), &pending_action.clone()).await;

        // start the relay
        // TODO: separate thread?
        if should_relay(carbon_config, pending_action.get_relay_details()) {
            start_relay(carbon_config, carbon_broadcaster.clone(), pending_action.nonce).await;
        }
    }
}

// starts the relay process on carbon which will release fees to relayer address
pub async fn start_relay(carbon_config: &Carbon, carbon_broadcaster: Sender<BroadcastRequest>, nonce: BigDecimal) {
    info!("Starting relay on {:?} for nonce {:?}", &carbon_config.rpc_url, &nonce);
    // Convert nonce to u64
    let nonce = nonce.to_u64().expect("could not convert nonce to u64");

    // Create a oneshot channel for the response
    let (callback_tx, callback_rx) = oneshot::channel();

    // Create MsgStartRelay
    let msg_start_relay = MsgStartRelay {
        relayer: carbon_config.relayer_address.clone(),
        nonce,
    };

    // Create a BroadcastRequest with the message and callback
    let broadcast_request = BroadcastRequest {
        msg: Box::new(msg_start_relay),
        callback: callback_tx,
    };

    // Send the BroadcastRequest through the carbon_broadcaster channel
    if let Err(e) = carbon_broadcaster.send(broadcast_request).await {
        eprintln!("Failed to send broadcast request: {:?}", e);
        return;
    }

    // Await the response
    match callback_rx.await {
        Ok(response) => {
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
        Err(e) => {
            eprintln!("Failed to receive callback response: {:?}", e);
            // Handle the error and possibly update the DB to reflect the failure
        }
    }
}

// process_bridge_revert_event processes the BridgeRevertedEvent
#[instrument(skip_all)]
async fn process_bridge_reverted_event(msg: String, pg_pool: Arc<PgPool>) {
    info!("Processing new BridgeRevertedEvent from Carbon");
    let events = extract_events(&msg, CARBON_BRIDGE_REVERT_EVENT).unwrap();
    for event in events {
        let bridge_reverted_event = parse_bridge_reverted_event(event);
        delete_bridge_pending_action_event(pg_pool.clone(), bridge_reverted_event.nonce).await
    }
}

// process_axelar_call_contract_event processes the AxelarCallContractEvent
#[instrument(skip_all)]
async fn process_axelar_call_contract_event(msg: String, pg_pool: Arc<PgPool>) {
    info!("Processing new AxelarCallContractEvent from Carbon");
    let events = extract_events(&msg, CARBON_AXELAR_CALL_CONTRACT_EVENT).unwrap();
    for event in events {
        let axelar_call_contract_event = parse_axelar_call_contract_event(event);
        save_axelar_call_contract_event(pg_pool.clone(), &axelar_call_contract_event.clone()).await
    }
}
