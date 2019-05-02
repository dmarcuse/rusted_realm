//! Parser for AVM2 metadata

use super::{Parse, ParseError};
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::iter::repeat_with;

/// A piece of metadata, storing key-value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    name_idx: u32,
    items: Vec<MetadataItem>,
}

impl Parse for Metadata {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let name_idx = u32::parse_avm2(input)?;

        let num_items = u32::parse_avm2(input)? as usize;
        let items = repeat_with(|| MetadataItem::parse_avm2(input))
            .take(num_items)
            .collect::<Result<_, _>>()?;

        Ok(Self { name_idx, items })
    }
}

/// A key-value pair of a metadata item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetadataItem {
    key_idx: u32,
    value_idx: u32,
}

impl Parse for MetadataItem {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let key_idx = u32::parse_avm2(input)?;
        let value_idx = u32::parse_avm2(input)?;

        Ok(Self { key_idx, value_idx })
    }
}
