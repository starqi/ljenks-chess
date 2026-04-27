// Native CLI platform implementations
use rand::{thread_rng, Rng};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn log(s: &str) {
    println!("{}", s);
}

pub fn error(s: &str) {
    eprintln!("{}", s);
}

pub fn random() -> f64 {
    thread_rng().gen()
}

pub fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u128
}
