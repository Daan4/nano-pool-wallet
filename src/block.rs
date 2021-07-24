use blake2b_simd::Hash;
use ed25519_dalek::PublicKey;
use serde_derive::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

use crate::address::Address;
use crate::rpc::{rpc_work_generate, RpcCommand};
use crate::unit::Raw;

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub r#type: String,
    pub account: Address,
    pub previous: String,
    pub representative: Address,
    pub balance: String,
    pub link: String,
    pub link_as_account: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
}

impl Block {
    pub fn new(
        account: Address,
        previous: String,
        representative: Address,
        balance: Raw,
        link: String,
        link_as_account: String,
        signature: Option<String>,
        work: Option<String>,
        subtype: Option<String>,
    ) -> Self {
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
            subtype
        }
    }

    /// todo implement own signing
    // pub fn sign(&mut self, private_key: Hash, public_key: PublicKey) {
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
    // }

    pub fn work(&mut self, rpc_tx: Sender<RpcCommand>, hash: String) {
        self.work = Some(
            rpc_work_generate(
                rpc_tx,
                hash,
                Some(true),
                None,
                None,
                None,
                None,
                Some(self),
                Some(true),
            )
            .unwrap(),
        );
    }
}
