use crate::seed::{Seed, seed_to_string};
use crate::account::Account;
use crate::pool::Pool;
use crate::unit::Raw;

pub struct Wallet {
    seed: Seed,
    account: Account,
    pool: Pool,
}

impl Wallet {
    pub fn new(seed: Seed) -> Wallet {
        Wallet {
            seed,
            account: Account::new(seed, 0),
            pool: Pool::new(),
        }
    }

    pub fn seed(&self) -> String {
        seed_to_string(self.seed)
    }

    pub fn account(&self) -> String {
        self.account.address()
    }

    pub fn await_payment(&self, amount: Raw) {

    }
}
