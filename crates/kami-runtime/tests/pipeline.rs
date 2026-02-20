//! Tests for the pipeline module.

use kami_runtime::pipeline::{
    resolve_step_input, PipelineDefinition, PipelineError, PipelineStep, StepResult,
};
use kami_types::ToolId;

fn step(tool: &str, input: Option<&str>, from: Option<&str>) -> PipelineStep {
    PipelineStep {
        tool: ToolId::new(tool).expect("valid tool id"),
        input: input.map(|s| serde_json::from_str(s).expect("valid json")),
        input_from: from.map(String::from),
    }
}

#[test]
fn empty_pipeline_detected() {
    let def = PipelineDefinition { steps: vec![] };
    assert!(def.steps.is_empty());
}

#[test]
fn resolve_explicit_input() {
    let s = step("dev.a.b", Some(r#"{"key":"val"}"#), None);
    let result = resolve_step_input(&s, 0, None).unwrap();
    assert!(result.contains("key"));
}

#[test]
fn resolve_previous_input() {
    let s = step("dev.a.b", None, Some("previous"));
    let prev = StepResult {
        tool: ToolId::new("dev.x.y").expect("valid"),
        output: r#"{"from":"prev"}"#.to_string(),
        success: true,
        duration_ms: 10,
    };
    let result = resolve_step_input(&s, 1, Some(&prev)).unwrap();
    assert_eq!(result, r#"{"from":"prev"}"#);
}

#[test]
fn resolve_previous_on_first_step_errors() {
    let s = step("dev.a.b", None, Some("previous"));
    let result = resolve_step_input(&s, 0, None);
    assert!(result.is_err());
}

#[test]
fn resolve_no_input_defaults_to_empty_object() {
    let s = step("dev.a.b", None, None);
    let result = resolve_step_input(&s, 0, None).unwrap();
    assert_eq!(result, "{}");
}

#[test]
fn pipeline_definition_serde_roundtrip() {
    let def = PipelineDefinition {
        steps: vec![step("dev.a.b", Some(r#""hello""#), None)],
    };
    let json = serde_json::to_string(&def).expect("should serialize");
    let back: PipelineDefinition = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(back.steps.len(), 1);
}

#[test]
fn resolve_unknown_source_falls_through_to_explicit() {
    let s = step("dev.a.b", Some(r#"{"x":1}"#), Some("unknown_source"));
    let result = resolve_step_input(&s, 0, None).unwrap();
    assert!(result.contains("x"));
}

#[test]
fn pipeline_error_empty_display() {
    let e = PipelineError::Empty;
    assert_eq!(e.to_string(), "pipeline has no steps");
}

#[test]
fn pipeline_error_no_previous_display() {
    let e = PipelineError::NoPreviousStep { index: 0 };
    assert!(e.to_string().contains("first step"));
}

#[test]
fn pipeline_error_step_failed_display() {
    let e = PipelineError::StepFailed {
        index: 1,
        tool: "my-tool".into(),
        reason: "crash".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("my-tool"));
    assert!(msg.contains("crash"));
}
