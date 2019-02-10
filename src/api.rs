use super::schema::Schema;
use super::wallet::Wallet;
use exonum::api::{Error as ApiError, Result, ServiceApiBuilder, ServiceApiState};
use exonum::crypto::{Hash, PublicKey};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TransactionRequest {
    pub raw: String,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_hash: Hash,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionHex {
    pub tx_body: String,
}

pub struct Api;

impl Api {
    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/wallet", Self::get_wallet)
            .endpoint("v1/wallets", Self::get_wallets);
    }

    pub fn get_wallet(state: &ServiceApiState, query: WalletQuery) -> Result<Wallet> {
        let snapshot = state.snapshot();
        let schema = Schema::new(snapshot);

        schema
            .wallet(&query.wallet)
            .ok_or_else(|| ApiError::NotFound("Wallet not found".to_owned()))
    }

    pub fn get_wallets(state: &ServiceApiState, _query: ()) -> Result<Vec<Wallet>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(snapshot);
        let idx = schema.wallets();
        let wallets = idx.values().collect();

        Ok(wallets)
    }
}
