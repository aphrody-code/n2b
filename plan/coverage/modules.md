# coverage/modules — matrice des 47 modules Node v24

> Minée depuis `docs/bun/runtime/nodejs-compat.mdx` (calé Node v23) × `docs/node/*.md`
> (v24.15 LTS « Krypton ») × `upstream/bun/src/js/node/` (62 fichiers = vérité-terrain).
>
> **Statut n2b** : ce que n2b reconnaît aujourd'hui (`BUILTINS` dans
> `node_imports.rs:7-70` — tous les builtins, mais sans distinction de compat).
>
> Bilan : **22 🟢 · 19 🟡 · 3 🔴 · 1 non classé** (`node:quic`).

## 🟢 Fully implemented (22) — sévérité cible `info`

| Module | `bun_reimpl` | Note compat Bun | Statut n2b | Règle cible |
|---|---|---|---|---|
| `assert` | ✅ | complet | reconnu (prefix) | `imports/node-assert` |
| `buffer` | natif Zig | complet | reconnu | `imports/node-buffer` |
| `console` | ✅ | complet | reconnu | `imports/node-console` |
| `dgram` | ✅ | complet, >90 % suite | reconnu | `imports/node-dgram` |
| `diagnostics_channel` | ✅ | complet, support C++ (v24) | reconnu | `imports/node-diagnostics_channel` |
| `dns` | ✅ | complet, >90 % suite | reconnu | `imports/node-dns` |
| `events` | ✅ | complet, 100 % suite | reconnu | `imports/node-events` |
| `fs` | ✅ | complet, 92 % suite | reconnu | `imports/node-fs` |
| `http` | ✅ | complet — **corps client bufferisé, pas streamé** | reconnu | `imports/node-http` |
| `https` | ✅ | complet — **`Agent` pas toujours utilisé** | reconnu | `imports/node-https` |
| `net` | ✅ | complet | reconnu | `imports/node-net` |
| `os` | ✅ | complet, 100 % suite | reconnu | `imports/node-os` |
| `path` | ✅ (+posix/win32) | complet, 100 % suite | reconnu | `imports/node-path` |
| `punycode` | ✅ | complet (déprécié par Node) | reconnu | `imports/node-punycode` |
| `querystring` | ✅ | complet, 100 % suite | reconnu | `imports/node-querystring` |
| `readline` | ✅ (+promises) | complet | reconnu | `imports/node-readline` |
| `stream` | ✅ (+web/consumers/promises) | complet | reconnu | `imports/node-stream` |
| `string_decoder` | natif | complet, 100 % suite | reconnu | `imports/node-string_decoder` |
| `timers` | ✅ (+promises) | complet (préférer globals) | reconnu | `imports/node-timers` |
| `tty` | ✅ | complet | reconnu | `imports/node-tty` |
| `url` | ✅ | complet | reconnu | `imports/node-url` |
| `zlib` | ✅ | complet, 98 % suite | reconnu | `imports/node-zlib` |

## 🟡 Partial (19) — sévérité cible `warning` + sous-API citée

Les sous-APIs manquantes sont la matière des règles granulaires `api/node-<module>-<subapi>`
(Phase 3 §3.7, Phase 4).

