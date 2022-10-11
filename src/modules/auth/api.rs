use crate::clients::client::{CosmTome, CosmosClient};
use cosmos_sdk_proto::cosmos::auth::v1beta1::{QueryAccountRequest, QueryAccountResponse};
use cosmos_sdk_proto::traits::Message;

use super::error::AccountError;
use super::model::AccountResponse;

// TODO: Stop unwrapping

pub struct Auth {}

impl Auth {
    pub(crate) async fn auth_query<T: CosmosClient>(
        &self,
        client: &CosmTome<T>,
        address: String,
    ) -> Result<AccountResponse, AccountError> {
        let msg = QueryAccountRequest { address };

        let res = client
            .client
            .query::<_, QueryAccountRequest, QueryAccountResponse>(
                msg,
                "/cosmos.auth.v1beta1.Query/Account",
            )
            .await?;

        println!("{:?}", res);

        let account = res
            .account
            // .ok_or(ChainError::AccountId {
            //     id: address.to_string(),
            // })?;
            .unwrap();

        let base_account =
            cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount::decode(account.value.as_slice())
                //.map_err(ChainError::prost_proto_de)?;
                .unwrap();

        Ok(AccountResponse {
            account: base_account.try_into().unwrap(),
        })
    }
}
