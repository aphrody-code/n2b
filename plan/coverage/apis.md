# coverage/apis — les 72 règles `api/*` actuelles

> Inventaire de `crates/n2b-rules/src/bun_apis.rs:53-591` (`RULES: Vec<ApiRule>`).
> **★ = réécriture effective** (13 règles) · les 59 autres sont warning/info pur.
>
> Cible Phase 1 : migrer dans `registry/apis.toml`. Cible Phase 5 : donner une `rewrite`
> exploitable à **chaque** ligne (aujourd'hui 13/72 ≈ 18 %).

## Règles avec réécriture effective (13) — `rewrite = template|static`

| Rule ID | Détecte | Cible Bun | Type |
|---|---|---|---|
| `api/fs-readFileSync` ★ | `fs.readFileSync(p,'utf8')` | `await Bun.file($1).text()` | template |
| `api/fs-writeFileSync` ★ | `fs.writeFileSync(p,d)` | `await Bun.write($1, $2)` | template |
| `api/fs-readFile-promise` ★ | `fsPromises.readFile(p,'utf8')` | `await Bun.file($1).text()` | template |
| `api/json-parse-readFileSync` ★ | `JSON.parse(fs.readFileSync(p,'utf8'))` | `await Bun.file($1).json()` | template |
| `api/fs-existsSync` ★ | `fs.existsSync(p)` | `await Bun.file($1).exists()` | template (dégradé si contexte dossier) |
| `api/dirname-esm` ★ | `__dirname = dirname(fileURLToPath(...))` | `const __dirname = import.meta.dir` | static |
| `api/filename-esm` ★ | `__filename = fileURLToPath(...)` | `const __filename = import.meta.path` | static |
| `api/buffer-alloc` ★ | `Buffer.alloc(n)` | `new Uint8Array($1)` | template |
| `api/buffer-from-string` ★ | `Buffer.from(s,'utf8')` | `new TextEncoder().encode($1)` | template |
| `api/buffer-byteLength` ★ | `Buffer.byteLength(s)` | `new TextEncoder().encode($1).length` | template |
| `api/sleep-promise` ★ | `new Promise(res=>setTimeout(res,ms))` | `Bun.sleep($1)` | template |
| `api/util-inspect` ★ | `util.inspect(` | `Bun.inspect(` | static |
| `api/json5-parse` ★ / `api/json5-stringify` ★ | `JSON5.parse/stringify(` | `Bun.JSON5.parse/stringify(` | static |

## Règles warning/info sans réécriture (59) — `rewrite` à compléter en Phase 5

### Process / runtime
| Rule ID | Détecte | Cible Bun | `rewrite` cible |
|---|---|---|---|
| `api/process-env` | `process.env.X` | `Bun.env.X` (ou natif) | info → `template` |
| `api/process-stdout-write` | `process.stdout.write(` | `Bun.stdout.write(` | `template` |
| `api/process-stderr-write` | `process.stderr.write(` | `Bun.stderr.write(` | `template` |
| `api/process-hrtime-bigint` | `process.hrtime.bigint()` | `Bun.nanoseconds()` | `template` |
| `api/performance-now` | `performance.now()` | natif (global) | info — `drop` warning |
| `api/set-immediate` | `setImmediate(` | `setTimeout(_,0)` ou natif | `template` |
| `api/require-resolve` | `require.resolve(` | `Bun.resolveSync(` | `template` |
| `api/util-promisify` | `util.promisify(` | API déjà Promise | `manual` |

### Child process / shell
| Rule ID | Détecte | Cible Bun | `rewrite` cible |
|---|---|---|---|
| `api/child-process-spawn` | `spawn(` | `Bun.spawn` | `manual` |
| `api/execSync` | `execSync(` | `Bun.$\`...\`` | `manual` |
| `api/exec` | `exec(` | `Bun.$` / `Bun.spawn` | `manual` |
| `api/execa-call` | `execa(` | `Bun.$` / `Bun.spawn` | `manual` |
| `api/which-call` | `which('…')` | `Bun.which(` | `template` |

### Crypto / hashing
| Rule ID | Détecte | Cible Bun | `rewrite` cible |
|---|---|---|---|
| `api/crypto-createHash` | `crypto.createHash('md5'…)` | `Bun.CryptoHasher` | `template` |
| `api/crypto-randomBytes` | `crypto.randomBytes(` | `crypto.getRandomValues` natif | `template` |
| `api/bcrypt-hash` / `api/bcrypt-compare` | `bcrypt.hash/compare(` | `Bun.password.hash/verify` | `template` |
| `api/argon2-hash` | `argon2.hash/verify(` | `Bun.password` (argon2id) | `template` |
| `api/uuid-v4` | `uuidv4()` / `v4()` | `Bun.randomUUIDv7()` | `template` (⚠️ PS1 — `v4()` homonyme) |
| `api/buffer-from-base64` | `Buffer.from(x,'base64')` | `Uint8Array.fromBase64` | `template` |

### HTTP / serveurs
| Rule ID | Détecte | Cible Bun | `rewrite` cible |
|---|---|---|---|
| `api/http-createServer` / `api/https-createServer` | `http(s).createServer(` | `Bun.serve` | `manual` |
| `api/http-request` / `api/https-request` | `http(s).request(` | `fetch` | `manual` |
| `api/express-server` / `api/express-app` | `require('express')()` / `express()` | `Bun.serve` | `manual` (shim) |
| `api/fastify-app` | `fastify(` | `Bun.serve` | `manual` (shim) |
| `api/koa-new` | `new Koa()` | `Bun.serve` | `manual` (shim) |
| `api/eventsource-new` | `new EventSource(` | `Bun.EventSource` (natif) | `template` |
| `next/custom-server-next-app` | `next({ dev })` | — | `manual` |
| `next/request-handler` | `app.getRequestHandler()` | — | `manual` |

### Buffer / encodage
| Rule ID | Détecte | Cible Bun | `rewrite` cible |
|---|---|---|---|
| `api/buffer-concat` | `Buffer.concat(` | `Bun.concatArrayBuffers` | `template` |
| `api/fileURLToPath` | `fileURLToPath(` | `Bun.fileURLToPath` | `template` |
| `api/new-url-import-meta` | `new URL('…',import.meta.url)` | natif | info — `drop` warning |

### OS / FS / path
| Rule ID | Détecte | `rewrite` cible |
|---|---|---|
| `api/os-platform` | `os.platform()` | info — natif OK |
| `api/os-homedir` | `os.homedir()` | info — natif OK |
| `api/path-join-dirname` | `path.join(__dirname,` | `manual` (lié à `globals/dirname`) |
| `api/fs-readFile-utf8` | `fs.readFile(p,'utf8',cb)` | `manual` (callback → Promise) |

### Parsing / formats
| Rule ID | Détecte | Cible Bun | `rewrite` cible |
|---|---|---|---|
| `api/toml-parse` | `TOML.parse(` | `Bun.TOML.parse(` | `template` |
| `api/yaml-parse` / `api/yaml-stringify` | `yaml.load/parse/dump(` | `Bun.YAML` | `template` |
| `api/semver` | `semver.satisfies(`… | `Bun.semver` | `template` |
| `api/marked-call` / `api/marked-parse` | `marked(` / `marked.parse(` | `Bun.markdown` | `template` (⚠️ PS1 — `marked` homonyme) |
| `api/escape-html` | `escapeHtml(` / `he.encode(` | `Bun.escapeHTML` | `template` |
| `api/strip-ansi` | `stripAnsi(` | `Bun.stripANSI` | `template` |
| `api/string-width` | `stringWidth(` | `Bun.stringWidth` | `template` |
| `api/slice-ansi` | `sliceAnsi(` | `Bun.sliceAnsi` | `template` |
| `api/chalk-call` | `chalk.red`… | natif (`Bun.color`) | `manual` |

### Compression
| Rule ID | Détecte | Cible Bun | `rewrite` cible |
|---|---|---|---|
| `api/zlib-gzipSync` | `zlib.gzipSync(` | `Bun.gzipSync` | `template` |
| `api/pako-gzip` / `api/pako-gunzip` | `pako.gzip/ungzip(` | `Bun.gzipSync/gunzipSync` | `template` |

### Divers
| Rule ID | Détecte | Cible Bun | `rewrite` cible |
|---|---|---|---|
| `api/cron-schedule` / `api/cronjob-new` | `cron.schedule(` / `new CronJob(` | `Bun.cron` | `template` |
| `api/fast-deep-equal` | `fastDeepEqual(` | `Bun.deepEquals` | `template` |
| `api/cookie-parse` / `api/cookie-serialize` | `cookie.parse/serialize(` | `Bun.Cookie` / `Bun.CookieMap` | `template` |
| `api/aws-sdk-s3-client` | `new S3Client(` | `Bun.s3` / `Bun.S3Client` | `manual` |
| `api/file-based-routing` | `next/router`… | `Bun.FileSystemRouter` | `manual` |

## Cibles Bun non encore exploitées

Surface `Bun.*` (depuis `docs/bun/runtime/bun-apis.mdx`) sans règle `api/*`
correspondante — **trous de couverture du pilier 2** à combler Phase 4/5 :

| Bun API | Remplace | Règle `api/*` à créer |
|---|---|---|
| `Bun.Transpiler` | `babel`, `ts-node` | `api/babel-transform` |
| `Bun.build` | `webpack`, `esbuild`, `rollup` | `api/bundler-call` |
| `HTMLRewriter` | `cheerio`, `jsdom` | `api/cheerio-load` |
| `Bun.listen` / `Bun.connect` | `net.createServer/connect` | `api/net-server` |
| `Bun.udpSocket` | `dgram.createSocket` | `api/dgram-socket` |
| `Bun.CSRF` | `csurf` | `api/csurf-call` |
| `Bun.dns.lookup` | `dns.lookup` | `api/dns-lookup` |
| `Bun.peek` / `Bun.deepMatch` | — | (natif, pas de dep à migrer) |
| `bun:ffi` | `node-gyp`, `ffi-napi` | `api/ffi-napi` |
| `Bun.FileSystemRouter` | routeurs custom | `api/file-based-routing` (étendre) |

## Note PS1 — règles à risque de faux positif

Ces règles matchent un **identifiant nu** et déclencheront un faux positif sur une
fonction locale homonyme tant que le matching n'est pas AST (Phase 2) :
`api/uuid-v4` (`v4()`), `api/marked-call` (`marked(`), `api/which-call` (`which(`),
`api/exec` / `api/execSync` (`exec(` — déjà patché ad-hoc par `is_member_exec_call`),
`api/fast-deep-equal` (`fastDeepEqual(`), `api/string-width` (`stringWidth(`),
`api/strip-ansi` (`stripAnsi(`), `api/escape-html` (`escapeHtml(`). Toutes gagnent le
champ `import_from` au registre (Phase 1) et un matching corrélé au binding (Phase 2).
