// file_io/mod.rs
// Placeholder for file I/O utilities.

use crate::errors::Result; // Use custom Result
use bzip2::read::BzDecoder;
use std::fs::File;
use std::io::Read; // Remove io import, use std::io::Read directly
use std::path::Path;

pub fn open_file<P: AsRef<Path>>(path: P) -> Result<Box<dyn Read>> {
    // Update return type
    let file = File::open(&path)?; // io::Error automatically converted by #[from]
    match path.as_ref().extension().and_then(|s| s.to_str()) {
        Some("bz2") => Ok(Box::new(BzDecoder::new(file))),
        _ => Ok(Box::new(file)),
    }
}
