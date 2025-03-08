use wasm_bindgen::prelude::*;
use deno_panic::symbolicate;

#[wasm_bindgen]
pub fn symbolicate(trace: String, symcache: &[u8]) -> Result<JsValue, String> {
    let mut input = trace.as_bytes().iter().copied();
    let value = symbolicate::symbolicate_addrs(&mut input, symcache)
      .map_err(|e| e.to_string())?;
    serde_wasm_bindgen::to_value(&value).map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn create_symcache(debug_file: &[u8]) -> Result<Vec<u8>, String> {
    symbolicate::create_symcache(debug_file).map_err(|e| e.to_string())
}
