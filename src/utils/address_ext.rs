use alloy::{hex, primitives::Address};

pub trait AddressExt {
    fn to_compact_str(self) -> String;
}

impl AddressExt for Address {
    fn to_compact_str(self) -> String {
        hex::encode(self.to_vec())
    }
}
