use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct WebSocketMessage {
    pub id: String,
    pub jsonrpc: String,
    pub result: WsResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WsResult {
    pub query: String,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    #[serde(rename = "type")]
    pub data_type: String,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Value {
    #[serde(rename = "TxResult")]
    pub tx_result: TxResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxResult {
    pub height: String,
    pub tx: String,
    pub result: TxResultInner,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxResultInner {
    pub data: String,
    pub log: String,
    pub gas_wanted: String,
    pub gas_used: String,
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub attributes: Vec<Attribute>,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attribute {
    pub index: bool,
    pub key: String,
    pub value: String,
}

// Extracts relevant events from a JSON message
pub fn extract_events(msg: &str, event_name: &str) -> anyhow::Result<Vec<Event>> {
    let query_response = serde_json::from_str::<WebSocketMessage>(msg)
        .with_context(|| format!("Failed to parse JSON, provided string was: {}", msg))?;

    let events = query_response.result.data.value.tx_result.result.events
        .into_iter()
        .filter(|e| e.event_type == event_name)
        .collect();
    Ok(events)
}

async fn send_transaction(tx_bytes: Vec<u8>, node_url: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    // Convert tx_bytes to base64
    let tx_base64 = general_purpose::STANDARD.encode(&tx_bytes);

    // Prepare the JSON payload
    let payload = json!({
        "tx_bytes": tx_base64,
        "mode": "BROADCAST_MODE_BLOCK" // Or use BROADCAST_MODE_SYNC / BROADCAST_MODE_ASYNC
    });

    // Send the transaction
    let response = client
        .post(format!("{}/cosmos/tx/v1beta1/txs", node_url))
        .json(&payload)
        .send()
        .await?;

    // Handle the response
    let response_text = response.text().await?;
    println!("Response: {}", response_text);

    Ok(())
}


// async fn send_msg_start_relay(
//     endpoint: &str,
//     relayer_address: &str,
//     relayer_private_key: &str,
//     nonce: u64,
//     pending_action_type: u64,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     let client = rpc::Client::new(endpoint)?;
//
//     let msg = MsgStartRelay {
//         relayer: relayer_address.to_string(),
//         nonce,
//         pending_action_type,
//     };
//
//     let account_info = client
//         .auth
//         .account_info(relayer_address)
//         .await?
//         .ok_or("Account not found")?;
//
//     let public_key = secp256k1::SigningKey::from_str(relayer_private_key)?
//         .verifying_key()
//         .to_public_key()?;
//
//     let signer_info = SignerInfo::new_single(
//         public_key,
//         account_info.sequence.into(),
//     );
//
//     let fee = Fee::from_amount_and_gas(
//         Coin::new(1000, "uatom"),
//         200_000,
//     );
//
//     let auth_info = AuthInfo::new(vec![signer_info], fee);
//
//     let body = Body::new(vec![msg], "memo", 0);
//
//     let sign_doc = SignDoc::new(
//         body.clone(),
//         auth_info.clone(),
//         client.chain_id().await?.into(),
//         account_info.account_number.into(),
//     );
//
//     let signature = secp256k1::SigningKey::from_str(relayer_private_key)?
//         .sign(sign_doc.to_bytes().as_ref())?;
//
//     let tx = Tx::new(body, auth_info, vec![signature]);
//
//     let response = client
//         .broadcast_tx(tx.into())
//         .await?;
//
//     Ok(response.txhash)
// }
//
// #[tokio::main]
// async fn main() {
//     let endpoint = "http://localhost:26657";
//     let relayer_address = "cosmos1...";
//     let relayer_private_key = "private_key";
//     let nonce = 1;
//     let pending_action_type = 1;
//
//     match send_msg_start_relay(endpoint, relayer_address, relayer_private_key, nonce, pending_action_type).await {
//         Ok(tx_hash) => println!("Transaction sent successfully. Tx Hash: {}", tx_hash),
//         Err(e) => eprintln!("Failed to send transaction: {}", e),
//     }
// }