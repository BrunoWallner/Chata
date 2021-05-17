use std::thread;
use std::time::Duration;
use std::sync::mpsc;

fn main() {
    let mut value: u16 = 0;

    let b = thread::spawn(move || -> u16 {
        for _ in 0..10 {
            value += 1;
            println!("debug: {}", value);
            thread::sleep(Duration::from_millis(10));
        }
        value
    });
    value = b.join().unwrap();
    loop {
        println!("{}", value);
        thread::sleep(Duration::from_millis(1000));
    }
}

