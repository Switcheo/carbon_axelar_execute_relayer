pub mod carbon_events;
pub mod evm_events;

use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use sqlx::types::{BigDecimal, Json};
use chrono::{DateTime, Utc};
use serde_json::{from_value, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadType {
    RegisterToken = 0,
    DeregisterToken,
    DeployToken,
    RegisterExecutable,
    DeregisterExecutable,
    Withdraw,
    ExecuteGateway,
    WithdrawAndExecute,
    PauseContract,
    UnpauseContract,
}

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
#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct DbPendingActionEvent {
    pub id: i32,
    pub connection_id: String,
    pub bridge_id: String,
    pub chain_id: String,
    pub nonce: BigDecimal,
    pub pending_action_type: i32,
    pub relay_details: Json<RelayDetails>,
}

// carbon
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeAcknowledgedEvent {
    pub id: i32,
    pub bridge_id: String,
    pub chain_id: String,
    pub gateway_address: String,
    pub nonce: BigDecimal,
}

// carbon
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeRevertedEvent {
    pub id: i32,
    pub bridge_id: String,
    pub chain_id: String,
    pub gateway_address: String,
    pub nonce: BigDecimal,
}

// carbon
#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct DbAxelarCallContractEvent {
    pub id: i32,
    pub nonce: BigDecimal,
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
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub sent_at: Option<DateTime<Utc>>,
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

impl FromStr for PayloadType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(PayloadType::RegisterToken),
            "1" => Ok(PayloadType::DeregisterToken),
            "2" => Ok(PayloadType::DeployToken),
            "3" => Ok(PayloadType::RegisterExecutable),
            "4" => Ok(PayloadType::DeregisterExecutable),
            "5" => Ok(PayloadType::Withdraw),
            "6" => Ok(PayloadType::ExecuteGateway),
            "7" => Ok(PayloadType::WithdrawAndExecute),
            "8" => Ok(PayloadType::PauseContract),
            "9" => Ok(PayloadType::UnpauseContract),
            _ => Err(()),
        }
    }
}

// impl PendingActionType {
//     pub fn to_i32(&self) -> i32 {
//         *self as i32
//     }
// }

impl FromStr for PendingActionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(PendingActionType::PendingRegisterTokenType),
            "1" => Ok(PendingActionType::PendingDeregisterTokenType),
            "2" => Ok(PendingActionType::PendingDeployNativeTokenType),
            "3" => Ok(PendingActionType::PendingWithdrawAndExecuteType),
            "4" => Ok(PendingActionType::PendingWithdrawType),
            "5" => Ok(PendingActionType::PendingExecuteType),
            _ => Err(()),
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
}