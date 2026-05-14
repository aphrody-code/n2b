use n2b_types::types::{Finding, MakeFindingOpts, Severity};
use n2b_util::{line_offsets, make_finding};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

enum ReplaceKind {
    None,
    Static(&'static str),
    Template(&'static str), // utilise $1..$n
}

struct ApiRule {
    id: &'static str,
    re: Regex,
    message: &'static str,
    replace: ReplaceKind,
    aggressive: bool,
    severity: Severity,
}

fn rule(
    id: &'static str,
    pat: &str,
    msg: &'static str,
    replace: ReplaceKind,
    aggressive: bool,
) -> ApiRule {
    ApiRule {
        id,
        re: Regex::new(pat).unwrap_or_else(|e| {
            panic!("invariant: ApiRule pattern for rule '{id}' is invalid: {e}")
        }),
        message: msg,
        replace,
        aggressive,
        severity: Severity::Warn,
    }
}

fn info_rule(
    id: &'static str,
    pat: &str,
    msg: &'static str,
    replace: ReplaceKind,
    aggressive: bool,
) -> ApiRule {
    ApiRule {
        severity: Severity::Info,
        ..rule(id, pat, msg, replace, aggressive)
    }
}

static RULES: Lazy<Vec<ApiRule>> = Lazy::new(|| {
    use ReplaceKind::*;
    vec![
        rule(
            "api/fs-readFileSync",
            r#"\bfs\.readFileSync\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)"#,
            "remplacer fs.readFileSync(path, 'utf8') par await Bun.file(path).text()",
            Template("await Bun.file($1).text()"),
            true,
        ),
        rule(
            "api/fs-writeFileSync",
            r#"\bfs\.writeFileSync\s*\(\s*([^,]+?)\s*,\s*([^)]+?)\s*\)"#,
            "remplacer fs.writeFileSync(path, data) par await Bun.write(path, data)",
            Template("await Bun.write($1, $2)"),
            true,
        ),
        info_rule(
            "api/process-env",
            r"\bprocess\.env\.([A-Z_][A-Z0-9_]*)\b",
            "Bun.env est un alias plus court de process.env (préférence stylistique)",
            None,
            false,
        ),
        rule(
            "api/dirname-esm",
            r"\b(?:const|let|var)\s+__dirname\s*=\s*(?:path\.)?dirname\s*\(\s*fileURLToPath\s*\(\s*import\.meta\.url\s*\)\s*\)",
            "dans un ESM Bun, utiliser directement import.meta.dir (ou import.meta.dirname)",
            Static("const __dirname = import.meta.dir"),
            true,
        ),
        rule(
            "api/filename-esm",
            r"\b(?:const|let|var)\s+__filename\s*=\s*fileURLToPath\s*\(\s*import\.meta\.url\s*\)",
            "dans un ESM Bun, utiliser import.meta.path (ou import.meta.filename)",
            Static("const __filename = import.meta.path"),
            true,
        ),
        rule(
            "api/express-server",
            r#"\b(?:const|let|var)\s+(\w+)\s*=\s*require\(\s*['"]express['"]\s*\)\s*\(\s*\)"#,
            "envisager Bun.serve() plutôt qu'Express pour un serveur simple (voir runtime/http)",
            None,
            false,
        ),
        rule(
            "api/child-process-spawn",
            r"\b(?:child_process\.)?spawn\s*\(",
            "Bun.spawn offre une API plus ergonomique (streams Web, ipc, preload)",
            None,
            false,
        ),
        rule(
            "api/crypto-createHash",
            r#"\bcrypto\.createHash\s*\(\s*['"](?:md5|sha1|sha256|sha512|blake2b256)['"]\s*\)"#,
            "Bun.hash / Bun.CryptoHasher est plus rapide (voir runtime/hashing)",
            None,
            false,
        ),
        rule(
            "api/buffer-from-base64",
            r#"\bBuffer\.from\s*\(\s*([^,]+?)\s*,\s*['"]base64['"]\s*\)"#,
            "utiliser atob() / btoa() ou Uint8Array pour du Web-standard",
            None,
            false,
        ),
        rule(
            "api/fileURLToPath",
            r"\bfileURLToPath\s*\(",
            "Bun.fileURLToPath() est équivalent (ou utiliser import.meta.dir/path)",
            None,
            false,
        ),
        rule(
            "api/uuid-v4",
            r"\b(?:uuidv4|v4)\s*\(\s*\)",
            "crypto.randomUUID() (global) ou Bun.randomUUIDv7() évite la dépendance uuid",
            None,
            false,
        ),
        rule(
            "api/fs-readFile-utf8",
            r#"\bfs\.readFile\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*,\s*([^)]+?)\s*\)"#,
            "remplacer fs.readFile(path, 'utf8', cb) par await Bun.file(path).text()",
            None,
            false,
        ),
        rule(
            "api/fs-readFile-promise",
            r#"\bfsPromises\.readFile\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)"#,
            "remplacer fsPromises.readFile(path, 'utf8') par await Bun.file(path).text()",
            Template("await Bun.file($1).text()"),
            true,
        ),
        rule(
            "api/json-parse-readFileSync",
            r#"\bJSON\.parse\s*\(\s*fs\.readFileSync\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)\s*\)"#,
            "remplacer JSON.parse(fs.readFileSync(path,'utf8')) par await Bun.file(path).json()",
            Template("await Bun.file($1).json()"),
            true,
        ),
        rule(
            "api/fs-existsSync",
            r"\bfs\.existsSync\s*\(\s*([^)]+?)\s*\)",
            "remplacer fs.existsSync(path) par await Bun.file(path).exists() (uniquement pour un fichier — pour un dossier, Bun.file().exists() retourne toujours false)",
            Template("await Bun.file($1).exists()"),
            true,
        ),
        rule(
            "api/http-createServer",
            r"\bhttp\.createServer\s*\(",
            "envisager Bun.serve() plutôt que http.createServer (API fetch-based, plus simple)",
            None,
            false,
        ),
        rule(
            "api/https-createServer",
            r"\bhttps\.createServer\s*\(",
            "envisager Bun.serve({ tls }) plutôt que https.createServer",
            None,
            false,
        ),
        rule(
            "api/execSync",
            r"\b(?:child_process\.)?execSync\s*\(",
            "utiliser le shell Bun ($`cmd`) ou Bun.spawnSync() à la place de execSync",
            None,
            false,
        ),
        rule(
            "api/exec",
            r"\b(?:child_process\.)?exec\s*\(",
            "utiliser le shell Bun ($`cmd`) ou Bun.spawn() à la place de exec",
            None,
            false,
        ),
        rule(
            "api/buffer-alloc",
            r"\bBuffer\.alloc\s*\(\s*([^)]+?)\s*\)",
            "remplacer Buffer.alloc(n) par new Uint8Array(n) (Web-standard)",
            Template("new Uint8Array($1)"),
            true,
        ),
        rule(
            "api/buffer-concat",
            r"\bBuffer\.concat\s*\(",
            "utiliser Uint8Array et concaténation Web-standard plutôt que Buffer.concat",
            None,
            false,
        ),
        rule(
            "api/buffer-from-string",
            r#"\bBuffer\.from\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)"#,
            "remplacer Buffer.from(str, 'utf8') par new TextEncoder().encode(str)",
            Template("new TextEncoder().encode($1)"),
            true,
        ),
        rule(
            "api/process-stdout-write",
            r"\bprocess\.stdout\.write\s*\(",
            "Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write",
            None,
            false,
        ),
        rule(
            "api/process-stderr-write",
            r"\bprocess\.stderr\.write\s*\(",
            "Bun.stderr.write() est l'équivalent natif Bun de process.stderr.write",
            None,
            false,
        ),
        rule(
            "api/sleep-promise",
            r"\bnew\s+Promise\s*\(\s*(?:resolve|res)\s*=>\s*setTimeout\s*\(\s*(?:resolve|res)\s*,\s*([^)]+?)\s*\)\s*\)",
            "remplacer new Promise(res => setTimeout(res, ms)) par Bun.sleep(ms)",
            Template("Bun.sleep($1)"),
            true,
        ),
        rule(
            "api/util-promisify",
            r"\butil\.promisify\s*\(",
            "préférer les APIs async natives de Bun/Node plutôt que util.promisify",
            None,
            false,
        ),
        rule(
            "api/util-inspect",
            r"\butil\.inspect\s*\(",
            "Bun.inspect() est l'équivalent natif (pretty-print avec couleurs)",
            Static("Bun.inspect("),
            true,
        ),
        info_rule(
            "api/new-url-import-meta",
            r#"\bnew\s+URL\s*\(\s*['"][^'"]+['"]\s*,\s*import\.meta\.url\s*\)"#,
            "utiliser import.meta.dir ou path.join(import.meta.dir, ...) plutôt que new URL(..., import.meta.url)",
            None,
            false,
        ),
        rule(
            "api/toml-parse",
            r"\b(?:TOML|toml)\.parse\s*\(",
            "Bun.TOML.parse() est disponible nativement — supprimer la dépendance TOML externe",
            None,
            false,
        ),
        rule(
            "api/semver",
            r"\b(?:semver\.satisfies|semver\.valid|semver\.gt|semver\.lt|semver\.gte|semver\.lte|semver\.coerce)\s*\(",
            "Bun.semver.satisfies() et autres helpers sont disponibles nativement",
            None,
            false,
        ),
        info_rule(
            "api/performance-now",
            r"\bperformance\.now\s*\(\s*\)",
            "Bun.nanoseconds() offre une horloge haute précision (retourne nanosecondes depuis démarrage)",
            None,
            false,
        ),
        // --- Nouvelles règles issues de l'analyse sur discord.js/nextjs/prisma ---
        rule(
            "api/buffer-byteLength",
            r#"\bBuffer\.byteLength\s*\(\s*([^,)]+?)\s*(?:,\s*['"]utf-?8['"]\s*)?\)"#,
            "remplacer Buffer.byteLength(str) par new TextEncoder().encode(str).length (Web-standard)",
            Template("new TextEncoder().encode($1).length"),
            true,
        ),
        rule(
            "api/require-resolve",
            r"\brequire\.resolve\s*\(",
            "Bun.resolveSync() remplace require.resolve() (ESM + CJS, plus rapide)",
            None,
            false,
        ),
        rule(
            "api/set-immediate",
            r"\bsetImmediate\s*\(",
            "setImmediate n'est pas Web-standard — utiliser queueMicrotask() ou setTimeout(fn, 0)",
            None,
            false,
        ),
        info_rule(
            "api/os-platform",
            r"\bos\.platform\s*\(\s*\)",
            "process.platform (global) retourne la même chose sans import 'os'",
            None,
            false,
        ),
        info_rule(
            "api/os-homedir",
            r"\bos\.homedir\s*\(\s*\)",
            "Bun expose Bun.env.HOME / process.env.HOME — évite l'import 'os'",
            None,
            false,
        ),
        rule(
            "api/path-join-dirname",
            r"\bpath\.join\s*\(\s*__dirname\s*,",
            "dans un ESM Bun, path.join(import.meta.dir, ...) évite __dirname",
            None,
            false,
        ),
        // --- Bun.password (bcrypt / argon2) ---
        rule(
            "api/bcrypt-hash",
            r"\bbcrypt(?:js)?\.hash\s*\(",
            "Bun.password.hash() supporte bcrypt et argon2 nativement (plus rapide, pas de dep native)",
            None,
            false,
        ),
        rule(
            "api/bcrypt-compare",
            r"\bbcrypt(?:js)?\.compare\s*\(",
            "Bun.password.verify() remplace bcrypt.compare()",
            None,
            false,
        ),
        rule(
            "api/argon2-hash",
            r#"\bargon2\.(?:hash|verify)\s*\("#,
            "Bun.password.hash({ algorithm: 'argon2id' }) / verify() est natif",
            None,
            false,
        ),
        // --- Bun.YAML ---
        rule(
            "api/yaml-parse",
            r#"\b(?:yaml|YAML|jsyaml)\.(?:load|parse)\s*\("#,
            "Bun.YAML.parse() est natif — plus besoin de js-yaml/yaml",
            None,
            false,
        ),
        rule(
            "api/yaml-stringify",
            r#"\b(?:yaml|YAML|jsyaml)\.(?:dump|stringify)\s*\("#,
            "Bun.YAML.stringify() est natif",
            None,
            false,
        ),
        // --- Bun.JSON5 / Bun.JSONC ---
        rule(
            "api/json5-parse",
            r"\bJSON5\.parse\s*\(",
            "Bun.JSON5.parse() est natif — plus besoin de la dep json5",
            Static("Bun.JSON5.parse("),
            true,
        ),
        rule(
            "api/json5-stringify",
            r"\bJSON5\.stringify\s*\(",
            "Bun.JSON5.stringify() est natif",
            Static("Bun.JSON5.stringify("),
            true,
        ),
        // --- Bun.markdown ---
        rule(
            "api/marked-call",
            r"\bmarked\s*\(",
            "Bun.markdown.html() est natif — plus besoin de marked",
            None,
            false,
        ),
        rule(
            "api/marked-parse",
            r"\bmarked\.parse\s*\(",
            "Bun.markdown.html() est natif",
            None,
            false,
        ),
        // --- Bun.escapeHTML ---
        rule(
            "api/escape-html",
            r"\b(?:escapeHtml|escape_html|he\.encode)\s*\(",
            "Bun.escapeHTML() est natif (UTF-8, ~3× plus rapide que 'he')",
            None,
            false,
        ),
        // --- Bun.stripANSI / Bun.stringWidth / Bun.sliceAnsi ---
        rule(
            "api/strip-ansi",
            r"\bstripAnsi\s*\(",
            "Bun.stripANSI() est natif",
            None,
            false,
        ),
        rule(
            "api/string-width",
            r"\bstringWidth\s*\(",
            "Bun.stringWidth() est natif (fullwidth + emoji aware)",
            None,
            false,
        ),
        rule(
            "api/slice-ansi",
            r"\bsliceAnsi\s*\(",
            "Bun.sliceAnsi() est natif",
            None,
            false,
        ),
        // --- Bun.which ---
        rule(
            "api/which-call",
            r#"\b(?:which|sync\.which)\s*\(\s*['"][^'"]+['"]\s*\)"#,
            "Bun.which() remplace la dep 'which' (natif, zéro allocation)",
            None,
            false,
        ),
        // --- Bun.cron ---
        rule(
            "api/cron-schedule",
            r#"\b(?:cron|nodeCron)\.schedule\s*\("#,
            "Bun.cron remplace node-cron / croner",
            None,
            false,
        ),
        rule(
            "api/cronjob-new",
            r"\bnew\s+CronJob\s*\(",
            "Bun.cron est l'API native — plus besoin de node-cron/croner",
            None,
            false,
        ),
        // --- Bun.deepEquals ---
        rule(
            "api/fast-deep-equal",
            r"\bfastDeepEqual\s*\(",
            "Bun.deepEquals() est natif (WebKit structural compare)",
            None,
            false,
        ),
        // --- Bun.gzipSync / Bun.gunzipSync ---
        rule(
            "api/pako-gzip",
            r"\bpako\.(?:gzip|deflate)\s*\(",
            "Bun.gzipSync() / Bun.deflateSync() remplacent pako (natif, SIMD)",
            None,
            false,
        ),
        rule(
            "api/pako-gunzip",
            r"\bpako\.(?:ungzip|inflate)\s*\(",
            "Bun.gunzipSync() / Bun.inflateSync() remplacent pako",
            None,
            false,
        ),
        // --- Bun.serve vs frameworks ---
        info_rule(
            "api/express-app",
            r"\b(?:const|let|var)\s+\w+\s*=\s*express\s*\(\s*\)",
            "Bun.serve() est un serveur HTTP natif zéro-config (fetch-based, routing intégré)",
            None,
            false,
        ),
        info_rule(
            "api/fastify-app",
            r"\b(?:const|let|var)\s+\w+\s*=\s*fastify\s*\(",
            "Bun.serve() + routing natif remplace fastify pour la plupart des cas simples",
            None,
            false,
        ),
        info_rule(
            "api/koa-new",
            r"\bnew\s+Koa\s*\(\s*\)",
            "Bun.serve() avec middleware chaîné couvre la plupart des usages Koa",
            None,
            false,
        ),
        // --- fetch vs http.request / https.request ---
        rule(
            "api/http-request",
            r"\b(?:const|let|var)\s+\w+\s*=\s*http\.request\s*\(",
            "fetch() (global) remplace http.request avec une API plus simple",
            None,
            false,
        ),
        rule(
            "api/https-request",
            r"\b(?:const|let|var)\s+\w+\s*=\s*https\.request\s*\(",
            "fetch() (global) remplace https.request",
            None,
            false,
        ),
        // --- crypto.randomBytes → Web crypto ---
        rule(
            "api/crypto-randomBytes",
            r"\bcrypto\.randomBytes\s*\(",
            "crypto.getRandomValues(new Uint8Array(n)) est Web-standard (pas besoin d'import 'crypto')",
            None,
            false,
        ),
        // --- Bun.EventSource ---
        info_rule(
            "api/eventsource-new",
            r"\bnew\s+EventSource\s*\(",
            "EventSource est global dans Bun (Bun.EventSource) — plus besoin de la dep 'eventsource'",
            None,
            false,
        ),
        // --- Cookie / CookieMap ---
        rule(
            "api/cookie-parse",
            r"\bcookie\.parse\s*\(",
            "new Bun.CookieMap(header) remplace cookie.parse()",
            None,
            false,
        ),
        rule(
            "api/cookie-serialize",
            r"\bcookie\.serialize\s*\(",
            "Bun.Cookie.toString() remplace cookie.serialize()",
            None,
            false,
        ),
        // --- Bun.s3 ---
        info_rule(
            "api/aws-sdk-s3-client",
            r"\bnew\s+S3Client\s*\(",
            "Bun.s3 / Bun.S3Client est natif (multipart upload, presign) — pas besoin de @aws-sdk",
            None,
            false,
        ),
        // --- Bun.FileSystemRouter ---
        info_rule(
            "api/file-based-routing",
            r#"\b(?:next-router|next/router|@tanstack/router-vite-plugin)"#,
            "Bun.FileSystemRouter expose un routeur file-based sans build step",
            None,
            false,
        ),
        // --- Bun.color / Bun.Terminal ---
        info_rule(
            "api/chalk-call",
            r"\bchalk\.(?:red|green|yellow|blue|magenta|cyan|white|gray|grey|bold|dim|italic|underline)\b",
            "Bun supporte ANSI nativement ; Bun.color() permet la conversion programmatique",
            None,
            false,
        ),
        // --- zlib.gzipSync (Bun version plus rapide) ---
        info_rule(
            "api/zlib-gzipSync",
            r"\bzlib\.gzipSync\s*\(",
            "Bun.gzipSync() est plus rapide que zlib.gzipSync (libdeflate SIMD)",
            None,
            false,
        ),
        // --- Bun.nanoseconds vs process.hrtime.bigint ---
        info_rule(
            "api/process-hrtime-bigint",
            r"\bprocess\.hrtime\.bigint\s*\(\s*\)",
            "Bun.nanoseconds() est l'équivalent natif (retourne un Number plutôt qu'un BigInt)",
            None,
            false,
        ),
        // --- Bun.spawn vs execa ---
        rule(
            "api/execa-call",
            r"\bexeca(?:Sync|Command|CommandSync)?\s*\(",
            "le shell Bun ($`cmd`) ou Bun.spawn() remplacent execa",
            None,
            false,
        ),
        // --- Next.js custom server ---
        rule(
            "next/custom-server-next-app",
            r"\bnext\s*\(\s*\{\s*(?:[^{}]*\b)?dev\s*:",
            "Pattern Next.js custom server détecté (const app = next({ dev }) — Bun.serve({ fetch }) remplace http.createServer + app.getRequestHandler pour de meilleures perfs",
            None,
            false,
        ),
        rule(
            "next/request-handler",
            r"\bapp\.getRequestHandler\s*\(\s*\)",
            "getRequestHandler() retourne un handler (req, res) — pour Bun.serve, wrapper : fetch: (req) => handler(req, Bun.nodeHttpResponse()) ou utiliser un adapter",
            None,
            false,
        ),
    ]
});

pub fn apply_bun_api_rules(path: &str, source: &str, aggressive: bool) -> (Vec<Finding>, String) {
    let offsets = line_offsets(source);
    let mut findings: Vec<Finding> = Vec::new();
    struct Edit {
        index: usize,
        len: usize,
        replacement: String,
    }
    let mut edits: Vec<Edit> = Vec::new();

    for r in RULES.iter() {
        for mat in r.re.captures_iter(source) {
            let whole = mat
                .get(0)
                .expect("invariant: capture group 0 is always present in a match");
            let index = whole.start();

            // Bug fix : api/exec matche aussi regex.exec() et string.exec() (accès membre).
            // Ne garder que les appels child_process → soit `child_process.exec(` explicite,
            // soit `exec(` en position d'appel directe (pas précédé d'un `.`).
            // Early continue pour éviter toute allocation inutile.
            if (r.id == "api/exec" || r.id == "api/execSync") && is_member_exec_call(source, index)
            {
                continue;
            }

            let original = whole.as_str();
            let original_len = original.len();
            let replacement = match &r.replace {
                ReplaceKind::None => None,
                ReplaceKind::Static(s) => Some((*s).to_string()),
                ReplaceKind::Template(t) => Some(expand(&mat, t)),
            };

            // Bug fix : fs.existsSync(path) suivi dans les ~15 lignes par fs.mkdirSync(path, ...)
            // indique un contexte DOSSIER — Bun.file().exists() retourne toujours false pour un dir.
            // On dégrade l'autofix en simple warning non-appliqué.
            let skip_autofix = r.id == "api/fs-existsSync"
                && mat.get(1).is_some_and(|m| {
                    let arg = m.as_str().trim();
                    looks_like_dir_context(source, index, arg)
                });

            let has_repl = replacement.is_some() && !skip_autofix;
            let replacement_for_edit = if aggressive && r.aggressive && !skip_autofix {
                replacement.clone()
            } else {
                None
            };
            findings.push(make_finding(
                path,
                &offsets,
                index,
                r.id,
                if skip_autofix {
                    "fs.existsSync(path) suivi d'un fs.mkdirSync(path) — chemin probablement un dossier, Bun.file().exists() inadapté (utiliser fs.mkdirSync(path, { recursive: true }))".to_string()
                } else {
                    r.message.to_string()
                },
                original.to_string(),
                if skip_autofix { None } else { replacement },
                MakeFindingOpts {
                    autofix: Some(has_repl),
                    aggressive: if r.aggressive { Some(true) } else { None },
                    severity: Some(r.severity),
                },
            ));
            if let Some(repl) = replacement_for_edit {
                edits.push(Edit {
                    index,
                    len: original_len,
                    replacement: repl,
                });
            }
        }
    }

    let mut out = source.to_string();
    if !edits.is_empty() {
        // Résout les overlaps : quand deux règles matchent des ranges imbriqués
        // (ex. api/fs-readFileSync à l'intérieur de api/json-parse-readFileSync),
        // on garde le range le plus large. Tri (index asc, len desc) puis filtre.
        edits.sort_by(|a, b| a.index.cmp(&b.index).then(b.len.cmp(&a.len)));
        let mut kept: Vec<Edit> = Vec::with_capacity(edits.len());
        for e in edits {
            let overlaps_prev = kept
                .last()
                .map(|p| p.index + p.len > e.index)
                .unwrap_or(false);
            if !overlaps_prev {
                kept.push(e);
            }
        }
        kept.sort_unstable_by_key(|e| std::cmp::Reverse(e.index));
        for e in kept {
            out.replace_range(e.index..e.index + e.len, &e.replacement);
        }
    }
    (findings, out)
}

fn expand(caps: &Captures, template: &str) -> String {
    let mut out = String::with_capacity(template.len());
    caps.expand(template, &mut out);
    out
}

/// Retourne true si un fs.mkdirSync(<arg>, ...) apparaît dans les ~600 octets
/// (≈ 15 lignes) qui suivent la position `pos` — indicateur qu'il s'agit d'un
/// dossier et non d'un fichier.
fn looks_like_dir_context(source: &str, pos: usize, arg: &str) -> bool {
    let end = (pos + 600).min(source.len());
    let window = &source[pos..end];
    let needle_sync = format!("fs.mkdirSync({arg}");
    let needle_async = format!("fs.mkdir({arg}");
    window.contains(&needle_sync) || window.contains(&needle_async)
}

/// Détecte les appels `.exec(` / `.execSync(` sur un objet (RegExp, etc.) plutôt
/// que child_process. Si le `exec` est précédé d'un `.` ET que ce qui précède
/// n'est pas `child_process`, alors c'est un appel membre → faux positif.
fn is_member_exec_call(source: &str, pos: usize) -> bool {
    if pos == 0 {
        return false;
    }
    let bytes = source.as_bytes();
    if bytes[pos - 1] != b'.' {
        return false;
    }
    // Si on est juste après "child_process." → c'est le vrai appel Node
    !source[..pos].ends_with("child_process.")
}
