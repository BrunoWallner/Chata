use std::time::Instant;
use std::time::Duration;
use std::thread;

use std::net::{TcpStream};
use std::io::{Read, Write};

static PORT: u32 = 3333;

pub fn login(ip: String, name: String, password: String) -> Result<[u8; 256], String> {
    match TcpStream::connect(ip + ":" + &PORT.to_string()) {
        Ok(mut stream) => {

            let msg = create_login(name, password);

            stream.write(&msg).unwrap();

            let mut data = [0 as u8; 256]; // using 8 KB buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if data == [0 as u8; 256] {
                        return Err("invalid login".to_string());
                    } else {
                        return Ok(data);
                    }
                },
                Err(_) => {
                    return Err("Failed to recieve data".to_string());
                }
            }
        },
        Err(_) => {
            return Err("Server unreachable".to_string());
        }
    }
}
fn create_login(name: String, password: String) -> [u8; 8192] {
    let mut buffer = [0x00 as u8; 8192];
    buffer[0] = 0x01;
    for i in 0..name.len() {
        buffer[i + 1] = name.as_bytes()[i];
    }
    for i in 0..password.len() {
        buffer[i + 1 + 4096] = password.as_bytes()[i];
    }
    buffer
}