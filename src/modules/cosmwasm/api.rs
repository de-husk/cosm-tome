use cosmrs::tx::Msg;
use serde::Serialize;

use cosmos_sdk_proto::cosmwasm::wasm::v1::{
    AccessConfig, QuerySmartContractStateRequest, QuerySmartContractStateResponse,
};
use cosmrs::cosmwasm::MsgStoreCode;

use crate::chain::error::ChainError;
use crate::chain::request::TxOptions;
use crate::chain::tx::sign_tx;
use crate::clients::client::CosmTome;

use crate::modules::auth::model::Address;
use crate::{clients::client::CosmosClient, key::key::SigningKey};

use super::model::{
    ExecRequest, ExecResponse, InstantiateBatchResponse, InstantiateRequest, MigrateRequest,
    MigrateResponse, QueryResponse,
};
use super::{
    error::CosmwasmError,
    model::{InstantiateResponse, StoreCodeResponse},
};

impl<T: CosmosClient> CosmTome<T> {
    /// There is no batch version of `wasm_store` because the txs are already very large since
    /// we send the entire wasm binary
    pub async fn wasm_store(
        &self,
        payload: Vec<u8>,
        key: &SigningKey,
        instantiate_perms: Option<AccessConfig>, // TODO: make my own type in chain::model
        tx_options: &TxOptions,
    ) -> Result<StoreCodeResponse, CosmwasmError> {
        let sender_addr = key.to_addr(&self.cfg.prefix)?;

        let msg = MsgStoreCode {
            sender: sender_addr.clone().into(),
            wasm_byte_code: payload,
            instantiate_permission: instantiate_perms
                .map(TryInto::try_into)
                .transpose()
                .map_err(|e| CosmwasmError::InstantiatePerms { source: e })?,
        }
        .into_any()
        .map_err(ChainError::proto_encoding)?;

        let tx_raw = sign_tx(self, vec![msg], key, sender_addr, tx_options).await?;

        let res = self.client.broadcast_tx(&tx_raw).await?;

        let code_id = res
            .find_event_tags("store_code".to_string(), "code_id".to_string())
            .get(0)
            .ok_or(CosmwasmError::MissingEvent)?
            .value
            .parse::<u64>()
            .unwrap();

        Ok(StoreCodeResponse { code_id, res })
    }

    pub async fn wasm_instantiate<S>(
        &self,
        req: InstantiateRequest<S>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<InstantiateResponse, CosmwasmError>
    where
        S: Serialize,
        T: CosmosClient,
    {
        let mut res = self
            .wasm_instantiate_batch(vec![req], key, tx_options)
            .await?;

        Ok(InstantiateResponse {
            address: res.addresses.remove(0),
            res: res.res,
        })
    }

    pub async fn wasm_instantiate_batch<S, I>(
        &self,
        reqs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<InstantiateBatchResponse, CosmwasmError>
    where
        S: Serialize,
        I: IntoIterator<Item = InstantiateRequest<S>>,
    {
        let sender_addr = key.to_addr(&self.cfg.prefix)?;

        let protos = reqs
            .into_iter()
            .map(|r| r.to_proto(sender_addr.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let msgs = protos
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        let tx_raw = sign_tx(self, msgs, key, sender_addr, tx_options).await?;

        let res = self.client.broadcast_tx(&tx_raw).await?;

        let events =
            res.find_event_tags("instantiate".to_string(), "_contract_address".to_string());

        if events.is_empty() {
            return Err(CosmwasmError::MissingEvent);
        }

        let addrs = events
            .into_iter()
            .map(|e| e.value.parse())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(InstantiateBatchResponse {
            addresses: addrs,
            res: res,
        })
    }

    pub async fn wasm_execute<S>(
        &self,
        req: ExecRequest<S>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<ExecResponse, CosmwasmError>
    where
        S: Serialize,
    {
        self.wasm_execute_batch(vec![req], key, tx_options).await
    }

    pub async fn wasm_execute_batch<S, I>(
        &self,
        reqs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<ExecResponse, CosmwasmError>
    where
        S: Serialize,
        I: IntoIterator<Item = ExecRequest<S>>,
    {
        let sender_addr = key.to_addr(&self.cfg.prefix)?;

        let protos = reqs
            .into_iter()
            .map(|r| r.to_proto(sender_addr.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let msgs = protos
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        let tx_raw = sign_tx(self, msgs, key, sender_addr, tx_options).await?;

        let res = self.client.broadcast_tx(&tx_raw).await?;

        Ok(ExecResponse { res })
    }

    pub async fn wasm_query<S: Serialize>(
        &self,
        address: Address,
        msg: &S,
    ) -> Result<QueryResponse, CosmwasmError> {
        let payload = serde_json::to_vec(msg).map_err(CosmwasmError::json)?;

        let req = QuerySmartContractStateRequest {
            address: address.into(),
            query_data: payload,
        };

        let res = self
            .client
            .query::<_, QuerySmartContractStateRequest, QuerySmartContractStateResponse>(
                req,
                "/cosmwasm.wasm.v1.Query/SmartContractState",
            )
            .await?;

        Ok(QueryResponse { res: res.into() })
    }

    pub async fn migrate<S>(
        &self,
        req: MigrateRequest<S>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<MigrateResponse, CosmwasmError>
    where
        S: Serialize,
    {
        self.migrate_batch(vec![req], key, tx_options).await
    }

    pub async fn migrate_batch<S, I>(
        &self,
        reqs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<MigrateResponse, CosmwasmError>
    where
        S: Serialize,
        I: IntoIterator<Item = MigrateRequest<S>>,
    {
        let sender_addr = key.to_addr(&self.cfg.prefix)?;

        let protos = reqs
            .into_iter()
            .map(|r| r.to_proto(sender_addr.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let msgs = protos
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        let tx_raw = sign_tx(self, msgs, key, sender_addr, tx_options).await?;

        let res = self.client.broadcast_tx(&tx_raw).await?;

        Ok(MigrateResponse { res })
    }

    // TODO: Finish
}
