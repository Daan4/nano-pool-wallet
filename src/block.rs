use serde_derive::Serialize;
use serde_aux::prelude::*;

use crate::address::Address;
use crate::unit::Raw;

#[derive(Serialize)]
pub struct Block {
    r#type: String,
    account: Address,
    previous: String,
    representative: Address,
    balance: Raw,
    link: String,
    link_as_account: Address,
    signature: String,
    work: String
}

impl Block {
    fn new(account: Address, previous: String, representative: Address, balance: Raw, link: String, link_as_account: Address, signature: String, work: String) -> Block {
        Block {
            r#type: "state".to_owned(),
            account,
            previous,
            representative,
            balance,
            link,
            link_as_account,
            signature,
            work
        }
    }
}
