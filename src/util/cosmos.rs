use serde::{Deserialize, Serialize};

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
