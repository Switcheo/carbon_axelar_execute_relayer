use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures_util::lock::Mutex;
use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use serde_json::Value;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Error as TungsteniteError, tungstenite::protocol::Message};
use url::Url;

// type MessageHandler = Arc<Mutex<dyn FnMut(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>>;

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
    pub fn new(url: Url, subscriptions: HashMap<String, Subscription>) -> Self {
        Self { url, subscriptions }
    }

    pub async fn connect(&self) -> tokio_tungstenite::tungstenite::Result<()> {
        loop {
            match connect_async(&self.url).await {
                Ok((ws_stream, _)) => {
                    println!("WebSocket connected to {:?}", self.url.to_string());
                    let (write, read) = ws_stream.split();
                    self.handle_connection(write, read).await;
                }
                Err(e) => println!("Failed to connect: {:?}", e),
            }

            println!("Attempting to reconnect in 5 seconds...");
            sleep(Duration::from_secs(5)).await;
        }
    }


    async fn handle_connection(&self, mut write: impl SinkExt<Message> + Unpin, mut read: impl StreamExt<Item=tokio_tungstenite::tungstenite::Result<Message>> + Unpin) {
        // Subscribe to each message using the HashMap
        for subscription in self.subscriptions.values() {
            let query = subscription.message.clone().into_text();
            println!("subscribing to: {:?}", query.unwrap());
            if write.send(subscription.message.clone()).await.is_err() {
                println!("Failed to send subscribe message");
                return; // Exit and attempt to reconnect
            }
        }

        // Listen for messages and dispatch to handlers based on the id
        while let Some(message) = read.next().await {
            self.handle_message(message).await;
        }
    }

    async fn handle_message(&self, message: Result<Message, TungsteniteError>) {
        let msg = match message {
            Ok(msg) => msg,
            Err(e) => {
                println!("Error reading message: {:?}", e);
                return;
            }
        };

        let text = match msg.into_text() {
            Ok(text) => text,
            Err(e) => {
                println!("Failed to convert message to text: {:?}", e);
                return;
            }
        };

        let json_msg = match serde_json::from_str::<Value>(&text) {
            Ok(json) => json,
            Err(e) => {
                println!("Failed to parse text to JSON: {:?}, text: {:?}", e, text);
                return;
            }
        };

        // Check if the `result` object is not empty
        if json_msg["result"].as_object().map_or(true, |obj| obj.is_empty()) {
            println!("Ignoring message with empty result: {:?}", json_msg);
            return;
        }

        if let Some(id) = json_msg["id"].as_str() {
            if let Some(subscription) = self.subscriptions.get(id) {
                let handler = subscription.handler.clone(); // Clone Arc to share ownership
                // let msg_clone = ; // Clone the message if necessary
                tokio::spawn(async move {
                    let mut handler = handler.lock().await; // Lock within the spawned task
                    (*handler)(text.clone()); // Invoke the handler
                });
            } else {
                println!("No subscription found for id: {}", id);
            }
        } else {
            println!("Message does not contain an id field");
        }
    }
}
