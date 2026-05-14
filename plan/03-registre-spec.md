# 03 — Spécification du registre + codegen `sync-coverage`

> Spec exhaustive des cinq `.toml` de `crates/n2b-registry/registry/`, des structs Rust
> correspondantes, des règles de validation, et du fonctionnement de
> `cargo xtask sync-coverage`.

## 1. Principes communs

- **Format TOML.** Lisible en review, supporte les tables de tableaux (`[[...]]`).
- **Tout ID est immuable.** Un `id` du registre = un Rule ID du contrat gelé. On
  *ajoute* des entrées, on n'en *renomme* jamais (cf. [contrat-et-risques.md](contrat-et-risques.md)).
- **Chaque entrée porte sa provenance.** Champ `docs` = chemin relatif vers le fichier
  `docs/` qui justifie la règle. Validé à l'existence au build.
- **La sévérité n'est pas saisie à la main** pour les règles liées à un module — elle
  est **dérivée** du statut `compat` du module hôte (cf. [04-compat-et-schema-v3.md](04-compat-et-schema-v3.md)).

## 2. `modules.toml` — les ~50 modules `node:*`

Décrit chaque module builtin Node, son statut de compat Bun, son équivalent natif.
**Régénéré par `xtask sync-coverage`** (les champs `compat`, `missing_apis`,
`bun_reimpl` sont issus de l'upstream ; les champs `equivalent`, `rewrite_hint` sont
saisis à la main et préservés à la régénération).

```toml
[[modules]]
id            = "imports/node-fs"          # Rule ID
module        = "fs"                        # nom node:* (sans préfixe)
compat        = "full"                      # full | partial | missing  (← sync-coverage)
bun_reimpl    = true                        # existe dans upstream/bun/src/js/node/  (← sync-coverage)
missing_apis  = []                          # sous-APIs absentes si partial  (← sync-coverage)
equivalent    = "Bun.file / Bun.write"      # alternative Bun-native (saisi main)
severity      = "info"                      # DÉRIVÉ de compat (full→info)
docs          = "docs/bun/runtime/nodejs-compat.mdx"

[[modules]]
id            = "imports/node-child_process"
module        = "child_process"
compat        = "partial"
bun_reimpl    = true
missing_apis  = ["proc.gid", "proc.uid", "Stream export", "socket handle IPC"]
equivalent    = "Bun.spawn / Bun.$"
severity      = "warning"                   # DÉRIVÉ (partial→warning)
docs          = "docs/bun/runtime/nodejs-compat.mdx"

[[modules]]
id            = "imports/node-sqlite"
module        = "sqlite"
compat        = "missing"
bun_reimpl    = false
missing_apis  = ["*"]
equivalent    = "bun:sqlite"
severity      = "error"                     # DÉRIVÉ (missing→error)
rewrite_hint  = "import { Database } from 'bun:sqlite'"
bunpp         = "@bun++/node-sqlite"        # polyfill si usage avancé (Phase 6)
docs          = "docs/bun/runtime/sqlite.mdx"
```

**Struct Rust** (`crates/n2b-registry/src/schema.rs`) :

```rust
#[derive(Deserialize)]
pub struct ModuleEntry {
    pub id: String,
    pub module: String,
    pub compat: Compat,                    // enum Full | Partial | Missing
    pub bun_reimpl: bool,
    #[serde(default)] pub missing_apis: Vec<String>,
    pub equivalent: String,
    pub severity: Severity,                // validé == derive_severity(compat)
    #[serde(default)] pub rewrite_hint: Option<String>,
    #[serde(default)] pub bunpp: Option<String>,
    pub docs: String,
}
```

## 3. `apis.toml` — API/méthode Node → réécriture Bun

Chaque entrée décrit un pattern d'appel et sa cible. Remplace `RULES: Vec<ApiRule>` de
`bun_apis.rs`.

