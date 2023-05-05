use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmrs::proto::cosmos::base::query::v1beta1::{PageRequest, PageResponse};

use crate::modules::auth::model::Account;

use super::fee::Fee;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
pub struct PaginationRequest {
    pub page: PageID,
    pub limit: u64,
    pub reverse: bool,
}

impl From<PaginationRequest> for PageRequest {
    fn from(p: PaginationRequest) -> PageRequest {
        let (key, offset) = match p.page {
            PageID::Key(key) => (key, OffsetParams::default()),
            PageID::Offset(offset) => (vec![], offset),
        };

        PageRequest {
            key,
            offset: offset.offset,
            count_total: offset.count_total,
            limit: p.limit,
            reverse: p.reverse,
        }
    }
}

impl From<PageRequest> for PaginationRequest {
    fn from(p: PageRequest) -> PaginationRequest {
        let page = if p.key.is_empty() {
            PageID::Offset(OffsetParams {
                offset: p.offset,
                count_total: p.count_total,
            })
        } else {
            PageID::Key(p.key)
        };

        PaginationRequest {
            page,
            limit: p.limit,
            reverse: p.reverse,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
pub enum PageID {
    /// key is the value in PaginationResponse.next_key used to query the next page.
    Key(Vec<u8>),

    /// offset is a numeric offset that can be used when key is unavailable.
    /// It is less efficient than using key.
    Offset(OffsetParams),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash, Default)]
pub struct OffsetParams {
    pub offset: u64,
    pub count_total: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash, Default)]
pub struct PaginationResponse {
    pub next_key: Vec<u8>,
    pub total: u64,
}

impl From<PageResponse> for PaginationResponse {
    fn from(p: PageResponse) -> PaginationResponse {
        PaginationResponse {
            next_key: p.next_key,
            total: p.total,
        }
    }
}

impl From<PaginationResponse> for PageResponse {
    fn from(p: PaginationResponse) -> PageResponse {
        PageResponse {
            next_key: p.next_key,
            total: p.total,
        }
    }
}

/// Options the user can set when executing txs on chain
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TxOptions {
    /// The block height after which this transaction will not be processed by the chain
    pub timeout_height: Option<u64>,

    /// If set, this fee will be used, instead of simulating the fee
    pub fee: Option<Fee>,

    /// If set, this fee will be used, instead of querying the account
    pub account: Option<Account>,

    /// An arbitrary memo to be added to the transaction
    pub memo: String,
}

impl Default for TxOptions {
    fn default() -> Self {
        Self {
            fee: None,
            account: None,
            timeout_height: Some(0),
            memo: "Made with cosm-tome client".to_string(),
        }
    }
}
