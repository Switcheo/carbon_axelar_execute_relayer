use std::fmt::Debug;
use cosmrs::{tx::Msg, ErrorReport, Result};
use prost::Name;
use prost_types::Any;

pub trait IntoAny: Send + Debug {
    fn into_any(self: Box<Self>) -> Any;
}

// ** MsgStartRelay ** //

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct MsgStartRelay {
    pub relayer: String,
    pub nonce: u64,
}

impl Name for crate::switcheo::carbon::bridge::MsgStartRelay {
    const NAME: &'static str = "MsgStartRelay";
    const PACKAGE: &'static str = "Switcheo.carbon.bridge";
}

impl Msg for MsgStartRelay {
    type Proto = crate::switcheo::carbon::bridge::MsgStartRelay;
}

impl TryFrom<crate::switcheo::carbon::bridge::MsgStartRelay> for MsgStartRelay {
    type Error = ErrorReport;

    fn try_from(proto: crate::switcheo::carbon::bridge::MsgStartRelay) -> Result<MsgStartRelay> {
        MsgStartRelay::try_from(&proto)
    }
}

impl TryFrom<&crate::switcheo::carbon::bridge::MsgStartRelay> for MsgStartRelay {
    type Error = ErrorReport;

    fn try_from(proto: &crate::switcheo::carbon::bridge::MsgStartRelay) -> Result<MsgStartRelay> {
        Ok(MsgStartRelay {
            relayer: proto.relayer.parse()?,
            nonce: proto.nonce,
        })
    }
}

impl From<MsgStartRelay> for crate::switcheo::carbon::bridge::MsgStartRelay {
    fn from(start_relay: MsgStartRelay) -> crate::switcheo::carbon::bridge::MsgStartRelay {
        crate::switcheo::carbon::bridge::MsgStartRelay::from(&start_relay)
    }
}

impl From<&MsgStartRelay> for crate::switcheo::carbon::bridge::MsgStartRelay {
    fn from(msg: &MsgStartRelay) -> crate::switcheo::carbon::bridge::MsgStartRelay {
        crate::switcheo::carbon::bridge::MsgStartRelay {
            relayer: msg.relayer.to_string(),
            nonce: msg.nonce,
        }
    }
}

impl From<Box<MsgStartRelay>> for Any {
    fn from(value: Box<MsgStartRelay>) -> Self {
        (*value).to_any().unwrap()
    }
}

impl IntoAny for MsgStartRelay {
    fn into_any(self: Box<Self>) -> Any {
        self.into()
    }
}

// ** MsgPruneExpiredPendingActions ** //

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct MsgPruneExpiredPendingActions {
    pub creator: String,
    pub nonces: Vec<u64>,
}

impl Name for crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions {
    const NAME: &'static str = "MsgPruneExpiredPendingActions";
    const PACKAGE: &'static str = "Switcheo.carbon.bridge";
}

impl Msg for MsgPruneExpiredPendingActions {
    type Proto = crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions;
}

impl TryFrom<crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions> for MsgPruneExpiredPendingActions {
    type Error = ErrorReport;

    fn try_from(proto: crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions) -> Result<MsgPruneExpiredPendingActions> {
        MsgPruneExpiredPendingActions::try_from(&proto)
    }
}

impl TryFrom<&crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions> for MsgPruneExpiredPendingActions {
    type Error = ErrorReport;

    fn try_from(proto: &crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions) -> Result<MsgPruneExpiredPendingActions> {
        Ok(MsgPruneExpiredPendingActions {
            creator: proto.creator.parse()?,
            nonces: proto.nonces.clone(),
        })
    }
}

impl From<MsgPruneExpiredPendingActions> for crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions {
    fn from(start_relay: MsgPruneExpiredPendingActions) -> crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions {
        crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions::from(&start_relay)
    }
}

impl From<&MsgPruneExpiredPendingActions> for crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions {
    fn from(msg: &MsgPruneExpiredPendingActions) -> crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions {
        crate::switcheo::carbon::bridge::MsgPruneExpiredPendingActions {
            creator: msg.creator.to_string(),
            nonces: msg.nonces.clone(),
        }
    }
}

impl From<Box<MsgPruneExpiredPendingActions>> for Any {
    fn from(value: Box<MsgPruneExpiredPendingActions>) -> Self {
        (*value).to_any().unwrap()
    }
}

impl IntoAny for MsgPruneExpiredPendingActions {
    fn into_any(self: Box<Self>) -> Any {
        self.into()
    }
}