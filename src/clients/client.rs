use async_trait::async_trait;
use cosmos_sdk_proto::traits::Message;
use cosmrs::tx::Raw;

use crate::chain::error::ChainError;
use crate::chain::fee::GasInfo;
use crate::chain::response::ChainTxResponse;
use crate::config::config::ChainConfig;

use super::cosmos_grpc::CosmosgRPC;
use super::tendermint_rpc::TendermintRPC;

#[async_trait]
pub trait CosmosClient {
    async fn query<T, I, O>(&self, msg: T, path: &str) -> Result<O, ChainError>
    where
        T: Message + Default + tonic::IntoRequest<I>,
        I: Message + 'static,
        O: Message + Default + 'static;

    async fn simulate_tx(&self, tx: &Raw) -> Result<GasInfo, ChainError>;

    async fn broadcast_tx(&self, tx: &Raw) -> Result<ChainTxResponse, ChainError>;
}

#[derive(Clone, Debug)]
pub struct CosmTome<T: CosmosClient> {
    pub(crate) cfg: ChainConfig,
    pub(crate) client: T,
}

impl<T: CosmosClient> CosmTome<T> {
    /// General usage CosmClient constructor accepting any client that impls `CosmosClient` trait
    pub fn new(cfg: ChainConfig, client: T) -> Self {
        Self { cfg, client }
    }

    pub fn with_tendermint_rpc(cfg: ChainConfig) -> Result<CosmTome<TendermintRPC>, ChainError> {
        Ok(CosmTome {
            client: TendermintRPC::new(&cfg.rpc_endpoint.clone())?,
            cfg,
        })
    }

    pub fn with_cosmos_grpc(cfg: ChainConfig) -> Result<CosmTome<CosmosgRPC>, ChainError> {
        Ok(CosmTome {
            client: CosmosgRPC::new(cfg.grpc_endpoint.clone()),
            cfg,
        })
    }
}
