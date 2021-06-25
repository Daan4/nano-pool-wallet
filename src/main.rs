use nano_pay::wallet::Wallet;
use nano_pay::seed::generate_random_seed;

fn main() {
    //let seed = generate_random_seed();
    let seed = [0; 32];
    let w = Wallet::new(seed);
    println!("{}", w.seed());
    println!("{}", w.account())
}
