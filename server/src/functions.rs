use crate::*;

// prints all Users at server startup
pub fn print_users(accounts: &Vec<Account>, width: usize) {
    print_header("List of all accounts".to_string(), width);
    let mut users: Vec<String> = Vec::new();

    for user in accounts.iter() {
        users.push(user.id.clone());
    }

    print_body(users, width);
}

pub fn print_header(string: String, width: usize) {
    print!("┌");
    for _ in 0..width {
        print!("─");
    }
    print!("┐");
    print!("\n");

    print!("│");
    for _ in 0..width / 2 - string.len() / 2 {
        print!(" ");
    }
    print!("{}", string);
    for _ in 0..width - ((width / 2 - string.len() / 2) + string.len()) {
        print!(" ");
    }
    print!("│\n");

    print!("├");
    for _ in 0..width {
        print!("─");
    }
    print!("┤");
    print!("\n");
}
use std::iter::FromIterator;
pub fn print_body(strings: Vec<String>, width: usize) {
    for string in strings.iter() {
        //let row = (i + 1).to_string() + ": " + &accounts[i].id[0..(36-4)];
        let row: String;
        if string.chars().count() <= width {
            row = string.to_string();
        } else {
            let char_vec: Vec<char> = string.chars().collect();
            row = String::from_iter(&char_vec[0..width]);
        }
        print!("│{}", row);
        for _ in 0..width - row.chars().count() {
            print!(" ");
        }
        print!("│\n");
    }
    print!("└");
    for _ in 0..width {
        print!("─");
    }
    print!("┘");
    print!("\n");
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
