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
                let account = Account::new(self.seed, self.index);
                self.index += 1;
                account
            }
        }
    }

    /// Return a used account to the free pool after a transaction
    /// If there is remaining balance on a pool account after a transaction it should be refunded
    pub fn return_account(&mut self, account: Account) {
        if account.balance() > 0 {
            account.refund();
        }
        self.free.push_back(account)
    }
}
