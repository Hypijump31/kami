//! Query types for tool repository lookups.

use serde::{Deserialize, Serialize};

/// Filtering and pagination for tool queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolQuery {
    /// Filter by name (substring match).
    pub name_filter: Option<String>,
    /// Filter by keyword.
    pub keyword: Option<String>,
    /// Only enabled tools.
    pub enabled_only: bool,
    /// Maximum results to return.
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
}

impl ToolQuery {
    /// Creates a query that matches all tools.
    pub fn all() -> Self {
        Self::default()
    }

    /// Sets the name filter.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name_filter = Some(name.into());
        self
    }

    /// Sets the limit.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_default_query() {
        let q = ToolQuery::all();
        assert!(q.name_filter.is_none());
        assert!(q.keyword.is_none());
        assert!(!q.enabled_only);
        assert!(q.limit.is_none());
        assert!(q.offset.is_none());
    }

    #[test]
    fn with_name_sets_filter() {
        let q = ToolQuery::all().with_name("fetch");
        assert_eq!(q.name_filter.as_deref(), Some("fetch"));
    }

    #[test]
    fn builder_chaining_sets_all_fields() {
        let q = ToolQuery::all().with_name("echo").with_limit(10);
        assert_eq!(q.name_filter.as_deref(), Some("echo"));
        assert_eq!(q.limit, Some(10));
    }
}
