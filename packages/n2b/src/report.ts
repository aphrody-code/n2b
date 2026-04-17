import type { FileFix, Finding, RunOptions } from "./types";
import { colors as c } from "./util";

export function renderText(fixes: FileFix[], opts: RunOptions): string {
  const lines: string[] = [];
  const totalFindings = fixes.reduce((n, f) => n + f.findings.length, 0);
  const changed = fixes.filter((f) => f.before !== f.after);

  const header = `${c.bold}node2bun${c.reset} ${c.dim}(mode: ${opts.mode})${c.reset}`;
  lines.push(header);
  lines.push(`  racine : ${opts.root}`);
  lines.push(`  fichiers scannés : ${fixes.length}`);
  lines.push(`  findings : ${totalFindings}`);
  lines.push(`  fichiers modifiés : ${changed.length}`);
  lines.push("");

  const bySeverity: Record<string, number> = { info: 0, warn: 0, error: 0 };
  for (const f of fixes) for (const x of f.findings) bySeverity[x.severity]!++;

  // Group findings by file.
  for (const fix of fixes) {
    if (fix.findings.length === 0) continue;
    const mark = fix.before !== fix.after ? `${c.green}✓${c.reset}` : `${c.yellow}•${c.reset}`;
    lines.push(`${mark} ${c.bold}${fix.file}${c.reset}`);
    for (const f of fix.findings) {
      const sev = severityColor(f.severity);
      const loc = `${c.dim}${f.line}:${f.col}${c.reset}`;
      const tag = `${c.cyan}${f.ruleId}${c.reset}`;
      const fix = f.replacement
        ? ` ${c.dim}→${c.reset} ${c.green}${ellipsis(f.replacement, 60)}${c.reset}`
        : "";
      lines.push(`    ${sev} ${loc} ${tag} ${f.message}${fix}`);
    }
  }

  lines.push("");
  lines.push(
    `${c.bold}Bilan${c.reset} : ${c.red}${bySeverity.error} errors${c.reset}, ` +
      `${c.yellow}${bySeverity.warn} warns${c.reset}, ` +
      `${c.dim}${bySeverity.info} infos${c.reset}`,
  );
  if (opts.mode === "check" && totalFindings > 0) {
    lines.push(
      `${c.dim}(relancer avec --fix pour appliquer les corrections sûres, --aggressive pour migrer les APIs)${c.reset}`,
    );
  }

  return lines.join("\n");
}

export function renderJson(fixes: FileFix[], opts: RunOptions): string {
  return JSON.stringify(
    {
      mode: opts.mode,
      root: opts.root,
      files: fixes.map((f) => ({
        path: f.file,
        changed: f.before !== f.after,
        findings: f.findings,
      })),
    },
    null,
    2,
  );
}

export function renderMarkdown(fixes: FileFix[], opts: RunOptions): string {
  const out: string[] = [];
  out.push("# node2bun report");
  out.push("");
  out.push(`- mode : \`${opts.mode}\``);
  out.push(`- racine : \`${opts.root}\``);
  out.push("");
  for (const fix of fixes) {
    if (fix.findings.length === 0) continue;
    out.push(`## \`${fix.file}\``);
    out.push("");
    out.push("| ligne | règle | message | remplacement |");
    out.push("| --- | --- | --- | --- |");
    for (const f of fix.findings) {
      out.push(
        `| ${f.line}:${f.col} | \`${f.ruleId}\` | ${f.message} | ${f.replacement ? `\`${escapeMd(f.replacement)}\`` : ""} |`,
      );
    }
    out.push("");
  }
  return out.join("\n");
}

function escapeMd(s: string) {
  return s.replace(/\|/g, "\\|").replace(/\n/g, " ");
}
function ellipsis(s: string, n: number) {
  return s.length > n ? `${s.slice(0, n - 1)}…` : s;
}

function severityColor(s: Finding["severity"]): string {
  switch (s) {
    case "error":
      return `${c.red}✗${c.reset}`;
    case "warn":
      return `${c.yellow}!${c.reset}`;
    default:
      return `${c.dim}i${c.reset}`;
  }
}
