import type { Finding } from "../types";
import { makeFinding } from "../util";

/**
 * Table of CLI command migrations. Patterns are applied in order of declaration.
 * Each `match` is a regex with a single capture that will be replaced by `replace`
 * (supports $1..$n).
 */
interface Mapping {
  match: RegExp;
  replace: string;
  ruleId: string;
  message: string;
}

const MAPPINGS: Mapping[] = [
  // npm ---------------------------------------------------------------
  { match: /\bnpm\s+install\s+--save-dev\b/g,  replace: "bun add --dev",            ruleId: "cli/npm-install-dev",   message: "npm install --save-dev → bun add --dev" },
  { match: /\bnpm\s+install\s+-D\b/g,          replace: "bun add --dev",            ruleId: "cli/npm-install-D",     message: "npm install -D → bun add --dev" },
  { match: /\bnpm\s+install\s+--save\b/g,      replace: "bun add",                  ruleId: "cli/npm-install-save",  message: "npm install --save → bun add" },
  { match: /\bnpm\s+i\s+--save-dev\b/g,        replace: "bun add --dev",            ruleId: "cli/npm-i-dev",         message: "npm i --save-dev → bun add --dev" },
  { match: /\bnpm\s+i\s+-D\b/g,                replace: "bun add --dev",            ruleId: "cli/npm-i-D",           message: "npm i -D → bun add --dev" },
  { match: /\bnpm\s+install\b(?!\s+-g)/g,      replace: "bun install",              ruleId: "cli/npm-install",       message: "npm install → bun install" },
  { match: /\bnpm\s+i\b(?!\s+-g)/g,            replace: "bun install",              ruleId: "cli/npm-i",             message: "npm i → bun install" },
  { match: /\bnpm\s+install\s+-g\b/g,          replace: "bun install -g",           ruleId: "cli/npm-install-global",message: "npm install -g → bun install -g" },
  { match: /\bnpm\s+ci\b/g,                    replace: "bun install --frozen-lockfile", ruleId: "cli/npm-ci",      message: "npm ci → bun install --frozen-lockfile" },
  { match: /\bnpm\s+run\s+/g,                  replace: "bun run ",                 ruleId: "cli/npm-run",           message: "npm run → bun run" },
  { match: /\bnpm\s+test\b/g,                  replace: "bun test",                 ruleId: "cli/npm-test",          message: "npm test → bun test" },
  { match: /\bnpm\s+start\b/g,                 replace: "bun start",                ruleId: "cli/npm-start",         message: "npm start → bun start" },
  { match: /\bnpm\s+uninstall\b/g,             replace: "bun remove",               ruleId: "cli/npm-uninstall",     message: "npm uninstall → bun remove" },
  { match: /\bnpm\s+rm\b/g,                    replace: "bun remove",               ruleId: "cli/npm-rm",            message: "npm rm → bun remove" },
  { match: /\bnpm\s+update\b/g,                replace: "bun update",               ruleId: "cli/npm-update",        message: "npm update → bun update" },
  { match: /\bnpm\s+outdated\b/g,              replace: "bun outdated",             ruleId: "cli/npm-outdated",      message: "npm outdated → bun outdated" },
  { match: /\bnpm\s+pack\b/g,                  replace: "bun pm pack",              ruleId: "cli/npm-pack",          message: "npm pack → bun pm pack" },
  { match: /\bnpm\s+publish\b/g,               replace: "bun publish",              ruleId: "cli/npm-publish",       message: "npm publish → bun publish" },
  { match: /\bnpm\s+link\b/g,                  replace: "bun link",                 ruleId: "cli/npm-link",          message: "npm link → bun link" },
  { match: /\bnpm\s+init\s+-y\b/g,             replace: "bun init -y",              ruleId: "cli/npm-init-y",        message: "npm init -y → bun init -y" },
  { match: /\bnpm\s+init\b/g,                  replace: "bun init",                 ruleId: "cli/npm-init",          message: "npm init → bun init" },
  { match: /\bnpm\s+exec\s+/g,                 replace: "bunx ",                    ruleId: "cli/npm-exec",          message: "npm exec → bunx" },
  { match: /\bnpx\s+/g,                        replace: "bunx ",                    ruleId: "cli/npx",               message: "npx → bunx" },

  // pnpm --------------------------------------------------------------
  { match: /\bpnpm\s+install\s+--save-dev\b/g, replace: "bun add --dev",            ruleId: "cli/pnpm-install-dev",  message: "pnpm install --save-dev → bun add --dev" },
  { match: /\bpnpm\s+add\s+--save-dev\b/g,     replace: "bun add --dev",            ruleId: "cli/pnpm-add-dev",      message: "pnpm add --save-dev → bun add --dev" },
  { match: /\bpnpm\s+add\s+-D\b/g,             replace: "bun add --dev",            ruleId: "cli/pnpm-add-D",        message: "pnpm add -D → bun add --dev" },
  { match: /\bpnpm\s+install\b/g,              replace: "bun install",              ruleId: "cli/pnpm-install",      message: "pnpm install → bun install" },
  { match: /\bpnpm\s+i\b/g,                    replace: "bun install",              ruleId: "cli/pnpm-i",            message: "pnpm i → bun install" },
  { match: /\bpnpm\s+add\b/g,                  replace: "bun add",                  ruleId: "cli/pnpm-add",          message: "pnpm add → bun add" },
  { match: /\bpnpm\s+remove\b/g,               replace: "bun remove",               ruleId: "cli/pnpm-remove",       message: "pnpm remove → bun remove" },
  { match: /\bpnpm\s+rm\b/g,                   replace: "bun remove",               ruleId: "cli/pnpm-rm",           message: "pnpm rm → bun remove" },
  { match: /\bpnpm\s+run\s+/g,                 replace: "bun run ",                 ruleId: "cli/pnpm-run",          message: "pnpm run → bun run" },
  { match: /\bpnpm\s+test\b/g,                 replace: "bun test",                 ruleId: "cli/pnpm-test",         message: "pnpm test → bun test" },
  { match: /\bpnpm\s+start\b/g,                replace: "bun start",                ruleId: "cli/pnpm-start",        message: "pnpm start → bun start" },
  { match: /\bpnpm\s+update\b/g,               replace: "bun update",               ruleId: "cli/pnpm-update",       message: "pnpm update → bun update" },
  { match: /\bpnpm\s+dlx\s+/g,                 replace: "bunx ",                    ruleId: "cli/pnpm-dlx",          message: "pnpm dlx → bunx" },
  { match: /\bpnpm\s+exec\s+/g,                replace: "bunx ",                    ruleId: "cli/pnpm-exec",         message: "pnpm exec → bunx" },

  // yarn --------------------------------------------------------------
  { match: /\byarn\s+install\s+--frozen-lockfile\b/g, replace: "bun install --frozen-lockfile", ruleId: "cli/yarn-install-frozen", message: "yarn install --frozen-lockfile → bun install --frozen-lockfile" },
  { match: /\byarn\s+add\s+--dev\b/g,          replace: "bun add --dev",            ruleId: "cli/yarn-add-dev",      message: "yarn add --dev → bun add --dev" },
  { match: /\byarn\s+add\s+-D\b/g,             replace: "bun add --dev",            ruleId: "cli/yarn-add-D",        message: "yarn add -D → bun add --dev" },
  { match: /\byarn\s+add\b/g,                  replace: "bun add",                  ruleId: "cli/yarn-add",          message: "yarn add → bun add" },
  { match: /\byarn\s+remove\b/g,               replace: "bun remove",               ruleId: "cli/yarn-remove",       message: "yarn remove → bun remove" },
  { match: /\byarn\s+install\b/g,              replace: "bun install",              ruleId: "cli/yarn-install",      message: "yarn install → bun install" },
  { match: /\byarn\s+run\s+/g,                 replace: "bun run ",                 ruleId: "cli/yarn-run",          message: "yarn run → bun run" },
  { match: /\byarn\s+test\b/g,                 replace: "bun test",                 ruleId: "cli/yarn-test",         message: "yarn test → bun test" },
  { match: /\byarn\s+start\b/g,                replace: "bun start",                ruleId: "cli/yarn-start",        message: "yarn start → bun start" },
  { match: /\byarn\s+dlx\s+/g,                 replace: "bunx ",                    ruleId: "cli/yarn-dlx",          message: "yarn dlx → bunx" },
  // Bare `yarn` → `bun install` (classic yarn treats it that way).
  { match: /^(\s*)yarn(\s|$)/gm,               replace: "$1bun install$2",          ruleId: "cli/yarn-bare",         message: "bare `yarn` → bun install" },
];

