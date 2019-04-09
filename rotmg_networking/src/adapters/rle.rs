//! Implementations of `NetworkAdapter` for types that wrap variable length
//! values

use super::{Adapter, Error, Result};
use bytes::{Buf, BufMut};
use num::{FromPrimitive, ToPrimitive};
use std::fmt::Display;
use std::marker::PhantomData;

/// A wrapper around a value (of type `T`) which can be serialized or
/// deserialized by prefixing the data with an integer representing the length
/// (in bytes) of the data, of type `S`. Implementations are provided for
/// `RLE<Vec<T>>` and `RLE<String>`.
pub struct RLE<T, S = u16> {
    inner: T,
    marker: PhantomData<S>,
}

impl<T, S> RLE<T, S> {
    /// Create a new run-length encoded wrapper for the given value
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            marker: PhantomData::default(),
        }
    }

    /// Unwrap this value into the contained type
    pub fn unwrap(self) -> T {
        self.inner
    }
}

impl<T, S> Adapter for RLE<Vec<T>, S>
where
    T: Adapter,
    S: Adapter + ToPrimitive + FromPrimitive + Display,
{
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        // decode length
        let len = S::get_be(bytes)?;

        // attempt to convert length to usize
        if let Some(len) = len.to_usize() {
            // decode remaining items and convert to vec
            (0..len)
                .map(|_| T::get_be(bytes))
                .collect::<Result<Vec<T>>>()
                .map(Self::new)
        } else {
            Err(Error::InvalidData(format!(
                "cannot cast length to usize: {}",
                len
            )))
        }
    }

    fn put_be(&self, buffer: &mut dyn BufMut) -> Result<()> {
        // attempt to convert length from a usize
        if let Some(len) = S::from_usize(self.inner.len()) {
            // encode length and then each element
            len.put_be(buffer)?;
            self.inner.iter().try_for_each(|i| i.put_be(buffer))
        } else {
            Err(Error::InvalidData(format!(
                "cannot cast length from usize: {}",
                self.inner.len()
            )))
        }
    }
}
