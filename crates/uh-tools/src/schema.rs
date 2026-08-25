//! Helper to build JSON Schema snippets for tool parameters.
//!
//! Keeps the per-tool spec concise.

use serde_json::{Value, json};

pub fn string_arg(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

pub fn optional_string_arg(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

pub fn bool_arg(description: &str) -> Value {
    json!({ "type": "boolean", "description": description })
}

pub fn int_arg(description: &str) -> Value {
    json!({ "type": "integer", "description": description })
}

pub fn object_schema(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}
