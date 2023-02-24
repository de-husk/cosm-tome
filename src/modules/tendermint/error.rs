use thiserror::Error;

use crate::{chain::error::ChainError, modules::auth::error::AccountError};

#[derive(Error, Debug)]
pub enum TendermintError {
    #[error("block missing from tendermint response")]
    MissingBlock,

    #[error("blockId missing from tendermint response")]
    MissingBlockId,

    #[error(transparent)]
    AccountError(#[from] AccountError),

    #[error(transparent)]
    ChainError(#[from] ChainError),
}
