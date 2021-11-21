use serde::Deserialize;
use std::fs;
use toml;

use crate::address::Address;

#[derive(Deserialize)]
pub struct Config {
    pub wallet_seed: Address,
    pub node_address: String,
    pub node_rpc_port: u16,
    pub node_ws_port: u16,
    pub representative: Address,
    pub transaction_timeout: u32,
}

pub fn get_config() -> Config {
    let contents =
        fs::read_to_string("config/Config.toml").expect("Something went wrong reading the file");

    toml::from_str(&contents).unwrap()
}
