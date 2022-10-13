use cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount;
use cosmrs::crypto::PublicKey;
use serde::{Deserialize, Serialize};

use crate::chain::{error::ChainError, model::PaginationResponse};

use super::error::AccountError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Account {
    /// Bech32 address of account
    pub address: String,

    pub pubkey: Option<PublicKey>, // TODO: Make my own type here

    pub account_number: u64,

    pub sequence: u64,
}

impl TryFrom<BaseAccount> for Account {
    type Error = AccountError;
    fn try_from(proto: BaseAccount) -> Result<Account, Self::Error> {
        Ok(Account {
            // NOTE: we are unwrapping an `std::convert::Infallible` error here
            address: proto.address.parse().unwrap(),
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountResponse {
    pub account: Account,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,

    pub next: Option<PaginationResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParamsResponse {
    pub params: Option<Params>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub max_memo_characters: u64,
    pub tx_sig_limit: u64,
    pub tx_size_cost_per_byte: u64,
    pub sig_verify_cost_ed25519: u64,
    pub sig_verify_cost_secp256k1: u64,
}

impl From<cosmos_sdk_proto::cosmos::auth::v1beta1::Params> for Params {
    fn from(p: cosmos_sdk_proto::cosmos::auth::v1beta1::Params) -> Params {
        Params {
            max_memo_characters: p.max_memo_characters,
            tx_sig_limit: p.tx_sig_limit,
            tx_size_cost_per_byte: p.tx_size_cost_per_byte,
            sig_verify_cost_ed25519: p.sig_verify_cost_ed25519,
            sig_verify_cost_secp256k1: p.sig_verify_cost_secp256k1,
        }
    }
}
