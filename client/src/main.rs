use cli_gui::*;
use std::time::Instant;
use std::time::Duration;
use std::thread;

use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

fn main() {
    let mut terminal = Terminal::init(Size::new(100, 50));
    
    let mut main_window = Window::new(Position::new(2, 2), Size::new(30, 20));

    let mut login_window = Window::new(Position::new(10, 12), Size::new(50, 26));
    login_window.set_border_color(Color::red());

    let mut chat_window = Window::new(Position::new(10, 12), Size::new(50, 26));

    let mut token: [u8; 8192] = [0; 8192];
    let mut status = String::new();

    let mut window: u32 = 0;

    loop {
        if window == 0 {
            terminal.clear();
            terminal.clear_windows();
            terminal.update();

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
            terminal.clear();
            terminal.clear_windows();
            terminal.update();
            
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

        }
    }
    terminal.quit();
}

fn check_login(ip: String, name: String, password: String) -> Result<[u8; 8192], String> {
    match TcpStream::connect(ip + ":3333") {
        Ok(mut stream) => {

            let mut msg = create_login(name, password);

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