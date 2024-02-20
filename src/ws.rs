use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use futures::lock::Mutex;
use futures::SinkExt;
use futures::stream::StreamExt;
use serde_json::{json, Value};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Error as TungsteniteError, tungstenite::protocol::Message};
use tracing::{error, warn, info, debug};
use url::Url;

type MessageHandler = Arc<Mutex<dyn FnMut(String) + Send>>;

pub struct Subscription {
    pub message: Message,
    pub handler: MessageHandler,
}

pub struct JSONWebSocketClient {
    url: Url,
    subscriptions: HashMap<String, Subscription>,
}


impl JSONWebSocketClient {
    pub fn new(url: Url) -> Self {
        Self { url, subscriptions: HashMap::new() }
    }

    pub async fn connect(&self) -> tokio_tungstenite::tungstenite::Result<()> {
        info!("Connecting to WS client");
        loop {
            match connect_async(&self.url).await {
                Ok((ws_stream, _)) => {
                    info!("WebSocket connected to {:?}", self.url.to_string());
                    let (write, read) = ws_stream.split();
                    self.handle_connection(write, read).await;
                }
                Err(e) => error!("Failed to connect: {:?}", e),
            }

            warn!("Attempting to reconnect in 10 seconds...");
            sleep(Duration::from_secs(10)).await;
        }
    }

    pub fn add_cosmos_subscription(&mut self, id: String, query: String, message_handler: MessageHandler) {
        self.subscriptions.insert(id.clone(), Subscription {
            message: Message::Text(
                json!({
                    "jsonrpc": "2.0",
                    "method": "subscribe",
                    "id": id.clone(),
                    "params": {
                        "query": query,
                    }
                }).to_string(),
            ),
            handler: message_handler,
        });
    }

    async fn handle_connection(&self, mut write: impl SinkExt<Message> + Unpin, mut read: impl StreamExt<Item=tokio_tungstenite::tungstenite::Result<Message>> + Unpin) {
        // Subscribe to each message using the HashMap
        for subscription in self.subscriptions.values() {
            let query = subscription.message.clone().into_text();
            info!("Subscribing to: {:?}", query.unwrap());
            if write.send(subscription.message.clone()).await.is_err() {
                error!("Failed to send subscribe message");
                return; // Exit and attempt to reconnect
            }
        }

        // Listen for messages and dispatch to handlers based on the id
        while let Some(message) = read.next().await {
            if let Err(e) = self.handle_message(message).await {
                error!("Failed to handle message: {}", e);
            }
        }
    }

    async fn handle_message(&self, message: Result<Message, TungsteniteError>) -> Result<()> {
        let msg = message.context("Error reading message")?;

        let text = msg.into_text().context("Failed to convert message to text")?;

        if text.is_empty() {
            // Log and return early for empty messages, not considered an error
            debug!("Received empty message, ignoring.");
            return Ok(());
        }

        let json_msg: Value = serde_json::from_str(&text)
            .with_context(|| format!("Failed to parse text to JSON, text: {:?}", text))?;

        if json_msg["result"].as_object().map_or(true, |obj| obj.is_empty()) {
            debug!("Ignoring message with empty result: {:?}", json_msg);
            return Ok(());
        }

        if let Some(id) = json_msg["id"].as_str() {
            if let Some(subscription) = self.subscriptions.get(id) {
                let handler = subscription.handler.clone(); // Clone Arc to share ownership
                tokio::spawn(async move {
                    let mut handler = handler.lock().await; // Lock within the spawned task
                    (*handler)(text); // Invoke the handler
                });
            } else {
                error!("No subscription found for id: {}", id);
            }
        } else {
            error!("Message does not contain an id field");
        }

        Ok(())
    }
}

