use crate::messages::registry::{MessageRegistry, MessageDef, FieldDef};
use byteorder::{ReadBytesExt, LittleEndian};
use std::collections::HashMap;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub log_type: u16,
    pub name: String,
    pub fields: Vec<(String, String)>,
}

pub fn extract_messages<R: Read>(
    reader: &mut R,
    registry: &MessageRegistry,
) -> Result<Vec<ParsedMessage>, std::io::Error> {
    let mut messages = Vec::new();
    let _header = reader.read_i32::<LittleEndian>()?;

    loop {
        let log_type = match reader.read_u16::<LittleEndian>() {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        };

        let length = reader.read_u16::<LittleEndian>()?;
        let mut payload = vec![0u8; length as usize];
        reader.read_exact(&mut payload)?;

        let log_type_key = log_type.to_string();
        if let Some(def) = registry.get(&log_type_key) {
            let fields = parse_fields(&payload, &def.fields)?;
            messages.push(ParsedMessage {
                log_type,
                name: def.name.clone(),
                fields,
            });
        }
    }

    Ok(messages)
}

fn parse_fields(payload: &[u8], field_defs: &[FieldDef]) -> Result<Vec<(String, String)>, std::io::Error> {
    let mut cursor = std::io::Cursor::new(payload);
    let mut parsed = Vec::new();

    for field in field_defs {
        let val = match field.r#type.as_str() {
            "Q" => cursor.read_u64::<LittleEndian>()?.to_string(),
            "I" => cursor.read_u32::<LittleEndian>()?.to_string(),
            "H" => cursor.read_u16::<LittleEndian>()?.to_string(),
            "B" => cursor.read_u8()?.to_string(),
            "i" => cursor.read_i32::<LittleEndian>()?.to_string(),
            "h" => cursor.read_i16::<LittleEndian>()?.to_string(),
            "f" => cursor.read_f32::<LittleEndian>()?.to_string(),
            "d" => cursor.read_f64::<LittleEndian>()?.to_string(),
            s if s.ends_with("s") => {
                let len = s[..s.len()-1].parse::<usize>().unwrap_or(0);
                let mut buf = vec![0u8; len];
                cursor.read_exact(&mut buf)?;
                String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string()
            },
            unknown => format!("[unsupported: {}]", unknown),
        };
        parsed.push((field.name.clone(), val));
    }

    Ok(parsed)
}
