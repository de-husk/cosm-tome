use cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount;
use cosmrs::crypto::PublicKey;

use super::error::AccountError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Account {
    /// Bech32 address of account
    pub address: String,

    pub pubkey: Option<PublicKey>,

    pub account_number: u64,

    pub sequence: u64,
}

impl TryFrom<BaseAccount> for Account {
    type Error = AccountError;
    fn try_from(proto: BaseAccount) -> Result<Account, AccountError> {
        Ok(Account {
            // TODO: Dont unwrap()
            address: proto.address.parse().unwrap(),
            pubkey: proto.pub_key.map(PublicKey::try_from).transpose().unwrap(),
            account_number: proto.account_number,
            sequence: proto.sequence,
        })
    }
}

#[derive(Clone, Debug)]
pub struct AccountResponse {
    pub account: Account,
}
