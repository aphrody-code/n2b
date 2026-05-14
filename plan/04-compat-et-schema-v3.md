# 04 — Modèle de compat → sévérité & impact schéma

> Comment la matrice 🟢/🟡/🔴 pilote la sévérité des findings, et comment exposer le
> statut de compat dans la sortie JSON **sans casser le contrat** — l'audit a montré
> qu'on peut éviter le breaking v2→v3 annoncé dans `REFACTOR_PLAN.md`.

## 1. La matrice de compat, mesurée

Source : `docs/bun/runtime/nodejs-compat.mdx` (calé Node v23) croisé avec `docs/node/`
(v24) et `upstream/bun/src/js/node/`.

**47 modules Node v24 :** 22 🟢 Fully · 19 🟡 Partial · 3 🔴 Missing · 1 non-couvert
(`node:quic`, absent du mdx — décalage v23/v24).

| Statut | Modules |
|---|---|
| 🟢 **full** (22) | assert, buffer, console, dgram, diagnostics_channel, dns, events, fs, http, https, net, os, path, punycode, querystring, readline, stream, string_decoder, timers, tty, url, zlib |
| 🟡 **partial** (19) | async_hooks, child_process, cluster, crypto, domain, http2, inspector, module, perf_hooks, process, sys, test, tls, util, v8, vm, wasi, worker_threads, *(+ `process` global)* |
| 🔴 **missing** (3) | repl, sqlite, trace_events |
| ❓ **non classé** (1) | quic *(à traiter comme missing en attendant)* |

Détail des sous-APIs manquantes par module 🟡 : [coverage/modules.md](coverage/modules.md).

## 2. La règle de dérivation sévérité

```rust
// crates/n2b-registry/src/schema.rs
pub fn derive_severity(compat: Compat) -> Severity {
    match compat {
        Compat::Full    => Severity::Info,    // marche tel quel — info, pas warning
        Compat::Partial => Severity::Warn,    // marche en partie — warning + sous-API citée
        Compat::Missing => Severity::Error,   // ne marche pas — error + pointeur bunpp
    }
}
```

C'est validé au build : toute entrée registre liée à un module doit avoir
`severity == derive_severity(compat)` (cf. [03-registre-spec.md](03-registre-spec.md) §7).

### Effet sur les findings `imports/*`

| Compat | Finding émis |
|---|---|
| 🟢 full | `info` — « `node:fs` est pleinement supporté par Bun » |
| 🟡 partial | `warning` — « `node:child_process` partiel : `proc.gid`/`proc.uid` absents, IPC limité » (sous-APIs **nommées**) |
| 🔴 missing | `error` — « `node:sqlite` non supporté → `bun:sqlite` ou `@bun++/node-sqlite` » (pointeur Phase 6) |

Aujourd'hui n2b émet un seul niveau indifférencié (« préfixer `node:` ») — il **se tait**
sur `cluster`/`vm`/`repl`. Le modèle de compat le fait parler, justement.

### Granularité : module vs sous-API

Les 19 modules 🟡 ont leurs sous-APIs manquantes **nommément identifiées** dans le mdx.
C'est de la matière pour des règles fines. Deux niveaux :

- **`imports/node-<module>`** — le module est importé : sévérité dérivée du statut global.
- **`api/node-<module>-<subapi>`** *(Phase 4)* — une sous-API **précise et manquante** est
  appelée (ex. `cluster` + transfert de handle) : `error`, même si le module est 🟡.

Le second niveau évite le faux confort : importer `node:vm` est 🟡 (OK), mais appeler
`vm.measureMemory()` est 🔴 (cassé). Le registre encode les deux.

## 3. Exposer `compat` dans la sortie — le champ `Finding.compat`

### Le constat de l'audit

`schema/v2.json` définit `Finding` avec **`additionalProperties: false`** (ligne 102).
Conséquence directe : **tout champ ajouté à l'objet JSON est rejeté par la validation
`jsonschema`** tant que le schéma n'est pas mis à jour. Donc ajouter `compat` impose
*nécessairement* de toucher `schema/v2.json`. La question est : breaking ou pas ?

