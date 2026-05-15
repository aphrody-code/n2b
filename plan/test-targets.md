# Cibles de test du refactor — fixtures réelles

Deux fixtures servent de banc d'essai canonique pour valider la progression des deux
piliers du refactor (cf. [REFACTOR_PLAN.md](../REFACTOR_PLAN.md)).

| Cible | Pilier | Phases | Profil |
|---|---|---|---|
| **bun-full** (`tests/targets/bun-full/`) | 1 — couverture entrée | 2, 3, 4 | Fixture canonique tout-Bun + Material Design 3 (motion/elevation/tokens). Tout résidu détecté = vrai bonus / faux positif à corriger. |
| **gemini-cli** (`tests/targets/gemini-cli`, gitignored) | 2 — cross-compilation sortie | 5, 6 | CLI Node 100 % monorepo (`packages/*`). Cible single-file exe `bun build --compile --target=bun-{linux,windows}-x64`. Mesure le ratio APIs réécrites vs résidus manuels. |

> shenron retiré comme cible (2026-05-15) — remplacé par `bun-full/` qui couvre
> exhaustivement les surfaces Bun (Bun.serve, bun:sqlite, bun:ffi, bun:test, HMR,
> bundler, macros, single-file exe) + un design system M3 fidèle à m3.material.io.

Les baselines vivent sous `tests/targets/<cible>/baseline.json` (commitées). Les clones
lourds (`gemini-cli/`) restent gitignored — régénérables via `tests/targets/refresh.sh`.

## Fixture bun-full — Pilier 1

**Fichier unique** : `tests/targets/bun-full/app.tsx` (~430 lignes, single-file).

Couvre :
- **HTTP/WS** : `Bun.serve` full-stack (typed routes, WebSocket upgrade, error)
- **JSX natif** : composants M3 (`FilledButton`, `Card`, `App`) — runtime `react-jsx`
- **CSS natif** : `import "./styles/m3-tokens.css"` (Bun bundler natif)
- **Hot reload** : `Bun.serve { development.hmr: true }` + `import.meta.hot`
- **Bundler** : `Bun.build` avec plugins, minifier, loader, splitting, define, naming
- **Macros** : `import { buildId } from "./macros/build-id.ts" with { type: "macro" }`
- **Test runner** : `bun:test` describe/it/expect/mock.module
- **DB** : `bun:sqlite` (file-backed, WAL, `using db = ...`, `.iterate()`)
- **SQL/Cache** : `Bun.SQL` tagged templates, `Bun.RedisClient`
- **Shell** : `Bun.spawn`, `Bun.$\`cmd\`` (parametrized, no injection)
- **Files** : `Bun.file`, `Bun.write` (lazy, atomic, sendfile/copy_file_range)
- **Crypto** : `Bun.password` (argon2id), `Bun.CryptoHasher`, `Bun.hash.xxHash3`
- **Misc** : `Bun.Glob`, `Bun.cron`, `Bun.S3Client`, `Bun.CookieMap`
- **HTML** : `HTMLRewriter` streaming
- **FFI** : `bun:ffi` `dlopen` + `cc()` inline
- **Single-file executable** : commentaires `bun build --compile --target=bun-{linux,windows,darwin-arm64}`
- **Design tokens M3** : `MotionDuration` (short1..extraLong4), `MotionEasing` (standard, emphasized, decelerate, accelerate — bezier curves de m3.material.io), `ColorRole`, `Elevation`, `Shape`

**Critère de succès** :
- Aucun finding `imports/bun-native` (déjà tout-Bun).
- Aucun finding `api/*` réel — les ~2 findings actuels (`api/process-env` dans le `define` du bundler, `api/child-process-spawn` dans le `Bun.spawn(["git", ...])`) sont dans le bruit du pattern regex, à filtrer en Phase 7.
- Findings `api/node-*` granulaires (Phase 4) sur les sous-APIs documentées comme manquantes — *aucun ici*, la fixture est volontairement clean.

## Fixture gemini-cli — Pilier 2

- `files_scanned` : **1303** (post-Phase 2 AST filter).
- `findings_total` : **3232** (post-Phase 2 — 7 faux positifs supprimés).
- Top : `imports/bun-native`, `api/fs-writeFileSync`, `api/fs-existsSync`, `api/chalk-call`, `api/execSync`, `cli/npm-run`.

**Critère de succès Pilier 2** :
1. `n2b --aggressive --migrate` réécrit ≥ 95 % des findings sans intervention.
2. `bun build --compile --target=bun-linux-x64 --outfile=gemini-cli-linux` produit un binaire qui répond à `--help` en < 200 ms cold-start.
3. Idem `--target=bun-windows-x64`.
4. Tailles binaires linux/windows reportées dans la report card Phase 5 (`.n2b/report.json`).

## Workflow

```bash
# (re)bootstrap des cibles
bash tests/targets/refresh.sh

# baseline
n2b tests/targets/bun-full           --report=json > tests/targets/bun-full/baseline.json
n2b tests/targets/gemini-cli         --report=json > tests/targets/gemini-cli-out/baseline.json

# diff de progression (une fois Phase 5 livrée)
n2b tests/targets/gemini-cli --aggressive --report=json | jq '.findings_total'
```

## Anti-régression

Chaque phase qui touche aux scanners/règles **doit** régénérer les deux baselines et
diff. Drift accepté = nouvelle règle qui ajoute des findings (gain de couverture) ;
drift refusé = règle existante qui en perd (faux négatif introduit).
