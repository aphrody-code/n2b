# Phase 1 — Registre data-driven

> Corrige **PS3**. Crée `crates/n2b-registry`, migre les règles existantes dans les
> `.toml` **sans changer un seul Rule ID ni une seule sortie**. Refactor pur.
>
> **Dépend de :** Phase 0. **Bloque :** Phases 2, 3, 4.

## Objectif

Sortir les règles du code Rust et les mettre dans un registre de données embarqué,
source unique de vérité. **Critère absolu : baselines octet-à-octet identiques.** Si une
baseline change, c'est un bug de migration, pas une amélioration.

## Travaux

### 1.1 — Créer le crate `n2b-registry`

**Fichiers.** Nouveau crate complet (cf. [02-architecture-cible.md](../02-architecture-cible.md) §2).

```
crates/n2b-registry/
  Cargo.toml          # deps: serde, toml, once_cell, regex ; dep inter-crate: n2b-types, n2b-util
  registry/
    modules.toml
    apis.toml
    packages.toml
    cli.toml
    globals.toml
  src/
    lib.rs
    schema.rs         # ModuleEntry, ApiEntry, PackageEntry, CliEntry, GlobalEntry + enums
    registry.rs       # Lazy<Vec<...>> via include_str! + validation
    engine.rs         # squelette (rempli en Phase 2 pour l'AST)
```

`Cargo.toml` workspace : ajouter `crates/n2b-registry` aux `members`, ajouter `toml` aux
`[workspace.dependencies]`.

### 1.2 — Définir les structs (`schema.rs`)

Les structs de [03-registre-spec.md](../03-registre-spec.md). Enums `Compat`,
`Severity`, `Rewrite`, `Confidence`, `Strategy`. `#[derive(Deserialize)]` partout.

### 1.3 — Migrer les données existantes vers les `.toml`

**C'est l'étape sensible.** Transcription mécanique, zéro réinterprétation :

| Source Rust actuelle | Cible `.toml` | Volume |
|---|---|---|
| `bun_apis.rs:53-591` — `RULES: Vec<ApiRule>` | `apis.toml` | 72 entrées `api/*` + 2 `next/*` |
| `node_imports.rs:86-661` — `BUN_REPLACEMENTS` | `packages.toml` | ~90 entrées |
| `node_imports.rs:7-70` — `BUILTINS` | `modules.toml` | ~50 modules (statut compat = à remplir Phase 3/4, ici juste la liste) |
| `cli_commands.rs` — les 41 mappings | `cli.toml` | 41 entrées |
| `api/dirname-esm`, `api/filename-esm` | `globals.toml` | 2 entrées (+ extensions Phase 4) |

**Règle de transcription** : chaque champ Rust → champ TOML, à l'identique. Le
`template` d'une `ReplaceKind::Template` → champ `template`. Un `ReplaceKind::None` →
`rewrite = "manual"` sans `codemod_hint` *encore* (le `codemod_hint` est ajouté en
Phase 5). Un `ReplaceKind::Static` → `rewrite = "template"` avec `template` constant.

Pour cette phase, `compat` et `severity` des entrées sont fixés à des valeurs qui
**reproduisent la sortie actuelle** (la sévérité réelle pilotée par compat arrive en
Phase 3). On encode l'existant, on ne l'améliore pas.

### 1.4 — Brancher `registry.rs`

`Lazy<Vec<ModuleEntry>>` etc. via `include_str!` + validation (cf.
[03-registre-spec.md](../03-registre-spec.md) §7). Test `cargo test -p n2b-registry`
qui force le chargement → valide IDs uniques, `docs` existants, regex compilent.

### 1.5 — `n2b-rules` consomme le registre

`bun_apis.rs`, `node_imports.rs`, `cli_commands.rs` : remplacer les `Vec`/`HashMap`
statiques en dur par des **lectures du registre**. Les fonctions
`apply_bun_api_rules` / `apply_node_import_rules` / `apply_cli_rules` gardent leur
signature publique (les scanners ne voient rien changer) — seule leur source de données
change.

`n2b-rules` ajoute `n2b-registry` à ses dépendances.

## Critères d'acceptation

- **`bash tests/compare-baseline.sh` — baselines octet-à-octet identiques.** Aucune
  régénération autorisée en Phase 1. C'est un refactor invisible.
- `cargo test --workspace` vert, dont le nouveau `n2b-registry`.
- `cargo clippy --workspace --all-targets -- -D warnings`.
- Aucun Rule ID modifié — vérifiable par `n2b rules --report=json` diff contre la
  baseline `rules.json`.
- `git grep 'BUN_REPLACEMENTS\|RULES: Vec<ApiRule>'` ne retourne plus de **définition**
  de données en dur dans `n2b-rules` (seulement des usages du registre).

## Commits attendus

```
feat(n2b-registry): nouveau crate — registre data-driven (.toml embarqués)
refactor(n2b-rules): bun_apis lit apis.toml au lieu de RULES en dur
refactor(n2b-rules): node_imports lit packages.toml + modules.toml (résout PS3)
refactor(n2b-rules): cli_commands lit cli.toml
```

## Vérification anti-régression spécifique

Avant de committer, lancer le scan sur `test/fixture/` et `rpb-dashboard` dans **tous**
les formats et `diff` contre les baselines. Le moindre octet de différence = la
transcription `.toml` a divergé de la source Rust → corriger le `.toml`, pas la baseline.

```bash
for f in text json jsonl md sarif; do
  ext=$f; [[ $f == text ]] && ext=txt
  ./target/release/n2b test/fixture --report=$f | diff - tests/snapshots/baseline/fixture.$ext
done
```

## Risques

| Risque | Mitigation |
|---|---|
| Transcription `.toml` introduit une divergence subtile (ordre, échappement regex) | diff baseline systématique ; transcrire par petits lots, committer fréquemment |
| Les regex TOML nécessitent un échappement différent du Rust | tester chaque regex au chargement (`registry.rs` valide la compilation) |
| `include_str!` gonfle le binaire | les `.toml` font quelques dizaines de Ko — négligeable vs `docs/` déjà embarqués nulle part dans le binaire |
| Ordre de matching change la sortie (overlap) | `apply_edits` (Phase 0) est déterministe ; conserver l'ordre d'itération du registre = ordre des `[[...]]` dans le `.toml` |
