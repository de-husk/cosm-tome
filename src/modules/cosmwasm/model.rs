use cosmos_sdk_proto::traits::MessageExt;
use cosmos_sdk_proto::{
    cosmwasm::wasm::v1::{
        MsgExecuteContract, MsgInstantiateContract, MsgMigrateContract,
        QuerySmartContractStateResponse,
    },
    Any,
};
use serde::{Deserialize, Serialize};

use crate::{
    chain::{
        coin::Coin,
        error::ChainError,
        response::{ChainResponse, ChainTxResponse, Code},
    },
    modules::auth::model::Address,
};

use super::error::CosmwasmError;

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
pub struct InstantiateRequest<S: Serialize> {
    pub code_id: u64,
    pub msg: S,
    pub label: String,
    pub admin: Option<Address>,
    pub funds: Vec<Coin>,
}

impl<S: Serialize> InstantiateRequest<S> {
    pub fn to_proto(self, signer_addr: Address) -> Result<InstantiateRequestProto, CosmwasmError> {
        let payload = serde_json::to_vec(&self.msg).map_err(CosmwasmError::json)?;

        Ok(InstantiateRequestProto {
            signer_addr: signer_addr,
            code_id: self.code_id,
            msg: payload,
            label: self.label,
            admin: self.admin,
            funds: self.funds,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstantiateRequestProto {
    pub signer_addr: Address,
    pub code_id: u64,
    pub msg: Vec<u8>,
    pub label: String,
    pub admin: Option<Address>,
    pub funds: Vec<Coin>,
}

impl TryFrom<InstantiateRequestProto> for Any {
    type Error = CosmwasmError;

    fn try_from(req: InstantiateRequestProto) -> Result<Self, Self::Error> {
        let proto: MsgInstantiateContract = req.try_into()?;
        Ok(proto.to_any().map_err(ChainError::prost_proto_encoding)?)
    }
}

impl TryFrom<Any> for InstantiateRequestProto {
    type Error = CosmwasmError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        MsgInstantiateContract::from_any(&any)
            .map_err(ChainError::prost_proto_decoding)?
            .try_into()
    }
}

impl TryFrom<MsgInstantiateContract> for InstantiateRequestProto {
    type Error = CosmwasmError;

    fn try_from(msg: MsgInstantiateContract) -> Result<Self, Self::Error> {
        let admin = if msg.admin.is_empty() {
            None
        } else {
            Some(msg.admin.parse()?)
        };

        Ok(Self {
            signer_addr: msg.sender.parse()?,
            code_id: msg.code_id,
            msg: msg.msg,
            label: msg.label,
            admin: admin,
            funds: msg
                .funds
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<InstantiateRequestProto> for MsgInstantiateContract {
    type Error = CosmwasmError;

    fn try_from(req: InstantiateRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {
            sender: req.signer_addr.into(),
            admin: req.admin.map(Into::into).unwrap_or_default(),
            code_id: req.code_id,
            label: req.label,
            msg: req.msg,
            funds: req.funds.into_iter().map(Into::into).collect(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstantiateResponse {
    pub address: Address,
    pub res: ChainTxResponse,
}

impl AsRef<ChainTxResponse> for InstantiateResponse {
    fn as_ref(&self) -> &ChainTxResponse {
        &self.res
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstantiateBatchResponse {
    pub addresses: Vec<Address>,
    pub res: ChainTxResponse,
}

impl AsRef<ChainTxResponse> for InstantiateBatchResponse {
    fn as_ref(&self) -> &ChainTxResponse {
        &self.res
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExecRequest<S: Serialize> {
    pub address: Address,
    pub msg: S,
    pub funds: Vec<Coin>,
}

impl<S: Serialize> ExecRequest<S> {
    pub fn to_proto(self, signer_addr: Address) -> Result<ExecRequestProto, CosmwasmError> {
        let payload = serde_json::to_vec(&self.msg).map_err(CosmwasmError::json)?;

        Ok(ExecRequestProto {
            signer_addr,
            contract_addr: self.address,
            msg: payload,
            funds: self.funds,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExecRequestProto {
    pub signer_addr: Address,
    pub contract_addr: Address,
    pub msg: Vec<u8>,
    pub funds: Vec<Coin>,
}

impl TryFrom<ExecRequestProto> for Any {
    type Error = CosmwasmError;

    fn try_from(req: ExecRequestProto) -> Result<Self, Self::Error> {
        let proto: MsgExecuteContract = req.try_into()?;
        Ok(proto.to_any().map_err(ChainError::prost_proto_encoding)?)
    }
}

impl TryFrom<Any> for ExecRequestProto {
    type Error = CosmwasmError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        MsgExecuteContract::from_any(&any)
            .map_err(ChainError::prost_proto_decoding)?
            .try_into()
    }
}

impl TryFrom<MsgExecuteContract> for ExecRequestProto {
    type Error = CosmwasmError;

    fn try_from(msg: MsgExecuteContract) -> Result<Self, Self::Error> {
        Ok(Self {
            signer_addr: msg.sender.parse()?,
            contract_addr: msg.contract.parse()?,
            msg: msg.msg,
            funds: msg
                .funds
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<ExecRequestProto> for MsgExecuteContract {
    type Error = CosmwasmError;

    fn try_from(req: ExecRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {
            sender: req.signer_addr.into(),
            contract: req.contract_addr.into(),
            msg: req.msg,
            funds: req.funds.into_iter().map(Into::into).collect(),
        })
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
pub struct MigrateRequest<S: Serialize> {
    pub address: Address,
    pub new_code_id: u64,
    pub msg: S,
}

impl<S: Serialize> MigrateRequest<S> {
    pub fn to_proto(self, signer_addr: Address) -> Result<MigrateRequestProto, CosmwasmError> {
        let payload = serde_json::to_vec(&self.msg).map_err(CosmwasmError::json)?;

        Ok(MigrateRequestProto {
            signer_addr,
            contract_addr: self.address,
            new_code_id: self.new_code_id,
            msg: payload,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct MigrateRequestProto {
    pub signer_addr: Address,
    pub contract_addr: Address,
    pub new_code_id: u64,
    pub msg: Vec<u8>,
}

impl TryFrom<MigrateRequestProto> for Any {
    type Error = CosmwasmError;

    fn try_from(req: MigrateRequestProto) -> Result<Self, Self::Error> {
        let proto: MsgMigrateContract = req.try_into()?;
        Ok(proto.to_any().map_err(ChainError::prost_proto_encoding)?)
    }
}

impl TryFrom<Any> for MigrateRequestProto {
    type Error = CosmwasmError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        MsgMigrateContract::from_any(&any)
            .map_err(ChainError::prost_proto_decoding)?
            .try_into()
    }
}

impl TryFrom<MsgMigrateContract> for MigrateRequestProto {
    type Error = CosmwasmError;

    fn try_from(msg: MsgMigrateContract) -> Result<Self, Self::Error> {
        Ok(Self {
            signer_addr: msg.sender.parse()?,
            contract_addr: msg.contract.parse()?,
            new_code_id: msg.code_id,
            msg: msg.msg,
        })
    }
}

impl TryFrom<MigrateRequestProto> for MsgMigrateContract {
    type Error = CosmwasmError;

    fn try_from(req: MigrateRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {
            sender: req.signer_addr.into(),
            contract: req.contract_addr.into(),
            code_id: req.new_code_id,
            msg: req.msg,
        })
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
