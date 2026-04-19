# node2bun report

- mode : `check`
- racine : `/home/ubuntu/vps/rust/n2b/test/fixture`

## `.github/workflows/ci.yml`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 8:9 | `ci/setup-node` | actions/setup-node → oven-sh/setup-bun@v2 | `uses: oven-sh/setup-bun@v2` |
| 10:11 | `ci/node-version` | remplacer 'node-version' par 'bun-version: latest' | `bun-version: latest` |
| 11:14 | `cli/npm-ci` | npm ci → bun install --frozen-lockfile | `bun install --frozen-lockfile` |
| 12:14 | `cli/npm-run` | npm run → bun run | `bun run ` |
| 13:14 | `cli/npm-test` | npm test → bun test | `bun test` |

## `package-lock.json`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `lock/rival` | lockfile concurrent 'package-lock.json' présent — exécuter 'bun install' puis supprimer ce fichier |  |

## `package.json`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `cli/npm-run` | npm run → bun run | `bun run ` |
| 1:18 | `cli/npx` | npx → bunx | `bunx ` |
| 1:1 | `cli/npm-test` | npm test → bun test | `bun test` |
| 1:1 | `cli/npm-i` | npm i → bun install | `bun install` |
| 1:16 | `cli/npm-run` | npm run → bun run | `bun run ` |
| 1:1 | `cli/npm-ci` | npm ci → bun install --frozen-lockfile | `bun install --frozen-lockfile` |
| 1:34 | `cli/yarn-run` | yarn run → bun run | `bun run ` |
| 1:1 | `cli/pnpm-add-D` | pnpm add -D → bun add --dev | `bun add --dev` |
| 1:25 | `cli/pnpm-dlx` | pnpm dlx → bunx | `bunx ` |
| 1:1 | `pkg/package-manager` | packageManager='pnpm@9.0.0' — remplacer par 'bun@<version>' ou supprimer |  |
| 1:1 | `pkg/engines-pm` | engines.{npm,pnpm,yarn} est superflu avec Bun — utiliser 'engines.bun' |  |
| 1:1 | `pkg/redundant-dep` | dépendance 'dotenv' redondante avec Bun (voir Bun.file / Bun.env / fetch global / bun:sqlite / bun test) |  |
| 1:1 | `pkg/redundant-dep` | dépendance 'node-fetch' redondante avec Bun (voir Bun.file / Bun.env / fetch global / bun:sqlite / bun test) |  |
| 1:1 | `ecosystem/express` | Express détecté — guide d'intégration Bun : https://bun.sh/guides/ecosystem/express | `https://bun.sh/guides/ecosystem/express` |
| 1:1 | `pkg/redundant-dep` | dépendance 'better-sqlite3' redondante avec Bun (voir Bun.file / Bun.env / fetch global / bun:sqlite / bun test) |  |
| 1:1 | `pkg/redundant-dep` | dépendance 'uuid' redondante avec Bun (voir Bun.file / Bun.env / fetch global / bun:sqlite / bun test) |  |
| 1:1 | `pkg/redundant-dep` | dépendance 'ts-node' redondante avec Bun (voir Bun.file / Bun.env / fetch global / bun:sqlite / bun test) |  |
| 1:1 | `pkg/redundant-dep` | dépendance 'rimraf' redondante avec Bun (voir Bun.file / Bun.env / fetch global / bun:sqlite / bun test) |  |

## `scripts/install.sh`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 4:1 | `cli/npm-install` | npm install → bun install | `bun install` |
| 5:1 | `cli/npx` | npx → bunx | `bunx ` |
| 7:1 | `cli/pnpm-dlx` | pnpm dlx → bunx | `bunx ` |
| 6:1 | `cli/yarn-add-dev` | yarn add --dev → bun add --dev | `bun add --dev` |

## `src/server.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `shebang/node` | shebang 'node' → 'bun' | `#!/usr/bin/env bun` |
| 9:31 | `imports/bun-native` | remplacer 'uuid' par Bun.randomUUIDv7 — utiliser Bun.randomUUIDv7() ou crypto.randomUUID() | `Bun.randomUUIDv7` |
| 4:19 | `imports/node-prefix` | préfixer 'path' avec 'node:' (recommandé) | `node:path` |
| 5:32 | `imports/node-prefix` | préfixer 'url' avec 'node:' (recommandé) | `node:url` |
| 6:20 | `imports/bun-native` | remplacer 'node-fetch' par <global fetch> — fetch est global dans Bun — supprimer l'import | `<global fetch>` |
| 2:17 | `imports/node-prefix` | préfixer 'fs' avec 'node:' (recommandé) | `node:fs` |
| 7:21 | `imports/node-prefix` | préfixer 'crypto' avec 'node:' (recommandé) | `node:crypto` |
| 8:23 | `imports/bun-native` | remplacer 'better-sqlite3' par bun:sqlite — préférer bun:sqlite (API similaire) | `bun:sqlite` |
| 15:16 | `api/fs-readFileSync` | remplacer fs.readFileSync(path, 'utf8') par await Bun.file(path).text() | `await Bun.file("./config.json").text()` |
| 16:1 | `api/fs-writeFileSync` | remplacer fs.writeFileSync(path, data) par await Bun.write(path, data) | `await Bun.write("./last.json", JSON.stringify({ run: Date.now()` |
| 23:12 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 13:1 | `api/dirname-esm` | dans un ESM Bun, utiliser directement import.meta.dir (ou import.meta.dirname) | `const __dirname = import.meta.dir` |
| 12:1 | `api/filename-esm` | dans un ESM Bun, utiliser import.meta.path (ou import.meta.filename) | `const __filename = import.meta.path` |
| 18:14 | `api/crypto-createHash` | Bun.hash / Bun.CryptoHasher est plus rapide (voir runtime/hashing) |  |
| 12:20 | `api/fileURLToPath` | Bun.fileURLToPath() est équivalent (ou utiliser import.meta.dir/path) |  |
| 13:32 | `api/fileURLToPath` | Bun.fileURLToPath() est équivalent (ou utiliser import.meta.dir/path) |  |
| 19:12 | `api/uuid-v4` | crypto.randomUUID() (global) ou Bun.randomUUIDv7() évite la dépendance uuid |  |
| 21:1 | `api/express-app` | Bun.serve() est un serveur HTTP natif zéro-config (fetch-based, routing intégré) |  |


