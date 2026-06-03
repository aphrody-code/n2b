# n2b — Node.js -> Bun codemod

`n2b` analyses a Node.js project and reports (or automatically fixes)
incompatibilities with the Bun runtime. It covers:

- Rewriting `npm` / `npx` / `pnpm` / `yarn` -> `bun` / `bunx` in `package.json`
  scripts, GitHub Actions workflows, shell scripts, and Dockerfiles.
- The `node:` prefix on builtin imports (`fs`, `path`, `crypto`, ...).
- Dependencies made redundant by native Bun APIs (`dotenv`, `node-fetch`,
  `uuid`, `better-sqlite3`, `rimraf`, ...).
- Migrating Node idioms to Bun (`fs.readFileSync` -> `Bun.file().text()`,
  `fileURLToPath(import.meta.url)` -> `import.meta.dir`, `node` shebang ->
  `bun`, `actions/setup-node` -> `oven-sh/setup-bun@v2`).
- Detecting rival lockfiles and Node APIs unsupported by Bun.

## Architecture (v0.4.0 — Turborepo style)

```
n2b/
├── schema/v2.json                      <- single source of truth for the JSON contract
├── crates/
│   ├── n2b-core/                       <- orchestrator
│   ├── n2b-types/                      <- data models (Rust)
│   ├── n2b-rules/                      <- rules
│   ├── n2b-scanners/                   <- AST / file scanners
│   ├── n2b-report/                     <- SARIF/JSON render engines
│   ├── n2b-ai/                         <- AI / LLM integration
│   ├── n2b-github/                     <- GitHub integration
│   ├── n2b-cli/                        <- `n2b` binary
│   └── n2b-native/                     <- cdylib FFI
├── packages/
│   ├── n2b/                            <- main TS wrapper (cli)
│   ├── n2b-types/                      <- generated TypeScript types
│   ├── n2b-plugin/                     <- native Bun plugin
│   └── n2b-shims/                      <- native Bun shims
├── turbo.json                          <- global Turborepo configuration
├── scripts/generate-schema-types.ts    <- Rust + TS codegen
└── tests/
    ├── fixture/                        <- test project
    ├── rpb-dashboard-baseline/         <- snapshots
    ├── snapshots/baseline/
    └── compare-baseline.sh
```

## Installation

```bash
# Rust CLI binary
cargo build --release -p n2b
sudo install -m755 target/release/n2b /usr/local/bin/n2b

# TypeScript facade
bun install
```

## CLI usage

```bash
# Dry-run audit (exit 1 if there are findings)
n2b .

# Apply the safe fixes
n2b . --fix

# Aggressive migration (rewrites Node APIs -> Bun)
n2b . --aggressive

# Full migration (--fix --aggressive + side effects: bun install, removes pnpm-lock.yaml, etc.)
n2b . --migrate

# Reports
n2b . --report=text                     # default, colourised
n2b . --report=json                     # schema v2 (see schema/v2.json)
n2b . --report=jsonl                    # streamable
n2b . --report=markdown
n2b . --report=sarif                    # GitHub Code Scanning

# Exclusions
n2b . --ignore="**/legacy/**" --ignore="**/fixtures/**"
```

## TypeScript usage — `@n2b/core`

### Subprocess wrapper

```ts
import { scan, rules } from "@n2b/core";

const report = await scan(".", { mode: "check", quiet: true });
console.log(`${report.findings_total} finding(s) in ${report.files_scanned} file(s)`);
```

### Bun plugin (lint at build time)

```ts
import { n2bPlugin } from "@n2b/core";

Bun.plugin(n2bPlugin({ onFindings: "warn" }));
// or "error" to fail builds that have findings
```

### Bun-native shims

```ts
import { env, fs, path, shell } from "@n2b/core/shims";

const DB = env.str("DATABASE_URL", { required: true });
const port = env.int("PORT", { default: 3000 });
const config = await fs.readJson<Config>(path.relativeTo(import.meta, "config.json"));
const result = await shell.run("git rev-parse HEAD");
```

## Rules

| Category | IDs | `--fix` | `--aggressive` |
|---|---|:-:|:-:|
| CLI (`npm`/`pnpm`/`yarn`/`npx`) | `cli/*` | yes | yes |
| `node:` prefix | `imports/node-prefix` | yes | yes |
| Shebang | `shebang/node` | yes | yes |
| GitHub Actions | `ci/*` | yes | yes |
| `package.json` (scripts, engines, deps) | `pkg/*` | partial | partial |
| Rival lockfiles | `lock/rival` | report | report |
| Package replacements | `imports/bun-native` | report | yes (`bun:` / `node:` specifiers) |
| Node APIs -> Bun | `api/*` | report | yes |

List the rules: `n2b rules` or `n2b rules --report=json`.

## Development

```bash
# Full test suite
turbo run test                            # full TS + Rust test via Turbo
cargo test --workspace                    # Rust tests (schema + contract + proptest)
bash tests/compare-baseline.sh            # CLI-as-API baseline (13 assertions)

# Quality (lint & format)
turbo run //#quality                      # global oxlint, oxfmt, taplo
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
turbo run typecheck                       # TypeScript type-checking

# Regenerate types from the schema
bun run codegen:schema
```

## Exit codes

- `0` — no findings, or fix/aggressive mode applied successfully
- `1` — findings in check mode (dry-run)
- `2` — error (invalid flag, internal crash)

## Reference documentation

The rules are derived from the official Bun docs (`runtime/nodejs-compat.md`,
`runtime/bun-apis.md`, `pm/`, `guides/util/import-meta-dir.md`).

## AI integration (Claude Code & Gemini CLI)

The official AI integration is packaged in this repository (a Claude Code
plugin and a Gemini CLI extension):

- **Claude Code plugin**: declared in `.claude-plugin/plugin.json`.
- **Gemini CLI extension**: declared in `gemini-extension.json`.
- **Skills (shared)**: `skills/` — shared between both assistants.
- **Commands (asymmetric)**: the commands in `commands/` exist in two formats.
  `.md` files are Claude Code-only, while `.toml` files are Gemini CLI-only.
- **Agents & output styles**: the `agents/` and `output-styles/` folders are
  Claude Code-specific. Gemini CLI ignores them (which can produce harmless
  "Invalid tool name" warnings at Gemini startup).
- **Bundled docs**: `docs/n2b/` (this README, CHANGELOG, STRUCTURE, roadmap) +
  `docs/bun-official/` (329 official `.mdx` files).

The plugin is **project-agnostic** — reusable in any Node -> Bun codebase.

## Contributing

The branching model, PR flow, and test commands are described in
[`CONTRIBUTING.md`](CONTRIBUTING.md). In short: GitHub Flow, PRs against `main`,
one PR per refactor phase, Conventional Commits.

Security policy: [`SECURITY.md`](SECURITY.md). Community standards:
[`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## License

[Apache 2.0](LICENSE) (c) 2026 aphrody-code contributors.
