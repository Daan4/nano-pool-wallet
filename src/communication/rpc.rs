use serde_json::{json, Value};
use curl::easy::Easy;
use std::io::Read;
use serde_derive::Deserialize;
use serde_aux::prelude::*;

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

#[derive(Deserialize)]
struct RpcAccountBalanceResponse {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    balance: Raw,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pending: Raw,
}

pub fn rpc_account_balance(address: &Address) -> Result<(Raw, Raw), String> {
    let message = json!({
        "action": "account_balance",
        "account": address
    });
    let response: RpcAccountBalanceResponse = serde_json::from_value(rpc(&message)?).unwrap();
    Ok((response.balance, response.pending))
   // match (response.balance, response.pending) {
    //     (Value::String(balance), Value::String(pending)) => {
    //         match (balance.parse::<Raw>(), pending.parse::<Raw>()) {
    //             (Ok(balance), Ok(pending)) => {
    //                 Ok((balance as Raw, pending as Raw))
    //             },            
    //             (_, _) => Err(format!("RPC error invalid datatypes in response {} to message {}", response, message))
    //         }
    //     },
    //     (_, _) => Err(format!("RPC error invalid fields in response {} to message {}", response, message))
    // }
}

// pub fn rpc_accounts_pending(addresses: &[Address], threshold: Raw, source: bool) -> Result<(), String> {
//     let message = json!({
//         "action": "accounts_pending",
//         "accounts": addresses,
//         "threshold": threshold.to_string(),
//         "source": source,
//         "include_only_confirmed": true,
//     });
//     let response = rpc(&message)?;
//     match response["blocks"] {
//         Value::Object(map) => {
//             Ok(())
//         },
//         _ => Err(format!("RPC error invalid fields in response {} to message {}", response, message))
//     }
// }

pub fn rpc_send() {

}

pub fn rpc_receive() {

}