```toml
[[apis]]
id            = "api/crypto-createHash"
node          = "crypto.createHash"         # pattern source
import_from   = "crypto"                    # binding requis (PS1 — matching AST)
bun           = "Bun.CryptoHasher"          # cible
compat        = "partial"                   # hérité du module hôte (crypto)
severity      = "warning"                   # dérivé
rewrite       = "template"                  # template | manual | drop
template      = "new Bun.CryptoHasher({0})"
confidence    = "high"                      # high | medium | low
docs          = "docs/bun/runtime/hashing.mdx"

[[apis]]
id            = "api/fs-readFileSync"
node          = "fs.readFileSync"
import_from   = "fs"
bun           = "Bun.file().text()"
compat        = "full"
severity      = "info"
rewrite       = "template"
template      = "await Bun.file({0}).text()"
confidence    = "high"
docs          = "docs/bun/runtime/file-io.mdx"

[[apis]]
id            = "api/child-process-spawn"
node          = "child_process.spawn"
import_from   = "child_process"
bun           = "Bun.spawn"
compat        = "partial"
severity      = "warning"
rewrite       = "manual"                    # signature trop différente pour un template
codemod_hint  = "Bun.spawn([cmd, ...args], { stdout: 'pipe' }) — pas de callback, API Promise"
confidence    = "medium"
docs          = "docs/bun/runtime/spawn.mdx"
```

**`rewrite` — les trois stratégies :**

| Valeur | Sens | Champ requis |
|---|---|---|
| `template` | Réécriture mécanique par substitution `{0}..{n}` | `template` |
| `manual` | Pas mécanisable — le finding porte une recette | `codemod_hint` |
| `drop` | L'appel disparaît (ex. `dotenv.config()` → `.env` autoload natif) | — |

**`import_from`** est la clé de PS1 : `engine.rs` ne déclenche `api/marked-call` que si
`marked` provient d'un `import … from "marked"`. Si `import_from` est absent (ex. global
`Buffer`), le matching est textuel mais reste contraint.

## 4. `packages.toml` — deps npm (résout PS3)

**Source unique** pour les deps. Remplace `BUN_REPLACEMENTS`. Chaque dep porte sa
stratégie d'import **et** la liste de ses APIs (qui génèrent des entrées `apis.toml`
virtuelles — l'`xtask` peut les expanser).

```toml
[[packages]]
id            = "imports/bun-native"        # Rule ID partagé (tous les pkgs le portent)
package       = "marked"
strategy      = "rewrite"                   # drop | rewrite | shim
target        = "Bun.markdown"
aggressive    = true                        # réécrit seulement en --aggressive/--migrate
note          = "Bun a un parseur markdown natif"
apis          = ["marked", "marked.parse"]  # → expansé en entrées apis.toml
docs          = "docs/bun/runtime/bun-apis.mdx"

[[packages]]
id            = "imports/bun-native"
package       = "dotenv"
strategy      = "drop"                      # .env est autoloadé nativement par Bun
target        = "<auto>"
aggressive    = true
note          = "Bun charge .env automatiquement — supprimer require('dotenv').config()"
apis          = ["dotenv.config"]
docs          = "docs/bun/runtime/env.mdx"

[[packages]]
id            = "imports/bun-native"
package       = "express"
strategy      = "shim"                      # pas d'équivalent 1:1 — shim @n2b/shims
target        = "Bun.serve"
aggressive    = false                       # trop intrusif pour l'automatique
note          = "réécriture manuelle vers Bun.serve — voir migration report card"
apis          = []
docs          = "docs/bun/runtime/http.mdx"
```

**`strategy` :**

| Valeur | Sens |
|---|---|
| `drop` | La dep + ses appels disparaissent (fonctionnalité native Bun) |
| `rewrite` | Remplacement mécanique import + appels |
| `shim` | Pas d'équivalent direct — pointer vers `@n2b/shims` ou réécriture manuelle |

## 5. `cli.toml` — commandes npm/pnpm/yarn → bun

Formalise les 41 mappings de `cli_commands.rs`. Déjà quasi-data, juste à externaliser.

```toml
[[cli]]
id          = "cli/npm-install"
pattern     = '\bnpm install\b'             # regex (non-JS — matching textuel)
replace     = "bun install"
respect_comments = true                     # NE PAS réécrire si ligne commentée (PS4)
docs        = "docs/bun/pm/index.mdx"

[[cli]]
id          = "cli/npx"
pattern     = '\bnpx\b'
replace     = "bunx"
respect_comments = true
docs        = "docs/bun/pm/bunx.mdx"
```

Le champ `respect_comments = true` est la correction data-driven de **PS4** : `engine.rs`
applique le même filtre de commentaires à la détection ET à l'édition.

