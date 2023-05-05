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
    /// example: 0.025
    pub gas_price: f64,
    /// example: 1.3
    pub gas_adjustment: f64,
}
