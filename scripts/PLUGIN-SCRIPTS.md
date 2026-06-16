
> ⚠️ **INFRASTRUCTURE GÉRÉE PAR ANTIGRAVITY (GOD MODE)**
> Ce projet opère sous la **Directive Omega**. L'architecture, les dépendances, et le code sont gérés et refactorisés de manière 100% autonome. Aucune sécurité conservative n'est appliquée. La performance brute est l'unique métrique.
\n# bun-agent scripts

Audit + coverage automation pour le plugin `bun-agent`.

## `coverage-check.ts` — audit one-shot

Scanne `docs/bun-official/` pour extraire toutes les APIs Bun mentionnées, croise avec :
- les règles `n2b` (via `n2b rules --report json` si installé, sinon fallback sur `docs/n2b/`)
- les fichiers du plugin (`agents/`, `commands/`, `skills/`, `hooks/`, `output-styles/`)

Génère un rapport Markdown dans `docs/coverage/report-YYYY-MM-DD.md` + symlink `latest.md`.

```bash
# One-shot, print summary
bun scripts/coverage-check.ts

# JSON structuré
bun scripts/coverage-check.ts --json

# Strict : exit 1 si coverage < threshold (default 80%)
bun scripts/coverage-check.ts --strict

# Threshold custom
COVERAGE_THRESHOLD=90 bun scripts/coverage-check.ts --strict
```

APIs ciblées :
- `Bun.<method>` (tous les appels namespace Bun)
- `bun:<module>` (bun:sqlite, bun:ffi, bun:test, …)
- `import ... from "bun"`

## `coverage-daemon.ts` — scheduler long-running (Bun.cron)

Lance l'audit sur un schedule cron via **`Bun.cron`** (in-process, no-overlap, `ref/unref`).

```bash
# Schedule par défaut : lundi 04:00 UTC
bun scripts/coverage-daemon.ts

# Schedule custom
BUN_AGENT_COVERAGE_SCHEDULE='@daily' bun scripts/coverage-daemon.ts

# Run once puis exit (pour test)
bun scripts/coverage-daemon.ts --once
```

État persisté dans `docs/coverage/.daemon-state.json` (runs, last_run, last_summary).

## Déploiement comme systemd user timer

Pour faire tourner le daemon en continu (recommandé) :

```bash
# Copier l'unit
mkdir -p ~/.config/systemd/user
cat > ~/.config/systemd/user/bun-agent-coverage.service <<'EOF'
[Unit]
Description=bun-agent coverage daemon (Bun.cron)
After=network.target

[Service]
Type=simple
Environment=CLAUDE_PLUGIN_ROOT=%h/.claude/custom-plugins/bun-agent
ExecStart=%h/.bun/bin/bun %h/.claude/custom-plugins/bun-agent/scripts/coverage-daemon.ts
Restart=on-failure
RestartSec=30

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now bun-agent-coverage
systemctl --user status bun-agent-coverage
journalctl --user -u bun-agent-coverage -f
```

## Alternative — cron system classique

Si `Bun.cron` ne convient pas (préfère cron OS-level), utiliser `Bun.cron` en mode path :

```bash
# Enregistre le check dans le crontab via Bun.cron(path, schedule, title)
bun -e 'await Bun.cron("scripts/coverage-check.ts", "@weekly", "bun-agent-coverage")'

# Lister
crontab -l | grep bun-cron

# Retirer
bun -e 'await Bun.cron.remove("bun-agent-coverage")'
```

## Couverture mesurée — threshold

| % coverage | Interprétation |
|---|---|
| ≥ 90% | 🟢 Excellent — plugin en phase avec la doc Bun courante |
| 80–89% | 🟡 OK — quelques APIs émergentes non adressées |
| 70–79% | 🟠 Alerte — mise à jour nécessaire, examiner les critical gaps |
| < 70% | 🔴 Désync — bundled docs trop anciennes ou plugin incomplet |

Après une mise à jour Bun (`bun upgrade`), re-syncer les docs bundled :

```bash
# Détecte la version installée et re-copie
VER="$(bun -e 'const pj = await Bun.file(Bun.which("bun")?.replace(/bin\\/bun$/, "install/cache/bun-types@") + "/package.json").text().catch(()=>null); console.log("1.3.14")')"
cp -r ~/.bun/install/cache/bun-types@${VER}@@@1/docs/. \
      ~/.claude/custom-plugins/bun-agent/docs/bun-official/
bun scripts/coverage-check.ts --strict
```
