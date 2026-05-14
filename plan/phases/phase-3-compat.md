# Phase 3 — Modèle de compat → sévérité & champ `compat`

> `registry/modules.toml` porte le statut 🟢/🟡/🔴 ; la sévérité en est dérivée ; le
> `Finding` expose un champ `compat`. **Pas de breaking** — champ optionnel (cf.
> [04-compat-et-schema-v3.md](../04-compat-et-schema-v3.md)).
>
> **Dépend de :** Phase 0 (codegen réparé), Phase 1 (registre). **Bloque :** Phase 6.
> **Parallélisable avec :** Phases 2, 4.

## Objectif

n2b doit *parler* sur les modules 🟡/🔴 au lieu de se taire. Aujourd'hui `imports/*`
émet un niveau unique indifférencié. Cible : 🟢→`info`, 🟡→`warning` + sous-API
manquante citée, 🔴→`error` + pointeur `bunpp`.

## Travaux

### 3.1 — Peupler `modules.toml` avec le statut compat

**Fichier.** `crates/n2b-registry/registry/modules.toml`.

Renseigner `compat`, `bun_reimpl`, `missing_apis`, `equivalent` pour les 47 modules.
Source : la matrice de [coverage/modules.md](../coverage/modules.md) (déjà minée depuis
`docs/bun/runtime/nodejs-compat.mdx` + `upstream/bun/src/js/node/`).

> En Phase 4, `xtask sync-coverage` *régénérera* ces champs. Ici on les saisit à la main
> à partir de la matrice de couverture — Phase 4 automatisera la re-synchro.

### 3.2 — `derive_severity` + validation

**Fichier.** `crates/n2b-registry/src/schema.rs`.

```rust
pub fn derive_severity(compat: Compat) -> Severity {
    match compat {
        Compat::Full    => Severity::Info,
        Compat::Partial => Severity::Warn,
        Compat::Missing => Severity::Error,
    }
}
```

`registry.rs` valide : toute entrée liée à un module a
`severity == derive_severity(module.compat)`. Échec build sinon.

### 3.3 — Ajouter `compat` à `schema/v2.json`

**Fichier.** `schema/v2.json`.

Ajouter la définition `Compat` et la propriété `compat` à `Finding` — dans
`properties`, **pas** dans `required` (rétro-compat, cf.
[04-compat-et-schema-v3.md](../04-compat-et-schema-v3.md) §3). `schema_version` reste
`2`. Forme exacte du champ : [04 §3](../04-compat-et-schema-v3.md).

### 3.4 — Régénérer les types

```bash
bun run codegen:schema   # → crates/n2b-types/src/schema.rs (+ TS si applicable)
```

Le codegen doit être fonctionnel (PS6 corrigé en Phase 0). Vérifier
`bun run codegen:schema:check` après.

### 3.5 — `engine.rs` attache le `compat` au `Finding`

**Fichier.** `crates/n2b-registry/src/engine.rs`.

Quand une entrée `imports/node-*` ou `api/node-*` matche, l'engine récupère le
`ModuleEntry` du module hôte et remplit `Finding.compat` :
`{ status, module, missing_apis, equivalent }`.

### 3.6 — `n2b-report` affiche `compat`

**Fichier.** `crates/n2b-report/src/lib.rs`.

- `text` : ligne supplémentaire `compat: partial — manque proc.gid, proc.uid`.
- `markdown` : colonne ou ligne dédiée.
- `sarif` : dans `properties` du `result`.
- `json` / `jsonl` : natif (sérialisation du champ).

### 3.7 — Règles sous-API granulaires (amorce)

**Fichier.** `crates/n2b-registry/registry/apis.toml`.

Pour les modules 🟡, ajouter des entrées `api/node-<module>-<subapi>` ciblant les
sous-APIs **précisément manquantes** (ex. `api/node-vm-measureMemory`,
`api/node-cluster-handle-transfer`). Sévérité `error` même si le module est 🟡 — appeler
une sous-API cassée est bloquant. Liste complète : [coverage/modules.md](../coverage/modules.md).

> C'est une *amorce* : la couverture exhaustive des sous-APIs est un travail de Phase 4.
> Ici on pose le mécanisme + les sous-APIs les plus courantes.

### 3.8 — Régénérer toutes les baselines

Le champ `compat` apparaît dans la sortie → **toutes** les baselines changent :
`tests/snapshots/baseline/*.{json,jsonl,md,sarif,txt}` +
`tests/rpb-dashboard-baseline/scan.*`. Régénération assumée et documentée.

### 3.9 — `contract.rs` & `CHANGELOG.md`

- `contract.rs` : `json_report_validates_against_schema_v2` doit rester vert (champ dans
  `properties`, schéma à jour). Ajouter un test : tout `Finding` d'un `imports/node-*` a
  un `compat`.
- `CHANGELOG.md` : « feat: champ optionnel `Finding.compat` — rétro-compatible, pas de
  bump `schema_version`. Sévérité des findings `imports/*` désormais dérivée du statut de
  compat Bun. »

## Critères d'acceptation

- `cargo test --workspace` vert, dont contract tests.
- `bun run codegen:schema:check` — pas de drift.
- `bash tests/compare-baseline.sh` — vert **après régénération assumée** des baselines.
- Validation `jsonschema` : un `Finding` *avec* `compat` valide ; un `Finding` *sans*
  `compat` valide aussi (rétro-compat vérifiée par un test dédié).
- `n2b test/fixture --report=text` montre les lignes `compat:` sur les findings
  `imports/*`.

## Coordination `rpb-dashboard`

Champ **optionnel** → aucune coordination *bloquante*. Mais courtoisie : prévenir que le
champ `compat` est désormais disponible (le dashboard peut s'en servir pour trier). Pas
de fenêtre de gel nécessaire.

## Commits attendus

```
feat(n2b-registry): modules.toml — statut compat 🟢🟡🔴 des 47 modules Node v24
feat(n2b-registry): derive_severity — sévérité pilotée par la compat
feat(schema): champ optionnel Finding.compat (rétro-compatible, schema_version inchangé)
feat(n2b-report): affiche compat dans text/markdown/sarif
feat(n2b-registry): amorce des règles api/node-*-<subapi> granulaires
test(n2b-cli): contract — Finding.compat présent sur imports/node-*, rétro-compat schéma
chore(baselines): régénère après ajout du champ compat
```

## Risques

| Risque | Mitigation |
|---|---|
| Un consommateur valide en mode strict et casse sur champ inconnu | le champ est dans le schéma `properties` → un validateur conforme l'accepte ; documenter dans CHANGELOG |
| `xtask sync-coverage` (Phase 4) régénérera `modules.toml` et écrasera la saisie manuelle | Phase 4 *merge* sur `id` et préserve les champs manuels ; ici on saisit aussi les champs upstream, Phase 4 les confirmera |
| Sévérité d'un `imports/*` passe de `warn` à `error` → exit code change (0/1→2) | c'est le comportement *voulu* (🔴 doit bloquer) ; documenté dans CHANGELOG ; baselines régénérées le capturent |
