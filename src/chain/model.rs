use std::str::FromStr;

use cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxResponse;
use cosmrs::tendermint::abci::{
    tag::{Key, Tag, Value},
    Code, Event,
};
use serde::Deserialize;
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
    pub events: Vec<Event>, // TODO: Make my own type here instead of exposing tendermint lib
    pub gas_wanted: u64,
    pub gas_used: u64,
    pub tx_hash: String,
    pub height: u64,
}

impl From<tx_commit::Response> for ChainTxResponse {
    fn from(res: tx_commit::Response) -> ChainTxResponse {
        ChainTxResponse {
            events: res.deliver_tx.events.clone(),
            gas_used: res.deliver_tx.gas_used.into(),
            gas_wanted: res.deliver_tx.gas_wanted.into(),
            res: res.deliver_tx.into(),
            tx_hash: res.hash.to_string(),
            height: res.height.into(),
        }
    }
}

impl From<TxResponse> for ChainTxResponse {
    fn from(res: TxResponse) -> ChainTxResponse {
        ChainTxResponse {
            res: ChainResponse {
                code: res.code.into(),
                data: Some(res.data.into()), // TODO
                log: res.raw_log,
            },
            // TODO: I need to make my own `Event` type, so that I dont leak external libs to users
            // AND so that I can just call `.into()` here (you cant define a From impl for 2 external crates)
            events: res
                .events
                .into_iter()
                .map(|e| Event {
                    type_str: e.r#type,
                    attributes: e
                        .attributes
                        .into_iter()
                        .map(|a| Tag {
                            key: Key::from_str(std::str::from_utf8(a.key.as_slice()).unwrap())
                                .unwrap(),
                            value: Value::from_str(
                                std::str::from_utf8(a.value.as_slice()).unwrap(),
                            )
                            .unwrap(),
                        })
                        .collect(),
                })
                .collect(),
            gas_wanted: res.gas_wanted as u64,
            gas_used: res.gas_used as u64,
            tx_hash: res.txhash,
            height: res.height as u64,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Coin {
    pub denom: String,
    pub amount: u64,
}

impl TryFrom<Coin> for cosmrs::Coin {
    type Error = ChainError;

    fn try_from(value: Coin) -> Result<Self, ChainError> {
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
