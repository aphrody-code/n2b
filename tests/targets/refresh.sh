#!/usr/bin/env bash
# Re-bootstrap the canonical real-world test targets and refresh baselines.
# See plan/test-targets.md for context.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGETS="$ROOT/tests/targets"
N2B="${N2B:-/usr/local/bin/n2b}"

# --- bun-full (single-file canonical fixture, committed) ---
if [[ -f "$TARGETS/bun-full/app.tsx" ]]; then
  "$N2B" "$TARGETS/bun-full" --report=json > "$TARGETS/bun-full/baseline.json"
  echo "bun-full: $(jq -r '.findings_total' "$TARGETS/bun-full/baseline.json") findings"
else
  echo "bun-full: SKIP (app.tsx introuvable)"
fi

# --- gemini-cli (clone, gitignored) ---
GEMINI="$TARGETS/gemini-cli"
if [[ ! -d "$GEMINI" ]]; then
  git clone --depth 1 https://github.com/google-gemini/gemini-cli.git "$GEMINI"
fi
mkdir -p "$TARGETS/gemini-cli-out"
"$N2B" "$GEMINI" --report=json > "$TARGETS/gemini-cli-out/baseline.json"
echo "gemini-cli: $(jq -r '.findings_total' "$TARGETS/gemini-cli-out/baseline.json") findings"
