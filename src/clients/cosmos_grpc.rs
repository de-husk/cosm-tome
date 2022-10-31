use async_trait::async_trait;
use cosmos_sdk_proto::cosmos::tx::v1beta1::service_client::ServiceClient;
use cosmos_sdk_proto::cosmos::tx::v1beta1::{BroadcastMode, BroadcastTxRequest, SimulateRequest};
use tonic::codec::ProstCodec;

use cosmos_sdk_proto::traits::Message;
use cosmrs::tx::Raw;

use crate::chain::fee::GasInfo;
use crate::chain::response::{ChainResponse, Code};
use crate::chain::{error::ChainError, response::ChainTxResponse};

use super::client::CosmosClient;

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
            .await;

        Ok(res?.into_inner())
    }
}

#[async_trait]
impl CosmosClient for CosmosgRPC {
    async fn query<T, I, O>(&self, msg: T, path: &str) -> Result<O, ChainError>
    where
        T: Message + Default + tonic::IntoRequest<I>,
        I: Message + 'static,
        O: Message + Default + 'static,
    {
        let res = self.grpc_call::<I, O>(msg, path).await?;

        Ok(res)
    }

    #[allow(deprecated)]
    async fn simulate_tx(&self, tx: &Raw) -> Result<GasInfo, ChainError> {
        let mut client = ServiceClient::connect(self.grpc_endpoint.clone()).await?;

        let req = SimulateRequest {
            tx: None,
            tx_bytes: tx.to_bytes().map_err(ChainError::proto_encoding)?,
        };

        let gas_info = client
            .simulate(req)
            .await
            .map_err(|e| ChainError::CosmosSdk {
                res: ChainResponse {
                    code: Code::Err(e.code() as u32),
                    log: e.message().to_string(),
                    ..Default::default()
                },
            })?
            .into_inner()
            .gas_info
            .ok_or(ChainError::Simulation)?;

        Ok(gas_info.into())
    }

    async fn broadcast_tx(&self, tx: &Raw) -> Result<ChainTxResponse, ChainError> {
        let mut client = ServiceClient::connect(self.grpc_endpoint.clone()).await?;

        let req = BroadcastTxRequest {
            tx_bytes: tx.to_bytes().map_err(ChainError::proto_encoding)?,
            mode: BroadcastMode::Block.into(),
        };

        let res = client.broadcast_tx(req).await?.into_inner();

        let res: ChainTxResponse = res.tx_response.unwrap().try_into()?;

        if res.res.code.is_err() {
            return Err(ChainError::CosmosSdk { res: res.res });
        }

        Ok(res)
    }
}
