use cosmrs::proto::cosmos::base::abci::v1beta1::GasInfo as ProtoGasInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::{coin::Coin, error::ChainError};
use crate::modules::auth::model::Address;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Fee {
    pub amount: Vec<Coin>,

    pub gas_limit: Gas,

    pub payer: Option<Address>,

    pub granter: Option<Address>,
}

impl Fee {
    pub fn new(
        amount: Coin,
        gas_limit: impl Into<Gas>,
        payer: Option<Address>,
        granter: Option<Address>,
    ) -> Self {
        Self {
            amount: vec![amount],
            gas_limit: gas_limit.into(),
            payer,
            granter,
        }
    }
}

impl TryFrom<cosmrs::tx::Fee> for Fee {
    type Error = ChainError;

    fn try_from(fee: cosmrs::tx::Fee) -> Result<Self, Self::Error> {
        Ok(Fee {
            amount: fee
                .amount
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            gas_limit: fee.gas_limit.into(),
            payer: fee.payer.map(Into::into),
            granter: fee.granter.map(Into::into),
        })
    }
}

impl TryFrom<Fee> for cosmrs::tx::Fee {
    type Error = ChainError;

    fn try_from(fee: Fee) -> Result<Self, Self::Error> {
        Ok(Self {
            amount: fee
                .amount
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            gas_limit: fee.gas_limit.into(),
            payer: fee.payer.map(Into::into),
            granter: fee.granter.map(Into::into),
        })
    }
}

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Default,
    Hash,
)]
pub struct Gas(u64);

impl Gas {
    pub fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Gas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} gas", self.0)
    }
}

impl From<u64> for Gas {
    fn from(g: u64) -> Gas {
        Gas(g)
    }
}

impl From<Gas> for u64 {
    fn from(g: Gas) -> u64 {
        g.0
    }
}

impl From<u32> for Gas {
    fn from(g: u32) -> Gas {
        Gas(g.into())
    }
}

impl From<u16> for Gas {
    fn from(g: u16) -> Gas {
        Gas(g.into())
    }
}

impl From<u8> for Gas {
    fn from(g: u8) -> Gas {
        Gas(g.into())
    }
}

#[derive(
    Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, PartialOrd, Ord, Default, Hash,
)]
pub struct GasInfo {
    pub gas_wanted: Gas,
    pub gas_used: Gas,
}

impl fmt::Display for GasInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wanted: {} used: {}", self.gas_wanted, self.gas_used)
    }
}

impl GasInfo {
    pub fn new(gas_wanted: impl Into<Gas>, gas_used: impl Into<Gas>) -> GasInfo {
        GasInfo {
            gas_wanted: gas_wanted.into(),
            gas_used: gas_used.into(),
        }
    }
}

impl From<ProtoGasInfo> for GasInfo {
    fn from(info: ProtoGasInfo) -> GasInfo {
        GasInfo {
            gas_wanted: info.gas_wanted.into(),
            gas_used: info.gas_used.into(),
        }
    }
}

impl From<GasInfo> for ProtoGasInfo {
    fn from(info: GasInfo) -> ProtoGasInfo {
        ProtoGasInfo {
            gas_wanted: info.gas_wanted.into(),
            gas_used: info.gas_used.into(),
        }
    }
}
