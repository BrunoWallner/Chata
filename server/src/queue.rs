use crate::*;
use std::sync::mpsc;

#[derive(Clone, Debug)]
pub enum Event {
    QueuePullRequest(),
    DeleteUser(String),
    CreateUser([String; 3]),
    MessageSent([String; 3]),
}

#[derive(Clone, Debug)]
pub struct Queue {
    events: Vec<Event>,
}

pub fn init(
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    queue_poll_time: u64,
    event_cooldown: u64,
) {
    let (queue_sender, queue_receiver) = mpsc::channel();

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
    let cloned_sender = sender.clone();
    thread::spawn(move || {
        execute(queue_receiver, cloned_sender, queue_poll_time, event_cooldown);
    });
}

fn execute(
    receiver: mpsc::Receiver<Queue>,
    sender: mpsc::Sender<Event>,
    queue_poll_time: u64,
    event_cooldown: u64,
) {
    loop {
        sender.send(Event::QueuePullRequest()).unwrap();
        let queue: Queue = receiver.recv().unwrap();
        if queue.events.len() > 0 {
            functions::print_header("async event execution".to_string(), 40)
        }
        let mut messages: Vec<String> = Vec::new();
        let now = std::time::Instant::now();
        for event in queue.events.iter() {
            match event {
                Event::MessageSent(value) => {
                    write_message(
                        value[0].to_string(),
                        value[1].to_string(),
                        value[2].to_string(),
                    );
                    messages.push("executed message send event".to_string());
                }
                Event::DeleteUser(user) => {
                    messages.push(format!("deleting user: {}", user));
                    match delete_user(user.to_string()) {
                        Ok(_) => messages.push("operation successfull".to_string()),
                        Err(_) => messages.push("could not find user".to_string()),
                    };
                }
                Event::CreateUser(data) => {
                    messages.push(format!("creating user: {}", data[2]));
                    let name = data[0].clone();
                    let passwd = data[1].clone();
                    let id = data[2].clone();
                    match create_user(name, passwd, id) {
                        Ok(_) => messages.push("operation successfull".to_string()),
                        Err(_) => messages.push("could not create user".to_string()),
                    }
                }
                _ => (),
            }
            thread::sleep(std::time::Duration::from_millis(event_cooldown));
        }
        messages.push(format!("execution time: {} ms", now.elapsed().as_millis()));
        if queue.events.len() > 0 {
            print_body(messages, 40)
        }
        thread::sleep(std::time::Duration::from_millis(queue_poll_time));
    }
}

fn delete_user(user: String) -> Result<(), ()> {
    // deletes user from ./userdata/
    match std::fs::remove_file("userdata/".to_string() + &user) {
        Ok(_) => (),
        Err(_) => (),
    };

    // deletes user from data.bin
    let mut file = File::open("data.bin").unwrap();
    let mut encoded: Vec<u8> = Vec::new();
    file.read_to_end(&mut encoded).unwrap();

    let mut accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();

    for i in 0..accounts.len() {
        if accounts[i].id == user {
            accounts.remove(i);
            break;
        }
    }

    let mut file = File::create("data.bin").unwrap();
    let serialized = bincode::serialize(&accounts).unwrap();
    file.write_all(&serialized).unwrap();

    Ok(())
}

fn create_user(name: String, password: String, id: String) -> Result<(), ()> {
    // reads user
    let mut file = File::open("data.bin").unwrap();
    let mut encoded: Vec<u8> = Vec::new();
    file.read_to_end(&mut encoded).unwrap();

    let mut accounts: Vec<Account> = bincode::deserialize(&encoded).unwrap();

    accounts.push(functions::create_account(name, password, id));

    // saves accounts back to data.bin
    let mut file = File::create("data.bin").unwrap();
    let serialized = bincode::serialize(&accounts).unwrap();
    file.write_all(&serialized).unwrap();

    Ok(())
}

pub fn write_message(string_id: String, message: String, sender_id: String) {
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
    let serialized: Vec<u8> = bincode::serialize(&userdata.clone()).unwrap();

    let mut file = File::create("userdata/".to_string() + &string_id.clone()).unwrap();
    file.write_all(&serialized).unwrap();
}
