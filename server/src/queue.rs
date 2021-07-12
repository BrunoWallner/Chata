use crate::*;
use std::sync::mpsc;

use std::time::Duration;

#[derive(Clone, Debug)]
pub enum Event {
    QueuePullRequest(),
    DeleteUser(String),
    CreateUser([String; 3]),
    SendMessage([String; 3]),

    RequestAccountData(mpsc::Sender<Vec<Account>>),
    SaveAccountData(),
}

#[derive(Clone, Debug)]
pub struct Queue {
    events: Vec<Event>,
}

pub fn init(
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    poll_time: u64,
    event_cooldown: u64,
    account_save_cooldown: u64,
) {
    let (queue_sender, queue_receiver) = mpsc::channel();

    // thread that listenes on incoming eventrequests
    thread::spawn(move || {
    	let mut queue: Queue = Queue { events: Vec::new() };
        loop {
                let event: Event = receiver.recv().unwrap();
                match event {
                    Event::QueuePullRequest() => {
                        queue_sender.send(queue.clone()).unwrap();
                        queue.events.clear();
                    }
                    _ => {
                        queue.events.push(event.clone());
                    }
                }
            }
    });
    // thread that sends Event to save Accountdata to disk
    let cloned_sender = sender.clone();
    thread::spawn(move || {
        loop {
            cloned_sender.send(Event::SaveAccountData()).unwrap();
            thread::sleep(Duration::from_millis(account_save_cooldown));
        }
    });
    // thread that pulls all events every n milliseconds and executes them in queue
    let cloned_sender = sender.clone();
    thread::spawn(move || {
        execute(
            queue_receiver,
            cloned_sender,
            poll_time,
            event_cooldown,
        );
    });
}

fn execute(
    receiver: mpsc::Receiver<Queue>,
    sender: mpsc::Sender<Event>,
    poll_time: u64,
    event_cooldown: u64,
) {
    let mut accounts: Vec<Account> = get_account_data();
    loop {
        //let mut accounts = accounts.clone();
        sender.send(Event::QueuePullRequest()).unwrap();
        let queue: Queue = receiver.recv().unwrap();

        for event in queue.events.iter() {
            match event {
                Event::SendMessage(value) => {
                    match write_message(
                        value[0].to_string(),
                        value[1].to_string(),
                        value[2].to_string(),
                    ) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("failed to execute message send event\n[{}]", e);
                        }
                    };
                }
                Event::DeleteUser(user) => {
                    match delete_user(&mut accounts, user.to_string()) {
                        Ok(_) => (),
                        Err(e) => println!("failed to delete user\n[{}]", e),
                    };
                }
                Event::CreateUser(data) => {
                    let name = data[0].clone();
                    let passwd = data[1].clone();
                    let id = data[2].clone();
                    match create_user(&mut accounts, name, passwd, id) {
                        Ok(_) => (),
                        Err(e) => println!("failed to create user\n[{}]", e),
                    }
                }
                Event::RequestAccountData(sender) => {
                    let accounts_clone = accounts.clone();
                    sender.send(accounts_clone).unwrap();
                },
                Event::SaveAccountData() => {
                    save_account_data(accounts.clone()).unwrap();
                }
                _ => (),
            }
            thread::sleep(std::time::Duration::from_millis(event_cooldown));
        }
        thread::sleep(std::time::Duration::from_millis(poll_time));
    }
}

fn save_account_data(accounts: Vec<Account>) -> Result<(), String> {
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

fn delete_user(accounts: &mut Vec<Account>, user: String) -> Result<(), String> {
    for i in 0..accounts.len() {
        if accounts[i].id == user {
            accounts.remove(i);
            break;
        }
    }
    Ok(())
}

fn create_user(accounts: &mut Vec<Account>, name: String, password: String, id: String) -> Result<(), String> {
    let account = functions::create_account(name, password, id);

    if !does_user_already_exist(&accounts, &account) {
        accounts.push(account);
    } else {
        return Err("user already exist".to_string());
    }

    Ok(())
}
fn does_user_already_exist(
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

pub fn write_message(string_id: String, message: String, sender_id: String) -> Result<(), String> {
    // imports userdata from ./userdata/[USERID]
    let mut file = match File::open("userdata/".to_string() + &string_id.clone()) {
        Ok(file) => file,
        Err(_) => {
            File::create("userdata/".to_string() + &string_id.clone()).unwrap();
            File::open("userdata/".to_string() + &string_id.clone()).unwrap()
        }
    };
    let mut encoded: Vec<u8> = Vec::new();
    file.read_to_end(&mut encoded).unwrap();

    let mut userdata: UserData = match bincode::deserialize(&encoded) {
        Ok(userdata) => userdata,
        Err(_) => UserData {
            messages: Vec::new(),
        },
    };

    // pushes final message
    userdata.messages.push(Message {
        value: message,
        id: sender_id,
    });

    // saves userdata back to file
    let serialized: Vec<u8> = match bincode::serialize(&userdata.clone()) {
        Ok(ser) => ser,
        Err(_) => return Err("failed to deserialize userdata".to_string()),
    };

    let mut file = match File::create("userdata/".to_string() + &string_id.clone()) {
        Ok(file) => file,
        Err(_) => {
            return Err(format!(
                "failed to create file: ./userdata/{}",
                string_id.clone()
            ))
        }
    };
    match file.write_all(&serialized) {
        Ok(_) => (),
        Err(_) => {
            return Err(format!(
                "failed to write userdata to: ./userdata/{}",
                string_id.clone()
            ))
        }
    };

    Ok(())
}

fn get_account_data() -> Vec<Account> {
    let mut file = match File::open("data.bin") {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let mut encoded: Vec<u8> = Vec::new();
    match file.read_to_end(&mut encoded) {
        Ok(_) => (),
        Err(_) => return Vec::new(),
    };

    match bincode::deserialize(&encoded) {
        Ok(a) => a,
        Err(_) => Vec::new(),
    }
}
