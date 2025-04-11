use crate::errors::{Result, WallaceError}; // Use custom Result and Error
use crate::messages::registry::{FieldDef, MessageDef, MessageRegistry};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom}; // Import Seek and SeekFrom

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub log_type: u16,
    pub name: String,
    pub fields: Vec<(String, String)>,
}

// Helper function to get byte size of a type string
// Note: This needs to be kept in sync with parse_fields logic
fn get_type_size(type_str: &str) -> Option<usize> {
    match type_str {
        "Q" | "q" | "d" => Some(8),
        "I" | "i" | "f" => Some(4),
        "H" | "h" => Some(2),
        "B" | "b" => Some(1),
        s if s.chars().all(|c| c == 'c') => Some(s.len()),
        s if s.ends_with("s") => s[..s.len() - 1].parse::<usize>().ok(),
        s if s.chars().all(|c| c == 'B') => Some(s.len()),
        s if s.chars().all(|c| c == 'b') => Some(s.len()),
        _ => None, // Unknown or unsupported type
    }
}

pub fn extract_messages<R: Read>(
    reader: &mut R,
    registry: &MessageRegistry,
) -> Result<(Vec<ParsedMessage>, Vec<String>, usize)> {
    // Update return type

    let mut messages = Vec::new();
    let mut warnings = Vec::new();
    let mut total_skipped_fields = 0;
    // Read header, convert potential io::Error to WallaceError::Io
    let _header = reader.read_i32::<LittleEndian>()?;

    loop {
        let log_type = match reader.read_u16::<LittleEndian>() {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break, // End of file is expected
            Err(e) => return Err(WallaceError::Io(e)),                        // Other IO errors
        };

        // Read length, convert potential io::Error
        let length = reader.read_u16::<LittleEndian>()?;
        let mut payload = vec![0u8; length as usize];
        // Read payload, convert potential io::Error
        reader.read_exact(&mut payload)?;

        let log_type_key = log_type.to_string();
        if let Some(def) = registry.get(&log_type_key) {
            // parse_fields now returns Result<(...), WallaceError>
            match parse_fields(&payload, &def.fields) {
                Ok((fields, field_warnings, skipped_fields)) => {
                    total_skipped_fields += skipped_fields;
                    messages.push(ParsedMessage {
                        log_type,
                        name: def.name.clone(),
                        fields,
                    });
                    for warn in field_warnings {
                        warnings.push(format!("log_type {} ({}): {}", log_type, def.name, warn));
                    }
                }
                Err(e) => {
                    // Propagate parsing errors, adding context
                    return Err(WallaceError::ParsingError {
                        log_type,
                        name: def.name.clone(),
                        reason: e.to_string(),
                    });
                }
            }
        } else {
            // Optionally add a warning for unknown message types if desired
            // warnings.push(format!("Unknown message type ID: {}", log_type));
            // Or return an error:
            // return Err(WallaceError::UnknownMessageType(log_type));
            // Current behavior is to silently skip, which we'll keep for now.
        }
    }

    Ok((messages, warnings, total_skipped_fields))
}

