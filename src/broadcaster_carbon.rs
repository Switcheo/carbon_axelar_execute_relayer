use std::sync::Arc;

use anyhow::{Context, Error, Result};
use num_traits::ToPrimitive;
use sqlx::PgPool;
use sqlx::postgres::PgQueryResult;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{Duration, interval};
use tracing::{debug, error, info, instrument};

use crate::conf::{Carbon};
use crate::db::{DbPendingActionEvent};
use crate::util::carbon_tx::send_msg_start_relay;

#[instrument(name = "broadcaster_carbon", skip_all)]
pub async fn init_all(carbon_config: &Carbon, pg_pool: Arc<PgPool>) {
    // initialize signature providers for each chain
    let channel_tx = init_channel(carbon_config.clone(), pg_pool.clone()).await;

    // listen for db events that have not been broadcast
    let pg_pool_clone = pg_pool.clone();
    poll_for_new_events(pg_pool_clone, channel_tx).await;
}

// Polls for PendingAction that has not startRelay saved in the DB and enqueues them into the broadcast channel
#[instrument(name = "poll_for_pending_actions_awaiting_relay", skip_all)]
async fn poll_for_new_events(pool: Arc<PgPool>, channel_tx: Sender<DbPendingActionEvent>) {
    info!("Watching for pending actions to see if we can relay them");
    let mut interval = interval(Duration::from_secs(5));
    loop {
        interval.tick().await;
        if let Err(e) = queue_new_start_relays(&pool, channel_tx.clone()).await {
            error!("Failed to queue new pending actions for start relay: {}", e);
        }
    }
}

// Checks the DB for events that can be executed and enqueues them into the broadcast channel
async fn queue_new_start_relays(pool: &PgPool, channel_tx: Sender<DbPendingActionEvent>) -> Result<()> {
    // check for new events
    debug!("Checking for pending actions that can start relay...");
    let events: Vec<DbPendingActionEvent> = sqlx::query_as!(
        DbPendingActionEvent,
        "SELECT * FROM pending_action_events WHERE broadcast_status = $1",
        "pending_broadcast"
    )
        .fetch_all(pool)
        .await?;

    for event in events {
        info!("DB event found: {:?}", event);

        if let Err(e) = channel_tx.send(event.clone()).await {
            error!("Failed to send DbPendingActionEvent to channel, err: {:?}", e);
        }
    }
    Ok(())
}

async fn init_channel(conf: Carbon, pg_pool: Arc<PgPool>) -> Sender<DbPendingActionEvent> {
    info!("Initializing receive_and_broadcast for Carbon");
    // init channel
    let (tx, rx) = mpsc::channel::<DbPendingActionEvent>(100); // Adjust the size based on expected load
    let pg_pool = pg_pool.clone();

    // spawn receiving logic
    tokio::spawn(async move {
        if let Err(e) = receive_and_broadcast(conf.clone(), rx, pg_pool).await {
            // Handle or log the error e
            error!("Error in receive_and_broadcast: {:?}", e);
        }
    });
    tx
}


#[instrument(name = "broadcaster_carbon::receive_and_broadcast", skip_all)]
async fn receive_and_broadcast(conf: Carbon, mut rx: Receiver<DbPendingActionEvent>, pg_pool: Arc<PgPool>) -> Result<()> {
    Ok(while let Some(event) = rx.recv().await {
        // TODO: check if expired
        // if expired, broadcast expire relay
        // delete pending action from db

        // Double check db to make sure it is still pending_broadcast
        let exists = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM pending_action_events WHERE id = $1 AND broadcast_status = 'pending_broadcast')",
                event.id.clone()
            )
            .fetch_one(pg_pool.as_ref())
            .await?
            .exists.unwrap_or(false);
        if !exists {
            info!("Skipping pending_action_event as it is not pending, and no broadcast is needed. nonce: {:?}", &event.nonce);
            continue;
        }

        // Update to broadcasting
        if let Err(e) = sqlx::query!(
                            "UPDATE pending_action_events SET broadcast_status = $1 WHERE id = $2",
                            "broadcasting",
                            event.id.clone()
                        )
            .execute(pg_pool.as_ref())
            .await {
            error!("UPDATE failed with error: {:?}", e);
            continue;
        }

        // broadcast
        let nonce = event.nonce.to_u64().expect("could not convert nonce to u64");
        send_msg_start_relay(conf.clone(), nonce).await?;

        // if no errors, we can update
        update_executed(&pg_pool, &event).await?;
    })
}

async fn update_executed(pg_pool: &Arc<PgPool>, event: &DbPendingActionEvent) -> std::result::Result<PgQueryResult, Error> {
    sqlx::query!(
                        "UPDATE contract_call_approved_events SET broadcast_status = $1 WHERE id = $2",
                        "executed",
                        &event.id
                    )
        .execute(pg_pool.as_ref())
        .await.context("Failed to update contract_call_approved_events")
}
