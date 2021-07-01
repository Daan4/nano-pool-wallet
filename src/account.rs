use bitvec::macros::internal::funty::IsNumber;
use blake2b_simd::{Params, Hash};
use byteorder::{BigEndian, WriteBytesExt};
use ed25519_dalek::{PublicKey, SecretKey};
use bitvec::prelude::*;
use std::iter::FromIterator;

use crate::seed::Seed;
use crate::common::{bytes_to_hexstring, encode_nano_base_32};
pub struct Account {
    seed: Seed,
    private_key: Hash,
    public_key: PublicKey,
    address: String
}

impl Account {
    pub fn new(seed: Seed, index: u32) -> Account {
        // Derive private key from seed
        let mut wtr = vec![];
        wtr.write_u32::<BigEndian>(index).unwrap();
        let private_key = Params::new()
            .hash_length(32)
            .to_state()
            .update(&seed)
            .update(&wtr)
            .finalize();

        // Derive public key from private key
        let public_key = PublicKey::from(&SecretKey::from_bytes(private_key.as_bytes()).unwrap());

        // Derive address from public key
        // Code based on Feeless project implementation
        let mut address = String::with_capacity(65);
        address.push_str("nano_");

        const PKP_LEN: usize = 4 + 8 * 32;
        const PKP_CAPACITY: usize = 4 + 8 * 32 + 4; 
        let mut bits: BitVec<Msb0, u8> = BitVec::with_capacity(PKP_CAPACITY);
        let pad: BitVec<Msb0, u8> = bitvec![Msb0, u8; 0; 4];
        bits.extend_from_bitslice(&pad);
        bits.extend_from_raw_slice(public_key.as_bytes());
        debug_assert_eq!(bits.capacity(), PKP_CAPACITY);
        debug_assert_eq!(bits.len(), PKP_LEN);
        let public_key_part = encode_nano_base_32(&bits);
        address.push_str(&public_key_part);

        let result = Params::new()
            .hash_length(5)
            .to_state()
            .update(public_key.as_bytes())
            .finalize();
        let bits: BitVec<Msb0, u8> = BitVec::from_iter(result.as_bytes().iter().rev());
        let checksum = encode_nano_base_32(&bits);
        address.push_str(&checksum);

        Account {
            seed,
            private_key,
            public_key,
            address
        }
    }

    pub fn seed(&self) -> Seed {
        self.seed
    }

    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn private_key(&self) -> String {
        self.private_key.to_hex().to_string()
    }

    pub fn public_key(&self) -> String {
        bytes_to_hexstring(self.public_key.as_bytes())
    }
}
