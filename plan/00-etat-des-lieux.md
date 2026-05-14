# 00 — État des lieux mesuré

> Audit factuel du dépôt au 2026-05-14, commit `3652724`. Tous les chiffres sont issus
> d'un scan croisé du code, pas d'estimations.

## 1. Le workspace Cargo — 10 crates, 21 586 lignes Rust

Workspace virtuel, `resolver = "3"`, `edition = "2024"`, `rust-version = "1.85"`,
`members = ["crates/*"]`. **Aucun crate `xtask`.** Aucun `build.rs`.

| Crate | Version | Rôle | LoC `src/` | Dépend de (inter-crate) |
|---|---|---|---|---|
| `n2b-types` | 0.5.0 | Types partagés + `schema.rs` généré (1076 l.) | 1 151 | — |
| `n2b-util` | 0.5.0 | Utilitaires bas-niveau (newlines, helpers) | 52 | types |
| `n2b-rules` | 0.5.0 | Regex/IDs de règles + parsing AST imports (oxc) | 2 046 | types, util |
| `n2b-scanners` | 0.5.0 | 18 scanners, un par type de fichier | 3 632 | types, util, rules |
| `n2b-report` | 0.5.0 | Rendu text/json/jsonl/markdown/sarif | 444 | types, ai, util |
| `n2b-ai` | 0.5.0 | Crosslink/embeddings ML | 251 | — |
| `n2b-github` | 0.5.0 | Wrapper API GitHub (octocrab) | 39 | — |
| `n2b-core` | 0.5.0 | Moteur : engine walk, `--migrate`, audit, llmstxt | 1 980 | les 7 libs |
| `n2b-cli` (pkg `n2b`) | 0.5.0 | Façade CLI, dispatch 13 subcommands, `[[bin]]` | ~12 000 | core uniquement |
| `n2b-native` (pkg `node2bun-native`) | 0.1.0 | `cdylib`, FFI Bun, ABI v1 gelée | 66 | — (isolé) |

Plus gros fichiers : `wasm_spec/codegen.rs` (1673), `rust_cmd.rs` (1431),
`package_json.rs` (1109), `schema.rs` (1076 généré), `bunpp_cmd.rs` (762),
`node_imports.rs` (762), `cargo_toml.rs` (734), `app_cmd.rs` (724), `bun_apis.rs` (724),
`wasm_cmd.rs` (863), `validator.rs` (731).

### DAG de dépendances

```
        n2b-types ──────────────┐ (feuille, 0 dep)
           │  │  │              │
           │  │  └──> n2b-util  │ (dep: types)
           │  │         │  │    │
           │  └─────────┤  │    │
           │            v  │    │
           │        n2b-rules  │ (dep: types, util)
           │            │      │
           │            v      │
           │       n2b-scanners (dep: types, util, rules)
           │            │
  n2b-ai ──┼──> n2b-report (dep: types, ai, util)
  (0 dep)  │            │
 n2b-github├────────────┤  (0 dep)
  (0 dep)  │            │
           v            v
        n2b-core <───────────────  (dep: les 7 libs)
           │
           v
        n2b-cli  (bin "n2b" — dep: core uniquement)

  n2b-native  (cdylib — ISOLÉ, 0 dep inter-crate)
```

Hub = `n2b-core`. `n2b-cli` ne voit que `core`. `n2b-native` est hors graphe.

## 2. Les packages TS — façade thin

Tous en 0.5.0, `type: module`, `engines.bun >= 1.3.0`, `main`/`types` pointent
directement sur `src/*.ts`.

| Package | name | Rôle |
|---|---|---|
| `packages/n2b-types` | `@n2b/types` | Types TS générés depuis `schema/v2.json` |
| `packages/n2b-shims` | `@n2b/shims` | Polyfills Bun (`env`, `fs`, `path`, `shell`) |
| `packages/n2b-plugin` | `@n2b/plugin` | `Bun.plugin` + bindings FFI `libnode2bun_native` |
| `packages/n2b` | `@n2b/core` | Façade thin : wrappe le binaire Rust `n2b` |

## 3. Ce que n2b reconnaît aujourd'hui

### 3.1 Imports — `crates/n2b-rules/src/node_imports.rs` (762 l.)

- **`BUILTINS`** (`:7-70`) : HashSet des builtins Node + sous-chemins (`fs/promises`,
  `stream/web`, `timers/promises`…).
