# Phase 7 — Garde-fous & documentation

> Verrouille les acquis : contract tests étendus, CI anti-drift, doc à jour, baselines
> finales. Optionnellement, découpe le crate `n2b-cli` (PS8 volet 2).
>
> **Dépend de :** Phases 2, 3, 5, 6. **Bloque :** rien (phase terminale).

## Objectif

Rendre le refactor **irréversible par accident** : tout retour en arrière (drift de
registre, Rule ID supprimé, schéma cassé) doit faire échouer la CI.

## Travaux

### 7.1 — `contract.rs` étendu

**Fichier.** `crates/n2b-cli/tests/contract.rs`.

Ajouter, aux 9 tests existants :
- **un test par catégorie de Rule ID** : `imports/*`, `api/*`, `cli/*`, `next/*`,
  `globals/*`, et les IDs des scanners (`pkg/*`, `tsconfig/*`, `npmrc/*`…). Chaque test
  vérifie qu'au moins une règle de la catégorie est exposée par `n2b rules` et produit un
  finding bien formé.
- **un test de rétro-compat schéma** : un `Finding` sérialisé *sans* `compat` valide
  toujours contre le schéma (garantit que Phase 3 n'a pas rendu `compat` requis par
  accident).
- **un test report card** : `n2b --migrate --report=json` sur `test/fixture/` expose un
  `report_card` avec `auto_migratable_pct ∈ [0,1]` et un `manual_residue` array.

### 7.2 — CI anti-drift

**Fichier.** workflow CI (`.github/workflows/*` — ou équivalent).

Ajouter :
- `cargo xtask sync-coverage --check` — échoue si le registre diverge de l'upstream
  snapshot (cf. [phase-4](phase-4-couverture.md) §risques : utilise
  `registry/.upstream-snapshot.toml` pour tourner hors-ligne).
- `bun run codegen:schema:check` — échoue si `schema.rs` est stale.
- `cargo test --workspace` + `bash tests/compare-baseline.sh` + `cargo clippy -D warnings`
  + `cargo fmt --check` — déjà en place, confirmer.

### 7.3 — Régénérer les baselines finales

**Fichiers.** `tests/snapshots/baseline/`, `tests/rpb-dashboard-baseline/`.

Régénération finale propre, après que toutes les phases ont stabilisé la sortie. Une
seule régénération « officielle » de fin de refactor, commit dédié, message qui
récapitule *pourquoi* chaque format a changé depuis le début du refactor.

### 7.4 — Documentation

**Fichiers.** `CLAUDE.md`, `CHANGELOG.md`, `docs/README.md`, `plan/README.md`.

- **`CLAUDE.md`** : réécrire la section architecture pour le registre data-driven
  (PS7 volet 2). Décrire `crates/n2b-registry/`, le rôle des `.toml`, `xtask
  sync-coverage`, le fait que « pour ajouter une règle, éditer un `.toml` » remplace
  « ajouter un scanner ou un regex ».
- **`CHANGELOG.md`** : entrée de version récapitulative — registre data-driven, champ
  `compat`, report card, nouveaux scanners, AST-first. Lister les breakings (PS4 fix,
  sévérités 🔴→error) et les additifs.
- **`docs/README.md`** : ajouter la procédure `xtask sync-coverage` à la section
  régénération.
- **`plan/README.md`** : passer le status board à ✅ sur toutes les phases, avec les
  hashes de commit.

### 7.5 — Découpe `n2b-cli` (PS8 volet 2 — OPTIONNEL)

**Hors chemin critique.** À ne faire que si le temps le permet.

`n2b-cli` est un crate plat de ~12 000 lignes. Deux options :
- **a)** réorganiser en modules internes (`commands/scaffold/{rust,app,bin,win32,linux,wasm}.rs`) ;
- **b)** extraire un sous-crate `n2b-scaffold` regroupant `rust_cmd`, `app_cmd`,
  `bin_cmd*`, `win32_cmd*`, `linux_cmd`, `wasm_cmd`, `wasm_spec/` (~8 000 lignes).

Option **b** réduit le temps de compilation incrémentale du CLI. Mais c'est un refactor
mécanique sans valeur fonctionnelle — **ne bloque pas la sortie « parfaite »**, à
trancher selon le budget restant.

## Critères d'acceptation

- `cargo test --workspace` vert — contract tests étendus inclus.
- `cargo xtask sync-coverage --check` — vert en CI (0 drift).
- `bun run codegen:schema:check` — vert.
- `bash tests/compare-baseline.sh` — vert.
- `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --check` — vert.
- **Les 4 critères de « parfait »** ([README.md](../README.md)) sont vérifiables et vérifiés :
  1. `sync-coverage --check` = 0 trou ;
  2. toute entrée 🟢/🟡 a une `rewrite` non-`manual` (ou `codemod_hint` justifié) ;
  3. report card d'un repo témoin = résidu manuel explicite ;
  4. proptest homonyme = 0 faux positif.
- `CLAUDE.md` reflète fidèlement le code (plus aucun PS7).

## Commits attendus

```
test(n2b-cli): contract — un test par catégorie de Rule ID + rétro-compat schéma + report card
ci: cargo xtask sync-coverage --check + codegen:schema:check anti-drift
chore(baselines): régénération finale du refactor — récap des changements de sortie
docs(claude): réécrit la section architecture pour le registre data-driven
docs(changelog): version récapitulative — registre, compat, report card, AST-first
refactor(n2b-cli): extrait n2b-scaffold   # OPTIONNEL — si budget le permet
```

## Risques

| Risque | Mitigation |
|---|---|
| `sync-coverage --check` en CI dépend de `upstream/` (gitignoré) | snapshot commité `registry/.upstream-snapshot.toml` ; le `--check` compare contre lui hors-ligne, un job périodique rafraîchit le snapshot |
| Régénération finale masque une régression introduite en cours de route | chaque phase a déjà régénéré et justifié ses baselines ; la régénération finale ne doit montrer **aucun** diff inattendu |
| La découpe `n2b-cli` (7.5) introduit un breaking de chemins de module | optionnelle, isolée en dernier commit ; si elle casse quoi que ce soit → la retirer, elle n'est pas nécessaire au « parfait » |
