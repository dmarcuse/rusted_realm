use crate::avm2::parsers::{Parse, ParseError};
use bytes::Buf;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::iter::{once, repeat_with};

/// An AVM2 constant pool
#[derive(Clone)]
pub struct ConstantPool {
    ints: Vec<i32>,
    uints: Vec<u32>,
    doubles: Vec<f64>,
    strings: Vec<String>,
    namespaces: Vec<Namespace>,
    ns_sets: Vec<NamespaceSet>,
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

    pub fn namespaces(&self) -> &[Namespace] {
        &self.namespaces
    }

    pub fn ns_sets(&self) -> &[NamespaceSet] {
        &self.ns_sets
    }
}

impl Debug for ConstantPool {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("ConstantPool")
            .field(&format!("ints[{}]", self.ints.len()), &self.ints)
            .field(&format!("uints[{}]", self.uints.len()), &self.uints)
            .field(&format!("doubles[{}]", self.doubles.len()), &self.doubles)
            .field(&format!("strings[{}]", self.strings.len()), &self.strings)
            .field(
                &format!("namespaces[{}]", self.namespaces.len()),
                &self.namespaces,
            )
            .field(&format!("ns_sets[{}]", self.ns_sets.len()), &self.ns_sets)
            .finish()
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

        let num_namespaces = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let namespaces = once(Ok(Namespace::default()))
            .chain(repeat_with(|| Namespace::parse_avm2(input)).take(num_namespaces))
            .collect::<Result<_, _>>()?;

        let num_nssets = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let ns_sets = once(Ok(NamespaceSet::default()))
            .chain(repeat_with(|| NamespaceSet::parse_avm2(input)).take(num_nssets))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            ints,
            uints,
            doubles,
            strings,
            namespaces,
            ns_sets,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Namespace {
    kind: u8,
    name_index: u32,
}

impl Parse for Namespace {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let kind = u8::parse_avm2(input)?;
        let name_index = u32::parse_avm2(input)?;

        Ok(Self { kind, name_index })
    }
}

#[derive(Debug, Clone, Default)]
pub struct NamespaceSet {
    namespace_indices: Vec<u32>,
}

impl Parse for NamespaceSet {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let count = u32::parse_avm2(input)? as usize;
        let namespace_indices = repeat_with(|| u32::parse_avm2(input))
            .take(count)
            .collect::<Result<_, _>>()?;

        Ok(Self { namespace_indices })
    }
}
