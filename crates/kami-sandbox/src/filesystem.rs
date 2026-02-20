//! Filesystem jail for sandboxed tool execution.
//!
//! Three-layer defence: reject absolute paths, reject `..` components,
//! then verify via canonicalization that the resolved path stays within the
//! jail root (anti-symlink).

use std::path::{Component, Path, PathBuf};

use crate::error::SandboxError;

/// Filesystem jail constraining tool access to a specific directory.
#[derive(Debug, Clone)]
pub struct FsJail {
    /// Root directory of the jail (NOT pre-canonicalized — done lazily).
    root: PathBuf,
}

impl FsJail {
    /// Creates a new filesystem jail rooted at the given path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Validates that `path` is safe and lies within the jail.
    ///
    /// # Errors
    ///
    /// Returns `SandboxError::FsDenied` if:
    /// - `path` is absolute
    /// - `path` contains `..` components (traversal attempt)
    /// - The resolved path escapes the jail root (symlink escape)
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf, SandboxError> {
        // 1. Reject absolute/rooted paths — only relative paths allowed.
        //    `has_root()` catches Unix-style `/foo` on Windows where
        //    `is_absolute()` requires a drive-letter prefix (e.g. `C:\`).
        if path.has_root() || path.is_absolute() {
            return Err(SandboxError::FsDenied {
                path: path.display().to_string(),
            });
        }

        // 2. Reject any ".." component — no parent-dir traversal
        for component in path.components() {
            if matches!(component, Component::ParentDir) {
                return Err(SandboxError::FsDenied {
                    path: path.display().to_string(),
                });
            }
        }

        // 3. Build the full path within the jail
        let full_path = self.root.join(path);

        // 4. If the path exists, canonicalize both sides and verify containment
        //    (protects against symlinks pointing outside the jail)
        if full_path.exists() {
            let canonical_root = self
                .root
                .canonicalize()
                .map_err(|e| SandboxError::FsDenied {
                    path: format!("cannot canonicalize root '{}': {e}", self.root.display()),
                })?;

            let real_path = full_path
                .canonicalize()
                .map_err(|e| SandboxError::FsDenied {
                    path: format!("cannot canonicalize '{}': {e}", full_path.display()),
                })?;

            if !real_path.starts_with(&canonical_root) {
                return Err(SandboxError::FsDenied {
                    path: path.display().to_string(),
                });
            }
        }

        Ok(full_path)
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

    #[test]
    fn reject_parent_traversal() {
        let jail = FsJail::new("/sandbox/tool1");
        let result = jail.validate_path(Path::new("../../../etc/passwd"));
        assert!(result.is_err());
    }

    #[test]
    fn reject_embedded_parent() {
        let jail = FsJail::new("/sandbox/tool1");
        let result = jail.validate_path(Path::new("data/../../etc/passwd"));
        assert!(result.is_err());
    }

    #[test]
    fn reject_absolute_path() {
        let jail = FsJail::new("/sandbox/tool1");
        let result = jail.validate_path(Path::new("/etc/passwd"));
        assert!(result.is_err());
    }

    #[test]
    fn accept_nested_valid_path() {
        let jail = FsJail::new("/sandbox/tool1");
        let result = jail.validate_path(Path::new("data/subdir/file.txt"));
        assert!(result.is_ok());
        let full = result.unwrap();
        assert!(full.starts_with("/sandbox/tool1"));
    }

    #[test]
    fn reject_single_dot_dot() {
        let jail = FsJail::new("/sandbox/tool1");
        let result = jail.validate_path(Path::new(".."));
        assert!(result.is_err());
    }
}
