use crate::*;

pub fn handle(
    mut stream: TcpStream,
    accounts: &mut Vec<Account>,
    sender: mpsc::Sender<queue::Event>,
) -> std::io::Result<()> {
    'keepalive: loop {
        let mut buffer = [0; 8]; // 8 Byte Buffer
        match stream.read(&mut buffer) {
            Ok(_) => {
                match buffer[0..8] {
                    // kill connection
                    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
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

                        match check_login(&accounts, name, passwd) {
                            Ok(i) => {
                                stream.write(&vec_to_buffer(&accounts[i].token))?;
                                stream.write(&string_to_buffer(accounts[i].id.clone()))?;
                            }
                            Err(_) => {
                                stream.write(&vec_to_buffer(&vec![0, 0, 0, 0, 0, 0, 0, 0]))?;
                            }
                        }
                    }
                    // signup
                    [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
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

                        let (tx, rx) = mpsc::channel();
                        sender
                            .send(queue::Event::CreateUser((
                                Some(tx),
                                [name, passwd, id])
                            ))
                            .unwrap();

                        match rx.recv().unwrap() {
                            Ok(token) => {
                                stream.write(&vec_to_buffer(&token)).unwrap();
                            },
                            Err(_) => {
                                stream.write(&vec_to_buffer(&vec![0x01, 0x01, 0x01, 0x01])).unwrap();
                            }
                        }
                    }
                    // chat message push request
                    [0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        let token: &[u8];
                        let message: String;
                        let sender_id: String;

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
                                let string_id = std::str::from_utf8(&id).unwrap().to_string();

                                match search_by_id(accounts, string_id.clone().to_string()) {
                                    Ok(_) => {
                                        //write_message(string_id, message, accounts[send_account_number].id.clone());
                                        sender_id = accounts[send_account_number].id.clone();
                                        sender
                                            .send(queue::Event::SendMessage([
                                                string_id, message, sender_id,
                                            ]))
                                            .unwrap();
                                    }
                                    Err(_) => {}
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    // message pull request
                    [0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
                        let mut buffer = [0u8; 256];
                        stream.read_exact(&mut buffer)?;

                        let token = &buffer[1..buffer[0] as usize + 1];

                        match check_token(&accounts, token) {
                            Ok(account_number) => {
                                // imports userdata from ./userdata/[USERID]
                                let string_id = &accounts[account_number].id.clone();

                                match user::request_single(string_id.clone(), sender.clone()) {
                                    Ok(userdata) => {
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
                                        stream.write(&vec_to_buffer(&vec![1, 1, 1, 1, 1, 1, 1, 1]))?;
                                    }
                                };
                            }
                            Err(_) => {
                                stream.write(&vec_to_buffer(&vec![1, 1, 1, 1, 1, 1, 1, 1]))?;
                            }
                        }
                    }
                    _ => break 'keepalive,
                };
            }
            Err(_) => {
                break 'keepalive;
            }
        }
        //print!("\n");
    }
    stream.shutdown(Shutdown::Both).unwrap();
    Ok(())
}
