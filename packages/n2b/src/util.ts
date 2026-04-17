import { computeLineOffsets, posFromOffsets } from "./native";
import type { Finding } from "./types";

export const colors = {
  reset: "\x1b[0m",
  dim: "\x1b[2m",
  bold: "\x1b[1m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  magenta: "\x1b[35m",
  cyan: "\x1b[36m",
};

// Cache des offsets de newlines par fichier. Clé = path ; on l'efface
// entre fichiers pour ne pas accumuler. Le scanner ne traite qu'un seul
// fichier à la fois dans un context donné, donc la taille reste
// minuscule, mais on borne par sûreté.
const OFFSETS_CACHE = new Map<string, { source: string; offsets: Uint32Array }>();
const CACHE_MAX = 8;

function getOffsets(path: string, source: string): Uint32Array {
  const hit = OFFSETS_CACHE.get(path);
  if (hit && hit.source === source) return hit.offsets;
  const offsets = computeLineOffsets(source);
  if (OFFSETS_CACHE.size >= CACHE_MAX) {
    const firstKey = OFFSETS_CACHE.keys().next().value;
    if (firstKey !== undefined) OFFSETS_CACHE.delete(firstKey);
  }
  OFFSETS_CACHE.set(path, { source, offsets });
  return offsets;
}

export function posFromIndex(source: string, index: number): { line: number; col: number } {
  // API conservée pour compat : pas de path → calcul direct sans cache.
  return posFromOffsets(computeLineOffsets(source), Math.min(index, source.length));
}

export function makeFinding(
  path: string,
  source: string,
  index: number,
  ruleId: string,
  message: string,
  original: string,
  replacement: string | undefined,
  opts: { severity?: Finding["severity"]; autofix?: boolean; aggressive?: boolean } = {},
): Finding {
  const offsets = getOffsets(path, source);
  const { line, col } = posFromOffsets(offsets, Math.min(index, source.length));
  return {
    file: path,
    line,
    col,
    ruleId,
    severity: opts.severity ?? "warn",
    message,
    original,
    replacement,
    autofix: opts.autofix ?? replacement !== undefined,
    aggressive: opts.aggressive,
  };
}
