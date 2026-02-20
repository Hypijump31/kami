//! Echo tool — KAMI demo guest component.
//!
//! Implements the `kami:tool/tool` WIT interface.
//! Returns input unchanged as the output.

wit_bindgen::generate!({
    world: "kami-tool",
    path: "../../../wit",
});

use exports::kami::tool::tool::Guest;

struct EchoTool;

impl Guest for EchoTool {
    fn run(input: String) -> Result<String, String> {
        Ok(input)
    }

    fn describe() -> String {
        r#"{"name":"echo","description":"Echoes input back unchanged","version":"1.0.0"}"#.to_string()
    }
}

export!(EchoTool);
