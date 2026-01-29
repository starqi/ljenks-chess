// Native CLI platform implementations
use std::time::{SystemTime, UNIX_EPOCH};
use rand::{thread_rng, Rng};

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