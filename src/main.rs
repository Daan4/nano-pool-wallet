use nano_pool::config::get_config;
use nano_pool::rpc::start_rpc;
use nano_pool::ws::start_ws;
use nano_pool::cli::start_cli;
use nano_pool::logger::start_logger;

fn main() {
    let cfg = get_config("config/Config.toml");
    start_logger().unwrap();
    let rpc_tx = start_rpc(&cfg);
    let ws_tx = start_ws(&cfg);
    start_cli(&cfg, rpc_tx, ws_tx);
    loop {}
}
