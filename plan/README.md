# plan/ — refactorisation n2b : couverture totale + cross-compilation Node→Bun

> Dossier de planification détaillé. Rédigé 2026-05-14, ancré sur un audit croisé de
> quatre surfaces : code actuel de n2b (10 crates, 21 586 lignes Rust), `docs/bun/`
> (canary `fd0b6f1`), `docs/node/` (LTS v24.15.1) et `upstream/bun/src/js/node/`.
>
> Ce dossier **remplace et approfondit** `REFACTOR_PLAN.md` (conservé comme résumé
> exécutif pointant ici).

## Pourquoi ce refactor

n2b doit devenir « parfait » au sens de deux piliers indissociables :

1. **Couverture totale (entrée)** — reconnaître *toute* surface Node.js qui peut
   apparaître dans du code à migrer.
2. **Cross-compilation complète (sortie)** — chaque pattern détecté a une stratégie de
   réécriture vers Bun, pilotée par la matrice de compat.

Mesure de départ : pilier 1 ≈ 60 %, pilier 2 ≈ **15 %** (et non 25 % — l'audit a montré
que seules ~8 des ~90 entrées `BUN_REPLACEMENTS` sont effectivement réécrites, cf.
[coverage/packages.md](coverage/packages.md)).

## Comment lire ce dossier

| Étape | Fichier | Rôle |
|---|---|---|
| **Comprendre l'existant** | [00-etat-des-lieux.md](00-etat-des-lieux.md) | Audit mesuré : crates, DAG, chiffres réels, codegen cassé |
| | [01-problemes-structurels.md](01-problemes-structurels.md) | PS1→PS8 — preuves `fichier:ligne`, remèdes |
| **Comprendre la cible** | [02-architecture-cible.md](02-architecture-cible.md) | Registre data-driven, `n2b-registry`, `xtask` |
| | [03-registre-spec.md](03-registre-spec.md) | Spec exhaustive des `.toml` + codegen `sync-coverage` |
| | [04-compat-et-schema-v3.md](04-compat-et-schema-v3.md) | Modèle 🟢🟡🔴 → sévérité, champ `compat`, schéma |
| | [05-manifeste-n2b-json.md](05-manifeste-n2b-json.md) | Le manifeste `n2b.json` — config par-repo, façon `turbo.json` |
| **Exécuter** | [phases/phase-0-socle.md](phases/phase-0-socle.md) … [phase-7](phases/phase-7-garde-fous.md) | 8 phases livrables, fichiers/changements/acceptation |
| **Données de couverture** | [coverage/modules.md](coverage/modules.md) | 47 modules Node v24 — matrice complète |
| | [coverage/apis.md](coverage/apis.md) | 72 règles `api/*` actuelles + cibles |
| | [coverage/packages.md](coverage/packages.md) | ~90 deps npm — stratégie de réécriture |
| **Cadrage** | [contrat-et-risques.md](contrat-et-risques.md) | Contrat gelé, registre de risques, séquençage |

## Status board

État au 2026-05-14. À tenir à jour au fil de l'exécution.

| Phase | Titre | Dépend de | État | Commit |
|---|---|---|---|---|
| 0 | Socle propre (PS1→PS8) | — | ☐ à faire | — |
| 1 | Registre data-driven | 0 | ☐ à faire | — |
| 2 | Scanner source AST-first | 1 | ☐ à faire | — |
| 3 | Modèle compat → sévérité (schéma) | 1 | ☐ à faire | — |
| 4 | Expansion couverture + `xtask` | 1 | ☐ à faire | — |
| 5 | Cross-compilation complète | 4 | ☐ à faire | — |
| 6 | Intégration `bunpp` (les 🔴) | 3 | ☐ à faire | — |
| 7 | Garde-fous & doc | 2,3,5,6 | ☐ à faire | — |

```
Phase 0 ──┬─→ Phase 1 ──┬─→ Phase 2 ───────────────┐
          │             ├─→ Phase 3 ──→ Phase 6    ├─→ Phase 7
          │             └─→ Phase 4 ──→ Phase 5 ───┘
```

## Règle d'or de l'exécution

Chaque phase est **livrable indépendamment**, finit verte
(`cargo test --workspace` + `bash tests/compare-baseline.sh` + `cargo clippy -D warnings`),
et est **commitée séparément**. Aucune phase ne casse le contrat externe gelé
silencieusement (cf. [contrat-et-risques.md](contrat-et-risques.md)). Le seul breaking
assumé est le passage du schéma JSON en Phase 3 — et l'audit montre qu'on peut même
l'éviter (champ `compat` optionnel, cf. [04-compat-et-schema-v3.md](04-compat-et-schema-v3.md)).

## Critère de « parfait »

n2b est parfait quand les **quatre** conditions sont vraies :

1. `cargo xtask sync-coverage --check` passe → **zéro** module/API Node sans entrée registre.
2. Toute entrée registre 🟢/🟡 a une `rewrite` non-`manual` → `--migrate` est mécanique.
3. La « migration report card » d'un repo Node réel affiche un résidu manuel **explicite
   et justifié** (les 🔴, les `require()` dynamiques) — jamais un trou silencieux.
4. Zéro faux positif sur identifiant homonyme — vérifié par proptest (Phase 2).

## Glossaire

| Terme | Définition |
|---|---|
| **Registre** | Les `.toml` data-driven dans `crates/n2b-registry/registry/` — source unique de vérité des règles. |
| **Manifeste** | `n2b.json` — fichier de config racine d'un repo cible (façon `turbo.json`). État volatil dans `.n2b/`. Cf. [05](05-manifeste-n2b-json.md). |
| **Compat 🟢/🟡/🔴** | Statut d'un module `node:*` : Fully / Partial / Missing, issu de `docs/bun/runtime/nodejs-compat.mdx`. |
| **Rewrite** | Stratégie de réécriture d'une entrée : `template` (mécanique), `manual` (recette dans le finding), `drop` (suppression). |
| **PS1→PS8** | Les 8 problèmes structurels à corriger avant d'ajouter de la couverture (cf. 01). |
| **Report card** | Sortie `--migrate --report=json` exposant `auto_migratable_pct` + `manual_residue`. |
| **Drift** | Écart entre le registre n2b et la vérité-terrain upstream (`docs/`, `upstream/bun/src/js/node/`). |
| **Contrat gelé** | Rule IDs, schéma JSON, exit codes, flags CLI, ABI cdylib — consommés par `rpb-dashboard`. |
