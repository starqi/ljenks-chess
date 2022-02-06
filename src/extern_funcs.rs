#[cfg(test)]
use rand::prelude::*;
#[cfg(test)]
use std::time::Instant;

mod definitions {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {

        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);

        #[wasm_bindgen(js_namespace = console)]
        pub fn error(s: &str);

        #[wasm_bindgen(js_namespace = Math)]
        pub fn random() -> f64;

        #[wasm_bindgen(js_namespace = Date)]
        pub fn now() -> u32;
    }
}

#[cfg(test)]
pub fn log(s: &str) {
    println!("{}", s);
}

#[cfg(not(test))]
pub fn log(s: &str) {
    definitions::log(s);
}

#[cfg(test)]
pub fn error(s: &str) {
    eprintln!("{}", s);
}

#[cfg(not(test))]
pub fn error(s: &str) {
    definitions::error(s);
}

#[cfg(test)]
pub fn random() -> f64 {
    thread_rng().gen()
}

#[cfg(not(test))]
pub fn random() -> f64 {
    definitions::random()
}

#[cfg(test)]
pub fn now() -> u128 {
    Instant::now().elapsed().as_millis()
}

#[cfg(not(test))]
pub fn now() -> u128 {
    definitions::now() as u128
}
