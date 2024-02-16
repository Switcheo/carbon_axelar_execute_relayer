use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use sqlx::types::{BigDecimal, Json};

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct DbWithdrawTokenAcknowledgedEvent {
    pub id: i32,
    pub coin: Json<Coin>,
    pub connection_id: String,
    pub receiver: String,
    pub relay_fee: Json<Coin>,
    pub relayer_deposit_address: String,
    pub sender: String,
    pub payload_hash: String, // hex string
    pub payload: String, // hex string
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct DbContractCallApprovedEvent {
    pub id: i32,
    pub blockchain: String,
    pub broadcast_status: String,
    pub command_id: String,
    pub source_chain: String, // hex string
    pub source_address: String,
    pub contract_address: String,
    pub payload_hash: String, // hex string
    pub source_tx_hash: String, // hex string
    pub source_event_index: BigDecimal, // Using BigDecimal to represent NUMERIC
    pub payload: String, // hex string
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coin {
    pub denom: String,
    #[serde(deserialize_with = "deserialize_amount")]
    pub amount: u64,
}

// Custom deserializer for the amount field to turn string into u64
fn deserialize_amount<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    u64::from_str(&s).map_err(serde::de::Error::custom)
}