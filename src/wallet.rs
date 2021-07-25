use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::account::Account;
use crate::address::Address;
use crate::common::bytes_to_hexstring;
use crate::pool::Pool;
use crate::rpc::RpcCommand;
use crate::seed::Seed;
use crate::unit::Raw;
use crate::ws::WsSubscription;

pub struct Wallet {
    seed: Seed,
    account: Arc<Mutex<Account>>,
    pool: Pool,
    rpc_tx: Sender<RpcCommand>,
    ws_tx: Sender<WsSubscription>,
}

impl Wallet {
    pub fn new(seed: Seed, rpc_tx: Sender<RpcCommand>, ws_tx: Sender<WsSubscription>) -> Wallet {
        Wallet {
            seed,
            account: Account::new(seed, 0, rpc_tx.clone(), ws_tx.clone()),
            pool: Pool::new(seed, rpc_tx.clone(), ws_tx.clone()),
            rpc_tx,
            ws_tx,
        }
    }

    pub fn seed(&self) -> String {
        bytes_to_hexstring(&self.seed)
    }

    pub fn account(&self) -> Arc<Mutex<Account>> {
        self.account.clone()
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Send an amount of nano from the wallet to a destination through the pool
    pub fn send_payment(&mut self, amount: Raw, destination: Address) -> Result<(), String> {
        let mut account = self.account.lock().unwrap();
        if amount == 0 {
            Err("Cannot send 0 raw".to_string())
        } else if amount < account.balance() {
            Err(format!(
                "Cannot send {} raw because the main account only holds {}",
                amount,
                account.balance()
            ))
        } else {
            let pool_account_arc = self.pool.get_account();
            let pool_account_arc_clone = pool_account_arc.clone();
            let mut pool_account = pool_account_arc.lock().unwrap();
            // println!("Attempting send from {}", pool_account.address());
            account.send(amount, pool_account.address())?;
            pool_account.receive_specific(amount)?;
            pool_account.send(amount, destination)?;
            self.pool.return_account(pool_account_arc_clone);
            Ok(())
        }
    }

    /// Receive some amount of nano through the pool (0 = any amount)
    pub fn receive_payment(&mut self, amount: Raw) -> Result<(), String> {
        let pool_account_arc = self.pool.get_account();
        let pool_account_arc_clone = pool_account_arc.clone();
        let mut pool_account = pool_account_arc.lock().unwrap();
        // println!(
        //     "Attempting to receive {} raw on {}",
        //     amount,
        //     pool_account.address()
        // );
        pool_account.receive_specific(amount)?;
        let account = self.account.lock().unwrap();
        pool_account.send(amount, account.address())?;
        self.pool.return_account(pool_account_arc_clone);
        Ok(())
    }

    /// Send a transaction directly from the main account
    pub fn send_direct(&mut self, amount: Raw, destination: Address) {
        let mut account = self.account.lock().unwrap();
        account.send(amount, destination).unwrap();
    }

    /// Receive all transactions coming directly to the main account
    pub fn receive_all_direct(&self) {
        let mut account = self.account.lock().unwrap();
        account.receive_all();
    }
}
