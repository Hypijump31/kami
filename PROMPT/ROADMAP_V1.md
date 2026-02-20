# KAMI — Roadmap to v1.0

> **Date** : 20 février 2026
> **Auteur** : Architecte principal
> **Objectif** : Passer de "démo d'ingénierie brillante" à "produit utilisable avec écosystème"
> **Horizon** : 16 semaines (4 phases de 4 semaines)

---

## Contexte décisionnel

### Où on en est

KAMI a des fondations exceptionnelles : 12 crates hexagonaux, 403 tests, 71% coverage, sécurité 8 couches, zero violation prod. L'architecture est textbook-clean.

### Le problème

Le moteur tourne à vide. Aucun développeur externe ne peut créer ni installer d'outil aujourd'hui :

- **Le use case #1** (outil qui appelle une API) **ne fonctionne pas** — WASI HTTP outgoing n'est pas câblé
- **Le SDK** (`kami-guest`) n'est **pas sur crates.io** — impossible de l'utiliser sans cloner le monorepo
- **Le registry** contient **zéro outil installable** — les repos GitHub référencés n'existent pas
- **Rust-only** — 90%+ des développeurs AI sont en Python/TypeScript
- **Le handler MCP** est dans un adapter (stdio) alors que HTTP en dépend — violation subtile

### Ce plan attaque ces 5 blocages dans l'ordre.

---

## Vue d'ensemble des phases

```
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 1 (sem. 1-4)     DÉBLOQUER LE CŒUR                     │
│  ► WASI HTTP outgoing    ► McpHandler → application layer      │
│  ► http-fetch réel       ► Fix archi adapter→adapter           │
├─────────────────────────────────────────────────────────────────┤
│  PHASE 2 (sem. 5-8)     OUVRIR AUX DÉVELOPPEURS               │
│  ► kami-guest sur crates.io  ► 5-10 outils réels               │
│  ► Registry hébergé          ► Pre-compiled .wasm              │
│  ► kami init → projet standalone autonome                      │
├─────────────────────────────────────────────────────────────────┤
│  PHASE 3 (sem. 9-12)    MULTI-LANGAGE & PRODUCTION            │
│  ► Python authoring (componentize-py)                          │
│  ► TypeScript authoring (jco)                                  │
│  ► Prometheus /metrics   ► Coverage 85%+                       │
│  ► CI multi-OS           ► cargo-deny                          │
├─────────────────────────────────────────────────────────────────┤
│  PHASE 4 (sem. 13-16)   ÉCOSYSTÈME & FUTURE-PROOF             │
│  ► Mandatory signatures  ► Tool sandboxed storage              │
│  ► Registry multi-version ► Namespace governance               │
│  ► MCP spec tracking     ► Plugin marketplace UX               │
│  ► v1.0 release                                                │
└─────────────────────────────────────────────────────────────────┘
```

---

## PHASE 1 — Débloquer le cœur (semaines 1-4)

> **Objectif** : Le use case "un outil qui fait un HTTP GET sur une API et retourne le résultat" fonctionne de bout en bout. L'architecture est propre.

### Sprint 1.1 — WASI HTTP Outgoing (sem. 1-2)

**Pourquoi c'est la priorité absolue** : Sans réseau sortant, KAMI ne peut exécuter que des outils de pure computation (JSON transform, greeting). Or le use case n°1 des agents AI, c'est "va chercher telle donnée sur telle API". C'est le deal-breaker.

#### Tâches

