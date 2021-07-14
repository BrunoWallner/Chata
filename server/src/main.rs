use std::net::Shutdown;
use std::net::TcpListener;
use std::net::TcpStream;

use std::thread;

use sha2::{Digest, Sha512};

use std::fs::File;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use rand::prelude::*;

use std::sync::mpsc;

mod client;
mod functions;
use functions::*;

mod input;
mod queue;
mod user;
mod console;
use console::*;

// 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 => Login request
// 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 => Signup request

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    name: Vec<u8>,
    password: Vec<u8>,
    token: Vec<u8>,
    id: String,
}

// Config
#[derive(Deserialize)]
pub struct Config {
    connection: Connection,
    authentification: Authentification,
}

#[derive(Deserialize)]
pub struct Connection {
    ip: String,
    port: u16,
}

#[derive(Deserialize)]
pub struct Authentification {
    auth_save_cooldown: u64,
    user_save_cooldown: u64,
}

fn main() -> std::io::Result<()> {
    {
        // clears the screen
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        std::io::stdout().flush().unwrap();

        // imports config
        let config_str = match std::fs::read_to_string("config.toml") {
            Ok(config) => config,
            Err(e) => {
                println!("could not find config.toml:\n{}", e);
                std::process::exit(1);
            }
        };
        let config: Config = toml::from_str(&config_str).unwrap();

        let (event_sender, event_receiver) = mpsc::channel();

        let now = std::time::Instant::now();

        // spawns thread for handling input
        input::handle(event_sender.clone());
        print(State::Information(String::from("initiated input handler")));

        // spawns thread for handling events
        queue::init(
            event_receiver,
            event_sender.clone(),
            config.authentification.auth_save_cooldown,
            config.authentification.user_save_cooldown,
        );

        print(State::Information(String::from("initiated event-handler")));

        let mut accounts = get_accounts(event_sender.clone());

        // generates tokens
        for account in accounts.iter_mut() {
            let mut token: Vec<u8> = Vec::new();
            let mut rng = thread_rng();
            for _ in 0..255 {
                token.push(rng.gen_range(0..255));
            }
            account.token = token;
        }

        let serialized: Vec<u8> = bincode::serialize(&accounts.clone()).unwrap();

        let mut file = File::create("data.bin")?;
        match file.write_all(&serialized) {
            Ok(..) => (),
            Err(e) => println!("{}", e),
        };
        print(State::Information(String::from("updated authentification tokens")));

        let address = config.connection.ip + ":" + &config.connection.port.to_string();
        let listener = match TcpListener::bind(&address) {
            Ok(listener) => listener,
            Err(e) => {
                print(State::CriticalError(format!(
                    "could not bind server to address {} [{}]",
                    &address, e
                )));
                std::process::exit(1);
            }
        };
        print(State::Information(format!(
            "server listens on: {}",
            listener.local_addr().unwrap()
        )));
        print(State::Information(format!(
            "started in: {:#?} milliseconds",
            now.elapsed().as_millis()
        )));

        print_users(&accounts, 40);

        for s in listener.incoming() {
            let client_event_sender = event_sender.clone();
            thread::spawn(move || -> std::io::Result<()> {
                let stream = s.unwrap();
                let mut accounts = get_accounts(client_event_sender.clone());

                client::handle(stream, &mut accounts, client_event_sender)?;

                Ok(())
            });
        }
    } // the socket is closed here
    Ok(())
}
