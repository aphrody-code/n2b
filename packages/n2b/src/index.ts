import { basename, extname, join } from "node:path";
import { Glob } from "bun";
import { RIVAL_LOCKFILES, checkLockfile } from "./scanners/lockfile";
import { scanPackageJson } from "./scanners/package-json";
import { scanShell } from "./scanners/shell";
import { scanSource } from "./scanners/source";
import { scanWorkflow } from "./scanners/workflows";
import type { FileFix, RunOptions } from "./types";

const SOURCE_EXTS = new Set([".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".mts", ".cts"]);
const SHELL_EXTS = new Set([".sh", ".bash", ".zsh"]);
const SHELL_NAMES = new Set(["Dockerfile", "Makefile", "Justfile"]);

const DEFAULT_IGNORE = [
  "**/node_modules/**",
  "**/.git/**",
  "**/dist/**",
  "**/build/**",
  "**/out/**",
  "**/.next/**",
  "**/.turbo/**",
  "**/coverage/**",
  "**/.bun/**",
  "**/target/**",
];

export async function run(opts: RunOptions): Promise<{ fixes: FileFix[] }> {
  const glob = new Glob("**/*");
  const ignorePatterns = [...DEFAULT_IGNORE, ...opts.ignore];
  const ignoreMatchers = ignorePatterns.map((p) => new Glob(p));

  // Collect candidate paths first, then process in parallel batches.
  const candidates: string[] = [];
  for await (const rel of glob.scan({ cwd: opts.root, dot: true, onlyFiles: true })) {
    if (ignoreMatchers.some((g) => g.match(rel))) continue;
    candidates.push(rel);
  }

  // Parallel file I/O; batch to avoid fd exhaustion on very large trees.
  const BATCH = 64;
  const fixes: FileFix[] = [];
  for (let i = 0; i < candidates.length; i += BATCH) {
    const slice = candidates.slice(i, i + BATCH);
    const results = await Promise.all(slice.map((rel) => processFile(rel, opts)));
    for (const r of results) if (r) fixes.push(r);
  }

  return { fixes };
}

async function processFile(rel: string, opts: RunOptions): Promise<FileFix | null> {
  const abs = join(opts.root, rel);
  const name = basename(abs);
  const ext = extname(abs).toLowerCase();

  if (RIVAL_LOCKFILES.includes(name)) {
    const f = checkLockfile(rel, name);
    return f ? { file: rel, before: "", after: "", findings: [f] } : null;
  }

  const isWorkflow = /\.github\/workflows\/.+\.ya?ml$/.test(rel.replace(/\\/g, "/"));
  const isPkgJson = name === "package.json";
  const isSource = SOURCE_EXTS.has(ext);
  const isShell = SHELL_EXTS.has(ext) || SHELL_NAMES.has(name);

  if (!isPkgJson && !isSource && !isShell && !isWorkflow) return null;

  const file = Bun.file(abs);
  if (file.size > 2 * 1024 * 1024) return null;
  const before = await file.text();

  let findings: FileFix["findings"] = [];
  let after = before;

  try {
    if (isPkgJson) ({ findings, content: after } = scanPackageJson(rel, before));
    else if (isWorkflow) ({ findings, content: after } = scanWorkflow(rel, before));
    else if (isSource) ({ findings, content: after } = scanSource(rel, before, opts));
    else if (isShell) ({ findings, content: after } = scanShell(rel, before));
  } catch (err) {
    return {
      file: rel,
      before,
      after: before,
      findings: [
        {
          file: rel,
          line: 1,
          col: 1,
          ruleId: "internal/error",
          severity: "error",
          message: `erreur interne : ${(err as Error).message}`,
          original: "",
          autofix: false,
        },
      ],
    };
  }

  if (findings.length === 0 && before === after) return null;

  if (opts.mode !== "check" && after !== before) {
    await Bun.write(abs, after);
  }
  return { file: rel, before, after, findings };
}

export type { RunOptions, FileFix };
export { node2bunPlugin } from "./plugin";
export type { Node2BunPluginOptions } from "./plugin";
