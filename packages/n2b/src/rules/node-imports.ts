import type { Finding } from "../types";
import { makeFinding } from "../util";

/** Modules built-in Node pouvant être préfixés par `node:`. */
const BUILTINS = new Set([
  "assert",
  "async_hooks",
  "buffer",
  "child_process",
  "cluster",
  "console",
  "crypto",
  "dgram",
  "diagnostics_channel",
  "dns",
  "domain",
  "events",
  "fs",
  "http",
  "http2",
  "https",
  "inspector",
  "module",
  "net",
  "os",
  "path",
  "perf_hooks",
  "process",
  "punycode",
  "querystring",
  "readline",
  "repl",
  "stream",
  "string_decoder",
  "sys",
  "timers",
  "tls",
  "trace_events",
  "tty",
  "url",
  "util",
  "v8",
  "vm",
  "wasi",
  "worker_threads",
  "zlib",
]);

/** Packages npm avec équivalent natif Bun. */
const BUN_REPLACEMENTS: Record<
  string,
  { replacement: string; note: string; aggressive?: boolean }
> = {
  sqlite3: {
    replacement: "bun:sqlite",
    note: "préférer bun:sqlite (natif, synchrone, rapide)",
    aggressive: true,
  },
  "better-sqlite3": {
    replacement: "bun:sqlite",
    note: "préférer bun:sqlite (API similaire)",
    aggressive: true,
  },
  "node-fetch": {
    replacement: "<global fetch>",
    note: "fetch est global dans Bun — supprimer l'import",
    aggressive: true,
  },
  "isomorphic-fetch": {
    replacement: "<global fetch>",
    note: "fetch est global dans Bun — supprimer l'import",
    aggressive: true,
  },
  "cross-fetch": {
    replacement: "<global fetch>",
    note: "fetch est global dans Bun — supprimer l'import",
    aggressive: true,
  },
  dotenv: {
    replacement: "<auto>",
    note: "Bun charge .env automatiquement, dotenv inutile",
    aggressive: true,
  },
  "dotenv/config": {
    replacement: "<auto>",
    note: "Bun charge .env automatiquement, dotenv inutile",
    aggressive: true,
  },
  pg: { replacement: "Bun.sql", note: "Bun.sql est un client PostgreSQL natif", aggressive: true },
  postgres: {
    replacement: "Bun.sql",
    note: "Bun.sql est un client PostgreSQL natif",
    aggressive: true,
  },
  ioredis: {
    replacement: "Bun.redis",
    note: "Bun.redis est un client Redis/Valkey natif",
    aggressive: true,
  },
  redis: {
    replacement: "Bun.redis",
    note: "Bun.redis est un client Redis/Valkey natif",
    aggressive: true,
  },
  ws: {
    replacement: "WebSocket",
    note: "WebSocket client/serveur est natif dans Bun",
    aggressive: true,
  },
  uuid: {
    replacement: "Bun.randomUUIDv7",
    note: "utiliser Bun.randomUUIDv7() ou crypto.randomUUID()",
    aggressive: true,
  },
  minimist: { replacement: "util.parseArgs", note: "Node fournit util.parseArgs" },
  chalk: { replacement: "<natif>", note: "Bun supporte les codes ANSI nativement" },
  rimraf: { replacement: "fs.rm", note: "fs.rm({ recursive: true }) suffit" },
  mkdirp: { replacement: "fs.mkdir", note: "fs.mkdir({ recursive: true }) suffit" },
  "node-cron": { replacement: "Bun.cron", note: "Bun.cron (API native)", aggressive: true },

  // Clients HTTP
  axios: {
    replacement: "<global fetch>",
    note: "fetch est global dans Bun — pas besoin d'axios",
    aggressive: true,
  },
  got: {
    replacement: "<global fetch>",
    note: "fetch est global dans Bun — pas besoin de got",
    aggressive: true,
  },
  "node-got": {
    replacement: "<global fetch>",
    note: "fetch est global dans Bun — pas besoin de node-got",
    aggressive: true,
  },
  superagent: {
    replacement: "<global fetch>",
    note: "fetch est global dans Bun — pas besoin de superagent",
    aggressive: true,
  },
  undici: {
    replacement: "<global fetch>",
    note: "fetch/WebSocket sont natifs dans Bun — undici inutile",
    aggressive: true,
  },
  "make-fetch-happen": {
    replacement: "<global fetch>",
    note: "fetch est global dans Bun",
    aggressive: true,
  },
  ky: { replacement: "<global fetch>", note: "fetch est global dans Bun — ky est inutile" },

  // Auth / Crypto
  bcrypt: {
    replacement: "Bun.password",
    note: "Bun.password.hash/verify supporte bcrypt et argon2",
    aggressive: true,
  },
  bcryptjs: {
    replacement: "Bun.password",
    note: "Bun.password.hash/verify supporte bcrypt et argon2",
    aggressive: true,
  },
  argon2: {
    replacement: "Bun.password",
    note: "Bun.password.hash({ algorithm: 'argon2id' })",
    aggressive: true,
  },

  // Globals Web (dispo nativement dans Bun)
  "form-data": {
    replacement: "FormData",
    note: "FormData est global dans Bun — supprimer l'import",
    aggressive: true,
  },
  "abort-controller": {
    replacement: "AbortController",
    note: "AbortController est global dans Bun — supprimer l'import",
    aggressive: true,
  },
  "whatwg-url": {
    replacement: "URL",
    note: "URL/URLSearchParams sont globaux dans Bun",
    aggressive: true,
  },
  "node-blob": { replacement: "Blob", note: "Blob est global dans Bun", aggressive: true },
  "web-streams-polyfill": {
    replacement: "<global>",
    note: "ReadableStream/WritableStream sont globaux dans Bun",
    aggressive: true,
  },

  // Test runners
  jest: {
    replacement: "bun:test",
    note: "bun:test est compatible Jest (describe/test/expect/mock)",
    aggressive: true,
  },
  mocha: {
    replacement: "bun:test",
    note: "bun:test remplace mocha avec une API similaire",
    aggressive: true,
  },
  vitest: {
    replacement: "bun:test",
    note: "bun:test offre les mêmes fonctionnalités que vitest",
    aggressive: true,
  },
  chai: { replacement: "bun:test", note: "bun:test inclut expect() avec assertions Jest/Chai" },
  "@jest/globals": {
    replacement: "bun:test",
    note: "importer depuis bun:test à la place",
    aggressive: true,
  },
  "ts-jest": {
    replacement: "bun:test",
    note: "bun:test supporte TypeScript nativement",
    aggressive: true,
  },
  "jest-circus": {
    replacement: "bun:test",
    note: "bun:test est le runner natif Bun",
    aggressive: true,
  },

  // Runners / transpilers remplacés par Bun
  "ts-node": {
    replacement: "bun run",
    note: "bun exécute TypeScript nativement sans ts-node",
    aggressive: true,
  },
  tsx: {
    replacement: "bun run",
    note: "bun exécute TSX nativement — tsx CLI inutile",
    aggressive: true,
  },
  esm: { replacement: "<natif>", note: "Bun supporte ESM nativement", aggressive: true },
  nodemon: {
    replacement: "bun --watch",
    note: "bun --watch ou bun --hot remplacent nodemon",
    aggressive: true,
  },
  concurrently: {
    replacement: "<natif>",
    note: "Bun.spawn permet de lancer des processus en parallèle",
  },

  // Types
  "@types/node": {
    replacement: "@types/bun",
    note: "utiliser @types/bun pour les types Bun (ou bun-types)",
  },

  // TOML
  "@iarna/toml": { replacement: "Bun.TOML", note: "Bun.TOML.parse() est natif", aggressive: true },
  toml: { replacement: "Bun.TOML", note: "Bun.TOML.parse() est natif", aggressive: true },
  "smol-toml": { replacement: "Bun.TOML", note: "Bun.TOML.parse() est natif", aggressive: true },

  // DNS
  "dns-packet": { replacement: "bun dns", note: "Bun expose dns.lookup/dns.resolve nativement" },

  // Divers
  mime: { replacement: "<natif>", note: "Bun.file().type retourne le MIME type automatiquement" },
  "mime-types": {
    replacement: "<natif>",
    note: "Bun.file().type retourne le MIME type automatiquement",
  },
  glob: {
    replacement: "bun (Glob)",
    note: "import { Glob } from 'bun' est natif (pas bun:glob)",
    aggressive: true,
  },
  "fast-glob": {
    replacement: "bun (Glob)",
    note: "import { Glob } from 'bun' est natif (pas bun:glob)",
    aggressive: true,
  },
  "glob-parent": { replacement: "bun (Glob)", note: "utiliser import { Glob } from 'bun'" },
};

