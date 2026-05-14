# coverage/packages — les ~90 deps npm de `BUN_REPLACEMENTS`

> Inventaire de `crates/n2b-rules/src/node_imports.rs:86-661`
> (`HashMap<&str, BunReplacement>`).
>
> **Constat de l'audit qui réajuste le pilier 2 à ~15 %** : seules les entrées dont le
> `replacement` commence par `bun:` *et* `aggressive = true` sont **effectivement
> réécrites** — soit **8 paquets** : `sqlite3`, `better-sqlite3`, `jest`, `mocha`,
> `vitest`, `@jest/globals`, `ts-jest`, `jest-circus`. Les ~82 autres sont du **warning
> pur** (`imports/bun-native` informatif, jamais appliqué).
>
> Cible Phase 1 : `registry/packages.toml`, source unique (résout PS3). Cible Phase 5 :
> chaque entrée a une `strategy` exploitable (`drop`/`rewrite`/`shim`).

## Catégorie : SQLite — `strategy = rewrite` (effectif aujourd'hui)

| Package | Cible | aggressive | Effectif | `strategy` cible |
|---|---|---|---|---|
| `sqlite3` | `bun:sqlite` | ✅ | ✅ | rewrite |
| `better-sqlite3` | `bun:sqlite` | ✅ | ✅ | rewrite |

## Catégorie : Test runners — `strategy = rewrite` (partiellement effectif)

| Package | Cible | aggressive | Effectif | `strategy` cible |
|---|---|---|---|---|
| `jest` | `bun:test` | ✅ | ✅ | rewrite |
| `mocha` | `bun:test` | ✅ | ✅ | rewrite |
| `vitest` | `bun:test` | ✅ | ✅ | rewrite |
| `@jest/globals` | `bun:test` | ✅ | ✅ | rewrite |
| `ts-jest` | `bun:test` | ✅ | ✅ | rewrite |
| `jest-circus` | `bun:test` | ✅ | ✅ | rewrite |
| `chai` | `bun:test` | ❌ | ❌ | rewrite (expect natif) |

## Catégorie : HTTP clients — `strategy = rewrite` → `fetch`

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `node-fetch` | `<global fetch>` | ✅ | rewrite (drop import, `fetch` global) |
| `isomorphic-fetch` | `<global fetch>` | ✅ | rewrite |
| `cross-fetch` | `<global fetch>` | ✅ | rewrite |
| `axios` | `<global fetch>` | ✅ | shim (API différente — cas simples seulement) |
| `got` / `node-got` | `<global fetch>` | ✅ | shim |
| `superagent` | `<global fetch>` | ✅ | shim |
| `undici` | `<global fetch>` | ✅ | rewrite |
| `make-fetch-happen` | `<global fetch>` | ✅ | shim |
| `ky` | `<global fetch>` | ❌ | shim |

## Catégorie : Bases de données / cache — `strategy = rewrite|shim`

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `pg` | `Bun.sql` | ✅ | shim (tagged templates) |
| `postgres` | `Bun.sql` | ✅ | rewrite |
| `ioredis` | `Bun.redis` | ✅ | shim |
| `redis` | `Bun.redis` | ✅ | shim |

## Catégorie : Crypto / auth — `strategy = rewrite`

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `bcrypt` | `Bun.password` | ✅ | rewrite |
| `bcryptjs` | `Bun.password` | ✅ | rewrite |
| `argon2` | `Bun.password` | ✅ | rewrite |
| `uuid` | `Bun.randomUUIDv7` | ✅ | rewrite |
| `keytar` | `Bun.secrets` | ✅ | rewrite |

## Catégorie : Web standards — `strategy = drop` (natif Bun)

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `form-data` | `FormData` | ✅ | drop (global) |
| `abort-controller` | `AbortController` | ✅ | drop (global) |
| `whatwg-url` | `URL` | ✅ | drop (global) |
| `node-blob` | `Blob` | ✅ | drop (global) |
| `web-streams-polyfill` | `<global>` | ✅ | drop (global) |
| `eventsource` | `Bun.EventSource` | ✅ | rewrite |
| `ws` | `WebSocket` / `Bun.serve` | ✅ | shim (serveur) / drop (client) |

## Catégorie : Glob — `strategy = rewrite` → `Bun.Glob`

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `glob` | `Bun.Glob` | ✅ | rewrite |
| `fast-glob` | `Bun.Glob` | ✅ | rewrite |
| `globby` | `Bun.Glob` | ✅ | rewrite |
| `tiny-glob` | `Bun.Glob` | ✅ | rewrite |
| `glob-parent` | `Bun.Glob` | ❌ | shim |

