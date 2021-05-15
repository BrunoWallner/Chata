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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Account {
    name: Vec<u8>,
    password: Vec<u8>,
    token: Vec<u8>,
    id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    value: String,
    id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserData {
    messages: Vec<Message>,
}


fn main() -> std::io::Result<()> {
    {
        //
        let mut file = File::open("data.bin")?;
        let mut encoded: Vec<u8> = Vec::new();
        file.read_to_end(&mut encoded)?;
    
        let mut accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();
        //
        println!("> List of all accounts: ");
        for i in 0..accounts.len() {
            println!("  => {}", accounts[i].id);
        }
        

        /*
        let mut accounts: Vec<Account> = Vec::new();
        accounts.push(create_account("Luca".to_string(), "3260".to_string(), "BrunoWallner".to_string()));
        */
    
        // generates tokens
        for account in accounts.iter_mut() {
            let mut token: Vec<u8> = Vec::new();
            let mut rng = thread_rng();
            for _ in 0..255 {
                token.push(rng.gen_range(0 .. u8::MAX));
            }
            account.token = token;
        }

        let serialized: Vec<u8> = bincode::serialize(&accounts.clone()).unwrap();

        let mut file = File::create("data.bin")?;
        match file.write_all(&serialized) {
            Ok(..) => (),
            Err(e) => println!("{}", e),
        };

        println!(">>> updated auth tokens");


        let listener = TcpListener::bind("127.0.0.1:8080")?;
        println!(">>> server listens on: {}", listener.local_addr().unwrap());
        print!("\n");
        for s in listener.incoming() {
            thread::spawn(move || -> std::io::Result<()> {
                let stream = s.unwrap();
                println!(">> new client connected: {}", stream.peer_addr()?);

                let mut file = File::open("data.bin")?;
                let mut encoded: Vec<u8> = Vec::new();
                file.read_to_end(&mut encoded)?;
            
                let mut accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();

                println!(">  read data from file");

                handel_client(stream, &mut accounts)?;

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

fn handel_client(mut stream: TcpStream, accounts: &mut Vec<Account>) -> std::io::Result<()> {
    let mut buffer = [0; 8]; // 8 Byte Buffer
    'keepalive: loop {
        match stream.read(&mut buffer) {
            Ok(_) => {
                    match buffer[0..8] {
                    // kill connection
                    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01] => {
                        break 'keepalive;
                    }
                    // login
                    [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        let name: &[u8];
                        let passwd: &[u8];

                        // recieves login name
                        let mut buffer = [0; 256];
                        stream.read_exact(&mut buffer)?;
                        name = &buffer[1..buffer[0] as usize + 1];

                        // recieves login passwd
                        let mut buffer = [0; 256];
                        stream.read_exact(&mut buffer)?;
                        passwd = &buffer[1..buffer[0] as usize + 1];

                        println!(">  login attempt");

                        match check_login(&accounts, name, passwd) {
                            Ok(i) => {
                                println!(">  valid login");
                                stream.write(&vec_to_buffer(&accounts[i].token))?;
                            },
                            Err(_) => {
                                println!(">  invalid login");
                                stream.write(&vec_to_buffer(&vec![0, 0, 0, 0, 0, 0, 0, 0]))?;
                            },
                        }
                        print!("\n");
                    },
                    // signup
                    [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        
                    },
                    // chat message push request
                    [0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        let token: &[u8];
                        let message: String;

                        // recieves token
                        let mut buffer = [0; 256];
                        stream.read_exact(&mut buffer)?;

                        token = &buffer[1..buffer[0] as usize + 1];

                        let mut buffer = [0; 256];
                        stream.read_exact(&mut buffer)?;

                        message = match std::str::from_utf8(&buffer[1..buffer[0] as usize + 1]) {
                            Ok(value) => value.to_string(),
                            Err(_) => "[INVALID]".to_string(),
                        };

                        let mut buffer = [0; 256];
                        stream.read_exact(&mut buffer)?;

                        let id: Vec<u8> = buffer[1..buffer[0] as usize + 1].to_vec();

                        println!("> message push request");

                        match check_token(accounts, &token[..]) {
                            Ok(send_account_number) => {
                                println!("> valid token");
                                let string_id = std::str::from_utf8(&id).unwrap().to_string();

                                match search_by_id(accounts, string_id.clone()) {
                                    Ok(_) => {
                                        println!("> found user with id");

                                        // imports userdata from ./userdata/[USERID]
                                        let mut file = match File::open("userdata/".to_string() + &string_id.clone()) {
                                            Ok(file) => {
                                                println!("> successfully opened file");
                                                file
                                            }
                                            Err(_) => { 
                                                println!("> could not open file, created new empty one");
                                                File::create("userdata/".to_string() + &string_id.clone()).unwrap();
                                                File::open("userdata/".to_string() + &string_id.clone()).unwrap()
                                            }
                                        };
                                        let mut encoded: Vec<u8> = Vec::new();
                                        file.read_to_end(&mut encoded)?;

                                        let mut userdata: UserData = match bincode::deserialize(&encoded) {
                                            Ok(userdata) => userdata,
                                            Err(e) => {
                                                println!("> failed to deserialize userdata from user: {}", string_id);
                                                println!("> {}", e);
                                                UserData { messages: Vec::new() }
                                            }
                                        };

                                        let sender_id = accounts[send_account_number].id.clone();
                                        userdata.messages.push(Message {value: message, id: sender_id });

                                        // saves userdata back to file
                                        let serialized: Vec<u8> = bincode::serialize(&userdata.clone()).unwrap();

                                        let mut file = File::create("userdata/".to_string() + &string_id.clone())?;
                                        match file.write_all(&serialized) {
                                            Ok(..) => println!("> wrote changes back to file"),
                                            Err(e) => println!("{}", e),
                                        };
                                    },
                                    Err(_) => {
                                        println!("> couldnt find user with id");
                                    },
                                }
                            }
                            Err(_) => {
                                println!("> invalid token");
                            }
                        }
                    },
                    // message pull request
                    [0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        println!("> message pull request");
                        let mut buffer = [0u8; 256];
                        stream.read_exact(&mut buffer)?;
                        println!("> recieved token");

                        let token = &buffer[1..buffer[0] as usize + 1];

                        match check_token(&accounts, token) {
                            Ok(account_number) => {
                                println!("> valid token");
                                // imports userdata from ./userdata/[USERID]
                                let string_id = &accounts[account_number].id.clone();

                                let mut file = match File::open("userdata/".to_string() + &string_id.clone()) {
                                    Ok(file) => {
                                        println!("> successfully opened file");
                                        file
                                    }
                                    Err(_) => { 
                                        println!("> could not open file, created new empty one");
                                        File::create("userdata/".to_string() + &string_id.clone()).unwrap();
                                        File::open("userdata/".to_string() + &string_id.clone()).unwrap()
                                    }
                                };

                                // reads file into buffer
                                let mut encoded: Vec<u8> = Vec::new();
                                file.read_to_end(&mut encoded)?;

                                // deserializes userdata from file
                                let userdata: UserData = match bincode::deserialize(&encoded) {
                                    Ok(userdata) => userdata,
                                    Err(e) => {
                                        println!("> failed to deserialize userdata from user: {}", string_id);
                                        println!("> {}", e);
                                        UserData { messages: Vec::new() }
                                    }
                                };

                                // cycles trough every message from user and sends it to client
                                for i in 0..userdata.messages.len() {
                                    // writes message
                                    stream.write(&string_to_buffer(userdata.messages[i].value.clone()))?;

                                    // writes sender id
                                    stream.write(&string_to_buffer(userdata.messages[i].id.clone()))?;
                                }
                                // all messages were sent
                                stream.write(&vec_to_buffer(&vec![2, 2, 2, 2, 2, 2, 2, 2]))?;

                            },
                            Err(_) => {
                                println!("> invalid token");
                                stream.write(&vec_to_buffer(&vec![1, 1, 1, 1, 1, 1, 1, 1]))?;
                            }
                        }
                    }
                    _ => (),
                };
            },
            Err(e) => {
                println!(">  error ocurred: [{}]", e);
                match stream.shutdown(Shutdown::Both) {
                    Ok(_) => println!(">  closed connection to client"),
                    Err(e) => println!(">  failed to close connection to client: [{}]", e),
                };
            },
        }
    print!("\n");
    }
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

fn check_token(accounts: &Vec<Account>, token: &[u8]) -> Result<usize, ()>{
    for i in 0..accounts.len() {
        if accounts[i].token == token {
            return Ok(i);
        }
    }
    return Err(());
}

fn create_account(name: String, password: String, id: String) -> Account {
    // creates data for Account
    let mut hashed_name = Sha512::new();
    hashed_name.update(name.as_bytes());

    let mut hashed_password = Sha512::new();
    hashed_password.update(password.as_bytes());

    let mut token: Vec<u8> = Vec::new();
    for _ in 0..255 {
        token.push(thread_rng().gen_range(0..u8::MAX))
    };

    // creates data and file in ./userdata
    let mut file = match File::create("userdata/".to_string() + &id.to_string()) {
        Ok(file) => file,
        Err(_) => panic!("failed to create file in ./userdata for user: {}", id),
    };
    let userdata = UserData{ messages: Vec::new() };
    let serialized = bincode::serialize(&userdata).unwrap();
    match file.write_all(&serialized) {
        Ok(..) => (),
        Err(e) => println!("{}", e),
    };

    Account { 
        name: hashed_name.finalize().to_vec(), 
        password: hashed_password.finalize().to_vec(),
        token: token,
        id: id,
    }
}

fn search_by_id(accounts: &Vec<Account>, id: String) -> Result<usize, ()> {
    for i in 0..accounts.len() {
        if id == accounts[i].id {
            return Ok(i);
        }
    }
    Err(())
}

fn string_to_buffer(string: String) -> [u8; 256] {
    let mut buffer: [u8; 256] = [0; 256];
    buffer[0] = string.len() as u8;

    for i in 0..string.len() {
        buffer[i + 1] = string.as_bytes()[i];
    }

    buffer
}

fn vec_to_buffer(vec: &Vec<u8>) -> [u8; 256] {
    let mut buffer: [u8; 256] = [0; 256];
    buffer[0] = vec.len() as u8;

    for i in 0..vec.len() {
        buffer[i + 1] = vec[i];
    }

    buffer
}