# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Principe directeur

**Rust = moteur unique. TypeScript = façade thin.** Toute la logique métier (scan, règles, rendu) vit dans `crates/n2b-core`. Le package TS `@n2b/core` ne fait que spawn le binaire `n2b` et parser son JSON. Ne jamais réintroduire de scanners/règles côté TS.

## Commandes essentielles

```bash
# Build
cargo build --release -p n2b                      # binaire CLI seul
cargo build --release --workspace                 # + cdylib
sudo install -m755 target/release/n2b /usr/local/bin/n2b

# Tests (40 au total répartis comme suit)
cargo test --workspace                            # 14 Rust (schema_test + contract + 3 proptest)
bun test packages/n2b/                            # 14 TS (cli + shims)
bash tests/compare-baseline.sh                    # 12 comparaisons snapshot (5 rpb skippées si rpb-dashboard absent)

# Test ciblé
cargo test --workspace contract                   # juste les contract tests
cargo test -p n2b-core --test proptest_source     # une proptest précise
bun test packages/n2b/test/shims/env.test.ts      # un fichier TS

# Lint
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
bun run typecheck

# Codegen depuis schema/v2.json (source unique)
bun run codegen:schema                            # régénère schema.rs + index.ts
bun run codegen:schema:check                      # --check (CI drift detection)

# Régénérer les baselines après bump version ou changement de sortie légitime
PATH="$PWD/target/release:$PATH" N2B=./target/release/n2b \
  bash -c 'for f in text json jsonl md sarif; do
    ext=$f; [[ $f == text ]] && ext=txt
    $N2B test/fixture --report=$f > tests/snapshots/baseline/fixture.$ext
  done'
```

## Contrat externe gelé — ne jamais casser

Ces surfaces sont consommées par `/home/ubuntu/rpb-dashboard` via subprocess. Toute modification casse la CI baseline :

| Surface | Fichier de vérité |
|---|---|
| Flags et subcommands CLI | `crates/n2b-cli/src/cli/args.rs` |
| Format JSON v2 | `schema/v2.json` (schéma gelé, bumpé en v3 si breaking) |
| Rule IDs (`cli/npm`, `imports/node-prefix`, …) | `crates/n2b-rules/src/*.rs` |
| Exit codes `0`/`1`/`2` | `crates/n2b-cli/src/commands/scan.rs` |
| ABI cdylib v1 (`find_newlines_u16`, `node2bun_abi_version`) | `crates/n2b-native/src/lib.rs` |

Le filet de sécurité est **triple** :
- `tests/compare-baseline.sh` — diff octet-à-octet contre `tests/snapshots/baseline/` et `tests/rpb-dashboard-baseline/`
- `crates/n2b-cli/tests/contract.rs` — 9 tests `assert_cmd` + validation `jsonschema` contre `schema/v2.json`
- `crates/n2b-cli/src/schema_test.rs` — `include_str!` des baselines + round-trip `serde_json` vers `N2bReport` (échoue à la **compilation** du CLI si le schéma diverge)

Si tu changes une règle existante, il faut **soit** justifier le breaking et régénérer les baselines, **soit** ajouter une nouvelle règle. Jamais modifier silencieusement.

## Architecture — pipeline de scan

```
  entry → crates/n2b-cli/src/main.rs (dispatch only)
        → cli::dispatch::run_from_args (args.rs → enum Cmd)
        → scan par défaut (pas de subcommand) ou commands/{rules,audit,prompt}.rs
            → n2b_core::run::run(opts)                    # --migrate vit ici aussi
                → engine walk (ignore + globset + crossbeam)
                → dispatch scanner par extension/nom de fichier
                    n2b-scanners/src/*.rs retournent Vec<Finding>
                    n2b-rules/src/*.rs fournissent regex/IDs partagés
                → n2b-report (text/json/jsonl/markdown/sarif)
        → subcommands annexes (hors pipeline de scan) :
          rust_cmd · app_cmd · bin_cmd · win32_cmd · linux_cmd
          wasm_cmd · patch · bunpp_cmd · analyze · core/llmstxt
```

### Subcommands (13 — `crates/n2b-cli/src/cli/args.rs` fait foi)

