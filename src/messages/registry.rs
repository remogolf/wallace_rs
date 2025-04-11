// messages/registry.rs
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
}

pub type MessageRegistry = HashMap<String, MessageDef>;

use crate::errors::Result; // Use custom Result
use std::fs::File;
use std::io::BufReader;

pub fn load_message_registry(path: &str) -> Result<MessageRegistry> {
    // Update return type
    let file = File::open(path)?; // io::Error automatically converted by #[from] in WallaceError
    let reader = BufReader::new(file);
    let registry: MessageRegistry = serde_json::from_reader(reader)?; // serde_json::Error automatically converted
    Ok(registry)
}
