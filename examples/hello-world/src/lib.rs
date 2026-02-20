//! Hello-world KAMI tool — minimal example.
//!
//! Takes a JSON `{ "name": "..." }` input and returns a greeting.

use kami_guest::kami_tool;
use serde::Deserialize;

kami_tool! {
    name: "dev.kami.hello-world",
    version: "0.1.0",
    description: "Returns a greeting for the given name",
    handler: handle,
}

/// Input schema for the hello-world tool.
#[derive(Deserialize)]
struct Input {
    name: String,
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    let greeting = format!("Hello, {}! Welcome to KAMI.", args.name);
    kami_guest::text_result(&greeting)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_with_name() {
        let result = handle(r#"{"name":"Alice"}"#).unwrap();
        assert!(result.contains("Hello, Alice!"));
    }

    #[test]
    fn missing_name_returns_error() {
        let result = handle(r#"{}"#);
        assert!(result.is_err());
    }
}
