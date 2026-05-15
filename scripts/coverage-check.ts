#!/usr/bin/env bun
/**
 * bun-agent coverage audit
 *
 * Vérifie que le plugin (agents/skills/commands) et n2b (rules) couvrent
 * toutes les APIs Bun documentées dans `docs/bun-official/`.
 *
 * Usage :
 *   bun scripts/coverage-check.ts              # run once, print summary + write report
 *   bun scripts/coverage-check.ts --json       # output structured JSON
 *   bun scripts/coverage-check.ts --strict     # exit 1 si coverage < threshold
 *
 * Env vars :
 *   CLAUDE_PLUGIN_ROOT    racine du plugin (default: derived from this script path)
 *   COVERAGE_THRESHOLD    % minimum pour --strict (default: 80)
 */

import { Glob } from "bun";
import { join, dirname, relative, resolve } from "node:path";

const SCRIPT_DIR = dirname(import.meta.path);
const PLUGIN_ROOT = process.env.CLAUDE_PLUGIN_ROOT ?? resolve(SCRIPT_DIR, "..");
const DOCS_BUN = join(PLUGIN_ROOT, "docs/bun-official");
const DOCS_N2B = join(PLUGIN_ROOT, "docs/n2b");
const COVERAGE_DIR = join(PLUGIN_ROOT, "docs/coverage");
const THRESHOLD = Number(process.env.COVERAGE_THRESHOLD ?? "80");

const args = new Set(process.argv.slice(2));
const FORMAT_JSON = args.has("--json");
const STRICT = args.has("--strict");

// ──────────────────────────────────────────────────────────────────
// 1. Extract Bun APIs from official docs
// ──────────────────────────────────────────────────────────────────

