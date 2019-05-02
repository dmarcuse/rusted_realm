use super::{Parse, ParseError};
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::iter::repeat_with;

/// An AVM2 constant pool
#[derive(Clone, Serialize, Deserialize)]
pub struct ConstantPool {
    ints: Vec<i32>,
    uints: Vec<u32>,
    doubles: Vec<f64>,
    strings: Vec<String>,
    namespaces: Vec<Namespace>,
    ns_sets: Vec<NamespaceSet>,
    multinames: Vec<Multiname>,
}

#[allow(dead_code)]
impl ConstantPool {
    pub fn int(&self, i: usize) -> i32 {
        self.ints[i - 1]
    }

    pub fn all_ints(&self) -> &[i32] {
        &self.ints
    }

    pub fn uint(&self, i: usize) -> u32 {
        self.uints[i - 1]
    }

    pub fn all_uints(&self) -> &[u32] {
        &self.uints
    }

    pub fn double(&self, i: usize) -> f64 {
        self.doubles[i - 1]
    }

    pub fn all_doubles(&self) -> &[f64] {
        &self.doubles
    }

    pub fn string(&self, i: usize) -> &str {
        &self.strings[i - 1]
    }

    pub fn all_strings(&self) -> &[String] {
        &self.strings
    }

    pub fn namespace(&self, i: usize) -> &Namespace {
        &self.namespaces[i - 1]
    }

    pub fn all_namespaces(&self) -> &[Namespace] {
        &self.namespaces
    }

    pub fn ns_set(&self, i: usize) -> &NamespaceSet {
        &self.ns_sets[i - 1]
    }

    pub fn ns_sets(&self) -> &[NamespaceSet] {
        &self.ns_sets
    }

    pub fn multiname(&self, i: usize) -> &Multiname {
        &self.multinames[i - 1]
    }

    pub fn multinames(&self) -> &[Multiname] {
        &self.multinames
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
            .field(
                &format!("multinames[{}", self.multinames.len()),
                &self.multinames,
            )
            .finish()
    }
}

impl Parse for ConstantPool {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let num_ints = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let ints = repeat_with(|| i32::parse_avm2(input))
            .take(num_ints)
            .collect::<Result<_, _>>()?;

        let num_uints = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let uints = repeat_with(|| u32::parse_avm2(input))
            .take(num_uints)
            .collect::<Result<_, _>>()?;

        let num_doubles = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let doubles = repeat_with(|| f64::parse_avm2(input))
            .take(num_doubles)
            .collect::<Result<_, _>>()?;

        let num_strings = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let strings = repeat_with(|| String::parse_avm2(input))
            .take(num_strings)
            .collect::<Result<_, _>>()?;

        let num_namespaces = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let namespaces = repeat_with(|| Namespace::parse_avm2(input))
            .take(num_namespaces)
            .collect::<Result<_, _>>()?;

        let num_nssets = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let ns_sets = repeat_with(|| NamespaceSet::parse_avm2(input))
            .take(num_nssets)
            .collect::<Result<_, _>>()?;

        let num_multinames = u32::parse_avm2(input)?.saturating_sub(1) as usize;
        let multinames = repeat_with(|| Multiname::parse_avm2(input))
            .take(num_multinames)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            ints,
            uints,
            doubles,
            strings,
            namespaces,
            ns_sets,
            multinames,
        })
    }
}

flag_enum! {
    NamespaceKind {
        Namespace = 0x08,
        PackageNamespace = 0x16,
        PackageInternalNs = 0x17,
        ProtectedNamespace = 0x18,
        ExplicitNamespace = 0x19,
        StaticProtectedNs = 0x1a,
        PrivateNs = 0x05
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Namespace {
    kind: NamespaceKind,
    name_index: u32,
}

impl Parse for Namespace {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let kind = NamespaceKind::parse_avm2(input)?;
        let name_index = u32::parse_avm2(input)?;

        Ok(Self { kind, name_index })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

flag_enum! {
    MultinameKind {
        QName = 0x07,
        QNameA = 0x0d,
        RTQName = 0x0f,
        RTQNameA = 0x10,
        RTQNameL = 0x11,
        RTQNameLA = 0x12,
        Multiname = 0x09,
        MultinameA = 0x0e,
        MultinameL = 0x1b,
        MultinameLA = 0x1c,
        Typename = 0x1d, // undocumented!
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Multiname {
    QName {
        kind: MultinameKind,
        ns_idx: u32,
        name_idx: u32,
    },
    RTQName {
        kind: MultinameKind,
        name_idx: u32,
    },
    RTQNameL {
        kind: MultinameKind,
    },
    Multiname {
        kind: MultinameKind,
        name_idx: u32,
        ns_set_idx: u32,
    },
    MultinameL {
        kind: MultinameKind,
        ns_set_idx: u32,
    },
    Typename {
        kind: MultinameKind,
        qname_index: u32,
        param_indices: Vec<u32>,
    },
}

impl Multiname {
    pub fn link_qname<'a>(&'a self, constants: &'a ConstantPool) -> (&'a str, &'a str) {
        match self {
            Multiname::QName {
                kind,
                ns_idx,
                name_idx,
            } => {
                let ns = match *ns_idx {
                    0 => "*",
                    i => match constants.namespace(i as usize).name_index {
                        0 => "",
                        i => constants.string(i as usize),
                    },
                };

                let name = match *name_idx {
                    0 => "*",
                    i => constants.string(i as usize),
                };

                (ns, name)
            }
            _ => panic!("Expected QName variant, got {:?}", self),
        }
    }
}

impl Parse for Multiname {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let kind = MultinameKind::parse_avm2(input)?;
        let multiname = match kind {
            MultinameKind::QName | MultinameKind::QNameA => {
                let ns_idx = u32::parse_avm2(input)?;
                let name_idx = u32::parse_avm2(input)?;

                Multiname::QName {
                    kind,
                    ns_idx,
                    name_idx,
                }
            }
            MultinameKind::RTQName | MultinameKind::RTQNameA => {
                let name_idx = u32::parse_avm2(input)?;

                Multiname::RTQName { kind, name_idx }
            }
            MultinameKind::RTQNameL | MultinameKind::RTQNameLA => Multiname::RTQNameL { kind },
            MultinameKind::Multiname | MultinameKind::MultinameA => {
                let name_idx = u32::parse_avm2(input)?;
                let ns_set_idx = u32::parse_avm2(input)?;

                Multiname::Multiname {
                    kind,
                    name_idx,
                    ns_set_idx,
                }
            }
            MultinameKind::MultinameL | MultinameKind::MultinameLA => {
                let ns_set_idx = u32::parse_avm2(input)?;
                Multiname::MultinameL { kind, ns_set_idx }
            }
            MultinameKind::Typename => {
                let qname_index = u32::parse_avm2(input)?;
                let num_params = u32::parse_avm2(input)? as usize;
                let param_indices = repeat_with(|| u32::parse_avm2(input))
                    .take(num_params)
                    .collect::<Result<_, _>>()?;

                Multiname::Typename {
                    kind,
                    qname_index,
                    param_indices,
                }
            }
        };

        Ok(multiname)
    }
}
