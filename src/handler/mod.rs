// file_io/mod.rs
use bzip2::read::BzDecoder;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub fn open_file<P: AsRef<Path>>(path: P) -> io::Result<Box<dyn Read>> {
    let file = File::open(&path)?;
    match path.as_ref().extension().and_then(|s| s.to_str()) {
        Some("bz2") => Ok(Box::new(BzDecoder::new(file))),
        _ => Ok(Box::new(file)),
    }
}
