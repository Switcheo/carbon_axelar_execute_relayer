use cosmrs::proto::traits::Message;
use cosmrs::tx::Msg;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Message, Serialize, Deserialize)]
pub struct MsgStartRelay {
    #[prost(string, tag = "1")]
    pub relayer: String,
    #[prost(uint64, tag = "2")]
    pub nonce: u64,
    #[prost(uint64, tag = "3")]
    pub pending_action_type: u64,
}

impl Msg for MsgStartRelay {
    type Proto = Self;

    fn to_proto(&self) -> Self::Proto {
        self.clone()
    }

    fn from_proto(proto: Self::Proto) -> Self {
        proto
    }

    fn type_url() -> String {
        "/bridge.MsgStartRelay".to_string()
    }
}