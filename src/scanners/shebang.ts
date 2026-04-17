import type { Finding } from "../types";
import { makeFinding } from "../util";

/** Rewrite shebangs using node → bun. */
export function scanShebang(path: string, content: string): { findings: Finding[]; content: string } {
  const findings: Finding[] = [];
  if (!content.startsWith("#!")) return { findings, content };
  const firstLine = content.slice(0, content.indexOf("\n"));
  // #!/usr/bin/env node   or   #!/usr/bin/node   or   #!node
  const re = /^#!\s*(?:\/usr\/bin\/env\s+node|\/usr\/bin\/node|node)(?=\s|$)/;
  const match = firstLine.match(re);
  if (!match) return { findings, content };

  const replacement = "#!/usr/bin/env bun";
  findings.push(makeFinding(path, content, 0, "shebang/node",
    `shebang 'node' → 'bun'`, firstLine, replacement, { autofix: true }));
  const rest = content.slice(firstLine.length);
  return { findings, content: replacement + rest };
}
