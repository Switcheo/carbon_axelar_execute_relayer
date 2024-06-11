use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use num_traits::ToPrimitive;
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use tokio::sync::{oneshot};
use tokio::sync::mpsc::Sender;
use tokio::time::interval;
use tracing::{debug, error, info, instrument};
use crate::carbon::broadcaster::BroadcastRequest;

use crate::conf::Carbon;
use crate::db::DbPendingActionEvent;
use crate::util::carbon_msg::{MsgPruneExpiredPendingActions, MsgStartRelay};
use crate::util::fee::should_relay;

#[instrument(name = "retry_carbon", skip_all)]
pub async fn init_all(carbon_config: &Carbon, pg_pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>) {
    poll_for_pending_action_events(carbon_config.clone(), pg_pool.clone(), carbon_broadcaster).await;
}

// Polls for new poll_for_pending_action_events saved in the DB that can be executed and enqueues them into the broadcast channel
#[instrument(name = "poll_for_pending_action_events", skip_all)]
async fn poll_for_pending_action_events(carbon_config: Carbon, pool: Arc<PgPool>, carbon_broadcaster: Sender<BroadcastRequest>) {
    info!("Watching for events to broadcast");
    let mut interval = interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        if let Err(e) = retry_pending_actions(&carbon_config.clone(), &pool, carbon_broadcaster.clone()).await {
            error!("Failed to queue new events for broadcast: {}", e);
        }
        if let Err(e) = expire_pending_actions(&carbon_config.clone(), &pool, carbon_broadcaster.clone()).await {
            error!("Failed to queue new events for broadcast: {}", e);
        }
    }
}

// Checks the DB for events that can be executed and enqueues them into the broadcast channel
async fn retry_pending_actions(carbon_config: &Carbon, pool: &PgPool, carbon_broadcaster: Sender<BroadcastRequest>) -> Result<()> {
    // check for new events
    debug!("Checking for pending_action_events to broadcast...");
    let events: Vec<DbPendingActionEvent> = sqlx::query_as!(
        DbPendingActionEvent,
        "SELECT * FROM pending_action_events WHERE broadcast_status = $1",
        "pending_broadcast"
    )
        .fetch_all(pool)
        .await?;

    for pending_action_event in events {
        info!("DB pending_action_event found: {:?}", pending_action_event);
        let relay_details = pending_action_event.get_relay_details();
        if relay_details.has_expired() {

            continue
        }
        if should_relay(&carbon_config, relay_details) {
            queue_start_relay(&carbon_config, carbon_broadcaster.clone(), pending_action_event.nonce).await;
        }
    }
    Ok(())
}

// queue the startRelay process to broadcaster for carbon which will release fees to relayer address
pub async fn queue_start_relay(carbon_config: &Carbon, carbon_broadcaster: Sender<BroadcastRequest>, nonce: BigDecimal) {
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

// Checks the DB for events that can be expired and enqueues them into the broadcast channel
async fn expire_pending_actions(carbon_config: &Carbon, pool: &PgPool, carbon_broadcaster: Sender<BroadcastRequest>) -> Result<()> {
    // check for new events
    debug!("Checking for expired pending_action_events...");
    let events: Vec<DbPendingActionEvent> = sqlx::query_as!(
        DbPendingActionEvent,
        r#"
        SELECT *
        FROM pending_action_events
        WHERE broadcast_status = $1
        AND (relay_details ->> 'expiry_block_time')::timestamp < NOW()
        "#,
        "pending_broadcast"
    )
        .fetch_all(pool)
        .await?;


    let expired_nonces: Vec<u64> = events
        .into_iter()
        .map(|event| event.nonce.to_u64().expect("could not convert nonce to u64"))
        .collect();

    if !expired_nonces.is_empty() {
        queue_expire_relay(carbon_config, carbon_broadcaster, expired_nonces).await;
    }
    Ok(())
}

// queue the expire_relay process to broadcaster for carbon which will expire a particular relay and refund fees to user
pub async fn queue_expire_relay(carbon_config: &Carbon, carbon_broadcaster: Sender<BroadcastRequest>, nonces: Vec<u64>) {
    info!("Expiring relay on {:?} for nonces {:?}", &carbon_config.rpc_url, &nonces);

    // Create a oneshot channel for the response
    let (callback_tx, callback_rx) = oneshot::channel();

    // Create msg
    let msg_expire_actions = MsgPruneExpiredPendingActions {
        creator: carbon_config.relayer_address.clone(),
        nonces,
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
