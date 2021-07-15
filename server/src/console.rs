use chrono::{DateTime, Utc};
use colored::*;

pub enum State {
    Information(String),
    ImportantInformation(String),
    Error(String),
    CriticalError(String),
}

pub fn print(state: State) {
    let now: DateTime<Utc> = Utc::now();

    match state {
        State::Information(string) => {
            println!("[{}] {}", now.to_string().yellow(), string.cyan());
        },
        State::ImportantInformation(string) => {
            println!("[{}] {}", now.to_string().yellow(), string.green());
        },
        State::Error(string) => {
            println!("[{}] {}", now.to_string().yellow(), string.yellow());
        },
        State::CriticalError(string) => {
            println!("[{}] {}", now.to_string().yellow(), string.red());
        },
        _ => println!("error")
    }
}
