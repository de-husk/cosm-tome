use crate::chain::fee::GasInfo;
use crate::{chain::error::ChainError, modules::tx::model::RawTx};
use async_trait::async_trait;
use cosmrs::proto::traits::Message;

#[cfg(feature = "mocks")]
use mockall::automock;
use tendermint_rpc::endpoint::broadcast::{tx_async, tx_commit, tx_sync};

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait CosmosClientQuery {
    async fn query<I, O>(&self, msg: I, path: &str) -> Result<O, ChainError>
    where
        Self: Sized,
        I: Message + Default + tonic::IntoRequest<I> + 'static,
        O: Message + Default + 'static;

    async fn simulate_tx(&self, tx: &RawTx) -> Result<GasInfo, ChainError>;
}

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait CosmosClientCommit {
    async fn broadcast_tx_commit(&self, tx: &RawTx) -> Result<tx_commit::Response, ChainError>;
}

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait CosmosClientSync {
    async fn broadcast_tx_sync(&self, tx: &RawTx) -> Result<tx_sync::Response, ChainError>;
}

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait CosmosClientAsync {
    async fn broadcast_tx_async(&self, tx: &RawTx) -> Result<tx_async::Response, ChainError>;
}

// #[cfg(test)]
// mod tests {
//     use super::CosmosClient;

//     const _MESSAGE_IS_OBJECT_SAFE: Option<
//         &dyn CosmosClient<TxResponse = (), AsyncTxResponse = ()>,
//     > = None;
// }
