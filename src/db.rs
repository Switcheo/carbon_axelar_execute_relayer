use std::str::FromStr;
use anyhow::anyhow;

use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};
use sqlx::FromRow;
use sqlx::types::{BigDecimal, Json, JsonValue};

use crate::util::datetime::{time_difference_str, timestamp_to_datetime};

pub mod carbon_events;
pub mod evm_events;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingActionType {
    PendingRegisterTokenType = 0,
    PendingDeregisterTokenType,
    PendingDeployNativeTokenType,
    PendingWithdrawAndExecuteType,
    PendingWithdrawType,
    PendingExecuteType,
}

// carbon
#[derive(Debug, Clone, PartialEq, FromRow, Deserialize, Serialize)]
pub struct DbPendingActionEvent {
    pub id: i32,
    pub connection_id: String,
    pub bridge_id: String,
    pub chain_id: String,
    pub nonce: i64,
    pub pending_action_type: i32,
    pub retry_count: i32,
    pub relay_details: JsonValue,
}

// carbon
#[derive(Debug, Clone, PartialEq)]
pub struct ExpiredPendingActionEvent {
    pub nonce: i64,
    pub pending_action_type: i32,
    pub connection_id: String,
    pub relay_details: JsonValue,
}

// carbon
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeAcknowledgedEvent {
    pub id: i32,
    pub bridge_id: String,
    pub chain_id: String,
    pub gateway_address: String,
    pub nonce: i64,
}

// carbon
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeRevertedEvent {
    pub id: i32,
    pub bridge_id: String,
    pub chain_id: String,
    pub gateway_address: String,
    pub nonce: i64,
}

// carbon
#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct DbAxelarCallContractEvent {
    pub id: i32,
    pub nonce: i64,
    pub payload_hash: String, // hex string
    pub payload: String, // hex string
    pub payload_encoding: String,
}

// evm
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
pub struct RelayDetails {
    pub fee_receiver_address: String,
    pub fee_sender_address: String,
    pub fee: Json<Coin>,
    pub expiry_block_time: pbjson_types::Timestamp,
    pub created_at: pbjson_types::Timestamp,
    pub sent_at: Option<pbjson_types::Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coin {
    pub denom: String,
    #[serde(deserialize_with = "deserialize_str_as_u64", serialize_with = "serialize_u64_as_str")]
    pub amount: u64,
}

// Custom deserializer for the amount field to turn string into u64
fn deserialize_str_as_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    u64::from_str(&s).map_err(serde::de::Error::custom)
}

// Custom serializer for u64 fields represented as strings
fn serialize_u64_as_str<S>(x: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    serializer.serialize_str(&x.to_string())
}

impl PendingActionType {
    pub fn from_prefix(prefix: &str) -> anyhow::Result<Self> {
        match prefix {
            "register_token" => Ok(PendingActionType::PendingRegisterTokenType),
            "deregister_token" => Ok(PendingActionType::PendingDeregisterTokenType),
            "deploy_native_token" => Ok(PendingActionType::PendingDeployNativeTokenType),
            _ if prefix.starts_with("withdraw_and_execute") => Ok(PendingActionType::PendingWithdrawAndExecuteType),
            _ if prefix.starts_with("withdraw") => Ok(PendingActionType::PendingWithdrawType),
            _ if prefix.starts_with("execute") => Ok(PendingActionType::PendingExecuteType),
            _ => Err(anyhow!("Invalid action type prefix: {}", prefix)),
        }
    }
}

impl From<PendingActionType> for i32 {
    fn from(action_type: PendingActionType) -> Self {
        action_type as i32
    }
}

impl TryFrom<i32> for PendingActionType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PendingActionType::PendingRegisterTokenType),
            1 => Ok(PendingActionType::PendingDeregisterTokenType),
            2 => Ok(PendingActionType::PendingDeployNativeTokenType),
            3 => Ok(PendingActionType::PendingWithdrawAndExecuteType),
            4 => Ok(PendingActionType::PendingWithdrawType),
            5 => Ok(PendingActionType::PendingExecuteType),
            _ => Err(anyhow::anyhow!("Invalid value for PendingActionType: {}", value)),
        }
    }
}

impl DbPendingActionEvent {
    pub fn get_relay_details(&self) -> RelayDetails {
        let relay_details_value = serde_json::to_value(&self.relay_details).expect("cannot parse relay_details");
        let relay_details: RelayDetails = from_value(relay_details_value).expect("cannot parse relay_details_value");
        relay_details
    }

    pub fn get_relay_details_value(&self) -> Value {
        serde_json::to_value(&self.relay_details).expect("cannot parse relay_details")
    }

    pub fn get_pending_action_type(&self) -> PendingActionType {
        self.pending_action_type.try_into().unwrap()
    }
}

impl RelayDetails {
    pub fn has_expired(&self) -> bool {
        let expiry_time = timestamp_to_datetime(&self.expiry_block_time);
        let current_time = Utc::now();
        current_time > expiry_time
    }

    pub fn get_expiry_duration(&self) -> String {
        let expiry_time = timestamp_to_datetime(&self.expiry_block_time);
        let current_time = Utc::now();
        let time_difference = current_time - expiry_time;
        time_difference_str(time_difference)
    }

    pub fn is_sent(&self) -> bool {
        self.sent_at.is_some()
    }
}

