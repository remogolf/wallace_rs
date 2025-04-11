pub mod group;

use crate::errors::Result; // Use custom Result
pub use crate::parser::ParsedMessage;
pub use group::group_by_type;

pub fn export_to_csv(path: &str, messages: &[ParsedMessage]) -> Result<()> {
    // Update return type
    if messages.is_empty() {
        return Ok(());
    }

    let mut writer = csv::Writer::from_path(path)?; // csv::Error automatically converted by #[from]

    // Handle case where message might have no fields (unlikely but possible)
    let headers: Vec<String> = messages
        .get(0)
        .map(|msg| msg.fields.iter().map(|(name, _)| name.clone()).collect())
        .unwrap_or_else(Vec::new);

    // Only write headers if there are any
    if !headers.is_empty() {
        writer.write_record(&headers)?; // csv::Error automatically converted
    }

    for msg in messages {
        let row: Vec<String> = msg.fields.iter().map(|(_, val)| val.clone()).collect();
        // Only write row if headers were written (i.e., fields exist)
        if !headers.is_empty() {
            writer.write_record(&row)?; // csv::Error automatically converted
        }
    }

    writer.flush()?; // io::Error automatically converted
    println!("✅ Wrote {} rows to '{}'", messages.len(), path);
    Ok(())
}
