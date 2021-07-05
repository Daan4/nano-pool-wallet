use std::collections::VecDeque;

use crate::account::Account;
use crate::seed::Seed;
use crate::unit::Raw;
use crate::address::Address;

pub struct Pool {
    free: VecDeque<Account>,
    index: u32,
    seed: Seed
}

impl Pool {
    pub fn new(seed: Seed) -> Pool {
        Pool {
            free: VecDeque::with_capacity(2^32-1),
            index: 0,
            seed
        }
    }

    /// Get a free account to use for a transaction
    pub fn get_account(&mut self) -> Account {
        match self.free.pop_front() {
            Some(account) => account,
            None => {
                let acc = Account::new(self.seed, self.index);
                self.index += 1;
                acc
            }
        }
    }

    /// Return a used account to the free pool after a transaction
    pub fn return_account(&mut self, account: Account) {
        self.free.push_back(account)
    }
}
