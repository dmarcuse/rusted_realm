//! Parsers and types for method signatures

use crate::avm2::{Parse, ParseError};
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::iter::repeat_with;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodInfo {
    return_type_idx: u32,
    param_type_indices: Vec<u32>,
    name_idx: u32,
    flags: u8,
    options: Option<Vec<OptionDetail>>,
    param_names: Option<Vec<u32>>,
}

#[allow(dead_code)]
impl MethodInfo {
    pub const NEED_ARGUMENTS: u8 = 0x01;
    pub const NEED_ACTIVATION: u8 = 0x02;
    pub const NEED_REST: u8 = 0x04;
    pub const HAS_OPTIONAL: u8 = 0x08;
    pub const SET_DXNS: u8 = 0x40;
    pub const HAS_PARAM_NAMES: u8 = 0x80;
}

impl Parse for MethodInfo {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let num_params = u32::parse_avm2(input)? as usize;

        let return_type_idx = u32::parse_avm2(input)?;

        let param_type_indices = repeat_with(|| u32::parse_avm2(input))
            .take(num_params)
            .collect::<Result<_, _>>()?;

        let name_idx = u32::parse_avm2(input)?;

        let flags = u8::parse_avm2(input)?;

        let options = if flags & Self::HAS_OPTIONAL == Self::HAS_OPTIONAL {
            let num_option_details = u32::parse_avm2(input)? as usize;
            Some(
                repeat_with(|| OptionDetail::parse_avm2(input))
                    .take(num_option_details)
                    .collect::<Result<_, _>>()?,
            )
        } else {
            None
        };

        let param_names = if flags & Self::HAS_PARAM_NAMES == Self::HAS_PARAM_NAMES {
            Some(
                repeat_with(|| u32::parse_avm2(input))
                    .take(num_params)
                    .collect::<Result<_, _>>()?,
            )
        } else {
            None
        };

        Ok(MethodInfo {
            return_type_idx,
            param_type_indices,
            name_idx,
            flags,
            options,
            param_names,
        })
    }
}

flag_enum! {
    OptionDetailKind {
        Int = 0x03,
        Uint = 0x04,
        Double = 0x06,
        Utf8 = 0x01,
        True = 0x0b,
        False = 0x0a,
        Null = 0x0c,
        Undefined = 0x00,
        Namespace = 0x08,
        PackageNamespace = 0x16,
        PackageInternalNs = 0x17,
        ProtectedNamespace = 0x18,
        ExplicitNamespace = 0x19,
        StaticProtectedNs = 0x1a,
        PrivateNs = 0x05,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionDetail {
    kind: OptionDetailKind,
    value: u32,
}

impl Parse for OptionDetail {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let value = u32::parse_avm2(input)?;
        let kind = OptionDetailKind::parse_avm2(input)?;
        Ok(OptionDetail { kind, value })
    }
}
