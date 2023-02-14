use super::client::CosmosClient;
use async_trait::async_trait;
use cosmrs::proto::cosmos::tx::v1beta1::{SimulateRequest, SimulateResponse};
use cosmrs::proto::traits::Message;
use tendermint_rpc::{Client, WebSocketClient};

use crate::chain::error::ChainError;
use crate::chain::fee::GasInfo;
use crate::chain::response::{AsyncChainTxResponse, ChainTxResponse};
use crate::modules::tx::model::{BroadcastMode, RawTx};

#[async_trait]
impl CosmosClient for WebSocketClient {
    async fn query<I, O>(&self, msg: I, path: &str) -> Result<O, ChainError>
    where
        I: Message + Default + tonic::IntoRequest<I> + 'static,
        O: Message + Default + 'static,
    {
        let bytes = encode_msg(msg)?;

        let res = self
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

        let bytes = encode_msg(req)?;

        let res = self
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
                .broadcast_tx_sync::<Vec<u8>>(tx.to_bytes()?.into())
                .await?
                .into(),
            BroadcastMode::Async => self
                .broadcast_tx_async::<Vec<u8>>(tx.to_bytes()?.into())
                .await?
                .into(),
        };

        if res.res.code.is_err() {
            return Err(ChainError::CosmosSdk { res: res.res });
        }

        Ok(res)
    }

    async fn broadcast_tx_block(&self, tx: &RawTx) -> Result<ChainTxResponse, ChainError> {
        todo!()
        // let res = self.broadcast_tx_commit(tx.to_bytes()?.into()).await?;

        // if res.check_tx.code.is_err() {
        //     return Err(ChainError::CosmosSdk {
        //         res: res.check_tx.into(),
        //     });
        // }
        // if res.deliver_tx.code.is_err() {
        //     return Err(ChainError::CosmosSdk {
        //         res: res.deliver_tx.into(),
        //     });
        // }

        // Ok(res.into())
    }
}

fn encode_msg<T: Message>(msg: T) -> Result<Vec<u8>, ChainError> {
    let mut data = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut data)
        .map_err(ChainError::prost_proto_encoding)?;
    Ok(data)
}
