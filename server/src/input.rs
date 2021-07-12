use std::io::stdin;

use crate::*;

pub fn handle(sender: mpsc::Sender<queue::Event>) {
    thread::spawn(move || loop {
        let input = input();
        let parameter: Vec<&str> = input.split("::").collect();
        match parameter[0] {
            "" => (),
            "clear" => {
                //print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                //print!("\x1B[2J\x1B[1;1H");
                print!("{esc}c", esc = 27 as char);
                std::io::stdout().flush().unwrap();
            }
            "exit" => std::process::exit(0),
            "print" => {
                if parameter.len() >= 1 {
                    match parameter[1] {
                        "users" => {
                            let accounts = functions::get_accounts(sender.clone());
                            print_users(&accounts, 40);
                        }
                        _ => println!("> invalid parameter"),
                    }
                } else {
                    println!("> invalid parameter");
                }
            }
            "users" => {
                if parameter.len() >= 1 {
                    match parameter[1] {
                        "delete" => {
                            if parameter.len() > 2 {
                                let user: String = parameter[2].to_string();
                                sender.send(queue::Event::DeleteUser(user)).unwrap();
                                println!("> sent userdeletion event");
                            } else {
                                println!("> invalid parameter");
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
                                println!("> sent usercreaton event");
                            } else {
                                println!("> invalid parameter");
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
                                println!("> sent user write event");
                            } else {
                                println!("> invalid parameter");
                            }
                        }
                        _ => println!("> invalid parameter"),
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
