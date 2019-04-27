//! Utilities to automatically extract mappings from the official ROTMG client

#![deny(missing_docs)]
#![deny(bare_trait_objects)]

pub mod mapping_extractor;
pub mod rabcdasm;

pub use mapping_extractor::extract_mappings;
