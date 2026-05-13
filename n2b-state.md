# État du Projet n2b — Refactor Turborepo Style ✅ TERMINÉ

> Mis à jour : 2026-05-13 — Refactor complet bout en bout en une session.

## ✅ Phase 1 — Modularisation Rust (7 micro-crates extraits)

`n2b-core` éclaté en 7 crates frères + 1 facade :

| Crate | Contenu | Dépend de |
|---|---|---|
| `n2b-types` | `schema.rs`, `types.rs` | `serde`, `schemars`, `ts-rs` |
| `n2b-util` | `util.rs` (line_offsets, pos_from_index, make_finding) | `n2b-types`, `memchr` |
| `n2b-ai` | `ai.rs` (helpers AI-friendly : category, docs_url, confidence) | `serde` |
| `n2b-github` | `github.rs` (client Octocrab) | `anyhow`, `octocrab` |
| `n2b-rules` | `rules/{bun_apis,cli_commands,imports_ast,node_imports}.rs` | `n2b-types`, `n2b-util`, `oxc_*` |
| `n2b-scanners` | `scanners/*.rs` (19 fichiers) | `n2b-types`, `n2b-util`, `n2b-rules`, `serde_yaml` |
| `n2b-report` | `report.rs` (text/json/jsonl/markdown/sarif) | `n2b-types`, `n2b-ai`, `n2b-util`, `colored` |
| `n2b-core` | `lib.rs` + `audit.rs`, `llmstxt/`, `run.rs` (orchestrateur) | tous les ci-dessus |

**Compat API** : `n2b-core` réexporte tous les sous-crates (`pub use n2b_ai as ai;` etc.), donc `n2b-cli` et `n2b-native` n'ont pas eu à changer leurs imports.

## ✅ Phase 2 — Workspace Cargo strict

- `resolver = "3"`, `edition = "2024"`, `rust-version = "1.85"`.
- `[workspace.metadata.groups]` : `n2b-libraries` + `n2b`.
- `[workspace.lints.rust]` + `[workspace.lints.clippy]`.
- Toutes deps externes dans `[workspace.dependencies]`, sous-crates en `{ workspace = true }`.
- Edition 2024 compat : `#[no_mangle]` → `#[unsafe(no_mangle)]`, `unsafe_op_in_unsafe_fn` corrigé dans `n2b-native`.

## ✅ Phase 3 — Remplacement chirurgical des imports

Fait en parallèle de chaque extraction (mass-rewrite via `sd`) :
- `use crate::types::` → `use n2b_types::types::`
- `use crate::util::` → `use n2b_util::`
- `use crate::rules::` → `use n2b_rules::`
- `use crate::scanners::` → `use n2b_scanners::` (puis `use crate::` pour les refs internes au crate)
- `use crate::ai::` → `use n2b_ai::`

Validation : `cargo build --release --workspace` passe à chaque étape.

## ✅ Phase 4 — TS / Pipeline Turborepo (4 packages)

`packages/n2b/packages/n2b/` (monolithe `@n2b/core`) éclaté en 4 packages frères :

| Package | Contenu | Dépend de |
|---|---|---|
| `@n2b/core` | `cli.ts` + `index.ts` (wrapper CLI thin) | `@n2b/types` |
| `@n2b/types` | `schema.ts` (types TS auto-générés v2) | — |
| `@n2b/plugin` | `plugin.ts` + `ffi.ts` (Bun.plugin + bindings FFI) | `@n2b/types`, `@n2b/core` |
| `@n2b/shims` | `shims/{env,fs,path,shell}.ts` (polyfills Bun) | — |

- `tsconfig.base.json` : paths mis à jour pour 4 packages.
- Project references : `@n2b/core → @n2b/types`, `@n2b/plugin → @n2b/types + @n2b/core`.
- Re-export `n2bPlugin` depuis `@n2b/core` retiré (cycle évité) : consommateurs importent désormais `import { n2bPlugin } from "@n2b/plugin"`.
- 14/14 tests TS passent (`bun test` dans `@n2b/core`).
- Build TS via `tsc -b` : tous les `dist/*.d.ts` générés.

## Validation finale

```
cargo build --release --workspace   → OK
cargo test --workspace --release    → 14 passes / 0 fail
target/release/n2b --version        → n2b 0.4.0
bun test (packages/n2b)             → 14 passes / 0 fail
tsc -b (4 packages)                 → OK
tests/compare-baseline.sh           → 1 diff cosmétique (path absolu legacy dans snapshot fixture.txt — non lié au refactor)
```

## Notes contract gelé

Le contrat externe (`schema/v2.json`, exit codes, rule IDs, ABI cdylib v1, flags CLI) est intact — `n2b-cli` continue d'importer via `n2b_core::{types, scanners, ai, llmstxt, ...}` grâce aux re-exports facade. Aucun breaking pour `rpb-dashboard` ni autre consommateur subprocess.
