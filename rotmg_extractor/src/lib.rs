//! Utilities to automatically extract mappings from the official ROTMG client

#![deny(missing_docs)]
#![deny(bare_trait_objects)]

mod avm2;

use failure::Fallible;
use failure_derive::Fail;
use lazy_static::lazy_static;
use regex::bytes::{Regex, RegexBuilder};
use rotmg_networking::mappings::RC4_LEN;
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
}

lazy_static! {
    static ref RC4_PATTERN: Regex =
        RegexBuilder::new(&format!(r"rc4.([a-fA-F0-9]{{{}}})", RC4_LEN * 2))
            .dot_matches_new_line(true)
            .build()
            .unwrap();
}

impl ParsedClient {
    /// Parse the given client and store the results in a `ClientExtractor` for
    /// further processing
    pub fn new(client: &'static [u8]) -> Fallible<Self> {
        Ok(Self {
            parsed: parse_movie(client).map(|(_extra, movie)| movie)?,
        })
    }

    /// Get the first DoAbc tag from the client
    fn abc(&self) -> Result<&DoAbc, ExtractionError> {
        self.parsed
            .tags
            .iter()
            .filter_map(|t| match t {
                Tag::DoAbc(abc) => Some(abc),
                _ => None,
            })
            .nth(0) // TODO: error when more than one DoAbc tag is present?
            .ok_or(ExtractionError::NoBytecodeFound)
    }

    /// Extract RC4 keys from this client, in hex form
    pub fn extract_rc4(&self) -> Fallible<&str> {
        // get the bytecode
        let abc = self.abc()?;

        // use regex to pick out the RC4 key
        let hex_rc4: &[u8] = RC4_PATTERN
            .captures(&abc.data[..])
            .ok_or(ExtractionError::NoRC4Found)?
            .get(1)
            .unwrap()
            .as_bytes();

        // convert it to a UTF-8 string
        from_utf8(hex_rc4).map_err(|e| e.into())
    }
}
