use nano_pool::wallet::Wallet;
use nano_pool::address::Address;
use nano_pool::common;
use nano_pool::account::Account;
use nano_pool::rpc::{RpcClient, RpcCommand};
use nano_pool::ws::WsClient;

use serde_derive::Deserialize;
use serde_json::Value;
use std::fs;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

#[derive(Deserialize)]
struct Config {
    wallet_seed: Address,
    node_address: String,
    node_rpc_port: u16,
    node_ws_port: u16,
}

fn main() {
    let contents = fs::read_to_string("config/Config.toml")
        .expect("Something went wrong reading the file");

    let config: Config = toml::from_str(&contents).unwrap();    

    let url = format!("{}:{}", config.node_address, config.node_rpc_port);
    let (rpc_tx, rpc_rx) = mpsc::channel::<RpcCommand>();
    let rpc = RpcClient::new(url, rpc_rx);
    thread::spawn(move || {
        rpc.run();
    });

    let url = format!("{}:{}", config.node_address, config.node_ws_port);
    
    let ws = WsClient::new(url);
    thread::spawn(move || {
        ws.run();
    });

    let seed = common::hexstring_to_bytes(&config.wallet_seed);
    let mut w = Wallet::new(seed, rpc_tx.clone());

    let acc1 = Account::new(seed, 1, rpc_tx.clone());
    w.send_direct(1, acc1.address());
    acc1.receive_all();
    // update balance on confirmation with websocket
}
