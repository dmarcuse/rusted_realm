use crate::extractor::ParsedClient;
use failure::Error;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

/// TypeScript code which will be inserted into the type definition file for the module.
#[wasm_bindgen(typescript_custom_section)]
const TS_TYPE_DEFINITIONS: &'static str = r#"
/**
 * Network related mappings including a packet ID map and
 * the RC4 cipher keys.
 */
export interface Mappings {
    /**
     * A map of packet IDs to packet types.
     */
    mappings: Record<number, string>;
    /**
     * A byte array containing both RC4 cipher keys. The outgoing key takes
     * up the first 13 bytes of the array and the incoming key takes up
     * the last 13 bytes of the array.
     */
    binary_rc4: number[];
}

/**
 * Various constants used throughout the game.
 */
export interface Parameters {
    /**
     * The current build version of the game.
     */
    version: string;
    /**
     * The port on which connections will be established.
     */
    port: number;
    /**
     * The game ID used to connect to the Tutorial.
     */
    tutorial_gameid: number;
    /**
     * The game ID used to connect to the Nexus.
     */
    nexus_gameid: number;
    /**
     * The game ID used to connect to a random realm.
     */
    random_gameid: number;
}

export interface Extractor {
    /**
     * Extracts the `Mappings` from the client.
     */
    mappings(): Mappings;
    /**
     * Extracts the `Parameters` from the client.
     */
    parameters(): Parameters;
}
"#;

/// A struct which can be used to parse a game client and then extract various bits of information
/// from that client.
#[wasm_bindgen]
pub struct Extractor {
    client: ParsedClient,
}

#[wasm_bindgen]
impl Extractor {
    /// Creates a new `Extractor` over the given `client`.
    #[wasm_bindgen(constructor)]
    pub fn new(client: &[u8]) -> Result<Extractor, JsValue> {
        let client = ParsedClient::new(client).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self { client })
    }

    /// Extracts the `Mappings` from the client.
    #[wasm_bindgen(skip_typescript)]
    pub fn mappings(&self) -> Result<JsValue, JsValue> {
        map_extraction(|| self.client.extract_mappings())
    }

    /// Extracts the `Parameters` from the client.
    #[wasm_bindgen(skip_typescript)]
    pub fn parameters(&self) -> Result<JsValue, JsValue> {
        map_extraction(|| self.client.extract_parameters())
    }
}

/// Calls the provided `extractor` and maps the returned `Result` into a `Result<JsValue,
/// JsValue>`.
fn map_extraction<R, F>(extractor: F) -> Result<JsValue, JsValue>
where
    R: Serialize,
    F: FnOnce() -> Result<R, Error>,
{
    extractor()
        .map_err(|e| JsValue::from_str(&e.to_string()))
        .and_then(|r| JsValue::from_serde(&r).map_err(|e| JsValue::from_str(&e.to_string())))
}
