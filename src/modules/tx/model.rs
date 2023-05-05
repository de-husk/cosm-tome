use cosmrs::proto::traits::MessageExt;
use cosmrs::proto::{
    cosmos::tx::v1beta1::{BroadcastMode as ProtoBroadcastMode, TxRaw},
    traits::Message,
};
use cosmrs::tx::Raw;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::chain::error::ChainError;

use super::error::TxError;

/// `BroadcastMode::Block` is deprecated and removed from latest version of cosmos-sdk.
/// `BroadcastMode` only contains the non-deprecated async broadcasting modes, starting from 2
/// for backwards compatability when converting between the cosmos-sdk proto.
#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, PartialOrd, Ord, Hash,
)]
#[repr(i32)]
pub enum BroadcastMode {
    Block = 1,
    /// BROADCAST_MODE_SYNC defines a tx broadcasting mode where the client waits for a CheckTx execution response only.
    Sync = 2,
    /// BROADCAST_MODE_ASYNC defines a tx broadcasting mode where the client returns immediately.
    Async = 3,
}

impl AsRef<str> for BroadcastMode {
    fn as_ref(&self) -> &str {
        match self {
            BroadcastMode::Block => "BROADCAST_MODE_BLOCK",
            BroadcastMode::Sync => "BROADCAST_MODE_SYNC",
            BroadcastMode::Async => "BROADCAST_MODE_ASYNC",
        }
    }
}

impl TryFrom<i32> for BroadcastMode {
    type Error = TxError;

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == BroadcastMode::Block as i32 => Ok(BroadcastMode::Block),
            x if x == BroadcastMode::Sync as i32 => Ok(BroadcastMode::Sync),
            x if x == BroadcastMode::Async as i32 => Ok(BroadcastMode::Async),
            _ => Err(TxError::BroadcastMode { i: v }),
        }
    }
}

impl From<BroadcastMode> for ProtoBroadcastMode {
    fn from(mode: BroadcastMode) -> Self {
        match mode {
            BroadcastMode::Block => ProtoBroadcastMode::Block,
            BroadcastMode::Sync => ProtoBroadcastMode::Sync,
            BroadcastMode::Async => ProtoBroadcastMode::Async,
        }
    }
}

impl TryFrom<ProtoBroadcastMode> for BroadcastMode {
    type Error = TxError;

    fn try_from(mode: ProtoBroadcastMode) -> Result<Self, Self::Error> {
        match mode {
            ProtoBroadcastMode::Block => Ok(BroadcastMode::Block),
            ProtoBroadcastMode::Sync => Ok(BroadcastMode::Sync),
            ProtoBroadcastMode::Async => Ok(BroadcastMode::Async),
            ProtoBroadcastMode::Unspecified => Err(TxError::BroadcastMode { i: 0 }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawTx(TxRaw);

impl RawTx {
    /// Deserialize raw transaction from serialized protobuf.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ChainError> {
        Ok(RawTx(
            Message::decode(bytes).map_err(ChainError::prost_proto_decoding)?,
        ))
    }

    /// Serialize raw transaction as a byte vector.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ChainError> {
        self.0.to_bytes().map_err(ChainError::prost_proto_encoding)
    }
}

impl From<RawTx> for TxRaw {
    fn from(tx: RawTx) -> Self {
        tx.0
    }
}

impl From<TxRaw> for RawTx {
    fn from(tx: TxRaw) -> Self {
        RawTx(tx)
    }
}

impl From<RawTx> for Raw {
    fn from(tx: RawTx) -> Self {
        tx.0.into()
    }
}

impl From<Raw> for RawTx {
    fn from(tx: Raw) -> Self {
        RawTx(tx.into())
    }
}
