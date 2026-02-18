//! Declarative macros for KAMI guest tools.
//!
//! Provides `kami_tool!` to generate the handler wiring and metadata
//! for tools targeting the KAMI runtime.

/// Generates the KAMI tool entry points from a handler function.
///
/// This macro creates:
/// - A `__kami_run` function that delegates to your handler
/// - A `__kami_describe` function that returns tool metadata JSON
///
/// When compiling to WASM with `wit-bindgen`, these are wired to the
/// Component Model exports. For native testing, they serve as the
/// canonical entry points.
///
/// # Usage
///
/// ```ignore
/// use kami_guest::kami_tool;
///
/// kami_tool! {
///     name: "dev.example.my-tool",
///     version: "1.0.0",
///     description: "Does something useful",
///     handler: my_handler,
/// }
///
/// fn my_handler(input: &str) -> Result<String, String> {
///     let args: serde_json::Value = serde_json::from_str(input)
///         .map_err(|e| e.to_string())?;
///     Ok(format!("processed: {}", args))
/// }
/// ```
#[macro_export]
macro_rules! kami_tool {
    (
        name: $name:expr,
        version: $version:expr,
        description: $desc:expr,
        handler: $handler:ident $(,)?
    ) => {
        /// Entry point: execute the tool with JSON input.
        pub fn __kami_run(input: &str) -> Result<String, String> {
            $handler(input)
        }

        /// Entry point: return tool metadata as JSON.
        pub fn __kami_describe() -> String {
            let meta = $crate::abi::ToolMetadata {
                name: $name.to_string(),
                description: $desc.to_string(),
                version: $version.to_string(),
            };
            meta.to_json()
        }
    };
}

#[cfg(test)]
mod tests {
    fn sample_handler(input: &str) -> Result<String, String> {
        Ok(format!("echo: {input}"))
    }

    kami_tool! {
        name: "dev.test.sample",
        version: "0.1.0",
        description: "Test tool",
        handler: sample_handler,
    }

    #[test]
    fn macro_generates_run() {
        let result = __kami_run("hello");
        assert_eq!(result, Ok("echo: hello".to_string()));
    }

    #[test]
    fn macro_generates_describe() {
        let json = __kami_describe();
        assert!(json.contains("dev.test.sample"));
        assert!(json.contains("0.1.0"));
    }
}
