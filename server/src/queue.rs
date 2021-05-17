use crate::*;
use std::sync::mpsc;

use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Null(),
    QueuePullRequest(),
    MessageSent([&'static str; 3])
}

#[derive(Clone, Copy, Debug)]
pub struct Queue {
    events: [Event; 5],
    events_len: usize,
}
impl Queue {
    pub fn empty() -> Self {
        Queue {
            events: [Event::Null(); 5],
            events_len: 0,
        }
    }
}

pub fn init(receiver: mpsc::Receiver<Event>, sender: mpsc::Sender<Event>) {
    let mut queue: Queue = Queue::empty();
    let (queue_sender, queue_receiver) = mpsc::channel();

    thread::spawn(move || {
        loop {
            let event: Event = receiver.recv().unwrap();
            match event {
                Event::QueuePullRequest() => {
                    println!("QueuePullRequest");
                    queue_sender.send(queue).unwrap();
                },
                _ => {
                    queue.events[queue.events_len] = event;
                    queue.events_len += 1;
                }
            }
        }
    });
    let cloned_sender = sender.clone();
    thread::spawn(move || {
        execute(queue_receiver, cloned_sender);
    });
}

fn execute(receiver: mpsc::Receiver<Queue>, sender: mpsc::Sender<Event>) {
    loop {
        sender.send(Event::QueuePullRequest()).unwrap();
        let queue: Queue = receiver.recv().unwrap();
        println!("{:#?}", queue);
        thread::sleep(std::time::Duration::from_millis(5000));
    }
}


