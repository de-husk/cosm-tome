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

pub async fn sign_tx<T: CosmosClient>(
    client: &CosmTome<T>,
    msg: Any,
    key: &SigningKey,
    fee: Option<Fee>,
    account_addr: String,
) -> Result<Raw, AccountError> {
    // TODO: Allow people to set (Fee, timeout_height, memo, simulate, etc) for each non,query tx
    let timeout_height = 0u16;
    let memo = "Made with cosm-client";

    let tx = Body::new(vec![msg], memo, timeout_height);

    let account = client.auth_query(account_addr).await?.account;

    let fee = if let Some(fee) = fee {
        fee
    } else {
        simulate_tx(client, tx.clone(), &account).await?
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
// https://github.com/cosmos/cosmos-sdk/blob/main/client/tx/factory.go#L359
// https://github.com/cosmos/cosmos-sdk/blob/main/x/auth/tx/builder.go#L292
pub async fn simulate_tx<T: CosmosClient>(
    client: &CosmTome<T>,
    tx: Body,
    account: &Account,
) -> Result<Fee, ChainError> {
    let denom: Denom = client.cfg.denom.parse().unwrap();

    let auth_info =
        SignerInfo::single_direct(None, account.sequence).auth_info(Fee::from_amount_and_gas(
            Coin {
                denom: denom.clone(),
                amount: 0u64.into(),
            },
            0u64,
        ));

    let tx_raw = TxRaw {
        body_bytes: tx.into_bytes().unwrap(),
        auth_info_bytes: auth_info.into_bytes().unwrap(),
        signatures: vec![vec![]],
    };

    let gas_info = client.client.simulate_tx(&tx_raw.into()).await.unwrap();

    // TODO: clean up this gas conversion code to be clearer
    let gas_limit = (gas_info.gas_used as f64 * client.cfg.gas_adjustment).ceil();
    let amount = Coin {
        denom: denom,
        amount: ((gas_limit * client.cfg.gas_prices).ceil() as u64).into(),
    };

    Ok(Fee::from_amount_and_gas(amount, gas_limit as u64))
}
