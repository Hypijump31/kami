# KAMI — Plan d'Action Détaillé

> Document de référence : audit architectural, dette technique, sécurité, roadmap.
> Date : 18 février 2026 | Version : 1.0

---

## Table des Matières

1. [Synthèse Exécutive](#1-synthèse-exécutive)
2. [Sprint 0 — URGENCE : Restaurer la Compilation](#2-sprint-0--urgence--restaurer-la-compilation)
3. [Sprint 1 — Assainissement de la Dette Technique](#3-sprint-1--assainissement-de-la-dette-technique)
4. [Sprint 2 — Sécurité : Combler les Failles](#4-sprint-2--sécurité--combler-les-failles)
5. [Sprint 3 — Couverture de Tests](#5-sprint-3--couverture-de-tests)
6. [Sprint 4 — Observabilité & Diagnostics](#6-sprint-4--observabilité--diagnostics)
7. [Sprint 5 — Productisation](#7-sprint-5--productisation)
8. [Sprint 6 — Developer Experience](#8-sprint-6--developer-experience)
9. [Sprint 7 — Features Avancées](#9-sprint-7--features-avancées)
10. [Sprint 8 — Écosystème & Go-to-Market](#10-sprint-8--écosystème--go-to-market)
11. [Matrice de Risques](#11-matrice-de-risques)
12. [Indicateurs de Succès (KPIs)](#12-indicateurs-de-succès-kpis)
13. [Planning Synthétique](#13-planning-synthétique)

---

## 1. Synthèse Exécutive

KAMI est un orchestrateur WASM/MCP positionné sur un marché en forte croissance (agents AI + isolation sécuritaire). L'architecture hexagonale est solide (11 crates, 4 couches, 7 ADRs), mais le projet présente des dettes bloquantes :

- **Le projet ne compile pas** (erreur syntaxique dans `init.rs`)
- **5 failles de sécurité** dont 2 de sévérité haute
- **Code mort structurel** dans la couche runtime (traits non implémentés)
- **Violation Clean Architecture** dans la couche domaine
- **Zéro CI/CD** malgré la mention en Phase 0
- **Observabilité absente** (`tracing` en dépendance mais aucune instrumentation effective)
- **Modules critiques non testés** (config, orchestrator, resolver)

Ce plan détaille 9 sprints séquentiels avec dépendances, critères d'acceptation, et fichiers impactés.

---

## 2. Sprint 0 — URGENCE : Restaurer la Compilation

> **Durée : 0.5 jour | Priorité : BLOQUANTE**
> **Critère : `cargo build && cargo test && cargo clippy -- -D warnings` = PASS**

### 2.1 Fix `init.rs` — Erreur de syntaxe

**Problème :** `crates/kami-cli/src/commands/init.rs:120` — `}}` non fermé dans un `format!()` qui génère du code Rust avec des accolades échappées (`{{`/`}}`). Le template de `lib.rs` généré utilise des `format!("... {{e}} ...")` qui, dans un contexte de `format!()` parent, crée un conflit d'accolades.

**Fichier :** `crates/kami-cli/src/commands/init.rs`

**Action :**
- Vérifier chaque paire `{{` / `}}` dans les chaînes `format!()` qui génèrent du code Rust
- S'assurer que les blocs `#[cfg(test)] mod tests {{ ... }}` ont leurs accolades correctement échappées
- Le `format!("... {{e}} ...")` à l'intérieur d'un `map_err` dans le code généré produit un `}}` orphelin
- Alternative : utiliser `include_str!()` avec un fichier template séparé au lieu de `format!()` inline

**Critère de validation :**
```bash
cargo build 2>&1 | grep -c "error" # doit être 0
cargo test -p kami-cli              # doit compiler
```

### 2.2 Vérifier les warnings clippy post-fix

**Action :** Après fix compilation, exécuter `cargo clippy -- -D warnings` et corriger tout warning (le rapport mentionne un potentiel `needless_borrows_for_generic_args` dans `cache.rs`).

---

## 3. Sprint 1 — Assainissement de la Dette Technique

> **Durée : 2-3 jours | Priorité : P1**
> **Dépendance : Sprint 0 terminé**

### 3.1 Supprimer le code mort

| Cible | Fichier | Action |
|-------|---------|--------|
| **`ToolExecutor` trait** | `crates/kami-runtime/src/executor.rs:40-46` | Supprimer le trait OU faire que `WasmToolExecutor` l'implémente. Recommandation : **implémenter le trait** en wrappant `execute_component()` pour restaurer le contrat d'abstraction. |
| **`ExecutionContext`** | `crates/kami-runtime/src/context.rs` | Soit l'intégrer comme paramètre de `ToolExecutor::execute()` (design original), soit le supprimer. Recommandation : **l'intégrer** — il porte `execution_id` qui est utile pour la traçabilité. |
| **`PoolConfig`** | `crates/kami-runtime/src/pool.rs` | Fusionner dans `RuntimeConfig` ou l'enrichir pour un vrai pool d'instances. Recommandation : supprimer pour l'instant, réintroduire quand le pool sera implémenté. |
| **`tests/fixtures/`** | `tests/fixtures/` (vide) | Supprimer le répertoire vide ou y placer les `.wasm` et `.toml` de test. |
| **`001_init.sql`** | `crates/kami-store-sqlite/src/migrations/001_init.sql` | Le schéma SQL diffère du code Rust (`migrations.rs`). Soit aligner et utiliser le fichier SQL, soit le supprimer. Recommandation : **supprimer** — les migrations Rust sont la source de vérité. |

**Détail sur `ToolExecutor` + `ExecutionContext` :**

Le design original prévoyait :
```rust
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute(&self, ctx: ExecutionContext) -> Result<ExecutionResult, RuntimeError>;
}
```

Mais `WasmToolExecutor::execute_component()` prend `(&Component, &str, &SecurityConfig)` — un contrat complètement différent. Deux options :

**Option A (recommandée) : Adapter le trait au réel**
```rust
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute(
        &self,
        component: &Component,
        input: &str,
        security: &SecurityConfig,
    ) -> Result<ExecutionResult, RuntimeError>;
}
```
Puis `impl ToolExecutor for WasmToolExecutor` qui délègue à `execute_component`.

**Option B : Enrichir `ExecutionContext`**
Intégrer `component`, `input`, `security` dans `ExecutionContext` et faire que `execute_component` prenne un `ExecutionContext`. Plus cohérent avec le Domain-Driven Design mais demande plus de refactoring.

### 3.2 Corriger la violation Clean Architecture dans `kami-types`

**Problème :** `crates/kami-types/src/manifest.rs` contient :
- Import `use std::path::Path;`
- Fonction `parse_tool_manifest_file()` qui fait du `std::fs::read_to_string()`
- Dépendance `toml = "0.8"` dans `crates/kami-types/Cargo.toml`

C'est de l'I/O dans la couche domaine — violation directe de la Clean Architecture.

**Action (3 étapes) :**

1. **Déplacer `parse_tool_manifest_file()` vers `kami-config`** (ou `kami-registry`)
   - La fonction fait du filesystem I/O → c'est un adapter
   - `kami-config/src/manifest_loader.rs` (nouveau fichier)

2. **Garder `parse_tool_manifest(content: &str)` dans `kami-types`** — c'est de la pure transformation de données, légitime en domaine

3. **Déplacer la dépendance `toml`** de `kami-types` vers `kami-config`
   - `kami-types/Cargo.toml` : retirer `toml`
   - `kami-config/Cargo.toml` : ajouter `toml`
   - Réexporter `parse_tool_manifest` depuis `kami-types` (string → struct) et `parse_tool_manifest_file` depuis `kami-config` (path → struct)

4. **Mettre à jour les imports** dans `kami-cli` qui appellent `parse_tool_manifest_file`

**Fichiers impactés :**
- `crates/kami-types/Cargo.toml`
- `crates/kami-types/src/manifest.rs`
- `crates/kami-types/src/lib.rs`
- `crates/kami-config/Cargo.toml`
- `crates/kami-config/src/lib.rs`
- `crates/kami-config/src/manifest_loader.rs` (nouveau)
- `crates/kami-cli/src/commands/install.rs`
- `crates/kami-cli/src/commands/init.rs` (si référence)

### 3.3 Découper les fichiers > 150 lignes

| Fichier | Lignes | Action de découpage |
|---------|--------|---------------------|
| `kami-store-sqlite/src/repository.rs` | ~386 | Extraire `row_to_tool()` et `OptionalExt` dans `helpers.rs`. Déplacer les tests dans `repository_tests.rs` ou un module `tests/`. |
| `kami-transport-stdio/src/handler.rs` | ~341 | Extraire `build_input_schema()` dans `schema_builder.rs`. Séparer chaque handler (`handle_initialize`, `handle_tools_list`, `handle_tools_call`) dans des sous-modules. |
| `kami-cli/src/commands/init.rs` | ~170 | Extraire les templates (Cargo.toml, tool.toml, lib.rs) dans des fichiers `templates/` ou des `const &str` dans un module `templates.rs`. |
| `kami-types/src/tool.rs` | ~163 | Extraire `impl FromStr for ToolVersion` et `impl Display` dans un module `version.rs`. |
| `kami-types/src/manifest.rs` | ~155 | OK après déplacement de `parse_tool_manifest_file`. |
| `kami-runtime/src/cache.rs` | ~153 | OK (tests inclus). |
| `kami-runtime/src/orchestrator.rs` | ~152 | OK (tests à ajouter). |

### 3.4 Éliminer le pattern `unwrap_or_default()` dangereux

**Fichier :** `crates/kami-store-sqlite/src/repository.rs:235-237`

**Problème :**
```rust
let security: SecurityConfig = serde_json::from_str(&security_json).unwrap_or_default();
let arguments: Vec<ToolArgument> = serde_json::from_str(&args_json).unwrap_or_default();
```

Si les colonnes JSON sont corrompues, l'outil est chargé avec une config par défaut (deny-all). C'est sûr par accident, mais :
- Le comportement est trompeur (l'outil peut ne plus fonctionner sans explication)
- Aucun log d'avertissement
- Impossible de diagnostiquer

**Action :**
```rust
let security: SecurityConfig = serde_json::from_str(&security_json)
    .map_err(|e| {
        tracing::warn!(tool_id = %id_str, error = %e, "corrupt security config, using defaults");
        e
    })
    .unwrap_or_default();
```

Ou mieux : **propager l'erreur** via `RepositoryError::DataCorruption`.

### 3.5 Harmoniser le pattern CLI (factoriser la duplication)

**Problème :** Le pattern suivant est copié-collé dans 6 fichiers de commande :
```rust
let db_path = args.db.clone().unwrap_or_else(output::default_db_path);
let repo = SqliteToolRepository::open(&db_path)?;
let repo = Arc::new(repo);
```

**Action :** Créer `crates/kami-cli/src/shared.rs` :
```rust
pub fn open_repository(db: &Option<String>) -> anyhow::Result<Arc<dyn ToolRepository>> {
    let path = db.clone().unwrap_or_else(output::default_db_path);
    let repo = SqliteToolRepository::open(&path)?;
    Ok(Arc::new(repo))
}

pub fn create_runtime(repo: Arc<dyn ToolRepository>) -> anyhow::Result<KamiRuntime> {
    let config = RuntimeConfig::default();
    KamiRuntime::new(config, repo).map_err(Into::into)
}
```

### 3.6 Adopter `#[tokio::main]` sur `main()`

**Problème :** Chaque commande crée son propre `tokio::runtime::Runtime::new()?.block_on()`. Non idiomatique.

**Action :**
- `main.rs` : `#[tokio::main] async fn main()`
- Chaque commande : `pub async fn execute(args: &XxxArgs) -> anyhow::Result<()>`
- Supprimer tous les `Runtime::new()?.block_on()` dans les commandes

**Fichiers :** `main.rs`, `run.rs`, `exec.rs`, `install.rs`, `list.rs`, `inspect.rs`, `uninstall.rs`, `serve.rs`

---

## 4. Sprint 2 — Sécurité : Combler les Failles

> **Durée : 2-3 jours | Priorité : P0-P1**
> **Dépendance : Sprint 0 terminé (Sprint 1 en parallèle)**

### 4.1 [HAUTE] Path Traversal dans `FsJail`

**Fichier :** `crates/kami-sandbox/src/filesystem.rs:24-29`

**Problème actuel :**
```rust
pub fn validate_path(&self, path: &Path) -> Result<PathBuf, SandboxError> {
    let canonical = self.root.join(path);  // ← PAS canonicalisé !
    if !canonical.starts_with(&self.root) {
        return Err(SandboxError::FsDenied { ... });
    }
    Ok(canonical)
}
```

`Path::join("../../../etc/passwd")` retourne `/sandbox/tool1/../../../etc/passwd`. `starts_with("/sandbox/tool1")` retourne `true` car `starts_with` fait un match **composant par composant** — MAIS `Path::join` ne résout pas les `..`. Donc l'attaque fonctionne si le path est ensuite utilisé tel quel par le filesystem.

De plus, les **symlinks** à l'intérieur du sandbox ne sont pas vérifiés — un tool pourrait créer un symlink vers `/etc/shadow`.

**Action (3 niveaux de défense) :**

```rust
pub fn validate_path(&self, path: &Path) -> Result<PathBuf, SandboxError> {
    // 1. Rejeter les chemins absolus
    if path.is_absolute() {
        return Err(SandboxError::FsDenied {
            path: path.display().to_string(),
        });
    }

    // 2. Rejeter les composants ".."
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(SandboxError::FsDenied {
                path: path.display().to_string(),
            });
        }
    }

    // 3. Canonicaliser le root (résoudre les symlinks du root)
    let canonical_root = self.root.canonicalize()
        .map_err(|e| SandboxError::FsDenied {
            path: format!("cannot canonicalize root: {e}"),
        })?;

    // 4. Construire le chemin final
    let full_path = canonical_root.join(path);

    // 5. Si le fichier existe déjà, vérifier le chemin canonique réel
    //    (protection anti-symlink)
    if full_path.exists() {
        let real_path = full_path.canonicalize()
            .map_err(|e| SandboxError::FsDenied {
                path: format!("cannot canonicalize: {e}"),
            })?;
        if !real_path.starts_with(&canonical_root) {
            return Err(SandboxError::FsDenied {
                path: path.display().to_string(),
            });
        }
    }

    Ok(full_path)
}
```

**Tests à ajouter :**
```rust
#[test] fn reject_parent_traversal();        // "../../../etc/passwd"
#[test] fn reject_absolute_path();            // "/etc/passwd"
#[test] fn reject_embedded_parent();          // "data/../../etc/passwd"
#[test] fn reject_symlink_escape();           // symlink → /etc/shadow (intégration)
#[test] fn accept_nested_valid_path();        // "data/subdir/file.txt"
#[test] fn reject_path_with_null_bytes();     // "data/\0/file.txt"
```

### 4.2 [HAUTE] IP Bypass sur Network Allow-List

**Fichier :** `crates/kami-sandbox/src/wasi.rs:94-101`

**Problème :**
```rust
builder.socket_addr_check(move |addr, _addr_use| {
    let host = addr.ip().to_string();  // "93.184.216.34"
    is_host_allowed(&host, &patterns)  // patterns = ["*.example.com"]
    // → false — bypass !
});
```

Un tool WASM qui connaît l'IP d'un serveur peut se connecter directement, contournant l'allow-list hostname-based.

**Action (double vérification) :**

1. **Résolution DNS inversée** (complexe, lent) — non recommandé
2. **Séparer les patterns IP et hostname** (recommandé) :

```rust
// Dans network.rs :
pub fn is_addr_allowed(addr: &std::net::SocketAddr, allow_list: &[String]) -> bool {
    let ip_str = addr.ip().to_string();

    allow_list.iter().any(|pattern| {
        // Pattern IP (ex: "93.184.216.34", "10.0.0.0/8")
        if let Ok(allowed_ip) = pattern.parse::<std::net::IpAddr>() {
            return addr.ip() == allowed_ip;
        }
        // Pattern CIDR (ex: "10.0.0.0/8") — optionnel, via crate `ipnet`
        // Pattern hostname (ex: "*.example.com") — IP ne matche jamais
        if let Some(suffix) = pattern.strip_prefix("*.") {
            ip_str == suffix || ip_str.ends_with(&format!(".{suffix}"))
        } else {
            ip_str == *pattern
        }
    })
}
```

3. **Ajouter un mode strict** dans `SecurityConfig` :
```toml
[security]
net_mode = "hostname-only"  # "hostname-only" | "ip-and-hostname" | "deny-all"
```

**Si `net_mode = "hostname-only"`**, toute connexion IP directe est refusée. C'est le défaut sécuritaire.

**Tests :**
```rust
#[test] fn ip_direct_connection_denied_by_default();
#[test] fn ip_allowed_when_explicitly_listed();
#[test] fn cidr_range_matching();
```

### 4.3 [MOYENNE] Intégrité WASM — Hash Verification

**Problème :** Aucune vérification d'intégrité des fichiers `.wasm`. Un binaire modifié (supply chain attack) s'exécute sans détection.

**Action :**

1. **Calculer le hash SHA-256 à l'install** :
   - `kami install .` → hash le `.wasm`, stocke dans la DB (nouvelle colonne `wasm_sha256`)
   - Ajouter dans `ToolManifest` (optionnel) : `sha256 = "abc123..."` pour verification tiers

2. **Vérifier le hash à l'exécution** :
   - `kami-runtime/resolver.rs` → après chargement du fichier, comparer le hash avec celui en DB
   - Si mismatch → `RuntimeError::IntegrityViolation`

3. **Commande `kami verify <tool-id>`** :
   - Recalcule le hash et compare avec la DB
   - Affiche OK/FAIL

**Fichiers impactés :**
- `crates/kami-types/src/tool.rs` : ajouter `wasm_sha256: Option<String>` au `ToolManifest`
- `crates/kami-store-sqlite/src/migrations.rs` : migration v2, colonne `wasm_sha256 TEXT`
- `crates/kami-store-sqlite/src/repository.rs` : store/query le hash
- `crates/kami-runtime/src/resolver.rs` : vérifier le hash au chargement
- `crates/kami-cli/src/commands/install.rs` : calculer le hash à l'install
- `crates/kami-cli/src/commands/verify.rs` (nouveau)

**Dépendance :** `sha2 = "0.10"` (standard, audité, maintenu par RustCrypto)

### 4.4 [MOYENNE] Env Allow-List Non Enforced

**Fichier :** `crates/kami-sandbox/src/wasi.rs`

**Problème :** `env_allow_list` est validée par `CapabilityChecker` mais `build_wasi_ctx()` ne filtre pas les variables d'environnement. Les variables passées via `wasi_config.env_vars` sont toutes ajoutées sans vérification.

**Action :**
```rust
// Dans build_wasi_ctx(), après la construction de env_vars :
for (key, value) in &wasi_config.env_vars {
    if !security.env_allow_list.is_empty()
        && !security.env_allow_list.contains(key)
    {
        tracing::warn!(key, "env var blocked by allow-list");
        continue;
    }
    builder.env(key, value);
}
```

**Test :**
```rust
#[test]
fn env_var_blocked_by_allow_list() {
    let security = SecurityConfig {
        env_allow_list: vec!["ALLOWED_VAR".to_string()],
        ..Default::default()
    };
    let wasi_config = WasiConfig {
        env_vars: vec![
            ("ALLOWED_VAR".to_string(), "ok".to_string()),
            ("SECRET_KEY".to_string(), "leaked!".to_string()),
        ],
        ..Default::default()
    };
    // SECRET_KEY ne doit PAS être dans le WasiCtx
}
```

### 4.5 [BASSE] SQL Injection LIMIT/OFFSET

**Fichier :** `crates/kami-store-sqlite/src/repository.rs:109-113`

**Problème :** `LIMIT` et `OFFSET` sont interpolés par `format!()`. Les valeurs sont `u32` donc techniquement safe, mais c'est une mauvaise pratique.

**Action :** Utiliser des paramètres rusqlite :
```rust
sql.push_str(" LIMIT ?");
params.push(Box::new(query.limit as i64));
sql.push_str(" OFFSET ?");
params.push(Box::new(query.offset as i64));
```

---

## 5. Sprint 3 — Couverture de Tests

> **Durée : 2-3 jours | Priorité : P1**
> **Dépendance : Sprint 1 (pour pouvoir tester le code refactoré)**

### 5.1 Tests unitaires manquants — par module

| Module | Fonctions à tester | Type |
|--------|--------------------|------|
| **`kami-config/loader.rs`** | `load_config(None)`, `load_config(Some("path"))`, env vars merge, invalid TOML | Unitaire |
| **`kami-config/schema.rs`** | `KamiConfig::default()`, serde round-trip, validation champs | Unitaire |
| **`kami-runtime/orchestrator.rs`** | `KamiRuntime::new()`, `execute()` avec mock repository | Unitaire + Intégration |
| **`kami-runtime/resolver.rs`** | `resolve()` cache miss, cache hit, tool not found, WASM missing | Unitaire |
| **`kami-runtime/context.rs`** | `ExecutionContext` construction (si conservé) | Unitaire |
| **`kami-runtime/error.rs`** | `From<RuntimeError> for KamiError` | Unitaire |
| **`kami-engine/component.rs`** | `load_component`, `create_linker`, `instantiate_component` | Intégration |
| **`kami-engine/memory.rs`** | `MemoryStats::usage_percent()` | Unitaire |
| **`kami-transport-stdio/server.rs`** | `McpServer::run()` avec mock transport | Intégration |
| **`kami-transport-stdio/error.rs`** | `From<TransportError> for KamiError` | Unitaire |
| **`kami-sandbox/filesystem.rs`** | Path traversal (voir Sprint 2) | Unitaire |

### 5.2 Tests d'intégration end-to-end

Créer `tests/e2e/` avec des scénarios complets :

```
tests/
├── e2e/
│   ├── install_and_run.rs       # kami install . && kami exec <id> '{}'
│   ├── mcp_protocol.rs          # Simuler un client MCP (initialize + tools/list + tools/call)
│   ├── security_deny.rs         # Vérifier que les accès non autorisés sont bloqués
│   └── concurrent_execution.rs  # Exécuter N tools simultanément
├── fixtures/
│   ├── echo/
│   │   ├── tool.toml
│   │   └── echo.wasm
│   └── malicious/
│       ├── tool.toml             # fs_access = "none" mais tente un read
│       └── malicious.wasm
```

### 5.3 Tests de propriétés (fuzzing)

Pour les modules critiques sécurité :
- `FsJail::validate_path()` — fuzzer les paths d'entrée
- `is_host_allowed()` — fuzzer les patterns réseau
- `parse_tool_manifest()` — fuzzer les TOML malformés
- `JsonRpcRequest` parsing — fuzzer les JSON malformés

**Dépendance :** `proptest = "1"` ou `cargo fuzz`

### 5.4 Benchmark de régression

```
benches/
├── compilation.rs    # Temps de compilation WASM
├── execution.rs      # Temps d'exécution d'un tool simple
├── cache.rs          # Hit/miss ratio et latence du cache
└── mcp_roundtrip.rs  # Latence complète d'un appel MCP
```

**Dépendance :** `criterion = "0.5"`

---

## 6. Sprint 4 — Observabilité & Diagnostics

> **Durée : 1-2 jours | Priorité : P1**
> **Dépendance : Sprint 0**

### 6.1 Instrumentation `tracing`

**Problème :** `tracing` est en dépendance dans 5 crates mais seuls quelques `debug!()` et `info!()` sont émis. Aucun span structuré, aucune métrique.

**Action — Instrumenter les 4 paths critiques :**

**a) Executor (`kami-runtime/executor.rs`)**
```rust
#[tracing::instrument(
    skip(self, component),
    fields(fuel_consumed, duration_ms, success)
)]
pub async fn execute_component(&self, ...) { ... }
```

**b) Resolver (`kami-runtime/resolver.rs`)**
```rust
#[tracing::instrument(skip(self), fields(cache_hit))]
pub async fn resolve(&self, id: &ToolId) { ... }
```

**c) MCP Handler (`kami-transport-stdio/handler.rs`)**
```rust
#[tracing::instrument(skip(self, params), fields(method))]
pub async fn dispatch(&self, request: &JsonRpcRequest) { ... }
```

**d) Scheduler (`kami-runtime/scheduler.rs`)**
```rust
#[tracing::instrument(fields(available_permits))]
pub async fn acquire(&self) { ... }
```

### 6.2 Structured Logging en JSON

**Fichier :** `crates/kami-cli/src/main.rs`

```rust
use tracing_subscriber::{fmt, EnvFilter};

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .json()  // JSON output pour agrégation (ELK, Datadog)
        .with_target(true)
        .with_span_events(fmt::format::FmtSpan::CLOSE)
        .init();
}
```

**CLI flag :** `--log-format plain|json` (plain pour dev, json pour prod)

### 6.3 Métriques d'exécution

Ajouter un module `crates/kami-runtime/src/metrics.rs` :

```rust
pub struct ExecutionMetrics {
    pub total_executions: AtomicU64,
    pub successful_executions: AtomicU64,
    pub failed_executions: AtomicU64,
    pub total_fuel_consumed: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub active_executions: AtomicU64,
}
```

Intégrer dans `KamiRuntime` et exposer via une commande `kami status` ou un endpoint MCP custom.

---

## 7. Sprint 5 — Productisation

> **Durée : 3-5 jours | Priorité : P1-P2**
> **Dépendance : Sprint 0 + Sprint 2**

### 7.1 CI/CD Pipeline

**Fichier :** `.github/workflows/ci.yml`

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo doc --no-deps

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-audit
      - run: cargo audit

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-tarpaulin
      - run: cargo tarpaulin --out xml
      - uses: codecov/codecov-action@v4
```

### 7.2 Transport HTTP/SSE

**Nouveau crate :** `crates/kami-transport-http/`

Essentiel pour le déploiement multi-client. Le stdio est limité à 1 connexion.

**Design :**
```
kami-transport-http/
├── Cargo.toml        # axum, tower, tokio
├── src/
│   ├── lib.rs
│   ├── router.rs     # Routes: POST /mcp, GET /health
│   ├── sse.rs        # Server-Sent Events pour notifications
│   ├── auth.rs       # Bearer token validation
│   └── error.rs
```

**Le `McpHandler` existant est réutilisé tel quel** — seul le transport change.

**CLI :** `kami serve --transport http --port 3000 --token <secret>`

### 7.3 Graceful Shutdown

**Problème :** Aucun signal handling. `Ctrl+C` coupe brutalement les exécutions en cours.

**Action :**
```rust
// Dans main.rs ou serve.rs :
tokio::select! {
    result = server.run() => result?,
    _ = tokio::signal::ctrl_c() => {
        info!("shutdown signal received");
        // Attendre les exécutions en cours (drain scheduler)
        runtime.shutdown().await;
    }
}
```

Ajouter `KamiRuntime::shutdown()` qui :
1. Stop d'accepter de nouvelles exécutions
2. Attend les exécutions en cours (avec timeout)
3. Flush les logs

### 7.4 Health & Readiness Probes

Pour le transport HTTP :
- `GET /health` → `200 OK` (le processus vit)
- `GET /ready` → `200 OK` si le runtime est initialisé, la DB connectée

### 7.5 Binary Releases

**Fichier :** `.github/workflows/release.yml`

Cross-compilation pour :
- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Utiliser `cross` ou `cargo-zigbuild` pour les builds Linux/macOS.

Publier sur GitHub Releases avec `gh release create`.

---

## 8. Sprint 6 — Developer Experience

> **Durée : 2-3 jours | Priorité : P2**
> **Dépendance : Sprint 0 + Sprint 1**

### 8.1 Fixer `kami init`

Le scaffolding est une feature DX critique. Après fix de la compilation :
- Tester la génération end-to-end
- S'assurer que le projet généré compile avec `cargo build --target wasm32-wasip2`
- Ajouter un test d'intégration qui exécute `init`, `build`, `install`, `exec`

### 8.2 Watch Mode

**Commande :** `kami dev --watch <tool-dir>`

Surveille les modifications dans le répertoire du tool, recompile automatiquement le WASM, et recharge dans le cache du runtime.

**Implémentation :**
- Dépendance : `notify = "6"` (filesystem watcher)
- À chaque changement : `cargo build --target wasm32-wasip2 --release`
- Invalidation du cache : `runtime.invalidate_cache(&tool_id).await`

### 8.3 Exemples de Tools Fonctionnels

```
examples/
├── hello-world/          # Tool minimal
│   ├── Cargo.toml
│   ├── tool.toml
│   └── src/lib.rs
├── echo/                 # Retourne l'input tel quel
│   ├── ...
├── json-transform/       # Transformation JSON
│   ├── ...
└── http-fetch/           # Fetch HTTP (nécessite réseau)
    ├── ...
```

Chaque exemple doit :
1. Compiler : `cargo build --target wasm32-wasip2`
2. S'installer : `kami install .`
3. S'exécuter : `kami exec <id> '{"input":"hello"}'`
4. Être documenté : README.md dans chaque sous-dossier

### 8.4 Messages d'Erreur Améliorés

Transformer les erreurs techniques en messages actionnables :

```
AVANT :
Error: engine error: unknown import: ...

APRÈS :
Error: The WASM component imports 'wasi:http/outgoing-handler@0.2.0' but KAMI
       does not provide this interface.
       
  Cause: Your tool uses HTTP but the security config has no network access.
  Fix:   Add to your tool.toml:
         [security]
         net_allow_list = ["api.example.com"]
```

Implémenter via un trait `DiagnosticError` avec `hint()` et `fix()`.

---

## 9. Sprint 7 — Features Avancées

> **Durée : 5-8 jours | Priorité : P2-P3**
> **Dépendance : Sprint 5**

### 9.1 Multi-Tool Orchestration (Pipelines)

**Concept :** Chaîner des tools : output de A → input de B.

```toml
# pipeline.toml
[[steps]]
tool = "dev.example.fetch-url"
input = { url = "https://api.example.com/data" }

[[steps]]
tool = "dev.example.json-transform"
input_from = "previous"  # Prend l'output du step précédent
transform = ".data.items"
```

**Nouveau module :** `crates/kami-runtime/src/pipeline.rs`

### 9.2 Versioning & Update

**Commandes :**
- `kami update <tool-id>` — met à jour vers la dernière version
- `kami update --all` — met à jour tous les tools
- `kami pin <tool-id> 1.2.3` — verrouille une version

**Implémentation :**
- Migration DB v3 : colonne `pinned_version TEXT`
- `ToolRepository::find_updates()` — compare versions locales vs source

### 9.3 Rate Limiting

**Module :** `crates/kami-runtime/src/rate_limiter.rs`

```rust
pub struct RateLimiter {
    limits: HashMap<ToolId, TokenBucket>,
    global_limit: TokenBucket,
}
```

**Config :**
```toml
[runtime]
rate_limit_per_tool = 100    # req/min par tool
rate_limit_global = 1000     # req/min total
```

### 9.4 Authentication pour Transport HTTP

**Module :** `crates/kami-transport-http/src/auth.rs`

- Bearer token (simple, suffisant pour v1)
- API key rotation
- Future : OAuth2 / OIDC

### 9.5 Plugin Marketplace (Vision Long Terme)

Phase exploratoire :
- Registry centralisé (crates.io-like pour WASM tools)
- `kami search <query>`
- `kami install registry://dev.example.fetch-url@1.2.3`
- Signature des packages (code signing)
- Métriques de téléchargement / popularité

---

## 10. Sprint 8 — Écosystème & Go-to-Market

> **Durée : 3-5 jours | Priorité : P2**
> **Dépendance : Sprint 5 + Sprint 6**

### 10.1 Documentation Utilisateur

| Document | Contenu | Public |
|----------|---------|--------|
| **Getting Started** | Install → premier tool → premier appel MCP en 5 min | Développeurs |
| **Tool Developer Guide** | Rust → WASM → tool.toml → publier | Auteurs de tools |
| **Operator Guide** | Deploy → configure → monitor → scale | Ops/DevOps |
| **Architecture Guide** | ADRs, diagrammes, crate map | Contributeurs |
| **Security Model** | Threat model, sandbox, capabilities | Security reviewers |

### 10.2 Intégrations AI Agent

Guides + tests pour :
- **Claude Desktop** : `claude_desktop_config.json` avec `kami serve`
- **Cursor** : Configuration MCP dans les settings
- **Continue.dev** : Plugin MCP
- **LangChain** : Custom MCP client wrapper
- **OpenAI Agents** : Tool use via HTTP transport

### 10.3 Contenu Thought Leadership

- Blog : *"Why WASM Sandboxing Matters for AI Tool Execution"*
- Blog : *"KAMI vs Docker for AI Tool Isolation"*
- Talk/Video : *"Building a Secure MCP Server in Rust"*

### 10.4 Community

- GitHub Discussions activé
- Issue templates (bug, feature, security)
- CONTRIBUTING.md
- Code of Conduct
- Security policy (`SECURITY.md` — responsible disclosure)

---

## 11. Matrice de Risques

| # | Risque | Probabilité | Impact | Sprint | Mitigation |
|---|--------|-------------|--------|--------|------------|
| R1 | **MCP breaking changes** (nouveau protocol version) | Haute | Haut | S7 | Abstraire le protocole derrière `trait McpHandler`. Versionner la sérialisation. Suivre le repo MCP spec. |
| R2 | **Wasmtime API changes** (v27 → v28+) | Moyenne | Haut | S5 | Opaque wrapper dans `kami-engine`. Pin la version. Tester les upgrades en CI. |
| R3 | **Supply chain attack** (WASM modifié) | Moyenne | Critique | S2 | Hash SHA-256 (Sprint 2). Signature future (Sprint 7). |
| R4 | **Performance dégradée** (compilation WASM lente) | Moyenne | Moyen | S3 | Benchmark CI (Sprint 3). Cache AOT avec serialisation. |
| R5 | **Adoption faible** (WASM dur pour les devs) | Haute | Haut | S6+S8 | SDK ergonomique. Exemples. Multi-langage compile-to-WASM. |
| R6 | **Concurrence TypeScript MCP** | Haute | Moyen | S8 | Se différencier sur sécurité, pas simplicité. |
| R7 | **Memory leak dans long-running server** | Moyenne | Haut | S4+S5 | Cache eviction. Metrics mémoire. Restart policy. |
| R8 | **Sandbox escape via WASI** | Basse | Critique | S2 | Tests adversarial. Suivre les CVEs wasmtime. Fuzzing. |
| R9 | **Scope creep** (trop de features) | Haute | Moyen | Tous | Prioritiser. MVP d'abord. |

---

## 12. Indicateurs de Succès (KPIs)

### Qualité technique

| KPI | Cible Sprint 3 | Cible v0.2 | Cible v1.0 |
|-----|----------------|------------|------------|
| Compilation | PASS | PASS | PASS |
| Tests | > 120 | > 200 | > 350 |
| Test coverage | > 60% | > 75% | > 85% |
| Clippy warnings | 0 | 0 | 0 |
| Fichiers > 150 lignes | 0 | 0 | 0 |
| `unwrap()` en prod | 0 | 0 | 0 |
| Failles sécurité connues | 0 HAUTE | 0 | 0 |
| CI pipeline | Oui | + security audit | + fuzzing |

### Performances

| KPI | Baseline | Cible v1.0 |
|-----|----------|------------|
| Compilation WASM (cold) | ~50ms | < 100ms (p99) |
| Exécution tool (warm) | ~5ms | < 10ms (p99) |
| Cache hit ratio | N/A | > 90% |
| MCP roundtrip (stdio) | ~10ms | < 20ms (p99) |
| MCP roundtrip (HTTP) | N/A | < 50ms (p99) |

### Adoption

| KPI | Cible 3 mois | Cible 6 mois | Cible 12 mois |
|-----|-------------|-------------|---------------|
| GitHub stars | 100 | 500 | 2000 |
| Tools publiés | 5 (exemples) | 20 | 100+ |
| Contributors | 2 | 5 | 15 |
| Downloads binaires | 50 | 500 | 5000/mois |
| Intégrations AI agents | 2 | 4 | 6+ |

---

## 13. Planning Synthétique

```
Semaine 1
├── Sprint 0 : Fix compilation (0.5j)      ██ BLOQUANT
├── Sprint 2 : Sécurité prioritaire (2j)   ████████
└── Sprint 1 : Dette technique (début)     ████

Semaine 2
├── Sprint 1 : Dette technique (fin)       ██████
├── Sprint 3 : Tests (début)               ████
└── Sprint 4 : Observabilité               ████

Semaine 3
├── Sprint 3 : Tests (fin)                 ████
├── Sprint 5 : CI/CD + Transport HTTP      ████████
└── Sprint 6 : DX (init, watch, exemples)  ████

Semaine 4
├── Sprint 5 : Productisation (fin)        ██████
├── Sprint 6 : DX (fin)                    ████
└── v0.2.0 RELEASE                         ██

Semaine 5-8
├── Sprint 7 : Features avancées           ████████████████
├── Sprint 8 : Écosystème                  ████████
└── v0.3.0 RELEASE                         ██

Mois 3-6
├── HTTP/SSE transport stable
├── Rate limiting
├── Marketplace exploratoire
└── v1.0.0 RELEASE
```

---

## Annexe A — Checklist Pre-Release v0.2

- [ ] `cargo build` — PASS
- [ ] `cargo test` — PASS, > 120 tests
- [ ] `cargo clippy -- -D warnings` — 0 warnings
- [ ] `cargo fmt --check` — PASS
- [ ] `cargo audit` — 0 vulnerabilities
- [ ] Aucun fichier > 150 lignes
- [ ] Aucun `unwrap()` en code production
- [ ] Aucune faille sécurité HAUTE connue
- [ ] CI/CD fonctionnel (build + test + lint)
- [ ] 3 exemples de tools fonctionnels
- [ ] README avec getting started
- [ ] CHANGELOG à jour
- [ ] Binaires Linux + macOS + Windows

## Annexe B — Checklist Pre-Release v1.0

- [ ] Tout v0.2 +
- [ ] Transport HTTP/SSE stable
- [ ] Test coverage > 85%
- [ ] Benchmarks de régression en CI
- [ ] Fuzzing sur modules sécurité
- [ ] Graceful shutdown
- [ ] Health probes
- [ ] Rate limiting
- [ ] Intégrations testées : Claude Desktop, Cursor
- [ ] Documentation complète (5 guides)
- [ ] Security audit externe (optionnel mais recommandé)
- [ ] SECURITY.md avec responsible disclosure policy
