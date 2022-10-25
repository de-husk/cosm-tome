use cosmos_sdk_proto::cosmwasm::wasm::v1::QuerySmartContractStateResponse;
use serde::{Deserialize, Serialize};

use crate::chain::response::{ChainResponse, ChainTxResponse, Code};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct StoreCodeResponse {
    pub code_id: u64,
    pub res: ChainTxResponse,
}

impl AsRef<ChainTxResponse> for StoreCodeResponse {
    fn as_ref(&self) -> &ChainTxResponse {
        &self.res
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstantiateResponse {
    pub address: String, // TODO: Use AccountAddr?
    pub res: ChainTxResponse,
}

impl AsRef<ChainTxResponse> for InstantiateResponse {
    fn as_ref(&self) -> &ChainTxResponse {
        &self.res
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExecResponse {
    pub res: ChainTxResponse,
}

impl AsRef<ChainTxResponse> for ExecResponse {
    fn as_ref(&self) -> &ChainTxResponse {
        &self.res
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct QueryResponse {
    pub res: ChainResponse,
}

impl AsRef<ChainResponse> for QueryResponse {
    fn as_ref(&self) -> &ChainResponse {
        &self.res
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct MigrateResponse {
    pub res: ChainTxResponse,
}

impl AsRef<ChainTxResponse> for MigrateResponse {
    fn as_ref(&self) -> &ChainTxResponse {
        &self.res
    }
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
