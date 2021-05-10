use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use core::str::from_utf8;

use sha2::{Sha256, Sha512, Digest};

use std::fs::File;

use serde::{Serialize, Deserialize};


use colorful::Color;
use colorful::Colorful;
use colorful::RGB;

// 0x00 -> empty
// 0x01 -> login
// 0x02 -> signup
// 0x03 -> password reset request
// 0x04 -> message pull request
// 0x05 -> message push requeset

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Account<'a> {
    name: &'a [u8],
    password: &'a [u8],
    token: &'a [u8],
} 

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 8192]; // using 50 byte buffer

    let mut file = std::fs::File::open("auth.bin").unwrap();
    let mut encoded: Vec<u8> = Vec::new();
    file.read_to_end(&mut encoded).unwrap();

    let accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();
    println!("> {}", "imported login data from auth.bin".color(RGB::new(200, 200, 50)));

    let mut connected: bool = false;
    while match stream.read(&mut data) {
        Ok(_) => {
            match data[0] {
                // login
                0x01 => {
                    println!("> {}", "recieved login attempt".color(RGB::new(0, 255, 255)));
                    let i = check_login(accounts.clone(), data);
                    if i != 0 {
                        stream.write(accounts[i-1].token.clone());
                        if !connected {
                            println!("> {}", "successfully logged in\n".color(RGB::new(0, 255, 0)))
                        }
                    } else {
                        stream.write(&[0; 8192]);
                        if !connected {
                            println!("> {}", "invalid login\n".color(RGB::new(255, 0, 0)));
                        }
                    }
                },
                // message
                0x05 => {
                    println!("> {}", "recieved message".color(RGB::new(0, 255, 255)));
                    if check_token(accounts.clone(), &data[7936..8192]) {
                        println!("> {}", "valid token".color(RGB::new(0, 255, 0)));
                    } else {
                        println!("> {}", "invalid token".color(RGB::new(255, 0, 0)));
                    }
                    println!("> message: {}\n", from_utf8(&data[1..7936]).unwrap() );
                },
                _ => (),
            }
            connected = true;
            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {break}
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("> Server listening on port 3333");

    /*
    let mut accounts: Vec<Account> = Vec::new();
    accounts.push(Account { 
        name: "".as_bytes(),
        password: "3260".as_bytes(),
        token: "0zn798wogxz".as_bytes(),
    });
    // hashes account informations
    let mut hashed_name = Sha512::new();
    hashed_name.update(&string_to_buffer(0x00, "Luca".to_string())[1..4096]);
    let finalized_name = &hashed_name.finalize();

    let mut hashed_password = Sha512::new();
    hashed_password.update(&string_to_buffer(0x00, "3260".to_string())[1..4096]);
    let finalized_password = &hashed_password.finalize();

    let finalized_token = &string_to_buffer(0x00, "fUMI0z7m9)90m8zzum09zJKHoTBt0n9zd=NZ(".to_string())[1..257];

    accounts[0].name = &finalized_name;
    accounts[0].password = &finalized_password;
    accounts[0].token = &finalized_token;


    let encoded: Vec<u8> = bincode::serialize(&accounts.clone()).unwrap();

    let mut file = File::create("auth.bin").expect("create failed");
    file.write_all(&encoded).expect("write failed");
    println!("> created file that contains hashed login at auth.bin\n");
    */

    // refreshes tokens
    let mut file = std::fs::File::open("auth.bin").unwrap();
    let mut encoded: Vec<u8> = Vec::new();
    file.read_to_end(&mut encoded).unwrap();

    let mut accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();

    let finalized_token = &string_to_buffer(0x00, "fUMI0z7m9)90m8zzum09zJKHoTBt0n9zd=NZ(".to_string())[1..257];
    for account in accounts.iter_mut() {
        account.token = &*finalized_token.clone();
    }

    let encoded: Vec<u8> = bincode::serialize(&accounts.clone()).unwrap();

    let mut file = File::create("auth.bin").expect("create failed");
    file.write_all(&encoded).expect("write failed");
    println!("> updated tokens\n");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("{} {}", "client connected as:".color(RGB::new(200, 200, 50)), stream.peer_addr().unwrap().to_string().color(RGB::new(255, 0, 0)));
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
    Ok(())
}

fn string_to_buffer(meta: u8, string: String) -> [u8; 8192] {
    let mut buffer = [0 as u8; 8192];
    buffer[0] = meta;
    for i in 1..=string.len() {
        buffer[i] = string.as_bytes()[i-1];
    }
    buffer
}

fn u8_to_buffer(meta: u8, u: &[u8]) -> [u8; 8192] {
    let mut buffer = [0 as u8; 8192];
    buffer[0] = meta;
    for i in 1..=u.len() {
        buffer[i] = u[i-1];
    }
    buffer
}

fn check_login(accounts: Vec<Account>, login: [u8; 8192]) -> usize {
    for i in 0..accounts.len() {

        let mut hashed_name = Sha512::new();
        hashed_name.update(&login[1..4096]);

        let mut hashed_password = Sha512::new();
        hashed_password.update(&login[4097..8192]);

        if *accounts[i].name ==  *hashed_name.finalize()
        && *accounts[i].password == *hashed_password.finalize() {
            return i+1;
        }
    }
    return 0;
}

fn check_token(accounts: Vec<Account>, token: &[u8]) -> bool {
    for account in accounts.iter() {
        if account.token == token {
            return true;
        }
    }
    return false;
}