use async_trait::async_trait;
use cosmos_sdk_proto::cosmos::tx::v1beta1::{SimulateRequest, SimulateResponse};
use cosmos_sdk_proto::traits::Message;
use cosmrs::rpc::HttpClient;
use cosmrs::tx::Raw;
use tendermint_rpc::Client;

use crate::chain::error::ChainError;
use crate::chain::model::{ChainTxResponse, GasInfo};

use super::client::CosmosClient;

pub struct TendermintRPC {
    client: HttpClient,
}

impl TendermintRPC {
    pub fn new(rpc_endpoint: &str) -> Self {
        Self {
            client: HttpClient::new(rpc_endpoint).unwrap(),
        }
    }
}

#[async_trait]
impl CosmosClient for TendermintRPC {
    async fn query<T, I, O>(&self, msg: T, path: &str) -> Result<O, ChainError>
    where
        T: Message + Default + tonic::IntoRequest<I>,
        I: Message + 'static,
        O: Message + Default + 'static,
    {
        let mut data = Vec::with_capacity(msg.encoded_len());
        msg.encode(&mut data).unwrap(); //.map_err(ClientError::prost_proto_en)?;

        let res = self
            .client
            .abci_query(Some(path.parse().unwrap()), data, None, false)
            .await
            .unwrap();

        println!("{:?}", res);

        // TODO:
        // if res.code != Code::Ok {
        //     return Err(ChainError::CosmosSdk { res: res.into() });
        // }

        let proto_res = O::decode(res.value.as_slice())
            //.map_err(ChainError::prost_proto_de)?
            .unwrap();

        Ok(proto_res)
    }

    #[allow(deprecated)]
    async fn simulate_tx(&self, tx: &Raw) -> Result<GasInfo, ChainError> {
        let req = SimulateRequest {
            tx: None,
            tx_bytes: tx.to_bytes().unwrap(), //.map_err(ClientError::proto_encoding)?,
        };

        // TODO: DRY this up in a util func here
        let mut data = Vec::with_capacity(req.encoded_len());
        req.encode(&mut data).unwrap();

        let res = self
            .client
            .abci_query(
                Some("/cosmos.tx.v1beta1.Service/Simulate".parse().unwrap()),
                data,
                None,
                false,
            )
            .await
            .unwrap();

        let gas_info = SimulateResponse::decode(res.value.as_slice())
            //.map_err(ChainError::prost_proto_de)?
            .unwrap()
            .gas_info
            .unwrap();

        println!("{:?}", gas_info);

        Ok(gas_info.into())
    }

    async fn broadcast_tx(&self, tx: &Raw) -> Result<ChainTxResponse, ChainError> {
        let res = tx
            .broadcast_commit(&self.client)
            .await
            //.map_err(ClientError::proto_encoding)?;
            .unwrap();

        // if res.check_tx.code.is_err() {
        //     return Err(ClientError::CosmosSdk {
        //         res: tx_commit_response.check_tx.into(),
        //     });
        // }
        // if res.deliver_tx.code.is_err() {
        //     return Err(ClientError::CosmosSdk {
        //         res: res.deliver_tx.into(),
        //     });
        // }

        Ok(res.into())
    }
}
