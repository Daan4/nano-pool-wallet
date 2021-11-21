use std::sync::mpsc;
use std::sync::mpsc::Sender;

use nano_pool::cli::CliClient;
use nano_pool::common;
use nano_pool::config::get_config;
use nano_pool::config::Config;
use nano_pool::logger;
use nano_pool::rpc::{RpcClient, RpcCommand};
use nano_pool::wallet::Wallet;
use nano_pool::ws::{WsClient, WsSubscription};

fn main() {
    let cfg = get_config();

    logger::init().unwrap();

    let rpc_tx = start_rpc_client(&cfg);

    let ws_tx = start_ws_client(&cfg);

    let seed = common::hexstring_to_bytes(&cfg.wallet_seed);
    let wallet = Wallet::new(seed, rpc_tx.clone(), ws_tx.clone());

    start_cli_client(wallet);

    // Halt; rpc, ws, cli threads still run
    loop {}
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

fn start_cli_client(wallet: Wallet) {
    CliClient::start(wallet);
}
