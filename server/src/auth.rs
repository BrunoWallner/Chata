use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use std::fs::File;
use std::io::{Read, Write};

use crate::Account;

pub enum Event {
    RequestAccountData(),
    ReceiveAccountData(Vec<Account>),
    ReadAccounts(),
    TokenValidation(bool),
}

pub fn init(
    tx: mpsc::Sender<(mpsc::Sender<Event>, Event)>,
    rx: mpsc::Receiver<(mpsc::Sender<Event>, Event)>,
) {
    thread::spawn(move || {
        execute(rx);
    });
    thread::spawn(move || {
        update(tx);
    });
}

fn execute(rx: mpsc::Receiver<(mpsc::Sender<Event>, Event)>) {
    let mut accounts: Vec<Account> = get_accounts();
    loop {
        let received: (mpsc::Sender<Event>, Event) = rx.recv().unwrap();
        match received.1 {
            Event::RequestAccountData() => {received.0.send(Event::ReceiveAccountData(accounts.clone())).unwrap()},
            Event::ReadAccounts() => {
                accounts = get_accounts();
                //println!("refreshed accounts");
            },
            _ => (),
        };
    }
}

fn update(tx: mpsc::Sender<(mpsc::Sender<Event>, Event)>) {
    loop {
        let (tx2, _) = mpsc::channel();
        tx.send((tx2, Event::ReadAccounts()));

        thread::sleep(Duration::from_secs(5000000));
    }
}

