use super::{Parse, ParseError};
use bytes::Buf;
use std::convert::TryInto;

/// A signed 24-bit integer - not fully implemented
#[derive(Debug, Clone, Copy)]
pub struct S24([u8; 3]);

impl Parse for S24 {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        if input.remaining() >= 3 {
            Ok(input.bytes()[..3]
                .try_into()
                .map(S24)
                .map_err(|e| ParseError::Other(e.into()))?)
        } else {
            Err(ParseError::InsufficientBytes {
                remaining: input.remaining(),
                needed: 3,
            })
        }
    }
}
