#[cfg(feature = "wasm")]
mod definitions {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {

        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);

        #[wasm_bindgen(js_namespace = console)]
        pub fn error(s: &str);

        // Supposed to be f64 for JS, look it up 

        #[wasm_bindgen(js_namespace = Math)]
        pub fn random() -> f64;

        #[wasm_bindgen(js_namespace = Date)]
        pub fn now() -> f64;
    }
}

#[cfg(feature = "wasm")]
pub use definitions::{log, error, random};

#[cfg(feature = "wasm")]
pub fn now() -> u128 {
    definitions::now() as u128 // Type cast, cannot directly re-export like log, error
}

mod native {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn log(s: &str) {
        println!("{}", s);
    }

    pub fn error(s: &str) {
        eprintln!("{}", s);
    }

    #[cfg(feature = "rand")]
    pub fn random() -> f64 {
        use rand::thread_rng;
        thread_rng().gen()
    }

    #[cfg(not(feature = "rand"))]
    pub fn random() -> f64 {
        0.5
    }

    pub fn now() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u128
    }
}

#[cfg(not(feature = "wasm"))]
pub use native::{log, error, random, now};
