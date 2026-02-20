//! Property-based tests for network allow-list enforcement.
//!
//! Uses `proptest` to fuzz hostnames, IP addresses, and patterns —
//! verifying that IP addresses never match hostname wildcard patterns.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use kami_sandbox::network::{is_addr_allowed, is_host_allowed};
use proptest::prelude::*;

/// Strategy producing random IPv4 addresses.
fn ipv4_strategy() -> impl Strategy<Value = Ipv4Addr> {
    (any::<u8>(), any::<u8>(), any::<u8>(), any::<u8>())
        .prop_map(|(a, b, c, d)| Ipv4Addr::new(a, b, c, d))
}

/// Strategy producing wildcard hostname patterns like `*.example.com`.
fn wildcard_pattern_strategy() -> impl Strategy<Value = String> {
    "[a-z]{2,8}\\.[a-z]{2,4}".prop_map(|domain| format!("*.{domain}"))
}

proptest! {
    /// An IP address must never match a wildcard hostname pattern.
    #[test]
    fn ip_never_matches_wildcard(
        ip in ipv4_strategy(),
        pattern in wildcard_pattern_strategy(),
    ) {
        let addr = SocketAddr::V4(SocketAddrV4::new(ip, 8080));
        let allow_list = vec![pattern.clone()];
        prop_assert!(
            !is_addr_allowed(&addr, &allow_list),
            "IP {ip} matched wildcard {pattern}",
        );
    }

    /// An IP address matches only when explicitly listed.
    #[test]
    fn ip_matches_only_when_listed(ip in ipv4_strategy()) {
        let addr = SocketAddr::V4(SocketAddrV4::new(ip, 443));
        let allow_list = vec![ip.to_string()];
        prop_assert!(is_addr_allowed(&addr, &allow_list));
    }

    /// A hostname never matches when the allow list is empty.
    #[test]
    fn empty_list_blocks_all_hosts(host in "[a-z]{3,12}\\.[a-z]{2,4}") {
        let allow_list: Vec<String> = vec![];
        prop_assert!(!is_host_allowed(&host, &allow_list));
    }

    /// Wildcard `*.domain.tld` matches `sub.domain.tld` but not `domain.tld`.
    #[test]
    fn wildcard_matches_subdomain_only(
        sub in "[a-z]{2,8}",
        domain in "[a-z]{2,8}",
        tld in "[a-z]{2,4}",
    ) {
        let pattern = format!("*.{domain}.{tld}");
        let allow_list = vec![pattern];
        let full_host = format!("{sub}.{domain}.{tld}");
        prop_assert!(is_host_allowed(&full_host, &allow_list));
    }

    /// Exact hostname match works.
    #[test]
    fn exact_hostname_match(host in "[a-z]{3,12}\\.[a-z]{2,4}") {
        let allow_list = vec![host.clone()];
        prop_assert!(is_host_allowed(&host, &allow_list));
    }
}
