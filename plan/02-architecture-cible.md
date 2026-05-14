# 02 — Architecture cible : registre de règles data-driven

> Le cœur du refactor. Aujourd'hui les règles **sont du code Rust** (`Vec<ApiRule>`,
> `HashMap<&str, BunReplacement>`). Cible : les règles **sont des données** embarquées,
> source unique de vérité, auditables par diff contre `docs/` et `upstream/`.

## 1. Le problème que résout le registre

Aujourd'hui pour ajouter le support d'un paquet npm il faut :
1. éditer `BUN_REPLACEMENTS` dans `node_imports.rs` (côté import) ;
2. éditer `RULES` dans `bun_apis.rs` (côté appel de méthode) ;
3. recompiler ;
4. espérer que les deux ne divergent pas (PS3).

Et n2b **ne sait pas** répondre à « quels modules Node n'ai-je pas encore couverts ? » —
la couverture n'est mesurable nulle part.

Avec le registre :
1. éditer **un `.toml`** ;
2. `cargo xtask sync-coverage --check` dit immédiatement si un trou de couverture subsiste.

La couverture de n2b devient une **fonction de la source de Bun/Node**,
re-synchronisable à chaque bump canary.

## 2. Le nouveau crate : `n2b-registry`

Un crate dédié, inséré dans le DAG entre `n2b-util` et `n2b-rules` :

```
n2b-types → n2b-util → n2b-registry → n2b-rules → n2b-scanners → n2b-core → n2b-cli
```

```
crates/n2b-registry/
  Cargo.toml
  registry/
    modules.toml      # node:* → statut compat + équivalent Bun + sévérité
    apis.toml         # API/méthode Node → template de réécriture Bun + confiance
    packages.toml     # dep npm → natif Bun ou bun:* + stratégie (drop/rewrite/shim)
    cli.toml          # npm/pnpm/yarn/npx → bun
    globals.toml      # __dirname, __filename, require, process.* → surface CJS/globals
  src/
    lib.rs            # re-exports
    schema.rs         # structs Rust du registre (ModuleEntry, ApiEntry, PackageEntry…)
    registry.rs       # charge les .toml via include_str! + valide au build
    engine.rs         # match registry ↔ findings (AST pour JS, regex pour le reste)
```

**`n2b-rules` ne disparaît pas** — il devient le moteur de *matching* qui consomme le
registre. `n2b-registry` porte les données + le chargement + la validation.

### Pourquoi un crate séparé et pas un module de `n2b-rules` ?

- **Compilation incrémentale** : éditer un `.toml` ne recompile que `n2b-registry`, pas
  tout `n2b-rules` + `n2b-scanners`.
- **Testabilité** : la validation du registre (IDs uniques, templates bien formés,
  `docs` existants) est un crate à tester isolément.
- **`xtask` cible** : `cargo xtask sync-coverage` régénère `registry/modules.toml` — un
  crate dédié rend la frontière codegen ↔ code claire.

## 3. Chargement & validation au build

`registry.rs` charge les `.toml` via `include_str!` (embarqués dans le binaire, zéro I/O
runtime) et valide via `once_cell::Lazy` :

```rust
// crates/n2b-registry/src/registry.rs
static MODULES: Lazy<Vec<ModuleEntry>> = Lazy::new(|| {
    let raw = include_str!("../registry/modules.toml");
    let parsed: ModulesFile = toml::from_str(raw)
        .expect("registry/modules.toml invalide");
    validate_unique_ids(&parsed.modules);     // panic au 1er test si doublon
    validate_docs_paths(&parsed.modules);     // chaque `docs` pointe un fichier réel
    validate_rewrite_templates(&parsed.modules);
    parsed.modules
});
```

La validation tourne au **premier accès** — donc dès le premier test
(`cargo test --workspace`). Un `.toml` malformé fait échouer la CI immédiatement, pas en
production.

> **Choix de dépendance** : `toml` (crate) pour le parsing. Pas `serde_yaml` (déjà au
> workspace mais YAML est moins lisible pour des tables denses). `toml` est ajouté à
> `[workspace.dependencies]`.

## 4. Le moteur de matching — `engine.rs`

`engine.rs` est le point de jonction registre ↔ scanners. Il expose :

