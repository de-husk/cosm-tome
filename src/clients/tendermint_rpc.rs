use async_trait::async_trait;
use cosmos_sdk_proto::cosmos::tx::v1beta1::{SimulateRequest, SimulateResponse};
use cosmos_sdk_proto::traits::Message;
use cosmrs::rpc::HttpClient;
use tendermint_rpc::Client;

use crate::chain::error::ChainError;
use crate::chain::fee::GasInfo;
use crate::chain::response::{AsyncChainTxResponse, ChainTxResponse};
use crate::modules::tx::model::{BroadcastMode, RawTx};

use super::client::CosmosClient;

#[derive(Clone, Debug)]
pub struct TendermintRPC {
    client: HttpClient,
}

impl TendermintRPC {
    pub fn new(rpc_endpoint: &str) -> Result<Self, ChainError> {
        Ok(Self {
            client: HttpClient::new(rpc_endpoint)?,
        })
    }

    fn encode_msg<T: Message>(msg: T) -> Result<Vec<u8>, ChainError> {
        let mut data = Vec::with_capacity(msg.encoded_len());
        msg.encode(&mut data)
            .map_err(ChainError::prost_proto_encoding)?;
        Ok(data)
    }
}

#[async_trait]
impl CosmosClient for TendermintRPC {
    async fn query<I, O>(&self, msg: I, path: &str) -> Result<O, ChainError>
    where
        I: Message + Default + tonic::IntoRequest<I> + 'static,
        O: Message + Default + 'static,
    {
        let bytes = TendermintRPC::encode_msg(msg)?;

        // TODO: dont hardcode `height` and `prove`?
        let res = self
            .client
            .abci_query(Some(path.parse()?), bytes, None, false)
            .await?;

        if res.code.is_err() {
            return Err(ChainError::CosmosSdk { res: res.into() });
        }

        let proto_res =
            O::decode(res.value.as_slice()).map_err(ChainError::prost_proto_decoding)?;

        Ok(proto_res)
    }

    #[allow(deprecated)]
    async fn simulate_tx(&self, tx: &RawTx) -> Result<GasInfo, ChainError> {
        let req = SimulateRequest {
            tx: None,
            tx_bytes: tx.to_bytes()?,
        };

        let bytes = TendermintRPC::encode_msg(req)?;

        let res = self
            .client
            .abci_query(
                Some("/cosmos.tx.v1beta1.Service/Simulate".parse()?),
                bytes,
                None,
                false,
            )
            .await?;

        if res.code.is_err() {
            return Err(ChainError::CosmosSdk { res: res.into() });
        }

        let gas_info = SimulateResponse::decode(res.value.as_slice())
            .map_err(ChainError::prost_proto_decoding)?
            .gas_info
            .ok_or(ChainError::Simulation)?;

        Ok(gas_info.into())
    }

    async fn broadcast_tx(
        &self,
        tx: &RawTx,
        mode: BroadcastMode,
    ) -> Result<AsyncChainTxResponse, ChainError> {
        let res: AsyncChainTxResponse = match mode {
            BroadcastMode::Sync => self
                .client
                .broadcast_tx_sync(tx.to_bytes()?.into())
                .await?
                .into(),
            BroadcastMode::Async => self
                .client
                .broadcast_tx_async(tx.to_bytes()?.into())
                .await?
                .into(),
        };

        if res.res.code.is_err() {
            return Err(ChainError::CosmosSdk { res: res.res });
        }

        Ok(res)
    }

    async fn broadcast_tx_block(&self, tx: &RawTx) -> Result<ChainTxResponse, ChainError> {
        let res = self
            .client
            .broadcast_tx_commit(tx.to_bytes()?.into())
            .await?;

        if res.check_tx.code.is_err() {
            return Err(ChainError::CosmosSdk {
                res: res.check_tx.into(),
            });
        }
        if res.deliver_tx.code.is_err() {
            return Err(ChainError::CosmosSdk {
                res: res.deliver_tx.into(),
            });
        }

        Ok(res.into())
    }
}