pub fn parse_fields(
    payload: &[u8],
    field_defs: &[FieldDef],
) -> Result<(Vec<(String, String)>, Vec<String>, usize)> {
    // Update return type
    let mut skip_count = 0;
    let mut cursor = std::io::Cursor::new(payload);
    let mut parsed = Vec::new();
    let mut warnings = Vec::new();

    for field in field_defs {
        let current_pos = cursor.position(); // Get position before read/skip

        // --- Refactored Skipping Logic ---
        if matches!(field.name.as_str(), "TRASH" | "PADDING" | "RESERVED") {
            if let Some(size_to_skip) = get_type_size(&field.r#type) {
                // Check if skipping exceeds payload bounds
                if current_pos + size_to_skip as u64 > payload.len() as u64 {
                    warnings.push(format!(
                        "Attempted to skip field '{}' ({}) of size {}, but it exceeds payload length {}. Skipping remaining {} bytes.",
                        field.name, field.r#type, size_to_skip, payload.len(), payload.len() as u64 - current_pos
                    ));
                    cursor.seek(SeekFrom::End(0))?; // Seek to end
                } else {
                    cursor.seek(SeekFrom::Current(size_to_skip as i64))?;
                }
                skip_count += 1;
                continue; // Move to the next field
            } else {
                // If size cannot be determined for a TRASH field, add warning and stop parsing this message?
                warnings.push(format!(
                    "Cannot determine size for skippable field '{}' with unknown type '{}'. Parsing may be incorrect.",
                    field.name, field.r#type
                ));
                // Depending on desired strictness, could return an error here:
                // return Err(WallaceError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown type for skippable field")));
                continue; // Or just continue, risking misaligned reads
            }
        }
        // --- End Refactored Skipping Logic ---

        // Check if reading the next field would exceed bounds *before* attempting to read
        // This requires knowing the size beforehand.
        let field_size = get_type_size(&field.r#type);

        if let Some(size) = field_size {
            if current_pos + size as u64 > payload.len() as u64 {
                warnings.push(format!(
                    "Attempted to read field '{}' ({}) of size {}, but it exceeds payload length {}. Stopping parse for this message.",
                    field.name, field.r#type, size, payload.len()
                 ));
                // Return potentially partial results and let caller decide? Or error out?
                // For now, let's stop parsing this message's fields.
                break;
            }
        } else if !matches!(field.r#type.as_str(), "c" if field.name == "FILE_CONTENTS") {
            // If size is unknown (and not the special FILE_CONTENTS case), we can't safely proceed.
            warnings.push(format!(
                "Cannot determine size for field '{}' with unknown type '{}'. Stopping parse for this message.",
                field.name, field.r#type
             ));
            break;
        }

        // Proceed with reading the field value
        let val = match field.r#type.as_str() {
            "Q" => cursor.read_u64::<LittleEndian>()?.to_string(),
            "q" => cursor.read_i64::<LittleEndian>()?.to_string(),
            "I" => cursor.read_u32::<LittleEndian>()?.to_string(),
            "H" => cursor.read_u16::<LittleEndian>()?.to_string(),
            "B" => cursor.read_u8()?.to_string(),
            "b" => cursor.read_i8()?.to_string(),
            "i" => cursor.read_i32::<LittleEndian>()?.to_string(),
            "h" => cursor.read_i16::<LittleEndian>()?.to_string(),
            "f" => cursor.read_f32::<LittleEndian>()?.to_string(),
            "d" => cursor.read_f64::<LittleEndian>()?.to_string(),
            // Handle variable length 'c' type (assumes it reads to end of payload)
            // This is potentially fragile if other fields follow FILE_CONTENTS.
            // The JSON definition should ideally only use this for the *last* field.
            "c" if field.name == "FILE_CONTENTS" => {
                let mut buf = Vec::new();
                cursor.read_to_end(&mut buf)?;
                String::from_utf8_lossy(&buf)
                    .trim_end_matches('\0')
                    .to_string()
            }
            // Fixed length string
            s if s.chars().all(|c| c == 'c') => {
                let len = s.len(); // Size already checked above
                let mut buf = vec![0u8; len];
                cursor.read_exact(&mut buf)?;
                String::from_utf8_lossy(&buf)
                    .trim_end_matches('\0')
                    .to_string()
            }
            // String with explicit length (e.g., "10s") - less common, maybe remove?
            // Size check was done above if get_type_size supports it.
            s if s.ends_with("s") => {
                if let Some(len) = get_type_size(s) {
                    let mut buf = vec![0u8; len];
                    cursor.read_exact(&mut buf)?;
                    String::from_utf8_lossy(&buf)
                        .trim_end_matches('\0')
                        .to_string()
                } else {
                    // Should not happen if get_type_size is consistent
                    warnings.push(format!(
                        "Internal error: Could not get size for type '{}' in field '{}'",
                        s, field.name
                    ));
                    "[error]".to_string()
                }
            }
            // Fixed length byte array (hex output)
            s if s.chars().all(|c| c == 'B') || s.chars().all(|c| c == 'b') => {
                let count = s.len(); // Size already checked above
                let mut buf = vec![0u8; count];
                cursor.read_exact(&mut buf)?;
                buf.iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            // Unknown type (size check failed earlier or wasn't possible)
            unknown => {
                // This branch might be less likely now due to earlier size checks
                warnings.push(format!(
                    "Unsupported type '{}' encountered for field '{}'",
                    unknown, field.name
                ));
                "[unsupported]".to_string()
            }
        };
        parsed.push((field.name.clone(), val));
    }

    // Check if cursor consumed the whole payload (optional, for strictness)
    if cursor.position() < payload.len() as u64 {
        warnings.push(format!(
            "Payload not fully consumed. Expected length {}, read {}. Remaining {} bytes.",
            payload.len(),
            cursor.position(),
            payload.len() as u64 - cursor.position()
        ));
    } else if cursor.position() > payload.len() as u64 {
        // This shouldn't happen with the bounds checks, but as a safeguard:
        warnings.push(format!(
            "Read past payload end. Expected length {}, read {}.",
            payload.len(),
            cursor.position()
        ));
    }

    Ok((parsed, warnings, skip_count)) // Ensure this is the last statement before the closing brace
} // <-- Ensure this closing brace matches the function definition

// Placeholder for parsing-related logic.
