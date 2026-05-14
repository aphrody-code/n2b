# 01 — Problèmes structurels (PS1→PS8)

> Le plan original identifiait PS1→PS5. L'audit du code a révélé **3 problèmes
> supplémentaires** (PS6→PS8). Tous doivent être corrigés en **Phase 0** avant d'ajouter
> de la couverture — sinon on empile sur du sable.
>
> Chaque problème : symptôme → preuve `fichier:ligne` → remède → phase de correction.

---

## PS1 — Détection non *import-aware*

**Symptôme.** `bun_apis.rs` matche des **identifiants nus** par regex. `marked(`,
`which(`, `v4()`, `exec(` déclenchent un finding même quand l'identifiant est une
fonction locale homonyme — aucun lien au site d'import.

**Preuve.**
- `crates/n2b-rules/src/bun_apis.rs:53-591` — `RULES: Vec<ApiRule>`, chaque règle porte
  une `regex` appliquée au texte brut.
- `bun_apis.rs:614-617` et `:714-724` — `is_member_exec_call` est un *hack* ad-hoc pour
  rattraper *un seul* faux positif (`regex.exec()`). C'est le symptôme : on patche au
  cas par cas faute de modèle de binding.
- À l'inverse, `imports_ast.rs` (174 l.) *sait déjà* résoudre les imports via `oxc` —
  mais seul `node_imports.rs:680` l'utilise. `bun_apis.rs` ignore l'AST.

**Remède.** Phase 2 — passer `bun_apis` en pipeline AST : chaque call-site est corrélé à
son binding d'import. Une règle `api/marked-call` ne matche que si `marked` provient
réellement d'un `import … from "marked"` ou `require("marked")`. Les regex ne survivent
que pour le non-JS (`.sh`, `Dockerfile`, configs).

**Phase : 2.**

---

## PS2 — Duplication de la logique d'édition

**Symptôme.** Le tri `index/len` + filtre d'overlap + `replace_range` est copié-collé
**à l'identique** dans deux fichiers. Le struct `Edit` aussi.

**Preuve.**
- `crates/n2b-rules/src/bun_apis.rs:671-690` — bloc `edits.sort_by(...)` → `overlaps_prev`
  → `replace_range`.
- `crates/n2b-rules/src/node_imports.rs:745-759` — **le même bloc, octet pour octet.**
- `bun_apis.rs:596-601` ⟂ `node_imports.rs:670-675` — struct `Edit { index, len,
  replacement }` dupliqué.

**Remède.** Phase 0 — extraire dans `n2b-util` :

```rust
// crates/n2b-util/src/edits.rs
pub struct Edit { pub index: usize, pub len: usize, pub replacement: String }

/// Applique des éditions en résolvant les overlaps (garde la plus longue à index égal).
pub fn apply_edits(source: &str, mut edits: Vec<Edit>) -> String { /* logique unique */ }
```

`bun_apis.rs` et `node_imports.rs` appellent `n2b_util::edits::apply_edits`.

**Phase : 0.**

---

## PS3 — Deux sources pour les mêmes deps

**Symptôme.** Les paquets npm shimmables sont décrits **deux fois**, dans deux structures
Rust distinctes, avec risque de divergence.

**Preuve.**
- `crates/n2b-rules/src/node_imports.rs:86-661` — `BUN_REPLACEMENTS` décrit `pako`,
  `bcrypt`, `marked`, `which`, `glob`… (specifier → replacement).
- `crates/n2b-rules/src/bun_apis.rs:53-591` — les règles `api/pako-gzip`,
  `api/bcrypt-hash`, `api/marked-call`, `api/which-call`, `api/glob`… décrivent **les
  mêmes paquets** sous l'angle « appel de méthode ».
- Aucune table ne référence l'autre. Ajouter le support d'une dep = éditer deux endroits.

