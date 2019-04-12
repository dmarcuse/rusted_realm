//! Low-level, efficient, asynchronous implementation of the ROTMG networking
//! protocol
//!
//! This crate provides utilities to represent ROTMG network packets, as well as
//! code to open ROTMG client or server connections using tokio streams.

#![deny(missing_docs)]
#![deny(bare_trait_objects)]

pub mod adapter;
pub mod connection;
mod ext;
pub mod mappings;
pub mod packets;
pub mod rc4;
