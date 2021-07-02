 use crate::seed::Seed;

const HEX: [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"];

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
