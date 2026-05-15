#!/usr/bin/env bash
# n2b autonomous refactor — env + shell functions.
# Source-le pour les commandes n2b-* :
#   echo 'source /home/ubuntu/n2b/scripts/n2b-env.sh' >> ~/.bashrc
# (déjà fait par le setup initial — ce fichier est idempotent)

export N2B_ROOT="${N2B_ROOT:-/home/ubuntu/n2b}"
export N2B_SESSION="${N2B_SESSION:-n2b}"
export N2B_LOG="${N2B_LOG:-/tmp/n2b-refactor.log}"

n2b-start() {
  if tmux has-session -t "$N2B_SESSION" 2>/dev/null; then
    echo "n2b: session '$N2B_SESSION' tourne déjà (utilise n2b-attach ou n2b-restart)" >&2
    return 1
  fi
  bash "$N2B_ROOT/scripts/refactor-start.sh"
}

n2b-attach()  { tmux attach -t "$N2B_SESSION"; }
n2b-peek()    { tmux capture-pane -t "$N2B_SESSION" -pS -2000 | tail -100; }
n2b-log()     { tail -f "$N2B_LOG"; }
n2b-state()   { bash "$N2B_ROOT/scripts/refactor-state.sh"; }

n2b-status() {
  if tmux has-session -t "$N2B_SESSION" 2>/dev/null; then
    echo "n2b: running"
    tmux list-sessions 2>/dev/null | grep "^$N2B_SESSION:"
  else
    echo "n2b: not running"
  fi
}

n2b-stop() {
  tmux has-session -t "$N2B_SESSION" 2>/dev/null || { echo "n2b: not running"; return 1; }
  tmux send-keys -t "$N2B_SESSION" C-c
  echo "n2b: SIGINT envoyé (Claude doit écrire STATE.md avant de sortir)"
}

n2b-kill() {
  tmux kill-session -t "$N2B_SESSION" 2>/dev/null && echo "n2b: killed" || echo "n2b: not running"
}

n2b-restart() {
  n2b-kill
  sleep 1
  n2b-start
}

n2b-green() {
  ( cd "$N2B_ROOT" && bun run green-gate )
}

n2b-help() {
  cat <<'HELP'
n2b autonomous refactor — shell helpers :
  n2b-start    Démarre la session tmux 'n2b' (Claude headless + plan/MISSION.md)
  n2b-attach   Attache au terminal interactif (Ctrl-b d pour détacher)
  n2b-peek     Snapshot non-interactif (dernières 100 lignes scrollback tmux)
  n2b-log      tail -f sur /tmp/n2b-refactor.log
  n2b-state    plan/STATE.md + git log + git status + Status board
  n2b-status   Tourne ou pas ?
  n2b-stop     Ctrl-C poli (handoff via STATE.md)
  n2b-kill     Tue brutalement
  n2b-restart  kill + start (reprise depuis STATE.md)
  n2b-green    Lance le green-gate complet (TS + Rust + baseline + codegen)
  n2b-help     Cette aide

Équivalents bun (utilisables depuis n'importe où dans le repo) :
  bun run refactor:{start,attach,peek,state,status,stop,kill,restart,log}
  bun run green-gate            # green-gate complet en bun
  turbo run //#green-gate       # idem mais orchestré par turbo (DAG parallèle)
HELP
}