### Deux options — et l'audit tranche

| | Option A — `compat` **optionnel** | Option B — `compat` **requis** (v3) |
|---|---|---|
| `schema/v2.json` | ajout dans `properties`, **pas** dans `required` | ajout dans `required`, `schema_version` enum `[2]`→`[3]` |
| Rétro-compat JSON Schema | ✅ un Finding sans `compat` valide toujours | ❌ breaking — anciens consommateurs cassent |
| `schema_test.rs` (round-trip baselines figées) | ✅ baselines sans `compat` désérialisent | ❌ baselines figées ne désérialisent plus |
| `rpb-dashboard` | aucune coordination requise | doit être prévenu et adapté avant |
| Baselines à régénérer | oui (le champ apparaît dans la sortie) | oui |
| Bump `schema_version` | non | oui → `v3.json` |

**Décision : Option A — `compat` optionnel.** `REFACTOR_PLAN.md` annonçait un breaking
v2→v3 « assumé ». L'audit montre qu'il est **évitable** : un champ optionnel garde la
rétro-compatibilité JSON Schema, ne casse pas `schema_test.rs`, et ne demande aucune
coordination bloquante avec `rpb-dashboard`. On garde `schema/v2.json` (pas de
`v3.json`).

> Le champ reste **toujours émis** par n2b en pratique — « optionnel » concerne la
> validation du schéma (rétro-compat), pas le comportement du binaire.

### Forme du champ

```json
"compat": {
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "status":       { "enum": ["full", "partial", "missing"] },
    "module":       { "type": "string" },
    "missing_apis": { "type": "array", "items": { "type": "string" } },
    "equivalent":   { "type": "string" }
  },
  "required": ["status"]
}
```

Exemple de `Finding` enrichi :

```json
{
  "rule_id": "imports/node-child_process",
  "severity": "warn",
  "message": "node:child_process est partiellement supporté par Bun",
  "compat": {
    "status": "partial",
    "module": "child_process",
    "missing_apis": ["proc.gid", "proc.uid", "socket handle IPC"],
    "equivalent": "Bun.spawn / Bun.$"
  },
  "...": "..."
}
```

## 4. Travaux schéma de la Phase 3 — checklist

1. **Réparer le codegen d'abord** (PS6, Phase 0) — sans `scripts/generate-schema-types.ts`
   fonctionnel, impossible de régénérer proprement. **Phase 3 dépend de Phase 0.**
2. Ajouter la définition `compat` à `schema/v2.json` (`properties` uniquement, pas
   `required`).
3. `bun run codegen:schema` → régénère `crates/n2b-types/src/schema.rs`.
4. Câbler la production du champ : `n2b-registry::engine` attache le `compat` au
   `Finding` depuis `modules.toml`.
5. `n2b-report` : afficher `compat` dans `text` (ligne « compat: partial — manque X »),
   `markdown`, `sarif` (propriété `properties`), `json`/`jsonl` (natif).
6. Régénérer **toutes** les baselines : `tests/snapshots/baseline/*.{json,jsonl,md,sarif,txt}`
   + `tests/rpb-dashboard-baseline/scan.*`.
7. `contract.rs` : le test `json_report_validates_against_schema_v2` doit rester vert
   (c'est le cas si `compat` est dans `properties` et le schéma mis à jour).
8. `CHANGELOG.md` : entrée « ajout du champ optionnel `Finding.compat` — rétro-compatible,
   pas de bump de `schema_version` ».

## 5. Pourquoi ce champ change la donne

Sans `compat`, `rpb-dashboard` (et tout consommateur) ne peut pas distinguer « ce
finding est un détail cosmétique » de « ce finding bloque la migration ». Avec `compat` :
- un dashboard peut **trier** : afficher les 🔴 en premier ;
- la « migration report card » (Phase 5) calcule `auto_migratable_pct` en excluant
  précisément les `compat.status == "missing"` ;
- `n2b --migrate` sait quels findings router vers `bunpp scaffold` (Phase 6).

Le champ `compat` est ce qui rend le **pilier 2 mesurable**.
