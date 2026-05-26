//! Boot image generator for the Finix language runtime.
//!
//! This crate is a minimal placeholder for future VM and native boot image creation.
//! It currently emits a simple stub file that can be extended into a real Finix bytecode
//! image generator in later phases.

use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("finix_boot.img"));

    let mut file = File::create(&output)?;
    file.write_all(b"FINIX_BOOT_IMAGE_V1\n")?;
    file.write_all(b"// Finix boot image stub. Replace with VM bytecode bundle later.\n")?;

    println!("Created boot image stub at {}", output.display());
    Ok(())
}
