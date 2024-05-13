use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result, Error};
use ethers::utils::hex::{decode, encode_prefixed};
use ethers::utils::keccak256;
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use tracing::{debug, error, info, instrument};
use url::Url;
use crate::conf::Carbon;
use crate::constants::events::{CARBON_BRIDGE_PENDING_ACTION_EVENT, CARBON_BRIDGE_ACKNOWLEDGE_EVENT, CARBON_BRIDGE_REVERT_EVENT, CARBON_PAYLOAD_SENT_EVENT};
use crate::db::carbon_events::{save_bridge_acknowledgement_event, save_bridge_pending_action_event, save_bridge_revert_event, save_payload_event};
use crate::db::PayloadType;
use crate::util::cosmos::{Event, WebSocketMessage};

use crate::ws::{JSONWebSocketClient};

#[instrument(name = "listener_carbon", skip_all)]
pub async fn init_ws(carbon_config: &Carbon, pg_pool: Arc<PgPool>) {
    info!("Initializing WS for Carbon. Watching {:?} on {:?} for events", &carbon_config.relayer_deposit_address, &carbon_config.ws_url);
    let url = Url::parse(&carbon_config.ws_url).expect(&format!("Invalid WS URL {:?}", &carbon_config.ws_url));

    // create new client
    let mut client = JSONWebSocketClient::new(url);

    // add WithdrawTokenConfirmedEvent subscription
    let pool = pg_pool.clone();
    client.add_cosmos_subscription(
        "1".to_string(),
        &format!("{} EXISTS", CARBON_BRIDGE_PENDING_ACTION_EVENT),
        Arc::new(Mutex::new(move |msg: String| {
            // Spawn an async task to handle the message
            let pool = pool.clone();
            tokio::spawn(async move {
                process_bridge_pending_action(msg, pool.clone()).await;
            });
        })));

    // add PayloadAcknowledgedEvent subscription
    let pool = pg_pool.clone();
    client.add_cosmos_subscription(
        "2".to_string(),
        &format!("{} EXISTS", CARBON_BRIDGE_ACKNOWLEDGE_EVENT),
        Arc::new(Mutex::new(move |msg: String| {
            let pool = pool.clone();
            // Spawn an async task to handle the message
            tokio::spawn(async move {
                process_bridge_acknowledgement_event(msg, pool.clone()).await;
            });
        })));

    // add PayloadAcknowledgedEvent subscription
    let pool = pg_pool.clone();
    client.add_cosmos_subscription(
        "3".to_string(),
        &format!("{} EXISTS", CARBON_BRIDGE_REVERT_EVENT),
        Arc::new(Mutex::new(move |msg: String| {
            let pool = pool.clone();
            // Spawn an async task to handle the message
            tokio::spawn(async move {
                process_bridge_revert_event(msg, pool.clone()).await;
            });
        })));

    // add PayloadAcknowledgedEvent subscription
    let pool = pg_pool.clone();
    let carbon_config = carbon_config.clone();
    client.add_cosmos_subscription(
        "4".to_string(),
        &format!("{} EXISTS", CARBON_PAYLOAD_SENT_EVENT),
        Arc::new(Mutex::new(move |msg: String| {
            let pool = pool.clone();
            let carbon_config = carbon_config.clone();
            // Spawn an async task to handle the message
            tokio::spawn(async move {
                process_payload_sent_event(msg, carbon_config.clone(), pool.clone()).await;
            });
        })));

    // connect to WS
    if let Err(e) = client.connect().await {
        error!("Error connecting to client: {:?}", e);
    }
}

// Extracts relevant events from a JSON message
fn extract_events(msg: &str, event_name: &str) -> Result<Vec<Event>> {
    let query_response = serde_json::from_str::<WebSocketMessage>(msg)
        .with_context(|| format!("Failed to parse JSON, provided string was: {}", msg))?;

    let events = query_response.result.data.value.tx_result.result.events
        .into_iter()
        .filter(|e| e.event_type == event_name)
        .collect();
    Ok(events)
}

// process_bridge_pending_action processes the BridgePendingActionEvent
#[instrument(skip_all)]
async fn process_bridge_pending_action(msg: String, pg_pool: Arc<PgPool>) {
    info!("Processing new BridgePendingActionEvent from Carbon");
    let events = extract_events(&msg, CARBON_BRIDGE_PENDING_ACTION_EVENT).unwrap();
    for event in events {
        // TODO: process relay fee and see if fee makes sense
        let relay_fee = event.attributes.iter().find(|a| a.key == "relay_fee").map(|a| a.value.clone()).unwrap_or_default();
        let relay_fee = serde_json::from_str::<serde_json::Value>(&relay_fee).unwrap_or_default();
        info!("relay_fee from Carbon {:?}", relay_fee);
        save_bridge_pending_action_event(pg_pool.clone(), &event.clone()).await
    }
}

// process_bridge_acknowledgement_event processes the BridgeAcknowledgeEvent
#[instrument(skip_all)]
async fn process_bridge_acknowledgement_event(msg: String, pg_pool: Arc<PgPool>) {
    info!("Processing new PayloadSentEvent from Carbon");
    let events = extract_events(&msg, CARBON_BRIDGE_ACKNOWLEDGE_EVENT).unwrap();
    for event in events {
        save_bridge_acknowledgement_event(pg_pool.clone(), &event.clone()).await
    }
}

// process_bridge_revert_event processes the BridgeRevertEvent
#[instrument(skip_all)]
async fn process_bridge_revert_event(msg: String, pg_pool: Arc<PgPool>) {
    info!("Processing new PayloadSentEvent from Carbon");
    let events = extract_events(&msg, CARBON_BRIDGE_REVERT_EVENT).unwrap();
    for event in events {
        save_bridge_revert_event(pg_pool.clone(), &event.clone()).await
    }
}

// process_payload_sent_event processes the PayloadSentEvent
#[instrument(skip_all)]
async fn process_payload_sent_event(msg: String, carbon_config: Carbon, pg_pool: Arc<PgPool>) {
    info!("Processing new PayloadSentEvent from Carbon");
    let events = extract_events(&msg, CARBON_PAYLOAD_SENT_EVENT).unwrap();
    for event in events {
        save_payload_event(&carbon_config, pg_pool.clone(), &event.clone()).await
    }
}
