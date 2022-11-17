use thiserror::Error;

use crate::chain::error::ChainError;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("invalid account Address: {message:?}")]
    Address { message: String },

    #[error("cannot parse account ID from bytes: {message:?}")]
    AccountIdParse { message: String },

    #[error(transparent)]
    ChainError(#[from] ChainError),
}
