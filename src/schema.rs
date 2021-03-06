use super::transaction::{ApprovedTransaction, PendingTransaction, SignTx};
use super::wallet::Wallet;
use exonum::crypto::{Hash, PublicKey};
use exonum::storage::{Fork, ProofListIndex, ProofMapIndex, Snapshot};

pub struct Schema<T> {
    view: T,
}

impl<T> Schema<T>
where
    T: AsRef<Snapshot>,
{
    pub fn new(view: T) -> Self {
        Schema { view }
    }

    pub fn wallets(&self) -> ProofMapIndex<&T, PublicKey, Wallet> {
        ProofMapIndex::new("wallets", &self.view)
    }

    pub fn wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }

    pub fn awaiting_txs(&self) -> ProofMapIndex<&T, Hash, SignTx> {
        ProofMapIndex::new("awaiting_txs", &self.view)
    }
}

impl<'a> Schema<&'a mut Fork> {
    pub fn wallets_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Wallet> {
        ProofMapIndex::new("wallets", &mut self.view)
    }

    pub fn awaiting_txs_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, SignTx> {
        ProofMapIndex::new("awaiting_txs", &mut self.view)
    }

    pub fn wallet_history_mut(
        &mut self,
        public_key: &PublicKey,
    ) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("history", public_key, &mut self.view)
    }

    pub fn create_wallet(&mut self, key: &PublicKey, name: &str, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(key);
            history.push(*transaction);

            let history_hash = history.merkle_root();

            Wallet::new(
                key,
                name,
                100,
                vec![],
                vec![],
                vec![],
                history.len(),
                &history_hash,
            )
        };

        println!("Creating a wallet {:?}", wallet);
        self.wallets_mut().put(key, wallet);
    }

    pub fn add_signer(
        &mut self,
        wallet: &Wallet,
        signer: &PublicKey,
        transaction: &Hash,
    ) -> Wallet {
        let new_wallet = {
            let mut history = self.wallet_history_mut(&wallet.pub_key);
            history.push(*transaction);

            let history_hash = history.merkle_root();

            wallet.clone().add_signer(signer, &history_hash)
        };

        println!(
            "Adding signer `{}` to the wallet {}",
            signer.to_hex(),
            wallet.pub_key.to_hex()
        );
        self.wallets_mut().put(&wallet.pub_key, new_wallet.clone());
        new_wallet
    }

    pub fn add_pending_tx(
        &mut self,
        wallet: &Wallet,
        tx_hash: &Hash,
        recipient: &PublicKey,
        amount: u64,
        transaction: &Hash,
    ) -> Wallet {
        let new_wallet = {
            let mut history = self.wallet_history_mut(&wallet.pub_key);
            history.push(*transaction);

            let history_hash = history.merkle_root();

            let pending_tx = PendingTransaction::new(tx_hash, recipient, amount);

            wallet.clone().add_pending_tx(pending_tx, &history_hash)
        };

        println!(
            "Creating pending transaction of {} coins transfer between wallets {} => {}",
            amount,
            wallet.pub_key.to_hex(),
            recipient.to_hex()
        );
        self.wallets_mut().put(&wallet.pub_key, new_wallet.clone());
        new_wallet
    }

    pub fn confirm_pending_tx(
        &mut self,
        wallet: &Wallet,
        tx: &PendingTransaction,
        confirmation_block: u64,
        transaction: &Hash,
    ) -> Wallet {
        let new_wallet = {
            let mut history = self.wallet_history_mut(&wallet.pub_key);
            history.push(*transaction);

            let history_hash = history.merkle_root();

            let approved_tx = ApprovedTransaction {
                tx_hash: tx.tx_hash,
                recipient: tx.recipient,
                amount: tx.amount,
                approvals: tx.approvals.clone(),
                confirmation_block,
            };

            wallet
                .clone()
                .remove_pending_tx(&tx.tx_hash, &history_hash)
                .add_approved_tx(approved_tx, &history_hash)
        };

        self.wallets_mut()
            .put(&new_wallet.pub_key, new_wallet.clone());
        new_wallet
    }

    pub fn sign_pending_tx(
        &mut self,
        wallet: &Wallet,
        tx_hash: &Hash,
        signer: &PublicKey,
        transaction: &Hash,
    ) -> Wallet {
        let new_wallet = {
            let mut history = self.wallet_history_mut(&wallet.pub_key);
            history.push(*transaction);

            let history_hash = history.merkle_root();

            wallet
                .clone()
                .sign_pending_tx(tx_hash, &signer, &history_hash)
        };

        self.wallets_mut()
            .put(&new_wallet.pub_key, new_wallet.clone());
        new_wallet
    }

    pub fn increase_wallet_balance(
        &mut self,
        wallet: &Wallet,
        amount: u64,
        transaction: &Hash,
    ) -> Wallet {
        let new_wallet = {
            let mut history = self.wallet_history_mut(&wallet.pub_key);
            history.push(*transaction);

            let history_hash = history.merkle_root();

            let balance = wallet.balance;
            wallet.clone().set_balance(balance + amount, &history_hash)
        };

        self.wallets_mut()
            .put(&new_wallet.pub_key, new_wallet.clone());
        new_wallet
    }

    pub fn decrease_wallet_balance(
        &mut self,
        wallet: &Wallet,
        amount: u64,
        transaction: &Hash,
    ) -> Wallet {
        let new_wallet = {
            let mut history = self.wallet_history_mut(&wallet.pub_key);
            history.push(*transaction);

            let history_hash = history.merkle_root();

            let balance = wallet.balance;
            wallet.clone().set_balance(balance - amount, &history_hash)
        };

        self.wallets_mut().put(&wallet.pub_key, new_wallet.clone());
        new_wallet
    }

    pub fn add_awaiting_tx(&mut self, service_hash: &Hash, origin_hash: &Hash, sender: &PublicKey) {
        self.awaiting_txs_mut().put(
            &service_hash,
            SignTx {
                origin: sender.clone(),
                tx_hash: origin_hash.clone(),
            },
        );
    }

    pub fn remove_awaiting_tx(&mut self, hash: &Hash) {
        self.awaiting_txs_mut().remove(&hash);
    }
}
