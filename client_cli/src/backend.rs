use std::str::from_utf8;
use std::net::TcpStream;

use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct Message {
    pub value: String,
    pub id: String,
}

pub fn connect(ip: String) -> Result<TcpStream, String> {
    match TcpStream::connect(ip.as_str()) {
        Ok(stream) => Ok(stream),
        Err(e) => Err(format!("{}", e)),
    }
}

pub fn login(mut stream: TcpStream, name: String, password: String) -> Result<(Vec<u8>, String), String> {
    stream.write(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
    stream.write(&string_to_buffer(name)).unwrap();
    stream.write(&string_to_buffer(password)).unwrap();

    let mut buffer = [0 as u8; 256]; // using 256 KB buffer
    match stream.read_exact(&mut buffer) {
        Ok(_) => {
            let token = buffer[1..buffer[0] as usize + 1].to_vec();
            if token == [0 as u8; 8] {
                return Err("invalid login".to_string());
            } else {
                match stream.read_exact(&mut buffer) {
                    Ok(_) => {
                        let id = &buffer[1..buffer[0] as usize + 1];
                        let id = String::from_utf8_lossy(id);
                        return Ok( (token, String::from(id)) );
                    },
                    Err(_) => {
                        return Err(String::from("Failed to receive data"));
                    }
                }
            }
        }
        Err(_) => {
            return Err("Failed to receive data".to_string());
        }
    }
}

pub fn signup(mut stream: TcpStream, name: String, password: String, id: String) -> Result<Vec<u8>, String> {
    let mut buffer = [0u8; 256];

    stream.write(&[0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
    stream.write(&string_to_buffer(name)).unwrap();
    stream.write(&string_to_buffer(password)).unwrap();
    stream.write(&string_to_buffer(id)).unwrap();

    match stream.read_exact(&mut buffer) {
        Ok(_) => {
            match buffer[1..buffer[0] as usize + 1] {
                [0x01, 0x01, 0x01, 0x01] => {
                    return Err(String::from("could not create user"));
                },
                _ => {
                    return(Ok(buffer[1..buffer[0] as usize + 1].to_vec()));
                }
            }
        },
        Err(_) => {
            return Err(String::from("could not receive data"));
        }
    }
}

pub fn send_message(mut stream: &mut TcpStream, message: String, id: String, token: &Vec<u8>) -> Result<(), String> {
    let mut buffer = [0u8; 256];

    if message.as_bytes().len() < 256 && id.as_bytes().len() < 256 {
        stream.write(&[0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
        stream.write(&vec_to_buffer(token)).unwrap();

        stream.write(&string_to_buffer(message)).unwrap();
        stream.write(&string_to_buffer(id)).unwrap();

        Ok(())
    } else {
        Err(String::from("message cant be longer than 255 bytes"))
    }
}

pub fn receive_messages(mut stream: &mut TcpStream, token: &Vec<u8>) -> Result<Vec<Message>, ()> {
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

use itertools::*;
pub fn convert_messages(messages: Vec<Message>) -> Vec<Vec<String>> {
    let mut converted_messages: Vec<Vec<String>> = Vec::new();

    let mut unique_ids: Vec<&String> = Vec::new();
    let mut ids: Vec<String> = Vec::new();

    for message in messages.iter() {
        ids.push(message.id.clone());
    }
    unique_ids = ids.iter().unique().collect::<Vec<_>>();

    for id in 0..unique_ids.len() {
        let mut vec: Vec<String> = Vec::new();
        vec.push(unique_ids[id].clone());
        converted_messages.push(vec);
        for message in messages.iter() {
            if message.id.clone() == unique_ids[id].clone() {
                converted_messages[id].push(message.value.clone());
            }
        }
    }
    converted_messages
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
