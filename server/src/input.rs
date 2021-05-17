use std::io::stdin;

use crate::*;

pub fn handle(sender: mpsc::Sender<queue::Event>) {
    thread::spawn(move || {
        loop {
            match input().as_str() {
                "" => (),
                "exit" => std::process::exit(0),
                "print_users" | "print users" => {
                    let mut file = File::open("data.bin").unwrap();
                    let mut encoded: Vec<u8> = Vec::new();
                    file.read_to_end(&mut encoded).unwrap();
            
                    let accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();
                    print_users(&accounts, 40);
                }
                "send" => {
                    sender.send(
                        queue::Event::MessageSent(["Hallo", "mein", "Meister"])
                    ).unwrap()
                }
                command => println!("{}: command not found.", command),
            }
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
