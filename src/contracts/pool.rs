use alloy::sol;

sol!(
    #[sol(rpc)]
    Pool,
    "abi/Pool.json"
);

pub mod liquidation_event {
    use alloy::primitives::{FixedBytes, keccak256};

    pub fn get_signature() -> FixedBytes<32> {
        keccak256(
            "LiquidationCall(address,address,address,uint256,uint256,address,bool)",
        )
    }
}

pub mod borrow_event {
    use alloy::primitives::{FixedBytes, keccak256};

    pub fn get_signature() -> FixedBytes<32> {
        keccak256(
            "Borrow(address, address, address, uint256, uint8, uint256, uint16)",
        )
    }
}

pub mod eth {
    use std::str::FromStr;

    use alloy::primitives::Address;
    pub const POOL_CONTRACT_ADDRESS: &str =
        "0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2";

    //TODO: this should be a part of config
    pub fn get_pool_contract_address() -> Address {
        Address::from_str(POOL_CONTRACT_ADDRESS).unwrap()
    }
}
