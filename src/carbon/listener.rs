use std::sync::Arc;
use futures::lock::Mutex;
use sqlx::PgPool;
use num_traits::ToPrimitive;
use tokio::sync::mpsc::Sender;
use tracing::{error, info, instrument};
use url::Url;
use crate::carbon::broadcaster::BroadcastRequest;
use crate::carbon::retry::queue_start_relay;

use crate::conf::{Carbon, Fee};
use crate::constants::events::{CARBON_AXELAR_CALL_CONTRACT_EVENT, CARBON_BRIDGE_EXPIRED_PENDING_ACTION_EVENT, CARBON_BRIDGE_PENDING_ACTION_EVENT, CARBON_BRIDGE_REVERT_EVENT};
use crate::db::carbon_events::{delete_bridge_pending_action_event, save_axelar_call_contract_event, save_bridge_pending_action_event};
use crate::util::carbon::parser::{parse_axelar_call_contract_event, parse_expired_pending_action_event, parse_bridge_pending_action_event, parse_bridge_reverted_event};
use crate::util::cosmos::{extract_events};
use crate::fee::fee::{has_enough_fees};
use crate::ws::JSONWebSocketClient;

#[instrument(name = "listener_carbon", skip_all)]
pub async fn init_ws(carbon_config: &Carbon, fee_config: &Fee, pg_pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>) {
    info!("Initializing WS for Carbon. Watching {:?} on {:?} for events", &carbon_config.relayer_address, &carbon_config.ws_url);
    let url = Url::parse(&carbon_config.ws_url).expect(&format!("Invalid WS URL {:?}", &carbon_config.ws_url));

    // create new client
    let mut client = JSONWebSocketClient::new(url);

    // add WithdrawTokenConfirmedEvent subscription
    let pool = pg_pool.clone();
    let carbon_config = carbon_config.clone();
    let fee_config = fee_config.clone();
    let carbon_broadcaster = carbon_broadcaster.clone();
    client.add_cosmos_subscription(
        "1".to_string(),
        &format!("{}.connection_id CONTAINS '{}/'", CARBON_BRIDGE_PENDING_ACTION_EVENT, &carbon_config.axelar_bridge_id),
        Arc::new(Mutex::new(move |msg: String| {
            // Spawn an async task to handle the message
            let pool = pool.clone();
            let carbon_config = carbon_config.clone();
            let fee_config = fee_config.clone();
            let carbon_broadcaster = carbon_broadcaster.clone();
            tokio::spawn(async move {
                process_bridge_pending_action(&carbon_config, &fee_config, msg, pool.clone(), carbon_broadcaster.clone()).await;
            });
        })));

    // add CARBON_BRIDGE_EXPIRED_PENDING_ACTION_EVENT subscription
    let pool = pg_pool.clone();
    client.add_cosmos_subscription(
        "2".to_string(),
        &format!("{} EXISTS", CARBON_BRIDGE_EXPIRED_PENDING_ACTION_EVENT),
        Arc::new(Mutex::new(move |msg: String| {
            let pool = pool.clone();
            // Spawn an async task to handle the message
            tokio::spawn(async move {
                process_expired_pending_action_event(msg, pool.clone()).await;
            });
        })));

    // add BridgeRevertEvent subscription
    let pool = pg_pool.clone();
    client.add_cosmos_subscription(
        "3".to_string(),
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
        "4".to_string(),
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
async fn process_bridge_pending_action(carbon_config: &Carbon, fee_config: &Fee, msg: String, pg_pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>) {
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
        if has_enough_fees(fee_config, pending_action.clone()).await {
            queue_start_relay(carbon_config, pg_pool.clone(), carbon_broadcaster.clone(), pending_action.nonce).await;
        }
    }
}

// process_bridge_revert_event processes the BridgeRevertedEvent
#[instrument(skip_all)]
async fn process_expired_pending_action_event(msg: String, pg_pool: Arc<PgPool>) {
    info!("Processing new BridgeRevertedEvent from Carbon");
    let events = extract_events(&msg, CARBON_BRIDGE_EXPIRED_PENDING_ACTION_EVENT).unwrap();
    for event in events {
        let expired_pending_action_event = parse_expired_pending_action_event(event);
        delete_bridge_pending_action_event(pg_pool.clone(), expired_pending_action_event.nonce).await
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
