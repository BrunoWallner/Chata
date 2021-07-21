use crate::*;
use std::sync::mpsc;

use std::time::Duration;

use crate::console::*;

#[derive(Clone, Debug)]
pub enum Event {
    DeleteUser(String),
    CreateUser([String; 3]),
    SendMessage([String; 3]),

    ServerShutdown(),

    RequestAccountData(mpsc::Sender<Vec<Account>>),
    SaveAuthData(),
    SaveUserData(),

    RequestUserData( (mpsc::Sender<Result<user::UserData, ()>>, String) ),
}

#[derive(Clone, Debug)]
pub struct Queue {
    events: Vec<Event>,
}

pub fn init(
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    auth_save_cooldown: u64,
    user_save_cooldown: u64,
) {
    // thread that sends Event to save Accountdata to disk
    let cloned_sender = sender.clone();
    thread::spawn(move || {
        loop {
            cloned_sender.send(Event::SaveAuthData()).unwrap();
            thread::sleep(Duration::from_millis(auth_save_cooldown));
        }
    });
    let cloned_sender = sender.clone();
    thread::spawn(move || {
        loop {
            cloned_sender.send(Event::SaveUserData()).unwrap();
            thread::sleep(Duration::from_millis(user_save_cooldown));
        }
    });

    // thread that receives and executes events
    thread::spawn(move || {
        execute(
            receiver,
        );
    });
}

fn execute(
    receiver: mpsc::Receiver<Event>,
) {
    let mut accounts: Vec<Account> = get_account_data();
    let mut userdata: Vec<user::UserData> = user::get_all();

    loop {
        let event: Event = receiver.recv().unwrap();

        match event {
            Event::SendMessage(value) => {
                match user::write_message(
                    value[0].to_string(),
                    value[1].to_string(),
                    value[2].to_string(),
                    &mut userdata,
                ) {
                    Ok(_) => print(State::Information(String::from("executed message send event"))),
                    Err(e) => {
                        print(State::Error(format!("failed to execute message send event [{}]", e)));
                    }
                };
            }
            Event::DeleteUser(user) => {
                match user::delete_user(&mut accounts, user.to_string()) {
                    Ok(_) => print(State::Information(String::from("executed user deletion event"))),
                    Err(e) => print(State::Error(format!("failed to delete user [{}]", e))),
                };
            }
            Event::CreateUser(data) => {
                let name = data[0].clone();
                let passwd = data[1].clone();
                let id = data[2].clone();
                match user::create_user(&mut accounts, &mut userdata, name, passwd, id) {
                    Ok(_) =>  print(State::Information(String::from("executed user creation event"))),
                    Err(e) => print(State::Error(format!("failed to create user [{}]", e))),
                }
            }
            Event::RequestAccountData(sender) => {
                let accounts_clone = accounts.clone();
                sender.send(accounts_clone).unwrap();
            },
            Event::SaveAuthData() => {
                match user::save_auth_data(accounts.clone()) {
                    Ok(_) => print(State::Information(format!("saved all hashed authentification information to data.bin"))),
                    Err(e) => print(State::CriticalError(format!("could not save auth data [{}]", e))),
                }
            },
            Event::SaveUserData() => {
                match user::save_user_data(&mut userdata) {
                    Ok(a) => print(State::Information(format!("saved userdata for {} user", a))),
                    Err(e) => print(State::CriticalError(format!("could not save user data [{}]", e))),
                };
            },
            Event::RequestUserData( (sender, id) ) => {
                let mut sent: bool = false;
                for i in 0..userdata.len() {
                    if &userdata[i].id == &id {
                        sender.send(Ok(userdata[i].clone())).ok();
                        sent = true;
                        break;
                    }
                }
                if !sent {sender.send(Err(())).unwrap()}

            },
            Event::ServerShutdown() => {
                print(State::Information(String::from("shutting down server...")));
                match user::save_auth_data(accounts.clone()) {
                    Ok(_) => print(State::Information(format!("saved all hashed authentification information to data.bin"))),
                    Err(e) => print(State::CriticalError(format!("could not save auth data [{}]", e))),
                }

                match user::save_user_data(&mut userdata.clone()) {
                    Ok(a) => print(State::Information(format!("saved userdata for {} user", a))),
                    Err(e) => print(State::CriticalError(format!("could not save user data [{}]", e))),
                };
                std::process::exit(0);
            },
        }
    }
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
