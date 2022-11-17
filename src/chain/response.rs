use cosmrs::proto::{
    cosmos::base::abci::v1beta1::TxResponse as CosmosResponse,
    tendermint::abci::{Event as ProtoEvent, EventAttribute},
};
use cosmrs::rpc::abci::{
    tag::{Key, Tag as TendermintProtoTag, Value},
    Code as TendermintCode, Event as TendermintEvent,
};
use cosmrs::rpc::endpoint::{
    abci_query::AbciQuery,
    broadcast::tx_async::Response as AsyncTendermintResponse,
    broadcast::tx_commit::{Response as BlockingTendermintResponse, TxResult},
    broadcast::tx_sync::Response as SyncTendermintResponse,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::error::{ChainError, DeserializeError};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct ChainResponse {
    pub code: Code,
    pub data: Option<Vec<u8>>,
    pub log: String,
}

impl ChainResponse {
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> Result<T, DeserializeError> {
        let r: T = serde_json::from_slice(
            self.data
                .as_ref()
                .ok_or(DeserializeError::EmptyResponse)?
                .as_slice(),
        )?;
        Ok(r)
    }
}

impl From<AbciQuery> for ChainResponse {
    fn from(res: AbciQuery) -> ChainResponse {
        ChainResponse {
            code: res.code.into(),
            data: Some(res.value),
            log: res.log.to_string(),
        }
    }
}

impl From<TxResult> for ChainResponse {
    fn from(res: TxResult) -> ChainResponse {
        ChainResponse {
            code: res.code.into(),
            data: res.data.map(|d| d.into()),
            log: res.log.to_string(),
        }
    }
}

impl From<tonic::Status> for ChainResponse {
    fn from(res: tonic::Status) -> ChainResponse {
        ChainResponse {
            code: res.code().into(),
            data: Some(res.details().into()),
            log: res.message().into(),
        }
    }
}

/// AsyncChainTxResponse is returned from the async `tx_broadcast()` api.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct AsyncChainTxResponse {
    pub res: ChainResponse,
    pub tx_hash: String,
}

impl AsRef<ChainResponse> for AsyncChainTxResponse {
    fn as_ref(&self) -> &ChainResponse {
        &self.res
    }
}

impl From<CosmosResponse> for AsyncChainTxResponse {
    fn from(res: CosmosResponse) -> Self {
        Self {
            res: ChainResponse {
                code: res.code.into(),
                data: Some(res.data.into()), // TODO
                log: res.raw_log,
            },
            tx_hash: res.txhash,
        }
    }
}

impl From<AsyncTendermintResponse> for AsyncChainTxResponse {
    fn from(res: AsyncTendermintResponse) -> Self {
        Self {
            res: ChainResponse {
                code: res.code.into(),
                data: Some(res.data.into()),
                log: res.log.to_string(),
            },
            tx_hash: res.hash.to_string(),
        }
    }
}

impl From<SyncTendermintResponse> for AsyncChainTxResponse {
    fn from(res: SyncTendermintResponse) -> Self {
        Self {
            res: ChainResponse {
                code: res.code.into(),
                data: Some(res.data.into()),
                log: res.log.to_string(),
            },
            tx_hash: res.hash.to_string(),
        }
    }
}

/// ChainTxResponse is returned from the blocking `tx_broadcast_block()` api.
/// Since we wait for the tx to be commited in the next block, we get the full tx data.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct ChainTxResponse {
    pub res: ChainResponse,
    pub events: Vec<Event>,
    pub gas_wanted: u64,
    pub gas_used: u64,
    pub tx_hash: String,
    pub height: u64,
}

impl ChainTxResponse {
    pub fn find_event_tags(&self, event_type: String, key_name: String) -> Vec<&Tag> {
        let mut events = vec![];
        for event in &self.events {
            if event.type_str == event_type {
                for attr in &event.attributes {
                    if attr.key == key_name {
                        events.push(attr);
                    }
                }
            }
        }
        events
    }
}

impl AsRef<ChainResponse> for ChainTxResponse {
    fn as_ref(&self) -> &ChainResponse {
        &self.res
    }
}

impl From<BlockingTendermintResponse> for ChainTxResponse {
    fn from(res: BlockingTendermintResponse) -> Self {
        ChainTxResponse {
            res: ChainResponse {
                code: res.deliver_tx.code.into(),
                data: res.deliver_tx.data.map(|d| d.into()),
                log: res.deliver_tx.log.to_string(),
            },
            events: res.deliver_tx.events.into_iter().map(Into::into).collect(),
            gas_used: res.deliver_tx.gas_used.into(),
            gas_wanted: res.deliver_tx.gas_wanted.into(),
            tx_hash: res.hash.to_string(),
            height: res.height.into(),
        }
    }
}

impl TryFrom<CosmosResponse> for ChainTxResponse {
    type Error = ChainError;

