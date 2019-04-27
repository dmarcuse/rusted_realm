use failure::Fallible;
use rotmg_extractor::extract_mappings;
use std::io::Cursor;

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