- **2 Rule IDs seulement** :
  - `imports/node-prefix` (`:698`) — builtin sans préfixe `node:`. **Réécriture
    effective**, appliquée même en `--fix`. `autofix: true`.
  - `imports/bun-native` (`:717`) — dep npm remplaçable. **Warning**, fix conditionnel :
    appliqué seulement si `aggressive && replacement starts_with("bun:"|"node:")`.
- **`BUN_REPLACEMENTS`** (`:86-661`) : ~90 entrées `HashMap<&str, BunReplacement>`. Mais
  seuls les replacements `bun:*` + `aggressive:true` sont réellement réécrits → **~8
  paquets effectivement migrés** (`sqlite3`, `better-sqlite3`, `jest`, `mocha`,
  `vitest`, `@jest/globals`, `ts-jest`, `jest-circus`). Les ~82 autres sont du warning
  pur. Détail complet : [coverage/packages.md](coverage/packages.md).

### 3.2 APIs — `crates/n2b-rules/src/bun_apis.rs` (724 l.)

- `RULES: Vec<ApiRule>` (`:53-591`) — **72 règles** `api/*` + 2 `next/*`.
- `ReplaceKind` : `None` | `Static` | `Template($1..$n)`.
- **13 règles seulement** ont une réécriture effective. Les 59 autres sont
  warning/info pur. Détail : [coverage/apis.md](coverage/apis.md).
- 100 % regex — aucune corrélation à l'origine d'import (cf. PS1).

### 3.3 CLI — `crates/n2b-rules/src/cli_commands.rs` (382 l.)

- 41 mappings npm/pnpm/yarn/npx → bun. Solide.
- **Bug** : `apply_cli_rules` réécrit les lignes commentées alors que les findings les
  filtrent (cf. PS4).

### 3.4 Scanners — `crates/n2b-scanners/src/` (18 scanners, 3 632 l.)

| Scanner | Fichiers | LoC |
|---|---|---|
| `package_json.rs` | `package.json` | 1109 |
| `cargo_toml.rs` | `Cargo.toml` | 734 |
| `next_config.rs` | `next.config.*` | 365 |
| `tsconfig.rs` | `tsconfig.*` | 252 |
| `npmrc.rs` | `.npmrc`, `.yarnrc[.yml]`, `.pnpmrc` | 150 |
| `husky.rs` | `.husky/*` | 112 |
| `components_json.rs` | `components.json` | 100 |
| `bunfig.rs` | `bunfig.toml` | 99 |
| `pnpm_workspace.rs` | `pnpm-workspace.yaml` | 92 |
| `turbo_json.rs` | `turbo.json` | 89 |
| `tauri_conf.rs` | `tauri.conf.json[5]` | 77 |
| `workflows.rs` | `.github/workflows/*` | 69 |
| `dockerfile.rs` | `Dockerfile*` | 52 |
| `shebang.rs` | (appelé par source) | 38 |
| `lockfile.rs` | `package-lock.json`, `yarn.lock`… | 31 |
| `nvmrc.rs` | `.nvmrc`, `.node-version` | 27 |
| `source.rs` | `.js .jsx .ts .tsx .mjs .cjs .mts .cts` | 20 |
| `shell.rs` | `.sh .bash .zsh`, `Dockerfile`, `Makefile`, `Justfile` | **6** |

`shell.rs` est un **stub de 6 lignes** qui délègue à `apply_cli_rules`. Aucune détection
shell réelle (`node script.js`, `nvm use`, `NODE_OPTIONS`).

### 3.5 Le seul code AST — `crates/n2b-rules/src/imports_ast.rs` (174 l.)

Utilise `oxc_parser` / `oxc_ast` / `oxc_ast_visit`. `extract_specifiers()` résout les
imports/exports ESM statiques, les imports dynamiques, et les `require()` CJS via un
visitor. **Appelé uniquement par `node_imports.rs:680`.** `bun_apis.rs` et
`cli_commands.rs` restent 100 % regex.

## 4. Le pipeline de scan

