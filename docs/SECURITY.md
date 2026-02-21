<p align="center"><img src="../site/assets/logo.png" alt="KAMI" height="48"></p>

# KAMI - Security Model

> Comprehensive security documentation for auditors and R&D engineers.

---

## Table of Contents

1. [Threat Model](#threat-model)
2. [Defense in Depth](#defense-in-depth)
3. [Capability-Based Security](#capability-based-security)
4. [WASM Sandbox Isolation](#wasm-sandbox-isolation)
5. [Network Security](#network-security)
6. [Filesystem Security](#filesystem-security)
7. [Environment Variable Security](#environment-variable-security)
8. [Resource Limits](#resource-limits)
9. [Security Configuration Validation](#security-configuration-validation)
10. [Attack Surface Analysis](#attack-surface-analysis)
11. [Security Checklist](#security-checklist)

---

## Threat Model

### Assets to Protect

| Asset | Impact if Compromised |
|-------|----------------------|
| Host filesystem | Data theft, malware installation |
| Host network | Lateral movement, data exfiltration |
| Host memory | Arbitrary code execution |
| Environment variables | Secret leakage (API keys, tokens) |
| Other tools in registry | Supply chain attack |
| Host CPU | Denial of service |

### Threat Actors

| Actor | Capability | Mitigation |
|-------|-----------|------------|
| Malicious tool author | Crafted WASM binary | Sandbox isolation, capability deny-all |
| Compromised registry | Tampered tool package | WASM path verification, future: content hashing |
| Buggy tool | Infinite loops, memory leaks | Fuel metering, memory limits, epoch timeout |
| Network attacker | MitM on tool's network calls | Allow-list restricts reachable hosts |

### Trust Boundaries

```
┌──────────────────────────────────────────────────┐
│ TRUSTED                                          │
│  KAMI Host Process                               │
│  ├── kami-cli (user-controlled)                  │
│  ├── kami-runtime (orchestrator)                 │
│  ├── kami-engine (wasmtime)                      │
│  └── kami-sandbox (policy enforcement)           │
├──────────────────────────────────────────────────┤
│ TRUST BOUNDARY ════════════════════════════════  │
├──────────────────────────────────────────────────┤
│ UNTRUSTED                                        │
│  └── Guest WASM Tool (third-party code)          │
│      ├── Cannot access host memory               │
│      ├── Cannot make syscalls directly            │
│      ├── All I/O mediated by WASI + capabilities │
│      └── Resource usage bounded by limits         │
└──────────────────────────────────────────────────┘
```

---

## Defense in Depth

KAMI applies **6 layers of defense**, all active simultaneously:

```
Layer 1: WASM Memory Isolation
  └── Guest cannot read/write host memory (Wasmtime guarantee)

Layer 2: Capability-Based Access Control
  └── Deny-all by default. Explicit grants in tool.toml.

Layer 3: Network Allow-List
  └── Only declared hosts reachable. Wildcard pattern matching.

Layer 4: Filesystem Jail
  └── None / Read-only / Sandboxed directory only.

Layer 5: Resource Limits (Fuel + Memory + Time)
  └── Prevent DoS via instruction budget, memory cap, timeout.

Layer 6: Security Config Validation
  └── Reject misconfigured tools before execution.
```

If any single layer is bypassed, the remaining layers still protect the host.

---

## Capability-Based Security

### Principle

Tools declare what they need. KAMI grants exactly that and nothing more.

### SecurityConfig Structure

```rust
pub struct SecurityConfig {
    /// Network hosts the tool can reach.
    pub net_allow_list: Vec<String>,     // Default: [] (deny-all)

    /// Filesystem access level.
    pub fs_access: FsAccess,             // Default: None

    /// Environment variables the tool can read.
    pub env_allow_list: Vec<String>,     // Default: [] (deny-all)

    /// Resource limits.
    pub limits: ResourceLimits {
        pub max_fuel: u64,               // Default: 1_000_000
        pub max_memory_mb: u32,          // Default: 64
        pub max_execution_ms: u64,       // Default: 5_000
    },
}
```

### CapabilityChecker Trait

```rust
pub trait CapabilityChecker {
    fn can_access_network(&self, host: &str) -> bool;
    fn can_access_fs(&self, access: FsAccess) -> bool;
    fn can_access_env(&self, var_name: &str) -> bool;
}
```

The `DefaultCapabilityChecker` evaluates each request against the tool's SecurityConfig at runtime.

---

## WASM Sandbox Isolation

### Wasmtime Guarantees

| Property | Mechanism |
|----------|-----------|
| Memory isolation | Linear memory, no host pointers accessible |
| No raw syscalls | All I/O via WASI imports (mediated by host) |
| Type safety | Component Model validates types at boundary |
| Deterministic execution | No threading, no shared memory (single-store) |

### WASI P2 Mediation

All guest I/O goes through WASI interfaces that KAMI controls:

```
Guest calls wasi:io/write ──► Host checks capability ──► Allow/Deny
Guest calls wasi:sockets/* ──► socket_addr_check() ──► Allow/Deny
Guest calls wasi:filesystem/* ──► fs jail check ──► Allow/Deny
```

### Store Isolation

Each tool execution gets a **fresh `Store<HostState>`** with:
- Independent linear memory
- Independent fuel counter
- Independent epoch deadline
- Independent WASI context (no shared state between executions)

---

## Network Security

### Allow-List Model

```toml
# tool.toml
[security]
net_allow_list = ["api.github.com", "*.example.com"]
```

- **Empty list** (default): all network access denied
- **Exact match**: `"api.github.com"` matches only that host
- **Wildcard prefix**: `"*.example.com"` matches `foo.example.com`, `bar.baz.example.com`
- **Invalid patterns rejected**: empty strings, patterns without `*` prefix dot

### Implementation

Network control is enforced via `socket_addr_check` callback on the WASI context:

```rust
// In build_wasi_ctx():
builder.socket_addr_check(move |addr, _addr_use| {
    let host = addr.to_string();
    check_network_access(&host, &allow_list)
});
```

This intercepts ALL outbound network connections at the socket level.

### Pattern Matching Rules

| Pattern | Matches | Does NOT Match |
|---------|---------|----------------|
| `api.github.com` | `api.github.com` | `github.com`, `evil.api.github.com` |
| `*.github.com` | `api.github.com`, `raw.github.com` | `github.com` |
| `*.*.com` | (rejected — invalid pattern) | — |

---

## Filesystem Security

### Access Levels

| Level | `fs_access` | Behavior |
|-------|-------------|----------|
| **None** | `"none"` | No filesystem access. Default. |
| **Read-Only** | `"read-only"` | Read files in jail directory only. |
| **Sandbox** | `"sandbox"` | Read + write in jail directory only. |

### Jail Directory

When fs_access is not `none`, a jail root is established:
- All file paths are validated to stay within the jail
- Path traversal (`../`) is detected and blocked
- Symlinks outside jail are not followed

### Implementation

```rust
pub fn validate_path(path: &Path, jail_root: &Path) -> Result<(), SandboxError> {
    let canonical = path.canonicalize()?;
    if !canonical.starts_with(jail_root) {
        return Err(SandboxError::PathTraversal { ... });
    }
    Ok(())
}
```

---

## Environment Variable Security

### Allow-List Model

```toml
[security]
env_allow_list = ["API_KEY", "HOME"]
```

- **Empty list** (default): no environment variables visible to guest
- Only explicitly listed variables are passed to the WASI context
- All other host env vars are invisible to the tool

### Implementation

```rust
// In build_wasi_ctx():
for var in &security.env_allow_list {
    if let Ok(value) = std::env::var(var) {
        builder.env(var, &value);
    }
}
// Variables not in the list are simply never added to the context
```

---

## Resource Limits

### Fuel Metering

Fuel is an **instruction budget**. Each WASM instruction consumes fuel. When fuel runs out, the store traps.

```
default: 1,000,000 fuel units
configured via: security.limits.max_fuel
enforcement: store.set_fuel(fuel)
trap: wasmtime::Trap::OutOfFuel
```

### Memory Limits

Memory is capped at the store level via `StoreLimits`:

```
default: 64 MB
configured via: security.limits.max_memory_mb
enforcement: StoreLimitsBuilder::new().memory_size(bytes).trap_on_grow_failure(true)
trap: memory.grow returns -1, then Trap on access
```

### Execution Timeout

Dual-layer timeout using epoch interruption + tokio timeout:

```
default: 5,000 ms
configured via: security.limits.max_execution_ms

Layer 1 (cooperative): Engine epoch incremented after timeout_ms
  → Store set to trap after 1 epoch tick
  → Background tokio::spawn sleeps, then increments epoch

Layer 2 (safety net): tokio::time::timeout at timeout_ms + 500ms
  → Catches cases where epoch check doesn't trigger quickly enough
```

### Limits Summary

| Resource | Default | Enforcement | Trap Behavior |
|----------|---------|-------------|---------------|
| CPU (fuel) | 1M instructions | `store.set_fuel()` | `Trap::OutOfFuel` |
| Memory | 64 MB | `StoreLimits` | `Trap` on grow failure |
| Time | 5000 ms | Epoch interruption | `Trap::Interrupt` |
| Concurrency | 4 parallel | Semaphore | Blocks until permit available |

---

## Security Configuration Validation

`validate_security_config()` is called BEFORE any resource allocation:

| Check | Error |
|-------|-------|
| `max_fuel == 0` | `InvalidConfig: max_fuel must be > 0` |
| `max_memory_mb == 0` | `InvalidConfig: max_memory_mb must be > 0` |
| `max_execution_ms == 0` | `InvalidConfig: max_execution_ms must be > 0` |
| Empty pattern in net_allow_list | `InvalidConfig: empty network pattern` |
| Invalid wildcard pattern | `InvalidConfig: invalid pattern` |

This prevents misconfigured tools from bypassing resource limits (e.g. fuel=0 means unlimited).

---

## Attack Surface Analysis

### External Inputs

| Input | Source | Validation |
|-------|--------|------------|
| JSON-RPC requests | stdin | `serde_json::from_str` (reject malformed) |
| tool.toml manifests | filesystem | `parse_tool_manifest` with field validation |
| .wasm binaries | filesystem | Wasmtime compilation validates WASM format |
| Tool input (arguments) | AI agent via MCP | Passed as string to guest (guest validates) |

### Internal Attack Vectors

| Vector | Mitigation |
|--------|------------|
| Guest reads host memory | Impossible (WASM linear memory isolation) |
| Guest makes raw syscalls | Impossible (WASI mediation layer) |
| Guest infinite loop | Fuel metering + epoch timeout |
| Guest allocates unbounded memory | StoreLimits trap |
| Guest connects to arbitrary hosts | socket_addr_check allow-list |
| Guest reads sensitive env vars | env_allow_list filtering |
| Guest writes outside jail | Path validation + canonicalization |
| Malformed JSON-RPC crash | serde_json error handling, no panic paths |
| SQL injection via tool ID | Parameterized queries in SQLite adapter |

---

## Security Checklist

### For Tool Authors

- [ ] Declare minimal `net_allow_list` — only hosts you actually need
- [ ] Use `fs_access = "none"` unless file I/O is required
- [ ] Declare only needed env vars in `env_allow_list`
- [ ] Set reasonable `max_memory_mb` for your workload
- [ ] Test with default resource limits before increasing

### For KAMI Operators

- [ ] Review tool.toml security section before `kami install`
- [ ] Use `kami inspect <tool-id>` to audit installed tools
- [ ] Set `--concurrency` limit appropriate for your hardware
- [ ] Monitor fuel consumption in execution logs
- [ ] Keep KAMI and Wasmtime updated for security patches

### For KAMI Developers

- [ ] Zero `unwrap()` — all errors handled via `Result`
- [ ] Zero `panic!()` — deterministic behavior
- [ ] No `unsafe` blocks in KAMI codebase
- [ ] All SQL uses parameterized queries
- [ ] All network I/O goes through capability checker
- [ ] All file I/O goes through jail validation
- [ ] Security config validated before any resource allocation
