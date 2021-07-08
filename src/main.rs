use nano_pool::wallet::Wallet;
use nano_pool::address::Address;
use nano_pool::communication::rpc::*;
use nano_pool::common;

use serde_derive::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Config {
    wallet_seed: Address,
    node_address: String,
    node_rpc_port: u16,
}

fn main() {
    let contents = fs::read_to_string("config/Config.toml")
        .expect("Something went wrong reading the file");

    let config: Config = toml::from_str(&contents).unwrap();    

    let w = Wallet::new(common::hexstring_to_bytes(&config.wallet_seed));
    
    println!("wallet seed: {}", w.seed());
    println!("wallet account private key: {}", w.account().private_key());
    println!("wallet account public key: {}", w.account().public_key());
    println!("wallet account address: {}", w.account().address());
    println!("wallet account balance: {}", w.account().balance());

    println!("{:?}", rpc_accounts_pending(vec![w.account().address()], 1, None, None, None, None, Some(true)).unwrap());
    println!("{:?}", rpc_accounts_pending(vec![w.account().address()], 1, Some(1), None, None, None, Some(true)).unwrap());
    println!("{:?}", rpc_accounts_pending(vec![w.account().address()], 1, Some(0), Some(true), None, None, Some(true)).unwrap());
}
