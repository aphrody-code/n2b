import type { Finding } from "../types";
import { makeFinding } from "../util";

/**
 * API-level suggestions: flag Node idioms that have a more idiomatic Bun form.
 * These are "advisory" by default — emitted as warnings. If the user passes
 * `--aggressive` and the pattern has a safe rewrite, we apply it.
 */
interface ApiRule {
  id: string;
  pattern: RegExp;
  message: string;
  replace?: (m: RegExpMatchArray) => string;
  aggressive?: boolean;
}

const RULES: ApiRule[] = [
  {
    id: "api/fs-readFileSync",
    pattern: /\bfs\.readFileSync\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)/g,
    message: "remplacer fs.readFileSync(path, 'utf8') par await Bun.file(path).text()",
    replace: (m) => `await Bun.file(${m[1]}).text()`,
    aggressive: true,
  },
  {
    id: "api/fs-writeFileSync",
    pattern: /\bfs\.writeFileSync\s*\(\s*([^,]+?)\s*,\s*([^)]+?)\s*\)/g,
    message: "remplacer fs.writeFileSync(path, data) par await Bun.write(path, data)",
    replace: (m) => `await Bun.write(${m[1]}, ${m[2]})`,
    aggressive: true,
  },
  {
    id: "api/process-env",
    pattern: /\bprocess\.env\.([A-Z_][A-Z0-9_]*)\b/g,
    message: "Bun.env est un alias plus court de process.env (préférence stylistique)",
    // non-autofix: too stylistic
  },
  {
    id: "api/dirname-esm",
    pattern:
      /\b(?:const|let|var)\s+__dirname\s*=\s*(?:path\.)?dirname\s*\(\s*fileURLToPath\s*\(\s*import\.meta\.url\s*\)\s*\)/g,
    message: "dans un ESM Bun, utiliser directement import.meta.dir (ou import.meta.dirname)",
    replace: () => "const __dirname = import.meta.dir",
    aggressive: true,
  },
  {
    id: "api/filename-esm",
    pattern: /\b(?:const|let|var)\s+__filename\s*=\s*fileURLToPath\s*\(\s*import\.meta\.url\s*\)/g,
    message: "dans un ESM Bun, utiliser import.meta.path (ou import.meta.filename)",
    replace: () => "const __filename = import.meta.path",
    aggressive: true,
  },
  {
    id: "api/express-server",
    pattern: /\b(?:const|let|var)\s+(\w+)\s*=\s*require\(\s*['"]express['"]\s*\)\s*\(\s*\)/g,
    message: "envisager Bun.serve() plutôt qu'Express pour un serveur simple (voir runtime/http)",
  },
  {
    id: "api/child-process-spawn",
    pattern: /\b(?:child_process\.)?spawn\s*\(/g,
    message: "Bun.spawn offre une API plus ergonomique (streams Web, ipc, preload)",
  },
  {
    id: "api/crypto-createHash",
    pattern: /\bcrypto\.createHash\s*\(\s*['"](?:md5|sha1|sha256|sha512|blake2b256)['"]\s*\)/g,
    message: "Bun.hash / Bun.CryptoHasher est plus rapide (voir runtime/hashing)",
  },
  {
    id: "api/buffer-from-base64",
    pattern: /\bBuffer\.from\s*\(\s*([^,]+?)\s*,\s*['"]base64['"]\s*\)/g,
    message: "utiliser atob() / btoa() ou Uint8Array pour du Web-standard",
  },
  {
    id: "api/fileURLToPath",
    pattern: /\bfileURLToPath\s*\(/g,
    message: "Bun.fileURLToPath() est équivalent (ou utiliser import.meta.dir/path)",
  },
  {
    id: "api/uuid-v4",
    pattern: /\b(?:uuidv4|v4)\s*\(\s*\)/g,
    message: "crypto.randomUUID() (global) ou Bun.randomUUIDv7() évite la dépendance uuid",
  },

  // --- Fichiers (async) ---
  {
    id: "api/fs-readFile-utf8",
    pattern: /\bfs\.readFile\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*,\s*([^)]+?)\s*\)/g,
    message: "remplacer fs.readFile(path, 'utf8', cb) par await Bun.file(path).text()",
  },
  {
    id: "api/fs-readFile-promise",
    pattern: /\bfsPromises\.readFile\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)/g,
    message: "remplacer fsPromises.readFile(path, 'utf8') par await Bun.file(path).text()",
    replace: (m) => `await Bun.file(${m[1]}).text()`,
    aggressive: true,
  },
  {
    id: "api/json-parse-readFileSync",
    pattern:
      /\bJSON\.parse\s*\(\s*fs\.readFileSync\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)\s*\)/g,
    message: "remplacer JSON.parse(fs.readFileSync(path,'utf8')) par await Bun.file(path).json()",
    replace: (m) => `await Bun.file(${m[1]}).json()`,
    aggressive: true,
  },
  {
    id: "api/fs-existsSync",
    pattern: /\bfs\.existsSync\s*\(\s*([^)]+?)\s*\)/g,
    message: "remplacer fs.existsSync(path) par await Bun.file(path).exists()",
    replace: (m) => `await Bun.file(${m[1]}).exists()`,
    aggressive: true,
  },

  // --- Serveur HTTP ---
  {
    id: "api/http-createServer",
    pattern: /\bhttp\.createServer\s*\(/g,
    message: "envisager Bun.serve() plutôt que http.createServer (API fetch-based, plus simple)",
  },
  {
    id: "api/https-createServer",
    pattern: /\bhttps\.createServer\s*\(/g,
    message: "envisager Bun.serve({ tls }) plutôt que https.createServer",
  },

  // --- Shell / Processus ---
  {
    id: "api/execSync",
    pattern: /\b(?:child_process\.)?execSync\s*\(/g,
    message: "utiliser le shell Bun ($`cmd`) ou Bun.spawnSync() à la place de execSync",
  },
  {
    id: "api/exec",
    pattern: /\b(?:child_process\.)?exec\s*\(/g,
    message: "utiliser le shell Bun ($`cmd`) ou Bun.spawn() à la place de exec",
  },

  // --- Buffer → Uint8Array ---
  {
    id: "api/buffer-alloc",
    pattern: /\bBuffer\.alloc\s*\(\s*([^)]+?)\s*\)/g,
    message: "remplacer Buffer.alloc(n) par new Uint8Array(n) (Web-standard)",
    replace: (m) => `new Uint8Array(${m[1]})`,
    aggressive: true,
  },
  {
    id: "api/buffer-concat",
    pattern: /\bBuffer\.concat\s*\(/g,
    message: "utiliser Uint8Array et concaténation Web-standard plutôt que Buffer.concat",
  },
  {
    id: "api/buffer-from-string",
    pattern: /\bBuffer\.from\s*\(\s*([^,)]+?)\s*,\s*['"]utf-?8['"]\s*\)/g,
    message: "remplacer Buffer.from(str, 'utf8') par new TextEncoder().encode(str)",
    replace: (m) => `new TextEncoder().encode(${m[1]})`,
    aggressive: true,
  },

  // --- stdout / stderr ---
  {
    id: "api/process-stdout-write",
    pattern: /\bprocess\.stdout\.write\s*\(/g,
    message: "Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write",
  },
  {
    id: "api/process-stderr-write",
    pattern: /\bprocess\.stderr\.write\s*\(/g,
    message: "Bun.stderr.write() est l'équivalent natif Bun de process.stderr.write",
  },

  // --- Timers ---
  {
    id: "api/sleep-promise",
    pattern:
      /\bnew\s+Promise\s*\(\s*(?:resolve|res)\s*=>\s*setTimeout\s*\(\s*(?:resolve|res)\s*,\s*([^)]+?)\s*\)\s*\)/g,
    message: "remplacer new Promise(res => setTimeout(res, ms)) par Bun.sleep(ms)",
    replace: (m) => `Bun.sleep(${m[1]})`,
    aggressive: true,
  },

  // --- util ---
  {
    id: "api/util-promisify",
    pattern: /\butil\.promisify\s*\(/g,
    message: "préférer les APIs async natives de Bun/Node plutôt que util.promisify",
  },
  {
    id: "api/util-inspect",
    pattern: /\butil\.inspect\s*\(/g,
    message: "Bun.inspect() est l'équivalent natif (pretty-print avec couleurs)",
    replace: () => "Bun.inspect(",
    aggressive: true,
  },

  // --- ESM __dirname patterns ---
  {
    id: "api/new-url-import-meta",
    pattern: /\bnew\s+URL\s*\(\s*['"][^'"]+['"]\s*,\s*import\.meta\.url\s*\)/g,
    message:
      "utiliser import.meta.dir ou path.join(import.meta.dir, ...) plutôt que new URL(..., import.meta.url)",
  },

  // --- TOML ---
  {
    id: "api/toml-parse",
    pattern: /\b(?:TOML|toml)\.parse\s*\(/g,
    message: "Bun.TOML.parse() est disponible nativement — supprimer la dépendance TOML externe",
  },

  // --- Semver ---
  {
    id: "api/semver",
    pattern:
      /\b(?:semver\.satisfies|semver\.valid|semver\.gt|semver\.lt|semver\.gte|semver\.lte|semver\.coerce)\s*\(/g,
    message: "Bun.semver.satisfies() et autres helpers sont disponibles nativement",
  },

  // --- Performance ---
  {
    id: "api/performance-now",
    pattern: /\bperformance\.now\s*\(\s*\)/g,
    message:
      "Bun.nanoseconds() offre une horloge haute précision (retourne nanosecondes depuis démarrage)",
  },
];

export function applyBunApiRules(
  path: string,
  source: string,
  aggressive: boolean,
): { findings: Finding[]; content: string } {
  const findings: Finding[] = [];
  type Edit = { index: number; len: number; replacement: string };
  const edits: Edit[] = [];

  for (const r of RULES) {
    r.pattern.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = r.pattern.exec(source)) !== null) {
      const original = m[0];
      const replacement = r.replace ? r.replace(m) : undefined;
      findings.push(
        makeFinding(path, source, m.index, r.id, r.message, original, replacement, {
          autofix: !!replacement,
          aggressive: r.aggressive,
        }),
      );
      if (aggressive && r.aggressive && replacement) {
        edits.push({ index: m.index, len: original.length, replacement });
      }
    }
  }

  let out = source;
  if (edits.length) {
    edits.sort((a, b) => b.index - a.index);
    for (const e of edits) out = out.slice(0, e.index) + e.replacement + out.slice(e.index + e.len);
  }
  return { findings, content: out };
}
