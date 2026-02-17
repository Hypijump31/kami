//! Network allow-list enforcement.

/// Checks if a host matches any pattern in the allow list.
///
/// Supports exact matches and wildcard patterns (`*.example.com`).
pub fn is_host_allowed(host: &str, allow_list: &[String]) -> bool {
    allow_list.iter().any(|pattern| {
        if let Some(suffix) = pattern.strip_prefix("*.") {
            host == suffix || host.ends_with(&format!(".{suffix}"))
        } else {
            host == pattern
        }
    })
}

/// Validates that all entries in a network allow list are well-formed.
pub fn validate_allow_list(patterns: &[String]) -> Result<(), String> {
    for pattern in patterns {
        if pattern.is_empty() {
            return Err("empty pattern in network allow list".to_string());
        }
        if pattern.starts_with("*.") && pattern.len() <= 2 {
            return Err(format!("invalid wildcard pattern: {pattern}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_host_match() {
        assert!(is_host_allowed(
            "api.github.com",
            &["api.github.com".to_string()]
        ));
    }

    #[test]
    fn wildcard_host_match() {
        assert!(is_host_allowed(
            "sub.example.com",
            &["*.example.com".to_string()]
        ));
    }

    #[test]
    fn host_not_in_list() {
        assert!(!is_host_allowed(
            "evil.com",
            &["*.example.com".to_string()]
        ));
    }

    #[test]
    fn empty_list_denies_all() {
        assert!(!is_host_allowed("example.com", &[]));
    }

    #[test]
    fn valid_patterns() {
        let patterns = vec![
            "*.example.com".to_string(),
            "api.github.com".to_string(),
        ];
        assert!(validate_allow_list(&patterns).is_ok());
    }

    #[test]
    fn empty_pattern_rejected() {
        let patterns = vec!["".to_string()];
        assert!(validate_allow_list(&patterns).is_err());
    }
}
