use nano_pay::wallet::Wallet;
use nano_pay::seed::generate_random_seed;

fn main() {
    //let seed = generate_random_seed();
    let seed = [0; 32];
    let w = Wallet::new(seed);
    println!("wallet seed: {}", w.seed());
    println!("wallet account private key: {}", w.account().private_key());
    println!("wallet account public key: {}", w.account().public_key());
    println!("wallet account address: {}", w.account().address());
}
