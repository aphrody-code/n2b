# n2b — Node.js → Bun codemod

`n2b` analyse un projet Node.js et signale (ou corrige automatiquement) les incompatibilités avec le runtime Bun. Il couvre :

- Réécriture `npm` / `npx` / `pnpm` / `yarn` → `bun` / `bunx` dans les scripts `package.json`, les workflows GitHub Actions, les shells et Dockerfiles.
- Préfixe `node:` sur les imports builtins (`fs`, `path`, `crypto`, …).
- Dépendances redondantes avec les APIs natives Bun (`dotenv`, `node-fetch`, `uuid`, `better-sqlite3`, `rimraf`, …).
- Migration d'idiomes Node → Bun (`fs.readFileSync` → `Bun.file().text()`, `fileURLToPath(import.meta.url)` → `import.meta.dir`, shebang `node` → `bun`, `actions/setup-node` → `oven-sh/setup-bun@v2`).
- Détection de lockfiles concurrents et d'API Node non supportées par Bun.

## Architecture (v0.3.0)

```
n2b/
├── schema/v2.json                      ← source unique du contrat JSON
├── crates/
│   ├── n2b-core/                       ← lib Rust : scanners, règles, report
│   ├── n2b-cli/                        ← binaire `n2b`
│   └── n2b-native/                     ← cdylib FFI (find_newlines_u16)
├── packages/n2b/                       ← @n2b/core (façade TS)
│   └── src/
│       ├── cli.ts                      ← scan()/rules()/promptMarkdown()
│       ├── plugin.ts                   ← Bun.plugin()
│       ├── ffi.ts                      ← computeLineOffsets (bun:ffi → cdylib)
│       ├── schema.ts                   ← types générés depuis schema/v2.json
│       └── shims/                      ← env / fs / path / shell (Bun-native)
├── scripts/generate-schema-types.ts    ← codegen Rust + TS
└── tests/
    ├── fixture/                        ← projet de test couvrant toutes les règles
    ├── rpb-dashboard-baseline/         ← snapshots CLI-as-API
    ├── snapshots/baseline/             ← snapshots fixture
    └── compare-baseline.sh             ← verrou contre régression
```

## Installation

```bash
# Binaire CLI Rust
cargo build --release -p n2b
sudo install -m755 target/release/n2b /usr/local/bin/n2b

# Façade TypeScript
bun install
```

## Usage CLI

```bash
# Audit dry-run (exit 1 si findings)
n2b .

# Appliquer les corrections sûres
n2b . --fix

# Migration agressive (réécrit les APIs Node → Bun)
n2b . --aggressive

# Migration complète (--fix --aggressive + side-effects : bun install, retrait pnpm-lock.yaml, etc.)
n2b . --migrate

# Rapports
n2b . --report=text                     # défaut, colorisé
n2b . --report=json                     # schéma v2 (voir schema/v2.json)
n2b . --report=jsonl                    # streamable
n2b . --report=markdown
n2b . --report=sarif                    # GitHub Code Scanning

# Exclusions
n2b . --ignore="**/legacy/**" --ignore="**/fixtures/**"
```

## Usage TypeScript — `@n2b/core`

### Wrapper subprocess

```ts
import { scan, rules } from "@n2b/core";

const report = await scan(".", { mode: "check", quiet: true });
console.log(`${report.findings_total} finding(s) in ${report.files_scanned} file(s)`);
```

### Bun plugin (lint au build)

```ts
import { n2bPlugin } from "@n2b/core";

Bun.plugin(n2bPlugin({ onFindings: "warn" }));
// ou "error" pour faire échouer les builds qui ont des findings
```

### Shims Bun-natifs

```ts
import { env, fs, path, shell } from "@n2b/core/shims";

const DB = env.str("DATABASE_URL", { required: true });
const port = env.int("PORT", { default: 3000 });
const config = await fs.readJson<Config>(path.relativeTo(import.meta, "config.json"));
const result = await shell.run("git rev-parse HEAD");
```

## Règles

| Catégorie | IDs | `--fix` | `--aggressive` |
|---|---|:-:|:-:|
| CLI (`npm`/`pnpm`/`yarn`/`npx`) | `cli/*` | ✓ | ✓ |
| Préfixe `node:` | `imports/node-prefix` | ✓ | ✓ |
| Shebang | `shebang/node` | ✓ | ✓ |
| GitHub Actions | `ci/*` | ✓ | ✓ |
| `package.json` (scripts, engines, deps) | `pkg/*` | partiel | partiel |
| Lockfiles concurrents | `lock/rival` | report | report |
| Remplacements de packages | `imports/bun-native` | report | ✓ (spécifiers `bun:` / `node:`) |
| APIs Node → Bun | `api/*` | report | ✓ |

Lister les règles : `n2b rules` ou `n2b rules --report=json`.

## Développement

```bash
# Tests complets
cargo test --workspace                    # Rust (schema + contract + proptest)
bun test packages/n2b/                    # TS (cli + shims)
bash tests/compare-baseline.sh            # baseline CLI-as-API (13 assertions)

# Lint
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
bun run typecheck

# Régénérer les types depuis le schéma
bun run codegen:schema
```

## Codes de sortie

- `0` — aucun finding, ou mode fix/aggressive appliqué avec succès
- `1` — findings en mode check (dry-run)
- `2` — erreur (flag invalide, crash interne)

## Documentation de référence

Les règles sont dérivées de `/home/ubuntu/rsbun/docs/bun-docs/` :

- `runtime/nodejs-compat.md` — matrice de compatibilité Node.js
- `runtime/bun-apis.md` — catalogue des APIs natives Bun
- `pm/` — `bun install`, `bunx`, `bun add`
- `guides/util/import-meta-dir.md` — `import.meta.dir` et famille
