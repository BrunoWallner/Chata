use crate::*;
use std::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub value: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserData {
    pub id: String,
    pub messages: Vec<Message>,
    pub changed: bool,
}

pub fn save_auth_data(accounts: Vec<Account>) -> Result<(), String> {
    // saves accounts back to data.bin
    let mut file = match File::create("data.bin") {
        Ok(file) => file,
        Err(_) => return Err("failed to create file".to_string()),
    };

    let serialized = match bincode::serialize(&accounts) {
        Ok(ser) => ser,
        Err(_) => return Err("failed to serialize account data".to_string()),
    };
    file.write_all(&serialized).unwrap();

    Ok(())
}

pub fn save_user_data(users: &mut Vec<UserData>) -> Result<(), String> {
    for user in users.iter_mut() {
        if user.changed {
            user.changed = false;

            match File::create("./userdata/".to_string() + &String::from(user.id.clone())) {
                Ok(mut file) => {
                    match bincode::serialize(&user) {
                        Ok(serialized) => {
                            match file.write_all(&serialized) {
                                Ok(..) => print(State::Information(format!("saved userdata for user: {}", user.id))),
                                Err(e) => print(State::CriticalError(format!("failed to save userdata in ./userdata for user: {}", user.id))),
                            };
                        },
                        Err(e) => print(State::CriticalError(format!("failed to deserialize userdata for user: {} {}", user.id, e))),
                    };
                },
                Err(e) => print(State::CriticalError(format!("failed to open file in ./userdata for user: {} {}", user.id, e))),
            };
        }
    }
    Ok(())
}

pub fn delete_user(accounts: &mut Vec<Account>, user: String) -> Result<(), String> {
    for i in 0..accounts.len() {
        if accounts[i].id == user {
            match fs::remove_file("./userdata".to_string() + &accounts[i].id) {
                // it is completely irrelevant
                Ok(_) => (),
                Err(_) => (),
            };
            accounts.remove(i);
            return Ok(())
        }
    }
    Err(String::from("could not find user"))
}

pub fn create_user(accounts: &mut Vec<Account>, name: String, password: String, id: String) -> Result<(), String> {
    if name.contains("__") || id.contains("::") {
        return Err(String::from("invalid username"));
    }
    let account = match functions::create_account(name, password, id) {
        Ok(acc) => acc,
        Err(e) => return Err(e),
    };

    if !does_user_already_exist(&accounts, &account) {
        accounts.push(account);
    } else {
        return Err("user already exist".to_string());
    }

    Ok(())
}
pub fn does_user_already_exist(
    accounts: &Vec<Account>,
    account: &Account,
) -> bool {
    for a in accounts.iter() {
        if a.name == account.name && a.id == account.id {
            return true;
        }
    }
    false
}

pub fn write_message(string_id: String, message: String, sender_id: String, userdata: &mut Vec<UserData>) -> Result<(), String> {

    for i in 0..userdata.len() {
        if userdata[i].id == string_id {
            userdata[i].changed = true;
            userdata[i].messages.push(Message {
                value: message,
                id: sender_id,
            });
            return Ok(());
        }
    }
    Err(String::from("could not find user"))
}

pub fn get_all() -> Vec<UserData> {
    let mut userdata: Vec<UserData> = Vec::new();
    let mut users: Vec<String> = Vec::new();

    let path = fs::read_dir("./userdata").unwrap();
    for data in path {
        let userpath = data.unwrap().path().display().to_string();
        users.push(userpath.clone());
    }

    for user in users.iter() {
        let mut file = match File::open(user.clone()) {
            Ok(file) => file,
            Err(_) => {
                File::create(user.clone()).unwrap();
                File::open(user.clone()).unwrap()
            }
        };
        let mut encoded: Vec<u8> = Vec::new();
        file.read_to_end(&mut encoded).unwrap();

        let userdata_part: UserData = match bincode::deserialize(&encoded) {
            Ok(userdata) => userdata,
            Err(_) => UserData {
                id: String::from("[ERROR]"),
                messages: Vec::new(),
                changed: true,
            },
        };
        userdata.push(userdata_part);
    }

    userdata
}

pub fn request_single(id: String, sender: mpsc::Sender<queue::Event>) -> Result<UserData, ()> {
    let (tx, rx) = mpsc::channel();

    sender.send(queue::Event::RequestUserData( (tx, id) )).unwrap();
    let user_data = match rx.recv() {
        Ok(user_data) => match user_data {
            Ok(user_data) => user_data,
            Err(_) => return Err(()),
        },
        Err(_) => return Err(()),
    };
    Ok(user_data)
}








































