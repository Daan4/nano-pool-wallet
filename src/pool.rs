use std::collections::VecDeque;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::account::Account;
use crate::address::Address;
use crate::rpc::RpcCommand;
use crate::seed::Seed;
use crate::ws::WsSubscription;

pub struct Pool {
    free: VecDeque<Arc<Mutex<Account>>>,
    index: u32,
    seed: Seed,
    rpc_tx: Sender<RpcCommand>,
    ws_tx: Sender<WsSubscription>,
    wallet_address: Address,
}

impl Pool {
    pub fn new(
        seed: Seed,
        rpc_tx: Sender<RpcCommand>,
        ws_tx: Sender<WsSubscription>,
        wallet_address: Address,
    ) -> Pool {
        Pool {
            free: VecDeque::with_capacity(2 ^ 32 - 1),
            index: 1,  // Index starts at once, so the wallet address can use index 0 if desired
            seed,
            rpc_tx,
            ws_tx,
            wallet_address,
        }
    }

    /// Get a free account to use for a transaction
    /// If there is any balance remaining on it sweep it to the main wallet account
    pub fn get_account(&mut self) -> Arc<Mutex<Account>> {
        match self.free.pop_front() {
            Some(account) => account,
            None => {
                let account = Account::new(
                    self.seed,
                    self.index,
                    self.rpc_tx.clone(),
                    self.ws_tx.clone(),
                );
                self.index += 1;
                let mut acc = account.lock().unwrap();
                let balance = acc.balance();
                if balance > 0 {
                    acc.send(balance, self.wallet_address.clone()).unwrap();
                }
                drop(acc);
                account
            }
        }
    }

    /// Return a used account to the free pool after a transaction
    /// If there is any balance remaining on it sweep it to the main wallet account
    pub fn return_account(&mut self, account: Arc<Mutex<Account>>) {
        let mut acc = account.lock().unwrap();
        let balance = acc.balance();
        if balance > 0 {
            acc.send(balance, self.wallet_address.clone()).unwrap();
        }
        drop(acc);
        self.free.push_back(account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::get_config;
    use crate::logger::start_logger;
    use crate::rpc::start_rpc;
    use crate::ws::start_ws;
    use crate::common::generate_random_seed_address;

    #[test]
    fn pool_no_node_required() {
        let cfg = get_config("config/config_test.toml");
        start_logger().unwrap();
        let rpc_tx = start_rpc(&cfg);
        let ws_tx = start_ws(&cfg);
        let (seed, address) = generate_random_seed_address();
        let mut pool = Pool::new(seed, rpc_tx, ws_tx, address);

        let a1 = pool.get_account();
        let a2 = pool.get_account();
        let a3 = pool.get_account();

        assert_eq!(a1.lock().unwrap().index(), 1);
        assert_eq!(a2.lock().unwrap().index(), 2);
        assert_eq!(a3.lock().unwrap().index(), 3);

        pool.return_account(a2);
        pool.return_account(a1);
        assert_eq!(pool.free.len(), 2);

        let a2 = pool.get_account();
        let a1 = pool.get_account();
        let a4 = pool.get_account();
        assert_eq!(pool.free.len(), 0);

        assert_eq!(a1.lock().unwrap().index(), 1);
        assert_eq!(a2.lock().unwrap().index(), 2);
        assert_eq!(a4.lock().unwrap().index(), 4);

        pool.return_account(a1);
        pool.return_account(a2);
        pool.return_account(a3);
        pool.return_account(a4);
        assert_eq!(pool.free.len(), 4);
    }
}
