use std::io::{Read, Write};
use std::net::TcpStream;

use std::io::stdin;
use std::thread;
use std::time::{Duration, Instant};

mod backend;
use backend::*;

mod text_styling;
use text_styling::*;

fn main() {
    let mut stream = match connect(String::from("localhost:33333")) {
        Ok(s) => s,
        Err(_) => {
            println!("could not connect to server");
            std::process::exit(1);
        }
    };

    let mut token: Vec<u8> = Vec::new();
    let mut id: String = String::new();

    'option_selection: loop {
        let option = input("[login / signup]: ");
        match option.as_str() {
            "login" | "l" => {
                let name = input("name: ");
                let passwd = input("password: ");

                match login(
                    stream.try_clone().unwrap(),
                    name,
                    passwd
                ) {
                    Ok((t, i)) => {
                        token = t;
                        id = i;
                        break 'option_selection
                    },
                    Err(e) => println!("{}", e),
                };
            },
            "signup" | "s" => {
                let name = input("name: ");
                let passwd = input("password: ");
                let id = input("id: ");

                match signup(
                    stream.try_clone().unwrap(),
                    name,
                    passwd,
                    id,
                ) {
                    Ok(t) => {
                        token = t;
                        break 'option_selection
                    },
                    Err(e) => {
                        println!("{}", e);
                        println!("INFO:\nname, password and id must contain at least 1 alphabetic character\nand must be 4 characters long");
                    }
                }
            },
            _ => println!("invalid input")
        }
    }
    println!("logged in as {}", id);

    'option_selection: loop {
        let option = input("[list_messages / write_message]: ");
        match option.as_str() {
            "list_messages" => {
                let messages = match receive_messages(
                    &mut stream.try_clone().unwrap(),
                    &token.clone(),
                ) {
                    Ok(m) => m,
                    Err(_) => vec![Message {id: String::from("ERROR"), value: String::from("ERROR")}]
                };

                let messages = convert_messages(messages.clone());
                let mut ids: Vec<String> = Vec::new();
                for i in 0..messages.len() {
                    ids.push(format!("[{}] {}", i, messages[i][0]));
                }
                print_header(String::from("Contacts"), 40);
                print_body(ids, 40);
                let id = input("id: ");
                let mut id_number: Option<usize> = match id.parse::<usize>() {
                    Ok(v) => Some(v),
                    Err(_) => None,
                };
                if id_number >= Some(messages.len()) {
                    id_number = Some(messages.len() - 1)
                }
                if messages.len() > 0 {
                    for message in messages.iter() {
                        if message[0] == id {
                            print_header(message[0].clone(), 40);
                            print_body(message[1..message.len()].to_vec(), 40);
                        }
                        if id_number.is_some() {
                            if message[0] == messages[id_number.unwrap()][0] {
                                print_header(message[0].clone(), 40);
                                print_body(message[1..message.len()].to_vec(), 40);
                            }
                        }
                    }
                } else {
                    println!("no messages");
                }
            },
            "write_message" => {
                let message = input("message: ");
                let id = input("id: ");

                match send_message(
                    &mut stream.try_clone().unwrap(),
                    message,
                    id,
                    &token.clone()
                ) {
                    Ok(_) => println!("sent message"),
                    Err(e) => println!("could not send message {}", e),
                }
            }
            _ => (),
        }
    }
}

fn input(string: &str) -> String {
    print!("{}", string);
    std::io::stdout().flush().ok();

    let mut input_string = String::new();
    stdin()
        .read_line(&mut input_string)
        .ok()
        .expect("Failed to read line");
    return input_string.trim().to_string();
}

pub fn string_to_buffer(string: String) -> [u8; 256] {
    let mut buffer: [u8; 256] = [0; 256];
    buffer[0] = string.len() as u8;

    for i in 0..string.len() {
        buffer[i + 1] = string.as_bytes()[i];
    }

    buffer
}

pub fn vec_to_buffer(vec: &Vec<u8>) -> [u8; 256] {
    let mut buffer: [u8; 256] = [0; 256];
    buffer[0] = vec.len() as u8;

    for i in 0..vec.len() {
        buffer[i + 1] = vec[i];
    }

    buffer
}
