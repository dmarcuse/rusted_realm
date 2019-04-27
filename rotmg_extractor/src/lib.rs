//! Utilities to automatically extract mappings from the official ROTMG client

#![deny(missing_docs)]
#![deny(bare_trait_objects)]

pub mod rabcdasm;

#[cfg(feature = "mappings")]
pub mod mappings;
