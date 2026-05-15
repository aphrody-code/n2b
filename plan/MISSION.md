# MISSION — refactor n2b complet, Phases 0 → 7

Tu es Claude Code, lancé en mode headless dans une tmux pour exécuter le refactor n2b de bout en bout sans interruption. Cette mission est lue à chaque relance (en cas de redémarrage / contexte plein / reprise).

**Mode** : autonomie maximale (cf. `~/.claude/CLAUDE.md`). Pas d'`AskUserQuestion`, pas de demande de confirmation, exécute jusqu'au bout. Garde-fous extrêmes uniquement (cf. global CLAUDE.md).

**Source de vérité** : `/home/ubuntu/n2b/plan/`. Lis `plan/README.md` d'abord, puis chaque `plan/phases/phase-N-*.md`. Données de couverture dans `plan/coverage/`.

## État au lancement initial (2026-05-15)

Toutes les phases sont à faire (`Status board` de `plan/README.md` toutes ☐). Le projet `CLAUDE.md` vient d'être audité et corrigé sur 6 points factuels (le diff est dans `git log`). L'agent Plan a déjà été lancé et a remonté 3 écarts à intégrer (voir section suivante).

## Corrections issues de l'audit Plan (à appliquer en Phase 0)

1. **PS8 — `node_modules/` faussement « commité »** : déjà gitignoré (`.gitignore` ligne 1, pattern global), `git ls-files node_modules` = 0. L'étape `git rm -r --cached node_modules` est un **no-op**, la sauter. **Ne PAS modifier `.gitignore`** pour node_modules. Les autres volets PS8 (script fantôme `install:cli:ts` dans `package.json`, en-tête `Cargo.toml` décrivant un layout `rust/`+`native/` obsolète) restent valides.

2. **PS6 — chaîne codegen ambiguë** : avant de coder `scripts/generate-schema-types.ts`, **spike obligatoire** :
   ```
   bunx cargo-typify schema/v2.json > /tmp/schema-regen.rs
   diff /tmp/schema-regen.rs crates/n2b-types/src/schema.rs
   ```
   - Si identique → `cargo-typify` est l'outil, écrire le script avec lui.
   - Si différent → soit `schema.rs` est stale (commit de régénération séparé EN PREMIER, R8), soit la chaîne est `schemars`+`ts-rs` depuis les types Rust (`crates/n2b-types/Cargo.toml` a déjà ces deux deps). Trancher avant d'écrire le script.

3. **Numéros de ligne du plan décalés de +13** : les commits d'en-tête Apache-2.0 sur 83 fichiers ont décalé toutes les positions citées dans `plan/01-problemes-structurels.md` et `plan/phases/*.md`. **Chercher les symboles**, pas les lignes : `BUN_REPLACEMENTS`, `is_member_exec_call`, `looks_like_dir_context`, `replace_all`, `apply_cli_rules`, `RULES`, `COMMENT_PREFIX`, struct `Edit`.

## Angles morts à connaître

- **`rpb-dashboard` est absent de cette machine** → `tests/compare-baseline.sh` SKIP la moitié rpb (5 comparaisons). `RPB_ROOT=/nonexistent` aussi en CI. Conséquence pour PS4 : la réécriture de la ligne `// npm install --save-dev prisma dotenv` (`tests/rpb-dashboard-baseline/scan.json` au ~ligne 2233) disparaîtra → `scan.*` rpb deviendra stale localement sans déclencher d'échec. **Documenter dans le commit PS4** que `tests/rpb-dashboard-baseline/` est à régénérer au prochain accès à rpb-dashboard. **Vérifier impérativement** que `cargo build --workspace` passe encore après PS4 — `crates/n2b-cli/src/schema_test.rs` fait `include_str!("../../../tests/rpb-dashboard-baseline/scan.json")` + `serde_json::from_str::<N2bReport>`, une baseline rpb cassée fait échouer la **compilation du CLI**.
- **Filet de sécurité triple** (pas double comme dit le plan v1) : `compare-baseline.sh` + `crates/n2b-cli/tests/contract.rs` + `crates/n2b-cli/src/schema_test.rs` (3ᵉ : `include_str!` des deux baselines + round-trip `N2bReport`).
- **Deux types `Finding` distincts coexistent** : `n2b_types::types::Finding` (runtime, `make_finding`, ~10 champs) ≠ `Finding` dans `crates/n2b-types/src/schema.rs` (sérialisé, ~15 champs : `category`, `confidence`, `start_byte`, `end_byte`, `docs_url`, `context`…). Couche de conversion probable dans `n2b-report`. Phase 3 (ajout de `compat`) doit toucher **les deux structs + la conversion + `schema/v2.json` + régénérer `schema.rs`**, pas juste le JSON.

## Protocole d'exécution

### Pré-vol au démarrage de chaque session

1. `cd /home/ubuntu/n2b && git status --short` — propre (ou WIP identifiés dans STATE.md).
2. `cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all -- --check && bash tests/compare-baseline.sh` — tout vert. Si non, diagnostiquer la racine, fixer, puis continuer.
3. Lire `plan/STATE.md` s'il existe. Reprendre où l'état dit (phase, sous-étape, fichiers WIP). Sinon démarrer Phase 0.

