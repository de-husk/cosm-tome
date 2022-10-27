use cosmos_sdk_proto::cosmos::{self, bank::v1beta1::Metadata};
use serde::{Deserialize, Serialize};

use crate::chain::{coin::Coin, request::PaginationResponse, response::ChainTxResponse};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct BalanceResponse {
    pub balance: Coin,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct BalancesResponse {
    pub balances: Vec<Coin>,

    pub next: Option<PaginationResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DenomMetadataResponse {
    pub meta: Option<DenomMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DenomsMetadataResponse {
    pub metas: Vec<DenomMetadata>,

    pub next: Option<PaginationResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DenomMetadata {
    pub description: String,

    pub denom_units: Vec<DenomUnit>,

    /// base represents the base denom (should be the DenomUnit with exponent = 0).
    pub base: String,

    /// display indicates the suggested denom string that should be displayed in clients.
    pub display: String,

    /// name defines the name of the token (eg: Cosmos Atom)
    ///
    /// Since: cosmos-sdk 0.43
    pub name: String,

    /// symbol is the token symbol usually shown on exchanges (eg: ATOM).
    /// This can be the same as the display.
    ///
    /// Since: cosmos-sdk 0.43
    pub symbol: String,
}

impl From<Metadata> for DenomMetadata {
    fn from(meta: Metadata) -> Self {
        Self {
            description: meta.description,
            denom_units: meta.denom_units.into_iter().map(Into::into).collect(),
            base: meta.base,
            display: meta.display,
            name: meta.name,
            symbol: meta.symbol,
        }
    }
}

impl From<DenomMetadata> for Metadata {
    fn from(meta: DenomMetadata) -> Self {
        Self {
            description: meta.description,
            denom_units: meta.denom_units.into_iter().map(Into::into).collect(),
            base: meta.base,
            display: meta.display,
            name: meta.name,
            symbol: meta.symbol,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DenomUnit {
    /// denom represents the string name of the given denom unit (e.g uatom).
    pub denom: String, // TODO: Use my Denom type instead

    /// exponent represents the power of 10 exponent that one must raise the base_denom to in order to equal the given DenomUnit's denom.
    /// 1 denom = 1^exponent base_denom
    /// (e.g. with a base_denom of uatom, one can create a DenomUnit of 'atom' with exponent = 6, thus: 1 atom = 10^6 uatom).
    pub exponent: u32,

    /// aliases is a list of string aliases for the given denom
    pub aliases: Vec<String>,
}

impl From<cosmos::bank::v1beta1::DenomUnit> for DenomUnit {
    fn from(du: cosmos::bank::v1beta1::DenomUnit) -> Self {
        Self {
            denom: du.denom,
            exponent: du.exponent,
            aliases: du.aliases,
        }
    }
}

impl From<DenomUnit> for cosmos::bank::v1beta1::DenomUnit {
    fn from(du: DenomUnit) -> Self {
        Self {
            denom: du.denom,
            exponent: du.exponent,
            aliases: du.aliases,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ParamsResponse {
    pub params: Option<Params>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Params {
    pub send_enabled: Vec<SendEnabled>,
    pub default_send_enabled: bool,
}

impl From<cosmos::bank::v1beta1::Params> for Params {
    fn from(p: cosmos::bank::v1beta1::Params) -> Self {
        Self {
            send_enabled: p.send_enabled.into_iter().map(Into::into).collect(),
            default_send_enabled: p.default_send_enabled,
        }
    }
}

impl From<Params> for cosmos::bank::v1beta1::Params {
    fn from(p: Params) -> Self {
        Self {
            send_enabled: p.send_enabled.into_iter().map(Into::into).collect(),
            default_send_enabled: p.default_send_enabled,
        }
    }
}

/// SendEnabled maps coin denom to a send_enabled status (whether a denom is sendable).
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SendEnabled {
    pub denom: String, // TODO: Use my Denom type instead
    pub enabled: bool,
}

impl From<cosmos::bank::v1beta1::SendEnabled> for SendEnabled {
    fn from(se: cosmos::bank::v1beta1::SendEnabled) -> Self {
        Self {
            denom: se.denom,
            enabled: se.enabled,
        }
    }
}

impl From<SendEnabled> for cosmos::bank::v1beta1::SendEnabled {
    fn from(se: SendEnabled) -> Self {
        Self {
            denom: se.denom,
            enabled: se.enabled,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SendResponse {
    pub res: ChainTxResponse,
}
