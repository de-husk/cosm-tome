use serde::{Deserialize, Serialize};

// TODO: Create a way to use the cosmos chain registry instead of manual

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChainConfig {
    pub denom: String, // TODO: Use my denom type
    pub prefix: String,
    pub chain_id: String,
    pub rpc_endpoint: Option<String>,
    pub grpc_endpoint: Option<String>,
    pub gas_prices: f64,
    pub gas_adjustment: f64,
}