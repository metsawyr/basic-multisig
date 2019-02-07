use super::wallet::Wallet;
use exonum::crypto::PublicKey;
use exonum::storage::{Fork, MapIndex, Snapshot};

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

    pub fn wallets(&self) -> MapIndex<&Snapshot, PublicKey, Wallet> {
        MapIndex::new("wallets", self.view.as_ref())
    }

    pub fn wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }
}

impl<'a> Schema<&'a mut Fork> {
    pub fn wallets_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        MapIndex::new("wallets", &mut self.view)
    }
}
