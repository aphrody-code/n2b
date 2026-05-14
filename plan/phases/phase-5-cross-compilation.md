# Phase 5 — Cross-compilation complète (pilier 2)

> Chaque entrée registre a une `rewrite` exploitable. `--migrate` devient mécanique sur
> tout ce qui est 🟢/🟡. La « migration report card » rend le pilier 2 mesurable.
>
> **Dépend de :** Phase 4 (registre peuplé). **Bloque :** Phase 7.

## Objectif

Faire passer le pilier 2 de ~15 % à ~100 % sur les modules 🟢/🟡. L'audit a montré que
seules **~8 des ~90** entrées `packages.toml` sont effectivement réécrites — le reste est
warning pur. Cible : toute entrée a `rewrite ∈ {template, manual, drop}`, et `manual`
porte une recette.

## Travaux

### 5.1 — Compléter les `rewrite` du registre

**Fichiers.** `registry/apis.toml`, `registry/packages.toml`, `registry/globals.toml`.

Chaque entrée sans `rewrite` exploitable est traitée :

| Cas | `rewrite` | Champ |
|---|---|---|
| Substitution mécanique possible | `template` | `template = "..."` |
| Signature trop différente | `manual` | `codemod_hint = "recette précise"` |
| L'appel disparaît | `drop` | — |

**Aucune entrée ne reste en `manual` sans `codemod_hint`** — validé au build (Phase 1
§7).

### 5.2 — Réécritures à haute valeur (depuis `docs/bun/`)

Compléter les `template` / `codemod_hint` pour les migrations à fort impact :

| Source Node | Cible Bun | `rewrite` |
|---|---|---|
| `child_process.spawn/exec/execSync` | `Bun.spawn` / `Bun.$` | `manual` (API Promise, pas de callback) |
| `pg` / `postgres` | `Bun.SQL` / `Bun.sql` | `manual` (tagged templates) |
| `ioredis` / `redis` | `Bun.redis` / `Bun.RedisClient` | `template` (cas simple) + `manual` (avancé) |
| `glob` / `fast-glob` / `globby` | `Bun.Glob` | `template` |
| `ws` | `Bun.serve({ websocket })` | `manual` (modèle serveur différent) |
| `better-sqlite3` / `node:sqlite` | `bun:sqlite` | `template` import + `manual` API |
| `jest` / `vitest` / `mocha` | `bun:test` | `template` (imports) + `manual` (config) |
| `node-fetch` / `axios` (cas simples) | `fetch` global | `template` |
| `dotenv` | — | `drop` (`.env` autoload natif) |
| `crypto.createHash` | `Bun.CryptoHasher` | `template` |
| `fs.readFileSync` / `writeFileSync` | `Bun.file` / `Bun.write` | `template` (déjà fait) |

### 5.3 — CJS → ESM

**Fichier.** `registry/globals.toml` + `engine.rs`.

- `__dirname` → `import.meta.dir`, `__filename` → `import.meta.path` — `template`.
- `require()` **statique** (argument littéral) → `import` — `template`.
- `require()` **dynamique** (argument variable) → `manual` + `codemod_hint`
  (« convertir en `await import()`, vérifier le top-level await »). Repéré via
  `ImportGraph.dynamic_requires` (Phase 2).
- `module.exports` / `exports` → `export` — `manual` (la forme dépend du pattern).

### 5.4 — La « migration report card »

**Fichiers.** `crates/n2b-core/src/run.rs`, `crates/n2b-report/src/lib.rs`,
`crates/n2b-cli/src/cli/args.rs`.

`n2b --migrate --report=json` expose un objet supplémentaire :

```json
{
  "report_card": {
    "auto_migratable_pct": 0.87,
    "total_findings": 142,
    "auto_migrated": 124,
    "manual_residue": [
      { "rule_id": "imports/node-repl", "reason": "module 🔴 — pas d'équivalent",
        "file": "src/cli.js", "line": 12, "suggestion": "@bun++/node-repl" },
      { "rule_id": "globals/require-dynamic", "reason": "require() dynamique non mécanisable",
        "file": "src/loader.js", "line": 45 }
    ]
  }
}
```

`auto_migratable_pct` = `auto_migrated / total_findings`. Le résidu manuel liste
**chaque** finding non mécanisable, avec sa raison. C'est ce qui rend le pilier 2
**mesurable** — un trou de couverture devient un chiffre, pas un silence.

Flag : le `report_card` est additif (n'apparaît qu'avec `--migrate`). Pas de breaking.

### 5.5 — Câbler `--migrate` aux `rewrite`

**Fichier.** `crates/n2b-cli/src/commands/migrate.rs`.

`--migrate` applique :
- les `rewrite = "template"` → substitution mécanique ;
- les `rewrite = "drop"` → suppression de la ligne/de l'appel ;
- les `rewrite = "manual"` → **pas** d'édition, mais entrée dans `manual_residue` du
  report card avec le `codemod_hint`.

Tout side-effect passe par `BackupGuard` (déjà en place — `subprocess/bun.rs`).

## Critères d'acceptation

- **Toute entrée registre 🟢/🟡 a une `rewrite` non-`manual`** *ou* un `codemod_hint`
  justifié (validé au build).
- Sur `test/fixture/` et un **repo Node réel témoin**, `n2b --migrate` produit un projet
  qui `bun install && bun test` au vert. C'est le critère dur du pilier 2.
- `n2b --migrate --report=json` expose un `report_card` avec `auto_migratable_pct` et un
  `manual_residue` non vide mais **entièrement justifié** (que des 🔴 et des `require()`
  dynamiques — jamais un trou).
- `cargo test --workspace` vert, baselines régénérées.
- Le rollback `BackupGuard` testé : `--migrate` sur un projet qui fait échouer
  `bun install` → restore complet.

## Repo témoin

Choisir un petit projet Node réel (CLI ou serveur Express simple) — le cloner dans
`tests/fixtures-real/` (gitignoré ou submodule léger). `--migrate` dessus doit aboutir à
`bun test` vert. Ce test est manuel/CI optionnel (pas dans `cargo test` standard car
nécessite réseau pour `bun install`).

## Commits attendus

```
feat(n2b-registry): rewrite complète — apis.toml/packages.toml, plus aucune entrée sans stratégie
feat(n2b-registry): templates haute valeur — child_process, pg, ioredis, glob, ws, sqlite
feat(n2b-registry): CJS→ESM — __dirname, require statique/dynamique, module.exports
feat(n2b-core): migration report card — auto_migratable_pct + manual_residue
feat(n2b-cli): --migrate applique template/drop, route manual vers le report card
chore(baselines): régénère après complétion des rewrites
```

## Risques

| Risque | Mitigation |
|---|---|
| Un `template` produit du code qui ne compile pas sous Bun | repo témoin + `bun test` est le garde-fou ; chaque template haute valeur testé sur un cas réel |
| `--migrate` casse un projet (rewrite trop agressif) | `BackupGuard` + restore ; les rewrites risqués sont `manual` par défaut, pas `template` |
| `auto_migratable_pct` gonflé artificiellement (compte des findings cosmétiques) | le dénominateur exclut les `info` purs ; le pct mesure le résidu *bloquant*, défini via `compat.status` |
| Repo témoin instable (deps qui bougent) | épingler les versions ; ou utiliser un repo figé minimal maintenu dans `tests/fixtures-real/` |
