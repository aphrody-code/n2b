# Plan de refactorisation n2b — résumé exécutif

> **Le plan détaillé vit dans [`plan/`](plan/).** Ce fichier n'est qu'un résumé d'une
> page. Pour exécuter le refactor, lire [`plan/README.md`](plan/README.md) puis les
> phases dans [`plan/phases/`](plan/phases/).

## Objectif — deux piliers

1. **Couverture totale (entrée)** — reconnaître *toute* surface Node.js migrable.
2. **Cross-compilation complète (sortie)** — chaque pattern a une réécriture vers Bun.

Point de départ mesuré : pilier 1 ≈ 60 %, pilier 2 ≈ **15 %** (8/90 packages réécrits,
13/72 APIs — cf. [`plan/coverage/`](plan/coverage/)).

## Le cœur du refactor

Sortir les règles du code Rust → **registre de données embarqué**
(`crates/n2b-registry/registry/*.toml`), source unique de vérité, auditable par diff
contre `docs/` et `upstream/`. La couverture devient une fonction de la source de
Bun/Node, re-synchronisable via `cargo xtask sync-coverage`.

## 8 problèmes structurels à corriger d'abord

PS1 détection non import-aware · PS2 duplication de la logique d'édition · PS3 deux
sources pour les mêmes deps · PS4 bug commentaires `cli_commands.rs` · PS5 constantes
magiques · **PS6 codegen schéma cassé** · **PS7 CLAUDE.md désynchronisé** · **PS8 cruft
repo**. (PS6-PS8 découverts à l'audit, absents de la v1 de ce plan.) Détail :
[`plan/01-problemes-structurels.md`](plan/01-problemes-structurels.md).

## 8 phases

| Phase | Titre | Corrige / livre |
|---|---|---|
| 0 | Socle propre | PS2, PS4, PS5, PS6, PS7, PS8 |
| 1 | Registre data-driven | PS3 — refactor pur, sortie octet-identique |
| 2 | Scanner source AST-first | PS1 — 0 faux positif homonyme |
| 3 | Modèle compat → sévérité | champ `Finding.compat` **optionnel** (pas de breaking v3) |
| 4 | Expansion couverture | `xtask sync-coverage`, 0 trou de couverture |
| 5 | Cross-compilation complète | toute entrée a une `rewrite` + migration report card |
| 6 | Intégration `bunpp` | les 🔴 pointent vers `@bun++/node-*` |
| 7 | Garde-fous & doc | CI anti-drift, contract tests étendus |

```
Phase 0 ──┬─→ Phase 1 ──┬─→ Phase 2 ───────────────┐
          │             ├─→ Phase 3 ──→ Phase 6    ├─→ Phase 7
          │             └─→ Phase 4 ──→ Phase 5 ───┘
```

## Contrat externe gelé

Aucun Rule ID renommé, aucun flag retiré, `schema_version` reste `2`, ABI cdylib v1
intacte. Trois changements de sortie **assumés et documentés** : fix PS4 (`--fix` ignore
les commentaires), Phase 2 (moins de faux positifs), Phase 3 (champ `compat` + 🔴 →
`error`). Détail : [`plan/contrat-et-risques.md`](plan/contrat-et-risques.md).

## Critère de « parfait »

1. `xtask sync-coverage --check` → 0 trou. 2. Toute entrée 🟢/🟡 a une `rewrite`
mécanique. 3. Report card d'un repo réel = résidu manuel explicite. 4. 0 faux positif
homonyme (proptest).
