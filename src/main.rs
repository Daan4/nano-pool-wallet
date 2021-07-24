use nano_pool::account::Account;
use nano_pool::common;
use nano_pool::config::get_config;
use nano_pool::rpc::{RpcClient, RpcCommand};
use nano_pool::wallet::Wallet;
use nano_pool::ws::{WsClient, WsSubscription};

use std::sync::mpsc;
use std::thread;

fn main() {
    let config = get_config();

    let url = format!("{}:{}", config.node_address, config.node_rpc_port);
    let (rpc_tx, rpc_rx) = mpsc::channel::<RpcCommand>();
    let rpc = RpcClient::new(url, rpc_rx);
    thread::Builder::new()
        .name("rpc".to_owned())
        .spawn(move || {
            rpc.run();
        })
        .unwrap();

    let url = format!("ws://{}:{}", config.node_address, config.node_ws_port);
    let (ws_tx, ws_rx) = mpsc::channel::<WsSubscription>();
    WsClient::start(url, ws_rx);

    let seed = common::hexstring_to_bytes(&config.wallet_seed);
    let mut w = Wallet::new(seed, rpc_tx.clone(), ws_tx.clone());

    let acc1 = Account::new(seed, 1, rpc_tx.clone(), ws_tx.clone());
    w.send_direct(1, acc1.lock().unwrap().address());
    acc1.lock().unwrap().send(1, w.account().lock().unwrap().address()).unwrap();

    loop {}
}
