use std::net::TcpListener;
use std::net::TcpStream;
use std::net::Shutdown;

use std::thread;

use sha2::{Sha512, Digest};

use std::fs::File;
use std::io::{Read, Write};

use serde::{Serialize, Deserialize};

use rand::prelude::*;

// 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 => Login request
// 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 => Signup request

#[derive(Serialize, Deserialize, Clone)]
struct Account<> {
    name: Vec<u8>,
    password: Vec<u8>,
    token: Vec<u8>,
} 

fn main() -> std::io::Result<()> {
    {
        let mut file = File::open("auth.bin")?;
        let mut encoded: Vec<u8> = Vec::new();
        file.read_to_end(&mut encoded)?;
    
        let mut accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();
    
        // generates tokens
        for account in accounts.iter_mut() {
            let mut token: Vec<u8> = Vec::new();
            let mut rng = thread_rng();
            for _ in 0..256 {
                token.push(rng.gen_range(0 .. u8::MAX));
            }
            account.token = token;
        }

        let serialized: Vec<u8> = bincode::serialize(&accounts.clone()).unwrap();

        let mut file = File::create("auth.bin")?;
        match file.write_all(&serialized) {
            Ok(..) => (),
            Err(e) => println!("{}", e),
        };
        println!("> updated auth tokens");



        let listener = TcpListener::bind("127.0.0.1:8080")?;
        //listener.set_nonblocking(true);
        //socket.set_nonblocking(true)?;
        for stream in listener.incoming() {
            thread::spawn(move || -> std::io::Result<()> {
                println!("> new client connected");

                let mut file = File::open("auth.bin")?;
                let mut encoded: Vec<u8> = Vec::new();
                file.read_to_end(&mut encoded)?;
            
                let mut accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();

                handel_client(stream.unwrap(), &mut accounts)?;

                Ok(())
            });
        }

    } // the socket is closed here
    Ok(())
}

fn handel_client(mut stream: TcpStream, accounts: &mut Vec<Account>) -> std::io::Result<()> {
    let mut buffer = [0; 8]; // 8 Byte Buffer
    match stream.read(&mut buffer) {
        Ok(_) => {
                match buffer[0..8] {
                // login
                [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                    let name: &[u8];
                    let passwd: &[u8];

                    // recieves login name
                    let mut buffer = [0; 256];
                    stream.read_exact(&mut buffer)?;
                    name = &buffer[1..buffer[0] as usize];

                    // recieves login passwd
                    let mut buffer = [0; 256];
                    stream.read_exact(&mut buffer)?;
                    passwd = &buffer[1..buffer[0] as usize];

                    println!("> login attempt");

                    match check_login(&accounts, name, passwd) {
                        Ok(i) => {
                            println!("> valid login");
                            stream.write(&accounts[i].token)?;
                        },
                        Err(_) => {
                            println!("> invalid login");
                            stream.write(&[0; 256])?;
                        },
                    }
                    println!("\n");
                },
                // signup
                [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                    
                }
                _ => (),
            };
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
        },
    } {}
    Ok(())
}

fn check_login(accounts: &Vec<Account>, name: &[u8], passwd: &[u8]) -> Result<usize, ()> {
    for i in 0..accounts.len() {

        let mut hashed_name = Sha512::new();
        hashed_name.update(&name);

        let mut hashed_password = Sha512::new();
        hashed_password.update(&passwd);

        if *accounts[i].name ==  *hashed_name.finalize()
        && *accounts[i].password == *hashed_password.finalize() {
            return Ok(i);
        }
    }
    return Err(());
}

fn check_token(accounts: Vec<Account>, token: &[u8]) -> bool {
    for account in accounts.iter() {
        if account.token == token {
            return true;
        }
    }
    return false;
}