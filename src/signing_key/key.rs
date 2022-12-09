use cosmrs::bip32;
use cosmrs::bip32::secp256k1::elliptic_curve::rand_core::OsRng;
use cosmrs::crypto::secp256k1;

#[cfg(feature = "os_keyring")]
use keyring::Entry;

use crate::chain::error::ChainError;
use crate::modules::auth::model::Address;

// https://github.com/confio/cosmos-hd-key-derivation-spec#the-cosmos-hub-path
const DERVIATION_PATH: &str = "m/44'/118'/0'/0/0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningKey {
    /// human readable key name
    pub name: String,
    /// private key associated with `name`
    pub key: Key,
}

impl SigningKey {
    pub fn to_addr(&self, prefix: &str) -> Result<Address, ChainError> {
        let key: secp256k1::SigningKey = self.try_into()?;
        let account = key
            .public_key()
            .account_id(prefix)
            .map_err(ChainError::crypto)?;
        Ok(account.into())
    }

    pub fn random_mnemonic(key_name: String) -> SigningKey {
        let mnemonic = bip32::Mnemonic::random(OsRng, Default::default());

        SigningKey {
            name: key_name,
            key: Key::Mnemonic(mnemonic.phrase().to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Key {
    /// Mnemonic allows you to pass the private key mnemonic words
    /// to Cosm-orc for configuring a transaction signing key.
    /// DO NOT USE FOR MAINNET
    Mnemonic(String),

    // TODO: Add Keyring password CRUD operations
    /// Use OS Keyring to access private key.
    /// Safe for testnet / mainnet.
    #[cfg(feature = "os_keyring")]
    Keyring(KeyringParams),
    // TODO: Add ledger support(under a new ledger feature flag / Key variant)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyringParams {
    pub service: String,
    pub key_name: String,
}

impl TryFrom<SigningKey> for secp256k1::SigningKey {
    type Error = ChainError;
    fn try_from(signer: SigningKey) -> Result<Self, Self::Error> {
        secp256k1::SigningKey::try_from(&signer)
    }
}

impl TryFrom<&SigningKey> for secp256k1::SigningKey {
    type Error = ChainError;
    fn try_from(signer: &SigningKey) -> Result<Self, Self::Error> {
        match &signer.key {
            Key::Mnemonic(phrase) => mnemonic_to_signing_key(phrase),

            #[cfg(feature = "os_keyring")]
            Key::Keyring(params) => {
                let entry = Entry::new(&params.service, &params.key_name);
                mnemonic_to_signing_key(&entry.get_password()?)
            }
        }
    }
}

fn mnemonic_to_signing_key(mnemonic: &str) -> Result<secp256k1::SigningKey, ChainError> {
    let seed = bip32::Mnemonic::new(mnemonic, bip32::Language::English)
        .map_err(|_| ChainError::Mnemonic)?
        .to_seed("");

    secp256k1::SigningKey::derive_from_path(seed, &DERVIATION_PATH.parse().unwrap())
        .map_err(|_| ChainError::DerviationPath)
}
