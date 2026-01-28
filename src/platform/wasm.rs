// WASM platform implementations

mod q {
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
        pub fn now() -> f64;
    }
}

pub fn log(s: &str) {
    q::log(s)
}

pub fn error(s: &str) {
    q::error(s)
}

pub fn random() -> f64 {
    q::random()
}

pub fn now() -> u128 {
    q::now() as u128
}
