use bitvec::prelude::*;
use blake2b_simd::{Hash, Params};
use byteorder::{BigEndian, WriteBytesExt};
use ed25519_dalek::{PublicKey, SecretKey};
use std::iter::FromIterator;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::address::Address;
use crate::block::Block;
use crate::common::{bytes_to_hexstring, encode_nano_base_32};
use crate::rpc::*;
use crate::seed::Seed;
use crate::unit::Raw;
use crate::ws::WsSubscription;

pub struct Account {
    seed: Seed,
    index: u32,
    private_key: Hash,
    public_key: PublicKey,
    address: Address,
    balance: Raw,
    frontier: String,
    frontier_confirmed: bool,
    confirmation_height: u64,
    rpc_tx: Sender<RpcCommand>,
}

impl Account {
    pub fn new(
        seed: Seed,
        index: u32,
        rpc_tx: Sender<RpcCommand>,
        ws_tx: Sender<WsSubscription>,
    ) -> Arc<Mutex<Self>> {
        // Derive private key from seed
        let private_key = Self::derive_private_key(seed, index);

        // Derive public key from private key
        let public_key = Self::derive_public_key(private_key);

        // Derive address from public key
        let address = Self::derive_address(public_key);

        // Fetch pending balance
        let (_, pending) = Self::fetch_balance(rpc_tx.clone(), &address);

        // Fetch account info
        let account_info = Self::fetch_info(rpc_tx.clone(), &address);
        let frontier = account_info.confirmed_frontier.unwrap();
        let frontier_confirmed = account_info.frontier == frontier;

        let account = Self {
            seed,
            index,
            private_key,
            public_key,
            address,
            balance: account_info
                .confirmed_balance
                .unwrap()
                .parse::<Raw>()
                .unwrap(),
            frontier,
            frontier_confirmed,
            confirmation_height: account_info
                .confirmed_height
                .unwrap()
                .parse::<u64>()
                .unwrap(),
            rpc_tx,
        };

        // Watch account with websocket client, waits until ws subscription/update is acked
        let account = Arc::new(Mutex::new(account));
        let (tx, rx) = mpsc::channel::<()>();
        let sub = WsSubscription::new(account.clone(), tx);
        ws_tx.send(sub).unwrap();
        rx.recv().unwrap();

        // If there is pending balance, receive it first
        if pending > 0 {
            account.lock().unwrap().receive_all()
        }

        account
    }

    /// Receive all pending blocks
    pub fn receive_all(&mut self) {
        loop {
            let pending_blocks = rpc_accounts_pending(
                self.rpc_tx.clone(),
                vec![self.address()],
                1,
                Some(0),
                Some(true),
                None,
                None,
                Some(true),
            )
            .unwrap();

            if pending_blocks.len() == 0 {
                // We can stop receiving if theres no more pending blocks
                return;
            }

            for (_, pending_blocks) in pending_blocks {
                for send_block in pending_blocks {
                    let block: Block;
                    match self.confirmation_height {
                        0 => {
                            block = rpc_block_create(
                                self.rpc_tx.clone(),
                                "0".to_owned(),
                                self.address.clone(),
                                self.address.clone(),
                                send_block.amount.unwrap(),
                                send_block.hash,
                                self.private_key(),
                            )
                            .unwrap();
                        }
                        _ => {
                            block = rpc_block_create(
                                self.rpc_tx.clone(),
                                self.frontier.clone(),
                                self.address.clone(),
                                self.address.clone(),
                                self.balance + send_block.amount.unwrap(),
                                send_block.hash,
                                self.private_key(),
                            )
                            .unwrap();
                        }
                    }
                    let hash = rpc_process(self.rpc_tx.clone(), SUBTYPE::RECEIVE, block).unwrap();
                    self.balance += send_block.amount.unwrap();
                    self.frontier_confirmed = false;
                    self.frontier = hash;
                }
            }
        }
    }

    /// Receive a specific amount once (0 = any amount)
    /// Times out if not received within TRANSACTION_TIMEOUT
    pub fn receive_specific(&self, amount: Raw) -> Result<(), String> {
        Ok(())
    }

