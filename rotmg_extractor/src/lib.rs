//! Utilities to automatically extract mappings from the official ROTMG client

#![deny(missing_docs)]
#![deny(bare_trait_objects)]

mod avm2;
mod extractor;

#[cfg(feature = "wasm")]
mod wasm;

pub use extractor::*;
