use log::info;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use serde_json::{json, Value};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use websocket::client::sync::Client;
use websocket::message::OwnedMessage;
use websocket::{ClientBuilder, Message};

use crate::account::Account;
use crate::address::Address;
use crate::block::Block;
use crate::config::CONFIG;
use crate::unit::Raw;

// start websocket interface
pub fn start_ws() -> Sender<WsSubscription> {
    let url = format!("ws://{}:{}", CONFIG.node_address, CONFIG.node_ws_port);
    let (ws_tx, ws_rx) = mpsc::channel::<WsSubscription>();
    WsClient::start(url, ws_rx);
    ws_tx
}

pub struct WsSubscription {
    account: Arc<Mutex<Account>>,
    tx_response: Sender<()>,
}

impl WsSubscription {
    pub fn new(account: Arc<Mutex<Account>>, tx_response: Sender<()>) -> Self {
        Self {
            account,
            tx_response,
        }
    }
}

pub struct WsClient {
    url: String,
    client: Client<TcpStream>,
    watched_accounts: HashMap<Address, Arc<Mutex<Account>>>,
    awaiting_ack: Option<(String, Sender<()>)>,
}

impl WsClient {
    pub fn start(url: String, rx: Receiver<WsSubscription>) {
        let mut builder = ClientBuilder::new(&url).unwrap();
        let client = builder.connect_insecure().unwrap();
        client.set_nonblocking(true).unwrap();
        let wsc = Self {
            url,
            client,
            watched_accounts: HashMap::new(),
            awaiting_ack: None,
        };

        let wsc = Arc::new(Mutex::new(wsc));
        let wsc2 = wsc.clone();

        // Monitor incoming account subscriptions
        thread::Builder::new()
            .name("ws sender".to_owned())
            .spawn(move || {
                WsClient::run_sender(wsc, rx);
            })
            .unwrap();

        // Run main ws client
        thread::Builder::new()
            .name("ws listener".to_owned())
            .spawn(move || {
                WsClient::run_listener(wsc2);
            })
            .unwrap();
    }

    fn subscribe(
        &mut self,
        topic: String,
        options: Option<Value>,
        tx_ack: Sender<()>,
    ) -> Result<(), String> {
        let json = JsonSubscribeMessage {
            action: "subscribe".to_owned(),
            ack: true,
            topic: topic.to_owned(),
            options,
        };
        self.awaiting_ack = Some(("subscribe".to_owned(), tx_ack));
        self.send(serde_json::to_value(json).unwrap()).unwrap();
        Ok(())
    }

    fn update(
        &mut self,
        topic: String,
        options: Option<Value>,
        tx_ack: Sender<()>,
    ) -> Result<(), String> {
        let json = JsonSubscribeMessage {
            action: "update".to_owned(),
            ack: true,
            topic: topic.to_owned(),
            options,
        };
        self.awaiting_ack = Some(("update".to_owned(), tx_ack));
        self.send(serde_json::to_value(json).unwrap()).unwrap();
        Ok(())
    }

    fn send(&mut self, json: Value) -> Result<(), String> {
        info!("WS send {}", json);
        let message = Message::text(json.to_string());
        self.client.send_message(&message).unwrap();
        Ok(())
    }

    /// Returns empty json if nothing to receive
    fn recv(&mut self) -> Result<Value, String> {
        let message = self.client.recv_message();
        match message {
            Ok(OwnedMessage::Text(t)) => {
                let json = serde_json::from_str(&t).unwrap();
                info!("WS recv {}", json);
                Ok(json)
            }
            Ok(OwnedMessage::Close(_)) => {
                let mut builder = ClientBuilder::new(&self.url).unwrap();
                self.client = builder.connect_insecure().unwrap();
                Err("WS error, connection closed by server and failed to reconnect".to_owned())
            }
            Err(_) => Ok(json!({})),
            _ => Err("WS error, non-text message received".to_owned()),
        }
    }

    fn watch_account(&mut self, sub: WsSubscription) {
        let account = sub.account;
        let address = account.lock().unwrap().address();
        if self.watched_accounts.len() == 0 {
            let options = json!({ "accounts": vec![address.clone()] });
            self.subscribe("confirmation".to_owned(), Some(options), sub.tx_response)
                .unwrap();
        } else {
            let options = json!({ "accounts_add": vec![address.clone()] });
            self.update("confirmation".to_owned(), Some(options), sub.tx_response)
                .unwrap();
        }
        self.watched_accounts.insert(address, account.clone());
    }

    // fn unwatch_account(&mut self, account: Arc<Mutex<Account>>) {
    //     let address = account.lock().unwrap().address();
    //     let options: Value = json!({ "accounts_del": vec![address.clone()] });
    //     self.update("confirmation".to_owned(), Some(options))
    //         .unwrap();
    //     self.watched_accounts.remove(&address);
    // }

    fn run_sender(wsc: Arc<Mutex<WsClient>>, rx: Receiver<WsSubscription>) {
        loop {
            let sub = rx.recv().unwrap();
            let mut wsc = wsc.lock().unwrap();
            if let Some(_) = wsc.awaiting_ack {
                // We are already waiting on another command ack, let's wait for that before sending another one...
                // todo move polling rate to config?
                drop(wsc);
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            wsc.watch_account(sub);
        }
    }

    fn run_listener(wsc: Arc<Mutex<WsClient>>) {
        loop {
            let mut wsc = wsc.lock().unwrap();
            match wsc.recv() {
                Err(e) => {
                    panic!("{}", e);
                }
                Ok(v) => {
                    if v == json!({}) {
                        // todo move polling rate to config?
                        drop(wsc); // key drop which unlocks wsc
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }

                    // Check for ack
                    let ack = v["ack"].as_str();
                    match (ack, &wsc.awaiting_ack) {
                        (Some(ack), Some((topic, tx))) => {
                            tx.send(()).unwrap();
                            if ack != *topic {
                                panic!("WS error unexpected ack?!");
                            }
                            wsc.awaiting_ack = None;
                            continue;
                        }
                        (None, _) => {}
                        _ => {
                            panic!("WS error unexpected ack?!");
                        }
                    }

                    let message: JsonConfirmation =
                        serde_json::from_value(v["message"].clone()).unwrap();

                    // Update the block sender info upon confirmation
                    match &wsc.watched_accounts.entry(message.account) {
                        Vacant(_) => {}
                        Occupied(entry) => {
                            entry.get().clone().lock().unwrap().update_info();
                        }
                    }

                    // Receive incoming send blocks to watched linked accounts
                    let linked_account = &wsc.watched_accounts.get(&message.block.link_as_account);
                    match linked_account {
                        Some(account) => account
                            .lock()
                            .unwrap()
                            .receive_block(message.hash, message.amount),
                        None => {}
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct JsonSubscribeMessage {
    action: String,
    topic: String,
    ack: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Value>,
}

#[derive(Deserialize)]
struct JsonConfirmation {
    account: Address,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    amount: Raw,
    block: Block,
    confirmation_type: String,
    hash: String,
}
