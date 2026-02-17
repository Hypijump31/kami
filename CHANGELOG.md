# Changelog

All notable changes to KAMI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Workspace setup with 11 crates following hexagonal architecture
- `kami-types`: Domain types (ToolId, ToolManifest, Capability, KamiError, DomainEvent)
- `kami-protocol`: JSON-RPC 2.0 types and MCP method definitions
- `kami-registry`: Abstract `ToolRepository` trait (port)
- `kami-engine`: Wasmtime instance manager, linker, memory stats (stubs)
- `kami-sandbox`: Capability checker, filesystem jail, network allow-list
- `kami-runtime`: Execution context, executor trait, priority scheduler (stubs)
- `kami-store-sqlite`: SQLite repository adapter (stub)
- `kami-transport-stdio`: Transport error types (stub)
- `kami-config`: Layered configuration with figment (defaults + TOML + env)
- `kami-cli`: CLI with install, run, list, inspect subcommands
- `kami-guest`: Guest SDK module (stub)
- WIT interface definitions (world.wit, tool.wit, host.wit)
- Default configuration file (config/default.toml)