    fn try_from(res: CosmosResponse) -> Result<ChainTxResponse, Self::Error> {
        Ok(ChainTxResponse {
            res: ChainResponse {
                code: res.code.into(),
                data: Some(res.data.into()), // TODO
                log: res.raw_log,
            },
            events: res
                .events
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            gas_wanted: res.gas_wanted as u64,
            gas_used: res.gas_used as u64,
            tx_hash: res.txhash,
            height: res.height as u64,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Code {
    Ok,
    Err(u32),
}

impl Default for Code {
    fn default() -> Code {
        Code::Ok
    }
}

impl Code {
    pub fn is_ok(self) -> bool {
        match self {
            Code::Ok => true,
            Code::Err(_) => false,
        }
    }

    pub fn is_err(self) -> bool {
        !self.is_ok()
    }

    pub fn value(self) -> u32 {
        u32::from(self)
    }
}

impl From<u32> for Code {
    fn from(value: u32) -> Code {
        match value {
            0 => Code::Ok,
            err => Code::Err(err),
        }
    }
}

impl From<Code> for u32 {
    fn from(code: Code) -> u32 {
        match code {
            Code::Ok => 0,
            Code::Err(err) => err,
        }
    }
}

impl From<u16> for Code {
    fn from(value: u16) -> Code {
        match value {
            0 => Code::Ok,
            err => Code::Err(err.into()),
        }
    }
}

impl From<u8> for Code {
    fn from(value: u8) -> Code {
        match value {
            0 => Code::Ok,
            err => Code::Err(err.into()),
        }
    }
}

impl From<TendermintCode> for Code {
    fn from(value: TendermintCode) -> Code {
        match value {
            TendermintCode::Ok => Code::Ok,
            TendermintCode::Err(err) => Code::Err(err.into()),
        }
    }
}

impl From<tonic::Code> for Code {
    fn from(value: tonic::Code) -> Code {
        // NOTE: `value` is an isize, so we are just manually
        // matching them to avoid any casting errors in the future
        match value {
            tonic::Code::Ok => Code::Ok,
            tonic::Code::Cancelled => Code::Err(1),
            tonic::Code::Unknown => Code::Err(2),
            tonic::Code::InvalidArgument => Code::Err(3),
            tonic::Code::DeadlineExceeded => Code::Err(4),
            tonic::Code::NotFound => Code::Err(5),
            tonic::Code::AlreadyExists => Code::Err(6),
            tonic::Code::PermissionDenied => Code::Err(7),
            tonic::Code::ResourceExhausted => Code::Err(8),
            tonic::Code::FailedPrecondition => Code::Err(9),
            tonic::Code::Aborted => Code::Err(10),
            tonic::Code::OutOfRange => Code::Err(11),
            tonic::Code::Unimplemented => Code::Err(12),
            tonic::Code::Internal => Code::Err(13),
            tonic::Code::Unavailable => Code::Err(14),
            tonic::Code::DataLoss => Code::Err(15),
            tonic::Code::Unauthenticated => Code::Err(16),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Event {
    pub type_str: String,
    pub attributes: Vec<Tag>,
}

impl From<TendermintEvent> for Event {
    fn from(e: TendermintEvent) -> Self {
        Self {
            type_str: e.type_str,
            attributes: e.attributes.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<Event> for TendermintEvent {
    type Error = ChainError;

    fn try_from(e: Event) -> Result<Self, Self::Error> {
        Ok(Self {
            type_str: e.type_str,
            attributes: e
                .attributes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<ProtoEvent> for Event {
    type Error = ChainError;

    fn try_from(e: ProtoEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            type_str: e.r#type,
            attributes: e
                .attributes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<Event> for ProtoEvent {
    fn from(e: Event) -> Self {
        Self {
            r#type: e.type_str,
            attributes: e.attributes.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

impl From<TendermintProtoTag> for Tag {
    fn from(tag: TendermintProtoTag) -> Self {
        Self {
            key: tag.key.to_string(),
            value: tag.value.to_string(),
        }
    }
}

impl TryFrom<Tag> for TendermintProtoTag {
    type Error = ChainError;

    fn try_from(tag: Tag) -> Result<Self, Self::Error> {
        Ok(Self {
            key: Key::from_str(&tag.key)?,
            value: Value::from_str(&tag.value)?,
        })
    }
}

impl From<Tag> for EventAttribute {
    fn from(tag: Tag) -> Self {
        Self {
            key: tag.key.into_bytes().into(),
            value: tag.value.into_bytes().into(),
            index: true,
        }
    }
}

impl TryFrom<EventAttribute> for Tag {
    type Error = ChainError;

    fn try_from(attr: EventAttribute) -> Result<Self, Self::Error> {
        Ok(Self {
            key: String::from_utf8(attr.key.into()).map_err(|e| ChainError::ProtoDecoding {
                message: e.to_string(),
            })?,
            value: String::from_utf8(attr.value.into()).map_err(|e| ChainError::ProtoDecoding {
                message: e.to_string(),
            })?,
        })
    }
}
