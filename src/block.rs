use serde_derive::{Deserialize, Serialize};

use crate::address::Address;
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
            subtype,
        }
    }
}
