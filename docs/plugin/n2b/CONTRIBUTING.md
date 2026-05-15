# Contribuer à rsbun

## Règles non-négociables

1. **Bun uniquement** — jamais `node`/`npm`/`npx`/`pnpm`/`yarn`
2. **CLI Rust** — `rg`, `fd`, `bat`, `sd`, `eza`, `dust`, `xh`, `delta` (jamais GNU)
3. **Ne pas polluer les clones** — toujours créer une branche locale (`feat/…`, `perf/…`) avant tout changement dans un dossier 📦
4. **Tâches via `just`** — pas de scripts ad-hoc

## Avant de committer (sous-projets own)

```bash
just fmt-check          # rustfmt
just lint               # clippy -D warnings
just n2b-test           # tests complets si modif n2b
```

## Ajouter une nouvelle recherche dans `docs/`

1. Créer le fichier dans `docs/research/<sujet>-YYYY.md`
2. Ajouter l'entrée dans `docs/README.md` (table)
3. Vérifier qu'il respecte `.editorconfig`

## Ajouter un nouveau projet à la racine

Voir [`STRUCTURE.md`](STRUCTURE.md) — section "Ajouter un nouveau projet".

## Architecture n2b

Voir [`n2b/CLAUDE.md`](n2b/CLAUDE.md) — contrats externes **gelés** (ne pas casser).

## Patches locaux sur clones upstream

Pour les clones `wasm/wasm-bindgen`, `wasm/wasm-pack`, `bun`, etc. :

1. Créer une branche locale : `git checkout -b perf/<sujet>`
2. Documenter le patch dans `wasm/<REPO>_BUN_PATCH.md` (ou équivalent)
3. Commit SANS pusher sur upstream (sauf PR explicite)

Exemple existant : `wasm/wasm-bindgen` branche `perf` avec 3 commits dont le patch Bun-aware.
