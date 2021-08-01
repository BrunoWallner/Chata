use colored::*;

use std::io::Write;

use crate::*;

pub fn init(sender: mpsc::Sender<queue::Event>, ip: String, port: u16, password_length: u8) {
    thread::spawn(move || {
        let address = ip + ":" + &port.to_string();
        let listener = match TcpListener::bind(&address) {
            Ok(listener) => listener,
            Err(e) => {
                print(State::CriticalError(format!(
                    "could not bind input server to address {} [{}]",
                    &address, e
                )));
                std::process::exit(1);
            }
        };
        for s in listener.incoming() {
            let sender_clone = sender.clone();
            thread::spawn(move || -> std::io::Result<()> {
                let stream = s.unwrap();

                receive_instructions(stream, sender_clone, password_length).ok();

                Ok(())
            });
        }
    });
}

fn receive_instructions(mut stream: TcpStream, sender: mpsc::Sender<queue::Event>, password_length: u8) -> std::io::Result<()> {
    // receives name
    let mut buffer = [0u8; 256];
    stream.read_exact(&mut buffer).unwrap();
    let name = match std::str::from_utf8(&buffer[1..buffer[0] as usize + 1]) {
        Ok(v) => v,
        Err(_) => "UTF-8_ENCODING_ERROR",
    };

    // creates token
    let mut rng = thread_rng();
    let mut auth_passwd: String = String::new();
    let auth_token: &mut [u8; 256] = &mut [0; 256];
    for i in 0..256 {
        auth_token[i] = rng.gen_range(0..255);
    }
    auth_token[0] = 1;
    for _ in 0..password_length {
        auth_passwd.push(rng.gen_range(0..9).to_string().chars().collect::<Vec<char>>()[0]);
    }

    print(console::State::ImportantInformation(format!("{} | {} requested full command controll, password: [{}]", stream.peer_addr().unwrap().to_string().red(), name.red(), auth_passwd.clone())));

    let mut buffer = [0u8; 256];
    stream.read_exact(&mut buffer).unwrap();
    let input_passwd = match std::str::from_utf8(&buffer[1..buffer[0] as usize + 1]) {
        Ok(v) => v,
        Err(_) => "[INVALID UTF8 ENCODING]"
    };

    if input_passwd == auth_passwd {
        print(console::State::ImportantInformation(format!("granted {} | {} full command access", stream.peer_addr().unwrap().to_string().red(), name.red())));
        stream.write(auth_token).unwrap();
    } else {
        print(console::State::ImportantInformation(format!("denied {} | {} full command access", stream.peer_addr().unwrap().to_string().red(), name.red())));
        stream.write(&vec_to_buffer(&vec![0, 0, 0, 0])).unwrap();
    }

    'keepalive: loop {
        let mut buffer = [0; 8]; // 8 Byte Buffer
        match stream.read_exact(&mut buffer) {
            Ok(_) => {
                match buffer[0..8] {
                    [1, 1, 1, 1, 1, 1, 1, 1] => {
                        let token: &[u8];
                        let instruction: &[u8];

                        // recieves login name
                        let mut buffer = [0; 256];
                        stream.read_exact(&mut buffer)?;
                        token = &buffer[0..256];

                        // recieves login passwd
                        let mut buffer = [0; 256];
                        stream.read_exact(&mut buffer)?;
                        instruction = &buffer[1..buffer[0] as usize + 1];

                        if token == auth_token {
                            match std::str::from_utf8(&instruction) {
                                Ok(value) => handle(sender.clone(), String::from(value), stream.try_clone().unwrap()),
                                Err(_) => (),
                            };
                        } else {
                            print(console::State::CriticalError(format!("{} | {}: invalid command token", stream.peer_addr().unwrap(), name.red())));
                            stream.write(&string_to_buffer(String::from("INVALID_TOKEN"))).unwrap();
                            stream.write(&[0u8; 256]).unwrap();
                            break 'keepalive;
                        }
                    },
                    _ => break 'keepalive,

                };
            }
            Err(_) => {
                break 'keepalive;
            }
        }
     }
     Ok(())
}