## Catégorie : Process / shell — `strategy = rewrite|shim`

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `which` | `Bun.which` | ✅ | rewrite |
| `cross-spawn` | `Bun.spawn` | ✅ | shim |
| `execa` | `Bun.$` / `Bun.spawn` | ✅ | shim |
| `shelljs` | `Bun.$` | ❌ | shim |
| `node-cron` | `Bun.cron` | ✅ | rewrite |
| `rimraf` | `fs.rm` | ❌ | rewrite |
| `mkdirp` | `fs.mkdir` | ❌ | rewrite |

## Catégorie : Parsing / formats — `strategy = rewrite` (natif Bun)

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `@iarna/toml` / `toml` / `smol-toml` | `Bun.TOML` | ✅ | rewrite |
| `js-yaml` / `yaml` | `Bun.YAML` | ✅ | rewrite |
| `marked` / `markdown-it` | `Bun.markdown` | ✅ | rewrite |
| `json5` | `Bun.JSON5` | ✅ | rewrite |
| `jsonc-parser` | `Bun.JSONC` | ✅ | rewrite |
| `cookie` | `Bun.Cookie` | ✅ | rewrite |
| `cookie-parser` | `Bun.CookieMap` | ✅ | rewrite |
| `csurf` / `csrf` | `Bun.CSRF` | ✅ | rewrite |
| `pako` | `Bun.gzipSync` | ✅ | rewrite |

## Catégorie : Texte / ANSI — `strategy = rewrite` (natif Bun)

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `he` / `escape-html` / `lodash.escape` | `Bun.escapeHTML` | ✅ | rewrite |
| `string-width` | `Bun.stringWidth` | ✅ | rewrite |
| `strip-ansi` | `Bun.stripANSI` | ✅ | rewrite |
| `ansi-regex` | `Bun.stripANSI` | ❌ | rewrite |
| `slice-ansi` | `Bun.sliceAnsi` | ✅ | rewrite |
| `chalk` / `kleur` / `colorette` / `picocolors` | `<natif>` | ❌ | drop (couleurs natives) |

## Catégorie : Comparaison — `strategy = rewrite` → `Bun.deepEquals`

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `fast-deep-equal` / `deep-equal` / `lodash.isequal` | `Bun.deepEquals` | ✅ / ✅ / ✅ | rewrite |

## Catégorie : Cloud / S3 — `strategy = rewrite|shim`

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `@aws-sdk/client-s3` | `Bun.s3` / `Bun.S3Client` | ✅ | shim |
| `aws-sdk` | `Bun.s3` / `fetch s3://` | ❌ | shim |

## Catégorie : Tooling / runtime — `strategy = drop|rewrite`

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `dotenv` / `dotenv/config` | `<auto>` | ✅ | **drop** (`.env` autoload natif) |
| `ts-node` / `tsx` | `bun run` | ✅ | drop (Bun exécute TS nativement) |
| `esm` | `<natif>` | ✅ | drop |
| `nodemon` | `bun --watch` | ✅ | rewrite (script) |
| `concurrently` | `<natif>` | ❌ | shim |
| `@types/node` | `@types/bun` | ❌ | rewrite (devDep) |
| `eslint` / `prettier` / `@typescript-eslint/*` / `eslint-config-prettier` | `@biomejs/biome` | ❌ | shim (outil différent) |

## Catégorie : Divers — `strategy` variée

| Package | Cible | aggressive | `strategy` cible |
|---|---|---|---|
| `minimist` | `util.parseArgs` | ❌ | rewrite |
| `dns-packet` | `bun dns` | ❌ | shim |
| `mime` / `mime-types` | `<natif>` | ❌ | drop |

## Synthèse — l'écart du pilier 2

| | Aujourd'hui | Cible Phase 5 |
|---|---|---|
| Entrées totales | ~90 | ~90 (+ ajouts couverture Phase 4) |
| **Effectivement réécrites** | **8** (≈ 9 %) | **toutes** ont une `strategy` exploitable |
| `drop` (natif Bun) | 0 explicite | ~15 (dotenv, web standards, chalk, mime, ts-node…) |
| `rewrite` (mécanique) | 8 | ~55 |
| `shim` (manuel/`@n2b/shims`) | 0 explicite | ~20 (axios, pg, ioredis, express, eslint…) |

Le pilier 2 n'est pas « à 25 % » comme estimé dans `REFACTOR_PLAN.md` — il est à **~9 %
sur les packages** (8/90) et **~18 % sur les APIs** (13/72, cf. [apis.md](apis.md)). La
moyenne pondérée ≈ **15 %**. C'est le vrai point de départ.
