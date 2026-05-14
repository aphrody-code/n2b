# Phase 2 — Scanner source AST-first

> Corrige **PS1**. Le scanner JS/TS passe en pipeline oxc : chaque call-site est corrélé
> à son binding d'import. Les regex ne survivent que pour le non-JS.
>
> **Dépend de :** Phase 1. **Bloque :** Phase 7. **Parallélisable avec :** Phases 3, 4.

## Objectif

Tuer les faux positifs sur identifiant homonyme. Aujourd'hui `bun_apis.rs` matche
`marked(`, `which(`, `v4()`, `exec(` par regex sur texte brut — une fonction locale
nommée `marked` déclenche un finding. Cible : une règle `api/*` ne matche que si
l'identifiant **provient réellement** de l'import attendu.

## État de départ

- `crates/n2b-rules/src/imports_ast.rs` (174 l.) **sait déjà** résoudre les imports via
  `oxc` — `extract_specifiers()` couvre imports ESM statiques, imports dynamiques,
  `require()` CJS.
- Mais il ne produit que la **liste des specifiers**, pas un graphe de bindings
  (`marked` → vient de `"marked"`).
- Et seul `node_imports.rs:680` l'utilise. `bun_apis.rs` est 100 % regex.
- `is_member_exec_call` (`bun_apis.rs:614-617`) est le hack ad-hoc qui prouve le besoin.

## Travaux

### 2.1 — Étendre `imports_ast.rs` en `ImportGraph`

**Fichier.** `crates/n2b-rules/src/imports_ast.rs`.

Remplacer (ou compléter) `extract_specifiers()` par un `build_import_graph()` qui
retourne :

```rust
pub struct ImportGraph {
    /// binding local → (specifier source, kind)
    /// ex. "marked" → ("marked", Named), "fs" → ("node:fs", Namespace)
    bindings: HashMap<String, ImportBinding>,
    /// require() dynamiques (argument non statique) — non résolus, à signaler
    dynamic_requires: Vec<Span>,
}

pub struct ImportBinding {
    pub specifier: String,       // "marked", "node:fs", "crypto"
    pub kind: BindingKind,       // Default | Named { imported } | Namespace | Require
    pub span: Span,
}

impl ImportGraph {
    /// Le symbole `name` provient-il d'un import du specifier `from` ?
    pub fn resolves(&self, name: &str, from: &str) -> bool { ... }
}
```

Le visitor oxc parcourt `ImportDeclaration`, `VariableDeclarator` avec `require()`,
`ImportExpression`. Pour chaque, enregistre le binding local.

### 2.2 — Passer `bun_apis` en matching AST

**Fichier.** `crates/n2b-rules/src/bun_apis.rs` + `crates/n2b-registry/src/engine.rs`.

`engine.rs::match_rules(MatchInput::Ast { source, imports })` :
- parse le source en AST oxc une seule fois ;
- pour chaque `CallExpression` / `MemberExpression`, cherche une entrée `apis.toml` dont
  le `node` correspond ;
- **filtre par `import_from`** : l'entrée ne matche que si
  `imports.resolves(callee_root, entry.import_from)`. Si `import_from` est absent (cas
  globals : `Buffer`, `process`), matching textuel contraint conservé.

`bun_apis.rs` devient un thin wrapper qui délègue à `engine.rs`. Le hack
`is_member_exec_call` **disparaît** — l'AST sait nativement que `re.exec()` est un
appel de méthode sur un objet, pas l'import `exec` de `child_process`.

### 2.3 — `source.rs` produit un `MatchInput::Ast`

**Fichier.** `crates/n2b-scanners/src/source.rs` (20 l. aujourd'hui).

Le scanner source :
1. lit le fichier ;
2. construit l'`ImportGraph` une fois ;
3. passe `MatchInput::Ast { source, imports }` à `engine::match_rules` ;
4. les regex pour `.sh`/`Dockerfile`/configs restent dans leurs scanners respectifs via
   `MatchInput::Text`.

### 2.4 — Constante magique `600` (PS5, volet 2)

`looks_like_dir_context` / `DIR_CONTEXT_WINDOW_BYTES` : avec l'AST, on peut savoir si le
**même symbole** est passé à `fs.mkdir` sans fenêtre arbitraire. Réévaluer : si l'AST
rend l'heuristique exacte, **supprimer** la constante et la fonction. Sinon, la garder
documentée (déjà fait Phase 0).

### 2.5 — Proptests anti-faux-positifs

**Fichier.** `crates/n2b-core/tests/proptest_source.rs` (étendre).

Nouveau test : génère un source avec une **fonction locale homonyme** d'une API connue
(`function marked() {}`, `const which = () => {}`, `let v4 = 0`) sans l'import
correspondant → **0 finding** `api/*`. Inverse : avec l'import → finding présent.

## Critères d'acceptation

- **Nouveau proptest « fonction locale homonyme → 0 finding » vert.**
- Les faux positifs connus disparaissent → certaines baselines `test/fixture` et
  `rpb-dashboard` **changent légitimement** (moins de findings). Régénérées + chaque
  disparition justifiée dans le commit (« `marked` ligne N était une fonction locale »).
- `cargo test --workspace` vert.
- `cargo clippy --workspace --all-targets -- -D warnings`.
- `is_member_exec_call` supprimé (`git grep` le confirme).
- Pas de régression de performance > 15 % sur un gros repo témoin (oxc parse une fois
  par fichier — déjà le cas pour `node_imports`).

## Commits attendus

```
feat(n2b-rules): ImportGraph — résolution binding→specifier via oxc
refactor(n2b-registry): engine.rs matche les api/* contre l'AST (résout PS1)
refactor(n2b-scanners): source.rs produit un MatchInput::Ast
test(n2b-core): proptest — fonction locale homonyme ne déclenche aucun api/*
chore(n2b-rules): supprime is_member_exec_call et looks_like_dir_context (rendus inutiles par l'AST)
```

## Risques

| Risque | Mitigation |
|---|---|
| L'AST oxc rate un pattern que la regex attrapait (réexport, alias profond) | garder un mode `Text` de secours configurable ; les proptests couvrent les cas nominaux ; documenter les limites connues |
| Baselines changent beaucoup → revue lourde | trier les diffs : chaque finding disparu doit être un *vrai* faux positif ; si un *vrai* finding disparaît, c'est un bug du graph |
| Coût parse double (node_imports + bun_apis) | mutualiser : `source.rs` parse une fois, passe l'AST aux deux. Un seul `oxc_parser::Parser::parse` par fichier |
| `require()` dynamique non résolu | enregistré dans `dynamic_requires` → signalé comme finding `globals/require-dynamic` (Phase 4/5), pas ignoré silencieusement |
