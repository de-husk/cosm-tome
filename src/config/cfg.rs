use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// TODO: Create a way to use the cosmos chain registry instead of manual

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ChainConfig {
    /// example: "uatom"
    pub denom: String,
    /// example: "cosmos"
    pub prefix: String,
    /// example: "cosmoshub-4"
    pub chain_id: String,
    /// example: "m/44'/118'/0'/0/0"
    pub derivation_path: String,
    /// example: "https://terra-testnet-rpc.polkachu.com"
    pub rpc_endpoint: Option<String>,
    /// example: "https://terra-testnet-grpc.polkachu.com:11790"
    pub grpc_endpoint: Option<String>,
    /// example: 0.025
    pub gas_prices: f64,
    /// example: 1.3
    pub gas_adjustment: f64,
}
