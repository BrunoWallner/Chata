use std::thread;
use std::time::Duration;
use std::time::Instant;

use std::io::{Read, Write};
use std::net::TcpStream;

static PORT: u32 = 8080;

pub fn login(ip: String, name: String, password: String) -> Result<[u8; 256], String> {
    match TcpStream::connect(ip + ":" + &PORT.to_string()) {
        Ok(mut stream) => {
            stream.write(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
            stream.write(&string_to_buffer(name)).unwrap();
            stream.write(&string_to_buffer(password)).unwrap();

            let mut data = [0 as u8; 256]; // using 256 KB buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if data == [0 as u8; 256] {
                        return Err("invalid login".to_string());
                    } else {
                        return Ok(data);
                    }
                }
                Err(_) => {
                    return Err("Failed to recieve data".to_string());
                }
            }
        }
        Err(_) => {
            return Err("Server unreachable".to_string());
        }
    }
}

fn string_to_buffer(name: String) -> [u8; 256] {
    let mut buffer: [u8; 256] = [0; 256];
    buffer[0] = name.len() as u8 + 1;

    for i in 0..name.len() {
        buffer[i + 1] = name.as_bytes()[i];
    }

    buffer
}
