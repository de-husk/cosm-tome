use serde::{Deserialize, Serialize};

// TODO:
// * Create easy way to read in the manual config from a yaml file
// * Create a way to use the cosmos chain registry instead of manual

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainConfig {
    pub denom: String,
    pub prefix: String,
    pub chain_id: String,
    pub rpc_endpoint: String,
    pub grpc_endpoint: String,
    pub gas_prices: f64,
    pub gas_adjustment: f64,
}
