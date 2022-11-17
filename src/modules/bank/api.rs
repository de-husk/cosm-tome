use cosmrs::proto::cosmos::bank::v1beta1::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest, QueryBalanceResponse,
    QueryDenomMetadataRequest, QueryDenomMetadataResponse, QueryDenomsMetadataRequest,
    QueryDenomsMetadataResponse, QueryParamsRequest, QueryParamsResponse,
    QuerySpendableBalancesRequest, QuerySpendableBalancesResponse, QuerySupplyOfRequest,
    QuerySupplyOfResponse, QueryTotalSupplyRequest, QueryTotalSupplyResponse,
};

use crate::{
    chain::{
        coin::Denom,
        request::{PaginationRequest, TxOptions},
    },
    clients::client::{CosmTome, CosmosClient},
    modules::{auth::model::Address, bank::model::SendResponse},
    signing_key::key::SigningKey,
};

use super::{
    error::BankError,
    model::{
        BalanceResponse, BalancesResponse, DenomMetadataResponse, DenomsMetadataResponse,
        ParamsResponse, SendRequest,
    },
};

impl<T: CosmosClient> CosmTome<T> {
    /// Send `amount` of funds from source (`from`) Address to destination (`to`) Address
    pub async fn bank_send(
        &self,
        req: SendRequest,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<SendResponse, BankError> {
        self.bank_send_batch(vec![req], key, tx_options).await
    }

    pub async fn bank_send_batch<I>(
        &self,
        reqs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<SendResponse, BankError>
    where
        I: IntoIterator<Item = SendRequest>,
    {
        let msgs = reqs
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        let tx_raw = self.tx_sign(msgs, key, tx_options).await?;

        let res = self.tx_broadcast_block(&tx_raw).await?;

        Ok(SendResponse { res })
    }

    /// Query the amount of `denom` currently held by an `address`
    pub async fn bank_query_balance(
        &self,
        address: Address,
        denom: Denom,
    ) -> Result<BalanceResponse, BankError> {
        let req = QueryBalanceRequest {
            address: address.into(),
            denom: denom.into(),
        };

        let res = self
            .client
            .query::<_, QueryBalanceResponse>(req, "/cosmos.bank.v1beta1.Query/Balance")
            .await?;

        // NOTE: we are unwrapping here, because unknown denoms still have a 0 balance returned here
        let balance = res.balance.unwrap().try_into()?;

        Ok(BalanceResponse { balance })
    }

    /// Query all denom balances held by an `address`
    pub async fn bank_query_balances(
        &self,
        address: Address,
        pagination: Option<PaginationRequest>,
    ) -> Result<BalancesResponse, BankError> {
        let req = QueryAllBalancesRequest {
            address: address.into(),
            pagination: pagination.map(Into::into),
        };

        let res = self
            .client
            .query::<_, QueryAllBalancesResponse>(req, "/cosmos.bank.v1beta1.Query/AllBalances")
            .await?;

        let balances = res
            .balances
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BalancesResponse {
            balances,
            next: res.pagination.map(Into::into),
        })
    }

    /// Get total spendable balance for an `address` (not currently locked away via delegation for example)
    pub async fn bank_query_spendable_balances(
        &self,
        address: Address,
        pagination: Option<PaginationRequest>,
    ) -> Result<BalancesResponse, BankError> {
        let req = QuerySpendableBalancesRequest {
            address: address.into(),
            pagination: pagination.map(Into::into),
        };

        let res = self
            .client
            .query::<_, QuerySpendableBalancesResponse>(
                req,
                "/cosmos.bank.v1beta1.Query/SpendableBalances",
            )
            .await?;

        let balances = res
            .balances
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BalancesResponse {
            balances,
            next: res.pagination.map(Into::into),
        })
    }

    /// Query global supply of `denom` for all accounts
    pub async fn bank_query_supply(&self, denom: Denom) -> Result<BalanceResponse, BankError> {
        let req = QuerySupplyOfRequest {
            denom: denom.into(),
        };

        let res = self
            .client
            .query::<_, QuerySupplyOfResponse>(req, "/cosmos.bank.v1beta1.Query/SupplyOf")
            .await?;

        // NOTE: we are unwrapping here, because unknown denoms still have a 0 balance returned here
        let balance = res.amount.unwrap().try_into()?;

        Ok(BalanceResponse { balance })
    }

