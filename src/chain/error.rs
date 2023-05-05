use cosmrs::proto::prost::{DecodeError, EncodeError};
use cosmrs::ErrorReport;
use thiserror::Error;

#[cfg(feature = "os_keyring")]
pub use keyring::Error as KeyringError;

pub use cosmrs::rpc::Error as TendermintRPCError;
pub use cosmrs::tendermint::Error as TendermintError;
pub use tonic::transport::Error as CosmosGRPCError;

// use super::response::ChainResponse;

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("invalid denomination: {name:?}")]
    Denom { name: String },

    #[error("invalid chainId: {chain_id:?}")]
    ChainId { chain_id: String },

    #[error("api endpoint is not configured {api_type:?}")]
    MissingApiEndpoint { api_type: String },

    #[error("invalid mnemonic")]
    Mnemonic,

    #[error("invalid derivation path")]
    DerviationPath,

    #[error("cryptographic error: {message:?}")]
    Crypto { message: String },

    #[error("invalid query path url: {url:?}")]
    QueryPath { url: String },

    #[error("proto encoding error: {message:?}")]
    ProtoEncoding { message: String },

    #[error("proto decoding error: {message:?}")]
    ProtoDecoding { message: String },

    #[error("invalid cosmos msg sent to simulate endpoint")]
    Simulation,

    #[cfg(feature = "os_keyring")]
    #[error(transparent)]
    Keyring(#[from] KeyringError),

    #[error("Tonic error {0}")]
    Tonic(#[from] tonic::Status),

    // #[error("CosmosSDK error: {res:?}")]
    // CosmosSdk { res: ChainResponse },
    #[error("TxSync error: {res:?}")]
    TxCommit {
        res: tendermint_rpc::endpoint::broadcast::tx_commit::Response,
    },

    #[error("TxSync error: {res:?}")]
    TxSync {
        res: tendermint_rpc::endpoint::broadcast::tx_sync::Response,
    },

    #[error("TxAsync error: {res:?}")]
    TxAsync {
        res: tendermint_rpc::endpoint::broadcast::tx_async::Response,
    },

    #[error("Tendermint error")]
    Tendermint(#[from] TendermintError),

    /// Tendermint RPC client errors
    #[error(transparent)]
    RPC(#[from] TendermintRPCError),

    /// Cosmos gRPC client errors
    #[error(transparent)]
    GRPC(#[from] CosmosGRPCError),
}

impl ChainError {
    pub(crate) fn crypto(e: ErrorReport) -> ChainError {
        ChainError::Crypto {
            message: e.to_string(),
        }
    }

    pub(crate) fn proto_encoding(e: ErrorReport) -> ChainError {
        ChainError::ProtoEncoding {
            message: e.to_string(),
        }
    }

    pub(crate) fn prost_proto_encoding(e: EncodeError) -> ChainError {
        ChainError::ProtoEncoding {
            message: e.to_string(),
        }
    }

    pub(crate) fn prost_proto_decoding(e: DecodeError) -> ChainError {
        ChainError::ProtoDecoding {
            message: e.to_string(),
        }
    }

    // pub(crate) fn tonic_status(e: tonic::Status) -> ChainError {
    //     e.into()
    // }
}

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("Raw chain response is empty")]
    EmptyResponse,

    #[error(transparent)]
    Serde(#[from] serde_json::error::Error),
}
