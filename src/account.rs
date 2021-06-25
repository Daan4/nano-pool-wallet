use blake2b_simd::Params;
use byteorder::{BigEndian, WriteBytesExt};

use super::seed::Seed;

pub struct Account {
    address: String
}

impl Account {
    pub fn new(seed: Seed, index: u32) -> Account {
        let mut wtr = vec![];
        wtr.write_u32::<BigEndian>(index).unwrap();
        Account {
            address: "nano_".to_string() + &Params::new()
                .hash_length(32)
                .to_state()
                .update(&seed)
                .update(&wtr)
                .finalize()
                .to_hex()
                .to_string()
        }
    }

    pub fn address(&self) -> String {
        self.address.clone()
    }
}
