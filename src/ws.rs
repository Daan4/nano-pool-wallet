use websocket::{ClientBuilder, Message};
use websocket::client::sync::Client;
use websocket::message::OwnedMessage;
use serde::{Serialize, Deserialize};
use serde_aux::prelude::*;
use serde_json::Value;
use std::net::TcpStream;

pub struct WsClient {
    url: String,
    client: Client<TcpStream>
}

impl WsClient {
    pub fn new(url: String) -> Self {
        let mut builder = ClientBuilder::new(&url).unwrap();
        let mut client = builder.connect_insecure().unwrap();
        Self {
            url,
            client
        }
    }

    fn subscribe(&mut self, topic: String) -> Result<(), String> {
        let json = JsonSubscribeMessage {
            action: "subscribe".to_owned(),
            ack: true,
            topic: topic.to_owned(),
        };
        self.send(serde_json::to_value(json).unwrap()).unwrap();
        
        let response: JsonSubscribeResponse = serde_json::from_value(self.recv().unwrap()).unwrap();
        if response.ack == "subscribe".to_owned() {
            Ok(())
        } else {
            Err("WS error, subscribe ack invalid".to_owned())
        }
    }

    fn send(&mut self, json: Value) -> Result<(), String> {
        println!("WS send {}\n", json);
        let message = Message::text(json.to_string());
        self.client.send_message(&message).unwrap();
        Ok(())
    }

    fn recv(&mut self) -> Result<Value, String> {
        let message = self.client.recv_message().unwrap();
        match message {
            OwnedMessage::Text(t) => {
                let json = serde_json::from_str(&t).unwrap();
                println!("WS recv {}\n", json);
                Ok(json)
            },
            OwnedMessage::Close(_) => {
                Err("WS error, connection closed by server".to_owned())
            },
            _ => {
                Err("WS error, non-text message received".to_owned())
            }
        }
    }

    pub fn run(&mut self) {
        self.subscribe("confirmation".to_owned()).unwrap();

        loop {
            let message = self.recv();
        }
    }
}

#[derive(Serialize, Deserialize)]
struct JsonSubscribeMessage {
    action: String,
    topic: String,
    ack: bool
}

#[derive(Deserialize)]
struct JsonSubscribeResponse {
    ack: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    time: u64,
}
