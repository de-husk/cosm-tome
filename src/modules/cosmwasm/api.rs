use cosmos_sdk_proto::Any;
use serde::Serialize;

use crate::chain::request::TxOptions;
use crate::clients::client::CosmTome;
use cosmos_sdk_proto::cosmwasm::wasm::v1::{
    QuerySmartContractStateRequest, QuerySmartContractStateResponse,
};

use crate::modules::auth::model::Address;
use crate::{clients::client::CosmosClient, signing_key::key::SigningKey};

use super::model::{
    ExecRequest, ExecResponse, InstantiateBatchResponse, InstantiateRequest, MigrateRequest,
    MigrateResponse, QueryResponse, StoreCodeRequest,
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
        req: StoreCodeRequest,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<StoreCodeResponse, CosmwasmError> {
        let sender_addr = key.to_addr(&self.cfg.prefix)?;

        let msg: Any = req.to_proto(sender_addr)?.try_into()?;

        let tx_raw = self.tx_sign(vec![msg], key, tx_options).await?;

        let res = self.tx_broadcast_block(&tx_raw).await?;

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

        let tx_raw = self.tx_sign(msgs, key, tx_options).await?;

        let res = self.tx_broadcast_block(&tx_raw).await?;

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
            res,
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

        let tx_raw = self.tx_sign(msgs, key, tx_options).await?;

        let res = self.tx_broadcast_block(&tx_raw).await?;

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
            .query::<_, QuerySmartContractStateResponse>(
                req,
                "/cosmwasm.wasm.v1.Query/SmartContractState",
            )
            .await?;

        Ok(QueryResponse { res: res.into() })
    }

    pub async fn wasm_migrate<S>(
        &self,
        req: MigrateRequest<S>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<MigrateResponse, CosmwasmError>
    where
        S: Serialize,
    {
        self.wasm_migrate_batch(vec![req], key, tx_options).await
    }

    pub async fn wasm_migrate_batch<S, I>(
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

        let tx_raw = self.tx_sign(msgs, key, tx_options).await?;

        let res = self.tx_broadcast_block(&tx_raw).await?;

        Ok(MigrateResponse { res })
    }

    // TODO: Finish
}
