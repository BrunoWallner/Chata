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

pub fn save_user_data(users: &mut Vec<UserData>) -> Result<u64, String> {
    let mut total_save: u64 = 0;
    for mut user in users.iter_mut() {
        if user.changed {
            total_save += 1;
            user.changed = false;

            match File::create("./userdata/".to_string() + &String::from(user.id.clone())) {
                Ok(mut file) => {
                    match bincode::serialize(&user) {
                        Ok(serialized) => {
                            match file.write_all(&serialized) {
                                Ok(..) => (),
                                Err(e) => return Err(format!("failed to save userdata in ./userdata for user: {} [{}]", user.id, e)),
                            };
                        },
                        Err(e) => return Err(format!("failed to deserialize userdata for user: {} {}", user.id, e)),
                    };
                },
                Err(e) => return Err(format!("failed to open file in ./userdata for user: {} {}", user.id, e)),
            };
        }
    }
    Ok(total_save)
}

pub fn delete_user(accounts: &mut Vec<Account>, user: String) -> Result<(), String> {
    for i in 0..accounts.len() {
        if accounts[i].id == user {
            let id = accounts[i].id.clone();
            accounts.remove(i);
            match fs::remove_file("./userdata/".to_string() + &id.clone()) {
                // it is completely irrelevant
                Ok(_) => (),
                Err(e) => return Err(format!("could not delete userdata for user {} [{}]", id.clone(), e)),
            };
            return Ok(())
        }
    }
    Err(String::from("could not find user"))
}

pub fn create_user(accounts: &mut Vec<Account>, userdata: &mut Vec<UserData>, name: String, password: String, id: String) -> Result<Vec<u8>, String> {
    if name.contains("::") || id.contains("::") {
        return Err(String::from("invalid username"));
    }
    let account = match create_account(name, password, id.clone()) {
        Ok(acc) => acc,
        Err(e) => return Err(e),
    };

    userdata.push(UserData{
        id: id.clone(),
        messages: Vec::new(),
        changed: true,
    });

    if !does_user_already_exist(&accounts, &account) {
        accounts.push(account.clone());
    } else {
        return Err("user already exist".to_string());
    }

    Ok(account.token)
}
pub fn does_user_already_exist(
    accounts: &Vec<Account>,
    account: &Account,
) -> bool {
    for a in accounts.iter() {
        if a.id == account.id {
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

#[allow(dead_code)]
pub fn create_account(name: String, password: String, id: String) -> Result<Account, String> {
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
        Err(_) => {
            return Err(format!("failed to create file in ./userdata for user: {}", id));
        }
    };
    let userdata = user::UserData {
        id: id.clone(),
        messages: Vec::new(),
        changed: true,
    };
    let serialized = bincode::serialize(&userdata).unwrap();
    match file.write_all(&serialized) {
        Ok(..) => (),
        Err(e) => return Err(format!("failed to save userdate {}", e)),
    };

    Ok(
        Account {
            name: hashed_name.finalize().to_vec(),
            password: hashed_password.finalize().to_vec(),
            token: token,
            id: id.clone(),
        }
    )
}

pub fn get_accounts(tx: mpsc::Sender<queue::Event>) -> Vec<Account> {
    let (acc_sender, acc_receiver) = mpsc::channel();
    tx.send(queue::Event::RequestAccountData(acc_sender)).unwrap();
    acc_receiver.recv().unwrap()
}

pub fn search_by_id(accounts: &Vec<Account>, id: String) -> Result<usize, ()> {
    for i in 0..accounts.len() {
        if id == accounts[i].id {
            return Ok(i);
        }
    }
    Err(())
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
