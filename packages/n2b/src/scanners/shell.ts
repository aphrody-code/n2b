import { applyCliRules } from "../rules/cli-commands";
import type { Finding } from "../types";

/** Scripts shell (.sh, .bash, Dockerfile, Makefile). */
export function scanShell(path: string, content: string): { findings: Finding[]; content: string } {
  return applyCliRules(path, content);
}
