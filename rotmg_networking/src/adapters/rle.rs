//! Implementations of `NetworkAdapter` for types that wrap variable length
//! values

use super::{Adapter, Error, Result};
use bytes::{Buf, BufMut};
use num::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ops::Deref;
use std::result::Result as StdResult;

/// A wrapper around a value (of type `T`) which can be serialized or
/// deserialized by prefixing the data with an integer representing the length
/// (in bytes) of the data, of type `S`. Implementations are provided for
/// `RLE<Vec<T>>` and `RLE<String>`.
///
/// # Examples
///
/// Encode a vector of bytes, prefixed with an unsigned 16-bit length:
/// ```
/// # use crate::rotmg_networking::adapters::{Adapter, RLE};
/// # use std::io::Cursor;
/// // wrap the bytes
/// let bytes: RLE<Vec<u8>> = RLE::new(vec![1, 2, 3]);
///
/// // encode the vector
/// let mut encoded = vec![];
/// bytes.put_be(&mut encoded).unwrap();
///
/// // check the results
/// assert_eq!(encoded, vec![0, 3, 1, 2, 3]);
/// ```
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

impl<S> Adapter for RLE<String, S>
where
    S: Adapter + ToPrimitive + FromPrimitive + Display,
{
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        // use the RLE<Vec<u8>> adapter to simplify this
        RLE::<Vec<u8>, S>::get_be(bytes)
            .and_then(|bytes| String::from_utf8(bytes.unwrap()).map_err(|e| Error::Other(e.into())))
            .map(|s| Self::new(s))
    }

    fn put_be(&self, buffer: &mut dyn BufMut) -> Result<()> {
        // attempt to convert length from a usize
        if let Some(len) = S::from_usize(self.inner.len()) {
            // encode length and then each byte
            len.put_be(buffer)?;
            buffer.put_slice(self.inner.as_bytes());
            Ok(())
        } else {
            Err(Error::InvalidData(format!(
                "cannot cast length from usize: {}",
                self.inner.len()
            )))
        }
    }
}

impl<T, S> Deref for RLE<T, S> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, S> AsRef<T> for RLE<T, S> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T: Debug, S> Debug for RLE<T, S> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self.inner)
    }
}

impl<T: Display, S> Display for RLE<T, S> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.inner)
    }
}

impl<T: PartialEq, S, S2> PartialEq<RLE<T, S2>> for RLE<T, S> {
    fn eq(&self, other: &RLE<T, S2>) -> bool {
        self.inner == other.inner
    }
}

impl<T: Clone, S> Clone for RLE<T, S> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use std::io::Cursor;
    use std::mem::size_of;

    #[test]
    fn check_rle_size() {
        assert_eq!(size_of::<RLE<Vec<u32>>>(), size_of::<Vec<u32>>());
    }

    #[test]
    fn test_rle_vec() {
        let mut buf = vec![];
        RLE::<Vec<u8>>::new(vec![1, 2, 3, 4, 5])
            .put_be(&mut buf)
            .expect("encoding error");
        assert_eq!(buf, vec![0, 5, 1, 2, 3, 4, 5]);

        let output = RLE::<Vec<u8>>::get_be(&mut Cursor::new(&buf)).expect("decoding error");
        assert_eq!(output.unwrap(), vec![1, 2, 3, 4, 5]);

        let large = (0..300).collect::<Vec<u16>>();
        assert_matches!(
            RLE::<_, u8>::new(large).put_be(&mut buf),
            Err(Error::InvalidData(_))
        );
    }

    #[test]
    fn test_rle_string() {
        let mut buf = vec![];
        RLE::<String>::new("hello world".to_owned())
            .put_be(&mut buf)
            .expect("encoding error");

        let expected_encoded = {
            let mut b = vec![0, 11];
            b.extend_from_slice(b"hello world");
            b
        };

        assert_eq!(buf, expected_encoded);

        let output = RLE::<String>::get_be(&mut Cursor::new(&buf)).expect("decoding error");
        assert_eq!(output.unwrap(), "hello world");

        let large = "abc".repeat(100);
        assert_matches!(
            RLE::<String, u8>::new(large).put_be(&mut buf),
            Err(Error::InvalidData(_))
        )
    }
}

impl<T: Serialize, S> Serialize for RLE<T, S> {
    fn serialize<SE: Serializer>(&self, serializer: SE) -> StdResult<SE::Ok, SE::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>, S> Deserialize<'de> for RLE<T, S> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        T::deserialize(deserializer).map(Self::new)
    }
}
