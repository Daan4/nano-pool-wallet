use log::info;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::account::Account;
use crate::address::Address;
use crate::common::bytes_to_hexstring;
use crate::config::get_config;
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
}

impl Wallet {
    pub fn new(seed: Seed, rpc_tx: Sender<RpcCommand>, ws_tx: Sender<WsSubscription>) -> Wallet {
        let account = Account::new(seed, 0, rpc_tx.clone(), ws_tx.clone());
        Wallet {
            seed,
            account: account.clone(),
            pool: Pool::new(
                seed,
                rpc_tx.clone(),
                ws_tx.clone(),
                account.clone().lock().unwrap().address(),
            ),
            rpc_tx,
        }
    }

    /// Get wallet account seed as string
    pub fn seed(&self) -> String {
        bytes_to_hexstring(&self.seed)
    }

    /// Get a reference to the wallet account
    pub fn account(&self) -> Arc<Mutex<Account>> {
        self.account.clone()
    }

    /// Get a reference to the wallet account pool
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Send an amount of nano from the wallet to a destination through the pool
    /// > send_payment nano_3qy8738374rbpc37sna1mb5hu8in7rbfapagba6gthsdnyrarf7457er5f39 1000000000000000000000000000
    pub fn send_payment(&mut self, amount: Raw, destination: Address) -> Result<(), String> {
        info!("WALLET sending {} to {} via pool", amount, destination);
        let mut account = self.account.lock().unwrap();
        if amount == 0 {
            Err("Cannot send 0 raw".to_string())
        } else if account.balance() < amount {
            Err(format!(
                "Cannot send {} raw because the main account only holds {}",
                amount,
                account.balance()
            ))
        } else {
            let pool_account_arc = self.pool.get_account();
            let pool_account = pool_account_arc.lock().unwrap();
            account.send(amount, pool_account.address())?;
            drop(account);

            let mut balance = 0;
            let address = &pool_account.address();
            drop(pool_account);
            while balance < amount {
                // todo non polling solution?
                thread::sleep(Duration::from_millis(1000));
                let (b, _) = Account::fetch_balance(self.rpc_tx.clone(), address);
                balance = b;
            }
            let mut pool_account = pool_account_arc.lock().unwrap();
            pool_account.send(amount, destination)?;
            self.pool.return_account(pool_account_arc.clone());
            Ok(())
        }
    }

    /// Receive some amount of nano through the pool (0 = any amount)
    /// > receive_payment 1000000000000000000000000000
    pub fn receive_payment(&mut self, amount: Raw) -> Result<(), String> {
        let pool_account_arc = self.pool.get_account();
        let pool_account = pool_account_arc.lock().unwrap();

        let mut balance = 0;
        let address = &pool_account.address();
        let transaction_timeout = get_config("config/config.toml").transaction_timeout * 1000;
        let mut total_duration: u32 = 0;
        drop(pool_account);
        info!("WALLET receiving {} on {}", amount, address);
        while balance != amount {
            // todo non polling solution?
            thread::sleep(Duration::from_millis(1000));
            let (b, _) = Account::fetch_balance(self.rpc_tx.clone(), address);
            balance = b;
            total_duration += 1000;
            if total_duration >= transaction_timeout {
                info!("WALLET timed out receiving {} on {}", amount, address);
                self.pool.return_account(pool_account_arc.clone());
                return Err("Timed out awaiting payment".to_owned());
            }
        }
        let mut pool_account = pool_account_arc.lock().unwrap();
        pool_account.send(amount, self.account.lock().unwrap().address())?;
        self.pool.return_account(pool_account_arc.clone());
        Ok(())
    }

    /// Send a transaction directly from the main account
    /// > send_direct nano_3qy8738374rbpc37sna1mb5hu8in7rbfapagba6gthsdnyrarf7457er5f39 1000000000000000000000000000
    pub fn send_direct(&mut self, amount: Raw, destination: Address) {
        info!("WALLET send {} to {} directly", amount, destination);
        let mut account = self.account.lock().unwrap();
        account.send(amount, destination).unwrap();
    }

    /// Receive all transactions coming directly to the main account
    pub fn receive_all_direct(&self) {
        let mut account = self.account.lock().unwrap();
        account.receive_all();
    }
}