    /// Send some amount of Raw to another nano address
    pub fn send(&mut self, amount: Raw, destination: Address) -> Result<(), String> {
        if self.balance < amount {
            Err(format!(
                "Account {} insufficient balance ({}) to send {}",
                self.address, self.balance, amount
            ))
        } else {
            let block = rpc_block_create(
                self.rpc_tx.clone(),
                self.frontier.clone(),
                self.address.clone(),
                self.address.clone(),
                self.balance - amount,
                destination,
                self.private_key(),
            )
            .unwrap();

            let hash = rpc_process(self.rpc_tx.clone(), SUBTYPE::SEND, block).unwrap();
            self.balance -= amount;
            self.frontier_confirmed = false;
            self.frontier = hash;
            Ok(())
        }
    }

    /// Refund any remaining balance
    pub fn refund(&self) {}

    /// Get the account seed as a string
    pub fn seed(&self) -> String {
        bytes_to_hexstring(&self.seed)
    }

    pub fn seed_as_bytes(&self) -> Seed {
        self.seed
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn address(&self) -> Address {
        self.address.clone()
    }

    pub fn private_key(&self) -> String {
        bytes_to_hexstring(self.private_key.as_bytes())
    }

    pub fn public_key(&self) -> String {
        bytes_to_hexstring(self.public_key.as_bytes())
    }

    pub fn balance(&self) -> Raw {
        self.balance
    }

    pub fn frontier_confirmed(&self) -> bool {
        self.frontier_confirmed
    }

    /// Derive private key from seed and index
    pub fn derive_private_key(seed: Seed, index: u32) -> Hash {
        let mut wtr = vec![];
        wtr.write_u32::<BigEndian>(index).unwrap();
        Params::new()
            .hash_length(32)
            .to_state()
            .update(&seed)
            .update(&wtr)
            .finalize()
    }

    /// Derive public key from private key
    pub fn derive_public_key(private_key: Hash) -> PublicKey {
        PublicKey::from(&SecretKey::from_bytes(private_key.as_bytes()).unwrap())
    }

    /// Derive address from public key
    pub fn derive_address(public_key: PublicKey) -> Address {
        // Code based on Feeless project implementation
        let mut address = String::with_capacity(65);
        address.push_str("nano_");

        const PKP_LEN: usize = 4 + 8 * 32;
        const PKP_CAPACITY: usize = 4 + 8 * 32 + 4;
        let mut bits: BitVec<Msb0, u8> = BitVec::with_capacity(PKP_CAPACITY);
        let pad: BitVec<Msb0, u8> = bitvec![Msb0, u8; 0; 4];
        bits.extend_from_bitslice(&pad);
        bits.extend_from_raw_slice(public_key.as_bytes());
        debug_assert_eq!(bits.capacity(), PKP_CAPACITY);
        debug_assert_eq!(bits.len(), PKP_LEN);
        let public_key_part = encode_nano_base_32(&bits);
        address.push_str(&public_key_part);

        let result = Params::new()
            .hash_length(5)
            .to_state()
            .update(public_key.as_bytes())
            .finalize();
        let bits: BitVec<Msb0, u8> = BitVec::from_iter(result.as_bytes().iter().rev());
        let checksum = encode_nano_base_32(&bits);
        address.push_str(&checksum);
        address
    }

    /// Fetch balance and pending balance for address
    fn fetch_balance(rpc_tx: Sender<RpcCommand>, address: &Address) -> (Raw, Raw) {
        rpc_account_balance(rpc_tx, address).unwrap()
    }

    /// Refresh account frontier, balance, confirmation_height.
    pub fn refresh_account_info(&mut self) {
        let account_info = Account::fetch_info(self.rpc_tx.clone(), &self.address);

        self.frontier = account_info.confirmed_frontier.unwrap();

        self.frontier_confirmed = account_info.frontier == self.frontier;

        self.balance = account_info
            .confirmed_balance
            .unwrap()
            .parse::<Raw>()
            .unwrap();

        self.confirmation_height = account_info
            .confirmed_height
            .unwrap()
            .parse::<u64>()
            .unwrap();
    }

    /// Fetch account info
    fn fetch_info(rpc_tx: Sender<RpcCommand>, address: &Address) -> JsonAccountInfoResponse {
        let response = rpc_account_info(rpc_tx, &address.clone(), Some(true));
        match response {
            Ok(r) => r,
            Err(_) => JsonAccountInfoResponse {
                frontier: "".to_owned(),
                confirmed_frontier: Some("".to_owned()),
                open_block: "".to_owned(),
                representative_block: "".to_owned(),
                balance: 0,
                confirmed_balance: Some("0".to_owned()),
                modified_timestamp: 0,
                block_count: 0,
                account_version: "".to_owned(),
                confirmation_height: None,
                confirmed_height: Some("0".to_owned()),
                confirmation_height_frontier: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::hexstring_to_bytes;

    #[test]
    fn account_keys() {
        struct KeySet<'a>(u32, Seed, &'a str, &'a str, &'a str);

        let mut test_cases: Vec<KeySet> = vec![];
        // zero seed index 0
        test_cases.push(KeySet(
            0,
            hexstring_to_bytes("0000000000000000000000000000000000000000000000000000000000000000"),
            "9F0E444C69F77A49BD0BE89DB92C38FE713E0963165CCA12FAF5712D7657120F",
            "C008B814A7D269A1FA3C6528B19201A24D797912DB9996FF02A1FF356E45552B",
            "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7",
        ));
        // zero seed index 1
        test_cases.push(KeySet(
            1,
            hexstring_to_bytes("0000000000000000000000000000000000000000000000000000000000000000"),
            "B73B723BF7BD042B66AD3332718BA98DE7312F95ED3D05A130C9204552A7AFFF",
            "E30D22B7935BCC25412FC07427391AB4C98A4AD68BAA733300D23D82C9D20AD3",
            "nano_3rrf6cus8pye6o1kzi5n6wwjof8bjb7ff4xcgesi3njxid6x64pms6onw1f9",
        ));
        // zero seed index 420
        test_cases.push(KeySet(
            420,
            hexstring_to_bytes("0000000000000000000000000000000000000000000000000000000000000000"),
            "6BFF533C4ABBCBC6FEB43546C9F475E7650BED2129729A647C5F8996C2C12176",
            "154A26B47F6FA9EBFBA26EE0B4C151A67D01B44BF0D29AD175B079ED7DF5AC12",
            "nano_17cc6tt9yuxbxhxt6uq1pm1o5bmx18t6qw8kmdaqde5sxoyzdd1kmw4ag595",
        ));
        // zero seed index max
        test_cases.push(KeySet(
            4294967295,
            hexstring_to_bytes("0000000000000000000000000000000000000000000000000000000000000000"),
            "7FD49E2BC5FB13ADD7CA976B0C83F982EA2D9C73C0586F8870CB833F7D18691D",
            "D25BEC353E71869B219694AC8562C63B1459316AEEC35D7E0755F34B636BBBBA",
            "nano_3nkuxitmwwe8meisf77eiojeegrnd6rpoup5doz1gohmbfjpqgxtscu5nxbc",
        ));
        // random seed index 0
        test_cases.push(KeySet(
            0,
            hexstring_to_bytes("3E2A10DAE7E0937D47CCFAC29F8CB11F1B0EEB6E082D64F48DCBCDACF62F7ED3"),
            "062BDAE2B28031AEF50751F8FBFAF80766DD5F06945B7D0BD6C4E7BC1B37423D",
            "49156DCCDE544C264486D93C4FE9132C4CEE1C110204C01146E891FE14F80747",
            "nano_1kaofq8fwo4e6s4afpbwbznj8d4exrg341i6r1anft6jzrchi3t9qxhqryqs",
        ));

        for case in test_cases {
            let private_key = Account::derive_private_key(case.1, case.0);
            let public_key = Account::derive_public_key(private_key);
            assert_eq!(bytes_to_hexstring(private_key.as_bytes()), case.2);
            assert_eq!(bytes_to_hexstring(public_key.as_bytes()), case.3);
            assert_eq!(Account::derive_address(public_key), case.4);
        }
    }
}
