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
mod auth;

// 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 => Login request
// 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 => Signup request

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    name: Vec<u8>,
    password: Vec<u8>,
    token: Vec<u8>,
    id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    value: String,
    id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserData {
    messages: Vec<Message>,
}

// Config
#[derive(Deserialize)]
pub struct Config {
    connection: Connection,
    queue: Queue,
}

#[derive(Deserialize)]
pub struct Connection {
    ip: String,
    port: u16,
}

#[derive(Deserialize)]
pub struct Queue {
    queue_poll_cooldown: u64,
    event_execution_cooldown: u64,
}

fn main() -> std::io::Result<()> {
    {
        // clears the screen
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        std::io::stdout().flush().unwrap();

        // imports config
        let config_str = std::fs::read_to_string("config.toml").unwrap();
        let config: Config = toml::from_str(&config_str).unwrap();

        let (event_sender, event_receiver) = mpsc::channel();
        let (auth_sender, aut_receiver) = mpsc::channel();

        let mut events: Vec<String> = Vec::new();
        print_header("Server Init".to_string(), 40);
        let now = std::time::Instant::now();

        // spawns thread for handling input
        input::handle(event_sender.clone());
        events.push("initiated input-handler".to_string());

        auth::init(
            auth_sender.clone(),
            aut_receiver
        );
        events.push(String::from("initiated auth-handler"));

        // spawns thread for handling events
        queue::init(
            event_receiver,
            event_sender.clone(),
            auth_sender.clone(),
            config.queue.queue_poll_cooldown,
            config.queue.event_execution_cooldown,
        );

        events.push("initiated event-handler".to_string());
        events.push("".to_string());
        events.push(format!(
            "queue_poll_cooldown: {} ms",
            config.queue.queue_poll_cooldown
        ));
        events.push(format!(
            "event_execution_cooldown: {} ms",
            config.queue.event_execution_cooldown
        ));
        events.push("".to_string());

        let mut accounts = get_accounts(auth_sender.clone());

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
        events.push("updated authentification tokens".to_string());

        let address = config.connection.ip + ":" + &config.connection.port.to_string();
        let listener = match TcpListener::bind(&address) {
            Ok(listener) => listener,
            Err(e) => {
                events.push(format!(
                    "could not bind server to address {}\n> [{}]",
                    &address, e
                ));
                std::process::exit(1);
            }
        };
        events.push(format!(
            "server listens on: {}",
            listener.local_addr().unwrap()
        ));
        events.push(format!(
            "started in: {:#?} milliseconds",
            now.elapsed().as_millis()
        ));
        print_body(events, 40);

        print_users(&accounts, 40);

        for s in listener.incoming() {
            let client_event_sender = event_sender.clone();
            let client_auth_sender = auth_sender.clone();
            thread::spawn(move || -> std::io::Result<()> {
                let stream = s.unwrap();

                print_header("new connection".to_string(), 40);
                print_body(
                    vec![format!("from: {}", stream.peer_addr().unwrap().to_string())],
                    40,
                );

                    let mut accounts = get_accounts(client_auth_sender);

                client::handle(stream, &mut accounts, client_event_sender)?;

                Ok(())
            });
        }
    } // the socket is closed here
    Ok(())
}
