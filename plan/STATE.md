# Refactor State

Last update: 2026-05-15 09:00 UTC

## Done

- [x] **Phase 1 — Registre data-driven (PS3)** — commits `11388b6` + `c65bd2b`
  - 1.1 + 1.3 + 1.4 `feat(n2b-registry): nouveau crate` (`11388b6`)
    - schema.rs, registry.rs (5 Lazy<Vec> + validation), engine.rs (squelette)
    - apis.toml (73), packages.toml (94), modules.toml (53), cli.toml (47), globals.toml (vide Phase 4)
    - n2b-types : derive Deserialize sur Severity (consommé par ApiEntry)
  - 1.5 `refactor(n2b-rules): consomme n2b-registry` (`c65bd2b`)
    - 1527 lignes supprimées de n2b-rules (les `Vec`/`HashMap` statiques)
    - Sortie octet-identique : fixture + shenron + gemini-cli, diff baselines vide
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

**Phase 2 — Scanner source AST-first** : non démarrée. Handoff à la session suivante.

**Cibles de test réelles (2026-05-15)** — `plan/test-targets.md` :
- shenron (Pilier 1) → 12 fichiers, 39 findings (0E/6W/33I). Top : `api/process-env` (13), `api/performance-now` (9), `api/child-process-spawn` (5).
- gemini-cli (Pilier 2, cloné dans `tests/targets/gemini-cli`, gitignored) → 1308 fichiers, 3239 findings. Top : `imports/bun-native` (1075), `api/fs-writeFileSync` (335), `api/fs-existsSync` (331), `api/chalk-call` (145), `api/execSync` (139).
- Script `tests/targets/refresh.sh` régénère les deux baselines. Diff vide après Phase 1.

Prochaine action exacte (Phase 2 sub-step 2.1 — `ImportGraph`) :
1. Étendre `crates/n2b-rules/src/imports_ast.rs` : `extract_specifiers()` → `build_import_graph()` retournant un `ImportGraph` (le squelette est déjà dans `crates/n2b-registry/src/schema.rs` mais doit gagner `BindingKind` + une vraie résolution).
2. `engine.rs` : implémenter `match_rules(MatchInput::Ast { source, imports })` qui consomme `APIS` et filtre par `import_from`.
3. `bun_apis.rs` devient un thin wrapper qui délègue à `engine.rs`.
4. Régénérer baselines fixture/shenron/gemini-cli (changement légitime : faux positifs disparaissent).

Prochaine action exacte (Phase 1 sub-step 1.1 — création du crate `n2b-registry`) :
1. Créer `crates/n2b-registry/Cargo.toml` avec `[lib]`, deps `serde`/`toml`/`once_cell`/`regex` + `n2b-types`/`n2b-util` workspace.
2. Ajouter `crates/n2b-registry` aux `members` de `/home/ubuntu/n2b/Cargo.toml` (déjà couvert par `crates/*` — vérifier).
3. Ajouter `toml = "0.8"` aux `[workspace.dependencies]` du root `Cargo.toml`.
4. Scaffolder `src/{lib.rs,schema.rs,registry.rs,engine.rs}` selon `plan/02-architecture-cible.md` §2 et `plan/03-registre-spec.md`.

Données à migrer (zero-drift requirement — diff baseline doit rester vide) :
- `crates/n2b-rules/src/bun_apis.rs` static `RULES` (72 entrées api/* + 2 next/*) → `registry/apis.toml`
- `crates/n2b-rules/src/node_imports.rs` static `BUILTINS` (47 modules) → `registry/modules.toml`
- `crates/n2b-rules/src/node_imports.rs` static `BUN_REPLACEMENTS` (~40 entrées) → `registry/packages.toml`
- `crates/n2b-rules/src/cli_commands.rs` static `MAPPINGS` (~50 entrées cli/*) → `registry/cli.toml`

## Decisions

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
