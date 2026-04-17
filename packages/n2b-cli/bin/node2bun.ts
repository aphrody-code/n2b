#!/usr/bin/env bun
import { resolve } from "node:path";
import { run } from "@n2b/core";
import { renderJson, renderMarkdown, renderText } from "@n2b/core/report";
import type { FileFix, RunOptions } from "@n2b/core/types";

const USAGE = `\
node2bun — analyse un package et corrige les incompatibilités avec Bun.

Usage:
  node2bun [chemin] [options]

Modes:
  (défaut)         dry-run : rapporte les findings, ne modifie rien
  --fix            applique les corrections sûres (CLI, node: prefix, shebang, workflow)
  --aggressive     applique aussi les migrations d'APIs (Node → Bun.*), à vérifier à la main

Rapport:
  --report=text    (défaut) sortie colorée
  --report=json    sortie JSON
  --report=md      sortie Markdown

Divers:
  --ignore=<glob>  ajouter un glob d'exclusion (cumulable)
  --quiet          réduire la sortie
  -h, --help       cette aide
  -v, --version

Exemples:
  node2bun .
  node2bun ./mon-app --fix
  node2bun . --aggressive --report=md > report.md
`;

function parseArgs(argv: string[]): { root: string; opts: RunOptions } {
  let root = ".";
  let mode: RunOptions["mode"] = "check";
  let report: RunOptions["report"] = "text";
  let quiet = false;
  const ignore: string[] = [];

  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]!;
    if (a === "-h" || a === "--help") {
      console.log(USAGE);
      process.exit(0);
    }
    if (a === "-v" || a === "--version") {
      const pkg = require("../package.json");
      console.log(`node2bun ${pkg.version}`);
      process.exit(0);
    }
    if (a === "--fix") {
      mode = "fix";
      continue;
    }
    if (a === "--aggressive") {
      mode = "aggressive";
      continue;
    }
    if (a === "--quiet") {
      quiet = true;
      continue;
    }
    if (a.startsWith("--report=")) {
      const v = a.slice("--report=".length);
      if (v === "text" || v === "json" || v === "md" || v === "markdown") {
        report = v === "md" ? "markdown" : (v as any);
      } else {
        fail(`valeur --report invalide: ${v}`);
      }
      continue;
    }
    if (a.startsWith("--ignore=")) {
      ignore.push(a.slice("--ignore=".length));
      continue;
    }
    if (a.startsWith("-")) fail(`option inconnue: ${a}`);
    root = a;
  }

  return {
    root: resolve(root),
    opts: { root: resolve(root), mode, report, quiet, ignore },
  };
}

function fail(msg: string): never {
  console.error(`node2bun: ${msg}`);
  console.error(`utiliser --help pour l'aide`);
  process.exit(2);
}

async function main() {
  const { opts } = parseArgs(process.argv.slice(2));
  try {
    const { fixes } = await run(opts);
    if (opts.report === "json") {
      console.log(renderJson(fixes, opts));
    } else if (opts.report === "markdown") {
      console.log(renderMarkdown(fixes, opts));
    } else if (!opts.quiet) {
      console.log(renderText(fixes, opts));
    }
    const hasErrors = fixes.some((f: FileFix) => f.findings.some((x) => x.severity === "error"));
    const hasFindings = fixes.some((f: FileFix) => f.findings.length > 0);
    process.exit(hasErrors ? 2 : opts.mode === "check" && hasFindings ? 1 : 0);
  } catch (err) {
    console.error(`node2bun a échoué : ${(err as Error).stack || err}`);
    process.exit(2);
  }
}

main();
