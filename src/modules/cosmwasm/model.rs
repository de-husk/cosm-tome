use cosmrs::proto::cosmwasm::wasm::v1::MsgStoreCode;
use cosmrs::proto::cosmwasm::wasm::v1::{
    AccessConfig as ProtoAccessConfig, AccessType as ProtoAccessType, MsgExecuteContract,
    MsgInstantiateContract, MsgMigrateContract, QuerySmartContractStateResponse,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::chain::error::DeserializeError;
use crate::chain::msg::Msg;
use crate::{
    chain::{
        coin::Coin,
        response::{ChainResponse, ChainTxResponse, Code},
    },
    modules::auth::model::Address,
};

use super::error::CosmwasmError;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct StoreCodeRequest {
    pub wasm_data: Vec<u8>,
    pub instantiate_perms: Option<AccessConfig>,
}

impl StoreCodeRequest {
    pub fn to_proto(self, signer_addr: Address) -> Result<StoreCodeProto, CosmwasmError> {
        Ok(StoreCodeProto {
            signer_addr,
            wasm_data: self.wasm_data,
            instantiate_perms: self.instantiate_perms,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct StoreCodeProto {
    pub signer_addr: Address,
    pub wasm_data: Vec<u8>,
    pub instantiate_perms: Option<AccessConfig>,
}

impl Msg for StoreCodeProto {
    type Proto = MsgStoreCode;
    type Err = CosmwasmError;
}

impl TryFrom<MsgStoreCode> for StoreCodeProto {
    type Error = CosmwasmError;

    fn try_from(msg: MsgStoreCode) -> Result<Self, Self::Error> {
        Ok(Self {
            signer_addr: msg.sender.parse()?,
            wasm_data: msg.wasm_byte_code,
            instantiate_perms: msg
                .instantiate_permission
                .map(TryFrom::try_from)
                .transpose()?,
        })
    }
}

impl TryFrom<StoreCodeProto> for MsgStoreCode {
    type Error = CosmwasmError;

    fn try_from(req: StoreCodeProto) -> Result<Self, Self::Error> {
        Ok(Self {
            sender: req.signer_addr.into(),
            wasm_byte_code: req.wasm_data,
            instantiate_permission: req.instantiate_perms.map(Into::into),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct StoreCodeResponse {
    pub code_id: u64,
    pub res: ChainTxResponse,
}

impl StoreCodeResponse {
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> Result<T, DeserializeError> {
        self.res.res.data()
    }
}

impl AsRef<ChainTxResponse> for StoreCodeResponse {
    fn as_ref(&self) -> &ChainTxResponse {
        &self.res
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct StoreCodeBatchResponse {
    pub code_ids: Vec<u64>,
    pub res: ChainTxResponse,
}

impl StoreCodeBatchResponse {
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> Result<T, DeserializeError> {
        self.res.res.data()
    }
}

impl AsRef<ChainTxResponse> for StoreCodeBatchResponse {
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
            signer_addr,
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

impl Msg for InstantiateRequestProto {
    type Proto = MsgInstantiateContract;
    type Err = CosmwasmError;
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
            admin,
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

impl InstantiateResponse {
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> Result<T, DeserializeError> {
        self.res.res.data()
    }
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

impl InstantiateBatchResponse {
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> Result<T, DeserializeError> {
        self.res.res.data()
    }
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

impl Msg for ExecRequestProto {
    type Proto = MsgExecuteContract;
    type Err = CosmwasmError;
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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct ExecResponse {
    pub res: ChainTxResponse,
}

impl ExecResponse {
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> Result<T, DeserializeError> {
        self.res.res.data()
    }
}

impl AsRef<ChainTxResponse> for ExecResponse {
    fn as_ref(&self) -> &ChainTxResponse {
        &self.res
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct QueryResponse {
    pub res: ChainResponse,
}

impl QueryResponse {
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> Result<T, DeserializeError> {
        self.res.data()
    }
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

impl Msg for MigrateRequestProto {
    type Proto = MsgMigrateContract;
    type Err = CosmwasmError;
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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct MigrateResponse {
    pub res: ChainTxResponse,
}

impl MigrateResponse {
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> Result<T, DeserializeError> {
        self.res.res.data()
    }
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

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccessConfig {
    pub permission: AccessType,
    pub account: Address,
}

impl From<AccessConfig> for cosmrs::cosmwasm::AccessConfig {
    fn from(config: AccessConfig) -> Self {
        Self {
            permission: config.permission.into(),
            address: config.account.into(),
        }
    }
}

impl From<cosmrs::cosmwasm::AccessConfig> for AccessConfig {
    fn from(config: cosmrs::cosmwasm::AccessConfig) -> Self {
        Self {
            permission: config.permission.into(),
            account: config.address.into(),
        }
    }
}

impl From<AccessConfig> for ProtoAccessConfig {
    fn from(config: AccessConfig) -> Self {
        Self {
            permission: config.permission as i32,
            address: config.account.into(),
        }
    }
}

impl TryFrom<ProtoAccessConfig> for AccessConfig {
    type Error = CosmwasmError;

    fn try_from(config: ProtoAccessConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            permission: config.permission.try_into()?,
            account: config.address.parse()?,
        })
    }
}

#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash, PartialOrd, Ord,
)]
#[repr(i32)]
pub enum AccessType {
    /// ACCESS_TYPE_UNSPECIFIED placeholder for empty value
    Unspecified = 0,
    /// ACCESS_TYPE_NOBODY forbidden TODO: better comments that explains what it actually does like we do in BroadcastMode
    Nobody = 1,
    /// ACCESS_TYPE_ONLY_ADDRESS restricted to an address
    OnlyAddress = 2,
    /// ACCESS_TYPE_EVERYBODY unrestricted
    Everybody = 3,
}

impl AsRef<str> for AccessType {
    fn as_ref(&self) -> &str {
        match self {
            AccessType::Unspecified => "ACCESS_TYPE_UNSPECIFIED",
            AccessType::Nobody => "ACCESS_TYPE_NOBODY",
            AccessType::OnlyAddress => "ACCESS_TYPE_ONLY_ADDRESS",
            AccessType::Everybody => "ACCESS_TYPE_EVERYBODY",
        }
    }
}

impl TryFrom<i32> for AccessType {
    type Error = CosmwasmError;

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == AccessType::Unspecified as i32 => Ok(AccessType::Unspecified),
            x if x == AccessType::Nobody as i32 => Ok(AccessType::Nobody),
            x if x == AccessType::OnlyAddress as i32 => Ok(AccessType::OnlyAddress),
            x if x == AccessType::Everybody as i32 => Ok(AccessType::Everybody),
            _ => Err(CosmwasmError::AccessType { i: v }),
        }
    }
}

impl From<AccessType> for ProtoAccessType {
    fn from(perm: AccessType) -> Self {
        match perm {
            AccessType::Unspecified => ProtoAccessType::Unspecified,
            AccessType::Nobody => ProtoAccessType::Nobody,
            AccessType::OnlyAddress => ProtoAccessType::OnlyAddress,
            AccessType::Everybody => ProtoAccessType::Everybody,
        }
    }
}

impl From<ProtoAccessType> for AccessType {
    fn from(perm: ProtoAccessType) -> Self {
        match perm {
            ProtoAccessType::Unspecified => AccessType::Unspecified,
            ProtoAccessType::Nobody => AccessType::Nobody,
            ProtoAccessType::OnlyAddress => AccessType::OnlyAddress,
            ProtoAccessType::Everybody => AccessType::Everybody,
        }
    }
}
