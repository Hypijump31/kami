# KAMI - Prompt d'Initialisation du Projet

## Vision

KAMI est un orchestrateur haute performance permettant à des agents IA d'exécuter des outils tiers de manière **sécurisée** et **isolée** via WebAssembly Component Model et le protocole MCP (Model Context Protocol).

---

## Principes Fondamentaux

| Principe | Implémentation |
|----------|----------------|
| **Isolation** | Chaque outil s'exécute dans un sandbox WASM avec capabilities explicites |
| **Sécurité** | Zero trust par défaut, permissions granulaires (réseau, fichiers) |
| **Performance** | Runtime Wasmtime optimisé, I/O 100% async via tokio |
| **Interopérabilité** | WASI P2, Component Model, protocole MCP (JSON-RPC 2.0) |
| **Scalabilité** | Architecture hexagonale, adapters interchangeables |

---

## Architecture Hexagonale

### Diagramme des Couches

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        🟣 INFRASTRUCTURE                                │
│                            kami-cli                                     │
│                     (Point d'entrée, composition)                       │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          🟢 ADAPTERS                                    │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐  │
│  │ kami-store-     │  │ kami-transport- │  │     kami-config         │  │
│  │    sqlite       │  │     stdio       │  │   (Configuration)       │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         🔶 APPLICATION                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐  │
│  │   kami-engine   │  │  kami-sandbox   │  │    kami-runtime         │  │
│  │   (Wasmtime)    │  │  (Isolation)    │  │   (Orchestration)       │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           🔷 DOMAIN                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐  │
│  │   kami-types    │  │  kami-protocol  │  │    kami-registry        │  │
│  │   (Types purs)  │  │   (MCP Types)   │  │   (Port/Traits)         │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Flux d'Exécution

```
┌─────────────────────────────────────────────────────────────────┐
│                           AI Agent                              │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼ MCP (JSON-RPC 2.0)
┌─────────────────────────────────────────────────────────────────┐
│                     kami-transport-stdio                        │
│                      Protocol Handler                           │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                        kami-runtime                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  Executor   │  │  Scheduler  │  │     Instance Pool       │  │
│  │   (Async)   │  │ (Priorities)│  │    (Warm Starts)        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                         kami-engine                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  Wasmtime   │  │   Linker    │  │   Memory Manager        │  │
│  │  Instance   │  │(Host funcs) │  │   (Limits/Fuel)         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                        kami-sandbox                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ Capability  │  │   Network   │  │    Filesystem           │  │
│  │  Checker    │  │ Allow-List  │  │       Jail              │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    WASM Component (Tool)                        │
│                    Isolated Execution                           │
└─────────────────────────────────────────────────────────────────┘
```

---

## Structure Complète du Workspace

```
kami/
├── Cargo.toml                    # Workspace manifest
├── rust-toolchain.toml           # Rust version pinning
├── .cargo/
│   └── config.toml               # Cargo configuration
│
├── crates/
│   │
│   │── kami-types/               # 🔷 DOMAIN: Types partagés (zero deps)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── tool.rs           # ToolId, ToolManifest, ToolVersion
│   │       ├── capability.rs     # Capability, Permission, ResourceLimit
│   │       ├── error.rs          # KamiError (enum unifié)
│   │       └── event.rs          # DomainEvent (observabilité)
│   │
│   │── kami-protocol/            # 🔷 DOMAIN: Protocole MCP (types only)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── jsonrpc.rs        # Request, Response, Error
│   │       ├── mcp/
│   │       │   ├── mod.rs
│   │       │   ├── tools.rs      # ToolsListRequest, ToolsCallRequest
│   │       │   ├── prompts.rs    # PromptsListRequest
│   │       │   └── resources.rs  # ResourcesReadRequest
│   │       └── schema.rs         # JSON Schema validation
│   │
│   │── kami-registry/            # 🔷 PORT: Interface abstraite catalogue
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── repository.rs     # trait ToolRepository
│   │       └── query.rs          # ToolQuery (filters, pagination)
│   │
│   │── kami-engine/              # 🔶 APPLICATION: Moteur WASM
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs          # EngineError
│   │       ├── instance.rs       # WasmInstance lifecycle
│   │       ├── linker.rs         # Host functions linking
│   │       ├── memory.rs         # Memory management & limits
│   │       └── component.rs      # Component Model support
│   │
│   │── kami-sandbox/             # 🔶 APPLICATION: Isolation & Sécurité
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs          # SandboxError
│   │       ├── capability.rs     # CapabilityChecker trait
│   │       ├── wasi.rs           # WasiCtx builder
│   │       ├── network.rs        # Network allow-list
│   │       └── filesystem.rs     # Filesystem jail
│   │
│   │── kami-runtime/             # 🔶 APPLICATION: Orchestrateur
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs          # RuntimeError
│   │       ├── executor.rs       # ToolExecutor (async)
│   │       ├── scheduler.rs      # Task queue & priorities
│   │       ├── pool.rs           # Instance pool (warm starts)
│   │       └── context.rs        # ExecutionContext
│   │
│   │── kami-store-sqlite/        # 🟢 ADAPTER: SQLite
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── migrations/
│   │       │   └── 001_init.sql
│   │       └── repository.rs     # impl ToolRepository
│   │
│   │── kami-transport-stdio/     # 🟢 ADAPTER: Transport stdio
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── handler.rs
│   │
│   │── kami-config/              # 🟢 ADAPTER: Configuration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── loader.rs         # File + Env + CLI merge
│   │       └── schema.rs
│   │
│   │── kami-cli/                 # 🟣 INFRASTRUCTURE: CLI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── commands/
│   │       │   ├── mod.rs
│   │       │   ├── install.rs
│   │       │   ├── run.rs
│   │       │   ├── list.rs
│   │       │   └── inspect.rs
│   │       └── output.rs
│   │
│   └── kami-guest/               # 📦 SDK: Pour développeurs
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── macros.rs         # #[kami_tool]
│           └── abi.rs
│
├── wit/                          # 📐 Interfaces WIT
│   ├── world.wit
│   ├── tool.wit
│   └── host.wit
│
├── config/
│   └── default.toml              # Config par défaut
│
├── tests/                        # Tests d'intégration
│   └── fixtures/
│       └── minimal.wasm          # Module WASM de test
│
└── docs/
    ├── ARCHITECTURE.md           # ADRs
    ├── PROTOCOL.md               # Spec MCP/JSON-RPC
    └── SECURITY.md               # Threat model
```

---

## Stack Technique Détaillée

### Dépendances par Crate

| Crate | Dépendances | Justification |
|-------|-------------|---------------|
| `kami-types` | `serde`, `serde_json` | Sérialisation types domaine |
| `kami-protocol` | `kami-types`, `serde` | Types MCP, JSON-RPC |
| `kami-registry` | `kami-types`, `async-trait` | Traits abstraits |
| `kami-engine` | `kami-types`, `wasmtime`, `wasmtime-wasi` | Runtime WASM |
| `kami-sandbox` | `kami-types`, `wasmtime-wasi` | Isolation |
| `kami-runtime` | `kami-engine`, `kami-sandbox`, `kami-registry`, `tokio` | Orchestration |
| `kami-store-sqlite` | `kami-registry`, `rusqlite`, `tokio` | Persistence |
| `kami-transport-stdio` | `kami-protocol`, `kami-runtime`, `tokio` | I/O |
| `kami-config` | `kami-types`, `serde`, `toml`, `figment` | Configuration |
| `kami-cli` | `kami-runtime`, `kami-config`, `clap`, `anyhow`, `tracing` | Interface |
| `kami-guest` | `wit-bindgen` | SDK développeur |

### Versions Recommandées

```toml
[workspace.dependencies]
# WASM Runtime
wasmtime = "27"
wasmtime-wasi = "27"

# Async
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Config
figment = { version = "0.10", features = ["toml", "env"] }

# CLI
clap = { version = "4", features = ["derive"] }

# Errors
thiserror = "2"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Database
rusqlite = { version = "0.32", features = ["bundled"] }

# SDK
wit-bindgen = "0.36"
```

---

## Patterns d'Implémentation

### Inversion de Dépendances

```rust
// kami-registry/src/repository.rs
// Le TRAIT est dans le crate "port"

use async_trait::async_trait;
use kami_types::{Tool, ToolId, ToolQuery};

#[async_trait]
pub trait ToolRepository: Send + Sync {
    async fn find_by_id(&self, id: &ToolId) -> Result<Option<Tool>, RepositoryError>;
    async fn find_all(&self, query: ToolQuery) -> Result<Vec<Tool>, RepositoryError>;
    async fn insert(&self, tool: &Tool) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &ToolId) -> Result<bool, RepositoryError>;
}
```

```rust
// kami-store-sqlite/src/repository.rs
// L'IMPLÉMENTATION dépend du trait

use kami_registry::{ToolRepository, RepositoryError};

pub struct SqliteToolRepository {
    pool: rusqlite::Connection,
}

#[async_trait]
impl ToolRepository for SqliteToolRepository {
    async fn find_by_id(&self, id: &ToolId) -> Result<Option<Tool>, RepositoryError> {
        // Implementation...
    }
}
```

### Error Handling Stratifié

```rust
// kami-types/src/error.rs - Erreur de domaine

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    InvalidInput,
    Timeout,
    ResourceExhausted,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KamiError {
    pub kind: ErrorKind,
    pub message: String,
    pub context: Option<String>,
}
```

```rust
// kami-engine/src/error.rs - Erreur spécifique

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Failed to compile WASM module: {reason}")]
    Compilation { 
        reason: String, 
        #[source] 
        source: wasmtime::Error 
    },
    
    #[error("Instance trapped: {message}")]
    Trap { message: String },
    
    #[error("Resource limit exceeded: {limit}")]
    ResourceExceeded { limit: String },
}

impl From<EngineError> for KamiError {
    fn from(e: EngineError) -> Self {
        KamiError {
            kind: ErrorKind::Internal,
            message: e.to_string(),
            context: None,
        }
    }
}
```

### Configuration Layered

```rust
// kami-config/src/schema.rs

use std::time::Duration;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KamiConfig {
    pub runtime: RuntimeConfig,
    pub sandbox: SandboxConfig,
    pub registry: RegistryConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
    
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub default_timeout: Duration,
}

fn default_max_concurrent() -> usize { 10 }
fn default_pool_size() -> usize { 5 }
fn default_timeout() -> Duration { Duration::from_secs(30) }
```

---

## Manifeste Outil (tool.toml)

```toml
[tool]
id = "dev.example.fetch-url"
name = "fetch-url"
version = "1.0.0"
authors = ["Your Name <you@example.com>"]
license = "MIT"
wasm = "fetch_url.wasm"

[mcp]
description = "Fetches content from a URL and returns the body"

[[mcp.arguments]]
name = "url"
type = "string"
description = "The URL to fetch"
required = true

[[mcp.arguments]]
name = "method"
type = "string"
description = "HTTP method (GET, POST, etc.)"
required = false
default = "GET"

[security]
# Network permissions
net_allow_list = ["*.example.com", "api.github.com"]

# Filesystem permissions
fs_access = "none"  # none | read-only | sandbox

# Resource limits
max_memory_mb = 64
max_execution_ms = 5000
max_fuel = 1000000

[metadata]
homepage = "https://github.com/example/fetch-url"
repository = "https://github.com/example/fetch-url"
keywords = ["http", "fetch", "network"]
```

---

## CI/CD (GitHub Actions)

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -D warnings

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Format
        run: cargo fmt --all -- --check
      
      - name: Clippy
        run: cargo clippy --all-targets --all-features
      
      - name: Build
        run: cargo build --all-targets
      
      - name: Test
        run: cargo test --all-targets
```

---

## Première Étape : Génération du Squelette

À exécuter pour initialiser le projet :

1. **Créer le workspace** avec tous les `Cargo.toml`
2. **Implémenter `kami-types`** avec les types fondamentaux
3. **Implémenter `kami-engine`** : chargement WASM minimal
4. **Test** : appeler une fonction `run() -> String` exportée

---

## Validation Requise

Avant de passer au code, confirmer :

1. ✅ Structure des crates acceptable ?
2. ✅ Séparation Domain/Application/Adapter claire ?
3. ✅ Stack technique validée ?
4. ✅ Prêt pour la génération du workspace ?
