use crate::chain::error::ChainError;
use crate::chain::request::PaginationRequest;
use crate::clients::client::CosmosClientQuery;
use crate::modules::auth::model::Account;
use async_trait::async_trait;
use cosmrs::proto::cosmos::auth::v1beta1::{
    BaseAccount, QueryAccountRequest, QueryAccountResponse, QueryAccountsRequest,
    QueryAccountsResponse, QueryParamsRequest, QueryParamsResponse,
};
use cosmrs::proto::traits::Message;

use super::error::AccountError;
use super::model::{AccountResponse, AccountsResponse, Address, ParamsResponse};

impl<T> Auth for T where T: CosmosClientQuery {}

#[async_trait]
pub trait Auth: CosmosClientQuery + Sized {
    async fn auth_query_account(&self, address: Address) -> Result<AccountResponse, AccountError> {
        let req = QueryAccountRequest {
            address: address.into(),
        };

        let res = self
            .query::<_, QueryAccountResponse>(req, "/cosmos.auth.v1beta1.Query/Account")
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

    async fn auth_query_accounts(
        &self,
        pagination: Option<PaginationRequest>,
    ) -> Result<AccountsResponse, AccountError> {
        let req = QueryAccountsRequest {
            pagination: pagination.map(Into::into),
        };

        let res = self
            .query::<_, QueryAccountsResponse>(req, "/cosmos.auth.v1beta1.Query/Accounts")
            .await?;

        let accounts: Vec<Account> = res
            .accounts
            .into_iter()
            .map(|a| {
                BaseAccount::decode(a.value.as_slice())
                    .map_err(ChainError::prost_proto_decoding)?
                    .try_into()
            })
            .collect::<Result<Vec<Account>, AccountError>>()?;

        Ok(AccountsResponse {
            accounts,
            next: res.pagination.map(Into::into),
        })
    }

    async fn auth_query_params(&self) -> Result<ParamsResponse, AccountError> {
        let req = QueryParamsRequest {};

        let res = self
            .query::<_, QueryParamsResponse>(req, "/cosmos.auth.v1beta1.Query/Params")
            .await?;

        Ok(ParamsResponse {
            params: res.params.map(Into::into),
        })
    }
}
