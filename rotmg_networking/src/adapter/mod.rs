//! Adapters to allow types to be converted to a big endian binary format
//! for use with ROTMG

mod complex;
mod primitives;
mod rle;

pub use self::rle::RLE;

use bytes::{Buf, BufMut};
use failure_derive::Fail;

/// An error occurring when converting a type to or from big endian
#[derive(Debug, Fail)]
pub enum Error {
    /// The type could not be deserialized since there were not enough bytes
    /// remaining in the buffer.
    #[fail(
        display = "Not enough bytes remaining in buffer: need {} bytes, {} bytes remaining",
        needed, remaining
    )]
    InsufficientBytes {
        /// The number of bytes remaining in the buffer
        remaining: usize,
        /// The number of bytes still needed to continue deserializing
        needed: usize,
    },

    /// The data was invalid, as described by the given message
    #[fail(display = "Invalid data: {}", _0)]
    InvalidData(String),

    /// A different type of error
    #[fail(display = "Unexpected error: {}", _0)]
    Other(failure::Error),
}

impl From<failure::Error> for Error {
    fn from(e: failure::Error) -> Self {
        Error::Other(e)
    }
}

/// The result of serializing or deserializing a type
pub type Result<T> = std::result::Result<T, Error>;

/// An adapter for converting a type to a ROTMG-compatible binary format
pub trait Adapter {
    /// Deserialize an instance from the given buffer. The amount of data
    /// remaining in the buffer should be checked and
    /// [`Error::InsufficientBytes`] should be returned when appropriate.
    fn get_be(bytes: &mut dyn Buf) -> Result<Self>
    where
        Self: Sized;

    /// Serialize an instance into the given buffer. It may be assumed that the
    /// buffer will be large enough to store the entire encoded sequence, so no
    /// size checks are necessary. It is recommended that a growable buffer is
    /// used to ensure this is the case when directly calling this method.
    fn put_be(&self, buffer: &mut dyn BufMut) -> Result<()>;
}