```
entry → n2b-cli/src/main.rs (dispatch only)
      → cli::dispatch::run_from_args (args.rs → enum Cmd)
      → commands/scan.rs (défaut) ou subcommands
          → n2b_core::run::run(opts)
              → engine walk (ignore + globset + crossbeam)
              → dispatch scanner par extension/nom (run.rs:133-236)
                  n2b-scanners/*.rs → Vec<Finding>
                  n2b-rules/*.rs → regex/IDs partagés
              → n2b-report (text/json/jsonl/markdown/sarif)
```

`SOURCE_EXTS`, `SHELL_EXTS`, `SHELL_NAMES` définis dans `run.rs:29-31`.
`--migrate` = `--fix --aggressive` + side-effects via `BackupGuard` (`commands/migrate.rs`).
`--aggressive` et `--migrate` aboutissent au **même `Mode::Aggressive`** (`scan.rs:11-16`).

## 5. Le contrat externe gelé

Consommé par `/home/ubuntu/rpb-dashboard` via subprocess.

| Surface | Vérité | État |
|---|---|---|
| Flags/subcommands CLI | `n2b-cli/src/cli/args.rs` (743 l.) | 13 subcommands + scan défaut |
| Format JSON v2 | `schema/v2.json` (175 l., draft-07) | `Finding` a `additionalProperties: false` ; `schema_version` enum `[2]` |
| Rule IDs | `n2b-rules/src/*.rs` | `imports/*`, `api/*`, `cli/*`, `next/*` + IDs scanners |
| Exit codes 0/1/2 | **`n2b-cli/src/commands/scan.rs:52-63`** (et non `dispatch.rs`) | 2 si erreur sévère, 1 si findings en `Check`, 0 sinon |
| ABI cdylib v1 | `n2b-native/src/lib.rs` (66 l.) | `find_newlines_u16`, `node2bun_abi_version() → 1` |

Filet de sécurité : `tests/compare-baseline.sh` (12 comparaisons octet-à-octet) +
`crates/n2b-cli/tests/contract.rs` (9 tests `assert_cmd` + validation `jsonschema`).
3 proptests `n2b-core` (256 cas chacun).

## 6. Ce qui est cassé ou désynchronisé — découvertes de l'audit

Ces points ne sont **pas** dans `REFACTOR_PLAN.md` et changent le périmètre :

1. **Le codegen schéma est cassé.** `scripts/generate-schema-types.ts` n'existe pas, le
   dossier `scripts/` n'existe pas. `crates/n2b-core/src/schema.rs` (référencé par
   CLAUDE.md) n'existe pas — le vrai fichier généré est
   `crates/n2b-types/src/schema.rs`. `packages/n2b/src/schema.ts` n'existe pas. Le
   `package.json` racine référence encore `bun run scripts/generate-schema-types.ts`
   (scripts `codegen:schema` / `codegen:schema:check` morts). → **PS6**.

2. **CLAUDE.md est désynchronisé** sur 4 points : chemin de `schema.rs`, codegen,
   localisation des exit codes, existence de `packages/n2b/src/schema.ts`. → **PS7**.

3. **Cruft repo** : `node_modules/` est commité à la racine ; le `package.json` racine
   référence `@n2b/cli` / `packages/n2b-cli/dist/node2bun` qui n'existe pas ; l'en-tête
   de `Cargo.toml` décrit un layout `rust/`+`native/` obsolète ; `n2b-cli` est un crate
   plat de ~12 000 lignes mélangeant 13 subcommands. → **PS8**.

4. **`docs/bun/project/roadmap.mdx` est un stub de 9 lignes** qui renvoie à l'issue
   GitHub #159. La source « roadmap » du plan original est inexploitable localement. À
   remplacer par : mining de l'issue via `n2b audit`, ou exploitation du
   `CHANGELOG_V24.md` Node (qui indique ce que Bun *devra* rattraper).

5. **`docs/bun/runtime/nodejs-compat.mdx` est calé sur Node v23**, pas v24. `node:quic`
   (nouveau Node v24) n'y figure pas. Plusieurs sous-APIs v24 sont des angles morts. La
   couverture n2b doit gérer ce décalage de version doc Bun ↔ réalité Node.

6. **`upstream/bun/src/js/node/` contient 62 fichiers**, dont `repl.ts` et
   `trace_events.ts` (marqués 🔴 dans le mdx mais existant en stub). Le statut 🔴 du mdx
   n'est donc pas fiable seul — il faut croiser `mdx ↔ src`.

→ Détail et remèdes : [01-problemes-structurels.md](01-problemes-structurels.md).
