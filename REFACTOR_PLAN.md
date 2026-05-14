# Plan de refactorisation n2b — couverture totale + cross-compilation Node→Bun

> Rédigé : 2026-05-14. Basé sur le scan croisé de trois surfaces :
> `docs/bun/` (canary `fd0b6f1`), `docs/node/` (LTS v24.15.1) et le code actuel de n2b.
> Sources upstream complètes dans `upstream/` (gitignoré) pour le mining.

## 1. Objectif

Deux piliers indissociables :

1. **Couverture totale (entrée)** — n2b doit reconnaître *toute* surface Node.js qui peut
   apparaître dans du code à migrer : les ~50 modules `node:*`, les globals, les patterns
   CJS/ESM, les deps npm shimmables, les fichiers de config de l'écosystème.
2. **Cross-compilation complète (sortie)** — chaque pattern détecté doit avoir une
   stratégie de réécriture vers son équivalent Bun, pilotée par la matrice de compat
   Bun. `--migrate` doit produire un projet qui tourne sous Bun sans intervention, ou
   lister précisément le résidu manuel (« migration report card »).

Aujourd'hui n2b fait ~60 % du pilier 1 et ~25 % du pilier 2. Le reste est ce plan.

## 2. Constat — l'écart mesuré

### Ce que Bun couvre (de `docs/bun/runtime/nodejs-compat.mdx`)
- 🟢 **Fully** : assert, buffer, console, dgram, diagnostics_channel, dns, events, fs,
  http, net, os, path, punycode, querystring, readline, stream, string_decoder,
  timers, tty, url, zlib.
- 🟡 **Partial** (sous-APIs manquantes) : async_hooks, child_process, cluster, crypto,
  domain, http2, https, module, perf_hooks, process, tls, util, v8, vm, wasi,
  worker_threads, inspector, node:test.
- 🔴 **Missing** : repl, node:sqlite (→ `bun:sqlite`), trace_events.

### Ce que n2b reconnaît aujourd'hui
- `imports/*` : ~42 modules builtin reconnus — mais **uniquement pour suggérer le
  préfixe `node:`**. Aucune distinction 🟢/🟡/🔴.
- `api/*` : 60+ règles, dont **~14 seulement avec réécriture effective**. Le reste est
  warning sans fix.
- `cli/*` : 41 mappings npm/pnpm/yarn — solide.
- Scanners : 19 types de fichiers, mais `.env`, `.yarnrc`, `docker-compose.yml`,
  `vue/svelte/astro`, `jest/vitest/webpack.config` non couverts.

### Les trous prioritaires
| Trou | Impact |
|---|---|
| Pas de modèle de compat (🟢/🟡/🔴) | n2b ne sait pas alerter sur `cluster`/`vm`/`repl` — il les préfixe `node:` et se tait |
| `api/*` = warning sans fix | pilier 2 incomplet — `--migrate` laisse 75 % du travail à la main |
| Regex sur identifiants nus | `marked(`, `which(`, `v4()` → faux positifs sur toute fonction homonyme |
| Listes de deps dupliquées | `BUN_REPLACEMENTS` (node_imports.rs) ⟂ règles `api/*` (bun_apis.rs) — deux sources |
| `shell.rs` = stub 6 lignes | aucune détection shell réelle (`node script.js`, `nvm use`, env vars) |
| Logique anti-overlap copiée-collée | `bun_apis.rs` ⟂ `node_imports.rs`, à l'identique |

## 3. Problèmes structurels à corriger d'abord

Avant d'ajouter de la couverture, assainir le socle (sinon on empile sur du sable) :

- **PS1 — Détection non *import-aware*.** `crates/n2b-rules/src/bun_apis.rs` matche des
  identifiants nus. n2b a déjà `oxc_parser` dans `imports_ast.rs` : la détection JS/TS
  doit corréler chaque call-site à son origine d'import. Les regex ne survivent que pour
  le non-JS (configs, shell, Dockerfile).
- **PS2 — Duplication de la logique d'édition.** Le tri index/len + filtre overlap +
  `replace_range` est dupliqué `bun_apis.rs` ⟂ `node_imports.rs`. À extraire dans
  `crates/n2b-util` (`apply_edits(source, Vec<Edit>) -> String`).
- **PS3 — Deux sources pour les mêmes deps.** `BUN_REPLACEMENTS` et les règles `api/*`
  décrivent pako, bcrypt, marked, which… deux fois. Source unique requise (→ §4).
- **PS4 — Bug `cli_commands.rs`.** `replace_all` réécrit les lignes commentées alors que
  les findings les filtrent. La réécriture doit partager le filtre de commentaires.
- **PS5 — Constantes magiques.** Fenêtre `600` octets en dur dans `looks_like_dir_context`,
  etc. → constantes nommées + documentées, ou supprimées si l'AST les rend inutiles.

