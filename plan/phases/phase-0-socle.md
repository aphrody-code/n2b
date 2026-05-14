# Phase 0 — Socle propre

> Corrige **PS2, PS4, PS5, PS6, PS7, PS8**. Aucune nouvelle couverture. Objectif : un
> socle assaini avant d'empiler le registre dessus.
>
> **Dépend de :** rien. **Bloque :** toutes les autres phases.

## Objectif

Rendre le dépôt cohérent et le codegen fonctionnel, sans changer la sortie du scan
(sauf le breaking justifié de PS4). C'est la phase la moins « visible » et la plus
**non négociable**.

## Travaux

### 0.1 — `apply_edits` partagé (PS2)

**Fichiers.** Nouveau : `crates/n2b-util/src/edits.rs`. Modifiés :
`crates/n2b-util/src/lib.rs`, `crates/n2b-rules/src/bun_apis.rs`,
`crates/n2b-rules/src/node_imports.rs`.

**Changement.** Extraire le struct `Edit` et la logique tri+overlap+`replace_range` —
aujourd'hui dupliquée à l'identique en `bun_apis.rs:671-690` / `:596-601` et
`node_imports.rs:745-759` / `:670-675`.

```rust
// crates/n2b-util/src/edits.rs
#[derive(Debug, Clone)]
pub struct Edit {
    pub index: usize,
    pub len: usize,
    pub replacement: String,
}

/// Applique des éditions en résolvant les chevauchements.
/// À index égal, garde l'édition la plus longue. Édite de la fin vers le début
/// pour ne pas invalider les offsets.
pub fn apply_edits(source: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by(|a, b| a.index.cmp(&b.index).then(b.len.cmp(&a.len)));
    let mut kept: Vec<Edit> = Vec::with_capacity(edits.len());
    for e in edits {
        let overlaps = kept.last().map(|p| p.index + p.len > e.index).unwrap_or(false);
        if !overlaps {
            kept.push(e);
        }
    }
    kept.sort_unstable_by_key(|e| std::cmp::Reverse(e.index));
    let mut out = source.to_string();
    for e in kept {
        out.replace_range(e.index..e.index + e.len, &e.replacement);
    }
    out
}
```

`bun_apis.rs` et `node_imports.rs` suppriment leur `Edit` local et leur bloc, et
appellent `n2b_util::edits::apply_edits`.

**Test.** Nouveau `crates/n2b-util/tests/edits.rs` : cas overlap, cas index égal, cas
vide, cas édition en fin de fichier.

### 0.2 — Bug commentaires `cli_commands.rs` (PS4)

**Fichier.** `crates/n2b-rules/src/cli_commands.rs:321-382` (`apply_cli_rules`).

**Changement.** Remplacer le `rule.re.replace_all(&out, rule.replace)` global
(`:372-378`) par une collecte de `Edit` filtrée comme les findings (`:334-347`) — la
détection et l'édition partagent le **même** filtre `COMMENT_PREFIX`. Puis
`apply_edits` (0.1).

**⚠️ Breaking contrat.** Ce changement modifie la sortie `--fix` pour les fichiers
contenant des commandes commentées. C'est un **fix de bug justifié**, pas un breaking
silencieux :
- documenter dans `CHANGELOG.md` (« fix: `--fix` ne réécrit plus les lignes commentées
  dans les scanners shell ») ;
- vérifier si `test/fixture/` ou `rpb-dashboard` contiennent des commandes commentées ;
  si oui → régénérer les baselines concernées et le justifier dans le message de commit.

**Test.** Cas dans `tests/` : fichier shell avec `# npm install` + `npm install` →
seule la 2ᵉ ligne réécrite.

### 0.3 — Constantes magiques (PS5)

**Fichier.** `crates/n2b-rules/src/bun_apis.rs:700-709` (`looks_like_dir_context`).

**Changement.**
```rust
/// Fenêtre de recherche pour décider si un `fs.existsSync` vise un dossier.
/// Heuristique : portée d'un bloc `fs.mkdir` typiquement proche du test d'existence.
/// TODO(phase-2) : supprimable une fois le matching AST en place — l'AST sait si
/// le même symbole est passé à fs.mkdir sans fenêtre arbitraire.
const DIR_CONTEXT_WINDOW_BYTES: usize = 600;
```
Remplacer le `600` littéral par la constante. Le `TODO` relie explicitement à Phase 2.

