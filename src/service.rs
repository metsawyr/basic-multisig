use super::api::Api;
use super::transaction::WalletTransaction;
use exonum::api::ServiceApiBuilder;
use exonum::blockchain::{self, Transaction, TransactionSet};
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

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        Api::wire(builder);
    }
}
