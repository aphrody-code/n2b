# Cibles de test du refactor — fixtures réelles

Deux dépôts servent de banc d'essai canonique pour valider la progression des deux
piliers du refactor (cf. [REFACTOR_PLAN.md](../REFACTOR_PLAN.md)).

| Cible | Pilier | Phases | Profil |
|---|---|---|---|
| **shenron** (`/home/ubuntu/vps/apps/shenron`) | 1 — couverture entrée | 2, 4 | Bot Discord déjà majoritairement Bun. Tout résidu détecté = vrai trou de couverture, pas du bruit. |
| **gemini-cli** (`tests/targets/gemini-cli`, gitignored) | 2 — cross-compilation sortie | 5, 6 | CLI Node 100 % monorepo (`packages/*`). Cible single-file exe `bun build --compile --target=bun-{linux,windows}-x64`. Mesure le ratio APIs réécrites vs résidus manuels. |

Les baselines vivent sous `tests/targets/<cible>/baseline.json` (commitées). Les clones lourds (`gemini-cli/`) restent gitignored — régénérables via `tests/targets/refresh.sh`.

## Baseline initiale (n2b 0.5.0, 2026-05-15)

### shenron — Pilier 1
- `files_scanned` : **12** (sur ~146 .ts/.tsx) — `files_scanned` ne compte que les fichiers **avec** findings.
- `findings_total` : **39** (0 errors, 6 warns, 33 infos).
- Top : `api/process-env` (13), `api/performance-now` (9), `api/child-process-spawn` (5).
- **Critère de succès Pilier 1** : après Phase 2 (AST-aware) + Phase 4 (sync-coverage), tout finding restant doit être *justifié* (résidu fonctionnel impossible à réécrire) — pas un trou de règle.

### gemini-cli — Pilier 2
- `files_scanned` : **1308**.
- `findings_total` : **3239**.
- Top : `imports/bun-native` (1075), `api/fs-writeFileSync` (335), `api/fs-existsSync` (331), `api/chalk-call` (145), `api/execSync` (139), `cli/npm-run` (116).
- **Critère de succès Pilier 2** :
  1. `n2b --aggressive --migrate` réécrit ≥ 95 % des findings sans intervention.
  2. `bun build --compile --target=bun-linux-x64 --outfile=gemini-cli-linux` produit un binaire qui répond à `--help` en < 200 ms cold-start.
  3. Idem `--target=bun-windows-x64`.
  4. Tailles binaires linux/windows reportées dans la report card Phase 5 (`.n2b/report.json`).

## Workflow

```bash
# (re)bootstrap des cibles
bash tests/targets/refresh.sh

# baseline
n2b /home/ubuntu/vps/apps/shenron --report=json > tests/targets/shenron/baseline.json
n2b tests/targets/gemini-cli       --report=json > tests/targets/gemini-cli-out/baseline.json

# diff de progression (une fois Phase 2/4/5 livrées)
n2b tests/targets/gemini-cli --aggressive --report=json | jq '.findings_total'
```

## Anti-régression

Chaque phase qui touche aux scanners/règles **doit** régénérer les deux baselines et
diff. Drift accepté = nouvelle règle qui ajoute des findings (gain de couverture) ;
drift refusé = règle existante qui en perd (faux négatif introduit).

À automatiser dans `tests/compare-baseline.sh` une fois Phase 1 livrée.