| Module | `bun_reimpl` | Sous-APIs manquantes / incomplètes | Règle module | Règles sous-API cibles |
|---|---|---|---|---|
| `async_hooks` | ✅ | `AsyncLocalStorage`/`AsyncResource` OK ; **v8 promise hooks non appelés** ; usage déconseillé | `imports/node-async_hooks` | — |
| `child_process` | ✅ | `proc.gid`, `proc.uid` ; classe `Stream` non exportée ; IPC sans socket handles ; IPC Node↔Bun = JSON only | `imports/node-child_process` | `api/node-child_process-gid-uid`, `api/node-child_process-ipc-handle` |
| `cluster` | ✅ | handles/FD non transférables entre workers ; load-balancing HTTP **Linux only** (`SO_REUSEPORT`) ; pas battle-tested | `imports/node-cluster` | `api/node-cluster-handle-transfer` |
| `crypto` | natif | `secureHeapUsed`, `setEngine`, `setFips` ; raw key formats (v24) à tracker | `imports/node-crypto` | `api/node-crypto-setEngine`, `api/node-crypto-setFips` |
| `domain` | ✅ | `Domain`, `active` | `imports/node-domain` | `api/node-domain-active` |
| `http2` | ✅ | `options.allowHTTP1`, `options.enableConnectProtocol`, ALTSVC, `http2stream.pushStream` ; v24 ajoute `http1Options` (écart qui se creuse) | `imports/node-http2` | `api/node-http2-allowHTTP1`, `api/node-http2-pushStream` |
| `inspector` | ✅ (+promises) | seul `Profiler` supporté (`enable`/`disable`/`start`/`stop`/`setSamplingInterval`) ; reste non implémenté | `imports/node-inspector` | `api/node-inspector-non-profiler` |
| `module` | ✅ | `syncBuiltinESMExports`, `Module#load()` ; `_extensions`/`_pathCache`/`_cache` = no-ops ; **`module.register` non implémenté → `Bun.plugin`** | `imports/node-module` | `api/node-module-register` |
| `perf_hooks` | ✅ | APIs présentes mais test suite Node ne passe pas | `imports/node-perf_hooks` | — |
| `process` (module + global) | natif | `process.binding` partiel ; `process.title` no-op macOS/Linux ; stubs `getActiveResources*` ; **`loadEnvFile`, `getBuiltinModule` non implémentés** | `imports/node-process` | `api/node-process-loadEnvFile`, `api/node-process-getBuiltinModule` |
| `sys` | — | alias de `node:util` (mêmes manques) ; déprécié | `imports/node-sys` | — |
| `test` | ✅ | manque **mocks, snapshots, timers** → `bun:test` ; v24 ajoute module mocks/worker ID/SIGINT (écart qui se creuse) | `imports/node-test` | `api/node-test-mock` |
| `tls` | ✅ (+`_tls_common`) | `tls.createSecurePair` | `imports/node-tls` | `api/node-tls-createSecurePair` |
| `util` | ✅ | `getCallSite(s)`, `getSystemErrorMap`, `getSystemErrorMessage`, `transferableAbortSignal`, `transferableAbortController` | `imports/node-util` | `api/node-util-getCallSites`, `api/node-util-transferableAbort` |
| `v8` | ✅ | `writeHeapSnapshot`/`getHeapSnapshot` OK ; `serialize`/`deserialize` = wire format JSC (≠ V8) ; `measureMemory` absent → `bun:jsc` | `imports/node-v8` | `api/node-v8-measureMemory`, `api/node-v8-serialize` |
| `vm` | ✅ | core + ES modules OK ; manque `vm.measureMemory` + partie `cachedData` | `imports/node-vm` | `api/node-vm-measureMemory` |
| `wasi` | ✅ | partiellement implémenté | `imports/node-wasi` | — |
| `worker_threads` | ✅ | `Worker` sans `stdin`/`stdout`/`stderr`/`trackedUnmanagedFds`/`resourceLimits` ; manque `markAsUntransferable`, `moveMessagePortToContext` | `imports/node-worker_threads` | `api/node-worker_threads-stdio`, `api/node-worker_threads-markAsUntransferable` |

## 🔴 Missing (3) — sévérité cible `error` + pointeur `bunpp`

| Module | `bun_reimpl` | Alternative | Règle cible | `bunpp` |
|---|---|---|---|---|
| `repl` | ✅ **stub présent** | aucune API module (`bun repl` côté CLI seulement) | `imports/node-repl` | `@bun++/node-repl` |
| `sqlite` | ❌ | **`bun:sqlite`** (API différente) — RC côté Node v24, priorité montante | `imports/node-sqlite` | `@bun++/node-sqlite` |
| `trace_events` | ✅ **stub présent** | aucune directe (`diagnostics_channel` partiel) | `imports/node-trace_events` | `@bun++/node-trace-events` |

> `repl.ts` et `trace_events.ts` existent dans `upstream/bun/src/js/node/` (stubs en
> cours). `xtask sync-coverage` marque `bun_reimpl = true` + note « statut susceptible
> d'évoluer » (cf. [phase-6](../phases/phase-6-bunpp.md) §6.4).

## ❓ Non classé (1) — angle mort du mdx

| Module | Présent dans | Absent de | Traitement |
|---|---|---|---|
| `quic` | `docs/node/quic.md` (nouveau Node v24) | `docs/bun/runtime/nodejs-compat.mdx` (calé v23) | `xtask sync-coverage` génère l'entrée avec `compat = "missing"` + commentaire `# absent de nodejs-compat.mdx (décalage v23/v24)` → drift report le signale « à vérifier » |

## Modules spéciaux (hors `node:*` classiques)

| Surface | Source | Traitement n2b |
|---|---|---|
| `node:sea` (single-executable applications) | `docs/node/single-executable-applications.md` | → `bun build --compile` ; règle dédiée Phase 4 |
| internes `_http_*`, `_stream_*`, `_tls_common` | `upstream/bun/src/js/node/` (14 fichiers) | suivent le statut de leur module public ; reconnus mais sévérité `info` (usage interne) |
| globals (`Buffer`, `process`, `__dirname`, `require`…) | `docs/node/globals.md` | → `registry/globals.toml` (cf. [coverage/apis.md](apis.md) + [03-registre-spec.md](../03-registre-spec.md) §6) |

## Décalage de version — point d'attention permanent

`nodejs-compat.mdx` suit Node **v23**. `docs/node/` est **v24**. Tant que Bun n'a pas
mis à jour son mdx, n2b doit traiter tout module/sous-API présent dans `docs/node/` mais
absent du mdx comme **potentiellement non couvert** (`missing` par défaut, marqué « à
vérifier »). `xtask sync-coverage` matérialise ce décalage dans le drift report — c'est
un signal, pas un bug.
