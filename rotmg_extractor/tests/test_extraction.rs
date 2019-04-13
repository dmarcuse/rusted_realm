use log::info;
use rotmg_extractor::rabcdasm::RabcdasmBinaries;

/// Test that we can properly unpack the rabcdasm binaries
#[test]
pub fn main() {
    simple_logger::init().unwrap();
    let binaries = RabcdasmBinaries::unpack();
    info!("Unpacked: {:?}", &binaries);
}
