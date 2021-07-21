use chrono::{DateTime, Utc};
use colored::*;

use std::fs::File;
use std::io::Write;

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
    }
    let mut log = match File::open("./log.txt") {
        Ok(file) => file,
        Err(_) => File::create("./log.txt").unwrap()
    };

    writeln!(log, "{}\n", "moin");
}
