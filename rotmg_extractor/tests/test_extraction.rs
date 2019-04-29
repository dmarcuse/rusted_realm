use failure::Fallible;
use log::info;
use rotmg_extractor::ParsedClient;

const CLIENT: &[u8] = include_bytes!("AssembleeGameClient1556108352.swf");

#[test]
fn test_extraction() -> Fallible<()> {
    simple_logger::init()?;

    let parsed = ParsedClient::new(CLIENT)?;
    let rc4 = parsed.extract_rc4()?;
    info!("Got RC4 key: {}", rc4);

    Ok(())
}
