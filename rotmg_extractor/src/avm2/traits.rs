//! Parsers for AVM2 traits

use crate::avm2::{Parse, ParseError};
use bytes::Buf;
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

data_struct! {
    TraitSlot {
        slot_id: u32,
        type_name_idx: u32,
        value_idx: u32,
        value_kind: ConstantKind
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
