---
description: "Drive the Node.js → Bun-native migration on rpb-dashboard via `n2b` (Rust CLI). Scoped by phase (see `bun/MIGRATION_PLAN.md`). Delegates heavy lifting to the `n2b` agent."
argument-hint: "[audit | status | phase N | diff | fix <path> | analyze | prompt | rules | github | migrate | rollback]"
---

Drives the **RPB Node.js → Bun-native migration** using the `n2b` binary (Rust, v0.2.0, at `/usr/local/bin/n2b`, sources in `/home/ubuntu/rsbun/n2b/rust/`). Workflow anchored in `bun/MIGRATION_PLAN.md` and enforced by the "migrer vs garder" matrix.

**Scope** : only `/home/ubuntu/rpb-dashboard`. If the current working dir is elsewhere, `cd` there first. If `rpb-dashboard/bun/MIGRATION_PLAN.md` is missing, abort — this repo is out of scope.

## n2b CLI — full surface (v0.2.0)

```bash
# ── Scan (dry-run, non-mutating) ─────────────────────
n2b <path>                                      # default: text
n2b <path> --report <fmt>                       # fmt ∈ text|md|markdown|json|jsonl|sarif
n2b <path> --quiet                              # suppress trailing summary
n2b <path> --ignore <glob>                      # cumulable (respects .n2bignore too)
n2b <path> --agent                              # LLM mode: ANSI off, logs→stderr, stdout=payload
                                                #   (text → auto-promoted to json)

# ── Autofix (mutating) ───────────────────────────────
n2b <path> --fix                                # safe rewrites: cli/*, imports/node-prefix, ci/*, shebang/*, husky/*
n2b <path> --fix --aggressive                   # + api/* templateables, + imports/bun-native (bun:/node: targets only)
n2b <path> --migrate                            # --fix --aggressive + side-effects:
                                                #   1) pnpm-workspace.yaml → workspaces[] in root package.json
                                                #   2) removes pnpm-lock.yaml | yarn.lock | package-lock.json
                                                #   3) bun install (rebuilds bun.lock)
                                                #   4) adds @types/bun if Bun.* is used

# ── Subcommands ──────────────────────────────────────
n2b rules [--report <fmt>]                      # list 68 active rules by category
n2b prompt <path> [--max-findings N] [--include-info]  # emit LLM-ready prompt
n2b audit <path> [--state open|closed|all] [--limit N] [--term <kw>]  # GitHub issues/PRs mentioning bun/node
n2b analyze <path> [--top-k N] [--threshold F] [--apply fix|aggressive]
                                                # scan + audit + ML embedding crosslink

# Exit codes:
#   0 = no findings, or fix/aggressive/migrate succeeded
#   1 = dry-run with findings present
#   2 = internal error or severity=Error (e.g. invalid package.json)
```

**No `--rule <id>` flag** — scope rules by `<path>` or post-filter the JSONL report with `jq`.

## Modes exposed by this command

### `audit` — default when no arg

```bash
cd /home/ubuntu/rpb-dashboard
n2b . --report text --ignore 'src/generated/**' --ignore 'bot/src/generated/**' --ignore '.next/**' --ignore 'bot/dist/**'
```

Then:
1. Group findings by rule, show top 10 files by count.
2. `delta bun/reports/n2b-baseline.md <(n2b . --report md ...)` — delta vs baseline.
3. Recommend the next phase from `bun/MIGRATION_PLAN.md` based on which phases already have a green commit.

### `status` — where are we in the plan

Parses `git log --oneline main..HEAD` for commits matching the phase prefixes (`chore(bun):`, `refactor(bun):`, `fix(bot):`, `perf(bun):`) and checks which phases are done. Lists remaining phases and scopes them against a fresh scan.

### `<N>` — run phase N (1-10)

