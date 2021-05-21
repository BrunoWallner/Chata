use crate::*;

pub fn handle(
    mut stream: TcpStream,
    accounts: &mut Vec<Account>,
    sender: mpsc::Sender<queue::Event>,
) -> std::io::Result<()> {
    let addr = stream.peer_addr().unwrap();
    'keepalive: loop {
        let mut buffer = [0; 8]; // 8 Byte Buffer
        match stream.read(&mut buffer) {
            Ok(_) => {
                match buffer[0..8] {
                    // kill connection
                    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        print_header("close request".to_string(), 40);
                        print_body(vec![format!("from: {}", addr)], 40);
                        break 'keepalive;
                    }
                    // login
                    [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        print_header("Login request".to_string(), 40);
                        let mut events: Vec<String> = Vec::new();
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

                        events.push(format!("from: {}", addr));

                        match check_login(&accounts, name, passwd) {
                            Ok(i) => {
                                events.push(format!("valid login"));
                                stream.write(&vec_to_buffer(&accounts[i].token))?;
                            }
                            Err(_) => {
                                events.push(format!("invalid login"));
                                stream.write(&vec_to_buffer(&vec![0, 0, 0, 0, 0, 0, 0, 0]))?;
                            }
                        }
                        print_body(events, 40);
                    }
                    // signup
                    [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        print_header("signup request".to_string(), 40);
                        let mut messages: Vec<String> = Vec::new();
                        messages.push(format!("from: {}", addr));

                        let mut buffer = [0; 256];
                        stream.read_exact(&mut buffer)?;
                        let name = std::str::from_utf8(&buffer[1..buffer[0] as usize + 1])
                            .unwrap()
                            .to_string();

                        let mut buffer = [0; 256];
                        stream.read_exact(&mut buffer)?;
                        let passwd = std::str::from_utf8(&buffer[1..buffer[0] as usize + 1])
                            .unwrap()
                            .to_string();

                        let mut buffer = [0; 256];
                        stream.read_exact(&mut buffer)?;
                        let id = std::str::from_utf8(&buffer[1..buffer[0] as usize + 1])
                            .unwrap()
                            .to_string();

                        sender
                            .send(queue::Event::CreateUser([name, passwd, id]))
                            .unwrap();
                        messages.push(format!("sent signup event"));
                        print_body(messages, 40);
                    }
                    // chat message push request
                    [0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        print_header("message push request".to_string(), 40);
                        let mut events: Vec<String> = Vec::new();
                        let token: &[u8];
                        let message: String;
                        let sender_id: String;

                        events.push(format!("from: {}", addr));

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

                        match check_token(accounts, &token[..]) {
                            Ok(send_account_number) => {
                                events.push(format!("invalid token"));
                                let string_id = std::str::from_utf8(&id).unwrap().to_string();

                                match search_by_id(accounts, string_id.clone().to_string()) {
                                    Ok(_) => {
                                        events.push(format!("found user with id"));

                                        //write_message(string_id, message, accounts[send_account_number].id.clone());
                                        sender_id = accounts[send_account_number].id.clone();
                                        sender
                                            .send(queue::Event::SendMessage([
                                                string_id, message, sender_id,
                                            ]))
                                            .unwrap();
                                    }
                                    Err(_) => {
                                        events.push(format!(
                                            "could not find user with id: {}",
                                            string_id.clone()
                                        ));
                                    }
                                }
                            }
                            Err(_) => {
                                events.push(format!("invalid token"));
                            }
                        }
                        print_body(events, 40);
                    }
                    // message pull request
                    [0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        print_header("message pull request ".to_string(), 40);
                        let mut events: Vec<String> = Vec::new();

                        let mut buffer = [0u8; 256];
                        stream.read_exact(&mut buffer)?;

                        let token = &buffer[1..buffer[0] as usize + 1];

                        match check_token(&accounts, token) {
                            Ok(account_number) => {
                                events.push(format!("valid token"));
                                // imports userdata from ./userdata/[USERID]
                                let string_id = &accounts[account_number].id.clone();

                                let mut file = match File::open(
                                    "userdata/".to_string() + &string_id.clone(),
                                ) {
                                    Ok(file) => {
                                        events.push(format!("successfully opened file"));
                                        file
                                    }
                                    Err(_) => {
                                        events
                                            .push(format!("could not open file, created new one"));
                                        File::create("userdata/".to_string() + &string_id.clone())
                                            .unwrap();
                                        File::open("userdata/".to_string() + &string_id.clone())
                                            .unwrap()
                                    }
                                };

                                // reads file into buffer
                                let mut encoded: Vec<u8> = Vec::new();
                                file.read_to_end(&mut encoded)?;

                                // deserializes userdata from file
                                let userdata: UserData = match bincode::deserialize(&encoded) {
                                    Ok(userdata) => userdata,
                                    Err(e) => {
                                        events.push(format!(
                                            "failed to deserialize user data from {}",
                                            string_id
                                        ));
                                        events.push(format!("[{}]", e));
                                        UserData {
                                            messages: Vec::new(),
                                        }
                                    }
                                };

                                // cycles trough every message from user and sends it to client
                                for i in 0..userdata.messages.len() {
                                    // writes message
                                    stream.write(&string_to_buffer(
                                        userdata.messages[i].value.clone(),
                                    ))?;

                                    // writes sender id
                                    stream.write(&string_to_buffer(
                                        userdata.messages[i].id.clone(),
                                    ))?;
                                }
                                // all messages were sent
                                stream.write(&vec_to_buffer(&vec![2, 2, 2, 2, 2, 2, 2, 2]))?;
                            }
                            Err(_) => {
                                events.push(format!("invalid token"));
                                stream.write(&vec_to_buffer(&vec![1, 1, 1, 1, 1, 1, 1, 1]))?;
                            }
                        }
                        print_body(events, 40);
                    }
                    _ => break 'keepalive,
                };
            }
            Err(e) => {
                print_header("an error occurred".to_string(), 40);
                print_body(vec![format!("[{}]", e)], 40);
                break 'keepalive;
            }
        }
        //print!("\n");
    }
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => {
            print_header("closed connection to client".to_string(), 35);
            print_body(vec!["operation successfull".to_string()], 35);
        }
        Err(e) => {
            print_header(format!("failed to close connection to client"), 35);
            print_body(vec![e.to_string()], 35);
        }
    };
    print!("\n");
    Ok(())
}
