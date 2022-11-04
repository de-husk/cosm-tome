use thiserror::Error;

use crate::{
    chain::error::ChainError,
    modules::{auth::error::AccountError, tx::error::TxError},
};

#[derive(Error, Debug)]
pub enum CosmwasmError {
    #[error("cannot serialize inputted msg as json")]
    JsonSerialize { source: serde_json::Error },

    // TODO: Store admin address and show it in the error string
    #[error("invalid admin address")]
    AdminAddress,

    #[error("unsupported instantiate permission AccessType: {i:?}")]
    AccessType { i: i32 },

    #[error(transparent)]
    TxError(#[from] TxError),

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