### Pendant chaque phase

- Suivre les sous-étapes du `plan/phases/phase-N-*.md` correspondant.
- **Un commit par sous-étape**, format conventionnel (`feat|fix|chore|refactor|docs(scope): message`), pas d'emoji, pas de signature Claude (cf. global CLAUDE.md).
- **Après chaque commit** : green-gate complet (build + test + clippy + fmt + baseline). Si rouge, **fixer avant le commit suivant** — jamais empiler des commits rouges. Le diff est ton ami pour repérer ce qui a changé.
- Si une sous-étape exige une action destructive non-listée dans le plan (drop DB, push --force, suppression non-réversible) : **SKIP**, logguer dans STATE.md, continuer.

### Après chaque phase

1. Vérifier un par un les critères § « Acceptation » de `plan/phases/phase-N-*.md`.
2. Mettre à jour `Status board` de `plan/README.md` (☐ → ✅ + hash du dernier commit de la phase).
3. Mettre à jour `plan/STATE.md` (créer s'il n'existe pas) :
   ```markdown
   # Refactor State

   Last update: YYYY-MM-DD HH:MM

   ## Done
   - [x] Phase 0 — Socle propre (PS1→PS8) — commit abc1234
   - [x] Phase 1 — Registre data-driven — commit def5678
   ...

   ## In progress
   Phase N — sous-étape M : <description courte>
   Fichiers modifiés non commités : <liste>
   Prochaine action exacte : <une phrase>

   ## Decisions
   - PS6 : cargo-typify retenu (spike OK le YYYY-MM-DD)
   - PS4 : baseline rpb stale, à régénérer au prochain accès à /home/ubuntu/rpb-dashboard
   ...

   ## Deviations from plan
   - <ce qui a divergé et pourquoi>
   ```
4. Commit `chore(plan): state board after phase N`.

### Si une phase échoue

- Diagnostiquer la cause racine. **Jamais** bypasser un hook (`--no-verify`), **jamais** force-push, **jamais** `git reset --hard` sur du travail non-identifié.
- Si recoverable : fix + commit + reprise.
- Si non-recoverable dans le scope : `git reset --soft` aux commits réussis de la phase, log dans STATE.md ce qui bloque, **passer à la phase suivante du DAG uniquement si elle ne dépend pas de la phase échouée** (DAG : `0 → 1 → {2, 3, 4}`, `4 → 5`, `3 → 6`, `{2,3,5,6} → 7`). Sinon stop propre + handoff dans STATE.md.

### Si le contexte se réduit

Quand tu sens que le contexte va dépasser : écrire un handoff complet dans `plan/STATE.md` (phase courante, sous-étape, fichiers touchés non commités, prochaine action exacte), commit WIP si propre, sinon `git stash` avec message explicite, puis terminer. Une session ultérieure reprendra depuis STATE.md via la commande de relance (cf. fin de ce fichier).

## Garde-fous (rappelés)

Du global CLAUDE.md, JAMAIS sans instruction explicite contextualisée :
- `git push --force` sur main/master
- `--no-verify`, `--no-gpg-sign`
- Suppression de données utilisateur réelles
- `git reset --hard` qui détruit du travail non identifié
- Commiter des secrets
- Modifier silencieusement le **contrat externe gelé** (rule IDs, schéma JSON v2, exit codes, flags CLI, ABI cdylib v1) — toute modification doit être assumée (commit dédié + raison + régénération baselines).
- Toucher à `tests/rpb-dashboard-baseline/` **sans** documenter la régénération à venir dans le commit.

Le seul `git push` autorisé : pousser une fois à la fin pour déclencher la CI GitHub (vérification cross-OS). PAS de push intermédiaire entre phases sauf demande explicite.

## Critères de sortie

Quand les 8 phases sont vertes et committées :
1. `plan/README.md` Status board : toutes phases ✅ avec hash.
2. `plan/STATE.md` : « FINI le YYYY-MM-DD », hash du dernier commit.
3. `git log --oneline` propre, un commit par sous-étape.
4. `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all -- --check && bash tests/compare-baseline.sh && bun run codegen:schema:check` tout vert.
5. Critères § « Critère de parfait » de `plan/README.md` remplis (les 4 : sync-coverage --check, rewrite non-manual sur 🟢/🟡, report card explicite, zéro faux positif proptest).
6. `git push origin main` une fois (CI cross-OS doit aussi être verte).
7. Commit final `docs(plan): refactor complet — Phases 0 → 7 vertes`.

## Commande de relance (en cas de reprise depuis STATE.md)

```bash
tmux attach -t n2b 2>/dev/null || tmux new -d -s n2b \
  "cd /home/ubuntu/n2b && claude -p \"$(cat /home/ubuntu/n2b/plan/MISSION.md)\" --permission-mode bypassPermissions 2>&1 | tee -a /tmp/n2b-refactor.log"
```

Lance-toi. Pré-vol → STATE.md ou Phase 0 → exécution. Pas de question, pas de pause.
