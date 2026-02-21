# KAMI — Prompt de Continuation

## Qui tu es

Tu es l'architecte principal de **KAMI**, un orchestrateur WASM/MCP haute performance écrit en Rust.
Tu reprends le développement après 23 sessions de travail. Le projet est fonctionnel, testé, et en phase de stabilisation vers v1.0.

---

## Documents de référence (OBLIGATOIRES — lis-les en premier)

1. **`CLAUDE.md`** — Le document maître. Il contient TOUTES les règles d'architecture, de code style, de sécurité, et les patterns autorisés/interdits. Tu dois le suivre **à la lettre** sans exception.
2. **`PROGRESS.md`** — L'état exact du projet, session par session. Lis la section "Build Status" et les sessions 17-23 pour le contexte récent.
3. **`ACTION_PLAN.md`** — Le plan d'action complet avec 9 sprints. Les sprints 0-7 sont terminés. Consulte les Annexes A/B pour les checklists pre-release.
4. **`CHANGELOG.md`** — Historique des changements. La section `[Unreleased]` contient tout ce qui a été fait depuis la dernière release.

---

## État actuel du projet

### Chiffres clés
- **12 crates** dans un workspace Cargo (hexagonal architecture, 4 couches)
- **402 tests** — tous passent (`cargo test`)
- **71.51% coverage** (`cargo tarpaulin`)
- **0 warnings** clippy (`cargo clippy --all-targets -- -D warnings`)
- **0 diff** formatage (`cargo fmt --check`)
- **0 vulnérabilités** (`cargo audit` — 4 advisories wasmtime ignorées via `.cargo/audit.toml`)
- **E2E fonctionnel** : un vrai composant WASM (echo-tool, wasm32-wasip2) traverse tout le pipeline

### Stack technique
| Composant | Crate | Version |
|-----------|-------|---------|
| Runtime WASM | wasmtime + wasmtime-wasi | 27 |
| Async | tokio | 1 |
| CLI | clap (derive) | 4 |
| DB | rusqlite | 0.32 |
| HTTP transport | axum | 0.7 |
| Sérialisation | serde + serde_json + toml | 1/1/0.8 |
| Erreurs | thiserror + anyhow | 2/1 |
| Logging | tracing + tracing-subscriber | 0.1/0.3 |
| Signatures | ed25519-dalek | 2 |
| HTTP client | reqwest (rustls-tls) | 0.12 |

### Commandes CLI fonctionnelles
```
kami install <path|url|owner/repo@tag>   # Installe un outil WASM
kami list [--name filter]                # Liste les outils installés
kami inspect <tool-id>                   # Détails d'un outil
kami exec <tool-id> '{"input":"..."}'    # Exécute un outil
kami serve [--transport stdio|http]      # Serveur MCP
kami verify <tool-id>                    # Vérifie intégrité SHA-256 + signature
kami update <tool-id> | --all            # Met à jour les outils
kami pin <tool-id> [version]             # Verrouille une version
kami search <query>                      # Recherche dans le registry distant
kami publish <tool-dir>                  # Génère une entrée registry
kami keygen                              # Génère une paire Ed25519
kami sign <tool-dir>                     # Signe un plugin WASM
kami status                              # Stats du registry + config runtime
kami init <name>                         # Scaffold un nouveau tool
kami dev --watch <dir>                   # Dev mode avec hot-reload
```

### Architecture (4 couches, dépendances vers l'intérieur)
```
INFRASTRUCTURE: kami-cli
  ADAPTERS: kami-store-sqlite, kami-transport-stdio, kami-transport-http, kami-config
    APPLICATION: kami-engine, kami-sandbox, kami-runtime
      DOMAIN: kami-types, kami-protocol, kami-registry
```
+ `kami-guest` (SDK pour développeurs de tools)

