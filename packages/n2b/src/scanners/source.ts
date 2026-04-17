import { applyBunApiRules } from "../rules/bun-apis";
import { applyNodeImportRules } from "../rules/node-imports";
import type { Finding, RunOptions } from "../types";
import { scanShebang } from "./shebang";

/**
 * Scan applicable à tout fichier source JS/TS.
 * Retourne les findings et le contenu après application des fixes autorisés
 * par `opts.mode` ('check' laisse intact, 'fix' applique safe, 'aggressive' applique tout).
 */
export function scanSource(
  path: string,
  content: string,
  opts: RunOptions,
): { findings: Finding[]; content: string } {
  const all: Finding[] = [];
  const aggressive = opts.mode === "aggressive";

  const shebang = scanShebang(path, content);
  all.push(...shebang.findings);
  let working = shebang.content;

  const nodeImp = applyNodeImportRules(path, working, aggressive);
  all.push(...nodeImp.findings);
  working = nodeImp.content;

  const bunApi = applyBunApiRules(path, working, aggressive);
  all.push(...bunApi.findings);
  working = bunApi.content;

  // En mode 'check', on ne modifie rien.
  if (opts.mode === "check") return { findings: all, content };

  return { findings: all, content: working };
}
