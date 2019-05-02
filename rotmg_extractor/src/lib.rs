//! Utilities to automatically extract mappings from the official ROTMG client

#![deny(missing_docs)]
#![deny(bare_trait_objects)]

pub(crate) mod avm2;

use avm2::abcfile::AbcFile;
use avm2::traits::TraitSlotValue;
use avm2::Parse;
use bimap::BiHashMap;
use failure::Fallible;
use failure_derive::Fail;
use rotmg_networking::mappings::Mappings;
use rotmg_networking::packets::PacketType;
use std::collections::HashMap;
use std::io::Cursor;
use swf_parser::parsers::movie::parse_movie;
use swf_tree::{Movie, Tag};

/// An error that may occur when extracting client data
#[derive(Debug, Fail)]
pub enum ExtractionError {
    /// No AS3 bytecode was found in the client
    #[fail(display = "No bytecode found in parsed client data")]
    NoBytecodeFound,

    /// The RC4 keys could not be found
    #[fail(display = "Could not find RC4 keys")]
    NoRC4Found,

    /// The GameServerConnection class could not be found
    #[fail(display = "Could not find packet IDs")]
    NoPacketsFound,

    /// Error parsing the SWF
    #[fail(display = "An internal parser error occurred: {}", _0)]
    ParserError(String),
}

/// A struct representing a parsed game client which can then be used to extract
/// assets and mappings. Assets/mappings are not extracted until the methods are
/// called.
pub struct ParsedClient {
    _parsed: Movie,
    abc: AbcFile,
}

impl ParsedClient {
    /// Parse the given game client
    pub fn new(client: &[u8]) -> Fallible<Self> {
        let (_, parsed) =
            parse_movie(client).map_err(|e| ExtractionError::ParserError(e.to_string()))?;

        let abc_tag = parsed
            .tags
            .iter()
            .filter_map(|t| match t {
                Tag::DoAbc(abc) => Some(abc),
                _ => None,
            })
            .nth(0)
            .ok_or(ExtractionError::NoBytecodeFound)?;

        let abc = AbcFile::parse_avm2(&mut Cursor::new(&abc_tag.data))?;

        Ok(Self {
            _parsed: parsed,
            abc,
        })
    }

    /// Extract RC4 key from this client, in hex form
    pub fn extract_rc4(&self) -> Fallible<&String> {
        self.abc
            .constants()
            .all_strings()
            .iter()
            .skip_while(|&s| s != "rc4")
            .nth(1)
            .ok_or(ExtractionError::NoRC4Found.into())
    }

    /// Extract packet mappings from this client
    pub fn extract_packets(&self) -> Fallible<BiHashMap<u8, PacketType>> {
        // get GameServerConnection class
        let gsc = self
            .abc
            .classes()
            .filter(|c| c.name.1 == "GameServerConnection")
            .nth(0)
            .ok_or(ExtractionError::NoPacketsFound)?;

        // construct map of unmapped packet names/types
        let mut names = PacketType::get_name_mappings()
            .iter()
            .map(|(&pkt_type, name)| (name.to_lowercase(), pkt_type))
            .collect::<HashMap<_, _>>();

        // construct mappings table
        let packets = gsc
            .consts
            .into_iter()
            .filter_map(|t| match t.value {
                TraitSlotValue::Int(i) => Some((t.name.1.to_lowercase().replace('_', ""), i)),
                _ => None,
            })
            .filter_map(|(name, id)| names.remove(&name).map(|pkt_type| (id as u8, pkt_type)))
            .collect();

        Ok(packets)
    }

    /// Extract a set of mappings from the game client, including RC4 key and
    /// packet IDs
    pub fn extract_mappings(&self) -> Fallible<Mappings> {
        let rc4 = self.extract_rc4()?;
        let packets = self.extract_packets()?;

        Ok(Mappings::new(packets, rc4)?)
    }
}
