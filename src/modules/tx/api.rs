use async_trait::async_trait;
use cosmrs::proto::cosmos::tx::v1beta1::TxRaw;
use cosmrs::tx::Body;
use cosmrs::tx::SignerInfo;
use serde::Serialize;

use crate::chain::coin::{Coin, Denom};
use crate::chain::error::ChainError;
use crate::chain::msg::Msg;
use crate::clients::client::CosmosClientQuery;
use crate::config::cfg::ChainConfig;
use crate::modules::auth::api::Auth;
use crate::modules::auth::model::Account;
use crate::{
    chain::{fee::Fee, request::TxOptions, Any},
    signing_key::key::SigningKey,
};

use super::error::TxError;
use super::model::RawTx;

// TODO: Query endpoints
// * tx_query_get_tx()
// * tx_query_get_txs_event()
// * tx_query_get_block_with_txs()

impl<T> Tx for T where T: CosmosClientQuery {}

#[async_trait]
pub trait Tx: CosmosClientQuery + Sized {
    async fn tx_sign<T>(
        &self,
        chain_cfg: &ChainConfig,
        msgs: Vec<T>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<RawTx, TxError>
    where
        T: Msg + Serialize + Send + Sync,
        <T as Msg>::Err: Send + Sync,
    {
        let sender_addr = key
            .to_addr(&chain_cfg.prefix, &chain_cfg.derivation_path)
            .await?;

        let timeout_height = tx_options.timeout_height.unwrap_or_default();

        let account = if let Some(ref account) = tx_options.account {
            account.clone()
        } else {
            self.auth_query_account(sender_addr).await?.account
        };

        let fee = if let Some(fee) = &tx_options.fee {
            fee.clone()
        } else {
            self.tx_simulate(
                &chain_cfg.denom,
                chain_cfg.gas_price,
                chain_cfg.gas_adjustment,
                msgs.iter()
                    .map(|m| m.to_any())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| ChainError::ProtoEncoding {
                        message: e.to_string(),
                    })?,
                &account,
            )
            .await?
        };

        let raw = key
            .sign(
                msgs,
                timeout_height,
                &tx_options.memo,
                account,
                fee,
                &chain_cfg.chain_id,
                &chain_cfg.derivation_path,
            )
            .await?;
        Ok(raw)
    }

    // Sends tx with an empty public_key / signature, like they do in the cosmos-sdk:
    // https://github.com/cosmos/cosmos-sdk/blob/main/client/tx/tx.go#L133
    async fn tx_simulate<I>(
        &self,
        denom: &str,
        gas_price: f64,
        gas_adjustment: f64,
        msgs: I,
        account: &Account,
    ) -> Result<Fee, TxError>
    where
        I: IntoIterator<Item = Any> + Send,
    {
        let tx = Body::new(msgs, "cosm-client memo", 0u16);

        let denom: Denom = denom.parse()?;

        let fee = Fee::new(
            Coin {
                denom: denom.clone(),
                amount: 0u128,
            },
            0u64,
            None,
            None,
        );

        let auth_info =
            SignerInfo::single_direct(None, account.sequence).auth_info(fee.try_into()?);

        let tx_raw = TxRaw {
            body_bytes: tx.into_bytes().map_err(ChainError::proto_encoding)?,
            auth_info_bytes: auth_info.into_bytes().map_err(ChainError::proto_encoding)?,
            signatures: vec![vec![]],
        };

        let gas_info = self.simulate_tx(&tx_raw.into()).await?;

        // TODO: clean up this gas conversion code to be clearer
        let gas_limit = (gas_info.gas_used.value() as f64 * gas_adjustment).ceil();
        let amount = Coin {
            denom,
            amount: ((gas_limit * gas_price).ceil() as u64).into(),
        };

        let fee = Fee::new(amount, gas_limit as u64, None, None);

        Ok(fee)
    }
}
