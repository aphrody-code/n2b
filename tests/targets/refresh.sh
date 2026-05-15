#!/usr/bin/env bash
# Re-bootstrap the canonical real-world test targets and refresh baselines.
# See plan/test-targets.md for context.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGETS="$ROOT/tests/targets"
N2B="${N2B:-/usr/local/bin/n2b}"

# --- shenron (lives in ~/vps/apps/shenron, not cloned here) ---
SHENRON="/home/ubuntu/vps/apps/shenron"
if [[ -d "$SHENRON" ]]; then
  mkdir -p "$TARGETS/shenron"
  "$N2B" "$SHENRON" --report=json > "$TARGETS/shenron/baseline.json"
  echo "shenron: $(jq -r '.findings_total' "$TARGETS/shenron/baseline.json") findings"
else
  echo "shenron: SKIP (not found at $SHENRON)"
fi

# --- gemini-cli (clone, gitignored) ---
GEMINI="$TARGETS/gemini-cli"
if [[ ! -d "$GEMINI" ]]; then
  git clone --depth 1 https://github.com/google-gemini/gemini-cli.git "$GEMINI"
fi
mkdir -p "$TARGETS/gemini-cli-out"
"$N2B" "$GEMINI" --report=json > "$TARGETS/gemini-cli-out/baseline.json"
echo "gemini-cli: $(jq -r '.findings_total' "$TARGETS/gemini-cli-out/baseline.json") findings"