```rust
pub enum MatchInput<'a> {
    /// Code JS/TS — l'AST a déjà résolu les bindings d'import.
    Ast { source: &'a str, imports: &'a ImportGraph },
    /// Tout le reste — configs, shell, Dockerfile : regex sur texte brut.
    Text { source: &'a str, kind: FileKind },
}

pub fn match_rules(input: MatchInput) -> Vec<Finding>;
```

- **Pour le JS/TS** (`MatchInput::Ast`) : chaque entrée `apis.toml` ne matche un
  call-site que si son origine d'import est connue (résout PS1). L'`ImportGraph` est
  produit par `imports_ast.rs` (étendu en Phase 2).
- **Pour le non-JS** (`MatchInput::Text`) : regex compilées depuis le champ `pattern` du
  registre, exactement comme aujourd'hui mais data-driven.

L'anti-overlap et l'application d'édits passent par `n2b_util::edits::apply_edits` (PS2).

## 5. Le crate `xtask` — codegen & vérification de drift

Nouveau crate `xtask/` (membre du workspace, convention Rust standard) :

```
xtask/
  Cargo.toml          # [[bin]] name = "xtask"
  src/
    main.rs           # dispatch des sous-commandes xtask
    sync_coverage.rs  # cargo xtask sync-coverage [--check]
```

`cargo xtask sync-coverage` :
1. lit `upstream/bun/src/js/node/` → liste des modules réellement réimplémentés par Bun ;
2. lit `docs/bun/runtime/nodejs-compat.mdx` → matrice 🟢/🟡/🔴 + sous-APIs manquantes ;
3. lit `docs/node/*.md` → surface API Node complète ;
4. **régénère** `registry/modules.toml` avec le statut compat à jour ;
5. **émet un rapport de drift** : tout module Node sans entrée registre = trou de couverture.

`--check` : ne régénère rien, échoue (exit ≠ 0) si le registre diverge de l'upstream.
Branché en CI (Phase 7).

> Détail complet de `sync-coverage` et de la spec des `.toml` :
> [03-registre-spec.md](03-registre-spec.md).

## 6. Layout cible du workspace (après refactor)

```
n2b/
  Cargo.toml              # workspace — members: crates/*, xtask
  xtask/                  # NOUVEAU — codegen & drift check
  crates/
    n2b-types/            # inchangé — types + schema.rs généré
    n2b-util/             # + edits.rs (PS2)
    n2b-registry/         # NOUVEAU — registry/*.toml + registry.rs + engine.rs
    n2b-rules/            # devient un thin matcher consommant n2b-registry
    n2b-scanners/         # source.rs passe AST-first (Phase 2)
    n2b-report/           # + champ compat dans le rendu (Phase 3)
    n2b-ai/               # inchangé
    n2b-github/           # inchangé
    n2b-core/             # inchangé (engine walk)
    n2b-cli/              # inchangé (+ découpe optionnelle Phase 7, PS8)
    n2b-native/           # inchangé (ABI v1 gelée)
  scripts/
    generate-schema-types.ts   # RECRÉÉ (PS6)
  registry/  →  vit dans crates/n2b-registry/registry/ (pas à la racine)
  schema/
    v2.json               # ou v3.json après Phase 3
```

## 7. Invariant d'architecture à préserver

> **Un scanner ne connaît pas les règles, un rule ne connaît pas les scanners.** Le
> contrat reste `Finding`.

Le registre ne casse pas cet invariant — il le **renforce** :
- les scanners produisent des `MatchInput` (AST ou Text) ;
- `n2b-registry::engine` produit des `Finding` depuis le registre ;
- le scanner ne sait pas quelle règle a matché, le registre ne sait pas quel fichier.

## 8. Bénéfices mesurables

| Avant | Après |
|---|---|
| Ajouter une dep = 2 fichiers Rust + recompile | Éditer 1 `.toml` |
| Couverture non mesurable | `xtask sync-coverage --check` = 0 trou ou échec CI |
| `BUN_REPLACEMENTS` ⟂ `RULES` divergent (PS3) | Source unique `packages.toml` |
| Sévérité codée en dur par règle | Sévérité **dérivée** du statut compat du module |
| Faux positifs identifiants nus (PS1) | Matching corrélé au binding d'import |
| Règles invisibles au review | `git diff registry/*.toml` lisible par un humain |
