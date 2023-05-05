use async_trait::async_trait;
use cosmrs::proto::cosmos::base::tendermint::v1beta1::{
    GetLatestBlockRequest, GetLatestBlockResponse, GetLatestValidatorSetRequest,
    GetLatestValidatorSetResponse, GetValidatorSetByHeightRequest, GetValidatorSetByHeightResponse,
};

use crate::{chain::request::PaginationRequest, clients::client::CosmosClientQuery};

use super::{
    error::TendermintError,
    model::{BlockResponse, ValidatorSetResponse},
};

impl<T> TendermintQuery for T where T: CosmosClientQuery {}

#[async_trait]
pub trait TendermintQuery: CosmosClientQuery + Sized {
    async fn tendermint_query_latest_block(&self) -> Result<BlockResponse, TendermintError> {
        let req = GetLatestBlockRequest {};

        let res = self
            .client
            .query::<_, GetLatestBlockResponse>(
                req,
                "/cosmos.base.tendermint.v1beta1.Service/GetLatestBlock",
            )
            .await?;

        res.try_into()
    }

    async fn tendermint_query_latest_validator_set(
        &self,
        pagination: Option<PaginationRequest>,
    ) -> Result<ValidatorSetResponse, TendermintError> {
        let req = GetLatestValidatorSetRequest {
            pagination: pagination.map(Into::into),
        };

        let res = self
            .client
            .query::<_, GetLatestValidatorSetResponse>(
                req,
                "/cosmos.base.tendermint.v1beta1.Service/GetLatestValidatorSet",
            )
            .await?;

        res.try_into()
    }

    async fn tendermint_query_validator_set_at_height(
        &self,
        block_height: u64,
        pagination: Option<PaginationRequest>,
    ) -> Result<ValidatorSetResponse, TendermintError> {
        let req = GetValidatorSetByHeightRequest {
            height: block_height as i64,
            pagination: pagination.map(Into::into),
        };

        let res = self
            .client
            .query::<_, GetValidatorSetByHeightResponse>(
                req,
                "/cosmos.base.tendermint.v1beta1.Service/GetValidatorSetByHeight",
            )
            .await?;

        res.try_into()
    }
}
