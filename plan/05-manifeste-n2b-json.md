# 05 — Le manifeste `n2b.json`

> Manque identifié : n2b n'a **aucune configuration par-repo**. Tout passe par flags CLI,
> à chaque invocation. Un repo ne peut pas dire « ici, ignore `vendor/`, désactive
> `api/process-env`, mon mode par défaut est `aggressive` ». `turbo.json`,
> `tsconfig.json`, `biome.json` ont tous un manifeste racine — n2b doit en avoir un.

## 1. Principe

`n2b.json` est un **manifeste de configuration** : racine du repo, versionné, déclaratif,
écrit par l'humain, lu par n2b au début de chaque scan. Comme `turbo.json` configure
comment Turborepo opère sur un monorepo, `n2b.json` configure comment n2b opère sur un
repo à migrer.

**Séparation stricte config / état**, dans l'esprit `turbo.json` ⟂ `.turbo/` :

| Fichier | Rôle | Versionné | Écrit par |
|---|---|---|---|
| `n2b.json` | **manifeste de config** — déclaratif | ✅ oui | l'humain |
| `.n2b/` | **état & artefacts** — `state.json`, rapports, cache embeddings | ❌ gitignoré | n2b |

Le manifeste reste propre et lisible. L'état de migration (volatil, machine-écrit) ne le
pollue pas.

## 2. Emplacement & résolution

- n2b cherche `n2b.json` en partant du `root` du scan et **remonte l'arbre** jusqu'à la
  racine git (ou `/`). Premier trouvé gagne — comme `tsconfig.json` / `turbo.json`.
- Si aucun `n2b.json` : n2b fonctionne avec ses défauts (comportement actuel inchangé —
  le manifeste est **opt-in**, jamais requis).
- **Précédence** : `flags CLI explicites` > `n2b.json` > `défauts du binaire`. Un
  `--aggressive` explicite l'emporte toujours sur un `"mode": "fix"` du manifeste.
- Le scanner walk **ignore `n2b.json` lui-même** (c'est la config de n2b, pas un fichier
  à migrer) et le dossier `.n2b/`.

## 3. Structure complète (annotée)

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/.../schema/n2b.schema.json",
  "version": 1,

  // ---- Comportement du scan ----
  "mode": "check",                       // check | fix | aggressive | migrate
  "include": ["src/**", "scripts/**"],   // si présent, restreint le scan à ces globs
  "ignore": ["dist/**", "vendor/**"],    // exclus du scan (EN PLUS de .gitignore)

  // ---- Overrides de règles ----
  "rules": {
    "api/process-env":     "off",        // off | info | warn | error
    "imports/node-cluster": "error",     // force la sévérité
    "cli/*":               "warn"        // glob de catégorie
  },

  // ---- Extensions du registre propres au repo ----
  "registry": {
    "packages": [
      { "package": "@acme/legacy-http", "strategy": "rewrite",
        "target": "fetch", "aggressive": true,
        "note": "client HTTP interne déprécié" }
    ],
    "apis": [
      { "id": "api/acme-db-query", "node": "acmeDb.query",
        "import_from": "@acme/db", "bun": "Bun.sql",
        "rewrite": "manual", "codemod_hint": "voir wiki interne /migration/db" }
    ]
  },

  // ---- Monorepo ----
  "targets": ["packages/*", "apps/*"],   // workspaces à scanner

  // ---- Préférences de migration ----
  "bun": {
    "min_version": "1.3.0",
    "scaffold_polyfills": false          // --migrate appelle-t-il bunpp scaffold ?
  },

  // ---- Sortie ----
  "report": {
    "format": "text",                    // format par défaut si pas de --report
    "output": ".n2b/report.json"         // chemin du rapport persisté (optionnel)
  }
}
```

## 4. Champ par champ

| Champ | Type | Défaut | Rôle |
|---|---|---|---|
| `$schema` | string/uri | — | pointeur JSON Schema (autocomplétion IDE) |
| `version` | integer | requis | version du format du manifeste — bumpé si breaking |
| `mode` | enum | `check` | mode par défaut : `check` \| `fix` \| `aggressive` \| `migrate` |
| `include` | string[] | tout | si présent, restreint le scan à ces globs |
| `ignore` | string[] | `[]` | globs exclus, en plus de `.gitignore` |
| `rules` | object | `{}` | override par Rule ID exact ou glob de catégorie → `off`/`info`/`warn`/`error` |
| `registry.packages` | array | `[]` | entrées `packages.toml` ajoutées au registre embarqué (cf. [03 §4](03-registre-spec.md)) |
| `registry.apis` | array | `[]` | entrées `apis.toml` ajoutées au registre embarqué (cf. [03 §3](03-registre-spec.md)) |
| `targets` | string[] | `["."]` | workspaces à scanner (monorepo) |
| `bun.min_version` | string | `1.3.0` | version Bun cible — influence quelles règles s'appliquent |
| `bun.scaffold_polyfills` | bool | `false` | `--migrate` appelle-t-il `bunpp scaffold` sur les 🔴 (cf. [phase-6](phases/phase-6-bunpp.md)) |
| `report.format` | enum | `text` | format par défaut si `--report` absent |
| `report.output` | string | — | chemin où persister le rapport |

## 5. Le champ `registry` — extensions par-repo

C'est le champ qui fait le lien avec toute l'architecture du plan
([02](02-architecture-cible.md), [03](03-registre-spec.md)). Le registre embarqué de n2b
couvre l'écosystème public. Mais **chaque repo a ses deps internes** (`@acme/legacy-*`)
que n2b ne peut pas connaître.

`registry.packages` / `registry.apis` laissent un repo **étendre le registre n2b** avec
ses propres règles, dans le **même format `.toml`** (ici en JSON). n2b les **merge** avec
le registre embarqué au chargement : une dep interne devient une règle first-class —
détectée, sévérité dérivée, réécrite si `strategy`/`rewrite` exploitable.

Contrainte : un override ne peut **pas** réutiliser un `id` du registre embarqué (erreur
de validation) — il *ajoute*, il ne *remplace* pas. Pour neutraliser une règle embarquée,
c'est `rules: { "id": "off" }`.

## 6. L'état — `.n2b/`

Dossier gitignoré, écrit par n2b, jamais par l'humain :

```
.n2b/
  state.json        # état de migration persisté (la report card de Phase 5, figée)
  report.json       # dernier rapport de scan (si report.output configuré)
  cache/            # embeddings ML de n2b-ai (analyze/crosslink)
