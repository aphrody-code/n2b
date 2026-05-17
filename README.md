# n2b — Node.js → Bun codemod

`n2b` analyse un projet Node.js et signale (ou corrige automatiquement) les incompatibilités avec le runtime Bun. Il couvre :

- Réécriture `npm` / `npx` / `pnpm` / `yarn` → `bun` / `bunx` dans les scripts `package.json`, les workflows GitHub Actions, les shells et Dockerfiles.
- Préfixe `node:` sur les imports builtins (`fs`, `path`, `crypto`, …).
- Dépendances redondantes avec les APIs natives Bun (`dotenv`, `node-fetch`, `uuid`, `better-sqlite3`, `rimraf`, …).
- Migration d'idiomes Node → Bun (`fs.readFileSync` → `Bun.file().text()`, `fileURLToPath(import.meta.url)` → `import.meta.dir`, shebang `node` → `bun`, `actions/setup-node` → `oven-sh/setup-bun@v2`).
- Détection de lockfiles concurrents et d'API Node non supportées par Bun.

## Architecture (v0.4.0 - Turborepo Style)

```
n2b/
├── schema/v2.json                      ← source unique du contrat JSON
├── crates/
│   ├── n2b-core/                       ← Orchestrateur
│   ├── n2b-types/                      ← Modèles de données (Rust)
│   ├── n2b-rules/                      ← Règles
│   ├── n2b-scanners/                   ← Scanners AST / Fichiers
│   ├── n2b-report/                     ← Moteurs de rendus SARIF/JSON
│   ├── n2b-ai/                         ← Intégration AI / LLM
│   ├── n2b-github/                     ← Intégration GitHub
│   ├── n2b-cli/                        ← Binaire `n2b`
│   └── n2b-native/                     ← cdylib FFI
├── packages/
│   ├── n2b/                            ← Wrapper TS principal (cli)
│   ├── n2b-types/                      ← Types TypeScript générés
│   ├── n2b-plugin/                     ← Plugin Bun natif
│   └── n2b-shims/                      ← Shims Bun natifs
├── turbo.json                          ← Configuration Turborepo globale
├── scripts/generate-schema-types.ts    ← codegen Rust + TS
└── tests/
    ├── fixture/                        ← projet de test
    ├── rpb-dashboard-baseline/         ← snapshots
    ├── snapshots/baseline/             
    └── compare-baseline.sh             
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
turbo run test                            # Test complet TS et Rust via Turbo
cargo test --workspace                    # Rust tests (schema + contract + proptest)
bash tests/compare-baseline.sh            # baseline CLI-as-API (13 assertions)

# Qualité (Lints & Format)
turbo run //#quality                      # Lancement global oxlint, oxfmt, taplo
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
turbo run typecheck                       # Typage TypeScript

# Régénérer les types depuis le schéma
bun run codegen:schema
```

## Codes de sortie

- `0` — aucun finding, ou mode fix/aggressive appliqué avec succès
- `1` — findings en mode check (dry-run)
- `2` — erreur (flag invalide, crash interne)

## Documentation de référence

Les règles sont dérivées de la doc Bun officielle (`runtime/nodejs-compat.md`, `runtime/bun-apis.md`, `pm/`, `guides/util/import-meta-dir.md`).

## AI Integration (Claude Code & Gemini CLI)

L'intégration officielle AI est packagée dans ce dépôt (plugin Claude Code et extension Gemini CLI) :

- **Claude Code Plugin** : Déclaré dans `.claude-plugin/plugin.json`.
- **Gemini CLI Extension** : Déclarée dans `gemini-extension.json`.
- **Skills (Communs)** : `skills/` — partagés entre les deux assistants.
- **Commands (Asymétrie)** : Les commandes dans `commands/` existent en double format. Les fichiers `.md` sont exclusifs à Claude Code, tandis que les fichiers `.toml` sont exclusifs à Gemini CLI.
- **Agents & Output Styles** : Les dossiers `agents/` et `output-styles/` sont spécifiques à Claude Code. Gemini CLI les ignore (ce qui peut générer des avertissements inoffensifs de type "Invalid tool name" au démarrage de Gemini).
- **Docs bundled** : `docs/n2b/` (ce README, CHANGELOG, STRUCTURE, roadmap) + `docs/bun-official/` (329 `.mdx` officiels).

Le plugin est **agnostique au projet** — réutilisable dans n'importe quel codebase Node → Bun.

Cycle de re-sync (upstream → plugin) :
```bash
rsync -a --delete docs/ ~/vps/agents/bun-agent/docs/n2b/
cp {README,CLAUDE,STRUCTURE,CHANGELOG,CONTRIBUTING,build-your-own-x}.md ~/vps/agents/bun-agent/docs/n2b/
```

## Contribuer

Le modèle de branches, le flux PR et les commandes de test sont décrits dans
[`CONTRIBUTING.md`](CONTRIBUTING.md). En résumé : GitHub Flow, flux PR sur
`main`, une PR par phase de refacto, commits conventionnels.

## License

[Apache 2.0](LICENSE) © 2026 aphrody-code.
