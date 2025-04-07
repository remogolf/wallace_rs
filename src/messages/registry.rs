// messages/registry.rs
use std::collections::HashMap;
use serde::Deserialize;

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

use std::fs::File;
use std::io::BufReader;

pub fn load_message_registry(path: &str) -> Result<MessageRegistry, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let registry: MessageRegistry = serde_json::from_reader(reader)?;
    Ok(registry)
}
