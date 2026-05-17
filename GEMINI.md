# GEMINI.md — bun-agent extension

This context file is loaded by Gemini CLI when the **bun-agent** extension is active. It mirrors `CLAUDE.md` for the Claude Code plugin (same repo, dual-tool parity) but is rephrased for Gemini's tool model and conventions.

## What this extension does

bun-agent ships:
- The **n2b binary** (Rust, `cargo build --release -p n2b` then `sudo install -m755 target/release/n2b /usr/local/bin/n2b`) — Node.js → Bun migration tool with 313 rules, 47 Node modules covered, 90 npm packages mapped.
- **10 agent skills** (`skills/`) discoverable via Gemini's agentskills.io standard:
  - `analyze`, `run`, `dream`, `deploy`, `n2b`, `move` — workflow skills.
  - `gemini-cli-jsx` — migrate Ink/React `.tsx` to Bun's native JSX runtime (414 files in this repo's Pilier 2 fixture).
  - `gemini-cli-cli` — migrate the gemini-cli CLI tooling (bin/scripts/lockfile/sandbox/SEA) to Bun.
  - `green-gate` — full anti-regression pipeline (fmt + build + test + clippy + codegen drift + baselines).
  - `regen-baseline` — regenerate the 7 tracked snapshots after a legitimate output change.
- **7 custom commands** (`commands/*.toml`): `/dream`, `/forget`, `/memory`, `/move`, `/n2b`, `/run`, `/status`.

## How to use

### Run the migration on a project

```bash
@n2b audit                 # dry-run scan, default text output
/n2b 2 packages/cli         # apply Phase 2 (--aggressive) on a subdir
/n2b migrate                # full --migrate with rollback if bun install fails
```

The `n2b` skill activates when you describe a Node→Bun task. Gemini reads `SKILL.md` and the registry context from `crates/n2b-registry/registry/*.toml`.

### Verify before deploy

```bash
/green-gate                 # cargo fmt/build/test/clippy + codegen + baselines
```

Stops at first failure. Use this before `git push origin main` or `sudo install`.

### Migrate gemini-cli itself

```bash
@gemini-cli-jsx             # 414 .tsx Ink/React → Bun native JSX runtime
@gemini-cli-cli             # bin entries + scripts + sandbox Docker + SEA
```

Both skills are grounded on the `tests/targets/gemini-cli/` fixture (Pilier 2).

## Project conventions Gemini must respect

- **Bun only.** Never invoke `node`, `npm`, `npx`, `pnpm`, `yarn` — that's exactly what n2b detects. Use `bun`, `bunx`, `bun --filter`.
- **Rust CLI tools** preferred for large scans (`rg`, `fd`, `tokei`) — but built-in tools are fine for small targeted operations.
- **Frozen contract surfaces** (consumed by `rpb-dashboard` via subprocess): rule IDs, JSON v2 schema, exit codes 0/1/2, ABI cdylib v1. Never modify silently — see `n2b-contract-guard` agent.
- **Schema-first codegen**: `schema/v2.json` is the source of truth. Edit it → run `bun run codegen:schema` → check in the regenerated `crates/n2b-types/src/schema.rs` and `packages/n2b-types/src/index.ts`.
- **Triple safety net for the contract**: `tests/compare-baseline.sh` (byte-diff), `crates/n2b-cli/tests/contract.rs` (assert_cmd + jsonschema), `crates/n2b-cli/src/schema_test.rs` (compile-time `include_str!` roundtrip).

## n2b CLI surface (v0.5.0)

```bash
# Default = scan
n2b <path>                                  # text report
n2b <path> --report=json|jsonl|md|sarif    # alternate formats
n2b <path> --quiet                          # suppress summary
n2b <path> --fix                            # apply safe rewrites
n2b <path> --aggressive                     # apply api/* rewrites (Bun replacements)
n2b <path> --migrate                        # --fix --aggressive + side-effects (BackupGuard rollback)
n2b <path> --migrate --scaffold-polyfills   # also scaffold @bun++/node-* polyfills for compat=missing modules

# Subcommands
n2b rules [--report=json]                   # list known rules (rpb-dashboard parses tableau plat)
n2b prompt                                  # generate LLM-ready migration prompt
n2b audit                                   # GitHub issues/PRs scan
n2b analyze                                 # multi-repo scan + crosslink (n2b-ai embeddings)
n2b rust|app|bin|win32|linux|wasm|patch|bunpp|llmstxt   # scaffold helpers
```

Exit codes: `0` = clean, `1` = findings in check mode (non-zero), `2` = invalid flag/usage.

## When NOT to use bun-agent

- The repo is a **VSCode extension** package — VSCode hosts extensions in its own Node runtime, mark with `n2b.json { "ignore": ["packages/vscode-*/**"] }`.
- The repo is a **Cloudflare Workers / Deno** project — different runtime, n2b's Bun replacements don't apply.

## Reference docs (in this repo)

- `docs/bun/` — Bun canary docs (329 .mdx files)
- `docs/node/` — Node LTS v24 API surface
- `docs/plugin/n2b/` — n2b roadmap, research notes
- `plan/coverage/{modules,apis,packages}.md` — coverage matrices

## Cross-tool parity

This extension is the Gemini CLI mirror of the Claude Code plugin defined in `.claude-plugin/plugin.json`. Skills (`skills/SKILL.md`) and commands (`commands/*.toml`) are co-located so the same source serves both tools. When updating one, update the other.

### Known stderr noise on startup

The `agents/*.md` files are Claude-specific (use Claude tool names: `Read`, `Write`, `Edit`, `Bash`, `Glob`, `Grep`, `Agent`). Gemini scans the same `agents/` directory but expects its own tool names (`read_file`, `write_file`, `replace`, `run_shell_command`, `glob`, `grep_search`). On every Gemini startup you'll see ~25 lines of `[ExtensionManager] Error loading agent ... Invalid tool name`. **This is harmless** — the agent files are intentionally for Claude; skills + commands are the cross-tool primary surface. Suppress them with `2>/dev/null` if needed.

### Hooks parity

Claude Code hooks live in `hooks/hooks.json` (events: `PostToolUse`, `Stop`, etc.). Gemini hooks use slightly different event names (`AfterTool`, `BeforeTool`) and live in `gemini-extension.json` or `~/.gemini/settings.json`. The cargo-fmt + schema-codegen drift hooks are currently Claude-only — see `hooks/hooks.json`. To port to Gemini, translate event names to Gemini equivalents and add a `hooks` block to `gemini-extension.json`.

## 🪟 Windows Cross-Compilation Mandate

1. **MSVC ABI over GNU**: All Windows binaries MUST target `x86_64-pc-windows-msvc`. Use `cargo-xwin` for native MSVC cross-compilation from Linux.
2. **Static CRT**: Force static linking of the C runtime (`-C target-feature=+crt-static`) to ensure zero-dependency executables.
3. **Bun Bytecode**: Use `--bytecode` during `bun build --compile` to optimize startup speed on Windows (if applicable).
4. **Baseline Compatibility**: Always use the `baseline` CPU target to ensure functionality on older VPS and CPU hardware.
