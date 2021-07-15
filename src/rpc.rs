use serde_json::{Value, Map};
use curl::easy::Easy;
use std::io::Read;
use serde_derive::{Serialize, Deserialize};
use serde_aux::prelude::*;
use std::collections::HashMap;

use crate::unit::Raw;
use crate::address::Address;
use crate::block::Block;

pub enum SUBTYPE {
    SEND,
    RECEIVE,
    CHANGE
}

fn rpc(json: &Value) -> Result<Value, String> {
    println!("RPC send {}\n", json);
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
    transfer.perform().expect("curl transfer failed");
    drop(transfer);
    let dst = String::from_utf8(dst).unwrap();
    let dst: Value = serde_json::from_str(&dst).unwrap();
    println!("RPC recv {}\n", dst);
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

#[derive(Debug)]
pub struct PendingBlock {
    pub hash: String,
    pub amount: Option<Raw>,
    pub source: Option<Address>
}

pub fn rpc_accounts_pending(addresses: Vec<Address>, count: usize, mut threshold: Option<Raw>, source: Option<bool>, 
                            include_active: Option<bool>, sorting: Option<bool>, include_only_confirmed: Option<bool>) -> Result<HashMap<Address, Vec<PendingBlock>>, String> {
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
    let mut output: HashMap<Address, Vec<PendingBlock>> = HashMap::new();
    for account in response.blocks.keys() {
        let mut blocks: Vec<PendingBlock> = vec![];
        match source {
            Some(b) if b => {
                // if source is included then we get the amount and source for each block hash
                let data: Result<HashMap<String, JsonBlock>, serde_json::Error> = serde_json::from_value(response.blocks[account].clone());
                match data {
                    Ok(d) => {
                        for (hash, block) in d {
                            blocks.push(PendingBlock{hash, amount: Some(block.amount), source: block.source});
                        }
                    },
                    Err(_) => {
                        // no pending blocks for this account
                        continue;
                    }
                }
            },
            _ => {
                match threshold {
                    Some(_) => {
                        // if threshold is included then then we get the amount for each block hash
                        let data: HashMap<String, String> = serde_json::from_value(response.blocks[account].clone()).unwrap();
                        for (hash, amount) in data {
                            let amount = amount.parse::<Raw>().unwrap();
                            blocks.push(PendingBlock{hash, amount: Some(amount), source: None});
                        }
                    },
                    _ => {
                        // if neither threshold nor source is included we just get an array of blocks
                        let data: Vec<String> = serde_json::from_value(response.blocks[account].clone()).unwrap();
                        for hash in data {
                            blocks.push(PendingBlock{hash, amount: None, source: None});
                        }
                    }
                }
            }
        }
        output.insert(account.to_owned(), blocks);
    }
    Ok(output)
}

#[derive(Serialize)]
struct JsonWorkGenerateMessage<'a>{
    action: String,
    hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_peers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    multiplier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    account: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    block: Option<&'a Block>,
    #[serde(skip_serializing_if = "Option::is_none")]
    json_block: Option<bool>
}

#[derive(Deserialize)]
struct JsonWorkGenerateResponse {
    work: String,
    difficulty: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    multiplier: f64,
    hash: String
}

pub fn rpc_work_generate(hash: String, use_peers: Option<bool>, difficulty: Option<String>, multiplier: Option<String>, 
                         account: Option<Address>, version: Option<String>, block: Option<&Block>, json_block: Option<bool>) -> Result<String, String> {
    let message = JsonWorkGenerateMessage {
        action: "work_generate".to_owned(),
        hash,
        use_peers,
        difficulty,
        multiplier,
        account,
        version,
        block,
        json_block
    };
    let message = serde_json::to_value(message).unwrap();
    let response: JsonWorkGenerateResponse = serde_json::from_value(rpc(&message)?).unwrap();
    Ok(response.work)
}

#[derive(Serialize)]
struct JsonAccountInfoMessage {
    action: String,
    account: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_confirmed: Option<bool>
}

#[derive(Deserialize)]
pub struct JsonAccountInfoResponse {
    pub frontier: String, 
    pub confirmed_frontier: Option<String>, 
    pub open_block: String, 
    pub representative_block: String, 
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub balance: Raw, 
    //#[serde(deserialize_with = "deserialize_option_number_from_string")] <-- deserialize_option_number_from_string is broken
    pub confirmed_balance: Option<String>, 
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub modified_timestamp: u64, 
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub block_count: u64, 
    pub account_version: String, 
    // #[serde(deserialize_with = "deserialize_option_number_from_string")] <-- deserialize_option_number_from_string is broken
    pub confirmation_height: Option<String>,
    // #[serde(deserialize_with = "deserialize_option_number_from_string")] <-- deserialize_option_number_from_string is broken
    pub confirmed_height: Option<String>, 
    pub confirmation_height_frontier: Option<String>
}

// only includes confirmed, maybe support unconfirmed account info at some point?
// support include confirmed and add those other fields as optional!!!
pub fn rpc_account_info(address: &Address, include_confirmed: Option<bool>) -> Result<JsonAccountInfoResponse, String> {
    let message = JsonAccountInfoMessage {
        action: "account_info".to_owned(),
        account: address.to_owned(),
        include_confirmed
    };
    let message = serde_json::to_value(message).unwrap();
    let response = serde_json::from_value(rpc(&message)?);
    match response {
        Ok(r) => {
            Ok(r)
        },
        Err(_) => Err("Account does not exist or malformed response".to_owned())
    }
}

#[derive(Serialize)]
struct JsonBlockCreateMessage {
    action: String,
    json_block: bool,
    r#type: String,
    previous: String,
    account: String,
    representative: String,
    balance: String,
    link: String,
    key: String,
}

#[derive(Deserialize)]
pub struct JsonBlockCreateResponse {
    hash: String,
    difficulty: String,
    block: Block,
}

pub fn rpc_block_create(previous: String, account: Address, representative: Address, balance: Raw, link: String, key: String) -> Result<Block, String> {
    let message = JsonBlockCreateMessage {
        action: "block_create".to_owned(),
        json_block: true,
        r#type: "state".to_owned(),
        previous,
        account,
        representative,
        balance: balance.to_string(),
        link,
        key
    };
    let message = serde_json::to_value(message).unwrap();
    let response: JsonBlockCreateResponse = serde_json::from_value(rpc(&message)?).unwrap();
    Ok(response.block)
}

#[derive(Serialize)]
struct JsonProcessMessage {
    action: String,
    json_block: bool,
    subtype: String,
    block: Block,
}

#[derive(Deserialize)]
pub struct JsonProcessResponse {
    hash: String,
}

pub fn rpc_process(subtype: SUBTYPE, block: Block) -> Result<String, String> {
    let subtypestr: String;
    match subtype {
        SUBTYPE::CHANGE => {
            subtypestr = "change".to_owned();
        },
        SUBTYPE::SEND => {
            subtypestr = "send".to_owned();
        },
        SUBTYPE::RECEIVE => {
            subtypestr = "receive".to_owned();
        }
    }
    let message = JsonProcessMessage {
        action: "process".to_owned(),
        json_block: true,
        subtype: subtypestr,
        block
    };
    let message = serde_json::to_value(message).unwrap();
    let response: JsonProcessResponse = serde_json::from_value(rpc(&message)?).unwrap();
    Ok(response.hash)
}
