use cosmos_sdk_proto::cosmos::bank::v1beta1::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest, QueryBalanceResponse,
    QueryDenomMetadataRequest, QueryDenomMetadataResponse, QueryDenomsMetadataRequest,
    QueryDenomsMetadataResponse, QueryParamsRequest, QueryParamsResponse,
    QuerySpendableBalancesRequest, QuerySpendableBalancesResponse, QuerySupplyOfRequest,
    QuerySupplyOfResponse, QueryTotalSupplyRequest, QueryTotalSupplyResponse,
};

use crate::{
    chain::{
        coin::Denom,
        request::{PaginationRequest, TxOptions},
        tx::sign_tx,
    },
    clients::client::{CosmTome, CosmosClient},
    key::key::SigningKey,
    modules::{auth::model::Address, bank::model::SendResponse},
};

use super::{
    error::BankError,
    model::{
        BalanceResponse, BalancesResponse, DenomMetadataResponse, DenomsMetadataResponse,
        ParamsResponse, SendRequest,
    },
};

#[derive(Clone, Debug)]
pub struct Bank {}

impl Bank {
    /// Send `amount` of funds from source (`from`) Address to destination (`to`) Address
    pub(crate) async fn bank_send<T>(
        &self,
        client: &CosmTome<T>,
        req: SendRequest,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<SendResponse, BankError>
    where
        T: CosmosClient,
    {
        self.bank_send_batch(client, vec![req], key, tx_options)
            .await
    }

    pub(crate) async fn bank_send_batch<T, I>(
        &self,
        client: &CosmTome<T>,
        reqs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<SendResponse, BankError>
    where
        T: CosmosClient,
        I: IntoIterator<Item = SendRequest>,
    {
        let sender_addr = key.to_addr(&client.cfg.prefix)?;

        let msgs = reqs
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        let tx_raw = sign_tx(client, msgs, key, sender_addr, tx_options).await?;

        let res = client.client.broadcast_tx(&tx_raw).await?;

        Ok(SendResponse { res })
    }

    /// Query the amount of `denom` currently held by an `address`
    pub(crate) async fn bank_query_balance<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
        address: Address,
        denom: Denom,
    ) -> Result<BalanceResponse, BankError> {
        let req = QueryBalanceRequest {
            address: address.into(),
            denom: denom.into(),
        };

        let res = client
            .client
            .query::<_, QueryBalanceRequest, QueryBalanceResponse>(
                req,
                "/cosmos.bank.v1beta1.Query/Balance",
            )
            .await?;

        // NOTE: we are unwrapping here, because unknown denoms still have a 0 balance returned here
        let balance = res.balance.unwrap().try_into()?;

        Ok(BalanceResponse { balance })
    }

    /// Query all denom balances held by an `address`
    pub(crate) async fn bank_query_balances<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
        address: Address,
        pagination: Option<PaginationRequest>,
    ) -> Result<BalancesResponse, BankError> {
        let req = QueryAllBalancesRequest {
            address: address.into(),
            pagination: pagination.map(Into::into),
        };

        let res = client
            .client
            .query::<_, QueryAllBalancesRequest, QueryAllBalancesResponse>(
                req,
                "/cosmos.bank.v1beta1.Query/AllBalances",
            )
            .await?;

        let balances = res
            .balances
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BalancesResponse {
            balances,
            next: res.pagination.map(Into::into),
        })
    }

    /// Get total spendable balance for an `address` (not currently locked away via delegation for example)
    pub(crate) async fn bank_query_spendable_balances<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
        address: Address,
        pagination: Option<PaginationRequest>,
    ) -> Result<BalancesResponse, BankError> {
        let req = QuerySpendableBalancesRequest {
            address: address.into(),
            pagination: pagination.map(Into::into),
        };

        let res = client
            .client
            .query::<_, QuerySpendableBalancesRequest, QuerySpendableBalancesResponse>(
                req,
                "/cosmos.bank.v1beta1.Query/SpendableBalances",
            )
            .await?;

        let balances = res
            .balances
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BalancesResponse {
            balances,
            next: res.pagination.map(Into::into),
        })
    }

    /// Query global supply of `denom` for all accounts
    pub(crate) async fn bank_query_supply<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
        denom: Denom,
    ) -> Result<BalanceResponse, BankError> {
        let req = QuerySupplyOfRequest {
            denom: denom.into(),
        };

        let res = client
            .client
            .query::<_, QuerySupplyOfRequest, QuerySupplyOfResponse>(
                req,
                "/cosmos.bank.v1beta1.Query/SupplyOf",
            )
            .await?;

        // NOTE: we are unwrapping here, because unknown denoms still have a 0 balance returned here
        let balance = res.amount.unwrap().try_into()?;

        Ok(BalanceResponse { balance })
    }

    /// Query global supply of all denoms for all accounts
    pub(crate) async fn bank_query_total_supply<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
        pagination: Option<PaginationRequest>,
    ) -> Result<BalancesResponse, BankError> {
        let req = QueryTotalSupplyRequest {
            pagination: pagination.map(Into::into),
        };

        let res = client
            .client
            .query::<_, QueryTotalSupplyRequest, QueryTotalSupplyResponse>(
                req,
                "/cosmos.bank.v1beta1.Query/TotalSupply",
            )
            .await?;

        let balances = res
            .supply
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BalancesResponse {
            balances,
            next: res.pagination.map(Into::into),
        })
    }

    /// Query bank metadata for a single denom
    pub(crate) async fn bank_query_denom_metadata<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
        denom: Denom,
    ) -> Result<DenomMetadataResponse, BankError> {
        let req = QueryDenomMetadataRequest {
            denom: denom.into(),
        };

        let res = client
            .client
            .query::<_, QueryDenomMetadataRequest, QueryDenomMetadataResponse>(
                req,
                "/cosmos.bank.v1beta1.Query/DenomMetadata",
            )
            .await?;

        Ok(DenomMetadataResponse {
            meta: res.metadata.map(Into::into),
        })
    }

    /// Query bank metadata for all denoms
    pub(crate) async fn bank_query_denoms_metadata<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
        pagination: Option<PaginationRequest>,
    ) -> Result<DenomsMetadataResponse, BankError> {
        let req = QueryDenomsMetadataRequest {
            pagination: pagination.map(Into::into),
        };

        let res = client
            .client
            .query::<_, QueryDenomsMetadataRequest, QueryDenomsMetadataResponse>(
                req,
                "/cosmos.bank.v1beta1.Query/DenomsMetadata",
            )
            .await?;

        Ok(DenomsMetadataResponse {
            metas: res.metadatas.into_iter().map(Into::into).collect(),
            next: res.pagination.map(Into::into),
        })
    }

    /// Query bank module cosmos sdk params
    pub(crate) async fn bank_query_params<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
    ) -> Result<ParamsResponse, BankError> {
        let req = QueryParamsRequest {};

        let res = client
            .client
            .query::<_, QueryParamsRequest, QueryParamsResponse>(
                req,
                "/cosmos.bank.v1beta1.Query/Params",
            )
            .await?;

        Ok(ParamsResponse {
            params: res.params.map(Into::into),
        })
    }
}
