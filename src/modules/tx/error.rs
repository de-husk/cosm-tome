use thiserror::Error;

use crate::{chain::error::ChainError, modules::auth::error::AccountError};

#[derive(Error, Debug)]
pub enum TxError {
    #[error("unsupported BroadcastMode: {i:?}")]
    BroadcastMode { i: i32 },

    #[error(transparent)]
    AccountError(#[from] AccountError),

    #[error(transparent)]
    ChainError(#[from] ChainError),
}
