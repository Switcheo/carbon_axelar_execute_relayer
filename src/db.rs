pub mod carbon_events;

use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use sqlx::types::{BigDecimal, Json};

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

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct DbPayloadAcknowledgedEvent {
    pub id: i32,
    // reference payload_types (from carbon x/bridge/types/payload_encoding.go)
    // RegisterToken = 0
    // DeregisterToken = 1
    // DeployToken = 2
    // RegisterExecutable = 3
    // DeregisterExecutable = 4
    // Withdraw = 5
    // ExecuteGateway = 6
    // WithdrawAndExecute = 7
    // PauseContract = 8
    // UnpauseContract = 9
    pub payload_type: i32,
    pub nonce: BigDecimal,
    pub payload: String, // hex string
    pub payload_hash: String, // hex string
    pub payload_encoding: String,
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct DbWithdrawTokenConfirmedEvent {
    pub id: i32,
    pub coin: Json<Coin>,
    pub connection_id: String,
    pub receiver: String,
    pub relay_fee: Json<Coin>,
    pub relayer_deposit_address: String,
    pub sender: String,
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

impl PayloadType {
    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
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
            _ => Err(()), // or Ok(PayloadType::Unknown) if you have an Unknown variant
        }
    }
}