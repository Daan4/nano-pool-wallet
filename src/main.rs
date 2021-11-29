use nano_pool::cli::start_cli;
use nano_pool::logger::start_logger;
use nano_pool::rpc::start_rpc;
use nano_pool::ws::start_ws;

fn main() {
    start_logger();
    let rpc_tx = start_rpc();
    let ws_tx = start_ws();
    start_cli(rpc_tx, ws_tx);
    loop {}
}
