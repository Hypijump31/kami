//! Multi-tool pipeline execution — chains tool outputs as inputs.
//!
//! A pipeline is an ordered sequence of steps where each step's output
//! can feed into the next step's input via `input_from: "previous"`.

use serde::{Deserialize, Serialize};

use kami_types::ToolId;

/// A single step in a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Tool to execute.
    pub tool: ToolId,
    /// Explicit JSON input (used if `input_from` is `None`).
    #[serde(default)]
    pub input: Option<serde_json::Value>,
    /// Source of input: `"previous"` uses the prior step's output.
    #[serde(default)]
    pub input_from: Option<String>,
}

/// Definition of a multi-step pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    /// Ordered list of execution steps.
    pub steps: Vec<PipelineStep>,
}

/// Result of a single pipeline step.
#[derive(Debug, Clone)]
pub struct StepResult {
    /// The tool that was executed.
    pub tool: ToolId,
    /// JSON output string.
    pub output: String,
    /// Whether execution succeeded.
    pub success: bool,
    /// Execution time in milliseconds.
    pub duration_ms: u64,
}

/// Result of a complete pipeline execution.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Results for each step, in order.
    pub steps: Vec<StepResult>,
    /// Whether all steps completed successfully.
    pub success: bool,
}

/// Errors specific to pipeline execution.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    /// The pipeline has no steps.
    #[error("pipeline has no steps")]
    Empty,
    /// A step references `input_from` but has no previous step.
    #[error("step {index} uses input_from but is the first step")]
    NoPreviousStep { index: usize },
    /// A step failed, halting the pipeline.
    #[error("step {index} (tool '{tool}') failed: {reason}")]
    StepFailed {
        index: usize,
        tool: String,
        reason: String,
    },
    /// Runtime error during execution.
    #[error("runtime error at step {index}: {source}")]
    Runtime {
        index: usize,
        #[source]
        source: crate::error::RuntimeError,
    },
}

/// Executes a pipeline against the given runtime.
///
/// Runs steps sequentially. If a step sets `input_from: "previous"`,
/// it receives the output of the prior step as its input.
///
/// # Errors
/// Returns `PipelineError` if any step fails or the definition is invalid.
pub async fn execute_pipeline(
    runtime: &crate::orchestrator::KamiRuntime,
    definition: &PipelineDefinition,
) -> Result<PipelineResult, PipelineError> {
    if definition.steps.is_empty() {
        return Err(PipelineError::Empty);
    }
    let mut results: Vec<StepResult> = Vec::with_capacity(definition.steps.len());

    for (index, step) in definition.steps.iter().enumerate() {
        let input = resolve_step_input(step, index, results.last())?;
        let exec = runtime
            .execute(&step.tool, &input)
            .await
            .map_err(|e| PipelineError::Runtime { index, source: e })?;
        if !exec.success {
            return Err(PipelineError::StepFailed {
                index,
                tool: step.tool.to_string(),
                reason: exec.content.clone(),
            });
        }
        results.push(StepResult {
            tool: step.tool.clone(),
            output: exec.content,
            success: exec.success,
            duration_ms: exec.duration_ms,
        });
    }

    Ok(PipelineResult {
        success: true,
        steps: results,
    })
}

/// Resolves the input for a step, either explicit or from previous output.
pub fn resolve_step_input(
    step: &PipelineStep,
    index: usize,
    previous: Option<&StepResult>,
) -> Result<String, PipelineError> {
    if let Some(ref source) = step.input_from {
        if source == "previous" {
            let prev = previous.ok_or(PipelineError::NoPreviousStep { index })?;
            return Ok(prev.output.clone());
        }
    }
    match &step.input {
        Some(v) => Ok(v.to_string()),
        None => Ok("{}".to_string()),
    }
}
