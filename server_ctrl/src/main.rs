use std::io::{Read, Write};
use std::net::TcpStream;

use std::io::stdin;

fn main() {
    print!("name: ");
    std::io::stdout().flush().ok();
    let name = input();

    print!("IP: ");
    std::io::stdout().flush().ok();
    let ip = input();

    let mut stream: TcpStream = match TcpStream::connect(ip.as_str()) {
        Ok(stream) => stream,
        Err(_) => {
            println!("{} refused the connection", ip);
            std::process::exit(1);
        }
    };

    stream.write(&string_to_buffer(String::from(name))).unwrap();


    print!("password: ");
    std::io::stdout().flush().ok();
    let passwd = input();
    stream.write(&string_to_buffer(passwd)).unwrap();

    let mut buffer = [0; 256];
    stream.read_exact(&mut buffer).unwrap();
    let token = &buffer[0..256];

    if token[0] != 1 {
        println!("peer denied command permission");
        std::process::exit(1);
    } else {
        println!("peer granted command permission");
    }

    loop {
        print!("> ");
        std::io::stdout().flush().ok();
        let input = input();

        stream.write(&[1, 1, 1, 1, 1, 1, 1, 1]).unwrap();
        stream.write(token).unwrap();
        stream.write(&string_to_buffer(input)).unwrap();

        let mut response: Vec<String> = Vec::new();
        'receiving: loop {
            let mut buffer = [0u8; 256];
            stream.read_exact(&mut buffer).unwrap();

            match buffer[0] {
                0 => {
                    //response.push(String::from(""));
                    break 'receiving;
                }
                _ => {
                    let string = match std::str::from_utf8(&buffer[1..buffer[0] as usize + 1]) {
                        Ok(v) => String::from(v),
                        Err(_) => String::from("[UTF-8 ERROR]"),
                    };
                    response.push(string);
                }
            }
        }
        match response[0].as_str() {
            "print::users" => {
                print_users(response[1..response.len() - 1].to_vec(), 40);
            }
            _ => (),
        };

        match response[response.len() - 1].as_str() {
            "INVALID_TOKEN" => {
                println!("INVALID TOKEN");
                std::process::exit(1);
            },
            "EXIT" => {
                println!("SERVER SHUTDOWN");
                std::process::exit(1);
            },
            r => println!("{}", r),
        };
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

// prints all Users at server startup
fn print_users(accounts: Vec<String>, width: usize) {
    print_header("List of all accounts".to_string(), width);
    print_body(accounts, width);
}

fn print_header(string: String, width: usize) {
    print!("┌");
    for _ in 0..width {
        print!("─");
    }
    print!("┐");
    print!("\n");

    print!("│");
    for _ in 0..width / 2 - string.len() / 2 {
        print!(" ");
    }
    print!("{}", string);
    for _ in 0..width - ((width / 2 - string.len() / 2) + string.len()) {
        print!(" ");
    }
    print!("│\n");

    print!("├");
    for _ in 0..width {
        print!("─");
    }
    print!("┤");
    print!("\n");
}
use std::iter::FromIterator;
fn print_body(strings: Vec<String>, width: usize) {
    for string in strings.iter() {
        //let row = (i + 1).to_string() + ": " + &accounts[i].id[0..(36-4)];
        let row: String;
        if string.chars().count() <= width {
            row = string.to_string();
        } else {
            let char_vec: Vec<char> = string.chars().collect();
            row = String::from_iter(&char_vec[0..width]);
        }
        print!("│{}", row);
        for _ in 0..width - row.chars().count() {
            print!(" ");
        }
        print!("│\n");
    }
    print!("└");
    for _ in 0..width {
        print!("─");
    }
    print!("┘");
    print!("\n");
}
