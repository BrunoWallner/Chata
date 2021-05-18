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

pub fn check_login(accounts: &Vec<Account>, name: &[u8], passwd: &[u8]) -> Result<usize, ()> {
    for i in 0..accounts.len() {
        let mut hashed_name = Sha512::new();
        hashed_name.update(&name);

        let mut hashed_password = Sha512::new();
        hashed_password.update(&passwd);

        if *accounts[i].name == *hashed_name.finalize()
            && *accounts[i].password == *hashed_password.finalize()
        {
            return Ok(i);
        }
    }
    return Err(());
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

pub fn check_token(accounts: &Vec<Account>, token: &[u8]) -> Result<usize, ()> {
    for i in 0..accounts.len() {
        if accounts[i].token == token {
            return Ok(i);
        }
    }
    return Err(());
}

#[allow(dead_code)]
pub fn create_account(name: String, password: String, id: String) -> Account {
    // creates data for Account
    let mut hashed_name = Sha512::new();
    hashed_name.update(name.as_bytes());

    let mut hashed_password = Sha512::new();
    hashed_password.update(password.as_bytes());

    let mut token: Vec<u8> = Vec::new();
    for _ in 0..255 {
        token.push(thread_rng().gen_range(0..255))
    }

    // creates data and file in ./userdata
    let mut file = match File::create("userdata/".to_string() + &id.to_string()) {
        Ok(file) => file,
        Err(_) => panic!("failed to create file in ./userdata for user: {}", id),
    };
    let userdata = UserData {
        messages: Vec::new(),
    };
    let serialized = bincode::serialize(&userdata).unwrap();
    match file.write_all(&serialized) {
        Ok(..) => (),
        Err(e) => println!("{}", e),
    };

    Account {
        name: hashed_name.finalize().to_vec(),
        password: hashed_password.finalize().to_vec(),
        token: token,
        id: id,
    }
}

pub fn search_by_id(accounts: &Vec<Account>, id: String) -> Result<usize, ()> {
    for i in 0..accounts.len() {
        if id == accounts[i].id {
            return Ok(i);
        }
    }
    Err(())
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
