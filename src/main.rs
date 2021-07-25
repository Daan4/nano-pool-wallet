use std::sync::mpsc;
use std::sync::mpsc::Sender;
use log::{SetLoggerError, LevelFilter};

use nano_pool::account::Account;
use nano_pool::common;
use nano_pool::config::get_config;
use nano_pool::config::Config;
use nano_pool::rpc::{RpcClient, RpcCommand};
use nano_pool::wallet::Wallet;
use nano_pool::ws::{WsClient, WsSubscription};

fn main() {
    let cfg = get_config();

    setup_logging();

    let rpc_tx = start_rpc_client(&cfg);

    let ws_tx = start_ws_client(&cfg);

    // Testing stuff
    let seed = common::hexstring_to_bytes(&cfg.wallet_seed);
    let mut w = Wallet::new(seed, rpc_tx.clone(), ws_tx.clone());

    let acc1 = Account::new(seed, 1, rpc_tx.clone(), ws_tx.clone());
    for i in 1..11 {
        w.send_direct(i, acc1.lock().unwrap().address());
    }

    // Halt; rpc and ws threads still run
    loop {}
}

fn setup_logging() {

}

fn start_rpc_client(cfg: &Config) -> Sender<RpcCommand> {
    let url = format!("{}:{}", cfg.node_address, cfg.node_rpc_port);
    let (rpc_tx, rpc_rx) = mpsc::channel::<RpcCommand>();
    RpcClient::start(url, rpc_rx);
    rpc_tx
}

fn start_ws_client(cfg: &Config) -> Sender<WsSubscription> {
    let url = format!("ws://{}:{}", cfg.node_address, cfg.node_ws_port);
    let (ws_tx, ws_rx) = mpsc::channel::<WsSubscription>();
    WsClient::start(url, ws_rx);
    ws_tx
}