| Subcommand | Fichier | Rôle |
|---|---|---|
| *(défaut)* | `commands/scan.rs` | Scan Node→Bun. Flags racine : `--fix`, `--aggressive`, `--migrate`, `--report`, `--ignore`. |
| `rules` | `commands/rules.rs` | Liste les règles connues (tableau plat). |
| `prompt` | `commands/prompt.rs` | Génère un prompt markdown prêt pour un LLM. |
| `audit` | `commands/audit.rs` | Scanne issues/PRs GitHub mentionnant bun/node. |
| `analyze` | `analyze.rs` | Scan + audit + crosslink ML (embeddings `n2b-ai`) multi-repos. |
| `rust` | `rust_cmd.rs` | Scaffold/check/deps/doctor Rust (13 flavors). |
| `app` | `app_cmd.rs` | Scaffold apps Bun (cli/tui/gui/exe) + `bun build --compile`. |
| `bin` | `bin_cmd.rs` | Scaffold plugin natif Bun.build / MDX→JSX / module WASM. |
| `win32` · `linux` | `win32_cmd.rs` · `linux_cmd.rs` | Scaffold projets Bun bas-niveau (FFI Rust, inline C). |
| `wasm` | `wasm_cmd.rs` + `commands/wasm_spec/` | Workflow Rust→WASM→Bun + référence spec WebAssembly. |
| `patch` | `patch.rs` | Wrapper `bun patch`, ou diff unifié du repo (`--self`). |
| `bunpp` | `bunpp_cmd.rs` | Scaffold polyfills `@bun++/node-*` pour les gaps Node de Bun. |
| `llmstxt` | `core/src/llmstxt/` | Génère llms.txt depuis une URL (wrapper siteone-crawler). |

> `mui-to-md3` a été retiré en v0.4.0 (déplacé vers le workspace `mui-rs`). n2b est **Node→Bun only**.

**Point clé** : un scanner ne connaît pas les règles, un rule ne connaît pas les scanners. Le contrat est `Finding` (défini dans `schema/v2.json` → généré dans `schema.rs`). Pour ajouter une règle, soit tu ajoutes un scanner (nouveau type de fichier), soit tu enrichis un scanner existant avec un nouveau regex dans `rules/`.

## Codegen schema-first

`schema/v2.json` est la source de vérité unique. `scripts/generate-schema-types.ts` produit :
- `crates/n2b-types/src/schema.rs` (re-exporté par `n2b-core` via `pub use n2b_types::schema`)
- `packages/n2b-types/src/index.ts` (le type TS consommé par `@n2b/core`)

Les deux fichiers générés sont **commités** et la CI échoue si drift. Jamais éditer ces fichiers à la main — modifier `schema/v2.json` puis relancer le codegen.

La chaîne (figée Phase 0.4) : `cargo-typify` (default flags) pour `schema.rs`, `json-schema-to-typescript --unreachableDefinitions` pour `index.ts`, bannière `@generated` swappée.

## Rollback transactionnel des migrations

`crates/n2b-cli/src/subprocess/bun.rs` fournit `BackupGuard` avec `Drop`. Tout side-effect de `--migrate` (`bun install`, retrait lockfiles rivaux, mutations `package.json`) doit passer par ce guard pour garantir un restore sur panic ou échec subprocess. Ne jamais écrire de side-effect migration en dehors de `commands/migrate.rs`.

## Règles du dépôt parent

- **Bun uniquement** (jamais `node`/`npm`/`npx`/`pnpm`/`yarn`) — c'est ironiquement ce que n2b détecte.
- **CLI Rust au lieu des binaires GNU** (`rg` au lieu de `grep`, `fd` au lieu de `find`, `bat` au lieu de `cat`, etc.).

## Base de connaissance — `docs/` + `upstream/`

Pour piloter la couverture des règles, le repo embarque la doc upstream :

| Dossier | Contenu | Tracké git |
|---|---|---|
| `docs/bun/` | Docs Bun canary (`oven-sh/bun` `docs/`) — markdown strippé des assets | oui |
| `docs/node/` | Docs API Node LTS v24 (`nodejs/node` `doc/api/`) | oui |
| `upstream/bun/` · `upstream/node/` | Clones complets `--depth 1` — source pour mining | non (gitignoré) |

`docs/README.md` documente les versions épinglées et la procédure de régénération.
**Source de vérité pour décider quelle règle ajouter** : `docs/bun/runtime/nodejs-compat.mdx`
(matrice de compat) et `docs/node/*.md` (surface API Node à détecter).

## Gotchas

- **proptest default = 256 cas** pour les 3 property tests sur `scanners/{package_json,source}.rs`. Si un panic historique est découvert, il faut le corriger dans le scanner plutôt que baisser le nombre de cas.
- **Bump de version** : `crates/n2b-core/Cargo.toml`, `crates/n2b-cli/Cargo.toml`, `packages/n2b/package.json` (3 fichiers). `crates/n2b-native` reste à 0.1.0 (ABI v1 gelée, ne bump que sur breaking ABI). Toujours régénérer les baselines après bump car la version apparaît dans JSON/JSONL/SARIF.
- **`n2b rules` retourne un tableau plat** (pas `{rules: [...]}`) — rpb-dashboard le parse ainsi.
- **`--report=md` et `--report=markdown` doivent être équivalents** (test contract dédié).
- Le flag `--aggressive` active les règles `api/*` (réécrit APIs Node en Bun) ; `--migrate` = `--fix --aggressive` + side-effects avec rollback.
