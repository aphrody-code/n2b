import { applyCliRules } from "../rules/cli-commands";
import type { Finding } from "../types";
import { makeFinding } from "../util";

/** Patch les workflows GitHub Actions : setup-node → setup-bun, commandes npm/pnpm → bun. */
export function scanWorkflow(
  path: string,
  content: string,
): { findings: Finding[]; content: string } {
  const findings: Finding[] = [];
  let out = content;

  // 1. actions/setup-node@v* → oven-sh/setup-bun@v2
  const setupNodeRe = /uses:\s*(?:actions\/setup-node@[^\s\n]+)/g;
  let m: RegExpExecArray | null;
  while ((m = setupNodeRe.exec(content)) !== null) {
    findings.push(
      makeFinding(
        path,
        content,
        m.index,
        "ci/setup-node",
        "actions/setup-node → oven-sh/setup-bun@v2",
        m[0],
        "uses: oven-sh/setup-bun@v2",
        { autofix: true },
      ),
    );
  }
  out = out.replace(setupNodeRe, "uses: oven-sh/setup-bun@v2");

  // 2. with: node-version: … → bun-version: latest (warn only; value is ambiguous)
  const nodeVersionRe = /node-version:\s*['"]?[^\n'"]+['"]?/g;
  while ((m = nodeVersionRe.exec(out)) !== null) {
    findings.push(
      makeFinding(
        path,
        out,
        m.index,
        "ci/node-version",
        "remplacer 'node-version' par 'bun-version: latest'",
        m[0],
        "bun-version: latest",
        { autofix: true },
      ),
    );
  }
  out = out.replace(nodeVersionRe, "bun-version: latest");

  // 3. cache: npm → cache est déjà automatique avec setup-bun, on supprime
  out = out.replace(/\n\s*cache:\s*['"]?(?:npm|yarn|pnpm)['"]?/g, "");

  // 4. CLI commands in run: steps (apply generic CLI rules).
  const { findings: cliFindings, content: cliRewritten } = applyCliRules(path, out);
  findings.push(...cliFindings);
  out = cliRewritten;

  return { findings, content: out };
}