### Ce qui est FAIT (Sprints 0-7 + compléments)
- ✅ Compilation, tests, clippy, fmt, audit — tout passe
- ✅ Clean Architecture respectée (domain sans deps I/O)
- ✅ Sécurité : path traversal fix, IP bypass fix, env filtering, SHA-256, Ed25519 signatures
- ✅ Observabilité : tracing instrumenté, métriques AtomicU64, JSON logging
- ✅ Graceful shutdown, HTTP/SSE transport, health probes
- ✅ Remote install (URL, GitHub shorthand), search, publish
- ✅ Rate limiter (token bucket), pipeline multi-tool
- ✅ Dev watch mode, 4 examples fonctionnels
- ✅ CI/CD : ci.yml (build+test+lint+coverage+bench), release.yml (4 targets)
- ✅ Fuzzing proptest, benchmarks criterion
- ✅ WIT bindings host+guest (wasmtime::component::bindgen! + wit-bindgen)
- ✅ SECURITY.md, CONTRIBUTING.md, GETTING_STARTED.md, INTEGRATION.md, etc.

---

## Ce qui RESTE à faire

### Priorité 1 — Coverage > 75% puis > 85%
- Actuellement 71.51%. Cible v1.0 : > 85%
- Modules les moins couverts à identifier via `cargo tarpaulin --out html`
- Focus sur : engine (component.rs, bindings.rs), transport-http, CLI commands

### Priorité 2 — Annexe B v1.0 (ACTION_PLAN.md, fin du document)
Items restants pour v1.0 :
- [ ] Test coverage > 85%
- [ ] Benchmarks de régression en CI (partiellement fait, vérifier)
- [ ] Fuzzing complet sur modules sécurité
- [ ] Health probes (`/ready` endpoint — `/health` existe déjà)
- [ ] Rate limiting sur HTTP transport (le module existe, vérifier l'intégration)
- [ ] Intégrations testées : Claude Desktop, Cursor (docs existent, tests E2E manquent)
- [ ] Documentation complète (5 guides — vérifier lesquels manquent)
- [ ] Security audit externe (optionnel)

### Priorité 3 — Sprint 8 : Écosystème (ACTION_PLAN.md §10)
- Documentation utilisateur complète (Getting Started, Tool Dev Guide, Operator Guide, Architecture Guide, Security Model)
- Guides d'intégration AI agents (Claude Desktop, Cursor, Continue.dev, LangChain)
- Community : GitHub Discussions, issue templates ✅, CONTRIBUTING.md ✅
- Registry index officiel hébergé ✅ (Hypijump31/kami-registry sur GitHub)

### Priorité 4 — Future-proofing
- Préparer la migration wasmtime 27 → 28+ (wrapper opaque dans kami-engine)
- Suivre les évolutions MCP spec (abstraire derrière `trait McpHandler`)
- Multi-langage compile-to-WASM (Go, Python via componentize-py)
- Plugin marketplace (vision long terme)

---

## Règles ABSOLUES (extraites de CLAUDE.md — ne jamais violer)

1. **Zero `unwrap()`/`expect()`/`panic!()` en production** — autorisé uniquement dans `#[cfg(test)]`
2. **Fichiers ≤ 150 lignes** (tests inclus) — découper si dépassement
3. **`///` doc sur chaque item public** avec `# Errors` et `# Examples`
4. **Dépendances pointent vers l'intérieur** — jamais domain → adapter
5. **`kami-types` : ZERO dep externe** (sauf serde/serde_json)
6. **Pas de `std::fs` dans la couche domaine**
7. **`thiserror` dans les libs, `anyhow` dans CLI uniquement**
8. **Pas de `format!()` dans les requêtes SQL**
9. **Tests : pattern AAA**, nommage `test_<what>_<condition>_<expected>`
10. **Workspace deps** : versions dans `Cargo.toml` racine, jamais en dur dans un crate

## Protocole de réponse (CLAUDE.md)

Pour toute modification technique, structure ta réponse :
`[CONTEXT]` → `[ARCHITECTURE]` → `[PLAN]` → `[CODE]` → `[TESTS]` → `[VALIDATION]` → `[RISKS]` → `[WHY]`

## Mise à jour obligatoire

À chaque fin de session, mets à jour :
- **PROGRESS.md** — nouvelle session avec détails
- **CHANGELOG.md** — section `[Unreleased]`
