use async_trait::async_trait;
use cosmos_sdk_proto::traits::Message;

use crate::chain::error::ChainError;
use crate::chain::fee::GasInfo;
use crate::chain::response::{AsyncChainTxResponse, ChainTxResponse};
use crate::config::cfg::ChainConfig;
use crate::modules::tx::model::{BroadcastMode, RawTx};

use super::cosmos_grpc::CosmosgRPC;
use super::tendermint_rpc::TendermintRPC;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CosmosClient {
    async fn query<I, O>(&self, msg: I, path: &str) -> Result<O, ChainError>
    where
        I: Message + Default + tonic::IntoRequest<I> + 'static,
        O: Message + Default + 'static;

    async fn simulate_tx(&self, tx: &RawTx) -> Result<GasInfo, ChainError>;

    async fn broadcast_tx(
        &self,
        tx: &RawTx,
        mode: BroadcastMode,
    ) -> Result<AsyncChainTxResponse, ChainError>;

    /// TODO: Block BroadcastMode support is being dropped from future Cosmos-Sdk versions.
    /// Cosm-tome will continue to support it by broadcasting with the Sync mode
    /// and then polling the GetTransaction endpoint until it has been committed in the block.
    async fn broadcast_tx_block(&self, tx: &RawTx) -> Result<ChainTxResponse, ChainError>;
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
