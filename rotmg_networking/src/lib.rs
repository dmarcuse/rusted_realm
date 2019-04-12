//! Implementation of the ROTMG networking protocol.

#![deny(missing_docs)]
#![deny(bare_trait_objects)]

pub mod adapter;
pub mod connection;
mod ext;
pub mod mappings;
pub mod packets;
pub mod rc4;
