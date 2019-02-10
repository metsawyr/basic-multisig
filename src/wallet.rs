use super::proto;
use super::transaction::{ApprovedTransaction, PendingTransaction};
use exonum::crypto::{Hash, PublicKey};
use exonum_derive::ProtobufConvert;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, ProtobufConvert, Serialize, Deserialize)]
#[exonum(pb = "proto::Wallet")]
pub struct Wallet {
    pub pub_key: PublicKey,
    pub name: String,
    pub balance: u64,
    pub signers: Vec<PublicKey>,
    pub pending_txs: Vec<PendingTransaction>,
    pub txs: Vec<ApprovedTransaction>,
    pub history_len: u64,
    pub history_hash: Hash,
}

impl Wallet {
    pub fn new(
        &pub_key: &PublicKey,
        name: &str,
        balance: u64,
        signers: Vec<PublicKey>,
        pending_txs: Vec<PendingTransaction>,
        txs: Vec<ApprovedTransaction>,
        history_len: u64,
        &history_hash: &Hash,
    ) -> Self {
        Self {
            pub_key,
            name: name.to_owned(),
            balance,
            signers,
            pending_txs,
            txs,
            history_len,
            history_hash,
        }
    }

    pub fn set_balance(self, balance: u64, history_hash: &Hash) -> Self {
        Self::new(
            &self.pub_key,
            &self.name,
            balance,
            self.signers,
            self.pending_txs,
            self.txs,
            self.history_len + 1,
            history_hash,
        )
    }

    pub fn add_signer(self, pub_key: &PublicKey, history_hash: &Hash) -> Self {
        let mut signers = self.signers.clone();
        signers.push(*pub_key);

        Self::new(
            &self.pub_key,
            &self.name,
            self.balance,
            signers,
            self.pending_txs,
            self.txs,
            self.history_len + 1,
            history_hash,
        )
    }

    pub fn add_pending_tx(self, tx: PendingTransaction, history_hash: &Hash) -> Self {
        let mut pending_txs = self.pending_txs.clone();
        pending_txs.push(tx);

        Self::new(
            &self.pub_key,
            &self.name,
            self.balance,
            self.signers,
            pending_txs,
            self.txs,
            self.history_len + 1,
            history_hash,
        )
    }

    pub fn sign_pending_tx(self, tx_hash: &Hash, signer: &PublicKey, history_hash: &Hash) -> Self {
        let mut pending_txs = self.pending_txs.clone();
        let tx_index = pending_txs
            .iter()
            .position(|item| item.tx_hash == *tx_hash)
            .unwrap();
        let tx = &mut pending_txs[tx_index];
        tx.approvals.push(*signer);

        Self::new(
            &self.pub_key,
            &self.name,
            self.balance,
            self.signers,
            pending_txs,
            self.txs,
            self.history_len + 1,
            history_hash,
        )
    }

    pub fn remove_pending_tx(self, tx_hash: &Hash, history_hash: &Hash) -> Self {
        let mut pending_txs = self.pending_txs.clone();
        let tx_index = pending_txs
            .iter()
            .position(|item| item.tx_hash == *tx_hash)
            .unwrap();
        pending_txs.remove(tx_index);

        Self::new(
            &self.pub_key,
            &self.name,
            self.balance,
            self.signers,
            pending_txs,
            self.txs,
            self.history_len + 1,
            history_hash,
        )
    }

    pub fn add_approved_tx(self, tx: ApprovedTransaction, history_hash: &Hash) -> Self {
        let mut txs = self.txs.clone();
        txs.push(tx);

        Self::new(
            &self.pub_key,
            &self.name,
            self.balance,
            self.signers,
            self.pending_txs,
            txs,
            self.history_len + 1,
            history_hash,
        )
    }
}
