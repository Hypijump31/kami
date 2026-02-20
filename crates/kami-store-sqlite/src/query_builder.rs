//! SQL query builder for `find_all` operations.

use kami_registry::ToolQuery;

use crate::repository_impl::COLS;

/// Builds the SQL and parameters for `find_all`.
pub(crate) fn build_find_all_query(
    query: &ToolQuery,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut sql = format!("SELECT {COLS} FROM tools WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    if query.enabled_only {
        sql.push_str(" AND enabled = 1");
    }
    if let Some(ref name) = query.name_filter {
        sql.push_str(" AND name LIKE ?");
        params.push(Box::new(format!("%{name}%")));
    }
    sql.push_str(" ORDER BY name ASC");
    if let Some(limit) = query.limit {
        sql.push_str(" LIMIT ?");
        params.push(Box::new(limit as i64));
    }
    if let Some(offset) = query.offset {
        sql.push_str(" OFFSET ?");
        params.push(Box::new(offset as i64));
    }
    (sql, params)
}
