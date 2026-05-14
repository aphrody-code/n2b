# Phase 4 — Expansion de la couverture (pilier 1)

> Crée `cargo xtask sync-coverage`, peuple le registre depuis l'upstream, ajoute les
> modules/scanners/globals manquants. Objectif : **zéro module Node sans entrée registre.**
>
> **Dépend de :** Phase 1. **Bloque :** Phase 5. **Parallélisable avec :** Phases 2, 3.

## Objectif

Faire passer le pilier 1 de ~60 % à ~100 %. La couverture devient une **fonction de la
source upstream**, re-synchronisable et vérifiable en CI.

## Travaux

### 4.1 — Le crate `xtask` + `sync-coverage`

**Fichiers.** Nouveau crate `xtask/` (cf. [02-architecture-cible.md](../02-architecture-cible.md) §5,
[03-registre-spec.md](../03-registre-spec.md) §8).

`cargo xtask sync-coverage` :
1. parse `docs/bun/runtime/nodejs-compat.mdx` → `(module, statut, missing_apis[])` ;
2. scanne `upstream/bun/src/js/node/*.ts` → `module → bun_reimpl: bool` ;
3. liste `docs/node/*.md` → ensemble de référence des modules Node ;
4. régénère `crates/n2b-registry/registry/modules.toml` (merge sur `id`, champs manuels
   préservés) ;
5. rapport de drift en stderr : modules Node sans entrée registre.

`--check` : exit 1 si drift. Gère le décalage v23/v24 et le croisement mdx↔src (cf.
[03 §8](../03-registre-spec.md)).

`Cargo.toml` workspace : ajouter `xtask` aux `members`.

### 4.2 — Nouveaux modules reconnus

**Fichier.** `registry/modules.toml` (via 4.1 + saisie des champs manuels).

| Module | Statut | Note |
|---|---|---|
| `node:sqlite` | 🔴 → `bun:sqlite` | RC côté Node v24 — priorité montante |
| `node:quic` | 🔴 (non classé dans le mdx) | nouveau Node v24, angle mort du mdx |
| `node:sea` (single-executable) | spécial | → `bun build --compile` |
| internes `_http_*`, `_stream_*` | suivent leur module public | présents dans `upstream/bun/src/js/node/` |
| `node:test` | 🟡 → `bun:test` | manque mocks/snapshots/timers |

### 4.3 — Nouveaux scanners

**Fichiers.** Nouveaux dans `crates/n2b-scanners/src/` + dispatch dans
`crates/n2b-core/src/run.rs:133-236`.

| Scanner | Fichiers | Détecte |
|---|---|---|
| `env.rs` | `.env`, `.env.*` | `NODE_ENV`, `NODE_OPTIONS`, vars Node-spécifiques |
| `yarnrc.rs` | `.yarnrc`, `.yarnrc.yml` | config yarn → bun *(partiellement couvert par `npmrc.rs` — vérifier, étendre)* |
| `docker_compose.rs` | `docker-compose.yml`, `compose.yaml` | images `node:*`, commandes `npm`/`yarn` |
| `procfile.rs` | `Procfile` | commandes `node`/`npm` |
| `js_config.rs` | `jest.config.*`, `vitest.config.*`, `webpack.config.*`, `babel.config.*`, `.mocharc.*` | configs d'outils remplaçables |
| `embedded_js.rs` | `.vue`, `.svelte`, `.astro` | extraire les blocs `<script>` et les passer au pipeline source |

### 4.4 — `shell.rs` : vrai scanner

**Fichier.** `crates/n2b-scanners/src/shell.rs` (stub 6 l. aujourd'hui).

Détecter, au-delà des commandes `cli.toml` :
- `node script.js` → `bun script.js` ;
- `nvm use` / `nvm install` → signaler (Bun n'a pas de version manager intégré) ;
- `NODE_OPTIONS=...`, `NODE_ENV=...` en préfixe de commande ;
- shebangs `#!/usr/bin/env node` (déjà couvert par `shebang.rs` côté source — étendre
  aux `.sh`).

### 4.5 — Globals Node comme surface à part entière

**Fichier.** `registry/globals.toml` (étendre les 2 entrées de Phase 1).

`__dirname`, `__filename`, `require` (statique vs dynamique), `process.argv`,
`process.cwd()`, `process.platform`, `process.env`, `global`, `module.exports`,
`exports`. Chacun avec son `bun` équivalent et son `rewrite`. Traités par l'AST
(Phase 2) — pas juste `api/dirname-esm`.

### 4.6 — `cargo_toml.rs` — exposer ses Rule IDs

**Fichier.** `crates/n2b-scanners/src/cargo_toml.rs` (734 l.).

L'audit a montré que ses Rule IDs sont construits dynamiquement (`format!`) et donc
invisibles au registre. Les expliciter dans `registry/` ou au moins les rendre
auditables — sinon `xtask sync-coverage` ne peut pas les compter.

## Critères d'acceptation

- **`cargo xtask sync-coverage --check` rapporte 0 module Node sans entrée registre.**
- `cargo test --workspace` vert (+ tests des nouveaux scanners).
- `cargo clippy --workspace --all-targets -- -D warnings`.
- Baselines régénérées (nouveaux scanners = nouveaux findings sur `test/fixture` si la
  fixture contient `.env`/`docker-compose.yml`/etc. — sinon inchangées). Ajouter des cas
  à `test/fixture/` pour couvrir les nouveaux scanners.
- `n2b rules` liste les nouvelles règles.

## Commits attendus

```
feat(xtask): cargo xtask sync-coverage — codegen modules.toml depuis upstream/ + drift report
feat(n2b-registry): modules.toml — node:sqlite, node:quic, node:sea, internes _http_*
feat(n2b-scanners): scanners .env, docker-compose, Procfile, configs jest/vitest/webpack
feat(n2b-scanners): embedded_js — extrait les <script> de .vue/.svelte/.astro
feat(n2b-scanners): shell.rs — vrai scanner (node script.js, nvm, NODE_OPTIONS)
feat(n2b-registry): globals.toml — __dirname/require/process.* comme surface complète
chore(baselines): régénère après expansion de couverture
```

## Risques

| Risque | Mitigation |
|---|---|
| Le parseur de `nodejs-compat.mdx` est fragile (format markdown libre) | parser tolérant + test sur le mdx commité ; si le format change au bump canary, le test casse explicitement |
| `embedded_js` (Vue/Svelte/Astro) — extraction `<script>` imparfaite | viser le cas simple (`<script>` standard) ; documenter les limites ; ne pas bloquer la phase sur les SFC exotiques |
| Explosion du nombre de findings sur les repos témoins | attendu — c'est l'objectif ; trier les baselines, vérifier qu'aucun n'est un faux positif |
| `xtask` lit `upstream/` qui est gitignoré → CI sans `upstream/` | `--check` en CI doit soit cloner `upstream/` en amont, soit comparer contre un snapshot commité de la matrice. Décision : commiter un `registry/.upstream-snapshot.toml` que `--check` utilise hors-ligne |
