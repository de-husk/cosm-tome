use cosmrs::proto::cosmos::base::tendermint::v1beta1::{
    GetLatestBlockRequest, GetLatestBlockResponse, GetLatestValidatorSetRequest,
    GetLatestValidatorSetResponse, GetValidatorSetByHeightRequest, GetValidatorSetByHeightResponse,
};

use crate::{
    chain::request::PaginationRequest,
    clients::client::{CosmTome, CosmosClient},
};

use super::{
    error::TendermintError,
    model::{BlockResponse, LatestValidatorSetResponse},
};

impl<T: CosmosClient> CosmTome<T> {
    pub async fn tendermint_query_latest_block(&self) -> Result<BlockResponse, TendermintError> {
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

    pub async fn tendermint_query_latest_validator_set(
        &self,
        pagination: Option<PaginationRequest>,
    ) -> Result<LatestValidatorSetResponse, TendermintError> {
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

    pub async fn tendermint_query_validator_set_at_height(
        &self,
        block_height: u64,
        pagination: Option<PaginationRequest>,
    ) -> Result<LatestValidatorSetResponse, TendermintError> {
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
