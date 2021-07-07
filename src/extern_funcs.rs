use wasm_bindgen::prelude::*;

// Use `js_namespace` here to bind `console.log(..)` instead of just `log(..)`
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
