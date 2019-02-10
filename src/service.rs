use super::api::Api;
use super::schema::Schema as AppSchema;
use super::transaction::{ConfirmationTx, WalletTransaction};
use exonum::api::ServiceApiBuilder;
use exonum::blockchain::{
    self, Schema as BlockchainSchema, ServiceContext, Transaction, TransactionSet,
};
use exonum::crypto::Hash;
use exonum::messages::RawTransaction;
use exonum::storage::Snapshot;
use failure::Error;

pub struct Service;

impl blockchain::Service for Service {
    fn service_name(&self) -> &'static str {
        "cryptocurrency"
    }

    fn service_id(&self) -> u16 {
        1
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<dyn Transaction>, Error> {
        WalletTransaction::tx_from_raw(raw).map(Into::into)
    }

    fn state_hash(&self, _: &Snapshot) -> Vec<Hash> {
        vec![]
    }

    fn after_commit(&self, context: &ServiceContext) {
        let blockchain_schema = BlockchainSchema::new(context.snapshot());
        let app_schema = AppSchema::new(context.snapshot());
        let txs = blockchain_schema.block_transactions(context.height());
        let awaiting_txs = app_schema.awaiting_txs();

        txs.iter().for_each(|tx_hash| {
            if let Some(tx) = awaiting_txs.get(&tx_hash) {
                context.broadcast_transaction(ConfirmationTx {
                    tx_hash: tx.tx_hash,
                    sender: tx.origin,
                    confirmation_block: context.height().0,
                });
            }
        });
    }

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        Api::wire(builder);
    }
}
