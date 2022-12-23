use cosmrs::proto::cosmos::tx::v1beta1::TxRaw;
use cosmrs::tx::{SignDoc, SignerInfo};
use cosmrs::{crypto::secp256k1, tx::Body};

use crate::chain::coin::{Coin, Denom};
use crate::chain::error::ChainError;
use crate::chain::response::AsyncChainTxResponse;
use crate::modules::auth::model::Account;
use crate::{
    chain::{fee::Fee, request::TxOptions, response::ChainTxResponse, Any},
    clients::client::{CosmTome, CosmosClient},
    signing_key::key::SigningKey,
};

use super::error::TxError;
use super::model::{BroadcastMode, RawTx};

// TODO: Query endpoints
// * tx_query_get_tx()
// * tx_query_get_txs_event()
// * tx_query_get_block_with_txs()

impl<T: CosmosClient> CosmTome<T> {
    pub async fn tx_sign<I>(
        &self,
        msgs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<RawTx, TxError>
    where
        I: IntoIterator<Item = Any>,
    {
        let sender_addr = key.to_addr(&self.cfg.prefix)?;
        let timeout_height = tx_options.timeout_height.unwrap_or_default();

        let tx = Body::new(msgs, &tx_options.memo, timeout_height);

        let account = self.auth_query_account(sender_addr).await?.account;
        let account_number = account.account_number;
        let sequence = account.sequence;

        // even if the user is supplying their own `Fee`, we will simulate the tx to ensure its valid
        let sim_fee = self.tx_simulate(tx.messages.clone(), account).await?; // TODO: Stop cloning msgs

        let fee = if let Some(fee) = &tx_options.fee {
            fee.clone()
        } else {
            sim_fee
        };

        // NOTE: if we are making requests in parallel with the same key, we need to serialize `account.sequence` to avoid errors
        let signing_key: secp256k1::SigningKey = key.try_into()?;
        let auth_info = SignerInfo::single_direct(Some(signing_key.public_key()), sequence)
            .auth_info(fee.try_into()?);

        let sign_doc = SignDoc::new(
            &tx,
            &auth_info,
            &self.cfg.chain_id.parse().map_err(|_| ChainError::ChainId {
                chain_id: self.cfg.chain_id.to_string(),
            })?,
            account_number,
        )
        .map_err(ChainError::proto_encoding)?;

        let raw = sign_doc.sign(&signing_key).map_err(ChainError::crypto)?;

        Ok(raw.into())
    }

    // Sends tx with an empty public_key / signature, like they do in the cosmos-sdk:
    // https://github.com/cosmos/cosmos-sdk/blob/main/client/tx/tx.go#L133
    pub async fn tx_simulate<I>(&self, msgs: I, account: Account) -> Result<Fee, TxError>
    where
        I: IntoIterator<Item = Any>,
    {
        let tx = Body::new(msgs, "cosm-client memo", 0u16);

        let denom: Denom = self.cfg.denom.parse()?;

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

        let gas_info = self.client.simulate_tx(&tx_raw.into()).await?;

        // TODO: clean up this gas conversion code to be clearer
        let gas_limit = (gas_info.gas_used.value() as f64 * self.cfg.gas_adjustment).ceil();
        let amount = Coin {
            denom,
            amount: ((gas_limit * self.cfg.gas_price).ceil() as u64).into(),
        };

        let fee = Fee::new(amount, gas_limit as u64, None, None);

        Ok(fee)
    }

    /// Non-blocking broadcast that will not wait for the tx to be committed in the next block.
    pub async fn tx_broadcast(
        &self,
        tx: &RawTx,
        mode: BroadcastMode,
    ) -> Result<AsyncChainTxResponse, TxError> {
        Ok(self.client.broadcast_tx(tx, mode).await?)
    }

    /// Blocking broadcast that will wait for the tx to be commited in the next block.
    pub async fn tx_broadcast_block(&self, tx: &RawTx) -> Result<ChainTxResponse, TxError> {
        Ok(self.client.broadcast_tx_block(tx).await?)
    }
}
