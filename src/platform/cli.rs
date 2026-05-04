// Native CLI platform implementations

use rand::{thread_rng, Rng};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static LOGGING_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn set_logging(enabled: bool) {
    LOGGING_ENABLED.store(enabled, Ordering::Relaxed); // TODO (Read)
}

pub fn log(s: &str) {
    if LOGGING_ENABLED.load(Ordering::Relaxed) {
        println!("{}", s);
    }
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
