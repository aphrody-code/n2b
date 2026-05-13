// shell.ts — thin re-export of Bun.$ for code that n2b flags as
// `api/child_process-exec`. `Bun.$` is Bun's shell template literal
// (cross-platform, no /bin/sh required) — callers can import it as a
// plain identifier via `import { sh } from "@n2b/core/shims"`.
//
// Rationale: in a migrating codebase, ad-hoc `import { $ } from "bun"`
// lines are as noisy as `import { exec } from "child_process"`. A stable
// alias `sh` makes the refactor mechanical and keeps the import name
// agnostic to the Bun version's surface.

import { $ } from "bun";

/** Alias for `Bun.$`. Template-literal shell: `await sh\`ls -la\``. */
export const sh = $;

export interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

/**
 * Run a shell command string and return its exit code, stdout, and stderr.
 * For template-literal syntax, use `sh` directly instead.
 */
export async function run(cmd: string): Promise<RunResult> {
  const proc = Bun.spawn(["sh", "-c", cmd], { stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, code] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  return { code, stdout, stderr };
}
