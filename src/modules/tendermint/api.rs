use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::{
    GetLatestBlockRequest, GetLatestBlockResponse,
};

use crate::clients::client::{CosmTome, CosmosClient};

use super::{error::TendermintError, model::BlockResponse};

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
}
