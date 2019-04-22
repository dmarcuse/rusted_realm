//! Utilities to unpack and use rabcdasm

use failure::Fallible;
use failure_derive::Fail;
use libflate::gzip::Decoder;
use log::info;
use std::fs::File;
use std::io::{copy, Cursor, Result as IoResult};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
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

/// An error running one of the rabcdasm binaries
#[derive(Debug, Fail)]
#[fail(
    display = "Error running {} binary - status code {}, stdout {}, stderr {}",
    name, code, stdout, stderr
)]
pub struct RabcdasmError {
    name: &'static str,
    code: ExitStatus,
    stdout: String,
    stderr: String,
}

impl RabcdasmError {
    fn new(name: &'static str, output: Output) -> Self {
        Self {
            name,
            code: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }
}

impl RabcdasmBinaries {
    /// Unpack the embedded rabcdasm binaries to a temporary directory
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
        info!("Unpacked rabcdasm binaries in {}ms", elapsed.as_millis());

        Ok(Self {
            dir,
            abcexport,
            rabcdasm,
        })
    }

    /// Run abcexport on the given swf, returning the path of the ABC file
    pub fn abcexport(&self, swf: &Path) -> Fallible<PathBuf> {
        assert!(swf.is_file(), "abcexport must be run on a file");

        info!("Running abcexport on {}", swf.display());
        let output = Command::new(&self.abcexport).arg(swf).output()?;

        if !output.status.success() {
            // handle unsuccessful execution
            Err(RabcdasmError::new("abcexport", output).into())
        } else {
            // construct path of output
            let mut name = swf.file_stem().unwrap().to_os_string();
            name.push("-0.abc");
            let abc = swf.with_file_name(name);

            // verify that output is present
            assert!(
                abc.is_file(),
                "unexpected condition - abcexport should produce file"
            );

            Ok(abc)
        }
    }

    /// Run rabcdasm on the given ABC file, returning the path to the output
    /// directory
    pub fn rabcdasm(&self, abc: &Path) -> Fallible<PathBuf> {
        assert!(abc.is_file(), "rabcdasm must be run on a file");

        info!("Running rabcdasm on {}", abc.display());
        let output = Command::new(&self.rabcdasm).arg(abc).output()?;

        if !output.status.success() {
            // handle unsuccessful execution
            Err(RabcdasmError::new("rabcdasm", output).into())
        } else {
            // construct path of output
            let asm = abc.with_file_name(abc.file_name().unwrap());

            // verify that output is present
            assert!(
                asm.is_dir(),
                "unexpected condition - rabcdasm should produce directory"
            );

            Ok(asm)
        }
    }
}
