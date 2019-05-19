#![allow(dead_code)]

pub mod avm2;
pub mod extractor;

use failure::Fallible;
use rotmg_data::Parameters;
use rotmg_extractor::ParsedClient;
use rotmg_networking::mappings::Mappings;
use serde::Serialize;
use std::fs::read;
use std::io::{stdin, stdout, Read};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opts {
    /// The path to the game client to extract data from. If a single hyphen (-)
    /// is passed for this argument, data will instead be read from standard in
    /// until the stream is closed, and then data will be extracted from that.
    client: PathBuf,

    /// Whether to extract packet IDs and RC4 keys from the client. If no flags
    /// are set, this one is enabled by default.
    #[structopt(long)]
    mappings: bool,

    /// Whether to extract client parameters (version, port, etc) from the game
    /// client.
    #[structopt(long)]
    parameters: bool,
}

#[derive(Default, Serialize)]
struct ExtractedData {
    mappings: Option<Mappings>,
    parameters: Option<Parameters>,
}

fn main() -> Fallible<()> {
    let mut opts: Opts = Opts::from_args();

    if !(opts.mappings || opts.parameters) {
        opts.mappings = true;
    }

    let parsed = if opts.client.as_os_str() != "-" {
        // read the file at the given path
        ParsedClient::new(&read(opts.client)?)
    } else {
        // read from stdin
        let mut buffer = Vec::new();
        stdin().read_to_end(&mut buffer)?;
        ParsedClient::new(&buffer)
    }?;

    // create an empty output container, then populate it with data depending on
    // which flags are set
    let mut data = ExtractedData::default();

    if opts.mappings {
        data.mappings = Some(parsed.extract_mappings()?);
    }

    if opts.parameters {
        data.parameters = Some(parsed.extract_parameters()?);
    }

    // write the results to stdout
    serde_json::to_writer(stdout(), &data)?;

    Ok(())
}