### 0.4 — Réparer le codegen schéma (PS6)

**Fichiers.** Nouveau : `scripts/generate-schema-types.ts`. Modifiés : `package.json`
racine, `CLAUDE.md`.

**Changement.** Recréer le script de codegen. Il doit :
1. lire `schema/v2.json` ;
2. générer `crates/n2b-types/src/schema.rs` via `cargo-typify` (ajouter `cargo-typify` en
   dev-dépendance ou l'invoquer via `bunx`) ;
3. si un type TS est attendu : générer `packages/n2b-types/src/index.ts` via
   `json-schema-to-typescript` ;
4. mode `--check` : régénère en mémoire, diff contre le fichier commité, exit 1 si drift.

Vérifier que `package.json` `codegen:schema` / `codegen:schema:check` pointent le bon
chemin. Lancer `bun run codegen:schema` et confirmer que `schema.rs` régénéré est
**identique** au fichier commité (sinon le fichier commité était déjà stale → commit de
régénération séparé, justifié).

### 0.5 — Corriger CLAUDE.md (PS7)

**Fichier.** `CLAUDE.md`.

**Changements (corrections factuelles uniquement — la refonte registre est en Phase 7) :**
- « `crates/n2b-core/src/schema.rs` » → `crates/n2b-types/src/schema.rs` (2 occurrences).
- Retirer la mention de `packages/n2b/src/schema.ts` (inexistant) ou la corriger en
  `packages/n2b-types/src/index.ts` selon ce que 0.4 produit.
- Exit codes : « `crates/n2b-cli/src/cli/dispatch.rs` » → `crates/n2b-cli/src/commands/scan.rs`.
- Vérifier que la commande `bun run codegen:schema` documentée correspond à 0.4.

### 0.6 — Hygiène repo (PS8)

**Fichiers.** `.gitignore`, `package.json` racine, `Cargo.toml`.

**Changements.**
- `.gitignore` : ajouter `node_modules/` (racine). `git rm -r --cached node_modules`
  pour le désuivre (le `.gitignore` actuel ignore `node_modules/` en sous-dossiers mais
  le dossier racine est suivi).
- `package.json` : supprimer ou corriger le script `install:cli:ts` (référence le
  package fantôme `@n2b/cli` / `packages/n2b-cli/dist/node2bun`).
- `Cargo.toml:1-12` : réécrire l'en-tête commentaire — il décrit un layout `rust/`+
  `native/` obsolète. Le remplacer par une description du layout `crates/*` réel.

> La découpe du crate `n2b-cli` (PS8, volet 2) est **reportée en Phase 7** et reste
> optionnelle — hors chemin critique.

## Critères d'acceptation

- `cargo build --workspace` OK.
- `cargo test --workspace` vert (14 tests Rust + nouveaux tests `edits`/`cli_commands`).
- `cargo clippy --workspace --all-targets -- -D warnings` — **zéro warning**.
- `cargo fmt --all -- --check` OK.
- `bash tests/compare-baseline.sh` — vert. **Exception PS4** : si des baselines changent
  à cause du fix commentaires, elles sont régénérées et le diff est justifié dans le
  commit.
- `bun run codegen:schema:check` — passe (codegen réparé, pas de drift).
- `git status` — `node_modules/` n'apparaît plus.

## Commits attendus (séparés)

```
refactor(n2b-util): apply_edits partagé — supprime la duplication bun_apis ⟂ node_imports (PS2)
fix(n2b-rules): cli_commands ne réécrit plus les lignes commentées (PS4)
refactor(n2b-rules): nomme DIR_CONTEXT_WINDOW_BYTES, documente l'heuristique (PS5)
fix(build): recrée scripts/generate-schema-types.ts — codegen schéma fonctionnel (PS6)
docs(claude): corrige les chemins schema.rs / exit codes / codegen (PS7)
chore(repo): désuit node_modules, corrige package.json, en-tête Cargo.toml (PS8)
```

## Risques

| Risque | Mitigation |
|---|---|
| Le fix PS4 change plus de baselines que prévu | grep préalable des baselines pour `#.*npm\|//.*npm` ; si volume → commit de régénération dédié et documenté |
| `schema.rs` commité était déjà stale | commit de régénération séparé *avant* les autres, pour isoler le diff |
| `git rm --cached node_modules` casse un script local | vérifier qu'aucun script CI ne dépend du `node_modules/` racine suivi |
