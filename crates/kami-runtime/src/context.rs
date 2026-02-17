//! Execution context for tool invocations.

use std::collections::HashMap;
use std::time::Duration;

use kami_types::ToolId;

/// Context passed to a tool execution.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The tool being executed.
    pub tool_id: ToolId,
    /// Arguments for the tool.
    pub arguments: HashMap<String, serde_json::Value>,
    /// Execution timeout.
    pub timeout: Duration,
    /// Unique execution ID for tracing.
    pub execution_id: String,
}