| # | Tâche | Fichiers impactés | Critère de validation |
|---|-------|-------------------|----------------------|
| 1.1.1 | **Ajouter `wasmtime-wasi-http` au workspace** | `Cargo.toml` (workspace deps), `crates/kami-engine/Cargo.toml` | `cargo build` passe |
| 1.1.2 | **Étendre le WIT world** : ajouter `import wasi:http/outgoing-handler@0.2.0` (optionnel, gardé avec feature flag) | `wit/world.wit` | `cargo component check` (ou `bindgen!` compile) |
| 1.1.3 | **Câbler `wasi_http::proxy::add_to_linker()` dans le linker** engine | `crates/kami-engine/src/component.rs`, `crates/kami-engine/src/bindings.rs` | Linker accepte les composants qui importent `wasi:http` |
| 1.1.4 | **Intégrer le contrôle d'accès réseau** : le `socket_addr_check` existant dans `kami-sandbox` doit s'appliquer aussi aux requêtes HTTP WASI — vérifier que `net_allow_list` filtre les hosts | `crates/kami-sandbox/src/network.rs`, `crates/kami-sandbox/src/wasi.rs` | Test : un outil tente `GET https://blocked.com` → deny |
| 1.1.5 | **Faire fonctionner `http-fetch` pour de vrai** : remplacer le placeholder par un vrai appel `wasi:http/outgoing-handler` dans le guest | `examples/http-fetch/src/lib.rs`, `examples/http-fetch/Cargo.toml` | `kami exec dev.example.http-fetch --input '{"url":"https://httpbin.org/get"}'` retourne du vrai JSON |
| 1.1.6 | **Tests d'intégration** : tests E2E avec un mock HTTP server (ou `httpbin.org`) | `crates/kami-runtime/tests/e2e_http.rs` | 3+ tests passent |
| 1.1.7 | **Documentation** : mettre à jour TOOL_AUTHOR_GUIDE.md et DEVELOPER.md pour les outils avec réseau | `docs/TOOL_AUTHOR_GUIDE.md`, `docs/DEVELOPER.md` | Sections "HTTP tools" ajoutées |

#### Risques & mitigations

| Risque | Probabilité | Mitigation |
|--------|-------------|------------|
| `wasmtime-wasi-http` API instable entre v27→v28 | Moyenne | Épingler wasmtime v27, réévaluer migration plus tard |
| Latence du proxy HTTP WASI ajoutée | Basse | Benchmark avant/après, documenté |
| Complexité du linker (double WASI + HTTP) | Moyenne | Approche incrémentale : d'abord linker compile, puis test réel |

#### Livrables

- [ ] `kami exec` sur un outil qui fait un vrai HTTP GET → résultat correct
- [ ] `net_allow_list` respecté sur les appels HTTP WASI
- [ ] Example `http-fetch` fonctionnel et honnête
- [ ] 3+ tests E2E réseau

---

### Sprint 1.2 — Extraire McpHandler dans l'application layer (sem. 3-4)

**Pourquoi** : Aujourd'hui `kami-transport-http` dépend de `kami-transport-stdio` pour accéder au `McpHandler`. C'est une **dépendance adapter→adapter** qui viole la Clean Architecture. Les deux transports devraient dépendre d'un composant partagé dans la couche application.

#### Tâches

