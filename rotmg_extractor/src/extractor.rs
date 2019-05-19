use crate::avm2::abcfile::AbcFile;
use crate::avm2::class::LinkedClass;
use crate::avm2::traits::TraitSlotValue;
use crate::avm2::Parse;
use bimap::BiHashMap;
use failure::Fallible;
use failure_derive::Fail;
use rotmg_data::Parameters;
use rotmg_networking::mappings::Mappings;
use rotmg_networking::packets::PacketType;
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Cursor;
use swf_parser::parsers::movie::parse_movie;
use swf_tree::{Movie, Tag};

/// A required AS3 class wasn't found in the disassembled client
#[derive(Debug, Fail)]
#[fail(display = "Required class was not found in disassembly: {}", _0)]
pub struct ClassNotFound(&'static str);

/// Error parsing the SWF
#[derive(Debug, Fail)]
#[fail(display = "An internal parser error occurred: {}", _0)]
pub struct ParserError(String);

/// No AS3 bytecode was found in the client
#[derive(Debug, Fail)]
#[fail(display = "No bytecode found in parsed client data")]
pub struct NoBytecodeFound;

/// Couldn't find RC4 key trait in client disassembly
#[derive(Debug, Fail)]
#[fail(display = "No RC4 key was found in the client disassembly")]
pub struct NoRC4Found;

/// Couldn't find packet traits in client disassembly
#[derive(Debug, Fail)]
#[fail(display = "No packets were found in the client disassembly")]
pub struct NoPacketsFound;

/// The trait for a required parameter wasn't found
#[derive(Debug, Fail)]
#[fail(display = "A required parameter wasn't found: {}", _0)]
pub struct ParameterNotFound(&'static str);

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
        let (_, parsed) = parse_movie(client).map_err(|e| ParserError(e.to_string()))?;

        let abc_tag = parsed
            .tags
            .iter()
            .filter_map(|t| match t {
                Tag::DoAbc(abc) => Some(abc),
                _ => None,
            })
            .nth(0)
            .ok_or(NoBytecodeFound)?;

        let abc = AbcFile::parse_avm2(&mut Cursor::new(&abc_tag.data))?;

        Ok(Self {
            _parsed: parsed,
            abc,
        })
    }

    /// Get a class with a given name. Package is ignored, only the name of the
    /// class itself is checked.
    fn class(&self, name: &'static str) -> Result<LinkedClass, ClassNotFound> {
        self.abc
            .classes()
            .filter(|c| c.name.1 == name)
            .nth(0)
            .ok_or_else(|| ClassNotFound(name))
    }

    /// Extract RC4 key from this client, in hex form
    pub fn extract_rc4(&self) -> Fallible<&String> {
        self.abc
            .constants()
            .all_strings()
            .iter()
            .skip_while(|&s| s != "rc4")
            .nth(1)
            .ok_or(NoRC4Found.into())
    }

    /// Extract packet mappings from this client
    pub fn extract_packets(&self) -> Fallible<BiHashMap<u8, PacketType>> {
        // get GameServerConnection class
        let gsc = self.class("GameServerConnection")?;

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

    /// Extract game client parameters
    pub fn extract_parameters(&self) -> Fallible<Parameters> {
        let params = self.class("Parameters")?;

        let map = params
            .consts
            .into_iter()
            .map(|t| (t.name.1, t.value))
            .collect::<HashMap<_, _>>();

        let get_param = |name| map.get(name).ok_or(ParameterNotFound(name));

        let version = {
            let build_version = get_param("BUILD_VERSION")?.as_str()?;
            let minor_version = get_param("MINOR_VERSION")?.as_str()?;
            format!("{}.{}", build_version, minor_version)
        };

        let port = get_param("PORT")?.as_int()?.try_into()?;

        let tutorial_gameid = get_param("TUTORIAL_GAMEID")?.as_int()?;

        let nexus_gameid = get_param("NEXUS_GAMEID")?.as_int()?;

        let random_gameid = get_param("RANDOM_REALM_GAMEID")?.as_int()?;

        Ok(Parameters {
            version,
            port,
            tutorial_gameid,
            nexus_gameid,
            random_gameid,
        })
    }
}
