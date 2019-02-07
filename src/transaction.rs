use super::schema::Schema;
use super::wallet::{Wallet, Signer};
use exonum::blockchain::{ExecutionError, ExecutionResult, Transaction};
use exonum::crypto::PublicKey;
use exonum::encoding_struct;
use exonum::messages::Message;
use exonum::storage::Fork;
use failure::Fail;

encoding_struct! {
    struct PendingTx {
        sender: &PublicKey,
        recipient: &PublicKey,
        amount: u64,
        approvals: Vec<Approval>,
    }
}

encoding_struct! {
    struct Approval {
        pub_key: &PublicKey,
    }
}

transactions! {
    pub Transactions {
        const SERVICE_ID = 1;

        struct TxCreateWallet {
            pub_key: &PublicKey,
            name: &str,
        }

        struct TxAddSigner {
            pub_key: &PublicKey,
            signer: &PublicKey,
        }

        struct TxTransfer {
            sender: &PublicKey,
            recipient: &PublicKey,
            amount: u64,
            seed: u64,
        }

        struct TxSign {
            sender: &PublicKey,
            origin: &PublicKey,
            tx_index: u64,
        }
    }
}

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum TxError {
    #[fail(display = "Wallet already exists")]
    WalletAlreadyExists = 0,

    #[fail(display = "Wallet not found")]
    WalletNotFound = 1,

    #[fail(display = "Sender doesn't exist")]
    SenderNotFound = 2,

    #[fail(display = "Recipient doesn't exist")]
    RecipientNotFound = 3,

    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 4,

    #[fail(display = "Pending transaction not found")]
    PendingTransactionNotFound = 5,

    #[fail(display = "Unauthorized signer")]
    UnauthorizedSigner = 6,
}

impl From<TxError> for ExecutionError {
    fn from(value: TxError) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

impl Transaction for TxCreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(view);

        if schema.wallet(self.pub_key()).is_some() {
            Err(TxError::WalletAlreadyExists)?
        }

        let wallet = Wallet::new(self.pub_key(), self.name(), 100, vec![], vec![]);
        println!("Creating a wallet {:?}", wallet);
        schema.wallets_mut().put(self.pub_key(), wallet);

        Ok(())
    }
}

impl Transaction for TxAddSigner {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(view);

        let target = match schema.wallet(self.pub_key()) {
            Some(val) => val,
            None => Err(TxError::WalletNotFound)?,
        };

        let signer = self.signer();
        let target_wallet = target.add_signer(signer);

        println!(
            "Adding signer `{}` to the wallet {:?}",
            signer, target_wallet
        );
        schema.wallets_mut().put(self.pub_key(), target_wallet);

        Ok(())
    }
}

impl Transaction for TxTransfer {
    fn verify(&self) -> bool {
        (*self.sender() != *self.recipient()) && self.verify_signature(self.sender())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(view);

        let sender = match schema.wallet(self.sender()) {
            Some(val) => val,
            None => Err(TxError::SenderNotFound)?,
        };

        let recipient = match schema.wallet(self.recipient()) {
            Some(val) => val,
            None => Err(TxError::RecipientNotFound)?,
        };

        let amount = self.amount();

        // Check if balance is higher than desired transfer amount
        if sender.balance() < amount {
            Err(TxError::InsufficientCurrencyAmount)?;
        }

        let mut wallets = schema.wallets_mut();

        // Check if wallet has trusted signers assigned, and create pending transaction if truthy
        // Immediately executes transfer in the other case
        if sender.signers().len() > 0 {
            println!(
                "Creating pending transaction of {} coins transfer between wallets {:?} => {:?}",
                amount, sender, recipient
            );

            let pending_tx = PendingTx::new(self.sender(), self.recipient(), amount, vec![]);
            let sender_wallet = sender.add_pending_tx(pending_tx);

            wallets.put(self.sender(), sender_wallet);
        } else {
            println!(
                "Transfering of {} coins between wallets {:?} => {:?}",
                amount, sender, recipient
            );

            let sender_wallet = sender.decrease_balance(amount);
            let recipient_wallet = recipient.increase_balance(amount);

            wallets.put(self.sender(), sender_wallet);
            wallets.put(self.recipient(), recipient_wallet);
        }

        Ok(())
    }
}

impl Transaction for TxSign {
    fn verify(&self) -> bool {
        self.verify_signature(self.sender())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(view);

        // Wallet, holding pending transactions
        let mut origin_wallet = match schema.wallet(self.origin()) {
            Some(val) => val,
            None => Err(TxError::SenderNotFound)?,
        };

        let pending_txs = origin_wallet.pending_txs();
        let tx_index = self.tx_index() as usize;

        // Check if pending transaction present in origin wallet
        if pending_txs.is_empty() || pending_txs.get(tx_index).is_none() {
            Err(TxError::PendingTransactionNotFound)?;
        }

        let transaction = &pending_txs[tx_index];

        // Get recipient wallet of pending transaction
        let mut recipient_wallet = schema.wallet(transaction.recipient()).unwrap();

        let signers = origin_wallet.signers();

        // Check if public key exist in origin wallet's `signers` vector
        if !signers.contains(&Signer::new(self.sender())) {
            Err(TxError::UnauthorizedSigner)?;
        }

        // Add approval to transaction
        origin_wallet = origin_wallet.sign_pending_tx(tx_index, Approval::new(self.sender()));
        
        let signers_amount = signers.len() as f64;
        let signs_amount = transaction.approvals().len() as u64;

        let mut wallets = schema.wallets_mut();

        // Check if 2/3 majority achieved, and immediately execute transfer if truthy
        if signs_amount >= (2f64 * signers_amount / 3f64).floor() as u64 + 1 {
            let amount = transaction.amount();

            origin_wallet = origin_wallet.remove_pending_tx(tx_index);
            origin_wallet = origin_wallet.decrease_balance(amount);
            recipient_wallet = recipient_wallet.increase_balance(amount);

            wallets.put(transaction.recipient(), recipient_wallet);
        }

        wallets.put(self.origin(), origin_wallet);

        Ok(())
    }
}
