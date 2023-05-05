use async_trait::async_trait;
use cosmrs::proto::cosmos::tx::v1beta1::{SimulateRequest, SimulateResponse};
use cosmrs::proto::traits::Message;
use cosmrs::rpc::Client;
use tendermint_rpc::endpoint::broadcast::{tx_async, tx_commit, tx_sync};

use crate::chain::error::ChainError;
use crate::chain::fee::GasInfo;
use crate::modules::tx::model::RawTx;

use super::client::{CosmosClientAsync, CosmosClientCommit, CosmosClientQuery, CosmosClientSync};

fn encode_msg<T: Message>(msg: T) -> Result<Vec<u8>, ChainError> {
    let mut data = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut data)
        .map_err(ChainError::prost_proto_encoding)?;
    Ok(data)
}

#[async_trait]
impl<T> CosmosClientQuery for T
where
    T: Client + Sync,
{
    async fn query<I, O>(&self, msg: I, path: &str) -> Result<O, ChainError>
    where
        I: Message + Default + tonic::IntoRequest<I> + 'static,
        O: Message + Default + 'static,
    {
        let bytes = encode_msg(msg)?;

        let res = self
            .abci_query(Some(path.to_string()), bytes, None, false)
            .await?;

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
                Some("/cosmos.tx.v1beta1.Service/Simulate".to_string()),
                bytes,
                None,
                false,
            )
            .await?;

        let gas_info = SimulateResponse::decode(res.value.as_slice())
            .map_err(ChainError::prost_proto_decoding)?
            .gas_info
            .ok_or(ChainError::Simulation)?;

        Ok(gas_info.into())
    }
}

#[async_trait]
impl<T> CosmosClientCommit for T
where
    T: Client + Sync,
{
    async fn broadcast_tx_commit(&self, tx: &RawTx) -> Result<tx_commit::Response, ChainError> {
        let res = tendermint_rpc::Client::broadcast_tx_commit(self, tx.to_bytes()?).await?;

        if res.check_tx.code.is_err() || res.deliver_tx.code.is_err() {
            return Err(ChainError::TxCommit { res });
        }

        Ok(res)
    }
}

#[async_trait]
impl<T> CosmosClientSync for T
where
    T: Client + Sync,
{
    async fn broadcast_tx_sync(&self, tx: &RawTx) -> Result<tx_sync::Response, ChainError> {
        let res = tendermint_rpc::Client::broadcast_tx_sync(self, tx.to_bytes()?).await?;

        if res.code.is_err() {
            return Err(ChainError::TxSync { res });
        }

        Ok(res)
    }
}

#[async_trait]
impl<T> CosmosClientAsync for T
where
    T: Client + Sync,
{
    async fn broadcast_tx_async(&self, tx: &RawTx) -> Result<tx_async::Response, ChainError> {
        let res = tendermint_rpc::Client::broadcast_tx_async(self, tx.to_bytes()?).await?;

        if res.code.is_err() {
            return Err(ChainError::TxAsync { res });
        }

        Ok(res)
    }
}
