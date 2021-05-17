use std::thread;
use std::time::Duration;
use std::time::Instant;

use std::io::{Read, Write};
use std::net::TcpStream;

use crate::Socket;

static PORT: u32 = 8080;

pub struct Message {
    pub value: String,
    pub id: String,
}

pub fn login(socket: &mut Socket, name: String, password: String) -> Result<Vec<u8>, String> {
    let mut stream = &socket.stream;

    stream.write(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
    stream.write(&string_to_buffer(name)).unwrap();
    stream.write(&string_to_buffer(password)).unwrap();

    let mut buffer = [0 as u8; 256]; // using 256 KB buffer
    match stream.read_exact(&mut buffer) {
        Ok(_) => {
            let data = buffer[1..buffer[0] as usize + 1].to_vec();
            if data == [0 as u8; 8] {
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

pub fn send_message(socket: &mut Socket, message: String, id: String, token: &Vec<u8>) -> Result<(), String> {
    let mut stream = &socket.stream;
    let mut buffer = [0u8; 256];

    stream.write(&[0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
    stream.write(&vec_to_buffer(token)).unwrap();

    stream.write(&string_to_buffer(message)).unwrap();
    stream.write(&string_to_buffer(id)).unwrap();

    Ok(())
}

pub fn recieve_messages(socket: &mut Socket, token: &Vec<u8>) -> Result<Vec<Message>, ()> {
    let mut stream = &socket.stream;
    let mut messages: Vec<Message> = Vec::new();
    let mut buffer = [0u8; 256];

    stream.write(&[0x04 ,0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
    stream.write(&vec_to_buffer(token)).unwrap();

    loop {
        // recieves message
        stream.read_exact(&mut buffer).unwrap();
        let buf = &buffer[1..buffer[0] as usize + 1];

        // breaks loop if final message was sent 
        if buf == [2, 2, 2, 2, 2, 2, 2, 2] {break}

        let message = match std::str::from_utf8(buf) {
            Ok(message) => message,
            Err(_) => "[INVALID]",
        };
        //recieves sender id
        let mut buffer = [0u8; 256];
        stream.read_exact(&mut buffer).unwrap();
        let buf = &buffer[1..buffer[0] as usize + 1];

        // breaks loop if final message was sent 
        if buf == [2, 2, 2, 2, 2, 2, 2, 2] {break}

        let sender_id = match std::str::from_utf8(buf) {
            Ok(message) => message,
            Err(_) => "[INVALID]",
        };
        messages.push(Message{value: message.to_string(), id: sender_id.to_string()});
    }

    Ok(messages)
}

fn string_to_buffer(string: String) -> [u8; 256] {
    let mut buffer: [u8; 256] = [0; 256];
    buffer[0] = string.len() as u8;

    for i in 0..string.len() {
        buffer[i + 1] = string.as_bytes()[i];
    }

    buffer
}

fn vec_to_buffer(vec: &Vec<u8>) -> [u8; 256] {
    let mut buffer: [u8; 256] = [0; 256];
    buffer[0] = vec.len() as u8;

    for i in 0..vec.len() {
        buffer[i + 1] = vec[i];
    }

    buffer
}
