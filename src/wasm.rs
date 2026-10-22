use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn version() -> String {
    crate::VERSION.to_owned()
}

#[wasm_bindgen(js_name = runProgram)]
pub fn run_program(source: &str, mode: &str, proof: &str) -> Result<String, JsValue> {
    crate::execute(source, mode, proof, crate::Limits::default())
        .map_err(|error| JsValue::from_str(&error.to_string()))
}
