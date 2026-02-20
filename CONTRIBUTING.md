# Contributing to KAMI

Thank you for your interest in contributing to KAMI! This document explains
how to set up a development environment, the coding standards we enforce,
and the workflow for submitting changes.

---

## Prerequisites

- **Rust** stable (see [rust-toolchain.toml](rust-toolchain.toml))
- **wasm32-wasip2** target: `rustup target add wasm32-wasip2`
- **Git** ≥ 2.30

## Getting Started

```bash
git clone <repo-url> && cd kami
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

All three commands must succeed before submitting a change.

## Architecture

KAMI follows **Clean Architecture** with four layers.
See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full ADR record.

| Layer | Crates | Rule |
|-------|--------|------|
| Domain | `kami-types`, `kami-protocol`, `kami-registry` | Zero external I/O |
| Application | `kami-engine`, `kami-sandbox`, `kami-runtime` | Traits, no frameworks |
| Adapters | `kami-store-sqlite`, `kami-transport-*`, `kami-config` | Concrete impls |
| Infrastructure | `kami-cli` | Composition root |

Dependencies always point **inward** (Infrastructure → Domain).

## Code Standards

These rules are strictly enforced. PRs that violate them will not merge.

### Mandatory

- **Max 150 lines per file** (including inline tests)
- **Zero `unwrap()` / `expect()` / `panic!()` in production code**
  - Allowed in `#[cfg(test)]` blocks only
- **Zero `#[allow(dead_code)]`** — remove unused code
- **`cargo fmt`** — no custom formatting
- **`cargo clippy -- -D warnings`** — zero warnings
- **`///` doc comment** on every public item

### Error Handling

| Layer | Pattern |
|-------|---------|
| Domain | `KamiError` (manual enum, zero deps) |
| Application | `thiserror` derive |
| Adapters | `thiserror` + `From<T>` |
| CLI | `anyhow` for context |

Never discard error context: `map_err(|_| Error::Unknown)` is forbidden.

### Testing

- Every public function has at least one test
- **AAA** pattern: Arrange, Act, Assert
- Naming: `fn test_<what>_<condition>_<expected>()` or `fn <what>_<scenario>()`
- Inline tests in `#[cfg(test)] mod tests { }` for unit tests
- Integration tests in `crate/tests/*.rs`
- Mocks via traits, never monkey-patching

### Rust Idioms

- Prefer `&str` over `String` in function parameters
- Prefer `impl Into<T>` for ergonomic constructors
- Use `Arc<dyn Trait>` for shared dependencies, never `Rc`
- Use `Vec::with_capacity(n)` when the size is known

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(runtime): add LRU cache eviction
fix(sandbox): canonicalize paths in FsJail
refactor(engine): extract linker setup to module
test(store): add query filter integration tests
docs: update CONTRIBUTING.md
```

Scope should match the crate name without the `kami-` prefix.

## Pull Request Checklist

- [ ] `cargo build` passes
- [ ] `cargo test` — all tests pass
- [ ] `cargo clippy --all-targets -- -D warnings` — clean
- [ ] `cargo fmt --check` — formatted
- [ ] No file exceeds 150 lines
- [ ] New public items have `///` doc comments
- [ ] New functionality has tests

## Security

If you discover a security vulnerability, **do not open a public issue**.
See [docs/SECURITY.md](docs/SECURITY.md) for responsible disclosure.

## License

By contributing, you agree that your contributions will be licensed under
the same terms as the project.
