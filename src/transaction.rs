use super::proto;
use super::schema::Schema;
use exonum::blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext};
use exonum::crypto::{Hash, PublicKey};
use exonum_derive::ProtobufConvert;
use failure::Fail;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, ProtobufConvert, Deserialize, Serialize)]
#[exonum(pb = "proto::PendingTransaction")]
pub struct PendingTransaction {
    pub tx_hash: Hash,
    pub recipient: PublicKey,
    pub amount: u64,
    pub approvals: Vec<PublicKey>,
}

impl PendingTransaction {
    pub fn new(&tx_hash: &Hash, &recipient: &PublicKey, amount: u64) -> Self {
        Self {
            tx_hash,
            recipient,
            amount,
            approvals: vec![],
        }
    }
}

#[derive(Clone, Debug, ProtobufConvert, Deserialize, Serialize)]
#[exonum(pb = "proto::ApprovedTransaction")]
pub struct ApprovedTransaction {
    pub tx_hash: Hash,
    pub recipient: PublicKey,
    pub amount: u64,
    pub approvals: Vec<PublicKey>,
    pub confirmation_block: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, TransactionSet)]
pub enum WalletTransaction {
    CreateWallet(CreateWalletTx),
    AddSigner(AddSignerTx),
    Transfer(TransferTx),
    Sign(SignTx),
    Confirmation(ConfirmationTx),
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::CreateWalletTx")]
pub struct CreateWalletTx {
    pub name: String,
}

impl Transaction for CreateWalletTx {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();
        let mut schema = Schema::new(context.fork());

        if schema.wallet(pub_key).is_some() {
            Err(TxError::WalletAlreadyExists)?
        }

        schema.create_wallet(&pub_key, &self.name, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::AddSignerTx")]
pub struct AddSignerTx {
    pub signer: PublicKey,
}

impl Transaction for AddSignerTx {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();
        let mut schema = Schema::new(context.fork());

        let wallet = match schema.wallet(pub_key) {
            Some(val) => val,
            None => Err(TxError::WalletNotFound)?,
        };

        schema.add_signer(&wallet, &self.signer, &hash);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::TransferTx")]
pub struct TransferTx {
    pub recipient: PublicKey,
    pub amount: u64,
    pub seed: u64,
}

impl Transaction for TransferTx {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();
        let mut schema = Schema::new(context.fork());

        let sender_wallet = match schema.wallet(pub_key) {
            Some(val) => val,
            None => Err(TxError::SenderNotFound)?,
        };

        let amount = self.amount;

        // Check if balance is higher than desired transfer amount
        if sender_wallet.balance < amount {
            Err(TxError::InsufficientCurrencyAmount)?;
        }

        // Check if wallet has trusted signers assigned, and create pending transaction if truthy
        // Immediately executes transfer in the other case
        schema.add_pending_tx(&sender_wallet, &hash, &self.recipient, amount, &hash);

        if sender_wallet.signers.len() == 0 {
            schema.add_awaiting_tx(&hash, &hash, &pub_key);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::SignTx")]
pub struct SignTx {
    pub origin: PublicKey,
    pub tx_hash: Hash,
}

impl Transaction for SignTx {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();
        let mut schema = Schema::new(context.fork());

        // Wallet, holding pending transactions
        let origin_wallet = match schema.wallet(&self.origin) {
            Some(val) => val,
            None => Err(TxError::SenderNotFound)?,
        };

        let tx_hash = self.tx_hash;

        // Check if pending transaction present in origin wallet
        let transaction = match origin_wallet
            .pending_txs
            .iter()
            .find(|item| item.tx_hash == tx_hash)
        {
            Some(tx) => tx,
            None => Err(TxError::PendingTransactionNotFound)?,
        };

        // Check if public key exist in origin wallet's `signers` vector
        if !origin_wallet.signers.contains(&pub_key) {
            Err(TxError::UnauthorizedSigner)?;
        }

        // Check if this signer already signed
        if transaction.approvals.contains(&pub_key) {
            Err(TxError::AlreadySigned)?;
        }

        let signers_amount = origin_wallet.signers.len() as f64;
        let signs_amount = transaction.approvals.len() as u64;

        schema.sign_pending_tx(&origin_wallet, &tx_hash, &pub_key, &hash);
        // Check if 2/3 majority achieved, and immediately execute transfer if truthy
        // +2 means +1 for transaction initiator and +1 for current singature that isn't added yet
        if signs_amount + 2 >= (2f64 * signers_amount / 3f64).floor() as u64 {
            schema.add_awaiting_tx(&hash, &tx_hash, &self.origin);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::ConfirmationTx")]
pub struct ConfirmationTx {
    pub tx_hash: Hash,
    pub sender: PublicKey,
    pub confirmation_block: u64,
}

impl Transaction for ConfirmationTx {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let hash = context.tx_hash();
        let mut schema = Schema::new(context.fork());

        let wallet = match schema.wallet(&self.sender) {
            Some(val) => val,
            None => Err(TxError::WalletNotFound)?,
        };

        let transaction = match wallet
            .pending_txs
            .iter()
            .find(|item| item.tx_hash == self.tx_hash)
        {
            Some(tx) => tx,
            None => Err(TxError::PendingTransactionNotFound)?,
        };

        // Get recipient wallet of pending transaction
        let recipient_wallet = schema.wallet(&transaction.recipient).unwrap();

        schema.remove_awaiting_tx(&self.tx_hash);
        let new_wallet =
            schema.confirm_pending_tx(&wallet, &transaction, self.confirmation_block, &hash);
        schema.decrease_wallet_balance(&new_wallet, transaction.amount, &hash);
        schema.increase_wallet_balance(&recipient_wallet, transaction.amount, &hash);

        Ok(())
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

    #[fail(display = "Already signed")]
    AlreadySigned = 7,
}

impl From<TxError> for ExecutionError {
    fn from(value: TxError) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}