const API_PATTERNS = [
  /\bBun\.([A-Za-z][A-Za-z0-9]*)/g,           // Bun.serve, Bun.file, Bun.spawn, ...
  /\bbun:([a-z][a-z0-9-]*)/g,                 // bun:sqlite, bun:ffi, bun:test, ...
  /\bimport\s*[{\s,\w]*\s*from\s*['"]bun['"]/g, // import { Glob } from "bun"
];

type ApiRef = {
  api: string;              // ex. "Bun.serve" | "bun:sqlite"
  category: "Bun.*" | "bun:*" | "import";
  sources: Set<string>;     // files where found
};

const apiMap = new Map<string, ApiRef>();

function recordApi(api: string, category: ApiRef["category"], source: string) {
  const existing = apiMap.get(api);
  if (existing) existing.sources.add(source);
  else apiMap.set(api, { api, category, sources: new Set([source]) });
}

async function scanDocs(): Promise<number> {
  const glob = new Glob("**/*.{md,mdx}");
  let fileCount = 0;

  for await (const rel of glob.scan({ cwd: DOCS_BUN })) {
    fileCount++;
    const abs = join(DOCS_BUN, rel);
    const text = await Bun.file(abs).text();

    // Bun.foo
    for (const m of text.matchAll(API_PATTERNS[0]!)) {
      const name = `Bun.${m[1]}`;
      // Filter out common false positives (Bun. as prose)
      if (m[1] && m[1].length > 1) recordApi(name, "Bun.*", rel);
    }

    // bun:foo
    for (const m of text.matchAll(API_PATTERNS[1]!)) {
      recordApi(`bun:${m[1]}`, "bun:*", rel);
    }

    // import from "bun"
    if (API_PATTERNS[2]!.test(text)) {
      recordApi('from "bun"', "import", rel);
    }
  }

  return fileCount;
}

// ──────────────────────────────────────────────────────────────────
// 2. Load n2b rule catalogue (via CLI if available, else doc scan)
// ──────────────────────────────────────────────────────────────────

type N2bRule = { id: string; replacement: string | null; aggressive: boolean };

async function loadN2bRules(): Promise<N2bRule[]> {
  const n2b = Bun.which("n2b");
  if (n2b) {
    try {
      const out = await Bun.$`${n2b} rules --report json`.quiet().text();
      const parsed = JSON.parse(out);
      if (Array.isArray(parsed)) {
        return parsed.map((r: any) => ({
          id: r.id ?? r.rule_id ?? "",
          replacement: r.replacement ?? r.suggestion ?? null,
          aggressive: Boolean(r.aggressive ?? r.autofix === "aggressive"),
        }));
      }
    } catch (e) {
      // fall through to doc scan
    }
  }
  // Fallback: grep the bundled n2b doc for rule ids
  const rules: N2bRule[] = [];
  const glob = new Glob("**/*.md");
  for await (const rel of glob.scan({ cwd: DOCS_N2B })) {
    const text = await Bun.file(join(DOCS_N2B, rel)).text();
    for (const m of text.matchAll(/`(api|cli|imports|pkg|ci|shebang|tsconfig|husky|lock|workspace)\/([a-z-]+)`/g)) {
      rules.push({ id: `${m[1]}/${m[2]}`, replacement: null, aggressive: false });
    }
  }
  // Dedup
  const seen = new Set<string>();
  return rules.filter((r) => (seen.has(r.id) ? false : (seen.add(r.id), true)));
}

// ──────────────────────────────────────────────────────────────────
// 3. Map n2b rules → Bun APIs they target (heuristic via rule id)
// ──────────────────────────────────────────────────────────────────

// Known mapping rule-id → target Bun API
const RULE_TO_API: Record<string, string> = {
  "api/fs-readFileSync": "Bun.file",
  "api/fs-writeFileSync": "Bun.write",
  "api/fs-readFile-promise": "Bun.file",
  "api/json-parse-readFileSync": "Bun.file",
  "api/fs-existsSync": "Bun.file",
  "api/dirname-esm": "import.meta.dir",
  "api/filename-esm": "import.meta.path",
  "api/sleep-promise": "Bun.sleep",
  "api/util-inspect": "Bun.inspect",
  "api/execSync": "Bun.$",
  "api/exec": "Bun.$",
  "api/child-process-spawn": "Bun.spawn",
  "api/crypto-createHash": "Bun.hash",
  "api/http-createServer": "Bun.serve",
  "api/https-createServer": "Bun.serve",
  "api/uuid-v4": "Bun.randomUUIDv7",
  "api/toml-parse": "Bun.TOML",
  "api/semver": "Bun.semver",
  "api/performance-now": "Bun.nanoseconds",
  "api/require-resolve": "Bun.resolveSync",
  "api/process-stdout-write": "Bun.stdout",
  "api/process-stderr-write": "Bun.stderr",
  "api/process-env": "Bun.env",
  "api/path-join-dirname": "import.meta.dir",
  "api/fileURLToPath": "Bun.fileURLToPath",
};

// ──────────────────────────────────────────────────────────────────
// 4. Scan plugin (.md) for API mentions
// ──────────────────────────────────────────────────────────────────

async function scanPlugin(): Promise<Set<string>> {
  const mentioned = new Set<string>();
  const glob = new Glob("{agents,commands,skills,hooks,output-styles}/**/*.md");
  for await (const rel of glob.scan({ cwd: PLUGIN_ROOT })) {
    const text = await Bun.file(join(PLUGIN_ROOT, rel)).text();
    for (const m of text.matchAll(API_PATTERNS[0]!)) mentioned.add(`Bun.${m[1]}`);
    for (const m of text.matchAll(API_PATTERNS[1]!)) mentioned.add(`bun:${m[1]}`);
    if (API_PATTERNS[2]!.test(text)) mentioned.add('from "bun"');
  }
  return mentioned;
}

// ──────────────────────────────────────────────────────────────────
// 5. Build coverage matrix + report
// ──────────────────────────────────────────────────────────────────

type CoverageRow = {
  api: string;
  category: string;
  in_docs: boolean;
  in_n2b: boolean;
  in_plugin: boolean;
  sources_count: number;
};

async function main() {
  console.log(`🔍 Scanning docs at ${relative(PLUGIN_ROOT, DOCS_BUN)}/ ...`);
  const docFiles = await scanDocs();

  console.log(`📚 Loading n2b rules ...`);
  const rules = await loadN2bRules();
  const n2bTargetApis = new Set(rules.map((r) => RULE_TO_API[r.id]).filter(Boolean) as string[]);

  console.log(`🔎 Scanning plugin mentions ...`);
  const pluginApis = await scanPlugin();

  // Build coverage rows
  const rows: CoverageRow[] = [...apiMap.values()]
    .map((r) => ({
      api: r.api,
      category: r.category,
      in_docs: true,
      in_n2b: n2bTargetApis.has(r.api),
      in_plugin: pluginApis.has(r.api),
      sources_count: r.sources.size,
    }))
    .sort((a, b) => (b.sources_count - a.sources_count) || a.api.localeCompare(b.api));

  const total = rows.length;
  const pluginCov = rows.filter((r) => r.in_plugin).length;
  const n2bCov = rows.filter((r) => r.in_n2b).length;
  const pluginPct = Math.round((pluginCov / total) * 100);
  const n2bPct = Math.round((n2bCov / total) * 100);

  const gaps = rows.filter((r) => !r.in_plugin);
  const criticalGaps = gaps.filter((r) => r.sources_count >= 3); // mentionnée dans 3+ docs

  const summary = {
    timestamp: new Date().toISOString(),
    doc_files_scanned: docFiles,
    n2b_rules_loaded: rules.length,
    unique_apis_in_docs: total,
    plugin_coverage_pct: pluginPct,
    n2b_coverage_pct: n2bPct,
    critical_gaps: criticalGaps.length,
    threshold: THRESHOLD,
    passed: pluginPct >= THRESHOLD,
  };

  // Write report
  await Bun.$`mkdir -p ${COVERAGE_DIR}`.quiet();
  const date = new Date().toISOString().slice(0, 10);
  const reportPath = join(COVERAGE_DIR, `report-${date}.md`);
  const mdReport = renderMarkdown(summary, rows, criticalGaps);
  await Bun.write(reportPath, mdReport);
  await Bun.$`ln -sf ${reportPath} ${join(COVERAGE_DIR, "latest.md")}`.quiet();

  // Output
  if (FORMAT_JSON) {
    console.log(JSON.stringify({ summary, rows }, null, 2));
  } else {
    console.log();
    console.log(`📊 Coverage report — ${summary.timestamp}`);
    console.log(`   Doc files scanned        : ${summary.doc_files_scanned}`);
    console.log(`   n2b rules loaded         : ${summary.n2b_rules_loaded}`);
    console.log(`   Unique Bun APIs in docs  : ${summary.unique_apis_in_docs}`);
    console.log(`   Plugin coverage          : ${pluginCov}/${total} (${pluginPct}%)`);
    console.log(`   n2b rule coverage        : ${n2bCov}/${total} (${n2bPct}%)`);
    console.log(`   Critical gaps (≥3 docs)  : ${criticalGaps.length}`);
    console.log(`   Threshold                : ${THRESHOLD}% ${summary.passed ? "✅" : "❌"}`);
    console.log();
    console.log(`   → ${relative(PLUGIN_ROOT, reportPath)}`);
    if (criticalGaps.length > 0) {
      console.log();
      console.log(`⚠ Critical gaps (top 10) :`);
      for (const g of criticalGaps.slice(0, 10)) {
        console.log(`   ${g.api.padEnd(40)} (${g.sources_count} docs)`);
      }
    }
  }

  if (STRICT && !summary.passed) process.exit(1);
}

function renderMarkdown(summary: any, rows: CoverageRow[], gaps: CoverageRow[]): string {
  const lines: string[] = [];
  lines.push(`# bun-agent coverage report`);
  lines.push("");
  lines.push(`Generated : **${summary.timestamp}**`);
  lines.push("");
  lines.push(`## Summary`);
  lines.push("");
  lines.push(`| Metric | Value |`);
  lines.push(`|---|---|`);
  lines.push(`| Doc files scanned | ${summary.doc_files_scanned} |`);
  lines.push(`| n2b rules loaded | ${summary.n2b_rules_loaded} |`);
  lines.push(`| Unique Bun APIs found | ${summary.unique_apis_in_docs} |`);
  lines.push(`| Plugin coverage | **${summary.plugin_coverage_pct}%** |`);
  lines.push(`| n2b rule coverage | ${summary.n2b_coverage_pct}% |`);
  lines.push(`| Critical gaps (≥3 docs) | ${summary.critical_gaps} |`);
  lines.push(`| Threshold | ${summary.threshold}% — ${summary.passed ? "✅ passed" : "❌ failed"} |`);
  lines.push("");

  if (gaps.length > 0) {
    lines.push(`## Critical gaps — APIs not mentioned in plugin`);
    lines.push("");
    lines.push(`| API | Category | Docs |`);
    lines.push(`|---|---|---|`);
    for (const g of gaps.slice(0, 50)) {
      lines.push(`| \`${g.api}\` | ${g.category} | ${g.sources_count} |`);
    }
    lines.push("");
  }

  lines.push(`## Full matrix`);
  lines.push("");
  lines.push(`| API | Category | Docs | n2b | Plugin |`);
  lines.push(`|---|---|---|---|---|`);
  for (const r of rows) {
    lines.push(`| \`${r.api}\` | ${r.category} | ${r.sources_count} | ${r.in_n2b ? "✓" : ""} | ${r.in_plugin ? "✓" : ""} |`);
  }
  lines.push("");
  return lines.join("\n");
}

await main();
