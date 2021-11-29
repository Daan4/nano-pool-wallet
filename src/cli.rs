use log::info;
use std::io::{stdin, stdout, Write};
use std::process;
use std::sync::mpsc::Sender;
use std::thread;

use crate::address::Address;
use crate::common;
use crate::config::CONFIG;
use crate::rpc::RpcCommand;
use crate::unit::Raw;
use crate::wallet::Wallet;
use crate::ws::WsSubscription;

/// Start command line interface
pub fn start_cli(rpc_tx: Sender<RpcCommand>, ws_tx: Sender<WsSubscription>) {
    let seed = common::hexstring_to_bytes(&CONFIG.wallet_seed);
    let wallet = Wallet::new(seed, rpc_tx.clone(), ws_tx.clone());
    CliClient::start(wallet);
}

/// CLI commands
#[derive(Debug, PartialEq)]
enum Command {
    /// Send directly from wallet account
    SendDirect(Address, Raw),
    /// Send payment via wallet account pool
    SendPayment(Address, Raw),
    /// Receive payment via wallet account pool
    ReceivePayment(Raw),
    /// Exit program
    Exit,
    /// Display help
    Help,
    /// Undefined command
    Undefined,
}

pub struct CliClient {
    wallet: Wallet,
}

impl CliClient {
    pub fn start(wallet: Wallet) {
        let mut cli = Self { wallet };

        thread::Builder::new()
            .name("cli".to_owned())
            .spawn(move || {
                cli.run();
            })
            .unwrap();
    }

    fn run(&mut self) {
        loop {
            let mut buf = String::new();
            print!(">");
            stdout().flush().unwrap();
            stdin().read_line(&mut buf).unwrap();
            let cmd = CliClient::process_input(&buf);
            match self.execute_command(cmd) {
                Err(e) => {
                    println!("<{}", e);
                }
                Ok(_) => {}
            }
        }
    }

    fn process_input(buf: &str) -> Command {
        let buf = buf.trim();
        let split: Vec<&str> = buf.split(' ').collect();
        if split.len() == 0 {
            return Command::Undefined;
        }
        let cmd: &str = &split[0].to_lowercase();
        match cmd {
            "send_direct" => {
                if split.len() < 3 {
                    Command::Undefined
                } else {
                    match split[2].to_owned().parse::<Raw>() {
                        Err(_) => Command::Undefined,
                        Ok(raw) => Command::SendDirect(split[1].to_owned(), raw),
                    }
                }
            }
            "send_payment" => {
                if split.len() < 3 {
                    Command::Undefined
                } else {
                    match split[2].to_owned().parse::<Raw>() {
                        Err(_) => Command::Undefined,
                        Ok(raw) => Command::SendPayment(split[1].to_owned(), raw),
                    }
                }
            }
            "receive_payment" => {
                if split.len() < 2 {
                    Command::Undefined
                } else {
                    match split[1].to_owned().parse::<Raw>() {
                        Err(_) => Command::Undefined,
                        Ok(raw) => Command::ReceivePayment(raw),
                    }
                }
            }
            "exit" => Command::Exit,
            "help" => Command::Help,
            _ => Command::Undefined,
        }
    }

    fn execute_command(&mut self, cmd: Command) -> Result<(), String> {
        info!("CLI exec {:?}", cmd);
        match cmd {
            Command::SendDirect(address, amount) => self.send_direct(address, amount),
            Command::SendPayment(address, amount) => self.send_payment(address, amount),
            Command::ReceivePayment(amount) => self.receive_payment(amount),
            Command::Exit => process::exit(0),
            Command::Help => CliClient::print_help(),
            Command::Undefined => {
                Err("Invalid command; type 'help' to see valid commands".to_owned())
            }
        }
    }

    fn send_direct(&mut self, address: Address, amount: Raw) -> Result<(), String> {
        self.wallet.send_direct(amount, address);
        Ok(())
    }

    fn send_payment(&mut self, address: Address, amount: Raw) -> Result<(), String> {
        self.wallet.send_payment(amount, address)
    }

    fn receive_payment(&mut self, amount: Raw) -> Result<(), String> {
        self.wallet.receive_payment(amount)
    }

    fn print_help() -> Result<(), String> {
        println!("<send_direct <nano_address> <amount_in_raw> -- Send raw from the wallet account directly to a nano address");
        println!("<send_payment <nano_address> <amount_in_raw> -- Send raw from the wallet account via the account pool");
        println!("<receive_payment <amount_in_raw> -- Receive a specific amo8unt of raw to the wallet account via the account pool");
        println!("<exit -- Exit the program");
        println!("<help -- Show this help text");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_process_input_no_node_required() {
        assert_eq!(
            CliClient::process_input("send_direct Arg1 123"),
            Command::SendDirect("Arg1".to_owned(), 123)
        );
        assert_eq!(
            CliClient::process_input("Send_direct Arg1 123"),
            Command::SendDirect("Arg1".to_owned(), 123)
        );
        assert_eq!(
            CliClient::process_input("send_direct Arg1 123 junk data here"),
            Command::SendDirect("Arg1".to_owned(), 123)
        );
        assert_eq!(CliClient::process_input("send_direct"), Command::Undefined);
        assert_eq!(
            CliClient::process_input("send_direct Arg1"),
            Command::Undefined
        );
        assert_eq!(
            CliClient::process_input("send_direct Arg1 Arg2"),
            Command::Undefined
        );

        assert_eq!(
            CliClient::process_input("send_payment Arg1 123"),
            Command::SendPayment("Arg1".to_owned(), 123)
        );
        assert_eq!(
            CliClient::process_input("Send_payment Arg1 123"),
            Command::SendPayment("Arg1".to_owned(), 123)
        );
        assert_eq!(
            CliClient::process_input("send_payment Arg1 123 junk data here"),
            Command::SendPayment("Arg1".to_owned(), 123)
        );
        assert_eq!(CliClient::process_input("send_payment"), Command::Undefined);
        assert_eq!(
            CliClient::process_input("send_payment Arg1"),
            Command::Undefined
        );
        assert_eq!(
            CliClient::process_input("send_payment Arg1 Arg2"),
            Command::Undefined
        );

        assert_eq!(
            CliClient::process_input("receive_payment 123"),
            Command::ReceivePayment(123)
        );
        assert_eq!(
            CliClient::process_input("receive_payment 123"),
            Command::ReceivePayment(123)
        );
        assert_eq!(
            CliClient::process_input("receive_payment 123 junk data here"),
            Command::ReceivePayment(123)
        );
        assert_eq!(
            CliClient::process_input("receive_payment"),
            Command::Undefined
        );
        assert_eq!(
            CliClient::process_input("receive_payment Arg1"),
            Command::Undefined
        );

        assert_eq!(CliClient::process_input("exit"), Command::Exit);
        assert_eq!(CliClient::process_input("Exit"), Command::Exit);
        assert_eq!(
            CliClient::process_input("exit and some more stuff"),
            Command::Exit
        );

        assert_eq!(CliClient::process_input("help"), Command::Help);
        assert_eq!(CliClient::process_input("Help"), Command::Help);
        assert_eq!(
            CliClient::process_input("help and some more stuff"),
            Command::Help
        );

        assert_eq!(CliClient::process_input(""), Command::Undefined);
        assert_eq!(
            CliClient::process_input("thisisnotacommand"),
            Command::Undefined
        );
        assert_eq!(
            CliClient::process_input("This Is Also Not A Command!"),
            Command::Undefined
        );
    }
}
