use std::io::stdin;
use colored::*;

use crate::*;

pub fn init(sender: mpsc::Sender<queue::Event>, ip: String, port: u16) {
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
            print(console::State::ImportantInformation(String::from("new instruction connection")));
            let sender_clone = sender.clone();
            thread::spawn(move || -> std::io::Result<()> {
                let stream = s.unwrap();

                receive_instructions(stream, sender_clone);

                Ok(())
            });
        }
    });
}

fn receive_instructions(mut stream: TcpStream, sender: mpsc::Sender<queue::Event>) -> std::io::Result<()> {
    // creates token
    let mut rng = thread_rng();
    let mut auth_token: &mut [u8; 256] = &mut [0; 256];
    for i in 0..256 {
        auth_token[i] = rng.gen_range(0..255);
    }
    print!("{} {}", stream.peer_addr().unwrap().to_string().red(), "requested full command controll, accept? <y/n> ".blue());
    std::io::stdout().flush();
    match input().as_str() {
        "y" | "Y" => stream.write(auth_token)?,
        "n" | "N" => stream.write(&vec_to_buffer(&vec![0, 0, 0, 0]))?,
        _ => 0,
    };

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
                            print(console::State::CriticalError(format!("{}: invalid command token", stream.peer_addr().unwrap())));
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
        "exit" => sender.send(queue::Event::ServerShutdown()).unwrap() ,
        "print" => {
            if parameter.len() >= 1 {
                match parameter[1] {
                    "users" => {
                        let accounts = get_accounts(sender.clone());
                        print_users(&accounts, 40);
                    }
                    _ => ()
                }
            }
        }
        "users" => {
            if parameter.len() >= 1 {
                match parameter[1] {
                    "delete" => {
                        if parameter.len() > 2 {
                            let user: String = parameter[2].to_string();
                            sender.send(queue::Event::DeleteUser(user)).unwrap();
                        }
                    }
                    "create" => {
                        if parameter.len() > 4 {
                            let name: String = parameter[2].to_string();
                            let passwd: String = parameter[3].to_string();
                            let id: String = parameter[4].to_string();

                            sender
                                .send(queue::Event::CreateUser([name, passwd, id]))
                                .unwrap();
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
                        }
                    }
                    "inspect" => {
                        if parameter.len() > 2 {
                            match user::request_single(String::from(parameter[2]), sender.clone()) {
                                Ok(user) => println!("{:#?}", user),
                                Err(_) => println!("could not find user"),
                            };
                        } else {
                            println!("> invalid parameter");
                        }
                    }
                    _ => println!("> invalid parameter"),
                }
            }
        }
        "echo" => {
            if parameter.len() > 1 {
                println!("> {}", parameter[1]);
            }
        }
        command => print(console::State::Error(format!("{}: command not found", command))),
    }
}

fn input() -> String {
    let mut input_string = String::new();
    stdin()
        .read_line(&mut input_string)
        .ok()
        .expect("Failed to read line");
    return input_string.trim().to_string();
}
