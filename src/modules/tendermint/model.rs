use cosmrs::{
    crypto::PublicKey,
    proto::{
        cosmos::base::tendermint::v1beta1::{
            GetLatestBlockResponse, GetLatestValidatorSetResponse, GetValidatorSetByHeightResponse,
            Validator as ProtoValidator,
        },
        tendermint::types::{Block, BlockId},
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    chain::{error::ChainError, request::PaginationResponse},
    modules::auth::model::Address,
};

use super::error::TendermintError;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockResponse {
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

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Validator {
    pub address: Address,
    pub pubkey: Option<PublicKey>,
    pub voting_power: i64,
    pub proposer_priority: i64,
}

impl From<Validator> for ProtoValidator {
    fn from(val: Validator) -> Self {
        Self {
            address: val.address.into(),
            pub_key: val.pubkey.map(Into::into),
            voting_power: val.voting_power,
            proposer_priority: val.proposer_priority,
        }
    }
}

impl TryFrom<ProtoValidator> for Validator {
    type Error = TendermintError;

    fn try_from(val: ProtoValidator) -> Result<Self, Self::Error> {
        Ok(Self {
            address: val.address.parse()?,
            pubkey: val
                .pub_key
                .map(TryFrom::try_from)
                .transpose()
                .map_err(ChainError::crypto)?,
            voting_power: val.voting_power,
            proposer_priority: val.proposer_priority,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ValidatorSetResponse {
    pub block_height: u64,
    pub validators: Vec<Validator>,
    pub next: Option<PaginationResponse>,
}

impl TryFrom<GetLatestValidatorSetResponse> for ValidatorSetResponse {
    type Error = TendermintError;

    fn try_from(res: GetLatestValidatorSetResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            block_height: res.block_height as u64,
            validators: res
                .validators
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            next: res.pagination.map(Into::into),
        })
    }
}

impl From<ValidatorSetResponse> for GetLatestValidatorSetResponse {
    fn from(res: ValidatorSetResponse) -> Self {
        Self {
            block_height: res.block_height as i64,
            validators: res.validators.into_iter().map(Into::into).collect(),
            pagination: res.next.map(Into::into),
        }
    }
}

impl TryFrom<GetValidatorSetByHeightResponse> for ValidatorSetResponse {
    type Error = TendermintError;

    fn try_from(res: GetValidatorSetByHeightResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            block_height: res.block_height as u64,
            validators: res
                .validators
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            next: res.pagination.map(Into::into),
        })
    }
}

impl From<ValidatorSetResponse> for GetValidatorSetByHeightResponse {
    fn from(res: ValidatorSetResponse) -> Self {
        Self {
            block_height: res.block_height as i64,
            validators: res.validators.into_iter().map(Into::into).collect(),
            pagination: res.next.map(Into::into),
        }
    }
}
