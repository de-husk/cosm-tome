use core::fmt::Debug;
use cosmrs::{proto::traits::TypeUrl, tx::MessageExt, Any};
use std::fmt::Display;

use super::error::ChainError;

pub trait Msg:
    Clone + Sized + TryFrom<Self::Proto, Error = Self::Err> + TryInto<Self::Proto, Error = Self::Err>
{
    /// Protobuf type
    type Proto: Default + MessageExt + Sized + TypeUrl;

    /// Protobuf conversion error type
    type Err: From<ChainError> + Debug + Display;

    /// Parse this message proto from [`Any`].
    fn from_any(any: &Any) -> Result<Self, Self::Err> {
        Self::Proto::from_any(any)
            .map_err(ChainError::prost_proto_decoding)?
            .try_into()
    }

    /// Serialize this message proto as [`Any`].
    fn to_any(&self) -> Result<Any, Self::Err> {
        self.clone().into_any()
    }

    /// Convert this message proto into [`Any`].
    fn into_any(self) -> Result<Any, Self::Err> {
        Ok(self
            .try_into()?
            .to_any()
            .map_err(ChainError::prost_proto_encoding)?)
    }
}