## 4. Architecture cible — registre de règles *data-driven*

Le cœur du refactor : sortir les règles du code Rust et les mettre dans **un registre de
données embarqué**, source unique de vérité.

```
crates/n2b-rules/
  registry/
    modules.toml      # node:* → statut compat + équivalent Bun + sévérité
    apis.toml         # API/méthode Node → template de réécriture Bun + confiance
    packages.toml     # dep npm → natif Bun ou bun:* + stratégie (drop / rewrite / shim)
    cli.toml          # npm/pnpm/yarn/npx → bun (déjà quasi data, à formaliser)
  src/
    registry.rs       # charge les .toml via include_str! + valide au build
    engine.rs         # match registry ↔ findings (AST pour JS, regex pour le reste)
    edits.rs          # apply_edits partagé (résout PS2)
```

Chaque entrée du registre porte :

```toml
[[apis]]
id            = "api/crypto-createHash"
node          = "crypto.createHash"          # pattern source
bun           = "Bun.CryptoHasher"           # cible
compat        = "partial"                    # 🟢/🟡/🔴 du module hôte
severity      = "warning"                    # dérivé de compat + criticité
rewrite       = "template"                   # template | manual | drop
template      = "new Bun.CryptoHasher({0})"
confidence    = "high"                       # high | medium | low
docs          = "docs/bun/runtime/hashing.mdx"
```

Bénéfices : kill PS3 (source unique), couverture **auditable** (diff registre ↔
`docs/node/`), ajout de règle = éditer un `.toml` (pas du Rust), et la matrice de compat
**pilote la sévérité**.

### Exploiter `upstream/` au maximum — codegen du registre

`cargo xtask sync-coverage` : lit `upstream/bun/src/js/node/` (liste des modules réellement
réimplémentés), `docs/bun/runtime/nodejs-compat.mdx` (matrice 🟢/🟡/🔴) et `docs/node/*.md`
(surface API complète), puis :
- **régénère** `registry/modules.toml` avec le statut compat à jour ;
- **émet un rapport de drift** : modules Node sans entrée registre = trous de couverture.

La couverture de n2b devient une **fonction de la source de Bun**, re-synchronisable à
chaque bump canary. C'est l'exploitation maximale demandée.

## 5. Phases d'exécution

Chaque phase est livrable indépendamment, finit verte (`cargo test --workspace` +
`bash tests/compare-baseline.sh`), et est commitée séparément.

### Phase 0 — Socle propre *(corrige PS1→PS5)*
- Extraire `apply_edits` dans `n2b-util` ; brancher `bun_apis.rs` + `node_imports.rs` dessus.
- Corriger le bug commentaires de `cli_commands.rs`.
- Nommer/documenter ou supprimer les constantes magiques.
- **Acceptation** : zéro changement de sortie (baselines intactes), `clippy -D warnings`.

### Phase 1 — Registre data-driven *(corrige PS3, base de tout le reste)*
- Créer `registry/{modules,apis,packages,cli}.toml` ; migrer les règles existantes
  dedans **sans changer un seul Rule ID ni une seule sortie** (refactor pur).
- `registry.rs` valide au build (`include_str!` + parse + assert IDs uniques).
- **Acceptation** : baselines **octet-à-octet identiques**. C'est un refactor invisible.

### Phase 2 — Scanner source AST-first *(corrige PS1)*
- `crates/n2b-scanners/src/source.rs` : passer en pipeline oxc — résoudre imports +
  call-expressions à leur binding. Les règles `api/*` ne matchent plus que des appels
  **dont l'origine d'import est connue**.
- Regex conservées uniquement pour `.sh`, `Dockerfile`, configs.
- **Acceptation** : nouveaux tests proptest « fonction locale homonyme → 0 finding ».
  Les faux positifs connus disparaissent → baselines régénérées + justifiées.

### Phase 3 — Modèle de compat → sévérité
- `registry/modules.toml` porte le statut 🟢/🟒/🔴 (issu de Phase 4 codegen ou saisi).
- `imports/node-*` émet : 🟢 → `info`, 🟡 → `warning` + sous-API manquante citée,
  🔴 → `error` + pointeur `@bun++` (cf. Phase 6).
- **Schéma v3** : ajout du champ `compat` au `Finding`. Bump `schema/v2.json` → `v3.json`,
  régénérer `schema.rs` + `schema.ts`, régénérer toutes les baselines, mettre à jour
  `contract.rs`. C'est le seul breaking assumé du plan — documenté dans le CHANGELOG.

### Phase 4 — Expansion de la couverture (pilier 1)
- **Codegen `xtask sync-coverage`** (cf. §4) — peuple `modules.toml` depuis `upstream/`.
- Nouveaux modules reconnus : `node:sqlite`, `node:sea`, `node:quic`, internes `_http_*`.
- Nouveaux scanners : `.env`, `.yarnrc[.yml]`, `docker-compose.yml`, `Procfile`,
  `jest/vitest/webpack/babel.config.*`, `.mocharc`. Code JS embarqué `.vue/.svelte/.astro`.
