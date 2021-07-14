use chrono::{DateTime, Utc};
use colored::*;

pub enum State {
    Information(String),
    Error(String),
    CriticalError(String),
}

pub fn print(state: State) {
    let now: DateTime<Utc> = Utc::now();

    match state {
        State::Information(string) => {
            println!("[{}] {}", now.to_string().yellow(), string.cyan());
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
