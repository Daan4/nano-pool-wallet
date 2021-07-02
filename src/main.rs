use nano_pool::wallet::Wallet;
use nano_pool::seed::generate_random_seed;
use nano_pool::common::bytes_to_hexstring;

fn main() {
    println!("{}", bytes_to_hexstring(&generate_random_seed()));
    let seed = [0; 32];
    let w = Wallet::new(seed);
    println!("wallet seed: {}", w.seed());
    println!("wallet account private key: {}", w.account().private_key());
    println!("wallet account public key: {}", w.account().public_key());
    println!("wallet account address: {}", w.account().address());
}
