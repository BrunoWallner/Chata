use std::net::TcpListener;
use std::net::TcpStream;
use std::net::Shutdown;

use std::thread;

use sha2::{Sha512, Digest};

use std::fs::File;
use std::io::{Read, Write};

use serde::{Serialize, Deserialize};

use rand::prelude::*;

mod client;
mod functions;
use functions::*;

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


fn main() -> std::io::Result<()> {
    {
        //
        let mut file = File::open("data.bin")?;
        let mut encoded: Vec<u8> = Vec::new();
        file.read_to_end(&mut encoded)?;
    
        let mut accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();
        //accounts.push(create_account("Luca3".to_string(), "4498".to_string(), "1_3_5_6".to_string()));
        
        // prints all Users at server startup
        println!("┌──────────────────────────────────────┐");
        println!("│         List of all accounts         │");
        println!("├──────────────────────────────────────┤");
        for i in 0..accounts.len() {
            //let row = (i + 1).to_string() + ": " + &accounts[i].id[0..(36-4)];
            let mut row = String::new();
            if accounts[i].id.clone().len() < 34 {
                row = (i + 1).to_string() + ": " + &accounts[i].id;
            } else {
                row = (i + 1).to_string() + ": " + &accounts[i].id[0..34];
            }
            print!("│ {}", row);
            for _ in 0..(37) - row.len() {
                print!(" ");
            }
            print!("│\n");
        }
        print!("└──────────────────────────────────────┘");
        print!("\n\n");

        /*
        let mut accounts: Vec<Account> = Vec::new();
        accounts.push(create_account("Luca".to_string(), "3260".to_string(), "BrunoWallner".to_string()));
        */
    
        // generates tokens
        for account in accounts.iter_mut() {
            let mut token: Vec<u8> = Vec::new();
            let mut rng = thread_rng();
            for _ in 0..255 {
                token.push(rng.gen_range(0 .. 255));
            }
            account.token = token;
        }

        let serialized: Vec<u8> = bincode::serialize(&accounts.clone()).unwrap();

        let mut file = File::create("data.bin")?;
        match file.write_all(&serialized) {
            Ok(..) => (),
            Err(e) => println!("{}", e),
        };

        println!("> updated auth tokens");


        let listener = TcpListener::bind("127.0.0.1:8080")?;
        println!("> server listens on: {}", listener.local_addr().unwrap());
        print!("\n");
        for s in listener.incoming() {
            thread::spawn(move || -> std::io::Result<()> {
                let stream = s.unwrap();
                print_addr(stream.peer_addr()?);
                println!("> connected");

                let mut file = File::open("data.bin")?;
                let mut encoded: Vec<u8> = Vec::new();
                file.read_to_end(&mut encoded)?;
            
                let mut accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();

                println!("> read data from file");
                print!("\n");

                client::handle(stream, &mut accounts)?;

                // writes accounts back to file
                let serialized: Vec<u8> = bincode::serialize(&accounts.clone()).unwrap();

                let mut file = File::create("data.bin")?;
                match file.write_all(&serialized) {
                    Ok(..) => (),
                    Err(e) => println!("{}", e),
                };

                //println!("{:#?}", accounts[0]);

                Ok(())
            });
        }

    } // the socket is closed here
    Ok(())
}