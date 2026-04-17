import type { Finding } from "../types";
import { makeFinding } from "../util";

export const RIVAL_LOCKFILES = [
  "package-lock.json",
  "npm-shrinkwrap.json",
  "pnpm-lock.yaml",
  "yarn.lock",
];

/** Repère les lockfiles concurrents et émet un finding. */
export function checkLockfile(path: string, name: string): Finding | null {
  if (!RIVAL_LOCKFILES.includes(name)) return null;
  return makeFinding(path, "", 0, "lock/rival",
    `lockfile concurrent '${name}' présent — exécuter 'bun install' puis supprimer ce fichier`,
    name, undefined, { autofix: false, severity: "warn" });
}
