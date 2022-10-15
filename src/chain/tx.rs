use crate::clients::client::CosmosClient;
use crate::modules::auth::error::AccountError;
use crate::modules::auth::model::Account;
use crate::{clients::client::CosmTome, key::key::SigningKey};
use cosmos_sdk_proto::cosmos::tx::v1beta1::TxRaw;
use cosmos_sdk_proto::Any;
use cosmrs::tx::{SignDoc, SignerInfo};
use cosmrs::{
    crypto::secp256k1,
    tx::{Body, Raw},
};

use super::error::ChainError;
use super::fee::{Coin, Denom, Fee};
use super::request::TxOptions;

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
    let auth_info = SignerInfo::single_direct(Some(signing_key.public_key()), account.sequence)
        .auth_info(fee.try_into()?);

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
    let denom: Denom = client.cfg.denom.parse()?;

    let fee = Fee::new(
        Coin {
            denom: denom.clone(),
            amount: 0u128,
        },
        0u64,
        None,
        None,
    );

    let auth_info = SignerInfo::single_direct(None, account.sequence).auth_info(fee.try_into()?);

    let tx_raw = TxRaw {
        body_bytes: tx.into_bytes().map_err(ChainError::proto_encoding)?,
        auth_info_bytes: auth_info.into_bytes().map_err(ChainError::proto_encoding)?,
        signatures: vec![vec![]],
    };

    let gas_info = client.client.simulate_tx(&tx_raw.into()).await?;

    // TODO: clean up this gas conversion code to be clearer
    let gas_limit = (gas_info.gas_used.value() as f64 * client.cfg.gas_adjustment).ceil();
    let amount = Coin {
        denom,
        amount: ((gas_limit * client.cfg.gas_prices).ceil() as u64).into(),
    };

    Ok(Fee::new(amount, gas_limit as u64, None, None))
}
