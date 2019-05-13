#![allow(dead_code)]

pub mod avm2;
pub mod extractor;

use failure::Fallible;
use rotmg_extractor::ParsedClient;
use serde_json::to_string as to_json;
use std::env::args;
use std::fs::read;
use std::path::Path;

/// Extract and serialize mappings from the given SWF
fn generate_from_client(client: &Path) -> Fallible<String> {
    let contents = read(client)?;
    let mappings = ParsedClient::new(&contents)?.extract_mappings()?;
    Ok(to_json(&mappings)?)
}

fn main() {
    for arg in args().skip(1) {
        match generate_from_client(Path::new(&arg)) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                println!("null");
                eprintln!("Error generating mappings for {}: {:?}", arg, e);
            }
        }
    }
}
