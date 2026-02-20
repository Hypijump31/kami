//! JSON-transform KAMI tool — manipulate JSON objects.
//!
//! Supports three actions:
//! - **pick**: Extract specific keys from an object
//! - **flatten**: Flatten nested objects (dot-notation keys)
//! - **count**: Count top-level keys in an object

use kami_guest::kami_tool;
use serde::Deserialize;
use serde_json::{Map, Value};

kami_tool! {
    name: "dev.kami.json-transform",
    version: "0.1.0",
    description: "Transforms JSON data — pick, flatten, or count",
    handler: handle,
}

/// Input schema for the json-transform tool.
#[derive(Deserialize)]
struct Input {
    action: String,
    data: Value,
    #[serde(default)]
    keys: Vec<String>,
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    match args.action.as_str() {
        "pick" => pick(&args.data, &args.keys),
        "flatten" => flatten(&args.data),
        "count" => count(&args.data),
        other => Err(format!("unknown action: {other}")),
    }
}

/// Extracts only the specified keys from an object.
fn pick(data: &Value, keys: &[String]) -> Result<String, String> {
    let obj = data.as_object().ok_or("data must be an object")?;
    let picked: Map<String, Value> = keys
        .iter()
        .filter_map(|k| obj.get(k).map(|v| (k.clone(), v.clone())))
        .collect();
    kami_guest::to_output(&picked)
}

/// Flattens a nested object into dot-notation keys.
fn flatten(data: &Value) -> Result<String, String> {
    let obj = data.as_object().ok_or("data must be an object")?;
    let mut result = Map::new();
    flatten_recursive(obj, String::new(), &mut result);
    kami_guest::to_output(&result)
}

/// Recursively flattens nested objects using dot-separated keys.
fn flatten_recursive(obj: &Map<String, Value>, prefix: String, out: &mut Map<String, Value>) {
    for (key, value) in obj {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        match value {
            Value::Object(nested) => flatten_recursive(nested, full_key, out),
            _ => {
                out.insert(full_key, value.clone());
            }
        }
    }
}

/// Counts the top-level keys in an object.
fn count(data: &Value) -> Result<String, String> {
    let obj = data.as_object().ok_or("data must be an object")?;
    kami_guest::text_result(&format!("{}", obj.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_extracts_keys() {
        let input = r#"{"action":"pick","data":{"a":1,"b":2,"c":3},"keys":["a","c"]}"#;
        let result = handle(input).expect("pick");
        let parsed: Value = serde_json::from_str(&result).expect("json");
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["c"], 3);
        assert!(parsed.get("b").is_none());
    }

    #[test]
    fn flatten_nested_objects() {
        let input = r#"{"action":"flatten","data":{"a":{"b":1},"c":2}}"#;
        let result = handle(input).expect("flatten");
        let parsed: Value = serde_json::from_str(&result).expect("json");
        assert_eq!(parsed["a.b"], 1);
        assert_eq!(parsed["c"], 2);
    }

    #[test]
    fn count_keys() {
        let input = r#"{"action":"count","data":{"x":1,"y":2,"z":3}}"#;
        let result = handle(input).expect("count");
        assert!(result.contains("3"));
    }

    #[test]
    fn unknown_action_returns_error() {
        let input = r#"{"action":"nope","data":{}}"#;
        assert!(handle(input).is_err());
    }

    #[test]
    fn pick_with_non_object_returns_error() {
        let input = r#"{"action":"pick","data":"not-an-object","keys":["a"]}"#;
        assert!(handle(input).is_err());
    }
}