    /// Query global supply of all denoms for all accounts
    pub async fn bank_query_total_supply(
        &self,
        pagination: Option<PaginationRequest>,
    ) -> Result<BalancesResponse, BankError> {
        let req = QueryTotalSupplyRequest {
            pagination: pagination.map(Into::into),
        };

        let res = self
            .client
            .query::<_, QueryTotalSupplyResponse>(req, "/cosmos.bank.v1beta1.Query/TotalSupply")
            .await?;

        let balances = res
            .supply
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BalancesResponse {
            balances,
            next: res.pagination.map(Into::into),
        })
    }

    /// Query bank metadata for a single denom
    pub async fn bank_query_denom_metadata(
        &self,
        denom: Denom,
    ) -> Result<DenomMetadataResponse, BankError> {
        let req = QueryDenomMetadataRequest {
            denom: denom.into(),
        };

        let res = self
            .client
            .query::<_, QueryDenomMetadataResponse>(req, "/cosmos.bank.v1beta1.Query/DenomMetadata")
            .await?;

        Ok(DenomMetadataResponse {
            meta: res.metadata.map(TryInto::try_into).transpose()?,
        })
    }

    /// Query bank metadata for all denoms
    pub async fn bank_query_denoms_metadata(
        &self,
        pagination: Option<PaginationRequest>,
    ) -> Result<DenomsMetadataResponse, BankError> {
        let req = QueryDenomsMetadataRequest {
            pagination: pagination.map(Into::into),
        };

        let res = self
            .client
            .query::<_, QueryDenomsMetadataResponse>(
                req,
                "/cosmos.bank.v1beta1.Query/DenomsMetadata",
            )
            .await?;

        Ok(DenomsMetadataResponse {
            metas: res
                .metadatas
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            next: res.pagination.map(Into::into),
        })
    }

    /// Query bank module cosmos sdk params
    pub async fn bank_query_params(&self) -> Result<ParamsResponse, BankError> {
        let req = QueryParamsRequest {};

        let res = self
            .client
            .query::<_, QueryParamsResponse>(req, "/cosmos.bank.v1beta1.Query/Params")
            .await?;

        Ok(ParamsResponse {
            params: res.params.map(TryInto::try_into).transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        chain::{
            error::ChainError,
            response::{ChainResponse, ChainTxResponse, Code},
        },
        clients::client::MockCosmosClient,
        modules::{bank::model::SendResponse, tx::error::TxError},
    };
    use cosmrs::proto::{
        cosmos::auth::v1beta1::{BaseAccount, QueryAccountRequest, QueryAccountResponse},
        traits::MessageExt,
    };

    use crate::{
        chain::{coin::Coin, fee::GasInfo, request::TxOptions},
        clients::client::CosmTome,
        config::cfg::ChainConfig,
        modules::bank::{error::BankError, model::SendRequest},
        signing_key::key::SigningKey,
    };

    #[tokio::test]
    async fn test_bank_send_empty() {
        let cfg = ChainConfig {
            denom: "utest".to_string(),
            prefix: "test".to_string(),
            chain_id: "test-1".to_string(),
            rpc_endpoint: Some("localhost".to_string()),
            grpc_endpoint: None,
            gas_prices: 0.1,
            gas_adjustment: 1.5,
        };

        let tx_options = TxOptions::default();
        let key = SigningKey::random_mnemonic("test_key".to_string());

        let cosm_tome = CosmTome {
            cfg: cfg.clone(),
            client: MockCosmosClient::new(),
        };

        // empty amount vec errors:
        let req = SendRequest {
            from: "juno10j9gpw9t4jsz47qgnkvl5n3zlm2fz72k67rxsg"
                .parse()
                .unwrap(),
            to: "juno1v9xynggs6vnrv2x5ufxdj398u2ghc5n9ya57ea"
                .parse()
                .unwrap(),
            amounts: vec![],
        };

        let res = cosm_tome
            .bank_send(req, &key, &tx_options)
            .await
            .err()
            .unwrap();

        assert!(matches!(res, BankError::EmptyAmount));

        // coin with 0 value errors:
        let req = SendRequest {
            from: "juno10j9gpw9t4jsz47qgnkvl5n3zlm2fz72k67rxsg"
                .parse()
                .unwrap(),
            to: "juno1v9xynggs6vnrv2x5ufxdj398u2ghc5n9ya57ea"
                .parse()
                .unwrap(),
            amounts: vec![
                Coin {
                    denom: cfg.denom.parse().unwrap(),
                    amount: 10,
                },
                Coin {
                    denom: cfg.denom.parse().unwrap(),
                    amount: 0,
                },
            ],
        };

        let res = cosm_tome
            .bank_send(req, &key, &tx_options)
            .await
            .err()
            .unwrap();

        assert!(matches!(res, BankError::EmptyAmount));
    }

    #[tokio::test]
    async fn test_bank_send() {
        let cfg = ChainConfig {
            denom: "utest".to_string(),
            prefix: "test".to_string(),
            chain_id: "test-1".to_string(),
            rpc_endpoint: None,
            grpc_endpoint: None,
            gas_prices: 0.1,
            gas_adjustment: 1.5,
        };
        let tx_options = TxOptions::default();
        let key = SigningKey::random_mnemonic("test_key".to_string());

        let mut mock_client = MockCosmosClient::new();

        mock_client
            .expect_query::<QueryAccountRequest, QueryAccountResponse>()
            .times(1)
            .returning(move |_, t: &str| {
                Ok(QueryAccountResponse {
                    account: Some(cosmrs::proto::Any {
                        type_url: t.to_owned(),
                        value: BaseAccount {
                            address: "juno10j9gpw9t4jsz47qgnkvl5n3zlm2fz72k67rxsg".to_string(),
                            pub_key: None,
                            account_number: 1337,
                            sequence: 1,
                        }
                        .to_bytes()
                        .unwrap(),
                    }),
                })
            });

        mock_client.expect_simulate_tx().times(1).returning(|_| {
            Ok(GasInfo {
                gas_wanted: 200u16.into(),
                gas_used: 100u16.into(),
            })
        });

        mock_client
            .expect_broadcast_tx_block()
            .times(1)
            .returning(|_| {
                Ok(ChainTxResponse {
                    res: ChainResponse {
                        code: Code::Ok,
                        data: None,
                        log: "log log log".to_string(),
                    },
                    events: vec![],
                    gas_wanted: 200,
                    gas_used: 100,
                    tx_hash: "TX_HASH_0".to_string(),
                    height: 1337,
                })
            });

        let cosm_tome = CosmTome {
            cfg: cfg.clone(),
            client: mock_client,
        };

        let req = SendRequest {
            from: "juno10j9gpw9t4jsz47qgnkvl5n3zlm2fz72k67rxsg"
                .parse()
                .unwrap(),
            to: "juno1v9xynggs6vnrv2x5ufxdj398u2ghc5n9ya57ea"
                .parse()
                .unwrap(),
            amounts: vec![Coin {
                denom: cfg.denom.parse().unwrap(),
                amount: 10,
            }],
        };

        let res = cosm_tome.bank_send(req, &key, &tx_options).await.unwrap();

        assert_eq!(
            res,
            SendResponse {
                res: ChainTxResponse {
                    res: ChainResponse {
                        code: Code::Ok,
                        data: None,
                        log: "log log log".to_string()
                    },
                    events: vec![],
                    gas_wanted: 200,
                    gas_used: 100,
                    tx_hash: "TX_HASH_0".to_string(),
                    height: 1337
                }
            }
        );
    }

    #[tokio::test]
    async fn test_bank_send_account_err() {
        let cfg = ChainConfig {
            denom: "utest".to_string(),
            prefix: "test".to_string(),
            chain_id: "test-1".to_string(),
            rpc_endpoint: None,
            grpc_endpoint: None,
            gas_prices: 0.1,
            gas_adjustment: 1.5,
        };

        let tx_options = TxOptions::default();
        let key = SigningKey::random_mnemonic("test_key".to_string());

        let mut mock_client = MockCosmosClient::new();

        mock_client
            .expect_query::<QueryAccountRequest, QueryAccountResponse>()
            .times(1)
            .returning(move |_, _| {
                Err(ChainError::CosmosSdk {
                    res: ChainResponse {
                        code: Code::Err(1),
                        data: None,
                        log: "error".to_string(),
                    },
                })
            });

        let cosm_tome = CosmTome {
            cfg: cfg.clone(),
            client: mock_client,
        };

        let req = SendRequest {
            from: "juno10j9gpw9t4jsz47qgnkvl5n3zlm2fz72k67rxsg"
                .parse()
                .unwrap(),
            to: "juno1v9xynggs6vnrv2x5ufxdj398u2ghc5n9ya57ea"
                .parse()
                .unwrap(),
            amounts: vec![Coin {
                denom: cfg.denom.parse().unwrap(),
                amount: 10,
            }],
        };

        let res = cosm_tome
            .bank_send(req, &key, &tx_options)
            .await
            .err()
            .unwrap();

        assert!(matches!(res, BankError::TxError(TxError::AccountError(..))));
    }

    // TODO: Add more happy path tests for other functions
}
