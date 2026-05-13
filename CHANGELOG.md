# Changelog

Toutes les évolutions notables de n2b — format [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), versionnage [SemVer](https://semver.org/lang/fr/).

## [0.5.0] — 2026-05-13

### Modifié

- **Refactor Turborepo-style** : `n2b-core` éclaté en 7 micro-crates frères (`n2b-types`, `n2b-util`, `n2b-ai`, `n2b-github`, `n2b-rules`, `n2b-scanners`, `n2b-report`) + facade `n2b-core` qui réexporte tout. Package TS `@n2b/core` éclaté en 4 packages frères (`@n2b/core` wrapper CLI, `@n2b/types` schemas auto-générés, `@n2b/plugin` Bun.plugin + FFI, `@n2b/shims` polyfills Bun).
- **Cargo workspace** : `resolver = "3"`, `edition = "2024"`, `rust-version = "1.85"`. Workspace lints clippy + rust. Toutes les deps externes consolidées dans `[workspace.dependencies]`.
- **Compat API préservée** : `n2b-cli` et consommateurs externes (rpb-dashboard) continuent d'importer via `n2b_core::{types, scanners, ai, llmstxt, ...}` grâce aux re-exports facade. Aucun breaking sur le contrat gelé.
- **Bump versions** : 0.4.0 → 0.5.0 sur 8 crates (cli, core, ai, github, report, util, rules, scanners). `n2b-types` 0.2.0 → 0.3.0. `n2b-native` reste à 0.1.0 (ABI v1 gelée). 4 packages TS bumpés en concert.

### Corrigé

- Edition 2024 : `#[no_mangle]` → `#[unsafe(no_mangle)]`, blocs `unsafe { }` explicites dans `n2b-native`.
- Cycle TS éliminé : retrait du re-export `n2bPlugin` depuis `@n2b/core` (consommateurs importent désormais `@n2b/plugin`).
- Hygiène monorepo parent : retrait Biome (CLAUDE.md interdit Biome depuis 2026-04-26), remplacé par `oxlint` + `cargo fmt`. Drop `biome.json`.

---

## [0.4.0] — 2026-04-19

### Breaking changes

- **`mui-to-md3` supprimée.** La sous-commande `n2b mui-to-md3` est retirée de ce workspace et déplacée vers `~/vps/rust/mui-rs/crates/mui-rs-codemod-staging/`. `n2b` est désormais **Node→Bun only** ; toute migration MUI v5→MD3 se fait via le workspace `mui-rs`.
- **`ecosystem/mui` et `ecosystem/mui-x` supprimés.** Les règles qui détectaient `@mui/*` et `@mui/x-*` dans `package.json` sont retirées. Ce périmètre UI-specific sort du scope de n2b. Le mapping complet des packages est archivé dans `~/vps/rust/mui-rs/crates/mui-rs-codemod-staging/MUI_PACKAGES.md`. Pour tout projet ayant `@mui/icons-material` ou `@mui/x-charts`, `findings_total` baisse de 2 en conséquence.

### Supprimé

- `scanners/package_json.rs` : 8 entrées `@mui/*` retirées de la table de détection d'écosystème (`@mui/material`, `@mui/icons-material`, `@mui/lab`, `@mui/system`, `@mui/base`, `@mui/x-data-grid`, `@mui/x-date-pickers`, `@mui/x-charts`).
- `commands/rules.rs` : `("ecosystem/mui", ...)` et `("ecosystem/mui-x", ...)` retirés de la table des règles listées par `n2b rules`.
- Snapshots `tests/rpb-dashboard-baseline/` mis à jour : 2 findings MUI retirés, `findings_total` 280 → 278.

### Corrigé

- `rust_cmd.rs` : `format!()` inutile sur un littéral statique remplacé par `.to_string()` (clippy `useless_format`).

---

## [0.3.0] — 2026-04-18

Refactor massif pour maintenabilité et précision Rust↔Bun.

### Breaking changes

- **`packages/n2b-cli/` supprimé.** La CLI TypeScript `node2bun` (binaire compilé via `bun build --compile`) est retirée. Elle dupliquait un sous-ensemble de la CLI Rust sans consommateur externe détecté. **Aucune action requise côté utilisateur** — la CLI Rust `n2b` (dans `/usr/local/bin/n2b`) reste le point d'entrée canonique.
- **`@n2b/core` passe de 0.1.0 à 0.3.0 et change d'API.** Les scanners et règles implémentés en TypeScript sont retirés : toute la logique métier vit désormais dans le binaire Rust. `@n2b/core` expose maintenant :
  - `scan()`, `rules()`, `promptMarkdown()`, `binaryVersion()` — subprocess wrappers typés.
  - `n2bPlugin()` — `Bun.plugin()` qui délègue le scan au binaire Rust.
  - `shims/{env,fs,path,shell}` — Bun-native helpers pour les patterns Node que n2b signale le plus souvent.
- **Layout Cargo workspace réorganisé** : `rust/` et `native/` remplacés par `crates/{n2b-core, n2b-cli, n2b-native}`. Les consommateurs de source directe doivent mettre à jour leurs chemins.
- **Schéma `schema/v2.json` aligné sur l'implémentation réelle.** Les champs racine sont désormais `{schema_version, tool, version, mode, root, files_scanned, findings_total, files}` au lieu de `{version, tool_version, summary, files}`. **Aucune breaking change côté payload JSON** : le binaire a toujours émis cette forme, seul le schéma documenté a été synchronisé. Les scripts qui parsaient déjà la vraie sortie ne changent pas.

### Ajouté

- **Contrat CLI-as-API formel** : `tests/compare-baseline.sh` (13 assertions snapshot) + `crates/n2b-cli/tests/contract.rs` (9 tests `assert_cmd` avec validation `jsonschema` contre `schema/v2.json`). Toute régression sur l'output JSON casse la CI.
- **Codegen de types depuis le schéma** : `scripts/generate-schema-types.ts` produit `crates/n2b-core/src/schema.rs` (via `cargo-typify`) et `packages/n2b/src/schema.ts` (via `json-schema-to-typescript`). Mode `--check` en CI détecte la dérive.
- **Tests round-trip** : `crates/n2b-cli/src/schema_test.rs` désérialise les baselines capturées sur `test/fixture/` et `rpb-dashboard/` dans `N2bReport`.
- **Rollback transactionnel de migration** : `crates/n2b-cli/src/subprocess/bun.rs` — `BackupGuard` sauvegarde `package.json`, `pnpm-workspace.yaml` et les lockfiles rivaux en `.n2b-bak` avant side-effects ; `restore_all()` sur échec `bun install`.
- **Property tests** sur les scanners critiques (`scanners/package_json.rs`, `scanners/source.rs`) via `proptest`.
- **CI** : `.github/workflows/ci.yml` matrice Ubuntu+macOS — `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, `bun typecheck`, `scripts/compare-baseline.sh`, `scripts/generate-schema-types.ts --check`.
- **Configs lint** : `rustfmt.toml`, `clippy.toml` (msrv 1.75, `avoid-breaking-exported-api`).
- **Shims Bun** : `packages/n2b/src/shims/env.ts` (lecture typée de `Bun.env`), `fs.ts` (wrappers `Bun.file`), `path.ts` (équivalents `fileURLToPath`/`__dirname`), `shell.ts` (alias `Bun.$`).

### Modifié

- `crates/n2b-cli/src/main.rs` : **1566 → 37 lignes** (dispatch uniquement).
- `wasm_spec.rs` (1364) → `commands/wasm_spec/{mod,parser,codegen,validator}.rs`.
- `bin_cmd.rs` (1055) → `bin_cmd.rs` + `bin_cmd_gpu.rs` + `bin_cmd_templates.rs`.
- `win32_cmd.rs` (988) → `win32_cmd.rs` + `win32_cmd_com.rs` + `win32_cmd_templates.rs`.
- `commands/audit.rs` utilise `tokio::runtime::Builder::new_current_thread()` au lieu d'un runtime multi-thread global.
- **41 `unwrap()`/`panic!` durcis** en `.context(...)?` ou `.expect("invariant: …")` documentés.

### Supprimé

- Dossiers `rust/`, `native/`, `packages/n2b-cli/`.
- `packages/n2b/src/{scanners,rules,types.ts,util.ts,report.ts}`.
- `dist/node2bun`.

### Préservé (contrat externe gelé)

- Tous les subcommands et flags de la CLI.
- Exit codes 0/1/2.
- `rule_id` (catégorie/nom) immuables.
- ABI FFI cdylib v1 (`find_newlines_u16`, `node2bun_abi_version`).
- Format JSON v2 (schéma gelé, syncé sur l'implémentation).

---

## [0.2.0] — avant le refactor

État initial figé dans le commit `ba65272` (branche `refactor/unified-rust-core`, point de départ). Voir l'historique git pour les détails.
