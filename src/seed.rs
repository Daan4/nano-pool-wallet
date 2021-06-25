use rand::Rng;

const HEX: [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"];

pub type Seed = [u8; 32];

pub fn generate_random_seed() -> Seed {
    let mut rng = rand::thread_rng();
    let mut seed: Seed = [0; 32];
    for i in 0..32 {
        seed[i] = rng.gen_range(0..16);
    }
    seed
}

pub fn seed_to_string(seed: Seed) -> String {
    let mut buf = String::new();
    for x in seed.iter() {
        buf += HEX[*x as usize];
    }
    buf
}
