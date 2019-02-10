#![allow(bare_trait_objects)]
#![allow(renamed_and_removed_lints)]

pub use self::schemes::{
    AddSignerTx, ApprovedTransaction, ConfirmationTx, CreateWalletTx, PendingTransaction, SignTx,
    TransferTx, Wallet,
};

include!(concat!(env!("OUT_DIR"), "/protobuf_mod.rs"));

pub use exonum::proto::schema::*;
