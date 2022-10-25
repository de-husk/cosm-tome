use thiserror::Error;

use crate::{chain::error::ChainError, modules::auth::error::AccountError};

#[derive(Error, Debug)]
pub enum BankError {
    #[error("Cannot send 0 amount of a token")]
    EmptyAmount,

    #[error(transparent)]
    AccountError(#[from] AccountError),

    #[error(transparent)]
    ChainError(#[from] ChainError),
}
