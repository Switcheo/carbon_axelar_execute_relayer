use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use num_traits::ToPrimitive;
use sqlx::PgPool;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::time::interval;
use tracing::{error, info, instrument};

use crate::carbon::broadcaster::BroadcastRequest;
use crate::conf::{Carbon, Fee};
use crate::db::carbon_events::{add_bridge_pending_action_event_retry_count, delete_bridge_pending_action_events, get_expired_pending_action_events};
use crate::db::DbPendingActionEvent;
use crate::fee::fee::has_enough_fees;
use crate::util::carbon::msg::{MsgPruneExpiredPendingActions, MsgStartRelay};
use crate::util::carbon::query::{get_pending_action, get_pending_action_nonces};

#[instrument(name = "retry_carbon", skip_all)]
pub async fn init_all(carbon_config: &Carbon, fee_config: &Fee, pg_pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>) {
    poll_for_pending_action_events(carbon_config, fee_config, pg_pool.clone(), carbon_broadcaster).await;
}

// Polls for new poll_for_pending_action_events saved in the DB that can be executed and enqueues them into the broadcast channel
#[instrument(name = "poll_for_pending_action_events", skip_all)]
async fn poll_for_pending_action_events(carbon_config: &Carbon, fee_config: &Fee, pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>) {
    info!("Watching for events to broadcast");
    let mut interval = interval(Duration::from_secs(60));
    let carbon_config = carbon_config.clone();
    let fee_config = fee_config.clone();
    loop {
        interval.tick().await;
        if let Err(e) = retry_pending_actions(&carbon_config, &fee_config, pool.clone(), carbon_broadcaster.clone()).await {
            error!("Failed to retry_pending_actions: {}", e);
        }
        if let Err(e) = expire_pending_actions(&carbon_config.clone(), pool.clone(), carbon_broadcaster.clone()).await {
            error!("Failed to expire_pending_actions: {}", e);
        }
    }
}

// Checks the DB for events that can be executed and enqueues them into the broadcast channel
async fn retry_pending_actions(carbon_config: &Carbon, fee_config: &Fee, pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>) -> Result<()> {
    // check for new events that are not expired
    info!("Checking for pending_action_events to broadcast...");
    let events: Vec<DbPendingActionEvent> = sqlx::query_as!(
        DbPendingActionEvent,
        "SELECT * FROM pending_action_events WHERE retry_count < $1 AND (relay_details ->> 'expiry_block_time')::timestamp > NOW()",
        carbon_config.maximum_start_relay_retry_count
    )
        .fetch_all(&*pool)
        .await?;

    if events.is_empty() {
       info!("No pending_action_events that need to be started found in the DB");
       return Ok(())
    }

    for pending_action_event in events {
        info!("DB pending_action_event found: {:?}", pending_action_event);
        if has_enough_fees(&fee_config, pending_action_event.clone()).await {
            queue_start_relay(&carbon_config, pool.clone(), carbon_broadcaster.clone(), pending_action_event.nonce).await;
        }
    }
    Ok(())
}

// queue the startRelay process to broadcaster for carbon which will release fees to relayer address
pub async fn queue_start_relay(carbon_config: &Carbon, pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>, nonce: i64) {
    info!("Starting relay on {:?} for nonce {:?}", &carbon_config.rpc_url, nonce);

    // Check carbon if we still need to start this relay
    if !is_awaiting_relay(carbon_config, nonce).await {
        info!("Nonce {:?} is no longer pending and no longer needs to be started", nonce);
    }

    // Create a oneshot channel for the response
    let (callback_tx, callback_rx) = oneshot::channel();

    // Create MsgStartRelay
    let msg_start_relay = MsgStartRelay {
        relayer: carbon_config.relayer_address.clone(),
        nonce: nonce.to_u64().unwrap(),
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
                }
                Err(e) => {
                    eprintln!("Failed to broadcast message: {:?}", e);
                }
            }
            // Update retry count + 1
            add_bridge_pending_action_event_retry_count(pool.clone(), nonce).await.expect("failed");
        }
        Err(e) => {
            eprintln!("Failed to receive callback response: {:?}", e);
            // Update retry count + 1
            add_bridge_pending_action_event_retry_count(pool.clone(), nonce).await.expect("failed");
        }
    }
}

async fn is_awaiting_relay(carbon_config: &Carbon, nonce: i64) -> bool {
    let action = get_pending_action(&carbon_config.rest_url, nonce).await;
    match action {
        Ok(pending_action_event) => {
            let relay_details = pending_action_event.get_relay_details();
            let is_expired = relay_details.has_expired();
            let is_sent = relay_details.is_sent();
            !(is_expired || is_sent)
        }
        Err(err) => {
            error!("Error checking on action carbon: {:?}", err);
            false
        }
    }
}

// Checks the DB for events that can be expired and enqueues them into the broadcast channel
async fn expire_pending_actions(carbon_config: &Carbon, pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>) -> Result<()> {
    // Check for new events
    info!("Checking for expired pending_action_events in the DB...");
    let expired_events = get_expired_pending_action_events(pool.clone()).await?;
    let expired_nonces: Vec<i64> = expired_events
        .into_iter()
        .map(|event| event.nonce)
        .collect();

    // Early return if empty
    if expired_nonces.is_empty() {
        info!("No expired pending_action_events found in the DB");
        return Ok(());
    }

    // Prune nonces that are no longer in the "pending" group
    let nonces_to_process = prune_processed_nonces(&carbon_config.rest_url, pool.clone(), &expired_nonces).await?;

    if !nonces_to_process.is_empty() {
        queue_expire_relay(carbon_config, carbon_broadcaster, nonces_to_process).await;
    }
    Ok(())
}

pub async fn prune_processed_nonces(rest_url: &str, pool: Arc<PgPool>, expired_nonces: &[i64]) -> Result<Vec<i64>> {
    // Fetch pending nonces from the API
    let pending_nonces = get_pending_action_nonces(rest_url).await.context("Failed to get pending nonces")?;

    // Find out the nonces we can delete
    let nonces_to_delete: Vec<i64> = expired_nonces
        .iter()
        .cloned()
        .filter(|nonce| !pending_nonces.contains(nonce))
        .collect();

    // Delete nonces that are no longer pending from the DB
    if !nonces_to_delete.is_empty() {
        delete_bridge_pending_action_events(pool.clone(), nonces_to_delete).await?;
    }

    // Return the nonces that are still pending
    let nonces_to_process: Vec<i64> = expired_nonces
        .iter()
        .cloned()
        .filter(|nonce| pending_nonces.contains(nonce))
        .collect();

    Ok(nonces_to_process)
}

// queue the expire_relay process to broadcaster for carbon which will expire a particular relay and refund fees to user
pub async fn queue_expire_relay(carbon_config: &Carbon, carbon_broadcaster: Sender<BroadcastRequest>, nonces: Vec<i64>) {
    info!("Expiring relay on {:?} for nonces {:?}", &carbon_config.rpc_url, &nonces);

    // Create a oneshot channel for the response
    let (callback_tx, callback_rx) = oneshot::channel();

    // Create msg
    let msg_expire_actions = MsgPruneExpiredPendingActions {
        creator: carbon_config.relayer_address.clone(),
        nonces: nonces.iter().map(|nonce| nonce.to_u64().unwrap() ).collect(),
    };

    // Create a BroadcastRequest with the message and callback
    let broadcast_request = BroadcastRequest {
        msg: Box::new(msg_expire_actions),
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
