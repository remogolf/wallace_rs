mod file_io;
mod parser;
mod utils;
mod messages {
    pub mod registry;
    pub use registry::{load_message_registry, FieldDef, MessageDef, MessageRegistry};
}

use file_io::open_file;
use messages::load_message_registry;
use parser::extract_messages;
use utils::{group_by_type, export_to_csv};
use std::fs;
use std::path::Path;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dat_file_path = "example.dat";
    let json_registry_path = "messages.json";

    let registry = load_message_registry(json_registry_path)?;
    let mut reader = open_file(dat_file_path)?;

    let all_messages = extract_messages(&mut reader, &registry)?;

    let grouped = group_by_type(all_messages);

    let output_dir = Path::new("output");
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    for (name, group) in &grouped {
        let file_path = output_dir.join(format!("{}.csv", name));
        export_to_csv(file_path.to_str().unwrap(), group)?;
    }


    println!("✅ Exported {} message types to CSV.", grouped.len());
    Ok(())
}