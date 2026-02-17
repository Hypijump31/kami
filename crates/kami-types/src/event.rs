//! Domain events for observability.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::tool::ToolId;

/// Domain events emitted during tool lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    /// A tool has been installed.
    ToolInstalled {
        tool_id: ToolId,
        timestamp: SystemTime,
    },
    /// A tool execution has started.
    ExecutionStarted {
        tool_id: ToolId,
        timestamp: SystemTime,
    },
    /// A tool execution has completed.
    ExecutionCompleted {
        tool_id: ToolId,
        duration_ms: u64,
        success: bool,
        timestamp: SystemTime,
    },
    /// A tool has been removed.
    ToolRemoved {
        tool_id: ToolId,
        timestamp: SystemTime,
    },
}

impl DomainEvent {
    /// Creates a tool-installed event.
    pub fn tool_installed(tool_id: ToolId) -> Self {
        Self::ToolInstalled {
            tool_id,
            timestamp: SystemTime::now(),
        }
    }

    /// Creates an execution-started event.
    pub fn execution_started(tool_id: ToolId) -> Self {
        Self::ExecutionStarted {
            tool_id,
            timestamp: SystemTime::now(),
        }
    }

    /// Creates an execution-completed event.
    pub fn execution_completed(
        tool_id: ToolId,
        duration_ms: u64,
        success: bool,
    ) -> Self {
        Self::ExecutionCompleted {
            tool_id,
            duration_ms,
            success,
            timestamp: SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_creation() {
        let id = ToolId::new("dev.test.tool").unwrap();
        let event = DomainEvent::tool_installed(id);
        match &event {
            DomainEvent::ToolInstalled { tool_id, .. } => {
                assert_eq!(tool_id.as_str(), "dev.test.tool");
            }
            _ => panic!("unexpected event variant"),
        }
    }
}
