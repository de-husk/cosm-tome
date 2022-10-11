use thiserror::Error;

use crate::chain::error::ChainError;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error(transparent)]
    ChainError(#[from] ChainError),
}
