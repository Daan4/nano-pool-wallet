use crate::seed::Seed;
use crate::account::Account;
use crate::pool::Pool;
use crate::unit::Raw;
use crate::address::Address;
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
            pool: Pool::new(seed),
        }
    }

    pub fn seed(&self) -> String {
        bytes_to_hexstring(&self.seed)
    }

    pub fn account(&self) -> &Account {
        &self.account
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Send an amount of nano from the wallet to a destination through the pool
    pub fn send_payment(&mut self, amount: Raw, destination: Address) -> Result<(), String> {
        if amount == 0 {
            Err("Cannot send 0 raw".to_string())
        } else if amount < self.account.balance() {
            Err(format!("Cannot send {} raw because the main account only holds {}", amount, self.account.balance()))
        } else {
            let mut pool_account = self.pool.get_account();
            println!("Attempting send from {}", pool_account.address());
            self.account.send(amount, pool_account.address())?;
            pool_account.receive_specific(amount)?;
            pool_account.send(amount, destination)?;
            self.pool.return_account(pool_account);
            Ok(())
        }
    }
 
    /// Receive some amount of nano through the pool (0 = any amount)
    pub fn receive_payment(&mut self, amount: Raw) -> Result<(), String> {
        let mut pool_account = self.pool.get_account();
        println!("Attempting to receive {} raw on {}", amount, pool_account.address());
        pool_account.receive_specific(amount)?;
        pool_account.send(amount, self.account.address())?;
        self.pool.return_account(pool_account);
        Ok(())
    }

    /// Send a transaction directly from the main account
    pub fn send_direct(&mut self, amount: Raw, destination: Address) {
        self.account.send(amount, destination).unwrap();
    }

    /// Receive all transactions coming directly to the main account
    pub fn receive_all_direct(&self) {
        self.account.receive_all();
    }
}
