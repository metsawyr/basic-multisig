use super::schema::Schema;
use super::wallet::Wallet;
use super::transaction::ApprovedTransaction;
use exonum::api::{Error as ApiError, Result, ServiceApiBuilder, ServiceApiState};
use exonum::crypto::PublicKey;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WalletQuery {
    pub_key: PublicKey,
}

#[derive(Serialize, Deserialize)]
pub struct SignerQuery {
    wallet: PublicKey,
    signer: PublicKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionHex {
    pub tx_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionsQuery {
    pub pub_key: PublicKey,
}

pub struct Api;

impl Api {
    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/wallet", Self::get_wallet)
            .endpoint("v1/wallets", Self::get_wallets)
            .endpoint("v1/wallet/txs", Self::get_approved_txs);
    }

    pub fn get_wallet(state: &ServiceApiState, query: WalletQuery) -> Result<Wallet> {
        let snapshot = state.snapshot();
        let schema = Schema::new(snapshot);

        schema
            .wallet(&query.pub_key)
            .ok_or_else(|| ApiError::NotFound("Wallet not found".to_owned()))
    }

    pub fn get_approved_txs(state: &ServiceApiState, query: TransactionsQuery) -> Result<Vec<ApprovedTransaction>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(snapshot);

        schema
            .wallet(&query.pub_key)
            .ok_or_else(|| ApiError::NotFound("Wallet not found".to_owned()))
            .map(|wallet| wallet.txs)
    }

    pub fn get_wallets(state: &ServiceApiState, _query: ()) -> Result<Vec<Wallet>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(snapshot);
        let idx = schema.wallets();
        let wallets = idx.values().collect();

        Ok(wallets)
    }
}
