#![allow(unused_imports)]
use iced::{
    button, scrollable, slider, text_input, Align, Button, Checkbox, Column, Container, Element,
    Length, ProgressBar, Radio, Row, Rule, Sandbox, Scrollable, Settings, Slider, Space, Text,
    TextInput,
};

use std::str::from_utf8;
use std::net::TcpStream;

mod backend;
mod style;

static IP: &'static str = "localhost:8080";

pub fn main() -> iced::Result {
    Chat::run(Settings {
        window: iced::window::Settings {
            min_size: Some((200, 300)),
            transparent: true,
            ..iced::window::Settings::default()
        },
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Default)]
struct Chat {
    socket: Socket,

    theme: style::Theme,
    status: Status,

    name_input_state: text_input::State,
    passwd_input_state: text_input::State,
    token: Vec<u8>,
    input_name: String,
    input_passwd: String,
    login_button: button::State,

    chat_input_state: text_input::State,
    chat_input: String,
    chat_input_id_state: text_input::State,
    chat_input_id: String,
    chat_send_button: button::State,

}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(style::Theme),
    NameChanged(String),
    PasswdChanged(String),
    LoginButtonPressed,

    ChatMessageChanged(String),
    ChatMessageIDChanged(String),
    ChatMessageSent,
}

pub struct Socket {
    stream: TcpStream,
    id: String
}

enum Status {
    Login,
    InvalidLogin,
    Chat,
    Signup,
}
impl Default for Status {
    fn default() -> Self {Status::Login}
}
impl Default for Socket {
    fn default() -> Self { Socket {
        stream: match TcpStream::connect(IP) {
            Ok(stream) => stream,
            Err(_) => {
                println!("{} refused the connection", IP);
                std::process::exit(1);
            },
        },
        id: String::new(),
    } }
}

impl Sandbox for Chat {
    type Message = Message;

    fn new() -> Self {
        Chat {
            ..Chat::default()
        }
    }

    fn title(&self) -> String {
        String::from("Chata")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::NameChanged(value) => self.input_name = value,
            Message::PasswdChanged(value) => self.input_passwd = value,
            Message::LoginButtonPressed => {
                self.token = match backend::login(
                    &mut self.socket,
                    self.input_name.clone(),
                    self.input_passwd.clone(),
                ) {
                    Ok(token) => {
                        self.status = Status::Chat;
                        token.to_vec()
                    },
                    Err(_) => {
                        self.status = Status::InvalidLogin;
                        vec![0]
                    }
                };
            }
            Message::ChatMessageChanged(value) => self.chat_input = value,
            Message::ChatMessageIDChanged(value) => self.chat_input_id = value,
            Message::ChatMessageSent => {
                let message = self.chat_input.clone();
                match backend::send_message(
                    &mut self.socket,
                    self.chat_input.clone(),
                    self.chat_input_id.clone(),
                    &self.token
                ) {
                    Ok(_) => (),
                    Err(_) => (),
                }
                self.chat_input = "".to_string();
            }
            _ => ()
        }
    }

    fn view(&mut self) -> Element<Message> {
        let theme_choosing = style::Theme::ALL.iter().fold(
            Row::new().spacing(20),
            |column, theme| {
                column.push(
                    Radio::new(
                        *theme,
                        &format!("{:?}", theme),
                        Some(self.theme),
                        Message::ThemeChanged,
                    )
                    .style(self.theme)
                )
                .align_items(Align::End)
            }
        );

        let content = match self.status {
            Status::Login | Status::InvalidLogin => {
                let name_input = TextInput::new(
                    &mut self.name_input_state,
                    "Name...",
                    &self.input_name,
                    Message::NameChanged,
                )
                .padding(10)
                .size(20)
                .style(self.theme);
        
                let passwd_input = TextInput::new(
                    &mut self.passwd_input_state,
                    "Password...",
                    &self.input_passwd,
                    Message::PasswdChanged,
                )
                .padding(10)
                .size(20)
                .style(self.theme);
        
                let login_button = Button::new(&mut self.login_button, Text::new("Submit"))
                    .padding(10)
                    .on_press(Message::LoginButtonPressed)
                    .style(self.theme);
        
                let text_field = Text::new(match self.status {
                    Status::Login => "",
                    Status::InvalidLogin => "invalid login",
                    _ => "bad"
                });

                Column::new()
                    .align_items(Align::Center)
                    .spacing(10)
                    .padding(10)
                    .max_width(600)
                    .push(theme_choosing)
                    .padding(10)
                    .push(Rule::horizontal(40).style(self.theme))
                    .push(name_input)
                    .push(passwd_input)
                    .push(login_button)
                    .push(text_field)
            },

            Status::Chat => {
                let chat_input_id = TextInput::new(
                    &mut self.chat_input_id_state,
                    "ID...",
                    &self.chat_input_id,
                    Message::ChatMessageIDChanged,
                )
                .padding(10)
                .size(20)
                .style(self.theme);

                let chat_input = TextInput::new(
                    &mut self.chat_input_state,
                    "Message...",
                    &self.chat_input,
                    Message::ChatMessageChanged,
                )
                .padding(10)
                .size(20)
                .style(self.theme);

                let chat_send_button = Button::new(&mut self.chat_send_button, Text::new("Send"))
                    .padding(10)
                    .style(self.theme)
                    .on_press(Message::ChatMessageSent);

                Column::new()
                    .align_items(Align::Center)
                    .max_width(600)
                    .spacing(10)
                    .padding(10)
                    .push(theme_choosing)
                    .push(Rule::horizontal(40).style(self.theme))
                    .push(Column::new()
                        .spacing(10)
                        .padding(10)
                        .align_items(Align::Center)
                        .push(chat_input_id)
                        .push(chat_input)
                        .push(chat_send_button)
                    )
            }
            _ => Column::new(),
        };
            
        
 

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(self.theme)
            .into()
    }
}
