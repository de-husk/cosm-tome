use std::fmt;

use cosmrs::proto::cosmos::bank::v1beta1::{
    DenomUnit as ProtoDenomUnit, Metadata, MsgSend, Params as ProtoParams,
    SendEnabled as ProtoSendEnabled,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::chain::coin::Denom;
use crate::chain::msg::Msg;
use crate::{
    chain::{coin::Coin, error::ChainError, request::PaginationResponse},
    modules::auth::model::Address,
};

use super::error::BankError;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct BalanceResponse {
    pub balance: Coin,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct BalancesResponse {
    pub balances: Vec<Coin>,

    pub next: Option<PaginationResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct DenomMetadataResponse {
    pub meta: Option<DenomMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct DenomsMetadataResponse {
    pub metas: Vec<DenomMetadata>,

    pub next: Option<PaginationResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
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
    pub uri: String,
    pub uri_hash: String,
}

impl TryFrom<Metadata> for DenomMetadata {
    type Error = ChainError;

    fn try_from(meta: Metadata) -> Result<Self, Self::Error> {
        Ok(Self {
            description: meta.description,
            denom_units: meta
                .denom_units
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            base: meta.base,
            display: meta.display,
            name: meta.name,
            symbol: meta.symbol,
            uri: meta.uri,
            uri_hash: meta.uri_hash,
        })
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
            uri: meta.uri,
            uri_hash: meta.uri_hash,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct DenomUnit {
    /// denom represents the string name of the given denom unit (e.g uatom).
    pub denom: Denom,

    /// exponent represents the power of 10 exponent that one must raise the base_denom to in order to equal the given DenomUnit's denom.
    /// 1 denom = 1^exponent base_denom
    /// (e.g. with a base_denom of uatom, one can create a DenomUnit of 'atom' with exponent = 6, thus: 1 atom = 10^6 uatom).
    pub exponent: u32,

    /// aliases is a list of string aliases for the given denom
    pub aliases: Vec<String>,
}

impl TryFrom<ProtoDenomUnit> for DenomUnit {
    type Error = ChainError;

    fn try_from(du: ProtoDenomUnit) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: du.denom.parse()?,
            exponent: du.exponent,
            aliases: du.aliases,
        })
    }
}

impl From<DenomUnit> for ProtoDenomUnit {
    fn from(du: DenomUnit) -> Self {
        Self {
            denom: du.denom.into(),
            exponent: du.exponent,
            aliases: du.aliases,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
pub struct ParamsResponse {
    pub params: Option<Params>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
pub struct Params {
    pub send_enabled: Vec<SendEnabled>,
    pub default_send_enabled: bool,
}

impl TryFrom<ProtoParams> for Params {
    type Error = ChainError;

    fn try_from(p: ProtoParams) -> Result<Self, Self::Error> {
        Ok(Self {
            send_enabled: p
                .send_enabled
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            default_send_enabled: p.default_send_enabled,
        })
    }
}

impl From<Params> for ProtoParams {
    fn from(p: Params) -> Self {
        Self {
            send_enabled: p.send_enabled.into_iter().map(Into::into).collect(),
            default_send_enabled: p.default_send_enabled,
        }
    }
}

/// SendEnabled maps coin denom to a send_enabled status (whether a denom is sendable).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
pub struct SendEnabled {
    pub denom: Denom,
    pub enabled: bool,
}

impl TryFrom<ProtoSendEnabled> for SendEnabled {
    type Error = ChainError;

    fn try_from(se: ProtoSendEnabled) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: se.denom.parse()?,
            enabled: se.enabled,
        })
    }
}

impl From<SendEnabled> for ProtoSendEnabled {
    fn from(se: SendEnabled) -> Self {
        Self {
            denom: se.denom.into(),
            enabled: se.enabled,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SendRequest {
    pub from: Address,
    pub to: Address,
    pub amounts: Vec<Coin>,
}

pub type SendRequestProto = SendRequest;

impl fmt::Display for SendRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} sends ", self.from)?;

        for a in &self.amounts {
            write!(f, "{a} ")?;
        }

        write!(f, "-> {}", self.to)
    }
}

impl Msg for SendRequestProto {
    type Proto = MsgSend;
    type Err = BankError;
}

impl TryFrom<MsgSend> for SendRequest {
    type Error = BankError;

    fn try_from(msg: MsgSend) -> Result<Self, Self::Error> {
        Ok(Self {
            from: msg.from_address.parse()?,
            to: msg.to_address.parse()?,
            amounts: msg
                .amount
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<SendRequest> for MsgSend {
    type Error = BankError;

    fn try_from(req: SendRequest) -> Result<Self, Self::Error> {
        if req.amounts.is_empty() {
            return Err(BankError::EmptyAmount);
        }

        for amount in &req.amounts {
            if amount.amount == 0 {
                return Err(BankError::EmptyAmount);
            }
        }

        Ok(Self {
            from_address: req.from.into(),
            to_address: req.to.into(),
            amount: req.amounts.into_iter().map(Into::into).collect(),
        })
    }
}

// #[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
// pub struct SendResponse {
//     pub res: ChainTxResponse,
// }