1. Read the phase block in `bun/MIGRATION_PLAN.md`.
2. Pre-flight : `git status --porcelain` doit être vide. Branche `chore/bun-native` vérifiée.
3. **Déléguer à l'agent `n2b`** via le tool Agent avec le numéro de phase et le scope.
4. L'agent choisit la bonne stratégie :
   - Phase 2 → `n2b <scope> --fix` (safe)
   - Phase 3 → `n2b <scope> --aggressive` (api/* templates, scope limité à `scripts/`, `prisma/`)
   - Phase 1, 4, 5, 7, 8, 9 → Edit manuel (rewrites report-only ou hors n2b)
   - Phase 6 → SKIP (décision du plan)
5. Gate de validation : `bun install --frozen-lockfile && bun db:generate && bun run build && bun bot:build && bun run lint`. Tout doit passer.
6. Commit avec le message conventional **exact** du plan.
7. Nouveau rapport : `n2b . --report md > bun/reports/n2b-after-phase-<N>.md` + diff par règle.

### `diff` — delta baseline → current

```bash
cd /home/ubuntu/rpb-dashboard
n2b . --agent --report jsonl > /tmp/n2b-current.jsonl
jq -r '.rule_id' bun/reports/n2b-baseline.jsonl | sort | uniq -c | sort -rn > /tmp/baseline-rules.txt
jq -r '.rule_id' /tmp/n2b-current.jsonl | sort | uniq -c | sort -rn > /tmp/current-rules.txt
delta /tmp/baseline-rules.txt /tmp/current-rules.txt
```

Output : règles entièrement résolues, règles restantes (avec delta numérique), top fichiers encore touchés.

### `fix <path>` — autofix scopé manuel

1. `n2b <path> --fix` (safe uniquement, **jamais** `--aggressive` par défaut en mode `fix`).
2. Gate de validation complet.
3. Si vert → demander explicitement avant de commit (ce mode court-circuite le plan phasé).
4. Si rouge → `git restore .` et rapporter ce qui casse.

### `analyze` — ML crosslink

```bash
n2b analyze . --top-k 3 --threshold 0.35 --report md
```

Crosslinks les findings n2b avec les issues GitHub (discord.js, discordx, nextjs, prisma, etc.) via embeddings. Utile avant une phase pour voir si un rewrite a déjà cassé ailleurs.

### `prompt` — export LLM

```bash
n2b prompt . --max-findings 100 --include-info > /tmp/n2b-prompt.md
```

Retourne le chemin. Utilisable pour un second avis externe.

### `rules` — catalogue complet

```bash
n2b rules --report md
```

68 règles actives. Préférer l'index exhaustif dans l'agent (`~/.claude/agents/n2b.md`).

### `github` — audit issues/PRs

```bash
n2b audit . --state all --limit 30 --report md
```

Issues et PRs mentionnant bun/node pour ce repo. Utile avant de planifier une phase.

### `migrate` — full auto (danger)

1. **Exige confirmation explicite** — ne jamais exécuter sans feu vert.
2. Refuse si `git status` n'est pas clean.
3. Refuse si des phases du plan ne sont pas terminées.
4. Applique : `--fix --aggressive` + suppression des lockfiles rivaux + `bun install` + migration workspaces + ajout `@types/bun`.
5. Gate de validation obligatoire avant commit.

### `rollback` — annuler la phase en cours

1. Si aucun commit fait : `git restore -W -S -- .` (working + staged) puis proposer `git clean -fd <scope-touched>` avec confirmation.
2. Si commit déjà fait : afficher le hash, proposer `git reset --hard HEAD~1` **avec confirmation explicite** (destructif).
3. Jamais de force-push sans demande explicite.

## Hard rules — never break

| Contrainte | Source |
|---|---|
| `src/generated/**`, `bot/src/generated/**`, `prisma/schema.prisma` — **ignorés** | Auto-générés par `prisma generate` |
| `pg`, `@prisma/adapter-pg` — **gardés** | Prisma v7 impose le driver adapter |
| `bot/src/**` (hors scripts) — **pas de `Bun.$`, pas de rewrites TS-direct** | Bot compilé par SWC (discordx + emitDecoratorMetadata) |
| `next.config.ts`, `src/app/api/**` — **`process.env` gardé** | Portabilité Next.js / SSR |
| `next build` — **jamais `--bun`** | Mémoire `feedback_no_bun_flag_build` (cohérence Turbopack) |
| `import 'reflect-metadata'` du bot — **gardé** | discordx |
| Phase 6 — **skip** (`api/process-env`, 173 info) | Portabilité |
| Mode `--aggressive` — **Phase 3 uniquement** | Toute autre phase → Edit manuel |
| Mode `--migrate` — **jamais** sauf demande explicite | Side-effects lourds |
| Flag scope — **toujours `n2b <path>`, jamais `n2b .`** en autofix | Éviter les rewrites hors scope |
| Commit — **1 phase = 1 commit** avec le message exact du plan | Bisectable |

## Gate de validation (après chaque phase mutante)

```bash
bun install --frozen-lockfile
bun db:generate                   # si prisma/ touché
bun run build                     # Next.js (SANS --bun)
bun bot:build                     # SWC pour le bot
bun run lint
```

Un échec → corriger avant de committer. Pas de commit cassé. Pas de `--no-verify`.

## Smoke tests (Phase 10 uniquement)

```bash
sudo systemctl restart rpb-dashboard rpb-bot
sudo journalctl -u rpb-dashboard -n 50 --no-pager
sudo journalctl -u rpb-bot -n 50 --no-pager
curl -fsS https://rpbey.fr/api/discord/stats | head
```

## Invocation

Mode autonome (cf. `rpb-dashboard/CLAUDE.md`) — pas de confirmation, auto-fix, auto-commit par phase validée. **Exceptions** :
- `migrate` et `rollback` → confirmation obligatoire (destructifs).
- `fix <path>` → confirmation avant commit (court-circuite le plan).

Pour toute opération impliquant du code, **déléguer à l'agent `n2b`** via le tool Agent — il connaît le catalogue complet des 68 règles, la matrice "garder", et les patterns d'autofix par phase.