## 6. `globals.toml` — surface CJS & globals Node

Nouveau. Traite `__dirname`, `__filename`, `require`, `process.*`, `Buffer`, `global` comme
une surface à part entière (aujourd'hui éparpillée dans `api/dirname-esm` etc.).

```toml
[[globals]]
id          = "globals/dirname"
symbol      = "__dirname"
context     = "esm"                         # n'est un problème qu'en contexte ESM
bun         = "import.meta.dir"
rewrite     = "template"
template    = "import.meta.dir"
severity    = "warning"
docs        = "docs/bun/runtime/modules.mdx"

[[globals]]
id          = "globals/require-dynamic"
symbol      = "require"
context     = "dynamic-arg"                  # require(variable) — non mécanisable
bun         = "await import()"
rewrite     = "manual"
codemod_hint = "require() dynamique : convertir en await import() — vérifier le top-level await"
severity    = "warning"
docs        = "docs/bun/runtime/modules.mdx"
```

## 7. Règles de validation (au build, dans `registry.rs`)

| Règle | Échec si |
|---|---|
| **IDs uniques** | deux entrées (tous fichiers confondus) ont le même `id` *et* le même pattern |
| **`docs` existe** | le chemin `docs` ne pointe pas un fichier réel du repo |
| **Sévérité dérivée** | `severity` ≠ `derive_severity(compat)` pour les entrées liées à un module |
| **Template bien formé** | `rewrite = "template"` sans champ `template`, ou placeholders `{n}` non séquentiels |
| **`manual` a une recette** | `rewrite = "manual"` sans `codemod_hint` |
| **`compat` cohérent** | `compat` d'une entrée `apis.toml` ≠ `compat` de son module hôte dans `modules.toml` |
| **Regex compile** | un `pattern` ne compile pas comme regex |

Toutes vérifiées par un test `cargo test -p n2b-registry` dédié.

## 8. `cargo xtask sync-coverage` — le codegen

### Entrées
- `upstream/bun/src/js/node/*.ts` — les 62 fichiers = modules réellement réimplémentés.
- `docs/bun/runtime/nodejs-compat.mdx` — matrice 🟢/🟡/🔴 + sous-APIs manquantes.
- `docs/node/*.md` — les 67 fichiers = surface API Node v24 complète.

### Algorithme
1. **Parser le mdx** : extraire `(module, statut, missing_apis[])` pour chaque `node:*`.
2. **Scanner `upstream/`** : `module → bun_reimpl: bool` (présence du `.ts`).
3. **Lister `docs/node/`** : l'ensemble de référence des modules Node existants.
4. **Régénérer `modules.toml`** : pour chaque module Node, écrire/mettre à jour l'entrée
   — champs upstream (`compat`, `bun_reimpl`, `missing_apis`) écrasés, champs manuels
   (`equivalent`, `rewrite_hint`, `bunpp`) **préservés** par merge sur `id`.
5. **Rapport de drift** : tout module de `docs/node/` sans entrée dans `modules.toml`
   après régénération = trou de couverture → listé en stderr.

### Modes
- `cargo xtask sync-coverage` — régénère + affiche le drift.
- `cargo xtask sync-coverage --check` — ne régénère rien, **exit 1** si le `modules.toml`
  commité diverge de ce que la régénération produirait, ou s'il reste un trou. Branché en
  CI (Phase 7).

### Gestion du décalage de version
`nodejs-compat.mdx` est calé sur Node v23 ; `docs/node/` est v24. `node:quic` est dans
`docs/node/` mais absent du mdx. `sync-coverage` traite ce cas : un module présent dans
`docs/node/` mais absent du mdx → entrée générée avec `compat = "missing"` +
`# TODO: absent de nodejs-compat.mdx (décalage v23/v24)` en commentaire. Le drift report
le signale explicitement comme « à vérifier manuellement ».

### Croisement mdx ↔ src
Un module marqué 🔴 dans le mdx mais **présent** dans `upstream/bun/src/js/node/` (cas
`repl.ts`, `trace_events.ts`) → `compat = "missing"` mais `bun_reimpl = true` +
commentaire `# stub présent dans upstream — statut à re-vérifier`. Évite de classer
définitivement 🔴 un module en cours d'implémentation.
