use nano_pool::wallet::Wallet;
use nano_pool::address::Address;
use nano_pool::rpc::*;
use nano_pool::common;
use nano_pool::account::Account;

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

    let seed = common::hexstring_to_bytes(&config.wallet_seed);
    let mut w = Wallet::new(seed);
    
    // Wallet main account info
    // println!("wallet seed: {}", w.seed());
    // println!("wallet account private key: {}", w.account().private_key());
    // println!("wallet account public key: {}", w.account().public_key());
    // println!("wallet account address: {}", w.account().address());
    // println!("wallet account balance: {}", w.account().balance());

    // Testing rpc_accounts_pending
    // println!("{:?}", rpc_accounts_pending(vec![w.account().address()], 1, None, None, None, None, Some(true)).unwrap());
    // println!("{:?}", rpc_accounts_pending(vec![w.account().address()], 1, Some(1), None, None, None, Some(true)).unwrap());
    // println!("{:?}", rpc_accounts_pending(vec![w.account().address()], 1, Some(0), Some(true), None, None, Some(true)).unwrap());

    // let pending_blocks = rpc_accounts_pending(vec![w.account().address()], 1, None, None, None, None, Some(true)).unwrap();
    // for (address, blocks) in pending_blocks {
    //     for block in blocks {
    //         println!("{}", rpc_work_generate(block.hash, Some(true), None, None, None, None, None, None).unwrap());
    //     }
    // }

    let acc1 = Account::new(seed, 1);
    w.send_direct(1, acc1.address());
    acc1.receive_all();
    // update balance on confirmation with websocket
}
