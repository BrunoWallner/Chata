use crate::*;

pub fn check_token(accounts: &Vec<Account>, token: &[u8]) -> Result<usize, ()> {
    for i in 0..accounts.len() {
        if accounts[i].token == token {
            return Ok(i);
        }
    }
    return Err(());
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
