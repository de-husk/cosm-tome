use cosmos_sdk_proto::{cosmos::base::abci::v1beta1::TxResponse, tendermint::abci::EventAttribute};
use cosmrs::tendermint::abci::Code;
use serde::{Deserialize, Serialize};
use tendermint_rpc::endpoint::{
    abci_query::AbciQuery,
    broadcast::tx_commit::{self, TxResult},
};

use super::error::{ChainError, DeserializeError};

#[derive(Clone, Debug, Default)]
pub struct ChainResponse {
    pub code: Code, // TODO: Make my own type here instead of exposing cosmrs lib
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
            code: res.code,
            data: Some(res.value),
            log: res.log.to_string(),
        }
    }
}

impl From<TxResult> for ChainResponse {
    fn from(res: TxResult) -> ChainResponse {
        ChainResponse {
            code: res.code,
            data: res.data.map(|d| d.into()),
            log: res.log.to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ChainTxResponse {
    pub res: ChainResponse,
    pub events: Vec<Event>,
    pub gas_wanted: u64,
    pub gas_used: u64,
    pub tx_hash: String,
    pub height: u64,
}

impl ChainTxResponse {
    pub fn find_event_tag(&self, event_type: String, key_name: String) -> Option<Tag> {
        for event in &self.events {
            if event.type_str == event_type {
                for attr in &event.attributes {
                    if attr.key == key_name {
                        return Some(attr.clone());
                    }
                }
            }
        }
        None
    }
}

impl From<tx_commit::Response> for ChainTxResponse {
    fn from(res: tx_commit::Response) -> ChainTxResponse {
        ChainTxResponse {
            events: res
                .deliver_tx
                .events
                .clone()
                .into_iter()
                .map(Into::into)
                .collect(),
            gas_used: res.deliver_tx.gas_used.into(),
            gas_wanted: res.deliver_tx.gas_wanted.into(),
            res: res.deliver_tx.into(),
            tx_hash: res.hash.to_string(),
            height: res.height.into(),
        }
    }
}

impl TryFrom<TxResponse> for ChainTxResponse {
    type Error = ChainError;

    fn try_from(res: TxResponse) -> Result<ChainTxResponse, Self::Error> {
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    #[serde(rename = "type")]
    pub type_str: String,
    pub attributes: Vec<Tag>,
}

impl From<cosmrs::tendermint::abci::Event> for Event {
    fn from(e: cosmrs::tendermint::abci::Event) -> Self {
        Self {
            type_str: e.type_str,
            attributes: e
                .attributes
                .into_iter()
                // TODO: Turn this into a `.map(Into::into)` as well
                .map(|a| Tag {
                    key: a.key.to_string(),
                    value: a.value.to_string(),
                })
                .collect(),
        }
    }
}

impl TryFrom<cosmos_sdk_proto::tendermint::abci::Event> for Event {
    type Error = ChainError;

    fn try_from(e: cosmos_sdk_proto::tendermint::abci::Event) -> Result<Self, Self::Error> {
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

impl TryFrom<EventAttribute> for Tag {
    type Error = ChainError;

    fn try_from(attr: EventAttribute) -> Result<Self, Self::Error> {
        Ok(Self {
            key: String::from_utf8(attr.key).map_err(|e| ChainError::ProtoDecoding {
                message: e.to_string(),
            })?,
            value: String::from_utf8(attr.value).map_err(|e| ChainError::ProtoDecoding {
                message: e.to_string(),
            })?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Coin {
    pub denom: String,
    pub amount: u64,
}

impl TryFrom<Coin> for cosmrs::Coin {
    type Error = ChainError;

    fn try_from(value: Coin) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: value.denom.parse().map_err(|_| ChainError::Denom {
                name: value.denom.clone(),
            })?,
            amount: value.amount.into(),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct GasInfo {
    pub gas_wanted: u64,
    pub gas_used: u64,
}

impl From<cosmos_sdk_proto::cosmos::base::abci::v1beta1::GasInfo> for GasInfo {
    fn from(info: cosmos_sdk_proto::cosmos::base::abci::v1beta1::GasInfo) -> GasInfo {
        GasInfo {
            gas_wanted: info.gas_wanted,
            gas_used: info.gas_used,
        }
    }
}
