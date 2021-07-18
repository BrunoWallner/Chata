use std::io::stdin;
use colored::*;


use crossterm::cursor;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use std::io::{stdout, Write};

use crate::*;

enum KeyReaderEvent {
    NewKey(char),
    RequestKey(mpsc::Sender<char>),
}

pub fn init(sender: mpsc::Sender<queue::Event>, ip: String, port: u16) {
    /* loop {
        let key = read_key();
        match key {
            Some(c) => println!("{}", c),
            None => (),
        };
    } */
    let (key_sender, key_receiver) = mpsc::channel();
    // key reader
    let key_sender_clone = key_sender.clone();
    thread::spawn(move || {
        loop {
            match read().unwrap() {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => key_sender_clone.send(KeyReaderEvent::NewKey(c)).unwrap(),
                _ => (),
            }
        }
    });

    // keyevent thread
    thread::spawn(move || {
        let mut key: char  = '\0';

        loop {
            match key_receiver.recv().unwrap() {
                KeyReaderEvent::NewKey(c) => key = c,
                KeyReaderEvent::RequestKey(sender) => {
                    sender.send(key).unwrap();
                }
            }
        }
    });

    let key_sender_clone = key_sender.clone();
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
            let key_sender_clone = key_sender_clone.clone();
            thread::spawn(move || -> std::io::Result<()> {
                let stream = s.unwrap();

                receive_instructions(stream, sender_clone, key_sender_clone);

                Ok(())
            });
        }
    });
}

fn receive_instructions(mut stream: TcpStream, sender: mpsc::Sender<queue::Event>, key_sender: mpsc::Sender<KeyReaderEvent>) -> std::io::Result<()> {
    // creates token
    let mut rng = thread_rng();
    let mut auth_token: &mut [u8; 256] = &mut [0; 256];
    for i in 0..256 {
        auth_token[i] = rng.gen_range(0..255);
    }

    print(console::State::ImportantInformation(format!("{} requested full command controll, accept? <y/n>", stream.peer_addr().unwrap().to_string().red())));

    let (tx, rc) = mpsc::channel();

    key_sender.send(KeyReaderEvent::RequestKey(tx)).unwrap();

    let key = rc.recv().unwrap();
    println!("got {}", key);

    match key.to_string().as_str() {
        "y" | "Y" => {
            print(console::State::ImportantInformation(format!("granted {} full command access", stream.peer_addr().unwrap().to_string().red())));
            stream.write(auth_token).ok();
        },
        "n" | "N" | "" => {
            print(console::State::ImportantInformation(format!("denied {} full command access", stream.peer_addr().unwrap().to_string().red())));
            stream.write(&vec_to_buffer(&vec![0, 0, 0, 0])).ok();
        },
        _ => (),
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

use std::time::Duration;

fn input() -> String {
    let mut input_string = String::new();
    stdin()
        .read_line(&mut input_string)
        .ok()
        .expect("Failed to read line");
    return input_string.trim().to_string();
}

fn read_key() -> Option<char> {
    match read().unwrap() {
        //i think this speaks for itself
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
            //clearing the screen and printing our message
        }) => return Some(c),
        _ => return None,
    }
}
/*
enum InputTimeoutEvent {
    IsTimeout(mpsc::Sender<bool>),
    Timeout(),
    IsInput(mpsc::Sender<bool>),
    Input(),
    Exit(),
}

fn input_timeout(
    timeout: u64,
    sender: mpsc::Sender<String>,
) {
    let (tx, rc) = mpsc::channel();

    thread::spawn(move || {
        let mut timeout: bool = false;
        let mut input: bool = false;
        let mut exit: u8 = 0;
        loop {
            match rc.recv().unwrap() {
                InputTimeoutEvent::IsTimeout(sender) => {
                    sender.send(timeout).unwrap();
                },
                InputTimeoutEvent::Timeout() => {
                    timeout = true;
                },
                InputTimeoutEvent::IsInput(sender) => {
                    sender.send(input).unwrap();
                },
                InputTimeoutEvent::Input() => {
                    input = true;
                },
                InputTimeoutEvent::Exit() => exit+=1,
            };
            if exit >= 2 {println!("exited loop"); break}
        }
    });

    let sender_clone = sender.clone();
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let (tx_1, rc_1) = mpsc::channel();
        let input = input();
        tx_clone.send(InputTimeoutEvent::IsTimeout(tx_1)).unwrap();
        if !rc_1.recv().unwrap() {
            tx_clone.send(InputTimeoutEvent::Input()).unwrap();
            println!("sent event to tell that its already inputed");
            sender_clone.send(input).unwrap();
        } else {
            println!("already timeouted");
        }
        tx_clone.send(InputTimeoutEvent::Exit()).unwrap();
    });

    let sender_clone = sender.clone();
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let (tx_1, rc_1) = mpsc::channel();
        thread::sleep(Duration::from_millis(timeout));
        tx_clone.send(InputTimeoutEvent::IsInput(tx_1)).unwrap();
        if !rc_1.recv().unwrap() {
            tx_clone.send(InputTimeoutEvent::Timeout()).unwrap();
            println!("sent event to tell that its already timeouted");
            sender_clone.send(String::from("")).unwrap();
        } else {
            println!("already inputed");
        }
        tx_clone.send(InputTimeoutEvent::Exit()).unwrap();
        tx_clone.send(InputTimeoutEvent::Exit()).unwrap();
    });
;
}
*/
