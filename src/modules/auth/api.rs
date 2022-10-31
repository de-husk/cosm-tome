use crate::chain::error::ChainError;
use crate::chain::request::PaginationRequest;
use crate::clients::client::{CosmTome, CosmosClient};
use crate::modules::auth::model::Account;
use cosmos_sdk_proto::cosmos::auth::v1beta1::{
    BaseAccount, QueryAccountRequest, QueryAccountResponse, QueryAccountsRequest,
    QueryAccountsResponse, QueryParamsRequest, QueryParamsResponse,
};
use cosmos_sdk_proto::traits::Message;

use super::error::AccountError;
use super::model::{AccountResponse, AccountsResponse, Address, ParamsResponse};

#[derive(Clone, Debug)]
pub struct Auth {}

impl Auth {
    pub(crate) async fn auth_query_account<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
        address: Address,
    ) -> Result<AccountResponse, AccountError> {
        let req = QueryAccountRequest {
            address: address.into(),
        };

        let res = client
            .client
            .query::<_, QueryAccountRequest, QueryAccountResponse>(
                req,
                "/cosmos.auth.v1beta1.Query/Account",
            )
            .await?;

        let account = res.account.ok_or(AccountError::Address {
            message: "Invalid account address".to_string(),
        })?;

        let base_account = BaseAccount::decode(account.value.as_slice())
            .map_err(ChainError::prost_proto_decoding)?;

        Ok(AccountResponse {
            account: base_account.try_into()?,
        })
    }

    pub(crate) async fn auth_query_accounts<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
        pagination: Option<PaginationRequest>,
    ) -> Result<AccountsResponse, AccountError> {
        let req = QueryAccountsRequest {
            pagination: pagination.map(Into::into),
        };

        let res = client
            .client
            .query::<_, QueryAccountsRequest, QueryAccountsResponse>(
                req,
                "/cosmos.auth.v1beta1.Query/Accounts",
            )
            .await?;

        let accounts: Vec<Account> = res
            .accounts
            .into_iter()
            .map(|a| {
                let base_account = BaseAccount::decode(a.value.as_slice())
                    .map_err(ChainError::prost_proto_decoding)?;
                Ok(base_account.try_into()?)
            })
            .collect::<Result<Vec<Account>, AccountError>>()?;

        Ok(AccountsResponse {
            accounts,
            next: res.pagination.map(Into::into),
        })
    }

    pub(crate) async fn auth_query_params<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
    ) -> Result<ParamsResponse, AccountError> {
        let req = QueryParamsRequest {};

        let res = client
            .client
            .query::<_, QueryParamsRequest, QueryParamsResponse>(
                req,
                "/cosmos.auth.v1beta1.Query/Params",
            )
            .await?;

        Ok(ParamsResponse {
            params: res.params.map(Into::into),
        })
    }
}
