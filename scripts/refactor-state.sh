#!/usr/bin/env bash
# Vue d'ensemble de l'état du refactor : STATE.md + git log + git status + Status board.
set -euo pipefail
N2B_ROOT="${N2B_ROOT:-/home/ubuntu/n2b}"
cd "$N2B_ROOT"

echo "── plan/STATE.md ──"
if [ -f plan/STATE.md ]; then
  cat plan/STATE.md
else
  echo "(STATE.md absent — Phase 0 pas encore commencée ou jamais committée)"
fi
echo
echo "── git log (20 derniers commits) ──"
git log --oneline -20
echo
echo "── git status ──"
out=$(git status --short)
if [ -z "$out" ]; then
  echo "(arbre propre)"
else
  echo "$out"
fi
echo
echo "── plan/README.md Status board ──"
# Extrait le tableau « Status board » (du marqueur « Status board » à la
# première ligne vide après le tableau).
awk '/^## Status board/{flag=1} flag && /^\| Phase /{print; getline; print; flag=2; next} flag==2 && /^\|/{print; next} flag==2 && !/^\|/{exit}' plan/README.md