type Loader = "js" | "jsx" | "ts" | "tsx";

export function loaderFromPath(path: string): Loader {
  if (path.endsWith(".tsx")) return "tsx";
  if (path.endsWith(".jsx")) return "jsx";
  if (path.endsWith(".ts") || path.endsWith(".mts") || path.endsWith(".cts")) return "ts";
  return "js";
}

/** Cache un transpiler par loader (évite de réallouer à chaque fichier). */
const transpilerCache = new Map<Loader, Bun.Transpiler>();
function getTranspiler(loader: Loader): Bun.Transpiler {
  let t = transpilerCache.get(loader);
  if (!t) {
    t = new Bun.Transpiler({ loader });
    transpilerCache.set(loader, t);
  }
  return t;
}

/**
 * Analyse les imports via `Bun.Transpiler.scanImports()` — natif, ignore commentaires
 * et strings, distingue type-only imports (qui sont éliminés).
 */
export function applyNodeImportRules(
  path: string,
  source: string,
  aggressive: boolean,
): { findings: Finding[]; content: string } {
  const findings: Finding[] = [];
  const edits: { index: number; len: number; replacement: string }[] = [];

  // Le transpiler ne reconnaît pas les shebangs : on le masque en commentaire
  // de même longueur pour préserver les offsets exacts dans la source originale.
  const forParse = source.startsWith("#!")
    ? `//${source.slice(2, source.indexOf("\n") === -1 ? source.length : source.indexOf("\n"))}${source.slice(source.indexOf("\n") === -1 ? source.length : source.indexOf("\n"))}`
    : source;

  let imports: { kind: string; path: string }[];
  try {
    imports = getTranspiler(loaderFromPath(path)).scanImports(forParse);
  } catch {
    return { findings, content: source };
  }

  // Pour chaque specifier, on localise sa position dans la source via une recherche
  // ciblée sur les quotes entourant le path (scanImports n'expose pas d'offsets).
  const seen = new Set<string>();
  const specDone = new Set<string>(); // dedupe per-specifier (scanImports duplique par occurrence)
  for (const imp of imports) {
    if (imp.kind === "import-rule" || imp.kind === "url-token") continue;
    const spec = imp.path;
    const key = `${spec}:${imp.kind}`;

    if (BUILTINS.has(spec)) {
      if (specDone.has(spec)) continue;
      specDone.add(spec);
      for (const pos of findSpecifierPositions(source, spec)) {
        edits.push({ index: pos, len: spec.length, replacement: `node:${spec}` });
        findings.push(
          makeFinding(
            path,
            source,
            pos,
            "imports/node-prefix",
            `préfixer '${spec}' avec 'node:' (recommandé)`,
            spec,
            `node:${spec}`,
            { autofix: true },
          ),
        );
      }
      continue;
    }

    // Suggestion de remplacement Bun.
    const repl = BUN_REPLACEMENTS[spec];
    if (repl && !seen.has(key)) {
      seen.add(key);
      const pos = firstSpecifierPosition(source, spec) ?? 0;
      findings.push(
        makeFinding(
          path,
          source,
          pos,
          "imports/bun-native",
          `remplacer '${spec}' par ${repl.replacement} — ${repl.note}`,
          spec,
          repl.replacement,
          { autofix: false, aggressive: true },
        ),
      );
      if (aggressive && /^(bun:|node:)/.test(repl.replacement)) {
        for (const p of findSpecifierPositions(source, spec)) {
          edits.push({ index: p, len: spec.length, replacement: repl.replacement });
        }
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

/** Localise toutes les occurrences d'un specifier entre quotes dans la source. */
function findSpecifierPositions(source: string, spec: string): number[] {
  const out: number[] = [];
  for (const q of ['"', "'"]) {
    const needle = q + spec + q;
    let idx = 0;
    while ((idx = source.indexOf(needle, idx)) !== -1) {
      out.push(idx + 1); // position du premier caractère du specifier (après le quote)
      idx += needle.length;
    }
  }
  return out;
}

function firstSpecifierPosition(source: string, spec: string): number | undefined {
  const positions = findSpecifierPositions(source, spec);
  return positions.length ? positions[0] : undefined;
}
