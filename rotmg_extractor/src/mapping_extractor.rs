//! Automatically generate `Mappings` from the official flash game client.

use crate::rabcdasm::RabcdasmBinaries;
use bimap::{BiHashMap, Overwritten};
use failure::Fallible;
use failure_derive::Fail;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use regex::Regex;
use rotmg_networking::mappings::Mappings;
use rotmg_networking::packets::PacketType;
use std::collections::HashMap;
use std::fs::{read_to_string, write, File};
use std::io::{copy, Read};
use std::path::Path;
use std::str::FromStr;
use tempfile::tempdir;

lazy_static! {
    static ref RC4_PATTERN: Regex = Regex::new(include_str!("regex/rc4.re")).unwrap();
    static ref PACKET_PATTERN: Regex = Regex::new(include_str!("regex/packet.re")).unwrap();
}

/// An error extracting `Mappings` from the game client
#[derive(Debug, Fail)]
pub enum MappingError {
    /// RC4 keys couldn't be found in the disassembled game
    #[fail(display = "RC4 keys could not be found")]
    MissingRC4,
}

fn extract_rc4(asm: &Path) -> Fallible<String> {
    // read contents of GameServerConnectionConcrete class
    let gscc = read_to_string(
        asm.join("kabam/rotmg/messaging/impl/GameServerConnectionConcrete.class.asasm"),
    )?;

    if let Some(matches) = RC4_PATTERN.captures(&gscc) {
        Ok(matches[1].to_string())
    } else {
        Err(MappingError::MissingRC4.into())
    }
}

fn extract_packets(asm: &Path) -> Fallible<BiHashMap<u8, PacketType>> {
    // create a map of names
    let mut names = PacketType::get_name_mappings()
        .iter()
        .map(|(&t, n)| (n.to_lowercase(), t))
        .collect::<HashMap<_, _>>();

    // read contents of GameServerConnection disassembly
    let gsc =
        read_to_string(asm.join("kabam/rotmg/messaging/impl/GameServerConnection.class.asasm"))?;

    // create map for packet types
    let mut packets = BiHashMap::with_capacity(PacketType::NUM_TYPES);

    for cap in PACKET_PATTERN.captures_iter(&gsc) {
        let name = cap[1].replace('_', "").to_lowercase();
        let id = u8::from_str(&cap[2]).unwrap();

        if let Some(typ) = names.remove(&name) {
            let overwritten = packets.insert(id, typ);
            debug!("Packet mapped: {:?} <> {}/{}", typ, &cap[1], id);
            debug_assert_eq!(overwritten, Overwritten::Neither, "packet mapping conflict");
        } else {
            warn!("No mapping found for packet {}/{} - skipping!", &cap[1], id);
        }
    }

    for (_, typ) in names {
        warn!("No mapping for for packet type {:?} - skipping!", typ);
    }

    Ok(packets)
}

/// Generate a `Mappings` instance from the given SWF data.
///
/// For convenience, the SWF may be any type that implements `Read`
pub fn extract_mappings(mut from: impl Read) -> Fallible<Mappings> {
    // start by unpacking rabcdasm
    let rabcdasm = RabcdasmBinaries::unpack()?;

    // create a temporary directory
    let dir = tempdir()?;
    let swf = dir.path().join("client.swf");

    // copy the swf contents
    let swf = dir.path().join("client.swf");
    let mut file = File::create(&swf)?;
    copy(&mut from, &mut file)?;

    // close the file and source to release any resources
    drop(file);
    drop(from);

    info!("Extracting mappings from {}", swf.display());

    // disassemble the game
    let abc = rabcdasm.abcexport(&swf)?;
    let asm = rabcdasm.rabcdasm(&abc)?;

    // extract the info
    let rc4 = extract_rc4(&asm)?;
    info!("RC4 key: {}", &rc4);

    let packets = extract_packets(&asm)?;
    info!("Mapped {} packet types", packets.len());

    Ok(Mappings::new(packets, &rc4)?)
}