fn handle(sender: mpsc::Sender<queue::Event>, instruction: String, mut stream: TcpStream) {
    let input = instruction;
    let parameter: Vec<&str> = input.split("::").collect();
    match parameter[0] {
        "" => (),
        "clear" => {
            //print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            //print!("\x1B[2J\x1B[1;1H");
            print!("{esc}c", esc = 27 as char);
            std::io::stdout().flush().unwrap();
        }
        "exit" => {
            stream.write(&string_to_buffer(String::from("EXIT"))).unwrap();
            stream.write(&[0u8; 256]).unwrap();
            sender.send(queue::Event::ServerShutdown()).unwrap();
        }
        "save" => {
            stream.write(&string_to_buffer(String::from("OK"))).unwrap();
            stream.write(&[0u8; 256]).unwrap();

            sender.send(queue::Event::SaveAuthData()).unwrap();
            sender.send(queue::Event::SaveUserData()).unwrap()
        }
        "print" => {
            if parameter.len() >= 1 {
                match parameter[1] {
                    "users" => {
                        stream.write(&string_to_buffer(String::from("print::users"))).unwrap();
                        let accounts = get_accounts(sender.clone());
                        for account in accounts.iter() {
                            stream.write(&string_to_buffer(account.id.clone())).unwrap();
                        }
                        stream.write(&string_to_buffer(String::from("OK"))).unwrap();
                        stream.write(&[0u8; 256]).unwrap();
                    },
                    _ => {
                        stream.write(&string_to_buffer(String::from("INVALID_PARAMETER"))).unwrap();
                        stream.write(&[0u8; 256]).unwrap();
                    },
                }
            } else {
                stream.write(&string_to_buffer(String::from("INVALID_PARAMETER"))).unwrap();
                stream.write(&[0u8; 256]).unwrap();
            }
        }
        "users" => {
            if parameter.len() >= 1 {
                match parameter[1] {
                    "delete" => {
                        if parameter.len() > 2 {
                            let user: String = parameter[2].to_string();
                            sender.send(queue::Event::DeleteUser(user)).unwrap();

                            stream.write(&string_to_buffer(String::from("OK"))).unwrap();
                            stream.write(&[0u8; 256]).unwrap();
                        } else {
                            stream.write(&string_to_buffer(String::from("INVALID_PARAMETER"))).unwrap();
                            stream.write(&[0u8; 256]).unwrap();
                        }
                    }
                    "create" => {
                        if parameter.len() > 4 {
                            let name: String = parameter[2].to_string();
                            let passwd: String = parameter[3].to_string();
                            let id: String = parameter[4].to_string();

                            let (tx, rx) = mpsc::channel();

                            sender
                                .send(queue::Event::CreateUser((
                                    Some(tx),
                                    [name, passwd, id]
                                )))
                                .unwrap();
                            match rx.recv().unwrap() {
                                Ok(_) => {
                                    stream.write(&string_to_buffer(String::from("OK"))).unwrap();
                                    stream.write(&[0u8; 256]).unwrap();
                                },
                                Err(_) => {
                                    stream.write(&string_to_buffer(String::from("ACCOUNT_DOES_NOT_MEET_REQUIREMENTS"))).unwrap();
                                    stream.write(&[0u8; 256]).unwrap();
                                }
                            }
                        } else {
                            stream.write(&string_to_buffer(String::from("INVALID_PARAMETER"))).unwrap();
                            stream.write(&[0u8; 256]).unwrap();
                        }
                    }
                    "write" => {
                        if parameter.len() > 3 {
                            let id: String = parameter[2].to_string();
                            let message: String = parameter[3].to_string();

                            sender
                                .send(queue::Event::SendMessage([
                                    id,
                                    message,
                                    "[CONSOLE]".to_string(),
                                ]))
                                .unwrap();
                            stream.write(&string_to_buffer(String::from("OK"))).unwrap();
                            stream.write(&[0u8; 256]).unwrap();
                        } else {
                            stream.write(&string_to_buffer(String::from("INVALID_PARAMETER"))).unwrap();
                            stream.write(&[0u8; 256]).unwrap();
                        }
                    }
                    _ => {
                        stream.write(&string_to_buffer(String::from("INVALID_PARAMETER"))).unwrap();
                        stream.write(&[0u8; 256]).unwrap();
                    },
                }
            } else {
                stream.write(&[0u8; 256]).unwrap();
            }
        }
        "echo" => {
            if parameter.len() > 1 {
                println!("> {}", parameter[1]);
                stream.write(&string_to_buffer(String::from("OK"))).unwrap();
                stream.write(&[0u8; 256]).unwrap();
            } else {
                stream.write(&string_to_buffer(String::from("INVALID_PARAMETER"))).unwrap();
                stream.write(&[0u8; 256]).unwrap();
            }
        }
        _ => {
            stream.write(&string_to_buffer(String::from("COMMAND_NOT_FOUND"))).unwrap();
            stream.write(&[0u8; 256]).unwrap();
        }
    }
}
