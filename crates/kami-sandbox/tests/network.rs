//! Integration tests for network allow-list enforcement.

use kami_sandbox::network::{is_addr_allowed, is_host_allowed, validate_allow_list};
use std::net::SocketAddr;

// --- is_host_allowed ---

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
    assert!(!is_host_allowed("evil.com", &["*.example.com".to_string()]));
}

#[test]
fn empty_list_denies_all() {
    assert!(!is_host_allowed("example.com", &[]));
}

// --- is_addr_allowed ---

#[test]
fn ip_direct_connection_denied_by_default() {
    let addr: SocketAddr = "93.184.216.34:443".parse().unwrap();
    assert!(!is_addr_allowed(&addr, &["*.example.com".to_string()]));
}

#[test]
fn ip_allowed_when_explicitly_listed() {
    let addr: SocketAddr = "93.184.216.34:443".parse().unwrap();
    assert!(is_addr_allowed(&addr, &["93.184.216.34".to_string()]));
}

#[test]
fn ip_denied_when_hostname_only_listed() {
    let addr: SocketAddr = "93.184.216.34:80".parse().unwrap();
    assert!(!is_addr_allowed(&addr, &["example.com".to_string()]));
}

#[test]
fn ipv6_explicitly_allowed() {
    let addr: SocketAddr = "[::1]:8080".parse().unwrap();
    assert!(is_addr_allowed(&addr, &["::1".to_string()]));
}

#[test]
fn ipv6_denied_by_hostname_pattern() {
    let addr: SocketAddr = "[::1]:8080".parse().unwrap();
    assert!(!is_addr_allowed(&addr, &["*.example.com".to_string()]));
}

// --- validate_allow_list ---

#[test]
fn valid_patterns() {
    let patterns = vec![
        "*.example.com".to_string(),
        "api.github.com".to_string(),
        "93.184.216.34".to_string(),
    ];
    assert!(validate_allow_list(&patterns).is_ok());
}

#[test]
fn empty_pattern_rejected() {
    let patterns = vec!["".to_string()];
    assert!(validate_allow_list(&patterns).is_err());
}

#[test]
fn bare_wildcard_rejected() {
    let patterns = vec!["*.".to_string()];
    assert!(validate_allow_list(&patterns).is_err());
}

#[test]
fn wildcard_matches_bare_domain_itself() {
    // *.example.com should match "example.com" (the root)
    assert!(is_host_allowed(
        "example.com",
        &["*.example.com".to_string()]
    ));
}

#[test]
fn wildcard_matches_deep_subdomain() {
    assert!(is_host_allowed(
        "a.b.c.example.com",
        &["*.example.com".to_string()]
    ));
}

#[test]
fn wildcard_rejects_unrelated_domain() {
    assert!(!is_host_allowed(
        "notexample.com",
        &["*.example.com".to_string()]
    ));
    assert!(!is_host_allowed(
        "example.org",
        &["*.example.com".to_string()]
    ));
}

#[test]
fn multiple_patterns_any_match_succeeds() {
    let list = vec!["api.github.com".to_string(), "*.gitlab.com".to_string()];
    assert!(is_host_allowed("api.github.com", &list));
    assert!(is_host_allowed("ci.gitlab.com", &list));
    assert!(!is_host_allowed("evil.net", &list));
}

#[test]
fn mixed_ip_and_hostname_allow_list() {
    let list = vec!["10.0.0.1".to_string(), "*.example.com".to_string()];
    let ip_match: SocketAddr = "10.0.0.1:80".parse().unwrap();
    let ip_miss: SocketAddr = "10.0.0.2:80".parse().unwrap();
    assert!(is_addr_allowed(&ip_match, &list));
    assert!(!is_addr_allowed(&ip_miss, &list));
}

#[test]
fn validate_empty_list_is_ok() {
    assert!(validate_allow_list(&[]).is_ok());
}

#[test]
fn validate_rejects_mixed_with_empty() {
    let list = vec!["api.github.com".to_string(), "".to_string()];
    assert!(validate_allow_list(&list).is_err());
}
