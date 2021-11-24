use bitvec::prelude::*;
use once_cell::sync::Lazy;
use rand::Rng;

use crate::seed::Seed;
use crate::address::Address;
use crate::account::Account;

const HEX: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F",
];

/// Convert bytes array to hex string, where each byte contains 2 hex digits.
pub fn bytes_to_hexstring(bytes: &[u8]) -> String {
    let mut buf = String::new();

    for x in bytes.iter() {
        buf += HEX[(*x >> 4) as usize];
        buf += HEX[0x0F & *x as usize];
    }
    buf
}

/// Convert hex string to bytes array of size 32, where each byte contains 2 hex digits.
pub fn hexstring_to_bytes(hexstring: &str) -> Seed {
    let mut buf: Seed = [0; 32];
    let mut i: usize = 0;
    let mut j_a: u8 = 0;
    let mut j_b: u8 = 0;

    for (a, b) in hexstring.chars().zip(hexstring.chars().skip(1)).step_by(2) {
        for (j, x) in HEX.iter().enumerate() {
            if a.to_string() == x.to_string() {
                j_a = j as u8;
            }
            if b.to_string() == x.to_string() {
                j_b = j as u8;
            }
        }
        buf[i] = j_a << 4 | j_b;
        i += 1;
    }
    buf
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

/// Generate a random seed and address (index 0)
pub fn generate_random_seed_address() -> (Seed, Address) {
    let mut rng = rand::thread_rng();
    let mut seed: Seed = [0; 32];
    for i in 0..32 {
        seed[i] = rng.gen_range(0..16) << 4 | rng.gen_range(0..16);
    }
    let private_key = Account::derive_private_key(seed, 0);
    let public_key = Account::derive_public_key(private_key);
    let address = Account::derive_address(public_key);
    (seed, address)
}
