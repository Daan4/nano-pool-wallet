use rand::Rng;

pub type Seed = [u8; 32];

pub fn generate_random_seed() -> Seed {
    let mut rng = rand::thread_rng();
    let mut seed: Seed = [0; 32];
    for i in 0..32 {
        seed[i] = rng.gen_range(0..16) << 4 | rng.gen_range(0..16);
    }
    seed
}
