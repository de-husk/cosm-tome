use cosmrs::ErrorReport;
use thiserror::Error;

use crate::{chain::error::ChainError, modules::auth::error::AccountError};

#[derive(Error, Debug)]
pub enum CosmwasmError {
    #[error("serde json serialization error")]
    JsonSerialize { source: serde_json::Error },

    // TODO: Store admin address and show it in the error string
    #[error("invalid admin address")]
    AdminAddress,

    #[error("invalid instantiate permissions")]
    InstantiatePerms { source: ErrorReport },

    #[error(transparent)]
    AccountError(#[from] AccountError),

    #[error(transparent)]
    ChainError(#[from] ChainError),
}

impl CosmwasmError {
    pub fn json(e: serde_json::Error) -> CosmwasmError {
        CosmwasmError::JsonSerialize { source: e }
    }
}
