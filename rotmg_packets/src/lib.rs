//! Type definitions for ROTMG network packets
//!
//! This crate does not implement actual networking, it only provides types and
//! conversions to/from decrypted binary form. Consider the rotmg_networking
//! crate if you need the networking code as well.

#![deny(missing_docs)]
#![deny(bare_trait_objects)]

pub mod adapter;
pub mod mappings;
pub mod packets;
