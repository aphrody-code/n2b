# Bun internals — architecture map

Référence pour n2b : quels composants Bun assemble, où ils vivent, ce que n2b
peut ou ne peut pas détecter.

**Sources upstream** (cloner localement à la demande via `git clone --depth 1`) :

- [oven-sh/bun](https://github.com/oven-sh/bun) — runtime principal
- [oven-sh/zig](https://github.com/oven-sh/zig) — fork Zig 0.15.x + patches Bun (cherry-picks depuis 0.14.1)
- [oven-sh/WebKit](https://github.com/oven-sh/WebKit) — fork WebKit/JSC avec patches listés ci-dessous

## Langages / composants

| Composant | Langage | Rôle |
|---|---|---|
| `src/*.zig` | Zig | Runtime principal, I/O, HTTP, process, crypto |
| `src/bun.js/bindings/*.cpp` | C++ | Bindings WebKit/JSC, modules natifs, bun.ArrayBufferSink |
| `src/js/node/*.ts` | TS | Shims des modules `node:*` (bundlés avec runtime) |
| `src/js/bun/*.ts` | TS | Modules `bun:*` (test, sqlite, ffi en partie) |
| `src/js/thirdparty/*.ts` | TS | Modules tiers pre-bundled (ws, undici shim, etc.) |
| `src/deps/` | C/C++ | uWebSockets, mimalloc, boringssl, zlib-ng, zstd |

## Points d'entrée build

- `build.zig` — orchestrateur Zig principal
- `scripts/build/deps/webkit.ts` — pin du commit WebKit dans le build system
- `./build/debug/bun-debug` — binaire debug (AddressSanitizer par défaut)
- `./build/release/bun` — binaire release

**Prérequis build :** LLVM 21.1.8 strictement (pas de wildcard), Zig via le fork custom, `bun run build` pour bootstrap.

## Patches WebKit spécifiques Bun

Ce qui différencie `oven-sh/WebKit` de `WebKit/WebKit` :

- **mémoire :** `bmalloc::api::availableMemory()` respecte cgroups (containers)
- **Error.captureStackTrace** — API V8/Node.js exposée via JSC
- **Date.now() override** — champ `overridenDateNow` pour mocking (utilisé par `bun test`)
- **onComputeErrorInfo** — formatage stack traces compatible V8
- **JSString itérateur** — buffers internes exposés sans alloc (zero-copy)
- **V8 date parser** — compat `Date(...)` format V8
- **API C sans verrous** — concurrence gérée par Bun
- **DOMJIT amélioré** — Typed Arrays plus rapides

## Patches Zig spécifiques Bun

Fork `oven-sh/zig` (branche `upgrade-0.15.2`) :

- LLVM + ZSTD statiques pour builds release reproductibles
- CI avec TSAN/ASAN pour le bootstrap
- ZLS (Zig Language Server) inclus
- Cherry-picks de fixes Zig upstream non-encore mergés
- Gestion merge via `zdiff3`

## Pertinence pour n2b

**Ce que n2b détecte côté utilisateur Bun :**
- Imports `bun:*` (bun:sqlite, bun:test, bun:ffi, bun:jsc, bun:bundle) — valides, pas flaggés
- Remplacements `npm package → Bun.* / bun:*` via `rules/node_imports.rs`
- APIs Node → APIs Bun via `rules/bun_apis.rs`

**Ce que n2b ne détecte pas (hors périmètre utilisateur) :**
- Patterns Zig ou C++ internes à Bun
- Bugs liés aux patches WebKit spécifiques
- Build system Zig (build.zig) — n2b cible l'utilisateur de Bun, pas le contributeur Bun

**APIs Bun avancées stables** (n'ont pas besoin de règle de migration car déjà Bun-native) :

| Module | Doc | Usage typique |
|---|---|---|
| `bun:ffi` | https://bun.sh/docs/api/ffi | Appeler C/Rust/Zig natif depuis JS |
| `bun:jsc` | https://bun.sh/reference/bun/jsc | Introspection JavaScriptCore (heap, deepStats) |
| `bun:sqlite` | https://bun.sh/docs/api/sqlite | SQLite embarqué |
| `bun:test` | https://bun.sh/docs/test | Test runner Jest-compatible |
| `bun:bundle` | https://bun.sh/docs/bundler | Bundler programmatique |
