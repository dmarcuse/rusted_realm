//! Parsers for basic AVM2 types

use crate::avm2::{Parse, ParseError};
use bytes::Buf;
use std::mem::size_of;

impl Parse for u8 {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        if input.remaining() >= size_of::<Self>() {
            Ok(input.get_u8())
        } else {
            Err(ParseError::InsufficientBytes {
                needed: size_of::<Self>(),
                remaining: input.remaining(),
            })
        }
    }
}

impl Parse for u16 {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        if input.remaining() >= size_of::<Self>() {
            Ok(input.get_u16_le())
        } else {
            Err(ParseError::InsufficientBytes {
                needed: size_of::<Self>(),
                remaining: input.remaining(),
            })
        }
    }
}

impl Parse for f64 {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        if input.remaining() >= size_of::<Self>() {
            Ok(input.get_f64_le())
        } else {
            Err(ParseError::InsufficientBytes {
                needed: size_of::<Self>(),
                remaining: input.remaining(),
            })
        }
    }
}

// this parser is used by the u32, s32, and u30 AVM2 primitives, all of which
// are variable-length integers consisting of sequences of one to five bytes of
// data
impl Parse for u32 {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        // so this mess is why flash died, huh
        // TODO: use Iterator::scan?

        // get the length of the sequence
        // the sequence is terminated by a byte with the high bit unset
        let length = 1 + input
            .bytes()
            .iter()
            .take(4)
            .take_while(|&b| (b & 0x80) == 0x80)
            .count();

        // ensure we have enough bytes left in the buffer
        if input.remaining() < length {
            return Err(ParseError::InsufficientBytes {
                remaining: input.remaining(),
                needed: length,
            });
        }

        // parse the value
        let value = input
            .iter()
            .take(length)
            .enumerate()
            .map(|(i, b)| (b as u32 & 0x7f) << (i * 7))
            .sum();

        Ok(value)
    }
}

impl Parse for i32 {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        Ok(u32::parse_avm2(input)? as i32)
    }
}

impl Parse for String {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        // get the length of the string
        let length = u32::parse_avm2(input)?;

        // get the data
        let data = input.take(length as usize).collect::<Vec<u8>>();

        // convert it to a UTF8 string and return it
        String::from_utf8(data).map_err(|e| ParseError::Other(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_u32_parsing() {
        const CASES: &[(&[u8], u32)] = &[
            (&[0x9f, 0x14], 2591),
            (&[0x01], 1),
            (&[0x81, 0x4c], 9729),
            (&[0xf4, 0x05], 756),
        ];

        for case in CASES {
            let mut buffer = Cursor::new(&case.0[..]);
            assert_eq!(case.1, u32::parse_avm2(&mut buffer).unwrap());
            assert!(!buffer.has_remaining(), "no bytes should remain");
        }
    }
}
