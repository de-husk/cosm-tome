use cosmos_sdk_proto::prost::{DecodeError, EncodeError};
use cosmrs::ErrorReport;
use thiserror::Error;

use super::response::ChainResponse;

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

    #[error("cryptographic error")]
    Crypto { message: String },

    #[error("invalid query path url: {url:?}")]
    QueryPath { url: String },

    #[error("proto encoding error: {message:?}")] // TODO: Do this for all string error messages
    ProtoEncoding { message: String },

    #[error("proto decoding error")]
    ProtoDecoding { message: String },

    #[error("invalid cosmos msg sent to simulate endpoint")]
    Simulation,

    #[error(transparent)]
    Keyring(#[from] keyring::Error),

    #[error("CosmosSDK error: {res:?}")]
    CosmosSdk { res: ChainResponse },

    // TODO: Stop exposing both of these since they are literally just the same rexported error
    #[error(transparent)]
    RPC(#[from] tendermint_rpc::Error),
    #[error(transparent)]
    RPCError(#[from] cosmrs::tendermint::Error),

    #[error(transparent)]
    GRPC(#[from] tonic::Status),
    #[error(transparent)]
    GRPCConnection(#[from] tonic::transport::Error),
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
}

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("Raw chain response is empty")]
    EmptyResponse,

    #[error(transparent)]
    Serde(#[from] serde_json::error::Error),
}
