pub mod group;

pub use group::group_by_type;
pub use crate::parser::ParsedMessage;

pub fn export_to_csv(path: &str, messages: &[ParsedMessage]) -> Result<(), Box<dyn std::error::Error>> {
    if messages.is_empty() {
        return Ok(());
    }

    let mut writer = csv::Writer::from_path(path)?;

    let headers: Vec<String> = messages[0].fields.iter().map(|(name, _)| name.clone()).collect();
    writer.write_record(&headers)?;

    for msg in messages {
        let row: Vec<String> = msg.fields.iter().map(|(_, val)| val.clone()).collect();
        writer.write_record(&row)?;
    }

    writer.flush()?;
    println!("✅ Wrote {} rows to '{}'", messages.len(), path);
    Ok(())
}
