import { applyCliRules } from "../rules/cli-commands";
import type { Finding } from "../types";
import { makeFinding } from "../util";

/** Packages qui sont remplacés par une API native de Bun. */
const REDUNDANT_DEPS = new Set([
  "node-fetch",
  "isomorphic-fetch",
  "cross-fetch",
  "dotenv",
  "dotenv-cli",
  "rimraf",
  "mkdirp",
  "better-sqlite3",
  "sqlite3",
  "uuid",
  "nanoid",
  "ts-node",
  "tsx",
  "ts-node-esm",
  "concurrently", // bun supporte `bun run --parallel`
]);

/** Scripts en jouant sur les scripts du package.json. */
export function scanPackageJson(
  path: string,
  content: string,
): { findings: Finding[]; content: string } {
  const findings: Finding[] = [];
  let parsed: any;
  try {
    parsed = JSON.parse(content);
  } catch (err) {
    findings.push(
      makeFinding(
        path,
        content,
        0,
        "pkg/parse",
        `package.json invalide : ${(err as Error).message}`,
        content.slice(0, 40),
        undefined,
        { severity: "error", autofix: false },
      ),
    );
    return { findings, content };
  }

  let mutated = false;

  // 1. Rewrite scripts.
  if (parsed.scripts && typeof parsed.scripts === "object") {
    for (const [name, raw] of Object.entries(parsed.scripts as Record<string, unknown>)) {
      if (typeof raw !== "string") continue;
      const { findings: scriptFindings, content: rewritten } = applyCliRules(
        `${path} [scripts.${name}]`,
        raw,
      );
      findings.push(...scriptFindings);
      if (rewritten !== raw) {
        parsed.scripts[name] = rewritten;
        mutated = true;
      }
    }
  }

  // 2. packageManager field → bun.
  if (typeof parsed.packageManager === "string" && !parsed.packageManager.startsWith("bun@")) {
    const original = parsed.packageManager;
    findings.push(
      makeFinding(
        path,
        content,
        0,
        "pkg/package-manager",
        `packageManager='${original}' — remplacer par 'bun@<version>' ou supprimer`,
        original,
        undefined,
        { autofix: false },
      ),
    );
  }

  // 3. engines.node / engines.npm → recommand Bun.
  if (parsed.engines && typeof parsed.engines === "object") {
    if ("npm" in parsed.engines || "pnpm" in parsed.engines || "yarn" in parsed.engines) {
      findings.push(
        makeFinding(
          path,
          content,
          0,
          "pkg/engines-pm",
          `engines.{npm,pnpm,yarn} est superflu avec Bun — utiliser 'engines.bun'`,
          JSON.stringify(parsed.engines),
          undefined,
          { autofix: false },
        ),
      );
    }
  }

  // 4. Dépendances redondantes avec les APIs Bun.
  const depSources = [
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
  ] as const;
  for (const key of depSources) {
    const deps = parsed[key];
    if (!deps || typeof deps !== "object") continue;
    for (const dep of Object.keys(deps)) {
      if (REDUNDANT_DEPS.has(dep)) {
        findings.push(
          makeFinding(
            path,
            content,
            0,
            "pkg/redundant-dep",
            `dépendance '${dep}' redondante avec Bun (voir Bun.file / Bun.env / fetch global / bun:sqlite / bun test)`,
            dep,
            undefined,
            { autofix: false, aggressive: true },
          ),
        );
      }
    }
  }

  // 5. main entry using commonjs while type=module → informational only.
  if (parsed.type === "module" && typeof parsed.main === "string" && parsed.main.endsWith(".cjs")) {
    findings.push(
      makeFinding(
        path,
        content,
        0,
        "pkg/main-mismatch",
        `"type":"module" mais main pointe sur un fichier .cjs`,
        parsed.main,
        undefined,
        { autofix: false },
      ),
    );
  }

  const newContent = mutated
    ? JSON.stringify(parsed, null, 2) + (content.endsWith("\n") ? "\n" : "")
    : content;
  return { findings, content: newContent };
}
