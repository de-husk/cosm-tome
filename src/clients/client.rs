use async_trait::async_trait;
use cosmos_sdk_proto::cosmwasm::wasm::v1::AccessConfig;
use cosmos_sdk_proto::traits::Message;
use cosmrs::tx::Raw;
use delegate::delegate;
use serde::Serialize;

use crate::chain::coin::{Coin, Denom};
use crate::chain::error::ChainError;
use crate::chain::fee::GasInfo;
use crate::chain::request::{PaginationRequest, TxOptions};
use crate::chain::response::ChainTxResponse;
use crate::modules::auth::api::Auth;
use crate::modules::auth::model::{AccountResponse, AccountsResponse, Address};
use crate::modules::bank::api::Bank;
use crate::modules::bank::error::BankError;
use crate::modules::bank::model::{
    BalanceResponse, BalancesResponse, DenomMetadataResponse, DenomsMetadataResponse, SendResponse,
};
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
    pub(crate) bank: Bank,
}

impl<T: CosmosClient> CosmTome<T> {
    /// General usage CosmClient constructor accepting any client that impls `CosmosClient` trait
    pub fn new(cfg: ChainConfig, client: T) -> Self {
        Self {
            cfg,
            client,
            wasm: Cosmwasm {},
            auth: Auth {},
            bank: Bank {},
        }
    }

    pub fn with_tendermint_rpc(cfg: ChainConfig) -> Result<CosmTome<TendermintRPC>, ChainError> {
        Ok(CosmTome {
            client: TendermintRPC::new(&cfg.rpc_endpoint.clone())?,
            cfg,
            wasm: Cosmwasm {},
            auth: Auth {},
            bank: Bank {},
        })
    }

    pub fn with_cosmos_grpc(cfg: ChainConfig) -> Result<CosmTome<CosmosgRPC>, ChainError> {
        Ok(CosmTome {
            client: CosmosgRPC::new(cfg.grpc_endpoint.clone()),
            cfg,
            wasm: Cosmwasm {},
            auth: Auth {},
            bank: Bank {},
        })
    }

    delegate! {
        to self.auth {
            pub async fn auth_query_account(
                &self,
                [&self],
                address: &Address
            ) -> Result<AccountResponse, AccountError>;

            pub async fn auth_query_accounts(
                &self,
                [&self],
                pagination: Option<PaginationRequest>
            ) -> Result<AccountsResponse, AccountError>;

            pub async fn auth_query_params(
                &self,
                [&self],
            ) -> Result<crate::modules::auth::model::ParamsResponse, AccountError>;
        }

        to self.bank {
            pub async fn bank_send<I>(
                &self,
                [&self],
                from: &Address,
                to: &Address,
                amounts: I,
                key: &SigningKey,
                tx_options: &TxOptions,
            ) -> Result<SendResponse, BankError>
            where
                I: IntoIterator<Item = Coin>;

            pub async fn bank_query_balance(
                &self,
                [&self],
                address: &Address,
                denom: Denom,
            ) -> Result<BalanceResponse, BankError>;

            pub async fn bank_query_balances(
                &self,
                [&self],
                address: &Address,
                pagination: Option<PaginationRequest>,
            ) -> Result<BalancesResponse, BankError>;

            pub async fn bank_query_spendable_balances(
                &self,
                [&self],
                address: &Address,
                pagination: Option<PaginationRequest>,
            ) -> Result<BalancesResponse, BankError>;

            pub async fn bank_query_supply(
                &self,
                [&self],
                denom: Denom,
            ) -> Result<BalanceResponse, BankError>;

            pub async fn bank_query_total_supply(
                &self,
                [&self],
                pagination: Option<PaginationRequest>,
            ) -> Result<BalancesResponse, BankError>;

            pub async fn bank_query_denom_metadata(
                &self,
                [&self],
                denom: Denom,
            ) -> Result<DenomMetadataResponse, BankError>;

            pub async fn bank_query_denoms_metadata(
                &self,
                [&self],
                pagination: Option<PaginationRequest>,
            ) -> Result<DenomsMetadataResponse, BankError>;

            pub async fn bank_query_params(
                &self,
                [&self],
            ) -> Result<crate::modules::bank::model::ParamsResponse, BankError>;
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

            pub async fn wasm_instantiate<S, I>(
                &self,
                [&self],
                code_id: u64,
                msg: &S,
                label: String,
                key: &SigningKey,
                admin: Option<Address>,
                funds: I,
                tx_options: &TxOptions,
            ) -> Result<InstantiateResponse, CosmwasmError>
            where
                S: Serialize,
                I: IntoIterator<Item = Coin>;

            pub async fn wasm_execute<S, I>(
                &self,
                [&self],
                address: &Address,
                msg: &S,
                key: &SigningKey,
                funds: I,
                tx_options: &TxOptions,
            ) -> Result<ExecResponse, CosmwasmError>
            where
                S: Serialize,
                I: IntoIterator<Item = Coin>;

            pub async fn wasm_query<S: Serialize>(
                &self,
                [&self],
                address: &Address,
                msg: &S,
            ) -> Result<QueryResponse, CosmwasmError> ;
        }
    }
}
