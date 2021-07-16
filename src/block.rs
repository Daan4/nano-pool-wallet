use serde_derive::{Serialize, Deserialize};
use serde_aux::prelude::*;
use ed25519_dalek::{PublicKey, SecretKey, ExpandedSecretKey};
use ed25519_dalek::ed25519::signature::Signature as InternalSignature;
use blake2b_simd::Hash;
use std::sync::mpsc::Sender;

use crate::address::Address;
use crate::unit::Raw;
use crate::rpc::{rpc_work_generate, RpcCommand};
use crate::common::{bytes_to_hexstring, hexstring_to_bytes};

#[derive(Serialize, Deserialize)]
pub struct Block {
    r#type: String,
    account: Address,
    previous: String,
    representative: Address,
    balance: String,
    link: String,
    link_as_account: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    work: Option<String>,
}

impl Block {
    pub fn new(account: Address, previous: String, representative: Address, balance: Raw, link: String, link_as_account: String, signature: Option<String>, work: Option<String>) -> Self {
        Self {
            r#type: "state".to_owned(),
            account,
            previous,
            representative,
            balance: balance.to_string(),
            link,
            link_as_account,
            signature,
            work,
        }
    }

    /// todo implement own signing
    pub fn sign(&mut self, private_key: Hash, public_key: PublicKey) {
        // let preamble: [u8; 32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6]; 
        // let account = hexstring_to_bytes(&self.account);
        // let previous = hexstring_to_bytes(&self.previous);
        // let representative = hexstring_to_bytes(&self.representative);
        // let balance = self.balance.as_bytes();
        // let link = hexstring_to_bytes(&self.link);
        // let message: Vec<u8> = [&preamble, &account, &previous, &representative, balance, &link].concat();
        // println!("{:?} {}", account, bytes_to_hexstring(&account));

        // let dalek: SecretKey = SecretKey::from_bytes(private_key.as_bytes()).unwrap();
        // let expanded_secret = ExpandedSecretKey::from(&dalek);
        // let internal_signed = expanded_secret.sign(&message, &public_key);
        // println!("signature {}", bytes_to_hexstring(internal_signed.as_bytes()));
        // self.signature = Some(bytes_to_hexstring(internal_signed.as_bytes()));
    }

    pub fn work(&mut self, rpc_tx: Sender<RpcCommand>, hash: String) {
        self.work = Some(rpc_work_generate(rpc_tx, hash, Some(true), None, None, None, None, Some(self), Some(true)).unwrap());
    }
}
