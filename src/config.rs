use serde::Deserialize;
use std::{fs, env::VarError};
use toml;
use lazy_static::lazy_static;
use std::env;

use crate::address::Address;

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

static TEST_CONFIG_PATH: &str = "config/config_test.toml";
static PROD_CONFIG_PATH: &str = "config/config.toml";

#[derive(Deserialize)]
pub struct Config {
    pub wallet_seed: Address,
    pub node_address: String,
    pub node_rpc_port: u16,
    pub node_ws_port: u16,
    pub representative: Address,
    pub transaction_timeout: u32,
}

impl Config {
    fn new() -> Self {
        let path;
        match env::var("RUST_ENV") {
            Ok(v) => {
                match v.as_str() {
                    "TEST" => path = TEST_CONFIG_PATH,
                    _ => path = PROD_CONFIG_PATH
                }
            },
            Err(_) => path = PROD_CONFIG_PATH
        }
        let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
        toml::from_str(&contents).unwrap()
    }
}
