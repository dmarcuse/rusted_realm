use crate::extractor::ParsedClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub fn extract_mappings(client: &[u8]) -> Result<JsValue, JsValue> {
    ParsedClient::new(client)
        .and_then(|c| c.extract_mappings())
        .map_err(|e| JsValue::from_str(&e.to_string()))
        .and_then(|m| JsValue::from_serde(&m).map_err(|e| JsValue::from_str(&e.to_string())))
}
