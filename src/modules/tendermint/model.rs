use cosmos_sdk_proto::{
    cosmos::base::tendermint::v1beta1::GetLatestBlockResponse,
    tendermint::types::{Block, BlockId},
};
use serde::{Deserialize, Serialize};

use super::error::TendermintError;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockResponse {
    // TODO: Dont expose these protos externally, make my own types
    pub id: BlockId,
    pub block: Block,
}

impl TryFrom<GetLatestBlockResponse> for BlockResponse {
    type Error = TendermintError;

    fn try_from(res: GetLatestBlockResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            id: res.block_id.ok_or(TendermintError::MissingBlockId)?,
            block: res.block.ok_or(TendermintError::MissingBlock)?,
        })
    }
}

impl From<BlockResponse> for GetLatestBlockResponse {
    fn from(res: BlockResponse) -> Self {
        Self {
            block_id: Some(res.id),
            block: Some(res.block),
        }
    }
}
