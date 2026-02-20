//! Unified error types for the KAMI domain layer.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Classification of domain errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// Resource not found.
    NotFound,
    /// Permission denied by capability checker.
    PermissionDenied,
    /// Invalid input data.
    InvalidInput,
    /// Operation timed out.
    Timeout,
    /// Resource limit exceeded (memory, fuel, etc.).
    ResourceExhausted,
    /// Internal error.
    Internal,
}

/// Domain-level error with structured context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KamiError {
    /// The kind of error.
    pub kind: ErrorKind,
    /// Human-readable error message.
    pub message: String,
    /// Optional additional context.
    pub context: Option<String>,
}

impl KamiError {
    /// Creates a new `KamiError`.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: None,
        }
    }

    /// Adds context to the error.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Creates a not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    /// Creates a permission-denied error.
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::PermissionDenied, message)
    }

    /// Creates an invalid-input error.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidInput, message)
    }
}

impl fmt::Display for KamiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.kind, self.message)?;
        if let Some(ctx) = &self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for KamiError {}

/// Transforms technical errors into user-actionable diagnostics.
///
/// Implementors provide optional `hint` (cause explanation) and `fix`
/// (concrete remediation step) for each error variant.
pub trait DiagnosticError {
    /// A human-readable explanation of the likely cause.
    fn hint(&self) -> Option<String> {
        None
    }
    /// A concrete fix the user can apply (e.g. a config change).
    fn fix(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_without_context() {
        let err = KamiError::new(ErrorKind::NotFound, "tool not found");
        assert_eq!(err.to_string(), "[NotFound] tool not found");
    }

    #[test]
    fn error_display_with_context() {
        let err = KamiError::not_found("tool not found").with_context("id: dev.example.fetch");
        assert!(err.to_string().contains("dev.example.fetch"));
    }

    #[test]
    fn error_serialization_roundtrip() {
        let err = KamiError::new(ErrorKind::PermissionDenied, "access denied");
        let json = serde_json::to_string(&err).expect("serialize");
        let back: KamiError = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.kind, ErrorKind::PermissionDenied);
        assert_eq!(back.message, "access denied");
    }

    #[test]
    fn not_found_constructor() {
        let err = KamiError::not_found("missing");
        assert_eq!(err.kind, ErrorKind::NotFound);
    }

    #[test]
    fn permission_denied_constructor() {
        let err = KamiError::permission_denied("nope");
        assert_eq!(err.kind, ErrorKind::PermissionDenied);
    }

    #[test]
    fn invalid_input_constructor() {
        let err = KamiError::invalid_input("bad data");
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn diagnostic_trait_defaults_to_none() {
        struct Dummy;
        impl DiagnosticError for Dummy {}
        let d = Dummy;
        assert!(d.hint().is_none());
        assert!(d.fix().is_none());
    }
}
