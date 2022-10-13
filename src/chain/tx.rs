use crate::clients::client::CosmosClient;
use crate::modules::auth::error::AccountError;
use crate::modules::auth::model::Account;
use crate::{clients::client::CosmTome, key::key::SigningKey};
use cosmos_sdk_proto::cosmos::tx::v1beta1::TxRaw;
use cosmos_sdk_proto::Any;
use cosmrs::tx::{Fee, SignDoc, SignerInfo};
use cosmrs::{
    crypto::secp256k1,
    tx::{Body, Raw},
};
use cosmrs::{Coin, Denom};

use super::error::ChainError;

/// Options the user can set when executing txs on chain
pub struct TxOptions {
    /// The block height after which this transaction will not be processed by the chain
    pub timeout_height: Option<u16>,

    /// If set will use this fee, instead of the simulated gas price
    pub fee: Option<Fee>,

    /// An arbitrary memo to be added to the transaction
    pub memo: Option<String>,
    // TODO: Broadcast mode (block, async, etc)
}

impl Default for TxOptions {
    fn default() -> Self {
        Self {
            fee: None,
            timeout_height: Some(0),
            memo: Some("Made with cosm-client".to_string()),
        }
    }
}

pub async fn sign_tx<T: CosmosClient>(
    client: &CosmTome<T>,
    msg: Any,
    key: &SigningKey,
    account_addr: String,
    tx_options: &TxOptions,
) -> Result<Raw, AccountError> {
    let timeout_height = tx_options.timeout_height.unwrap_or_default();
    let memo = tx_options.memo.clone().unwrap_or_default();

    let tx = Body::new(vec![msg], memo, timeout_height);

    let account = client.auth_query_account(account_addr).await?.account;

    // even if the user is supplying their own `Fee`, we will simulate the tx to ensure its valid
    let sim_fee = simulate_tx(client, tx.clone(), &account).await?;

    let fee = if let Some(fee) = &tx_options.fee {
        fee.clone()
    } else {
        sim_fee
    };

    // NOTE: if we are making requests in parallel with the same key, we need to serialize `account.sequence` to avoid errors
    let signing_key: secp256k1::SigningKey = key.try_into()?;
    let auth_info =
        SignerInfo::single_direct(Some(signing_key.public_key()), account.sequence).auth_info(fee);

    let sign_doc = SignDoc::new(
        &tx,
        &auth_info,
        &client
            .cfg
            .chain_id
            .parse()
            .map_err(|_| ChainError::ChainId {
                chain_id: client.cfg.chain_id.to_string(),
            })?,
        account.account_number,
    )
    .map_err(ChainError::proto_encoding)?;

    let tx_raw = sign_doc.sign(&signing_key).map_err(ChainError::crypto)?;

    Ok(tx_raw)
}

// Sends tx with an empty public_key / signature, like they do in the cosmos-sdk:
// https://github.com/cosmos/cosmos-sdk/blob/main/client/tx/tx.go#L133
pub async fn simulate_tx<T: CosmosClient>(
    client: &CosmTome<T>,
    tx: Body,
    account: &Account,
) -> Result<Fee, ChainError> {
    let denom: Denom = client.cfg.denom.parse().map_err(|_| ChainError::Denom {
        name: client.cfg.denom.clone(),
    })?;

    let auth_info =
        SignerInfo::single_direct(None, account.sequence).auth_info(Fee::from_amount_and_gas(
            Coin {
                denom: denom.clone(),
                amount: 0u64.into(),
            },
            0u64,
        ));

    let tx_raw = TxRaw {
        body_bytes: tx.into_bytes().map_err(ChainError::proto_encoding)?,
        auth_info_bytes: auth_info.into_bytes().map_err(ChainError::proto_encoding)?,
        signatures: vec![vec![]],
    };

    let gas_info = client.client.simulate_tx(&tx_raw.into()).await?;

    // TODO: clean up this gas conversion code to be clearer
    let gas_limit = (gas_info.gas_used as f64 * client.cfg.gas_adjustment).ceil();
    let amount = Coin {
        denom: denom,
        amount: ((gas_limit * client.cfg.gas_prices).ceil() as u64).into(),
    };

    Ok(Fee::from_amount_and_gas(amount, gas_limit as u64))
}
