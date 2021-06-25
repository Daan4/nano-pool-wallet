use nano_pay::wallet::Wallet;
use nano_pay::seed::generate_random_seed;

fn main() {
    let w = Wallet::new(generate_random_seed());
    println!("{:?}", w.seed());
    println!("{}", w.account())
}