/** Ignore lines that start with `#` as shell comments, or are inside the
 *  line `"name": "npm"` of a package.json. We do not have a real parser here
 *  but we want to avoid the obvious false positives. */
const COMMENT_PREFIX = /^\s*(#|\/\/)/;

export function applyCliRules(path: string, source: string): { findings: Finding[]; content: string } {
  const findings: Finding[] = [];
  let out = source;

  for (const rule of MAPPINGS) {
    // find all matches first (positions are for the pre-replace buffer)
    rule.match.lastIndex = 0;
    const hits: { index: number; text: string }[] = [];
    let m: RegExpExecArray | null;
    while ((m = rule.match.exec(out)) !== null) {
      // skip comments in shell/yaml-like files
      const lineStart = out.lastIndexOf("\n", m.index) + 1;
      const lineEnd = out.indexOf("\n", m.index);
      const line = out.slice(lineStart, lineEnd === -1 ? undefined : lineEnd);
      if (COMMENT_PREFIX.test(line)) continue;
      hits.push({ index: m.index, text: m[0] });
    }
    if (hits.length === 0) continue;

    for (const hit of hits) {
      findings.push(
        makeFinding(path, out, hit.index, rule.ruleId, rule.message, hit.text, hit.text.replace(rule.match, rule.replace), { autofix: true }),
      );
    }
    // Apply replacement globally after recording findings.
    rule.match.lastIndex = 0;
    out = out.replace(rule.match, rule.replace);
  }

  return { findings, content: out };
}
