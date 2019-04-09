//! Implementations of `Adapter` for non-primitive standard types

use super::{Adapter, Result};
use bytes::{Buf, BufMut};

/// Will only attempt to deserialize when bytes are remaining in the buffer
/// Will only serialize when `Some(T)` is passed
impl<T: Adapter> Adapter for Option<T> {
    fn get_be(bytes: &mut dyn Buf) -> Result<Self>
    where
        Self: Sized,
    {
        if bytes.has_remaining() {
            T::get_be(bytes).map(Some)
        } else {
            Ok(None)
        }
    }

    fn put_be(&self, buffer: &mut dyn BufMut) -> Result<()> {
        match self {
            Some(v) => v.put_be(buffer),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::Adapter;
    use bytes::IntoBuf;

    #[test]
    fn test_primitive_adapters() {
        let mut buffer = vec![];

        // write some data
        Some(42i32).put_be(&mut buffer).unwrap();
        None::<i32>.put_be(&mut buffer).unwrap();

        // read some data

        let mut reader = buffer.into_buf();

        assert_eq!(Some(42i32), Adapter::get_be(&mut reader).unwrap());
        assert_eq!(None::<i32>, Adapter::get_be(&mut reader).unwrap());
    }
}
