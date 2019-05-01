//! Basic parser for AVM2 bytecode

use bytes::Buf;
use failure_derive::Fail;

#[macro_use]
pub mod macros;

pub mod abcfile;
pub mod constants;
pub mod methods;
pub mod primitives;
pub mod s24;

/// An error parsing an AVM2 type
#[derive(Debug, Fail)]
pub enum ParseError {
    /// Not enough bytes remained in the buffer to deserialize this type
    #[fail(
        display = "Not enough bytes remaining in buffer: need {} bytes, {} bytes remaining",
        needed, remaining
    )]
    InsufficientBytes { remaining: usize, needed: usize },

    /// A different error occurred
    #[fail(display = "Unexpected error: {}", _0)]
    Other(failure::Error),
}

/// A trait defining functionality for parsing an AVM2 type
pub trait Parse: Sized {
    /// Parse this type from the provided bytes
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError>;
}
