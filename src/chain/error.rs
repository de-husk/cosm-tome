use cosmrs::ErrorReport;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("cryptographic error")]
    Crypto { source: ErrorReport },

    #[error("invalid denomination: {name:?}")]
    Denom { name: String },

    #[error("invalid chainId: {chain_id:?}")]
    ChainId { chain_id: String },

    #[error("invalid mnemonic")]
    Mnemonic,

    #[error("invalid derivation path")]
    DerviationPath,

    #[error("proto encoding error")]
    ProtoEncoding { source: ErrorReport },

    #[error("proto decoding error")]
    ProtoDecoding { source: ErrorReport },

    #[error(transparent)]
    Keyring(#[from] keyring::Error),
}

impl ChainError {
    pub fn crypto(e: ErrorReport) -> ChainError {
        ChainError::Crypto { source: e }
    }

    pub fn proto_encoding(e: ErrorReport) -> ChainError {
        ChainError::ProtoEncoding { source: e }
    }
}

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("Raw tendermint response is empty")]
    EmptyResponse,

    #[error(transparent)]
    Serde(#[from] serde_json::error::Error),
}
