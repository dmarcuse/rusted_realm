use crate::avm2::parsers::{Parse, ParseError};
use bytes::Buf;
use std::iter::{once, repeat_with};

/// An AVM2 constant pool
#[derive(Debug, Clone)]
pub struct ConstantPool {
    ints: Vec<i32>,
    uints: Vec<u32>,
    doubles: Vec<f64>,
    strings: Vec<String>,
    // TODO: namespaces, namespacesets, multinames?
}

#[allow(dead_code)]
impl ConstantPool {
    pub fn ints(&self) -> &[i32] {
        &self.ints
    }

    pub fn uints(&self) -> &[u32] {
        &self.uints
    }

    pub fn doubles(&self) -> &[f64] {
        &self.doubles
    }

    pub fn strings(&self) -> &[String] {
        &self.strings
    }
}

impl Parse for ConstantPool {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let num_ints = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let ints = once(Ok(0))
            .chain(repeat_with(|| i32::parse_avm2(input)).take(num_ints))
            .collect::<Result<_, _>>()?;

        let num_uints = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let uints = once(Ok(0))
            .chain(repeat_with(|| u32::parse_avm2(input)).take(num_uints))
            .collect::<Result<_, _>>()?;

        let num_doubles = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let doubles = once(Ok(0.0))
            .chain(repeat_with(|| f64::parse_avm2(input)).take(num_doubles))
            .collect::<Result<_, _>>()?;

        let num_strings = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let strings = once(Ok(String::new()))
            .chain(repeat_with(|| String::parse_avm2(input)).take(num_strings))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            ints,
            uints,
            doubles,
            strings,
        })
    }
}
