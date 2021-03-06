//! Parsers for AVM2 traits

use super::constants::ConstantPool;
use super::{Parse, ParseError};
use bytes::Buf;
use failure_derive::Fail;
use serde::{Deserialize, Serialize};
use std::iter::repeat_with;

flag_enum! {
    TraitKind {
        Slot = 0,
        Method = 1,
        Getter = 2,
        Setter = 3,
        Class = 4,
        Function = 5,
        Const = 6
    }
}

flag_enum! {
    ConstantKind {
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
        PrivateNs = 0x05
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitSlot {
    slot_id: u32,
    type_name_idx: u32,
    value_idx: u32,
    value_kind: Option<ConstantKind>,
}

impl Parse for TraitSlot {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let slot_id = u32::parse_avm2(input)?;
        let type_name_idx = u32::parse_avm2(input)?;
        let value_idx = u32::parse_avm2(input)?;

        let value_kind = if value_idx == 0 {
            None
        } else {
            Some(ConstantKind::parse_avm2(input)?)
        };

        Ok(Self {
            slot_id,
            type_name_idx,
            value_idx,
            value_kind,
        })
    }
}

data_struct! {
    TraitClass {
        slot_id: u32,
        class_idx: u32
    }
}

data_struct! {
    TraitFunction {
        slot_id: u32,
        function_idx: u32
    }
}

data_struct! {
    TraitMethod {
        disp_id: u32,
        method_idx: u32
    }
}

/// An AVM2 trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trait {
    /// Represents a slot or const
    Slot {
        name_idx: u32,
        kind: TraitKind,
        attrs: u8,
        data: TraitSlot,
        metadata_indices: Vec<u32>,
    },

    /// Represents a class
    Class {
        name_idx: u32,
        kind: TraitKind,
        attrs: u8,
        data: TraitClass,
        metadata_indices: Vec<u32>,
    },

    /// Represents a function
    Function {
        name_idx: u32,
        kind: TraitKind,
        attrs: u8,
        data: TraitFunction,
        metadata_indices: Vec<u32>,
    },

    /// Represents a method, getter, or setter
    Method {
        name_idx: u32,
        kind: TraitKind,
        attrs: u8,
        data: TraitMethod,
        metadata_indices: Vec<u32>,
    },
}

impl Trait {
    pub const ATTR_FINAL: u8 = 0x1;
    pub const ATTR_OVERRIDE: u8 = 0x2;
    pub const ATTR_METADATA: u8 = 0x4;
}

impl Parse for Trait {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let name_idx = u32::parse_avm2(input)?;

        let kind = u8::parse_avm2(input)?;

        let attrs = kind >> 4;
        let kind = TraitKind::from_u8(kind & 0x0f)?;

        let get_metadata = |input: &mut dyn Buf| {
            if attrs & Self::ATTR_METADATA == Self::ATTR_METADATA {
                let metadata_count = u32::parse_avm2(input)? as usize;
                repeat_with(|| u32::parse_avm2(input))
                    .take(metadata_count)
                    .collect()
            } else {
                Ok(Vec::new())
            }
        };

        let parsed = match kind {
            TraitKind::Slot | TraitKind::Const => Trait::Slot {
                name_idx,
                kind,
                attrs,
                data: TraitSlot::parse_avm2(input)?,
                metadata_indices: get_metadata(input)?,
            },
            TraitKind::Class => Trait::Class {
                name_idx,
                kind,
                attrs,
                data: TraitClass::parse_avm2(input)?,
                metadata_indices: get_metadata(input)?,
            },
            TraitKind::Function => Trait::Function {
                name_idx,
                kind,
                attrs,
                data: TraitFunction::parse_avm2(input)?,
                metadata_indices: get_metadata(input)?,
            },
            TraitKind::Method | TraitKind::Getter | TraitKind::Setter => Trait::Method {
                name_idx,
                kind,
                attrs,
                data: TraitMethod::parse_avm2(input)?,
                metadata_indices: get_metadata(input)?,
            },
        };

        Ok(parsed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraitSlotValue<'a> {
    Int(i32),
    Uint(u32),
    Double(f64),
    String(&'a str),
    None,
}

#[derive(Debug, Fail)]
#[fail(display = "Invalid type conversion")]
pub struct InvalidType;

impl<'a> TraitSlotValue<'a> {
    pub fn as_str(self) -> Result<&'a str, InvalidType> {
        match self {
            TraitSlotValue::String(s) => Ok(s),
            _ => Err(InvalidType),
        }
    }

    pub fn as_int(self) -> Result<i32, InvalidType> {
        match self {
            TraitSlotValue::Int(i) => Ok(i),
            _ => Err(InvalidType),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinkedTraitSlot<'a> {
    pub name: (&'a str, &'a str),
    pub slot_id: u32,
    pub value: TraitSlotValue<'a>,
}

impl Trait {
    pub fn is_slot(&self) -> bool {
        match self {
            Trait::Slot { .. } => true,
            _ => false,
        }
    }

    pub fn link_slot<'a>(&'a self, constants: &'a ConstantPool) -> LinkedTraitSlot<'a> {
        match self {
            Trait::Slot { name_idx, data, .. } => {
                let name = constants
                    .multiname((*name_idx) as usize)
                    .link_qname(constants);

                let value = match data.value_kind {
                    Some(ConstantKind::Int) => {
                        TraitSlotValue::Int(constants.int(data.value_idx as usize))
                    }
                    Some(ConstantKind::Utf8) => {
                        TraitSlotValue::String(constants.string(data.value_idx as usize))
                    }
                    _ => TraitSlotValue::None, // TODO i guess?
                };

                LinkedTraitSlot {
                    name,
                    slot_id: data.slot_id,
                    value,
                }
            }
            _ => panic!("Expected Slot variant, got {:?}", self),
        }
    }
}
