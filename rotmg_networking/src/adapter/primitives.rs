//! Implementations of `Adapter` for standard primitive types

use super::{Adapter, Error, Result};
use bytes::{Buf, BufMut};
use std::mem::size_of;

// define macros for integer and floating point types
macro_rules! int_adapter {
    ($($type:ty),* $(,)?) => {
        $(
            impl Adapter for $type {
                fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
                    if bytes.remaining() < size_of::<Self>() {
                        Err(Error::InsufficientBytes {
                            remaining: bytes.remaining(),
                            needed: size_of::<Self>(),
                        })
                    } else {
                        let mut raw = [0u8; size_of::<Self>()];
                        bytes.copy_to_slice(&mut raw[..]);
                        Ok(Self::from_be_bytes(raw))
                    }
                }

                fn put_be(&self, bytes: &mut dyn BufMut) -> Result<()> {
                    bytes.put_slice(&self.to_be_bytes());
                    Ok(())
                }
            }
        )*
    }
}

macro_rules! float_adapter {
    ($($type:ty),* $(,)?) => {
        $(
            impl Adapter for $type {
                fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
                    Adapter::get_be(bytes).map(Self::from_bits)
                }

                fn put_be(&self, buffer: &mut dyn BufMut) -> Result<()> {
                    self.to_bits().put_be(buffer)
                }
            }
        )*
    }
}

// use the macros
int_adapter! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
}

float_adapter! { f32, f64 }

// manually define an adapter for booleans
impl Adapter for bool {
    fn get_be(bytes: &mut dyn Buf) -> Result<bool>
    where
        Self: Sized,
    {
        u8::get_be(bytes).map(|b| b != 0)
    }

    fn put_be(&self, buffer: &mut dyn BufMut) -> Result<()> {
        (*self as u8).put_be(buffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::adapter::Adapter;
    use bytes::{Buf, IntoBuf};

    #[test]
    fn test_primitive_adapters() {
        let mut buffer = vec![];

        // write some data

        123u8.put_be(&mut buffer).unwrap();
        123u16.put_be(&mut buffer).unwrap();
        123u32.put_be(&mut buffer).unwrap();
        123u64.put_be(&mut buffer).unwrap();
        123u128.put_be(&mut buffer).unwrap();

        (-123i8).put_be(&mut buffer).unwrap();
        (-123i16).put_be(&mut buffer).unwrap();
        (-123i32).put_be(&mut buffer).unwrap();
        (-123i64).put_be(&mut buffer).unwrap();
        (-123i128).put_be(&mut buffer).unwrap();

        3.14f32.put_be(&mut buffer).unwrap();
        3.14f64.put_be(&mut buffer).unwrap();

        true.put_be(&mut buffer).unwrap();

        // read some data

        let mut reader = buffer.into_buf();

        assert_eq!(123u8, Adapter::get_be(&mut reader).unwrap());
        assert_eq!(123u16, Adapter::get_be(&mut reader).unwrap());
        assert_eq!(123u32, Adapter::get_be(&mut reader).unwrap());
        assert_eq!(123u64, Adapter::get_be(&mut reader).unwrap());
        assert_eq!(123u128, Adapter::get_be(&mut reader).unwrap());

        assert_eq!(-123i8, Adapter::get_be(&mut reader).unwrap());
        assert_eq!(-123i16, Adapter::get_be(&mut reader).unwrap());
        assert_eq!(-123i32, Adapter::get_be(&mut reader).unwrap());
        assert_eq!(-123i64, Adapter::get_be(&mut reader).unwrap());
        assert_eq!(-123i128, Adapter::get_be(&mut reader).unwrap());

        assert_eq!(3.14f32, Adapter::get_be(&mut reader).unwrap());
        assert_eq!(3.14f64, Adapter::get_be(&mut reader).unwrap());

        assert_eq!(true, Adapter::get_be(&mut reader).unwrap());

        assert_eq!(reader.remaining(), 0);
    }
}
