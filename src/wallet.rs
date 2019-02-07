use super::transaction::{Approval, PendingTx};
use exonum::crypto::PublicKey;
use exonum::encoding_struct;

encoding_struct! {
    struct Wallet {
        pub_key: &PublicKey,
        name: &str,
        balance: u64,
        signers: Vec<Signer>,
        pending_txs: Vec<PendingTx>,
    }
}

impl Wallet {
    pub fn increase_balance(self, amount: u64) -> Self {
        let balance = self.balance() + amount;

        Self::new(
            self.pub_key(),
            self.name(),
            balance,
            self.signers(),
            self.pending_txs(),
        )
    }

    pub fn decrease_balance(self, amount: u64) -> Self {
        let balance = self.balance() - amount;

        Self::new(
            self.pub_key(),
            self.name(),
            balance,
            self.signers(),
            self.pending_txs(),
        )
    }

    pub fn add_signer(self, pub_key: &PublicKey) -> Self {
        let mut signers = self.signers();
        signers.push(Signer::new(pub_key));

        Self::new(
            self.pub_key(),
            self.name(),
            self.balance(),
            signers,
            self.pending_txs(),
        )
    }

    pub fn add_pending_tx(self, tx: PendingTx) -> Self {
        let mut pending_txs = self.pending_txs();
        pending_txs.push(tx);

        Self::new(
            self.pub_key(),
            self.name(),
            self.balance(),
            self.signers(),
            pending_txs,
        )
    }

    pub fn sign_pending_tx(self, tx_index: usize, approval: Approval) -> Self {
        let pending_txs = self.pending_txs();
        pending_txs[tx_index].approvals().push(approval);

        Self::new(
            self.pub_key(),
            self.name(),
            self.balance(),
            self.signers(),
            pending_txs,
        )
    }

    pub fn remove_pending_tx(self, tx_index: usize) -> Self {
        let mut pending_txs = self.pending_txs();
        pending_txs.remove(tx_index);

        Self::new(
            self.pub_key(),
            self.name(),
            self.balance(),
            self.signers(),
            pending_txs,
        )
    }
}

encoding_struct! {
    struct Signer {
        pub_key: &PublicKey,
    }
}

impl From<&PublicKey> for Signer {
    fn from(pub_key: &PublicKey) -> Self {
        Self::new(pub_key)
    }
}
