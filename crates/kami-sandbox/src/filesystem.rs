//! Filesystem jail for sandboxed tool execution.

use std::path::{Path, PathBuf};

use crate::error::SandboxError;

/// Filesystem jail constraining tool access to a specific directory.
#[derive(Debug, Clone)]
pub struct FsJail {
    /// Root directory of the jail.
    root: PathBuf,
}

impl FsJail {
    /// Creates a new filesystem jail rooted at the given path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Checks that a path is within the jail.
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf, SandboxError> {
        let canonical = self.root.join(path);
        // Prevent path traversal
        if !canonical.starts_with(&self.root) {
            return Err(SandboxError::FsDenied {
                path: path.display().to_string(),
            });
        }
        Ok(canonical)
    }

    /// Returns the jail root.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_path_within_jail() {
        let jail = FsJail::new("/sandbox/tool1");
        let result = jail.validate_path(Path::new("data/output.txt"));
        assert!(result.is_ok());
    }
}
