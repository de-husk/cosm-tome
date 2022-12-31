use std::{fmt, str::FromStr};

use cosmrs::proto::cosmos::auth::v1beta1::{BaseAccount, Params as CosmosParams};
use cosmrs::{crypto::PublicKey, AccountId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::chain::{error::ChainError, request::PaginationResponse};

use super::error::AccountError;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Address(AccountId);

impl Address {
    pub fn new(prefix: &str, bytes: &[u8]) -> Result<Self, AccountError> {
        let account_id =
            AccountId::new(prefix, bytes).map_err(|e| AccountError::AccountIdParse {
                message: e.to_string(),
            })?;

        Ok(Self(account_id))
    }

    pub fn prefix(&self) -> &str {
        self.0.prefix()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Address {
    type Err = AccountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Address(AccountId::from_str(s).map_err(|e| {
            AccountError::Address {
                message: e.to_string(),
            }
        })?))
    }
}

impl From<AccountId> for Address {
    fn from(account: AccountId) -> Address {
        Address(account)
    }
}

impl From<Address> for AccountId {
    fn from(account: Address) -> AccountId {
        account.0
    }
}

impl From<Address> for String {
    fn from(address: Address) -> Self {
        address.0.into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Account {
    /// Bech32 address of account
    pub address: Address,

    pub pubkey: Option<PublicKey>,

    pub account_number: u64,

    pub sequence: u64,
}

impl TryFrom<BaseAccount> for Account {
    type Error = AccountError;

    fn try_from(proto: BaseAccount) -> Result<Self, Self::Error> {
        Ok(Account {
            address: proto.address.parse()?,
            pubkey: proto
                .pub_key
                .map(PublicKey::try_from)
                .transpose()
                .map_err(ChainError::crypto)?,
            account_number: proto.account_number,
            sequence: proto.sequence,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountResponse {
    pub account: Account,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,

    pub next: Option<PaginationResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
pub struct ParamsResponse {
    pub params: Option<Params>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
pub struct Params {
    pub max_memo_characters: u64,
    pub tx_sig_limit: u64,
    pub tx_size_cost_per_byte: u64,
    pub sig_verify_cost_ed25519: u64,
    pub sig_verify_cost_secp256k1: u64,
}

impl From<CosmosParams> for Params {
    fn from(p: CosmosParams) -> Self {
        Self {
            max_memo_characters: p.max_memo_characters,
            tx_sig_limit: p.tx_sig_limit,
            tx_size_cost_per_byte: p.tx_size_cost_per_byte,
            sig_verify_cost_ed25519: p.sig_verify_cost_ed25519,
            sig_verify_cost_secp256k1: p.sig_verify_cost_secp256k1,
        }
    }
}

impl From<Params> for CosmosParams {
    fn from(p: Params) -> Self {
        Self {
            max_memo_characters: p.max_memo_characters,
            tx_sig_limit: p.tx_sig_limit,
            tx_size_cost_per_byte: p.tx_size_cost_per_byte,
            sig_verify_cost_ed25519: p.sig_verify_cost_ed25519,
            sig_verify_cost_secp256k1: p.sig_verify_cost_secp256k1,
        }
    }
}
