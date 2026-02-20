//! Shared types for tool execution results and the executor trait.

use async_trait::async_trait;
use wasmtime::component::Component;

use kami_types::SecurityConfig;

use crate::error::RuntimeError;

/// Result of a tool execution.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Output content from the tool.
    pub content: String,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Whether execution succeeded.
    pub success: bool,
    /// Fuel consumed during execution.
    pub fuel_consumed: u64,
}

/// Trait for executing compiled WASM components.
///
/// Implementations apply the full isolation pipeline:
/// sandbox, resource limits, epoch timeout.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Executes a compiled component with the given input and security policy.
    async fn execute(
        &self,
        component: &Component,
        input: &str,
        security: &SecurityConfig,
    ) -> Result<ExecutionResult, RuntimeError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_result_clone_preserves_fields() {
        let r = ExecutionResult {
            content: "ok".into(),
            duration_ms: 42,
            success: true,
            fuel_consumed: 1000,
        };
        let c = r.clone();
        assert_eq!(c.content, "ok");
        assert_eq!(c.duration_ms, 42);
        assert!(c.success);
        assert_eq!(c.fuel_consumed, 1000);
    }

    #[test]
    fn execution_result_debug_format() {
        let r = ExecutionResult {
            content: "x".into(),
            duration_ms: 0,
            success: false,
            fuel_consumed: 0,
        };
        let dbg = format!("{r:?}");
        assert!(dbg.contains("ExecutionResult"));
    }
}
