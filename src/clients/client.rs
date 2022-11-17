use async_trait::async_trait;
use cosmrs::proto::traits::Message;

use crate::chain::error::ChainError;
use crate::chain::fee::GasInfo;
use crate::chain::response::{AsyncChainTxResponse, ChainTxResponse};
use crate::config::cfg::ChainConfig;
use crate::modules::tx::model::{BroadcastMode, RawTx};

use super::cosmos_grpc::CosmosgRPC;
use super::tendermint_rpc::TendermintRPC;

#[cfg(feature = "mocks")]
use mockall::automock;

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait CosmosClient {
    // NOTE: We can make `query()` dynamically dispatched and trait object usable
    // if prost fixes this: https://github.com/tokio-rs/prost/issues/742
    async fn query<I, O>(&self, msg: I, path: &str) -> Result<O, ChainError>
    where
        Self: Sized,
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
}

impl CosmTome<TendermintRPC> {
    pub fn with_tendermint_rpc(cfg: ChainConfig) -> Result<CosmTome<TendermintRPC>, ChainError> {
        let rpc_endpoint = cfg
            .rpc_endpoint
            .clone()
            .ok_or(ChainError::MissingApiEndpoint {
                api_type: "tendermint_rpc".to_string(),
            })?;

        Ok(CosmTome {
            client: TendermintRPC::new(&rpc_endpoint)?,
            cfg,
        })
    }
}

impl CosmTome<CosmosgRPC> {
    pub fn with_cosmos_grpc(cfg: ChainConfig) -> Result<CosmTome<CosmosgRPC>, ChainError> {
        let grpc_endpoint = cfg
            .grpc_endpoint
            .clone()
            .ok_or(ChainError::MissingApiEndpoint {
                api_type: "cosmos_grpc".to_string(),
            })?;

        Ok(CosmTome {
            client: CosmosgRPC::new(grpc_endpoint),
            cfg,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CosmosClient;

    const _MESSAGE_IS_OBJECT_SAFE: Option<&dyn CosmosClient> = None;
}
