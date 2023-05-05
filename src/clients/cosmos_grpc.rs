use async_trait::async_trait;
use cosmrs::proto::cosmos::base::abci::v1beta1::TxResponse;
use cosmrs::proto::cosmos::tx::v1beta1::service_client::ServiceClient;
use cosmrs::proto::cosmos::tx::v1beta1::{BroadcastTxRequest, SimulateRequest};
use cosmrs::proto::traits::Message;
use tendermint_rpc::endpoint::broadcast::{tx_async, tx_commit, tx_sync};
use tonic::codec::ProstCodec;

use crate::chain::error::ChainError;
use crate::chain::fee::GasInfo;
use crate::modules::tx::model::{BroadcastMode, RawTx};

use super::client::{CosmosClientAsync, CosmosClientCommit, CosmosClientQuery, CosmosClientSync};

#[derive(Clone, Debug)]
pub struct CosmosgRPC {
    grpc_endpoint: String,
}

impl CosmosgRPC {
    pub fn new(grpc_endpoint: String) -> Self {
        Self { grpc_endpoint }
    }

    // Uses underlying grpc client to make calls to any gRPC service
    // without having to use the tonic generated clients for each cosmos module
    async fn grpc_call<I, O>(
        &self,
        req: impl tonic::IntoRequest<I>,
        path: &str,
    ) -> Result<O, ChainError>
    where
        I: Message + 'static,
        O: Message + Default + 'static,
    {
        let conn = tonic::transport::Endpoint::new(self.grpc_endpoint.clone())?
            .connect()
            .await?;

        let mut client = tonic::client::Grpc::new(conn);

        client.ready().await?;

        // NOTE: `I` and `O` in ProstCodec have static lifetime bounds:
        let codec: ProstCodec<I, O> = tonic::codec::ProstCodec::default();
        let res = client
            .unary(
                req.into_request(),
                path.parse().map_err(|_| ChainError::QueryPath {
                    url: path.to_string(),
                })?,
                codec,
            )
            .await
            .map_err(ChainError::tonic_status)?;

        Ok(res.into_inner())
    }
}

#[async_trait]
impl CosmosClientQuery for CosmosgRPC {
    async fn query<I, O>(&self, msg: I, path: &str) -> Result<O, ChainError>
    where
        I: Message + Default + tonic::IntoRequest<I> + 'static,
        O: Message + Default + 'static,
    {
        let res = self.grpc_call::<I, O>(msg, path).await?;

        Ok(res)
    }

    #[allow(deprecated)]
    async fn simulate_tx(&self, tx: &RawTx) -> Result<GasInfo, ChainError> {
        let mut client = ServiceClient::connect(self.grpc_endpoint.clone()).await?;

        let req = SimulateRequest {
            tx: None,
            tx_bytes: tx.to_bytes()?,
        };

        let gas_info = client
            .simulate(req)
            .await?
            .into_inner()
            .gas_info
            .ok_or(ChainError::Simulation)?;

        Ok(gas_info.into())
    }
}

#[async_trait]
impl CosmosClientCommit for CosmosgRPC {
    // type BlockTxResponse = BroadcastTxResponse;
    async fn broadcast_tx_commit(&self, tx: &RawTx) -> Result<tx_commit::Response, ChainError> {
        let mut client = ServiceClient::connect(self.grpc_endpoint.clone()).await?;

        let req = BroadcastTxRequest {
            tx_bytes: tx.to_bytes()?,
            mode: BroadcastMode::Block as i32,
        };

        let res = client
            .broadcast_tx(req)
            .await
            .map_err(ChainError::tonic_status)?
            .into_inner();

        Ok(res)
    }
}

#[async_trait]
impl CosmosClientSync for CosmosgRPC {
    async fn broadcast_tx_sync(&self, tx: &RawTx) -> Result<tx_sync::Response, ChainError> {
        let mut client = ServiceClient::connect(self.grpc_endpoint.clone()).await?;

        let req = BroadcastTxRequest {
            tx_bytes: tx.to_bytes()?,
            mode: BroadcastMode::Sync as i32,
        };

        let res = client
            .broadcast_tx(req)
            .await
            .map_err(ChainError::tonic_status)?
            .into_inner();

        let res = res.tx_response.unwrap();

        Ok(res)
    }
}

#[async_trait]
impl CosmosClientAsync for CosmosgRPC {
    async fn broadcast_tx_async(&self, tx: &RawTx) -> Result<tx_async::Response, ChainError> {
        let mut client = ServiceClient::connect(self.grpc_endpoint.clone()).await?;

        let req = BroadcastTxRequest {
            tx_bytes: tx.to_bytes()?,
            mode: BroadcastMode::Async as i32,
        };

        let res = client
            .broadcast_tx(req)
            .await
            .map_err(ChainError::tonic_status)?
            .into_inner();

        let res = res.tx_response.unwrap();

        Ok(res)
    }
}
