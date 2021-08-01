use crate::*;

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
