# Phase 6 — Intégration `bunpp` (les 🔴)

> Pour chaque module 🔴, le finding pointe vers le polyfill `@bun++/node-*`
> correspondant. `--migrate` peut appeler `bunpp scaffold`.
>
> **Dépend de :** Phase 3 (le champ `compat` + le statut 🔴). **Bloque :** Phase 7.

## Objectif

Les modules 🔴 (`repl`, `node:sqlite` en usage avancé, `trace_events`, `quic`) n'ont pas
d'équivalent Bun natif. Plutôt qu'un trou silencieux, n2b doit pointer vers le polyfill
`@bun++/node-*` — le crate `bunpp_cmd.rs` (762 l.) scaffolde déjà ces polyfills.

## État de départ

- `crates/n2b-cli/src/bunpp_cmd.rs` — subcommand `bunpp`, scaffolde des polyfills
  `@bun++/node-*` pour les gaps Node de Bun. Déjà conscient des gaps canary.
- `registry/modules.toml` — champ `bunpp` déjà prévu dans la spec (cf.
  [03-registre-spec.md](../03-registre-spec.md) §2) : `bunpp = "@bun++/node-sqlite"`.
- Le lien `modules.toml` ↔ `bunpp_cmd.rs` n'existe pas encore.

## Travaux

### 6.1 — Renseigner le champ `bunpp` des modules 🔴

**Fichier.** `crates/n2b-registry/registry/modules.toml`.

Pour chaque module `compat = "missing"`, renseigner `bunpp` avec le nom du polyfill :

```toml
[[modules]]
id     = "imports/node-repl"
module = "repl"
compat = "missing"
bunpp  = "@bun++/node-repl"
...

[[modules]]
id     = "imports/node-trace_events"
module = "trace_events"
compat = "missing"
bunpp  = "@bun++/node-trace-events"
...
```

Vérifier la cohérence avec ce que `bunpp_cmd.rs` sait scaffolder — si un polyfill n'est
pas encore supporté par `bunpp`, le champ `bunpp` reste absent et le finding dit
« pas de polyfill disponible — réécriture manuelle requise ».

### 6.2 — Le finding 🔴 pointe vers `bunpp`

**Fichier.** `crates/n2b-registry/src/engine.rs`.

Quand un `imports/node-*` matche un module 🔴, le `Finding` :
- `severity = error` ;
- `message` cite le polyfill : « `node:sqlite` non supporté par Bun — utiliser
  `bun:sqlite`, ou `bunpp scaffold node-sqlite` pour un polyfill compatible API Node » ;
- `compat.equivalent` = `bun:sqlite` ;
- un champ du `Finding` (réutiliser `replacement` ou `docs_url`, ou ajouter à
  `compat`) porte la commande `bunpp` suggérée.

### 6.3 — `--migrate` peut appeler `bunpp scaffold`

**Fichier.** `crates/n2b-cli/src/commands/migrate.rs`.

Quand `--migrate` rencontre un 🔴 avec un champ `bunpp` :
- par défaut : ajoute le module au `manual_residue` du report card avec la suggestion
  `bunpp scaffold` ;
- avec un flag opt-in (`--migrate --scaffold-polyfills`) : appelle réellement
  `bunpp scaffold <module>` via subprocess, sous `BackupGuard`.

Le scaffold n'est **pas** automatique par défaut — il crée des fichiers dans le projet
cible, c'est une action visible qui mérite un opt-in explicite.

### 6.4 — Croisement avec `upstream/`

`repl.ts` et `trace_events.ts` **existent** dans `upstream/bun/src/js/node/` (stubs en
cours d'implémentation). `xtask sync-coverage` (Phase 4) marque ces modules
`bun_reimpl = true` malgré `compat = "missing"`. Le finding 🔴 doit le mentionner :
« stub présent dans Bun canary — statut susceptible d'évoluer ». Évite de pousser un
polyfill pour un module qui sera natif au prochain bump.

## Critères d'acceptation

- Tout module `compat = "missing"` du registre a soit un champ `bunpp`, soit une note
  explicite « pas de polyfill disponible ».
- Un scan sur du code important `node:repl` ou `node:sqlite` émet un `error` avec la
  suggestion `bunpp` dans le message.
- `n2b --migrate --report=json` : les 🔴 apparaissent dans `manual_residue` avec leur
  `bunpp` suggéré.
- `n2b --migrate --scaffold-polyfills` testé : scaffolde réellement, sous `BackupGuard`,
  rollback si échec.
- `cargo test --workspace` vert.

## Commits attendus

```
feat(n2b-registry): champ bunpp sur les modules 🔴 — repl, sqlite, trace_events, quic
feat(n2b-registry): engine — le finding 🔴 pointe vers bunpp scaffold
feat(n2b-cli): --migrate --scaffold-polyfills — appelle bunpp scaffold sous BackupGuard
```

## Risques

| Risque | Mitigation |
|---|---|
| `bunpp_cmd.rs` ne sait pas scaffolder tous les 🔴 | champ `bunpp` optionnel ; absence = note « pas de polyfill » au lieu d'une commande cassée |
| Un module 🔴 devient 🟢 au bump canary → suggestion `bunpp` obsolète | `xtask sync-coverage` re-synchronise le statut ; le finding mentionne « stub présent dans canary » quand `bun_reimpl = true` |
| `--scaffold-polyfills` crée des fichiers non voulus | opt-in explicite, jamais par défaut ; sous `BackupGuard` ; documenté |
