use cli_gui::*;
use std::time::Instant;
use std::time::Duration;
use std::thread;

use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

static PORT: &str = "3333";

fn main() {
    let mut terminal = Terminal::init(Size::new(100, 50));
    
    let mut main_window = Window::new(Position::new(0, 0), Size::new(30, 20));

    let mut login_window = Window::new(Position::new(0, 0), Size::new(50, 26));
    login_window.set_border_color(Color::red());

    let mut chat_window = Window::new(Position::new(0, 0), Size::new(50, 26));
    let mut chat_input_window = Window::new(Position::new(0, 0), Size::new(50, 3));

    let mut token: [u8; 8192] = [0; 8192];
    let mut status = String::new();

    let mut window: u32 = 0;

    let mut input: Vec<char> = Vec::new();
    let mut messages: Vec<String> = Vec::new();
    loop {
        terminal.clear();
        terminal.clear_windows();
        terminal.update();
        
        if window == 0 {
            main_window.clear();
            main_window.center(&terminal);

            let menu: String = "0 => leave\n1 => login\n2 => signup\n".to_string();

            main_window.write(Position::new(9, 2), menu, Color::green());
            main_window.pos.x += 1;

            match terminal.key_event.code {
                KeyCode::Char('0') => break,
                KeyCode::Char('1') => window = 1,
                KeyCode::Char('2') => window = 2,
                _ => (),
            }

            main_window.decorate();
            terminal.write_window(&main_window);

            terminal.render_accurate();
        }

        if window == 1 {
            login_window.center(&terminal);

            login_window.clear();
            login_window.decorate();

            login_window.write(Position::new(17, 20), status.clone(), Color::green());

            terminal.write_window(&login_window);
            terminal.render();

            let ip = terminal.read_line(Position::new(login_window.pos.x + 5, login_window.pos.y + 3), "iP: ", Color::red(), true);
            let name = terminal.read_line(Position::new(login_window.pos.x + 5, login_window.pos.y + 5), "Name: ", Color::red(), true);
            let password = terminal.read_line(Position::new(login_window.pos.x + 5, login_window.pos.y + 7), "Password: ", Color::red(), true);

            if name.len() >= 1 {
                if name == "quit".to_string() {break};
                token = match check_login(ip, name, password) {
                    Ok(token) => {
                        //logged_in = true;
                        status = "logged in".to_string();
                        window = 2;
                        token
                    }
                    Err(_) => {
                        status = "invalid login".to_string();
                        token
                    },
                } 
            }
        } 

        if window == 2 {
            chat_window.center(&terminal);
            chat_window.clear();
            chat_window.decorate();

            for i in 0..messages.len() {
                chat_window.write(Position::new(1, i as i32 + 1), messages[i].clone(), Color::white());
            }

            chat_input_window.clear();
            chat_input_window.decorate();
            chat_input_window.set_position(Position::new(
                chat_window.pos.x,
                chat_window.pos.y + chat_window.size.y
            ));

            let input_string: String = input.iter().cloned().collect();
            chat_input_window.write(Position::new(1, 1), input_string.clone(), Color::white());

            terminal.write_window(&chat_window);
            terminal.write_window(&chat_input_window);

            terminal.render_accurate();

            match terminal.key_event.code {
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Enter => {
                    messages.push(input_string.clone());
                    input.clear();
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                _ => ()
            }

            match input_string.clone().as_str() {
                "exit" => break,
                _ => ()
            }
        }
        //thread::sleep(Duration::from_millis(16));
    }
    terminal.quit();
}

fn check_login(ip: String, name: String, password: String) -> Result<[u8; 8192], String> {
    match TcpStream::connect(ip + ":" + PORT) {
        Ok(mut stream) => {

            let msg = create_login(name, password);

            stream.write(&msg).unwrap();

            let mut data = [0 as u8; 8192]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if data == [0 as u8; 8192] {
                        return Err("invalid login".to_string());
                    } else {
                        return Ok(data);
                    }
                },
                Err(e) => {
                    return Err("Failed to recieve data".to_string());
                }
            }
        },
        Err(e) => {
            return Err("Failed to connect".to_string());
        }
    }
    println!("Terminated.");
    return Err("connection terminated".to_string());
}

fn write_message(ip: String, message: String) -> Result<(), String> {
    match TcpStream::connect(ip + ":" + PORT) {
        Ok(mut stream) => {
            let msg = create_message(message.clone());
        
            stream.write(&msg).unwrap();

            let mut data = [0 as u8; 8192];
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if data == [0 as u8; 8192] {
                        return Err("Invalid Token".to_string());
                    }
                    else {
                        return Ok(());
                    }
                }
                Err(_) => {
                    return Err("Failed to recieve data".to_string());
                }
            }
        },
        Err(_) => {
            return Err("connection terminated".to_string());
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

fn create_message(message: String) -> [u8; 8192] {
    let mut buffer = [0x00 as u8; 8192];
    buffer[0] = 0x05;
    for i in 0..message.len() {
        buffer[i + 1] = message.as_bytes()[i];
    }
    buffer
}