
use cosmrs::{
    crypto::PublicKey,
    tx::{Msg, MsgType},
    Coin, ErrorReport, Result,
};
use serde::{Deserialize, Serialize};

/// BaseAccount defines a base account type. It contains all the necessary fields
/// for basic account functionality. Any custom account type should extend this
/// type for additional functionality (e.g. vesting).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BaseAccount {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub pub_key: ::core::option::Option<::prost_types::Any>,
    #[prost(uint64, tag = "3")]
    pub account_number: u64,
    #[prost(uint64, tag = "4")]
    pub sequence: u64,
}
impl ::prost::Name for BaseAccount {
    const NAME: &'static str = "BaseAccount";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}

/// MsgSend represents a message to send coins from one account to another.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSend {
    #[prost(string, tag = "1")]
    pub from_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub to_address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgCreateConnection {
    pub creator: String,
    pub bridge_id: u64,
    pub chain_id: String,
    pub chain_display_name: String,
    pub token_gateway_address: String,
    pub encoding: String,
    pub is_enabled: bool,
}

impl Msg for MsgCreateConnection {
    type ValidationError = ErrorReport; // Use cosmrs's ErrorReport for validation errors
    type Proto = prost_types::Any; // The Protobuf type corresponding to this message

    fn to_proto(&self) -> Result<Self::Proto> {
        let type_url = "/Switcheo.carbon.bridge.MsgCreateConnection".to_string(); // URL used in the Cosmos SDK for this message type

        let value = serde_json::to_vec(&self)?; // Serialize this struct as JSON
        Ok(prost_types::Any { type_url, value })
    }

    fn from_proto(proto: Self::Proto) -> Result<Self> {
        if proto.type_url != "/Switcheo.carbon.bridge.MsgCreateConnection" {
            return Err(ErrorReport::msg("incorrect type URL"));
        }

        serde_json::from_slice(&proto.value).map_err(Into::into) // Deserialize the JSON back into the struct
    }

    fn validate_basic(&self) -> Result<()> {
        // Add basic validation logic if necessary
        Ok(())
    }
}

impl MsgType for MsgCreateConnection {
    const TYPE_URL: &'static str = "/Switcheo.carbon.bridge.MsgCreateConnection";
}