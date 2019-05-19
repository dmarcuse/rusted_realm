use failure::Fallible;
use log::info;
use rotmg_extractor::ParsedClient;
use std::time::Instant;

const CLIENT: &[u8] = include_bytes!("AssembleeGameClient1558086835.swf");

#[test]
fn test_extraction() -> Fallible<()> {
    simple_logger::init()?;

    let started = Instant::now();

    let parsed = ParsedClient::new(CLIENT)?;
    let mappings = parsed.extract_mappings()?;

    info!("Extracted mappings: {:#?}", &mappings);

    let unmapped = mappings.find_unmapped().collect::<Vec<_>>();

    if !unmapped.is_empty() {
        panic!("Missing packet mappings: {:?}", unmapped);
    } else {
        info!("No unmapped packet types!");
    }

    let params = parsed.extract_parameters()?;

    info!("Client parameters: {:?}", params);

    info!(
        "Total extraction time: {} ms",
        started.elapsed().as_millis()
    );

    Ok(())
}
