#!/usr/bin/env bash
# compare-baseline.sh — vérifie que la CLI `n2b` courante produit la même
# sortie que les snapshots baseline. Utilisé en CI et localement avant
# chaque commit de refactor. Échec si diff non-vide.
#
# Env:
#   N2B        : binaire à tester (défaut: n2b sur PATH)
#   RPB_ROOT   : racine du repo consommateur rpbey (défaut: /home/ubuntu/rpbey)
#   FIXTURE    : fixture locale (défaut: test/fixture)
#
# Les comparaisons JSON/JSONL passent par `jq -S` pour normaliser l'ordre
# des clés (insensible à preserve_order). Les autres formats sont comparés
# octet par octet.

set -euo pipefail

N2B="${N2B:-n2b}"
RPB_ROOT="${RPB_ROOT:-/home/ubuntu/rpbey}"
FIXTURE="${FIXTURE:-test/fixture}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

FAIL=0
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

compare() {
    local label="$1"
    local current="$2"
    local baseline="$3"
    local mode="$4"  # "json" | "bytes"

    if [[ ! -f "$baseline" ]]; then
        echo "SKIP   $label (baseline manquante: $baseline)"
        return
    fi

    if [[ "$mode" == "json" ]]; then
        if diff <(jq -S . "$current") <(jq -S . "$baseline") > /dev/null 2>&1; then
            echo "OK     $label"
        else
            echo "DIFF   $label"
            diff <(jq -S . "$current") <(jq -S . "$baseline") | head -40
            FAIL=1
        fi
    else
        if diff -q "$current" "$baseline" > /dev/null 2>&1; then
            echo "OK     $label"
        else
            echo "DIFF   $label"
            diff "$current" "$baseline" | head -40
            FAIL=1
        fi
    fi
}

echo "→ Fixture baselines ($FIXTURE)"
for fmt in text json jsonl md sarif; do
    ext="$fmt"
    [[ "$fmt" == "md" ]] && ext="md"
    [[ "$fmt" == "text" ]] && ext="txt"
    "$N2B" "$FIXTURE" --report="$fmt" > "$TMP/fixture.$ext" 2>&1 || true
    case "$fmt" in
        json|jsonl) compare "fixture.$ext" "$TMP/fixture.$ext" "tests/snapshots/baseline/fixture.$ext" json ;;
        *)          compare "fixture.$ext" "$TMP/fixture.$ext" "tests/snapshots/baseline/fixture.$ext" bytes ;;
    esac
done

"$N2B" rules --report=json > "$TMP/rules.json" 2>&1 || true
compare "rules.json" "$TMP/rules.json" "tests/snapshots/baseline/rules.json" json

"$N2B" rules --report=text > "$TMP/rules.txt" 2>&1 || true
compare "rules.txt" "$TMP/rules.txt" "tests/snapshots/baseline/rules.txt" bytes

if [[ -d "$RPB_ROOT" ]]; then
    echo "→ rpb-dashboard baselines ($RPB_ROOT)"
    for fmt in text json jsonl md sarif; do
        ext="$fmt"
        [[ "$fmt" == "md" ]] && ext="md"
        [[ "$fmt" == "text" ]] && ext="txt"
        "$N2B" "$RPB_ROOT" --report="$fmt" > "$TMP/rpb.$ext" 2>&1 || true
        case "$fmt" in
            json|jsonl) compare "rpb.$ext" "$TMP/rpb.$ext" "tests/rpb-dashboard-baseline/scan.$ext" json ;;
            *)          compare "rpb.$ext" "$TMP/rpb.$ext" "tests/rpb-dashboard-baseline/scan.$ext" bytes ;;
        esac
    done
else
    echo "SKIP   rpb-dashboard ($RPB_ROOT introuvable)"
fi

exit "$FAIL"
