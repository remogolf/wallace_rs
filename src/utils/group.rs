use crate::parser::ParsedMessage;
use std::collections::HashMap;

pub fn group_by_type<'a>(messages: &'a [ParsedMessage]) -> HashMap<String, Vec<ParsedMessage>> {
    let mut grouped: HashMap<String, Vec<ParsedMessage>> = HashMap::new();
    for msg in messages {
        grouped
            .entry(msg.name.clone())
            .or_default()
            .push(msg.clone());
    }
    grouped
}
