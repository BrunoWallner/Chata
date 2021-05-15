use crate::*;

pub fn handle(mut stream: TcpStream, accounts: &mut Vec<Account>) -> std::io::Result<()> {
    let addr = stream.peer_addr().unwrap();
    let mut buffer = [0; 8]; // 8 Byte Buffer
    'keepalive: loop {
        match stream.read(&mut buffer) {
            Ok(_) => {
                match buffer[0..8] {
                    // kill connection
                    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        print_addr(addr);
                        println!("> close request");
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

                        println!("> login attempt");

                        match check_login(&accounts, name, passwd) {
                            Ok(i) => {
                                println!("> valid login");
                                stream.write(&vec_to_buffer(&accounts[i].token))?;
                            },
                            Err(_) => {
                                println!("> invalid login");
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
                println!(">  error ocurred:\n> [{}]", e);
                break 'keepalive;
            },
        }
    //print!("\n");
    }
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => println!("> closed connection to client"),
        Err(e) => println!("> failed to close connection to client:\n> [{}]", e),
    };
    print!("\n");
    Ok(())
}