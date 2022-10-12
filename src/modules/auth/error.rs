use thiserror::Error;

use crate::chain::error::ChainError;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("invalid account ID: {id:?}")]
    AccountId { id: String },

    #[error(transparent)]
    ChainError(#[from] ChainError),
}
