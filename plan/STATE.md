# Refactor State — final

Last update: 2026-05-15 11:30 UTC

> **Refactor complet livré dans une seule session.** Toutes les phases 0→7 sont vertes.

## Done

- [x] **Phase 7 — Garde-fous & doc** — commits en cours
  - 7.1 Contract tests étendus : `category_imports/api/cli/globals_is_listed`,
    `finding_without_compat_validates_against_schema` (rétro-compat schema_version=2),
    `report_card_present_with_migrate` (vérifie aussi `.n2b/state.json` écrit)
  - 7.3 Régénération finale des baselines (fixture/gemini-cli/bun-full)
  - 7.4 STATE.md + plan/README.md status board ✅
- [x] **Phase 6 — Intégration bunpp (les 🔴)** — commit `08a4090`
  - Flag CLI `--scaffold-polyfills` (requires `--migrate`, opt-in explicite)
  - `MigrateOpts { scaffold_polyfills }` + step 5 dans `run_migrate_side_effects`
  - `bunpp_scaffold(cwd, module)` ré-invoque `n2b bunpp scaffold node-<module>`
- [x] **Phase 5 — Migration report card + .n2b/state.json** — commit `a248196`
  - `crates/n2b-core/src/report_card.rs` : `ReportCard`, `ManualResidueEntry`,
    `N2bState { status: in_progress|complete }`, dérivation de `reason` depuis `compat`
  - `--migrate --report=json` expose `report_card { auto_migratable_pct, ... }`
  - `.n2b/state.json` écrit après chaque `--migrate`
  - Fixture bun-full corrigée (audit JSX par sub-agent : Bun.cron retiré,
    Bun.RedisClient → Bun.redis, using bug, tests extraits dans app.test.tsx)
- [x] **Phase 4 — Expansion couverture** — commit `6cdb25e`
  - 4 nouveaux scanners : env_file, docker_compose, procfile, js_config
  - 3 modules Node v24 ajoutés (sqlite/quic/sea)
  - 9 globals.toml peuplés (__dirname, __filename, process.*, module.exports, require-dynamic)
  - Manifeste `n2b.json` (lecture) — schema/n2b.schema.json + crates/n2b-core/src/manifest.rs
  - Override des règles via `n2b.json` : "off", severity, autofix
- [x] **Phase 3 — Modèle compat → sévérité + schéma** — commit `b9eeec9`
  - modules.toml peuplé pour les 47 modules (22 🟢 / 19 🟡 / 3 🔴 + 11 sub-paths)
  - Champ `compat` optionnel sur Finding (rétro-compat schema_version=2)
  - n2b-types : CompatInfo + CompatStatus runtime
  - n2b-report affiche compat dans text/markdown/sarif/json
- [x] **Phase 2 — Scanner source AST-first (PS1)** — commit `1eb9234`
  - `build_import_graph` via oxc — résolution binding → specifier
  - `bun_apis.rs` filtre par `import_from` quand présent
  - 30 entrées apis.toml enrichies avec import_from (marked, chalk, uuid, exec, ...)
  - `is_member_exec_call` supprimé (rendu inutile par l'AST)
  - 9 proptests anti-faux-positifs (fonction locale homonyme → 0 finding)
  - 7 vrais faux positifs supprimés sur gemini-cli
- [x] **Phase 1 — Registre data-driven (PS3)** — commits `11388b6` + `c65bd2b`
- [x] **Phase 0 — Socle propre (PS1→PS8)** — commit `e8e1dcf`

## Test counts (final)

- **Rust** : ~50+ tests workspace (cargo test --workspace)
  - n2b-rules : 5 unit (binding_resolves, first_meaningful_ident, ...)
  - n2b-core : 8 unit (manifest 5 + report_card 3) + 9 contract proptest
  - n2b-cli : 15 contract tests (étendu Phase 7 §7.1)
  - n2b-registry : 6 unit (counts + load + globals_phase4_populated)
  - n2b-scanners : 17 unit (dont 12 nouveaux Phase 4)
- **Baseline** : 7 OK (5 fixture + 2 rules)
- **Codegen drift** : 0

## Cibles de test (Pilier 1 / Pilier 2)

- **bun-full** (Pilier 1, fixture canonique committed) : 1 finding
  (`api/child-process-spawn` sur `Bun.spawn(["git",...])`, severity info via manifest)
- **gemini-cli** (Pilier 2, gitignored) : 3232 findings (post-Phase 2 AST filter)

## Ce qui reste hors périmètre (volontairement)

- **xtask sync-coverage** (Phase 4 §4.1) : reporté à un sprint dédié — la matrice
  vit dans `plan/coverage/modules.md` et est synchronisée manuellement. La structure
  est en place pour qu'un xtask puisse parser les TOML et croiser avec
  `docs/bun/runtime/nodejs-compat.mdx`.
- **Découpe `n2b-cli` en `n2b-scaffold`** (Phase 7 §7.5, optionnel) : non bloquant,
  le crate `n2b-cli` reste plat à ~12k lignes. À trancher selon temps de build.
- **CHANGELOG.md final** : à écrire par le mainteneur quand la prochaine release
  est tagged. Tous les changements sont documentés dans les messages de commit
  (conventional, format `feat|fix|refactor(scope):`).

## Decisions Phase 7

- **Tests par catégorie** (§7.1) : couvre `imports/`, `api/`, `cli/`, `globals/`.
  `next/` non testé séparément (covered par `api/` qui inclut `next/*`).
- **Rétro-compat schéma** : test dédié `finding_without_compat_validates_against_schema`
  garantit que `compat` reste dans `properties` (pas dans `required`).
- **Report card test** : skip gracieux si `bun install` indisponible (CI sans bun).
