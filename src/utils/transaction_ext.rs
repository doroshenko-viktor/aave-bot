use alloy::{
    hex,
    primitives::{FixedBytes, TxHash},
};
use eyre::{Error, Result};

pub trait TransactionExt {
    fn to_compact_str(&self) -> String;
    fn from_compact_str(compact_str: &str) -> Result<TxHash>;
}

impl TransactionExt for FixedBytes<32> {
    fn to_compact_str(&self) -> String {
        hex::encode(self)
    }

    fn from_compact_str(compact_str: &str) -> Result<TxHash> {
        todo!()
        // TxHash::from_slice(&hex::decode(compact_str))
    }
}