**Remède.** Phase 1 — registre data-driven. `registry/packages.toml` devient la source
unique : chaque dep porte à la fois sa stratégie d'import (`drop`/`rewrite`/`shim`) et la
liste de ses APIs à réécrire. `BUN_REPLACEMENTS` et les `ApiRule` deviennent des **vues
générées** du registre, pas des sources.

**Phase : 1.**

---

## PS4 — Bug commentaires de `cli_commands.rs`

**Symptôme.** Une ligne commentée (`# npm install`) ne génère **aucun finding** mais est
quand même **réécrite** en `# bun install` par `--fix`. Incohérence détection ⟂ édition.

**Preuve.** `crates/n2b-rules/src/cli_commands.rs` :
- `:334-347` — les findings filtrent les lignes commentées via `COMMENT_PREFIX`
  (`^\s*(#|//)`, défini `:315-317`).
- `:372-378` — la réécriture utilise `rule.re.replace_all(&out, rule.replace)` **global,
  sans filtre**. Le code l'admet en commentaire : *« on reproduit le comportement
  historique qui lui remplaçait globalement »*.

**Remède.** Phase 0 — la réécriture doit partager le filtre de commentaires des findings.
Réécrire `apply_cli_rules` pour produire des `Edit` ciblés (mêmes positions que les
findings) au lieu d'un `replace_all` global, puis passer par `apply_edits` (PS2).
**Attention contrat** : ce changement modifie la sortie `--fix` sur les fichiers à
commandes commentées → justifier le breaking et régénérer les baselines concernées.

**Phase : 0.**

---

## PS5 — Constantes magiques

**Symptôme.** Des nombres en dur, non nommés, non documentés, encodent des heuristiques.

**Preuve.**
- `crates/n2b-rules/src/bun_apis.rs:700-709` — `looks_like_dir_context` : fenêtre de
  `600` octets en dur (`let end = (pos + 600).min(source.len());`) pour décider si un
  `fs.existsSync` vise un dossier.

**Remède.** Phase 0 — soit constante nommée + documentée
(`const DIR_CONTEXT_WINDOW_BYTES: usize = 600; // heuristique : portée d'un bloc fs.mkdir typique`),
soit suppression si le passage AST (Phase 2) rend l'heuristique inutile (un AST sait si
le même symbole est passé à `fs.mkdir`). Décision : nommer en Phase 0, réévaluer la
suppression en Phase 2.

**Phase : 0 (nommage), 2 (suppression éventuelle).**

---

## PS6 — Codegen schéma cassé *(nouveau — non identifié dans REFACTOR_PLAN.md)*

**Symptôme.** La chaîne `schema/v2.json` → types Rust/TS est **non-fonctionnelle**. Le
script de codegen n'existe pas. CLAUDE.md décrit un état fictif.

**Preuve.**
- `scripts/generate-schema-types.ts` — **inexistant**. Le dossier `scripts/` n'existe pas.
- `crates/n2b-core/src/schema.rs` (référencé par CLAUDE.md) — **inexistant**. Le vrai
  fichier généré est `crates/n2b-types/src/schema.rs` (en-tête `@generated`, mentionne le
  même script fantôme).
- `packages/n2b/src/schema.ts` — **inexistant**. `packages/n2b/src/` ne contient que
  `cli.ts` et `index.ts`.
- `package.json` racine — scripts `codegen:schema` et `codegen:schema:check` pointent
  vers le script fantôme. Aucune dépendance `cargo-typify` / `json-schema-to-typescript`
  dans le workspace.

**Cause.** Le refactor Turborepo v0.5.0 a déplacé `schema.rs` de `n2b-core` vers
`n2b-types` sans recréer le script de codegen ni mettre à jour `package.json`/CLAUDE.md.

**Remède.** Phase 0 — recréer `scripts/generate-schema-types.ts` ciblant
`crates/n2b-types/src/schema.rs` (via `cargo-typify`) et, si un type TS est requis,
`packages/n2b-types/src/index.ts` (via `json-schema-to-typescript`). Corriger les
chemins dans `package.json`. **Bloquant pour Phase 3** (qui modifie le schéma — sans
codegen fonctionnel, impossible de régénérer les types proprement).

