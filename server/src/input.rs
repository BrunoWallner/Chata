use std::io::stdin;

use crate::*;

pub fn handle(sender: mpsc::Sender<queue::Event>) {
    thread::spawn(move || loop {
        let input = input();
        let parameter: Vec<&str> = input.split(" ").collect();
        match parameter[0] {
            "" => (),
            "clear" => {
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                std::io::stdout().flush().unwrap();
            },
            "exit" => std::process::exit(0),
            "print" => {
                if parameter.len() > 1 {
                    match parameter[1] {
                        "users" => {
                            let mut file = File::open("data.bin").unwrap();
                            let mut encoded: Vec<u8> = Vec::new();
                            file.read_to_end(&mut encoded).unwrap();
        
                            let accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();
                            print_users(&accounts, 40);
                        },
                        _ => println!("> invalid parameter"),
                    }
                } else {
                    println!("> invalid parameter");
                }
            }
            "delete" => {
                if parameter.len() > 1 {
                    match parameter[1] {
                        "user" => {
                            if parameter.len() > 2 {
                                let user: String = parameter[2].to_string();
                                sender.send(queue::Event::DeleteUser(user)).unwrap();
                                println!("> sent userdeletion event");
                            } else {
                                println!("> invalid parameter");
                            }
                        }
                        _ => (),
                    }
                } else {
                    println!("> invalid parameter");
                }
            }
            "create" => {
                if parameter.len() > 1 {
                    match parameter[1] {
                        "user" => {
                            if parameter.len() > 3 {
                                let name: String = parameter[2].to_string();
                                let passwd: String = parameter[3].to_string();
                                let id: String = parameter[4].to_string();

                                sender.send(queue::Event::CreateUser([name, passwd, id])).unwrap();
                                println!("> sent usercreaton event");
                            } else {
                                println!("> invalid parameter");
                            }
                        }
                        _ => (),
                    }
                } else {
                    println!("> invalid parameter");
                }
            }
            "echo" => {
                if parameter.len() > 1 {
                    println!("> {}", parameter[1]);
                } else {
                    println!("");
                }
            }
            command => println!("> {}: command not found", command),
        }
    });
}

fn input() -> String {
    let mut input_string = String::new();
    stdin()
        .read_line(&mut input_string)
        .ok()
        .expect("Failed to read line");
    return input_string.trim().to_string();
}