use std::{fmt, num::ParseIntError, str::FromStr};

use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::error::ChainError;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
pub struct Coin {
    pub denom: Denom,
    pub amount: u128,
}

impl fmt::Display for Coin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.denom)
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

impl TryFrom<cosmrs::proto::cosmos::base::v1beta1::Coin> for Coin {
    type Error = ChainError;

    fn try_from(coin: cosmrs::proto::cosmos::base::v1beta1::Coin) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: coin.denom.parse()?,
            amount: coin
                .amount
                .parse()
                .map_err(|e: ParseIntError| ChainError::ProtoDecoding {
                    message: e.to_string(),
                })?,
        })
    }
}

impl From<Coin> for cosmrs::proto::cosmos::base::v1beta1::Coin {
    fn from(coin: Coin) -> Self {
        Self {
            denom: coin.denom.into(),
            amount: coin.amount.to_string(),
        }
    }
}

#[derive(
    Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, PartialOrd, Ord, Hash,
)]
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
            Ok(Denom(s.to_owned()))
        } else {
            Err(ChainError::Denom { name: s.to_owned() })
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

impl From<Denom> for String {
    fn from(d: Denom) -> Self {
        d.0
    }
}
