use super::constants::ConstantPool;
use crate::avm2::{Parse, ParseError};
use bytes::Buf;

#[derive(Debug, Clone)]
pub struct AbcFile {
    minor_version: u16,
    major_version: u16,
    constants: ConstantPool,
}

impl Parse for AbcFile {
    fn parse_avm2(input: &mut dyn Buf) -> Result<Self, ParseError> {
        let minor_version = u16::parse_avm2(input)?;
        let major_version = u16::parse_avm2(input)?;
        let constants = ConstantPool::parse_avm2(input)?;

        Ok(Self {
            minor_version,
            major_version,
            constants,
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
        println!("Parsed in {} ms", start.elapsed().as_millis());

        Ok(())
    }
}
