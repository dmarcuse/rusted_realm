//! Utilities to unpack and use rabcdasm

use libflate::gzip::Decoder;
use log::info;
use std::fs::File;
use std::io::{copy, Cursor, Result as IoResult};
use std::path::PathBuf;
use std::time::Instant;
use tar::Archive;
use tempfile::{tempdir, TempDir};

#[cfg(target_os = "linux")]
const RABCDASM: &[u8] = include_bytes!("rabcdasm_linux.tar.gz");

#[cfg(target_os = "windows")]
const RABCDASM: &[u8] = include_bytes!("rabcdasm_windows.tar.gz");

/// Extracted rabcdasm binaries
#[derive(Debug)]
pub struct RabcdasmBinaries {
    dir: TempDir,
    abcexport: PathBuf,
    rabcdasm: PathBuf,
}

impl RabcdasmBinaries {
    /// Unpack the embedded rabcdasm binaries
    pub fn unpack() -> IoResult<RabcdasmBinaries> {
        let start = Instant::now();
        let dir = tempdir()?;

        // open embedded archive
        let mut archive = Archive::new(Decoder::new(Cursor::new(RABCDASM))?);

        // unpack binaries
        for entry in archive.entries()? {
            let mut entry = entry?;

            let path = entry.path()?;
            let destination = dir.path().join(path.file_stem().unwrap());

            // unpack the binary
            info!("Unpacking {} to {}", path.display(), destination.display());

            // open destination file
            let mut file = File::create(destination)?;

            // copy contents
            copy(&mut entry, &mut file)?;
        }

        // resolve the paths for each binary
        let abcexport = dir.path().join("abcexport");
        let rabcdasm = dir.path().join("rabcdasm");

        let elapsed = start.elapsed();
        info!("Unpacked rabcdasm in {}ms", elapsed.as_millis());

        Ok(Self {
            dir,
            abcexport,
            rabcdasm,
        })
    }
}
