//! Network allow-list enforcement.
//!
//! Supports hostname patterns AND explicit IP addresses.
//! Direct IP connections are blocked by default unless the IP is explicitly
//! listed — this prevents bypassing hostname-based allow-lists.

use std::net::IpAddr;
use std::net::SocketAddr;

/// Checks if a socket address is permitted by the allow list.
///
/// The allow list supports:
/// - Exact hostnames: `"api.github.com"`
/// - Wildcard hostnames: `"*.example.com"`
/// - Explicit IPv4/IPv6 addresses: `"93.184.216.34"`
///
/// **Security:** Raw IP connections only succeed if the IP is explicitly
/// listed. Hostnames are checked separately by `is_host_allowed`.
/// This prevents bypassing hostname allow-lists via direct IP connections.
pub fn is_addr_allowed(addr: &SocketAddr, allow_list: &[String]) -> bool {
    let ip = addr.ip();
    let ip_str = ip.to_string();

    allow_list.iter().any(|pattern| {
        // Explicit IP match (e.g. "93.184.216.34" or "::1")
        if let Ok(allowed_ip) = pattern.parse::<IpAddr>() {
            return ip == allowed_ip;
        }
        // Hostname pattern — IP strings never match hostname patterns
        // (prevents "*.example.com" from matching "93.184.216.34")
        is_ip_str_matching_hostname_pattern(&ip_str, pattern)
    })
}

/// Checks if a hostname string matches any pattern in the allow list.
///
/// Supports:
/// - Exact matches: `"api.github.com"`
/// - Wildcard subdomain: `"*.example.com"` (matches `sub.example.com`)
pub fn is_host_allowed(host: &str, allow_list: &[String]) -> bool {
    allow_list.iter().any(|pattern| {
        if let Some(suffix) = pattern.strip_prefix("*.") {
            host == suffix || host.ends_with(&format!(".{suffix}"))
        } else {
            host == pattern
        }
    })
}

/// Returns false for IP strings against hostname patterns.
///
/// An IP address string (e.g. `"93.184.216.34"`) can never match a hostname
/// pattern (e.g. `"*.example.com"` or `"api.github.com"`). This function
/// enforces that separation to prevent IP-based bypasses.
fn is_ip_str_matching_hostname_pattern(ip_str: &str, pattern: &str) -> bool {
    // Only allow if the pattern IS a literal IP that matches exactly
    // (already handled above via IpAddr parse). For hostname patterns,
    // an IP string should never match.
    ip_str == pattern
}

/// Validates that all entries in a network allow list are well-formed.
///
/// # Errors
///
/// Returns an error string if any pattern is empty or a malformed wildcard.
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
