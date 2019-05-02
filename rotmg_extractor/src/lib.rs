//! Utilities to automatically extract mappings from the official ROTMG client

#![deny(missing_docs)]
#![deny(bare_trait_objects)]

pub(crate) mod avm2;

use avm2::abcfile::AbcFile;
use avm2::Parse;
use failure::Fallible;
use failure_derive::Fail;
use lazy_static::lazy_static;
use regex::bytes::{Regex, RegexBuilder};
use rotmg_networking::mappings::RC4_LEN;
use std::io::Cursor;
use std::str::from_utf8;
use swf_parser::parsers::movie::parse_movie;
use swf_tree::tags::DoAbc;
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
}

/// A struct representing a parsed game client which can then be used to extract
/// assets and mappings. Assets/mappings are not extracted until the methods are
/// called.
pub struct ParsedClient {
    parsed: Movie,
    abc: AbcFile,
}

impl ParsedClient {
    /// Parse the given game client
    pub fn new(client: &'static [u8]) -> Fallible<Self> {
        let (_, parsed) = parse_movie(client)?;

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

        Ok(Self { parsed, abc })
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
}
