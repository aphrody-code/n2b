# node2bun

Analyse un package Node.js et corrige les incompatibilités avec le runtime Bun.

- Réécrit les commandes `npm` / `npx` / `pnpm` / `yarn` vers `bun` / `bunx` (scripts `package.json`, workflows GitHub Actions, shells, Dockerfiles, Makefiles).
- Préfixe les builtins Node (`fs`, `path`, `crypto`, …) par `node:` quand l'import est nu.
- Signale les dépendances redondantes avec les APIs natives Bun (`dotenv`, `node-fetch`, `uuid`, `better-sqlite3`, `rimraf`, …).
- Signale (et en mode `--aggressive`, réécrit) les idiomes Node qui ont un équivalent plus rapide côté Bun (`fs.readFileSync` → `Bun.file().text()`, `fileURLToPath(import.meta.url)` → `import.meta.dir`, shebang `node` → `bun`, `actions/setup-node` → `oven-sh/setup-bun@v2`).
- Détecte les lockfiles concurrents (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`).

## Optimisations Bun natives

- **Analyse AST** des imports via `Bun.Transpiler.scanImports()` (rapide, ignore strings et commentaires, voit les `require`, `dynamic-import`, `import-statement`).
- **Scan de répertoire** via `Bun.Glob.scan()` (streaming, natif en Zig).
- **I/O parallélisée** par batches de 64 fichiers (`Promise.all` + `Bun.file` / `Bun.write`).
- **Binaire standalone** via `bun build --compile --minify --bytecode --target=bun` :
  ```bash
  bun run build:compile     # → dist/node2bun (≈ 97 MB, démarre en ~15 ms)
  bun run install:local     # copie vers /usr/local/bin/node2bun
  ```

## Usage

```bash
# Audit en dry-run (aucune écriture, exit 1 s'il y a des findings)
bun run bin/node2bun.ts ./mon-app

# Applique les corrections sûres (CLI, node: prefix, shebang, workflow CI)
bun run bin/node2bun.ts ./mon-app --fix

# Applique aussi les migrations d'API Node → Bun (à relire à la main)
bun run bin/node2bun.ts ./mon-app --aggressive

# Rapport JSON ou Markdown
bun run bin/node2bun.ts ./mon-app --report=json
bun run bin/node2bun.ts ./mon-app --report=md > report.md

# Exclusions supplémentaires
bun run bin/node2bun.ts . --ignore="**/legacy/**"
```

## Règles

| Catégorie | IDs | Applicable en `--fix` | Applicable en `--aggressive` |
|---|---|:---:|:---:|
| CLI (`npm`/`pnpm`/`yarn`/`npx` → `bun`/`bunx`) | `cli/*` | ✓ | ✓ |
| Préfixe `node:` sur imports builtins | `imports/node-prefix` | ✓ | ✓ |
| Shebang `node` → `bun` | `shebang/node` | ✓ | ✓ |
| GitHub Actions (`setup-node` → `setup-bun`) | `ci/*` | ✓ | ✓ |
| `package.json` : scripts, `packageManager`, `engines`, deps redondantes | `pkg/*` | partiel | partiel |
| Lockfiles concurrents | `lock/rival` | report only | report only |
| Remplacements de packages (`node-fetch`, `better-sqlite3`, `uuid`…) | `imports/bun-native` | report only | ✓ (spécifiers `bun:` / `node:` uniquement) |
| APIs Node → Bun (`fs.readFileSync`, `fileURLToPath`, `__dirname` ESM…) | `api/*` | report only | ✓ |

## Codes de sortie

- `0` : aucun finding, ou mode fix/aggressive appliqué avec succès
- `1` : findings en mode `check` (dry-run)
- `2` : erreur (flag invalide, crash interne)

## Exemple

`test/fixture/` contient un projet avec tous les motifs pris en charge (`npm/npx/pnpm/yarn` dans scripts et shell, workflows CI, imports sans préfixe, APIs Node).

## Documentation de référence

Les règles sont basées sur `/home/ubuntu/rsbun/docs/bun-docs/` :
- `runtime/nodejs-compat.md` — matrice de compatibilité
- `runtime/bun-apis.md` — catalogue des APIs natives
- `pm/` — équivalents `bun install`, `bunx`, `bun add`, …
- `guides/util/import-meta-dir.md` — `import.meta.dir` et famille