```

`.n2b/state.json` — la **migration report card persistée** (cf. [phase-5](phases/phase-5-cross-compilation.md) §5.4) :

```json
{
  "version": 1,
  "status": "in-progress",
  "last_run": "2026-05-14T22:00:00Z",
  "n2b_version": "0.5.0",
  "auto_migratable_pct": 0.87,
  "total_findings": 142,
  "auto_migrated": 124,
  "manual_residue": [
    { "rule_id": "imports/node-repl", "file": "src/cli.js", "line": 12,
      "reason": "module 🔴 — pas d'équivalent", "suggestion": "@bun++/node-repl" }
  ],
  "migrated_files": ["src/index.js", "src/server.js"]
}
```

Au premier `--migrate`, n2b crée `.n2b/` et ajoute `.n2b/` au `.gitignore` du repo cible
(ou le suggère si pas de `.gitignore`).

## 7. Schéma & type Rust

- **`schema/n2b.schema.json`** — JSON Schema draft-07, comme `schema/v2.json`. n2b valide
  `n2b.json` à son chargement ; un manifeste invalide → **erreur de config claire**
  (pas un finding), exit `2`.
- **`crates/n2b-types/src/manifest.rs`** — struct `N2bManifest` générée par le codegen
  réparé en Phase 0 (PS6). Même chaîne `cargo-typify` que `schema.rs`.

## 8. Cycle de vie

```
n2b <repo>
  1. resolve_manifest(root)            → cherche n2b.json en remontant l'arbre
  2. valider contre n2b.schema.json    → exit 2 si invalide
  3. merge : flags CLI > n2b.json > défauts
  4. merge registry embarqué + registry.{packages,apis} du manifeste
  5. scan (engine walk applique include/ignore/targets/rules overrides)
  6. [si --migrate] écrire .n2b/state.json + report card
```

`n2b init` *(évolution possible, hors v1 strict)* : génère un `n2b.json` de démarrage
commenté. Non bloquant — mentionné pour mémoire.

## 9. Impact sur le contrat externe

`n2b.json` devient une **surface publique** : une fois la v1 publiée, son schéma est
semi-gelé.

- Évolutions **additives** (nouveau champ optionnel) : sans bump de `version`.
- Évolutions **breaking** (champ retiré/resignifié) : bump `version` 1→2, n2b lit les
  deux le temps d'une dépréciation.
- `rpb-dashboard` peut écrire un `n2b.json` pour piloter n2b de façon déclarative au lieu
  d'assembler des flags — c'est un *gain* de stabilité de contrat, pas un risque.

## 10. Hors périmètre v1 (anti-scope-creep)

Volontairement **exclus** de la première version, pour garder le manifeste simple :

- `extends` / manifestes par-workspace (un `n2b.json` par package qui hérite du racine) —
  `targets` suffit pour le v1.
- Hooks / scripts custom (`preMigrate`, `postMigrate`).
- Profils nommés (`n2b --profile=ci`).
- Configuration du moteur ML `n2b-ai`.

Ces extensions sont toutes **additives** — elles pourront arriver sans breaking si le
besoin se confirme.

## 11. Implémentation — où ça vit dans les phases

| Volet | Phase | Détail |
|---|---|---|
| `schema/n2b.schema.json` + `N2bManifest` (type Rust) | **4** (§4.7) | dépend du codegen réparé en Phase 0 |
| Résolution + lecture du manifeste (`mode`, `include`, `ignore`, `rules`, `targets`) | **4** (§4.7) | `n2b-core` charge le manifeste avant l'engine walk |
| Merge `registry.{packages,apis}` dans le registre | **4** (§4.7) | dépend du registre — Phase 1 |
| Écriture `.n2b/state.json` après `--migrate` | **5** (§5.6) | la report card persistée |
| `bun.scaffold_polyfills` câblé à `bunpp` | **6** | lit le champ pour décider du scaffold auto |
