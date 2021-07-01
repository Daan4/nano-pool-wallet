use crate::seed::Seed;
use crate::account::Account;
use crate::pool::Pool;
use crate::unit::Raw;
use crate::common::bytes_to_hexstring;

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
        bytes_to_hexstring(&self.seed)
    }

    pub fn account(&self) -> &Account {
        &self.account
    }

    pub fn await_payment(&self, amount: Raw) {

    }
}
