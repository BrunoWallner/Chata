use std::io::{Read, Write};
use std::net::TcpStream;

use std::io::stdin;

static IP: &str = "localhost:10000";

fn main() {
    let mut stream: TcpStream = match TcpStream::connect(IP) {
        Ok(stream) => stream,
        Err(_) => {
            println!("{} refused the connection", IP);
            std::process::exit(1);
        }
    };

    let mut buffer = [0; 256];
    stream.read_exact(&mut buffer).unwrap();
    let token = &buffer[0..256];

    if token[0] == 4 {
        println!("peer revoked command permission");
        std::process::exit(1);
    } else {
        println!("peer granted command permission");
    }

    loop {
        let input = input();

        stream.write(&[1, 1, 1, 1, 1, 1, 1, 1]);
        stream.write(token);
        stream.write(&string_to_buffer(input));
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
