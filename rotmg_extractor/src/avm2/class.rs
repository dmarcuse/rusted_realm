//! Parsers for AVM2 classes

use crate::avm2::traits::Trait;
use crate::avm2::{Parse, ParseError};
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

        println!("Parsing class - cinit idx {}", cinit_idx);

        let num_traits = dbg!(u32::parse_avm2(input)? as usize);
        let traits = repeat_with(|| Trait::parse_avm2(input))
            .take(num_traits)
            .collect::<Result<_, _>>()?;

        Ok(Self { cinit_idx, traits })
    }
}
