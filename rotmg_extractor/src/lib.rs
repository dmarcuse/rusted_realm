//! Utilities to automatically extract mappings from the official ROTMG client

#![deny(missing_docs)]
#![deny(bare_trait_objects)]

pub(crate) mod avm2;
mod extractor;

pub use extractor::*;
