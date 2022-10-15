use cosmos_sdk_proto::cosmos::base::abci::v1beta1::GasInfo as CosmosProtoGasInfo;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use super::error::ChainError;
use crate::modules::auth::model::AccountAddr;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Fee {
    pub amount: Vec<Coin>,

    pub gas_limit: Gas,

    pub payer: Option<AccountAddr>,

    pub granter: Option<AccountAddr>,
}

impl Fee {
    pub fn new(
        amount: Coin,
        gas_limit: impl Into<Gas>,
        payer: Option<AccountAddr>,
        granter: Option<AccountAddr>,
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

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Denom(String);

impl AsRef<str> for Denom {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Denom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for Denom {
    type Err = ChainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // using the default denom validation from cosmos-sdk:
        // https://github.com/cosmos/cosmos-sdk/blob/main/types/coin.go#L869
        let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9/:._-]{2,127}$").unwrap();

        if re.is_match(s) {
            Ok(Denom(s.to_string()))
        } else {
            Err(ChainError::Denom {
                name: s.to_string(),
            })
        }
    }
}

impl TryFrom<cosmrs::Denom> for Denom {
    type Error = ChainError;

    fn try_from(d: cosmrs::Denom) -> Result<Self, Self::Error> {
        d.as_ref().parse()
    }
}

impl TryFrom<Denom> for cosmrs::Denom {
    type Error = ChainError;

    fn try_from(d: Denom) -> Result<Self, Self::Error> {
        d.0.parse().map_err(|_| ChainError::Denom { name: d.0 })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Coin {
    pub denom: Denom,
    pub amount: u128,
}

impl fmt::Display for Coin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.denom)
    }
}

impl TryFrom<Coin> for cosmrs::Coin {
    type Error = ChainError;

    fn try_from(coin: Coin) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: coin.denom.try_into()?,
            amount: coin.amount,
        })
    }
}

impl TryFrom<cosmrs::Coin> for Coin {
    type Error = ChainError;

    fn try_from(coin: cosmrs::Coin) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: coin.denom.try_into()?,
            amount: coin.amount,
        })
    }
}

#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Default, Hash,
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

impl From<cosmrs::tx::Gas> for Gas {
    fn from(g: cosmrs::tx::Gas) -> Gas {
        g.value().into()
    }
}

impl From<Gas> for cosmrs::tx::Gas {
    fn from(g: Gas) -> cosmrs::tx::Gas {
        g.value().into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default, Hash)]
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

impl From<CosmosProtoGasInfo> for GasInfo {
    fn from(info: CosmosProtoGasInfo) -> GasInfo {
        GasInfo {
            gas_wanted: info.gas_wanted.into(),
            gas_used: info.gas_used.into(),
        }
    }
}

impl From<GasInfo> for CosmosProtoGasInfo {
    fn from(info: GasInfo) -> CosmosProtoGasInfo {
        CosmosProtoGasInfo {
            gas_wanted: info.gas_wanted.into(),
            gas_used: info.gas_used.into(),
        }
    }
}
