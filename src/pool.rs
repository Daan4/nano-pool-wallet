use std::collections::VecDeque;
use std::sync::mpsc::{Sender, Receiver};
use serde_json::Value;

use crate::account::Account;
use crate::seed::Seed;
use crate::address::Address;
use crate::rpc::RpcCommand;

pub struct Pool {
    free: VecDeque<Account>,
    index: u32,
    seed: Seed,
    rpc_tx: Sender<RpcCommand>,
}

impl Pool {
    pub fn new(seed: Seed, rpc_tx: Sender<RpcCommand>) -> Pool {
        Pool {
            free: VecDeque::with_capacity(2^32-1),
            index: 0,
            seed,
            rpc_tx
        }
    }

    /// Get a free account to use for a transaction
    pub fn get_account(&mut self) -> Account {
        match self.free.pop_front() {
            Some(account) => account,
            None => {
                let account = Account::new(self.seed, self.index, self.rpc_tx.clone());
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

    /// Get the account adress at a given index
    pub fn get_account_address(&self, index: u32) -> Address {
        let private_key = Account::derive_private_key(self.seed, index);
        let public_key = Account::derive_public_key(private_key);
        Account::derive_address(public_key)
    }
}
