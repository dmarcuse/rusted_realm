//! Parsers for AVM2 classes

use super::traits::Trait;
use super::{Parse, ParseError};
use crate::avm2::constants::ConstantPool;
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::iter::repeat_with;

/// An AVM2 class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    /// Index into method array for static initializer of class
    cinit_idx: u32,

    /// Traits of this class
    traits: Vec<Trait>,
}

impl Parse for Class {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let cinit_idx = u32::parse_avm2(input)?;

        let num_traits = u32::parse_avm2(input)? as usize;
        let traits = repeat_with(|| Trait::parse_avm2(input))
            .take(num_traits)
            .collect::<Result<_, _>>()?;

        Ok(Self { cinit_idx, traits })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    name_idx: u32,
    super_name_idx: u32,
    flags: u8,
    protected_ns_idx: Option<u32>,
    interface_indices: Vec<u32>,
    iinit_idx: u32,
    traits: Vec<Trait>,
}

#[derive(Debug)]
pub struct LinkedClass<'a> {
    /// (namespace, name)
    pub name: (&'a str, &'a str),

    /// Option<(namespace, name)>
    pub super_name: Option<(&'a str, &'a str)>,
}

impl Instance {
    pub const CLASS_SEALED: u8 = 0x01;
    pub const CLASS_FINAL: u8 = 0x02;
    pub const CLASS_INTERFACE: u8 = 0x04;
    pub const CLASS_PROTECTED_NS: u8 = 0x08;

    pub fn link<'a>(&'a self, class: &'a Class, constants: &'a ConstantPool) -> LinkedClass<'a> {
        let name = constants
            .multiname(self.name_idx as usize)
            .link_qname(constants);

        if name.1.contains("Game") {
            println!("Linking: {:?}", self);
            println!(
                "Multiname: {:?}",
                constants.multiname(self.name_idx as usize)
            )
        }

        let super_name = match self.super_name_idx {
            0 => None,
            i => Some(constants.multiname(i as usize).link_qname(constants)),
        };

        LinkedClass { name, super_name }
    }
}

impl Parse for Instance {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let name_idx = u32::parse_avm2(input)?;
        let super_name_idx = u32::parse_avm2(input)?;
        let flags = u8::parse_avm2(input)?;

        let protected_ns_idx = if flags & Self::CLASS_PROTECTED_NS == Self::CLASS_PROTECTED_NS {
            Some(u32::parse_avm2(input)?)
        } else {
            None
        };

        let num_interfaces = u32::parse_avm2(input)? as usize;
        let interface_indices = repeat_with(|| u32::parse_avm2(input))
            .take(num_interfaces)
            .collect::<Result<_, _>>()?;

        let iinit_idx = u32::parse_avm2(input)?;

        let num_traits = u32::parse_avm2(input)? as usize;
        let traits = repeat_with(|| Trait::parse_avm2(input))
            .take(num_traits)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            name_idx,
            super_name_idx,
            flags,
            protected_ns_idx,
            interface_indices,
            iinit_idx,
            traits,
        })
    }
}
