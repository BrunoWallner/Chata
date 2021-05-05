#![allow(unused_imports)]
use iced::{
    button, scrollable, slider, text_input, Align, Button, Checkbox, Column,
    Container, Element, Length, ProgressBar, Radio, Row, Rule, Sandbox,
    Scrollable, Settings, Slider, Space, Text, TextInput,
};

use std::str::from_utf8;

mod style;
mod backend;

pub fn main() -> iced::Result {
    Styling::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Default)]
struct Styling {
    theme: style::Theme,

    name_input_state: text_input::State,
    passwd_input_state: text_input::State,
    login_status: String,
    input_name: String,
    input_passwd: String,
    login_button: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(style::Theme),
    NameChanged(String),
    PasswdChanged(String),
    LoginButtonPressed,
}

impl Sandbox for Styling {
    type Message = Message;

    fn new() -> Self {
        Styling::default()
    }

    fn title(&self) -> String {
        String::from("Styling - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::NameChanged(value) => self.input_name = value,
            Message::PasswdChanged(value) => self.input_passwd = value,
            Message::LoginButtonPressed => {self.login_status = match backend::login("localhost".to_string(), self.input_name.clone(), self.input_passwd.clone()) {
                Ok(token) => from_utf8(&token).unwrap().to_string(),
                Err(e) => e,
            }},
        }
    }

    fn view(&mut self) -> Element<Message> {
        let choose_theme = style::Theme::ALL.iter().fold(
            Column::new().spacing(5).push(Text::new("Choose a theme:")),
            |column, theme| {
                column.push(
                    Radio::new(
                        *theme,
                        &format!("{:?}", theme),
                        Some(self.theme),
                        Message::ThemeChanged,
                    )
                    .style(self.theme),
                )
            },
        );

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

        let text_field = Text::new(self.login_status.clone());


        let content = Column::new()
            .spacing(10)
            .padding(10)
            .max_width(600)
            .push(choose_theme).padding(10)
            .push(Rule::horizontal(40).style(self.theme))
            .push(name_input)
            .push(passwd_input)
            .push(login_button)
            .push(text_field);


        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(self.theme)
            .into()
    }
}