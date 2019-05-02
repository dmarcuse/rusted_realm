use crate::avm2::class::Class;
use crate::avm2::constants::ConstantPool;
use crate::avm2::metadata::Metadata;
use crate::avm2::methods::MethodInfo;
use crate::avm2::{Parse, ParseError};
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::iter::repeat_with;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcFile {
    minor_version: u16,
    major_version: u16,
    constants: ConstantPool,
    methods: Vec<MethodInfo>,
    metadata: Vec<Metadata>,
    classes: Vec<Class>,
}

impl Parse for AbcFile {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let minor_version = u16::parse_avm2(input)?;
        let major_version = u16::parse_avm2(input)?;
        let constants = ConstantPool::parse_avm2(input)?;

        let num_methods = u32::parse_avm2(input)? as usize;
        let methods = repeat_with(|| MethodInfo::parse_avm2(input))
            .take(num_methods)
            .collect::<Result<_, _>>()?;

        let num_metadata = u32::parse_avm2(input)? as usize;
        let metadata = repeat_with(|| Metadata::parse_avm2(input))
            .take(num_metadata)
            .collect::<Result<_, _>>()?;

        let num_classes = u32::parse_avm2(input)? as usize;
        let classes = repeat_with(|| Class::parse_avm2(input))
            .take(num_classes)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            minor_version,
            major_version,
            constants,
            methods,
            metadata,
            classes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Fallible;
    use std::io::Cursor;
    use std::time::Instant;
    use swf_parser::parsers::movie::parse_movie;
    use swf_tree::Tag;

    const CLIENT: &[u8] = include_bytes!("../../tests/AssembleeGameClient1556108352.swf");

    #[test]
    fn test_parse_constants() -> Fallible<()> {
        let start = Instant::now();
        let (_, movie) = parse_movie(CLIENT)?;
        let abc = movie
            .tags
            .iter()
            .filter_map(|t| match t {
                Tag::DoAbc(abc) => Some(abc),
                _ => None,
            })
            .nth(0)
            .unwrap();

        let mut buf = Cursor::new(&abc.data);
        let abc = AbcFile::parse_avm2(&mut buf)?;
        println!("Classes: {:#?}", abc.classes);
        println!("Parsed in {} ms", start.elapsed().as_millis());

        Ok(())
    }
}
