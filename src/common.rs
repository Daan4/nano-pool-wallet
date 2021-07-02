use bitvec::prelude::*;
use once_cell::sync::Lazy;

const HEX: [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"];

pub fn bytes_to_hexstring(bytes: &[u8]) -> String {     
    let mut buf = String::new();
    for x in bytes.iter() {
        buf += HEX[(*x >> 4) as usize];
        buf += HEX[0x0F & *x as usize];
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
