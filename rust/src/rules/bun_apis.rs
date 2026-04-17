use crate::types::{Finding, MakeFindingOpts, Severity};
use crate::util::{line_offsets, make_finding};
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

fn rule(id: &'static str, pat: &str, msg: &'static str, replace: ReplaceKind, aggressive: bool) -> ApiRule {
    ApiRule {
        id,
        re: Regex::new(pat).unwrap(),
        message: msg,
        replace,
        aggressive,
        severity: Severity::Warn,
    }
}

fn info_rule(id: &'static str, pat: &str, msg: &'static str, replace: ReplaceKind, aggressive: bool) -> ApiRule {
    ApiRule {
        severity: Severity::Info,
        ..rule(id, pat, msg, replace, aggressive)
    }
}

static RULES: Lazy<Vec<ApiRule>> = Lazy::new(|| {
    use ReplaceKind::*;
    vec![
        rule("api/fs-readFileSync",
            r#"\bfs\.readFileSync\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)"#,
            "remplacer fs.readFileSync(path, 'utf8') par await Bun.file(path).text()",
            Template("await Bun.file($1).text()"), true),
        rule("api/fs-writeFileSync",
            r#"\bfs\.writeFileSync\s*\(\s*([^,]+?)\s*,\s*([^)]+?)\s*\)"#,
            "remplacer fs.writeFileSync(path, data) par await Bun.write(path, data)",
            Template("await Bun.write($1, $2)"), true),
        info_rule("api/process-env",
            r"\bprocess\.env\.([A-Z_][A-Z0-9_]*)\b",
            "Bun.env est un alias plus court de process.env (préférence stylistique)",
            None, false),
        rule("api/dirname-esm",
            r"\b(?:const|let|var)\s+__dirname\s*=\s*(?:path\.)?dirname\s*\(\s*fileURLToPath\s*\(\s*import\.meta\.url\s*\)\s*\)",
            "dans un ESM Bun, utiliser directement import.meta.dir (ou import.meta.dirname)",
            Static("const __dirname = import.meta.dir"), true),
        rule("api/filename-esm",
            r"\b(?:const|let|var)\s+__filename\s*=\s*fileURLToPath\s*\(\s*import\.meta\.url\s*\)",
            "dans un ESM Bun, utiliser import.meta.path (ou import.meta.filename)",
            Static("const __filename = import.meta.path"), true),
        rule("api/express-server",
            r#"\b(?:const|let|var)\s+(\w+)\s*=\s*require\(\s*['"]express['"]\s*\)\s*\(\s*\)"#,
            "envisager Bun.serve() plutôt qu'Express pour un serveur simple (voir runtime/http)",
            None, false),
        rule("api/child-process-spawn",
            r"\b(?:child_process\.)?spawn\s*\(",
            "Bun.spawn offre une API plus ergonomique (streams Web, ipc, preload)",
            None, false),
        rule("api/crypto-createHash",
            r#"\bcrypto\.createHash\s*\(\s*['"](?:md5|sha1|sha256|sha512|blake2b256)['"]\s*\)"#,
            "Bun.hash / Bun.CryptoHasher est plus rapide (voir runtime/hashing)",
            None, false),
        rule("api/buffer-from-base64",
            r#"\bBuffer\.from\s*\(\s*([^,]+?)\s*,\s*['"]base64['"]\s*\)"#,
            "utiliser atob() / btoa() ou Uint8Array pour du Web-standard",
            None, false),
        rule("api/fileURLToPath",
            r"\bfileURLToPath\s*\(",
            "Bun.fileURLToPath() est équivalent (ou utiliser import.meta.dir/path)",
            None, false),
        rule("api/uuid-v4",
            r"\b(?:uuidv4|v4)\s*\(\s*\)",
            "crypto.randomUUID() (global) ou Bun.randomUUIDv7() évite la dépendance uuid",
            None, false),
        rule("api/fs-readFile-utf8",
            r#"\bfs\.readFile\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*,\s*([^)]+?)\s*\)"#,
            "remplacer fs.readFile(path, 'utf8', cb) par await Bun.file(path).text()",
            None, false),
        rule("api/fs-readFile-promise",
            r#"\bfsPromises\.readFile\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)"#,
            "remplacer fsPromises.readFile(path, 'utf8') par await Bun.file(path).text()",
            Template("await Bun.file($1).text()"), true),
        rule("api/json-parse-readFileSync",
            r#"\bJSON\.parse\s*\(\s*fs\.readFileSync\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)\s*\)"#,
            "remplacer JSON.parse(fs.readFileSync(path,'utf8')) par await Bun.file(path).json()",
            Template("await Bun.file($1).json()"), true),
        rule("api/fs-existsSync",
            r"\bfs\.existsSync\s*\(\s*([^)]+?)\s*\)",
            "remplacer fs.existsSync(path) par await Bun.file(path).exists() (uniquement pour un fichier — pour un dossier, Bun.file().exists() retourne toujours false)",
            Template("await Bun.file($1).exists()"), true),
        rule("api/http-createServer",
            r"\bhttp\.createServer\s*\(",
            "envisager Bun.serve() plutôt que http.createServer (API fetch-based, plus simple)",
            None, false),
        rule("api/https-createServer",
            r"\bhttps\.createServer\s*\(",
            "envisager Bun.serve({ tls }) plutôt que https.createServer",
            None, false),
        rule("api/execSync",
            r"\b(?:child_process\.)?execSync\s*\(",
            "utiliser le shell Bun ($`cmd`) ou Bun.spawnSync() à la place de execSync",
            None, false),
        rule("api/exec",
            r"\b(?:child_process\.)?exec\s*\(",
            "utiliser le shell Bun ($`cmd`) ou Bun.spawn() à la place de exec",
            None, false),
        rule("api/buffer-alloc",
            r"\bBuffer\.alloc\s*\(\s*([^)]+?)\s*\)",
            "remplacer Buffer.alloc(n) par new Uint8Array(n) (Web-standard)",
            Template("new Uint8Array($1)"), true),
        rule("api/buffer-concat",
            r"\bBuffer\.concat\s*\(",
            "utiliser Uint8Array et concaténation Web-standard plutôt que Buffer.concat",
            None, false),
        rule("api/buffer-from-string",
            r#"\bBuffer\.from\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)"#,
            "remplacer Buffer.from(str, 'utf8') par new TextEncoder().encode(str)",
            Template("new TextEncoder().encode($1)"), true),
        rule("api/process-stdout-write",
            r"\bprocess\.stdout\.write\s*\(",
            "Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write",
            None, false),
        rule("api/process-stderr-write",
            r"\bprocess\.stderr\.write\s*\(",
            "Bun.stderr.write() est l'équivalent natif Bun de process.stderr.write",
            None, false),
        rule("api/sleep-promise",
            r"\bnew\s+Promise\s*\(\s*(?:resolve|res)\s*=>\s*setTimeout\s*\(\s*(?:resolve|res)\s*,\s*([^)]+?)\s*\)\s*\)",
            "remplacer new Promise(res => setTimeout(res, ms)) par Bun.sleep(ms)",
            Template("Bun.sleep($1)"), true),
        rule("api/util-promisify",
            r"\butil\.promisify\s*\(",
            "préférer les APIs async natives de Bun/Node plutôt que util.promisify",
            None, false),
        rule("api/util-inspect",
            r"\butil\.inspect\s*\(",
            "Bun.inspect() est l'équivalent natif (pretty-print avec couleurs)",
            Static("Bun.inspect("), true),
        info_rule("api/new-url-import-meta",
            r#"\bnew\s+URL\s*\(\s*['"][^'"]+['"]\s*,\s*import\.meta\.url\s*\)"#,
            "utiliser import.meta.dir ou path.join(import.meta.dir, ...) plutôt que new URL(..., import.meta.url)",
            None, false),
        rule("api/toml-parse",
            r"\b(?:TOML|toml)\.parse\s*\(",
            "Bun.TOML.parse() est disponible nativement — supprimer la dépendance TOML externe",
            None, false),
        rule("api/semver",
            r"\b(?:semver\.satisfies|semver\.valid|semver\.gt|semver\.lt|semver\.gte|semver\.lte|semver\.coerce)\s*\(",
            "Bun.semver.satisfies() et autres helpers sont disponibles nativement",
            None, false),
        info_rule("api/performance-now",
            r"\bperformance\.now\s*\(\s*\)",
            "Bun.nanoseconds() offre une horloge haute précision (retourne nanosecondes depuis démarrage)",
            None, false),

        // --- Nouvelles règles issues de l'analyse sur discord.js/nextjs/prisma ---
        rule("api/buffer-byteLength",
            r#"\bBuffer\.byteLength\s*\(\s*([^,)]+?)\s*(?:,\s*['"]utf-?8['"]\s*)?\)"#,
            "remplacer Buffer.byteLength(str) par new TextEncoder().encode(str).length (Web-standard)",
            Template("new TextEncoder().encode($1).length"), true),
        rule("api/require-resolve",
            r"\brequire\.resolve\s*\(",
            "Bun.resolveSync() remplace require.resolve() (ESM + CJS, plus rapide)",
            None, false),
        rule("api/set-immediate",
            r"\bsetImmediate\s*\(",
            "setImmediate n'est pas Web-standard — utiliser queueMicrotask() ou setTimeout(fn, 0)",
            None, false),
        info_rule("api/os-platform",
            r"\bos\.platform\s*\(\s*\)",
            "process.platform (global) retourne la même chose sans import 'os'",
            None, false),
        info_rule("api/os-homedir",
            r"\bos\.homedir\s*\(\s*\)",
            "Bun expose Bun.env.HOME / process.env.HOME — évite l'import 'os'",
            None, false),
        rule("api/path-join-dirname",
            r"\bpath\.join\s*\(\s*__dirname\s*,",
            "dans un ESM Bun, path.join(import.meta.dir, ...) évite __dirname",
            None, false),
    ]
});

pub fn apply_bun_api_rules(path: &str, source: &str, aggressive: bool) -> (Vec<Finding>, String) {
    let offsets = line_offsets(source);
    let mut findings: Vec<Finding> = Vec::new();
    struct Edit { index: usize, len: usize, replacement: String }
    let mut edits: Vec<Edit> = Vec::new();

    for r in RULES.iter() {
        for mat in r.re.captures_iter(source) {
            let whole = mat.get(0).unwrap();
            let index = whole.start();
            let original = whole.as_str().to_string();
            let replacement = match &r.replace {
                ReplaceKind::None => Option::<String>::None,
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
                original.clone(),
                if skip_autofix { None } else { replacement.clone() },
                MakeFindingOpts {
                    autofix: Some(has_repl),
                    aggressive: if r.aggressive { Some(true) } else { None },
                    severity: Some(r.severity),
                },
            ));
            if aggressive && r.aggressive && !skip_autofix {
                if let Some(repl) = replacement {
                    edits.push(Edit { index, len: original.len(), replacement: repl });
                }
            }
        }
    }

    let mut out = source.to_string();
    if !edits.is_empty() {
        edits.sort_by(|a, b| b.index.cmp(&a.index));
        for e in edits {
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
