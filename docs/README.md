# docs/ — base de connaissance unifiée Bun + Node

Documentation upstream extraite pour piloter la couverture de n2b. **Source de vérité
pour les règles** : ce que Bun supporte, ce que Node expose, ce qu'il faut détecter et
réécrire.

## Contenu

| Dossier | Source | Version épinglée |
|---|---|---|
| `bun/` | `oven-sh/bun` · `docs/` | canary `fd0b6f1` (1.3.x) |
| `node/` | `nodejs/node` · `doc/api/` | LTS v24.15.1 (`9fe7634c`) |

Les clones complets vivent dans `upstream/` (gitignoré). `docs/` ne garde que le
markdown — assets binaires (images, icônes, logos) strippés.

## Régénérer

```bash
# cloner / rafraîchir les sources upstream
git -C upstream/bun pull   # ou re-clone --depth 1
git -C upstream/node pull

# réextraire les docs
rm -rf docs/bun docs/node && mkdir -p docs/bun docs/node
cp -r upstream/bun/docs/. docs/bun/
cp -r upstream/node/doc/api/. docs/node/
cp upstream/node/doc/changelogs/CHANGELOG_V24.md docs/node/
rm -rf docs/bun/{icons,images,logo} docs/bun/{style.css,normalize-internal-links.js,feedback.mdx}
```

## Fichiers clés pour la couverture n2b

| Fichier | Usage |
|---|---|
| `bun/runtime/nodejs-compat.mdx` | Matrice de compat Node→Bun — quels modules Node marchent, partiellement, ou pas |
| `bun/runtime/bun-apis.mdx` | Surface des APIs natives `Bun.*` — cibles de réécriture |
| `bun/runtime/*.mdx` | Détail par API (ffi, sqlite, shell, sql, redis, s3, glob…) |
| `bun/project/roadmap.mdx` | Roadmap Bun — anticiper les règles futures |
| `node/*.md` | Surface API Node complète (un fichier par module) — ce que n2b doit savoir détecter |
| `node/CHANGELOG_V24.md` | Nouveautés Node v24 LTS |