| # | Tâche | Fichiers impactés | Critère |
|---|-------|-------------------|---------|
| 1.2.1 | **Créer `kami-mcp` crate** dans la couche application | `Cargo.toml` (workspace), `crates/kami-mcp/` (nouveau) | Crate compiles |
| 1.2.2 | **Déplacer `McpHandler` + dispatch logic** de `kami-transport-stdio` vers `kami-mcp` | `crates/kami-mcp/src/handler.rs`, `crates/kami-mcp/src/dispatch/` | Tous les tests de dispatch passent |
| 1.2.3 | **`kami-transport-stdio` dépend de `kami-mcp`** (pas l'inverse) | `crates/kami-transport-stdio/Cargo.toml`, `src/server.rs` | `cargo build` + tests passent |
| 1.2.4 | **`kami-transport-http` dépend de `kami-mcp`** (supprime la dep sur stdio) | `crates/kami-transport-http/Cargo.toml`, `src/router.rs` | `cargo build` + tests passent |
| 1.2.5 | **Mettre à jour le diagramme d'architecture** | `CLAUDE.md`, `docs/ARCHITECTURE.md`, `README.md` | Nouveau crate visible dans la structure |

#### Nouvelle architecture transport

```
        ┌─────────────────┐         ┌──────────────────┐
        │  kami-transport  │         │  kami-transport   │
        │    -stdio        │         │    -http          │
        │   (adapter)      │         │   (adapter)       │
        └────────┬─────────┘         └────────┬──────────┘
                 │                             │
                 ▼                             ▼
        ┌────────────────────────────────────────────────┐
        │              kami-mcp                           │
        │        (APPLICATION layer)                     │
        │  McpHandler, dispatch, inputSchema builder     │
        └────────────────────┬───────────────────────────┘
                             │
                   ┌─────────▼──────────┐
                   │   kami-registry     │
                   │   kami-runtime      │
                   │   kami-protocol     │
                   └────────────────────┘
```

#### Livrables

- [ ] `kami-transport-http` ne dépend plus de `kami-transport-stdio`
- [ ] `kami-mcp` est dans la couche application
- [ ] Aucune régression sur les 403+ tests
- [ ] Diagrammes mis à jour

---

## PHASE 2 — Ouvrir aux développeurs (semaines 5-8)

> **Objectif** : Un développeur Rust externe peut écrire, compiler, publier et installer un outil KAMI **sans cloner le monorepo**.

### Sprint 2.1 — Publier kami-guest sur crates.io (sem. 5)

**Pourquoi** : C'est le **bloqueur n°1** pour les contributeurs externes. Tant que le SDK est un path dep, personne ne peut écrire d'outil dehors.

#### Tâches

| # | Tâche | Détails |
|---|-------|---------|
| 2.1.1 | **Préparer `kami-guest` pour publication** | `kami-types` en dépendance publiée (ou inlined), version 0.1.0, license MIT, `categories`, `keywords` dans Cargo.toml |
| 2.1.2 | **Publier `kami-types` sur crates.io** | Prérequis si `kami-guest` en dépend. Sinon, inliner les types nécessaires (ToolMetadata) dans `kami-guest` pour couper la dépendance. **Décision à prendre** : inline vs publish `kami-types` |
| 2.1.3 | **Publier `kami-guest` 0.1.0** sur crates.io | `cargo publish -p kami-guest` |
| 2.1.4 | **Mettre à jour `kami init`** : le template Cargo.toml utilise `kami-guest = "0.1"` au lieu du chemin local | `crates/kami-cli/src/commands/templates.rs` |
| 2.1.5 | **Mettre à jour la documentation** : TOOL_AUTHOR_GUIDE, DEVELOPER.md, README | Remplacer les refs au path local par crates.io |
| 2.1.6 | **Tester le workflow complet** : `kami init → cargo build --target wasm32-wasip2 → kami install → kami exec` depuis un répertoire externe au monorepo | Test E2E manuel + documenté |

#### Décision architecturale : `kami-types` publié ou pas ?

| Option | Avantage | Inconvénient |
|--------|----------|--------------|
| **A : Publier `kami-types` + `kami-guest`** | Cohérent, types partagés, DRY | 2 crates à maintenir sur crates.io |
| **B : Inliner dans `kami-guest` seul** | 1 seul crate à publier, plus simple | Duplication des types (ToolMetadata) |

**Recommandation** : **Option A**. `kami-types` est stable et minimaliste (zero deps externes hors serde). C'est le contrat d'interface — il doit être public.

#### Livrables

- [ ] `kami-guest` 0.1.0 sur crates.io
- [ ] `kami-types` 0.1.0 sur crates.io
- [ ] `kami init` génère un projet autonome qui compile sans le monorepo
- [ ] README badge "crates.io" ajouté

---

### Sprint 2.2 — Outils réels dans le registry (sem. 6-7)

**Pourquoi** : Un registry vide est un produit mort. Il faut du contenu day-one.

#### Plan de création de 8 outils

| Outil | ID | Catégorie | Complexité | Réseau |
|-------|----|-----------|------------|--------|
| **echo** | `dev.kami.echo` | Debug | Simple | Non |
| **hello-world** | `dev.kami.hello-world` | Demo | Simple | Non |
| **json-transform** | `dev.kami.json-transform` | Data | Moyenne | Non |
| **http-fetch** | `dev.kami.http-fetch` | Network | Moyenne | Oui |
| **base64-codec** | `dev.kami.base64` | Utility | Simple | Non |
| **markdown-to-html** | `dev.kami.markdown-html` | Content | Moyenne | Non |
| **url-parse** | `dev.kami.url-parse` | Utility | Simple | Non |
| **json-schema-validate** | `dev.kami.json-schema` | Data | Moyenne | Non |

#### Workflow par outil

1. Créer un repo GitHub `Hypijump31/kami-<name>`
2. Code source Rust + `tool.toml`, utilisant `kami-guest` depuis crates.io
3. `cargo build --target wasm32-wasip2 --release`
4. GitHub Release avec le `.wasm` compilé en asset
5. Mettre à jour `registry/index.json` avec le vrai SHA-256
6. Vérifier : `kami install Hypijump31/kami-<name>@v1.0.0` fonctionne

#### Livrables

- [ ] 8 repos GitHub créés et tagués avec releases
- [ ] `registry/index.json` contient 8 entrées avec vrais SHA-256
- [ ] `kami search echo` retourne des résultats
- [ ] `kami install Hypijump31/kami-echo@v1.0.0` fonctionne E2E

---

### Sprint 2.3 — Registry hébergé & DX polish (sem. 8)

#### Tâches

| # | Tâche | Détails |
|---|-------|---------|
| 2.3.1 | **Héberger le registry** sur GitHub Pages (`Hypijump31/kami-registry`) | `index.json` + `schema.json` servis statiquement; URL stable du type `https://hypijump31.github.io/kami-registry/index.json` |
| 2.3.2 | **Configurer l'URL par défaut** dans `kami search` | Pointer vers le registry hébergé au lieu de raw GitHub |
| 2.3.3 | **Pre-compiled `.wasm` dans les examples/** | Ajouter des fichiers `.wasm` pré-compilés pour hello-world + echo afin que les utilisateurs puissent tester sans installer la toolchain `wasm32-wasip2` |
| 2.3.4 | **Quick Start "30 secondes"** | Section dans README : `curl install + kami install + kami exec` — 3 commandes, résultat visible |
| 2.3.5 | **`kami doctor`** — nouvelle commande de diagnostic | Vérifie : rustc version, wasm32-wasip2 target, kami version, registry reachable, DB writable. Aide au onboarding. |
| 2.3.6 | **Publier le binaire `kami` sur GitHub Releases** | Le workflow release.yml existe — créer la première release v0.9.0-beta |
| 2.3.7 | **Homebrew formula** (macOS) / **cargo-binstall** metadata | Installation simplifiée sans compiler depuis les sources |

#### Livrables

- [ ] `kami search` fonctionne out-of-the-box (registry par défaut)
- [ ] Installation en 1 commande (binaire pré-compilé)
- [ ] `kami doctor` valide l'environnement
- [ ] Quick Start 30s dans le README

---

## PHASE 3 — Multi-langage & production (semaines 9-12)

> **Objectif** : Un développeur Python ou TypeScript peut écrire un outil KAMI. Le produit est prêt pour la production.

### Sprint 3.1 — Python tool authoring (sem. 9-10)

**Pourquoi** : Python est le langage de l'écosystème AI. C'est le bassin de 90% des auteurs potentiels.

#### Approche technique

Utiliser [`componentize-py`](https://github.com/bytecodealliance/componentize-py) (Bytecode Alliance) pour compiler du Python en WASM Component Model.

#### Tâches

| # | Tâche | Détails |
|---|-------|---------|
| 3.1.1 | **PoC** : Écrire un outil KAMI en Python pur, le compiler avec `componentize-py`, l'exécuter via `kami exec` | Valider que le workflow fonctionne bout en bout |
| 3.1.2 | **SDK Python** : créer `kami-guest-py` — un package pip minimal qui fournit les types et helpers | `pip install kami-guest` → `from kami_guest import tool, text_result` |
| 3.1.3 | **`kami init --language python`** : scaffold un projet Python avec les WIT, le bindings layer, et le build script | `crates/kami-cli/src/commands/init.rs` + templates Python |
| 3.1.4 | **`kami build`** : nouvelle commande qui détecte le langage et compile | Rust → `cargo build --target wasm32-wasip2`, Python → `componentize-py` |
| 3.1.5 | **Exemple Python** : `examples/python-hello/` | Un outil hello-world en Python, compilé et testé |
| 3.1.6 | **Documentation** : "Write a KAMI tool in Python" guide | Nouveau doc ou section dans TOOL_AUTHOR_GUIDE |

#### Contraintes connues

- `componentize-py` est encore jeune (v0.x) — limiter le scope au WASI P2 de base
- Le binaire WASM Python sera plus gros (~5-10MB vs ~100KB Rust) — documenter
- Pas de networking Python côté guest pour le moment (WASI HTTP support TBD dans componentize-py)

#### Livrables

- [ ] `kami init --language python` génère un projet fonctionnel
- [ ] `kami build` compile Python → WASM
- [ ] Un outil Python exécuté via `kami exec` → résultat correct
- [ ] Guide Python dans la documentation

---

### Sprint 3.2 — TypeScript/JavaScript authoring (sem. 11)

#### Approche technique

Utiliser [`jco`](https://github.com/bytecodealliance/jco) (Bytecode Alliance) pour compiler du JavaScript/TypeScript en WASM Component Model.

#### Tâches

| # | Tâche | Détails |
|---|-------|---------|
| 3.2.1 | **PoC** : Écrire un outil en TypeScript, compiler avec `jco componentize`, exécuter via `kami exec` | Valider le workflow |
| 3.2.2 | **`kami init --language typescript`** | Template TypeScript avec tsconfig, WIT bindings, build script |
| 3.2.3 | **`kami build` TypeScript path** | Détecte `package.json` → appelle `jco componentize` |
| 3.2.4 | **Exemple TypeScript** : `examples/ts-hello/` | Hello-world en TypeScript |
| 3.2.5 | **Documentation** : "Write a KAMI tool in TypeScript" guide | |

#### Livrables

- [ ] Outil TypeScript exécuté via `kami exec`
- [ ] `kami init --language typescript` fonctionnel
- [ ] Guide TypeScript dans la doc

---

### Sprint 3.3 — Production hardening (sem. 12)

#### Tâches

| # | Tâche | Pourquoi | Effort |
|---|-------|----------|--------|
| 3.3.1 | **Endpoint `/metrics` Prometheus** dans `kami-transport-http` | Les `ExecutionMetrics` AtomicU64 existent — il manque juste l'exposition HTTP. Ajouter `/metrics` en format OpenMetrics (text/plain) | 1 jour |
| 3.3.2 | **Intégrer `rate_limiter` dans le transport HTTP** | Le module existe mais n'est pas câblé au handler HTTP. Middleware axum. | 1 jour |
| 3.3.3 | **CI multi-OS** | Ajouter `matrix: [ubuntu-latest, macos-latest, windows-latest]` dans ci.yml | 0.5 jour |
| 3.3.4 | **Monter la couverture à 85%** | Focus sur : executor.rs, resolver.rs, handler dispatch, download.rs | 3 jours |
| 3.3.5 | **`cargo-deny`** | Vérification licences + supply-chain + advisories dupliquées | 0.5 jour |
| 3.3.6 | **Dependabot / Renovate** | `.github/dependabot.yml` pour updates automatiques | 0.5 jour |
| 3.3.7 | **Gate de couverture** | `fail_ci_if_error: true` + seuil 80% minimum dans tarpaulin | 0.5 jour |

#### Livrables

- [ ] `GET /metrics` retourne des métriques Prometheus
- [ ] Rate limiting actif sur HTTP transport
- [ ] CI passe sur 3 OS
- [ ] Couverture ≥ 85%
- [ ] `cargo deny check` dans la CI

---

## PHASE 4 — Écosystème & future-proof (semaines 13-16)

> **Objectif** : Le produit est prêt pour une release v1.0 stable. L'écosystème est conçu pour durer.

### Sprint 4.1 — Sécurité avancée (sem. 13)

| # | Tâche | Détails |
|---|-------|---------|
| 4.1.1 | **Policy `require_signatures`** | Nouveau champ dans `KamiConfig` : `require_signatures = true|false`. Si `true`, `ToolResolver` refuse d'exécuter un outil non signé. | 
| 4.1.2 | **WASM import scanning** | Avant exécution, inspecter les imports du composant WASM. Si un outil déclare `fs_access = "none"` mais importe `wasi:filesystem`, refuser. Incohérence = rejet. |
| 4.1.3 | **Audit log** | Chaque exécution d'outil est loguée dans un fichier append-only (`~/.kami/audit.log`) : timestamp, tool_id, user, input hash, result status. Pour la compliance. |
| 4.1.4 | **Content Security Policy** pour les outils HTTP | Limiter non seulement les hosts mais aussi les méthodes HTTP (GET-only, POST-only) et les headers sensibles. Extension de `net_allow_list` en `net_policy`. |

#### Livrables

- [ ] `require_signatures = true` bloque les outils non signés
- [ ] Import scan détecte les incohérences tool.toml ↔ WASM imports
- [ ] Audit log activable
- [ ] Tests adversariaux pour chaque feature

---

### Sprint 4.2 — Persistance & état (sem. 14)

| # | Tâche | Détails |
|---|-------|---------|
| 4.2.1 | **Tool sandboxed storage** | Pour `fs_access = "sandbox"`, créer un répertoire persistant par outil (`~/.kami/data/<tool-id>/`), monté en preopened dir dans WasiCtx. Les données survivent entre invocations. |
| 4.2.2 | **KV store host function** | Nouvelle interface WIT `kami:tool/kv@0.1.0` avec `get(key) → option<string>`, `set(key, value)`, `delete(key)`. Implémentée côté host via le filesystem sandboxé ou une DB embarquée. |
| 4.2.3 | **Tool-to-tool calls** | Nouvelle interface WIT `kami:tool/invoke@0.1.0` avec `invoke(tool-id, input) → result<string, string>`. Implémentée côté host via le runtime. Attention au cycle de dépendances et à la récursion (profondeur max). |

#### Livrables

- [ ] Un outil peut persister des données entre invocations
- [ ] Host KV store fonctionnel
- [ ] Un outil peut appeler un autre outil (avec depth limit)

---

### Sprint 4.3 — Registry mature (sem. 15)

| # | Tâche | Détails |
|---|-------|---------|
| 4.3.1 | **Multi-version registry** | `index.json` passe d'un array plat à un format groupé par ID avec historique de versions. Semver resolution (`^1.0`, `~1.2`, `>=2.0`). |
| 4.3.2 | **Namespace governance** | Format d'ID `<namespace>.<project>.<name>`. Pour publier sous `dev.kami.*`, il faut être membre de l'org `Hypijump31`. Vérifié par le CI du registry. |
| 4.3.3 | **`kami outdated`** | Compare les versions installées aux versions disponibles dans le registry. Affiche un tableau des updates disponibles. |
| 4.3.4 | **Categories & tags** dans le registry | Champs `categories: ["network", "data"]` et `tags: ["json", "http"]` dans `index.json`. `kami search --category network`. |
| 4.3.5 | **Download count tracking** | Compteur de téléchargements par outil (via GitHub Releases API ou un service simple). Affiché dans `kami search`. |

#### Livrables

- [ ] `kami install dev.kami.echo@^1.0` résout la version
- [ ] Namespaces vérifiés en CI
- [ ] `kami outdated` fonctionne
- [ ] Catégories/tags dans le registry

---

### Sprint 4.4 — MCP spec tracking & v1.0 release (sem. 16)

| # | Tâche | Détails |
|---|-------|---------|
| 4.4.1 | **Tracker la spec MCP** | Créer `crates/kami-protocol/src/mcp/version.rs` avec un système de feature flags par version de spec (`2024-11-05`, `2025-xx-xx`). Négociation de version dans le handshake `initialize`. |
| 4.4.2 | **MCP resources support** | Les types existent dans `kami-protocol` mais ne sont pas câblés. Implémenter `resources/list` et `resources/read` dans le handler pour exposer des fichiers ou des données outils. |
| 4.4.3 | **MCP prompts support** | Idem — `prompts/list` et `prompts/get` disponibles mais non câblés. |
| 4.4.4 | **Mettre à jour ADR-007** | Le cache est LRU depuis la session 14, mais l'ADR mentionne encore FIFO. |
| 4.4.5 | **Nettoyer INIT_PROMPT.md** | Stale refs (`wit-bindgen 0.36`, etc.) |
| 4.4.6 | **Release v1.0.0** | Changelog complet, GitHub Release, binaires 4 cibles, annonce |

#### Checklist v1.0

| Critère | Cible |
|---------|-------|
| Tests | ≥ 500 |
| Couverture | ≥ 85% |
| Clippy | 0 warnings |
| Audit | 0 unignored advisories |
| OS testés en CI | 3 (Linux, macOS, Windows) |
| Outils dans le registry | ≥ 10 |
| Langages supportés | 3 (Rust, Python, TypeScript) |
| MCP resources/prompts | Implémentés |
| Documentation | 10+ guides |
| `cargo deny` | Pass |
| Signature policy | Configurable |
| HTTP outgoing | Fonctionnel |

---

## Matrice de décisions architecturales

### Décisions à prendre en Phase 1

| Question | Options | Recommandation | Justification |
|----------|---------|----------------|---------------|
| Nom du nouveau crate pour le handler MCP | `kami-mcp` / `kami-mcp-handler` / `kami-dispatch` | **`kami-mcp`** | Court, clair, aligné avec le protocole |
| Le crate `kami-mcp` dépend de quoi ? | `kami-protocol` + `kami-runtime` + `kami-registry` | Oui, les 3 | Il orchestre protocol (types) + runtime (exec) + registry (lookup) |
| Feature flag pour WASI HTTP ? | Feature sur `kami-engine` ou toujours activé | **Feature `wasi-http`** activée par défaut | Permet de compiler sans HTTP si besoin (embedded, tests) |

### Décisions à prendre en Phase 2

| Question | Options | Recommandation | Justification |
|----------|---------|----------------|---------------|
| Publier `kami-types` sur crates.io ? | Oui / Inliner dans `kami-guest` | **Oui, publier** | C'est le contrat d'interface, il doit être public et versionné |
| Hébergement du registry | GitHub Pages / Cloudflare Pages / Propre serveur | **GitHub Pages** | Zero cost, CI intégrée, suffisant pour v1 |
| Format des pre-compiled .wasm | Git LFS / GitHub Release assets / Commité | **GitHub Release assets** | Pas de blob dans le repo, versionnés naturellement |

### Décisions à prendre en Phase 3

| Question | Options | Recommandation | Justification |
|----------|---------|----------------|---------------|
| `kami build` intégré ou script externe ? | Commande CLI / Makefile / justfile | **Commande CLI** | UX unifiée, détection automatique du langage |
| SDK Python : package pip ou juste un template ? | pip package / template files | **pip package** (`kami-guest`) | Professionnel, versionné, instalable |

---

## KPIs de succès

| Indicateur | Semaine 4 | Semaine 8 | Semaine 12 | Semaine 16 (v1.0) |
|------------|-----------|-----------|------------|-------------------|
| Use case HTTP E2E | **Fonctionne** | Fonctionne | Fonctionne | Fonctionne |
| Outils dans le registry | 0 → 3 | **8** | 10 | **12+** |
| SDK sur crates.io | Non | **Oui** | Oui | Oui |
| Langages supportés | 1 (Rust) | 1 (Rust) | **3** (Rust+Py+TS) | 3 |
| Tests | 403+ | 450+ | 500+ | **550+** |
| Couverture | 71% | 75% | **85%** | 85%+ |
| Contributeurs externes | 0 | 0-1 | 1-3 | **5+** |
| GitHub stars | baseline | +50 | +100 | +200 |
| `kami install` depuis registry | Impossible | **Fonctionne** | Fonctionne | Fonctionne |
| MCP resources/prompts | Non | Non | Non | **Oui** |

---

## Principes directeurs

1. **Ship, don't polish** — La perfection architecturale est atteinte. Maintenant il faut du contenu et des utilisateurs.
2. **Outside-in** — Chaque feature doit être testable du point de vue de l'utilisateur final : `kami install X → kami exec X → résultat`.
3. **Compatibility > Innovation** — Tracker la spec MCP, supporter les clients existants (Claude Desktop, Cursor). Ne pas inventer de protocole custom.
4. **Security is the brand** — C'est le seul differenciateur. Chaque nouvelle feature doit maintenir ou renforcer le modèle de sécurité.
5. **Multi-language or die** — Un SDK Rust-only dans un monde Python/TypeScript est une sentence de mort. Phase 3 est non-négociable.

---

## Dépendances critiques du plan

```
Phase 1           Phase 2           Phase 3           Phase 4
         │                 │                 │
   WASI HTTP ──────► http-fetch réel        │
         │                 │                 │
   McpHandler ─────► kami-mcp crate         │
         │                 │                 │
         │     kami-guest crates.io ──► Python SDK
         │                 │            │
         │     8 outils ───┼────► TS SDK │
         │                 │            │
         │     Registry ───┼──────► Multi-version
         │                 │            │
         │                 │     CI multi-OS ──► v1.0
         │                 │     Coverage 85%
```

Chaque phase débloque la suivante. Pas de raccourci possible sur la Phase 1 (WASI HTTP + archi fix).
