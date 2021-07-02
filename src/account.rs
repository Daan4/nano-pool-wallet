use blake2b_simd::{Params, Hash};
use byteorder::{BigEndian, WriteBytesExt};
use ed25519_dalek::{PublicKey, SecretKey};
use bitvec::prelude::*;
use std::iter::FromIterator;
use once_cell::sync::Lazy;

use crate::seed::Seed;
use crate::common::bytes_to_hexstring;

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

    pub fn seed(&self) -> String {
        bytes_to_hexstring(&self.seed)
    }

    pub fn seed_as_bytes(&self) -> Seed {
        self.seed
    }

    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn private_key(&self) -> String {
        bytes_to_hexstring(self.private_key.as_bytes())
    }

    pub fn public_key(&self) -> String {
        bytes_to_hexstring(self.public_key.as_bytes())
    }
}

// Function based on Feeless project implementation
const ALPHABET: &str = "13456789abcdefghijkmnopqrstuwxyz";
static ALPHABET_VEC: Lazy<Vec<char>> = Lazy::new(|| ALPHABET.chars().collect());
const ENCODING_BITS: usize = 5;

pub fn encode_nano_base_32(bits: &BitSlice<Msb0, u8>) -> String {
    debug_assert_eq!(
        bits.len() % ENCODING_BITS,
        0,
        "BitSlice must be divisible by 5"
    );
    let mut s = String::new(); // TODO: with_capacity
    for idx in (0..bits.len()).step_by(ENCODING_BITS) {
        let chunk: &BitSlice<Msb0, u8> = &bits[idx..idx + ENCODING_BITS];
        let value: u8 = chunk.load_be();
        let char = ALPHABET_VEC[value as usize];
        s.push(char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::hexstring_to_bytes;

    #[test]
    fn account_keys() {
        struct KeySet<'a>(u32, Seed, &'a str, &'a str, &'a str);

        let mut test_cases: Vec<KeySet> = vec![];
        // zero seed index 0
        test_cases.push(KeySet(
            0,
            hexstring_to_bytes("0000000000000000000000000000000000000000000000000000000000000000"),
            "9F0E444C69F77A49BD0BE89DB92C38FE713E0963165CCA12FAF5712D7657120F",
            "C008B814A7D269A1FA3C6528B19201A24D797912DB9996FF02A1FF356E45552B",
            "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"));
        // zero seed index 1
        test_cases.push(KeySet(
            1,
            hexstring_to_bytes("0000000000000000000000000000000000000000000000000000000000000000"),
            "B73B723BF7BD042B66AD3332718BA98DE7312F95ED3D05A130C9204552A7AFFF",
            "E30D22B7935BCC25412FC07427391AB4C98A4AD68BAA733300D23D82C9D20AD3",
            "nano_3rrf6cus8pye6o1kzi5n6wwjof8bjb7ff4xcgesi3njxid6x64pms6onw1f9"));
        // zero seed index 420
        test_cases.push(KeySet(
            420,
            hexstring_to_bytes("0000000000000000000000000000000000000000000000000000000000000000"),
            "6BFF533C4ABBCBC6FEB43546C9F475E7650BED2129729A647C5F8996C2C12176",
            "154A26B47F6FA9EBFBA26EE0B4C151A67D01B44BF0D29AD175B079ED7DF5AC12",
            "nano_17cc6tt9yuxbxhxt6uq1pm1o5bmx18t6qw8kmdaqde5sxoyzdd1kmw4ag595"));
        // zero seed index max
        test_cases.push(KeySet(
            4294967295,
            hexstring_to_bytes("0000000000000000000000000000000000000000000000000000000000000000"),
            "7FD49E2BC5FB13ADD7CA976B0C83F982EA2D9C73C0586F8870CB833F7D18691D",
            "D25BEC353E71869B219694AC8562C63B1459316AEEC35D7E0755F34B636BBBBA",
            "nano_3nkuxitmwwe8meisf77eiojeegrnd6rpoup5doz1gohmbfjpqgxtscu5nxbc"));
        // random seed index 0
        test_cases.push(KeySet(
            0,
            hexstring_to_bytes("3E2A10DAE7E0937D47CCFAC29F8CB11F1B0EEB6E082D64F48DCBCDACF62F7ED3"),
            "062BDAE2B28031AEF50751F8FBFAF80766DD5F06945B7D0BD6C4E7BC1B37423D",
            "49156DCCDE544C264486D93C4FE9132C4CEE1C110204C01146E891FE14F80747",
            "nano_1kaofq8fwo4e6s4afpbwbznj8d4exrg341i6r1anft6jzrchi3t9qxhqryqs"));

        for case in test_cases {
            let w = Account::new(case.1, case.0);
            assert_eq!(w.seed_as_bytes(), case.1);
            assert_eq!(w.private_key(), case.2);
            assert_eq!(w.public_key(), case.3);
            assert_eq!(w.address(), case.4);
        }
    }
}