- `shell.rs` : vrai scanner — `node script.js`, `nvm use`, `NODE_OPTIONS`, `NODE_ENV`.
- Globals Node : `__dirname`/`__filename`/`require`/`process.*` traités comme surface
  à part entière (pas juste `api/dirname-esm`).
- **Acceptation** : `xtask sync-coverage` rapporte 0 module Node sans entrée registre.

### Phase 5 — Cross-compilation complète (pilier 2)
- **Chaque entrée registre a une `rewrite`** : `template`, `manual` (+ recette de codemod
  dans le finding), ou `drop` (ex. `require('dotenv').config()` → suppression, `.env`
  autoload natif).
- Réécritures à haute valeur à compléter (de `docs/bun/`) : `child_process.*` → `Bun.spawn`/
  `Bun.$`, `pg`/`postgres` → `Bun.SQL`, `ioredis` → `Bun.redis`, `glob` → `Bun.Glob`,
  `ws` → `Bun.serve({websocket})`, `better-sqlite3` → `bun:sqlite`, `jest`/`vitest` →
  `bun:test`, `node-fetch`/`axios` (cas simples) → `fetch`.
- **CJS→ESM** : normaliser `__dirname`/`__filename` → `import.meta.dir`/`.dirname`,
  `require()` statique → `import`, signaler les `require()` dynamiques (non mécaniques).
- **Migration report card** : `n2b --migrate --report=json` expose
  `{ auto_migratable_pct, manual_residue: [...] }`. Le pilier 2 devient mesurable.
- **Acceptation** : sur `test/fixture/` et un repo Node réel témoin, `--migrate` produit
  un projet qui `bun install && bun test` au vert.

### Phase 6 — Intégration `bunpp` (les 🔴)
- Pour chaque module 🔴 (`repl`, `node:sqlite` en usage avancé, `trace_events`), le
  finding pointe vers le polyfill `@bun++/node-*` correspondant.
- `n2b --migrate` peut appeler `bunpp scaffold <module>` quand un 🔴 est rencontré.
- Relie le registre `modules.toml` et `bunpp_cmd.rs` (déjà conscient des gaps canary).

### Phase 7 — Garde-fous & doc
- `contract.rs` : un test par catégorie de Rule ID, validation `jsonschema` contre `v3.json`.
- CI : ajouter `cargo xtask sync-coverage --check` (échoue si drift registre ↔ upstream).
- Mettre à jour `CLAUDE.md` (architecture registre) + `CHANGELOG.md` (schéma v3, breaking).
- Régénérer `tests/snapshots/baseline/` et `tests/rpb-dashboard-baseline/`.

## 6. Impact sur le contrat externe gelé

| Surface | Impact | Mitigation |
|---|---|---|
| Rule IDs | **Inchangés** (Phase 1 = refactor pur). Nouveaux IDs seulement *ajoutés*. | aucune |
| Flags CLI | Inchangés. `--report=json` enrichi (additif). | aucune |
| Format JSON | **Breaking en Phase 3** : champ `compat` ajouté → schéma v3. | bump v3, baselines régénérées, CHANGELOG, prévenir `rpb-dashboard` |
| Exit codes | Inchangés. | aucune |
| ABI cdylib v1 | Hors scope, intact. | aucune |

Le seul breaking assumé est le passage v2→v3 du schéma JSON (Phase 3). Tout le reste est
additif ou invisible. `rpb-dashboard` doit être prévenu avant Phase 3.

## 7. Ordonnancement & dépendances

```
Phase 0 ──┬─→ Phase 1 ──┬─→ Phase 2 ───────────────┐
          │             ├─→ Phase 3 ──→ Phase 6    ├─→ Phase 7
          │             └─→ Phase 4 ──→ Phase 5 ───┘
```

- Phase 0 et 1 sont prioritaires et **non négociables** (socle).
- Phase 2 (AST) et Phase 4 (couverture) peuvent avancer en parallèle après Phase 1.
- Phase 3 (schéma v3) doit être synchronisée avec `rpb-dashboard`.
- Phase 5 dépend de Phase 4 (le registre doit être peuplé avant d'écrire les templates).

## 8. Critère de « parfait »

n2b est « parfait » au sens demandé quand :
1. `cargo xtask sync-coverage --check` passe → **zéro** module/API Node sans entrée registre.
2. Toute entrée registre 🟢/🟡 a une `rewrite` non-`manual` → `--migrate` est mécanique.
3. La « migration report card » d'un repo Node réel affiche un résidu manuel explicite
   et justifié (les 🔴 et les `require()` dynamiques), pas un trou silencieux.
4. Zéro faux positif sur identifiant homonyme (Phase 2) — vérifié par proptest.
