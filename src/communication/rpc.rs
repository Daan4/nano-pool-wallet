use serde_json::{Value, Map};
use curl::easy::Easy;
use std::io::Read;
use serde_derive::{Serialize, Deserialize};
use serde_aux::prelude::*;
use std::collections::HashMap;

use crate::unit::Raw;
use crate::address::Address;

fn rpc(json: &Value) -> Result<Value, String> {
    println!("RPC send {}", json);
    let data = json.to_string();
    let mut data = data.as_bytes();
    let mut easy = Easy::new();
    easy.url("127.0.0.1:17076").unwrap();
    easy.post(true).unwrap();
    easy.post_field_size(data.len() as u64).unwrap();
    let mut dst = Vec::new();
    let mut transfer = easy.transfer();
    transfer.read_function(|buf| {
        Ok(data.read(buf).unwrap_or(0))
    }).unwrap();
    transfer.write_function(|data| {
        dst.extend_from_slice(data);
        Ok(data.len())
    }).unwrap();
    transfer.perform().unwrap();
    drop(transfer);
    let dst = String::from_utf8(dst).unwrap();
    let dst: Value = serde_json::from_str(&dst).unwrap();
    println!("RPC recv {}", dst);
    Ok(dst)
}

#[derive(Serialize)]
struct JsonAccountBalanceMessage {
    action: String,
    account: Address
}

#[derive(Deserialize)]
struct JsonAccountBalanceResponse {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    balance: Raw,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pending: Raw,
}

pub fn rpc_account_balance(address: &Address) -> Result<(Raw, Raw), String> {
    let message = JsonAccountBalanceMessage {
        action: "account_balance".to_owned(),
        account: address.to_owned(),
    };
    let message = serde_json::to_value(message).unwrap();
    let response: JsonAccountBalanceResponse = serde_json::from_value(rpc(&message)?).unwrap();
    Ok((response.balance, response.pending))
}

#[derive(Serialize)]
struct JsonAccountsPendingMessage {
    action: String,
    accounts: Vec<Address>,
    count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    threshold: Option<Raw>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sorting: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_only_confirmed: Option<bool>
}

#[derive(Deserialize)]
struct JsonAccountsPendingResponse {
    blocks: Map<String, Value>
}

#[derive(Deserialize)]
struct JsonBlock {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    amount: Raw,
    source: Option<Address>
}

struct PendingBlock {
    hash: String,
    amount: Option<Raw>,
    source: Option<Address>
}

pub fn rpc_accounts_pending(addresses: Vec<Address>, count: usize, mut threshold: Option<Raw>, source: Option<bool>, 
                            include_active: Option<bool>, sorting: Option<bool>, include_only_confirmed: Option<bool>) -> Result<(), String> {
    // treat 0 threshold as None threshold
    match threshold {
        Some(value) if value == 0 => {
            threshold = None;
        },
        _ => {}
    }
    let message = JsonAccountsPendingMessage {
        action: "accounts_pending".to_owned(),
        accounts: addresses,
        count,
        threshold,
        source,
        include_active,
        sorting,
        include_only_confirmed
    };
    let message = serde_json::to_value(message).unwrap();
    let response: JsonAccountsPendingResponse = serde_json::from_value(rpc(&message)?).unwrap();
    for account in response.blocks.keys() {
        let value = 
        match source {
            Some(b) if b => {
                // if source is included then we get the amount and source for each block hash
                let blocks: HashMap<String, JsonBlock> = serde_json::from_value(response.blocks[account].clone()).unwrap();
            },
            _ => {
                match threshold {
                    Some(_) => {
                        // if threshold is included then then we get the amount for each block hash
                        let blocks: HashMap<String, String> = serde_json::from_value(response.blocks[account].clone()).unwrap();
                    },
                    _ => {
                        // if neither threshold nor source is included we just get an array of blocks
                        let blocks: Vec<String> = serde_json::from_value(response.blocks[account].clone()).unwrap();
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn rpc_send() {

}

pub fn rpc_receive() {

}
