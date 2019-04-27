use failure::Fallible;
use log::info;
use rotmg_extractor::extract_mappings;
use rotmg_extractor::rabcdasm::RabcdasmBinaries;
use std::fs::File;
use std::io::{copy, Cursor};
use std::path::Path;
use tempfile::{tempdir, tempfile};

const CLIENT: &[u8] = include_bytes!("AssembleeGameClient1556108352.swf");

/// Test that we can properly unpack the rabcdasm binaries
#[test]
pub fn main() -> Fallible<()> {
    simple_logger::init()?;

    let mappings = extract_mappings(Cursor::new(CLIENT))?;

    let unmapped = mappings.find_unmapped().collect::<Vec<_>>();

    if !unmapped.is_empty() {
        panic!("Mappings missing: {:#?}", unmapped);
    }

    Ok(())
}
