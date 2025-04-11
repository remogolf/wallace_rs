mod errors; // Add errors module
mod file_io;
mod parser;
mod utils;
use std::io::Write;
mod messages {
    pub mod registry;
    pub use registry::{load_message_registry, FieldDef, MessageDef, MessageRegistry};
}

use crate::errors::{Result, WallaceError};
use clap::{App, Arg};
use file_io::open_file;
use messages::load_message_registry;
use parser::extract_messages;
use std::fs;
use std::path::{Path, PathBuf};
use utils::{export_to_csv, group_by_type};

fn main() -> Result<()> {
    // Update return type
    // --- Clap Argument Parsing ---
    // Define command-line arguments using Clap
    let matches = App::new("Wallace Log Parser")
        .version("0.1.0")
        .author("Cline")
        .about("Parses binary flight logs based on a JSON definition")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input log file path (e.g., example.dat, log.bz2)")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("registry")
                .short("r")
                .long("registry")
                .value_name("JSON_FILE")
                .help("Sets the message definition JSON file path")
                .takes_value(true)
                .default_value("messages.json"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("DIRECTORY")
                .help("Sets the output directory for CSV files")
                .takes_value(true)
                .default_value("output"),
        )
        .get_matches();

    // Extract command-line arguments
    let input_path = matches.value_of("input").unwrap(); // Required, so unwrap is safe
    let registry_path = matches.value_of("registry").unwrap(); // Has default
    let output_path = matches.value_of("output").unwrap(); // Has default

    // --- End Argument Parsing ---

    // --- Load data and process messages ---

    // Load message registry from JSON
    let registry = load_message_registry(registry_path)?;

    // Open the input file (handles bzip2 decompression)
    let mut reader = open_file(input_path)?;

    // Extract messages from the input file
    let (all_messages, warnings, skipped_fields) =
        extract_messages(&mut reader, &registry)?;

    // Group messages by type
    let grouped = group_by_type(&all_messages);

    // Create the output directory if it doesn't exist
    let output_dir = Path::new(output_path);
    // Create the output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?; // io::Error automatically converted by #[from]
    }

    // Export each message group to a CSV file
    for (name, group) in &grouped {
        let file_path = output_dir.join(format!("{}.csv", name));
        // Handle potential path conversion error
        let file_path_str =
            file_path
                .to_str()
                .ok_or_else(|| WallaceError::PathConversionError {
                    path: file_path.clone(),
                })?;
        export_to_csv(file_path_str, group)?;
    }

    // --- Handle warnings ---
    // Check if there are any warnings
    if !warnings.is_empty() {
        // Ensure warnings log is also placed in the specified output directory
        let warnings_path = output_dir.join("warnings.log");
        let mut log_file = std::fs::File::create(&warnings_path)?; // io::Error automatically converted
        for line in &warnings {
            writeln!(log_file, "{}", line)?; // io::Error automatically converted
        }
        println!(
            "⚠️  Wrote {} warnings to '{}'",
            warnings.len(),
            warnings_path.display()
        );
    }

    // --- Print summary of skipped fields ---
    // Check if any ignorable fields were skipped
    if skipped_fields > 0 {
        println!(
            "⏭️  Skipped {} ignorable fields like TRASH, PADDING, RESERVED",
            skipped_fields
        );
    }

    Ok(())
}
