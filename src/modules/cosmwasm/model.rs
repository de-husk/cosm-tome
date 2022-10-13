use cosmos_sdk_proto::cosmwasm::wasm::v1::QuerySmartContractStateResponse;
use cosmrs::tendermint::abci::Code;
use serde::{Deserialize, Serialize};

use crate::chain::model::{ChainResponse, ChainTxResponse};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreCodeResponse {
    pub code_id: u64,
    pub res: ChainTxResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstantiateResponse {
    pub address: String,
    pub res: ChainTxResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecResponse {
    pub res: ChainTxResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResponse {
    pub res: ChainResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrateResponse {
    pub res: ChainTxResponse,
}

impl From<QuerySmartContractStateResponse> for ChainResponse {
    fn from(res: QuerySmartContractStateResponse) -> ChainResponse {
        ChainResponse {
            code: Code::Ok,
            data: Some(res.data),
            ..Default::default()
        }
    }
}
