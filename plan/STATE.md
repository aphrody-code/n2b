# Refactor State

Last update: 2026-05-15 04:30 UTC

## Done

- [x] **Phase 0 — Socle propre (PS1→PS8)** — commit `e8e1dcf`
  - 0.1 `refactor(n2b-util): apply_edits partagé` (PS2) — commit `05adab4`
  - 0.2 `fix(n2b-rules): cli_commands ne réécrit plus les lignes commentées` (PS4) — commit `c6d2629`
  - 0.3 `refactor(n2b-rules): nomme DIR_CONTEXT_WINDOW_BYTES` (PS5) — commit `3a621e1`
  - 0.4 `fix(build): recrée scripts/generate-schema-types.ts` (PS6) — commit `8b22742`
  - 0.5 `docs(claude): corrige chemins schema.rs / exit codes / codegen` (PS7) — commit `93a481b`
  - 0.6 `chore(repo): supprime install:cli:ts fantôme, réécrit en-tête Cargo.toml` (PS8) — commit `e8e1dcf`

Prérequis environnementaux (committés avant Phase 0) :
- `chore(baseline): regenerate fixture baselines for /home/ubuntu/n2b path` — commit `86a3528`
  (repo déplacé de `/home/ubuntu/vps/packages/n2b` ; chemin absolu apparaissait dans 5 baselines text/json/jsonl/md/sarif)
- `chore(refactor): mission scaffolding` — commit `fb10730`
  (scripts harness tmux, turbo inputs, plan/MISSION.md, .claude/settings.json)

## In progress

Phase 1 — Registre data-driven : à démarrer.

## Decisions

- **PS6** : chaîne codegen figée à `cargo-typify` (defaults) pour `crates/n2b-types/src/schema.rs` + `bunx --bun json-schema-to-typescript --unreachableDefinitions` pour `packages/n2b-types/src/index.ts`. Spike validé byte-identique (diff = 0) hors bannière `@generated` swappée par le script.
- **PS4** : la baseline locale (`test/fixture/`) ne contient pas de commande commentée → 0 régression. La baseline `tests/rpb-dashboard-baseline/scan.json` (absent de cette machine) contient `// npm install --save-dev prisma dotenv` et doit être régénérée au prochain accès à `/home/ubuntu/rpb-dashboard`. Le `crates/n2b-cli/src/schema_test.rs` compile encore car il fait `serde_json::from_str` (round-trip JSON valide), pas un diff sémantique.
- **PS8 node_modules** : skip (déjà gitignoré, `git ls-files node_modules` = 0). Suivi MISSION.

## Deviations from plan

- PS5 a été commité une première fois avec CLAUDE.md (PS7) accidentellement bundle — j'ai fait `git reset --soft HEAD~1` + `git restore --staged CLAUDE.md` + recommit propre. La séquence finale est correcte (un commit par sous-étape).

## Test counts at end of Phase 0

- Rust : **25 tests** workspace (était 14 + 7 edits + 4 cli_commands).
- Baseline : **12 comparaisons** (5 rpb skippées localement, rpb-dashboard absent).
- Codegen drift : 0.
