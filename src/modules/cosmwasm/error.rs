use cosmrs::ErrorReport;
use thiserror::Error;

use crate::{chain::error::ChainError, modules::auth::error::AccountError};

#[derive(Error, Debug)]
pub enum CosmwasmError {
    #[error("cannot serialize inputted msg as json")]
    JsonSerialize { source: serde_json::Error },

    // TODO: Store admin address and show it in the error string
    #[error("invalid admin address")]
    AdminAddress,

    #[error("invalid contract address: {addr:?}")]
    ContractAddress { addr: String },

    #[error("invalid instantiate permissions")]
    InstantiatePerms { source: ErrorReport },

    #[error(transparent)]
    AccountError(#[from] AccountError),

    #[error("missing event from chain response")]
    MissingEvent,

    #[error(transparent)]
    ChainError(#[from] ChainError),
}

impl CosmwasmError {
    pub(crate) fn json(e: serde_json::Error) -> CosmwasmError {
        CosmwasmError::JsonSerialize { source: e }
    }
}
