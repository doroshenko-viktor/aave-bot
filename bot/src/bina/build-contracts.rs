use alloy::{json_abi::JsonAbi, sol};
use std::{
    fs,
    path::{self, Path},
};

use eyre::Result;

sol!(Pool, "abi/Pool.json");

fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next().unwrap(); // skip program name

    let contract_name = "Pool".to_string();
    let contract_path = Path::new(&env!("CARGO_MANIFEST_DIR")).join("abi/Pool.json");
    let pool_abi = std::fs::read(&contract_path)?;
    let abi: JsonAbi = serde_json::from_slice(&pool_abi)?;

    println!("Generating bindings for {:?}", contract_path);

    // let bindings = Abigen::new(&contract_name, pool_abi.clone())?.generate()?;

    let output_path = Path::new(&env!("CARGO_MANIFEST_DIR")).join("src/contracts/pool.rs");
    // bindings.write_to_file(output_path)?;

    Ok(())
}
