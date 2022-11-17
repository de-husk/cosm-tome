use thiserror::Error;

use crate::{
    chain::error::ChainError,
    modules::{auth::error::AccountError, tx::error::TxError},
};

pub use serde_json::Error as SerdeJsonError;

#[derive(Error, Debug)]
pub enum CosmwasmError {
    #[error("cannot serialize inputted msg as json")]
    JsonSerialize { source: SerdeJsonError },

    #[error("unsupported instantiate permission AccessType: {i:?}")]
    AccessType { i: i32 },

    #[error("missing event from chain response")]
    MissingEvent,

    #[error(transparent)]
    TxError(#[from] TxError),

    #[error(transparent)]
    AccountError(#[from] AccountError),

    #[error(transparent)]
    ChainError(#[from] ChainError),
}

impl CosmwasmError {
    pub(crate) fn json(e: serde_json::Error) -> CosmwasmError {
        CosmwasmError::JsonSerialize { source: e }
    }
}
