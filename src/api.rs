use super::schema::Schema;
use super::transaction::Transactions;
use super::wallet::Wallet;
use exonum::api::{Error, Result, ServiceApiBuilder, ServiceApiState};
use exonum::blockchain::Transaction;
use exonum::crypto::{Hash, PublicKey};
use exonum::node::TransactionSend;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_hash: Hash,
    pub tx_index: usize,
}

#[derive(Serialize, Deserialize)]
pub struct WalletQuery {
    wallet: PublicKey,
}

#[derive(Serialize, Deserialize)]
pub struct SignerQuery {
    wallet: PublicKey,
    signer: PublicKey,
}

pub struct Api;

impl Api {
    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/wallet", Self::get_wallet)
            .endpoint("v1/wallets", Self::get_wallets)
            .endpoint_mut("v1/wallets/signers/add", Self::post_transaction)
            .endpoint_mut("v1/wallets", Self::post_transaction)
            .endpoint_mut("v1/wallets/sign", Self::post_transaction)
            .endpoint_mut("v1/wallets/transfer", Self::post_transaction);
    }

    pub fn get_wallet(state: &ServiceApiState, query: WalletQuery) -> Result<Wallet> {
        let snapshot = state.snapshot();
        let schema = Schema::new(snapshot);

        schema
            .wallet(&query.wallet)
            .ok_or_else(|| Error::NotFound("Wallet not found".to_owned()))
    }

    pub fn get_wallets(state: &ServiceApiState, _query: ()) -> Result<Vec<Wallet>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(snapshot);
        let idx = schema.wallets();
        let wallets = idx.values().collect();

        Ok(wallets)
    }

    pub fn post_transaction(
        state: &ServiceApiState,
        query: Transactions,
    ) -> Result<TransactionResponse> {
        let transaction: Box<Transaction> = query.into();
        let tx_hash = transaction.hash();
        state.sender().send(transaction)?;

        Ok(TransactionResponse {
            tx_hash,
            tx_index: 0,
        })
    }
}
