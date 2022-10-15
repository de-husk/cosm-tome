use async_trait::async_trait;
use cosmos_sdk_proto::cosmwasm::wasm::v1::AccessConfig;
use cosmos_sdk_proto::traits::Message;
use cosmrs::tx::Raw;
use delegate::delegate;
use serde::Serialize;

use crate::chain::error::ChainError;
use crate::chain::fee::{Coin, GasInfo};
use crate::chain::request::{PaginationRequest, TxOptions};
use crate::chain::response::ChainTxResponse;
use crate::modules::auth::api::Auth;
use crate::modules::auth::model::{AccountResponse, AccountsResponse, ParamsResponse};
use crate::modules::cosmwasm::model::{ExecResponse, QueryResponse};
use crate::{
    config::config::ChainConfig,
    key::key::SigningKey,
    modules::auth::error::AccountError,
    modules::cosmwasm::{
        api::Cosmwasm,
        error::CosmwasmError,
        model::{InstantiateResponse, StoreCodeResponse},
    },
};

use super::cosmos_grpc::CosmosgRPC;
use super::tendermint_rpc::TendermintRPC;

#[async_trait]
pub trait CosmosClient {
    async fn query<T, I, O>(&self, msg: T, path: &str) -> Result<O, ChainError>
    where
        T: Message + Default + tonic::IntoRequest<I>,
        I: Message + 'static,
        O: Message + Default + 'static;

    async fn simulate_tx(&self, tx: &Raw) -> Result<GasInfo, ChainError>;

    async fn broadcast_tx(&self, tx: &Raw) -> Result<ChainTxResponse, ChainError>;
}

#[derive(Clone, Debug)]
pub struct CosmTome<T: CosmosClient> {
    pub(crate) cfg: ChainConfig,
    pub(crate) client: T,

    // cosmos modules:
    pub(crate) wasm: Cosmwasm,
    pub(crate) auth: Auth,
    // bank: Bank,
}

impl<T: CosmosClient> CosmTome<T> {
    /// General usage CosmClient constructor accepting any client that impls `CosmosClient` trait
    pub fn new(cfg: ChainConfig, client: T) -> Self {
        Self {
            cfg,
            client,
            wasm: Cosmwasm {},
            auth: Auth {},
        }
    }

    pub fn with_tendermint_rpc(cfg: ChainConfig) -> Result<CosmTome<TendermintRPC>, ChainError> {
        Ok(CosmTome {
            client: TendermintRPC::new(&cfg.rpc_endpoint.clone())?,
            cfg,
            wasm: Cosmwasm {},
            auth: Auth {},
        })
    }

    pub fn with_cosmos_grpc(cfg: ChainConfig) -> Result<CosmTome<CosmosgRPC>, ChainError> {
        Ok(CosmTome {
            client: CosmosgRPC::new(cfg.grpc_endpoint.clone()),
            cfg,
            wasm: Cosmwasm {},
            auth: Auth {},
        })
    }

    delegate! {
        to self.auth {
            pub async fn auth_query_account(
                &self,
                [&self],
                address: String
            ) -> Result<AccountResponse, AccountError>;

            pub async fn auth_query_accounts(
                &self,
                [&self],
                pagination: Option<PaginationRequest>
            ) -> Result<AccountsResponse, AccountError>;

            pub async fn auth_query_params(
                &self,
                [&self],
            ) -> Result<ParamsResponse, AccountError>;
        }
        to self.wasm {
            pub async fn wasm_store(
                &self,
                [&self],
                payload: Vec<u8>,
                key: &SigningKey,
                instantiate_perms: Option<AccessConfig>,
                tx_options: &TxOptions,
            ) -> Result<StoreCodeResponse, CosmwasmError>;

            pub async fn wasm_instantiate<S: Serialize>(
                &self,
                [&self],
                code_id: u64,
                msg: &S,
                label: String,
                key: &SigningKey,
                admin: Option<String>,
                funds: Vec<Coin>,
                tx_options: &TxOptions,
            ) -> Result<InstantiateResponse, CosmwasmError>;

            pub async fn wasm_execute<S: Serialize>(
                &self,
                [&self],
                address: String,
                msg: &S,
                key: &SigningKey,
                funds: Vec<Coin>,
                tx_options: &TxOptions,
            ) -> Result<ExecResponse, CosmwasmError>;

            pub async fn wasm_query<S: Serialize>(
                &self,
                [&self],
                address: String,
                msg: &S,
            ) -> Result<QueryResponse, CosmwasmError> ;
        }
    }
}
