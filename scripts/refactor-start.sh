#!/usr/bin/env bash
# Lance Claude headless dans une session tmux détachée pour exécuter
# plan/MISSION.md de bout en bout.
set -euo pipefail

N2B_ROOT="${N2B_ROOT:-/home/ubuntu/n2b}"
N2B_SESSION="${N2B_SESSION:-n2b}"
N2B_LOG="${N2B_LOG:-/tmp/n2b-refactor.log}"

# Pré-conditions
command -v tmux   >/dev/null 2>&1 || { echo "ERROR: tmux pas installé" >&2; exit 1; }
command -v claude >/dev/null 2>&1 || { echo "ERROR: claude pas dans PATH" >&2; exit 1; }
[ -f "$N2B_ROOT/plan/MISSION.md" ] || { echo "ERROR: $N2B_ROOT/plan/MISSION.md introuvable" >&2; exit 1; }

if tmux has-session -t "$N2B_SESSION" 2>/dev/null; then
  echo "n2b: session '$N2B_SESSION' déjà active — abort" >&2
  echo "     (n2b-attach pour voir, n2b-restart pour relancer)" >&2
  exit 1
fi

MISSION="$(cat "$N2B_ROOT/plan/MISSION.md")"

# `printf %q` quote la mission de façon shell-safe (gère sauts de ligne,
# guillemets, $, backticks, etc.) — indispensable pour la passer à `claude -p`
# à travers la couche `tmux new` qui ré-évalue la commande.
QUOTED_MISSION=$(printf '%q' "$MISSION")

# Marqueur de session dans le log pour distinguer les runs successifs.
{
  echo
  echo "════════════════════════════════════════════════════════════"
  echo "  n2b refactor — session démarrée $(date -Iseconds)"
  echo "  N2B_ROOT  = $N2B_ROOT"
  echo "  N2B_LOG   = $N2B_LOG"
  echo "  git HEAD  = $(git -C "$N2B_ROOT" rev-parse --short HEAD 2>/dev/null || echo '?')"
  echo "════════════════════════════════════════════════════════════"
} | tee -a "$N2B_LOG" >/dev/null

tmux new -d -s "$N2B_SESSION" \
  "cd '$N2B_ROOT' && claude -p $QUOTED_MISSION --dangerously-skip-permissions 2>&1 | tee -a '$N2B_LOG'"

echo "n2b: started"
echo "  session : $N2B_SESSION"
echo "  log     : $N2B_LOG"
echo "  attach  : tmux attach -t $N2B_SESSION    (ou n2b-attach)"
echo "  peek    : n2b-peek                       (snapshot non-interactif)"
echo "  state   : n2b-state                      (STATE.md + git)"