**Phase : 0.**

---

## PS7 — CLAUDE.md désynchronisé *(nouveau)*

**Symptôme.** Le `CLAUDE.md` projet contient des affirmations fausses qui induisent en
erreur tout futur travail (humain ou agent).

**Preuve.** Divergences relevées par l'audit :
- « `crates/n2b-core/src/schema.rs` via `cargo-typify` » → le fichier est dans `n2b-types`.
- « `packages/n2b/src/schema.ts` via `json-schema-to-typescript` » → ce fichier n'existe pas.
- « Exit codes `0`/`1`/`2` … `crates/n2b-cli/src/cli/dispatch.rs` » → la logique réelle
  est dans `crates/n2b-cli/src/commands/scan.rs:52-63`.
- `bun run codegen:schema` présenté comme fonctionnel → cf. PS6.

**Remède.** Phase 0 — corriger ces 4 points dans `CLAUDE.md`. Phase 7 — réécrire la
section architecture pour refléter le registre data-driven. Le `CLAUDE.md` doit rester
un miroir fidèle du code, sinon il devient un PS3 documentaire (source de vérité
divergente).

**Phase : 0 (corrections factuelles), 7 (section registre).**

---

## PS8 — Cruft repo & crate CLI monolithique *(nouveau)*

**Symptôme.** Plusieurs scories nuisent à la lisibilité et à l'hygiène du dépôt.

**Preuve.**
- `node_modules/` **commité à la racine** (présent dans `ls`, devrait être gitignoré).
- `package.json` racine — script `install:cli:ts` référence `@n2b/cli` /
  `packages/n2b-cli/dist/node2bun` : ce package **n'existe pas** (seul `crates/n2b-cli`
  existe, c'est du Rust).
- `Cargo.toml:1-12` — en-tête commentaire décrit un layout `rust/`+`native/` **obsolète**
  (le workspace est en `crates/*` depuis v0.5.0).
- `crates/n2b-cli` — crate **plat de ~12 000 lignes** : 13 subcommands mélangés, des
  fichiers `*_cmd*.rs` au même niveau que `cli/` et `commands/`. `rust_cmd.rs` fait 1431
  lignes, `wasm_spec/codegen.rs` 1673.

**Remède.**
- Phase 0 — gitignorer `node_modules/`, le retirer du suivi (`git rm -r --cached`) ;
  corriger/supprimer le script `install:cli:ts` ; réécrire l'en-tête `Cargo.toml`.
- Phase 7 (optionnel, non bloquant) — découper `n2b-cli` : un module par groupe de
  subcommands, ou extraire les scaffolders (`rust_cmd`, `app_cmd`, `bin_cmd`, `win32`,
  `linux`, `wasm`) dans un sous-crate `n2b-scaffold`. **Hors chemin critique** — à ne
  faire que si le temps le permet, ne bloque aucune autre phase.

**Phase : 0 (hygiène), 7 (découpe optionnelle).**

---

## Récapitulatif — quelle phase corrige quoi

| Problème | Gravité | Phase(s) | Bloquant pour |
|---|---|---|---|
| PS1 — pas import-aware | haute | 2 | qualité pilier 1 |
| PS2 — duplication édition | moyenne | 0 | propreté Phase 1 |
| PS3 — deux sources deps | haute | 1 | tout le registre |
| PS4 — bug commentaires CLI | moyenne | 0 | cohérence `--fix` |
| PS5 — constantes magiques | basse | 0 (+2) | lisibilité |
| PS6 — codegen cassé | **haute** | 0 | **Phase 3** |
| PS7 — CLAUDE.md désynchro | moyenne | 0 (+7) | fiabilité doc |
| PS8 — cruft repo | basse | 0 (+7) | hygiène |

**Phase 0 corrige PS2, PS4, PS5, PS6, PS7, PS8.** PS1 et PS3 sont structurels et
adressés par les Phases 2 et 1 respectivement (ils *définissent* ces phases).
