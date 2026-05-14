# Contribuer à n2b

## Modèle de branches — GitHub Flow

`main` est la seule branche longue durée. Elle est **protégée**, toujours
stable et releasable. Aucun commit direct : tout passe par une Pull Request
qui doit avoir la CI verte (`CI Success`) pour être mergée.

### Branches de travail

Préfixe conventionnel + slug court en kebab-case :

| Préfixe | Usage | Exemple |
|---|---|---|
| `feat/` | nouvelle fonctionnalité | `feat/scanner-docker-compose` |
| `fix/` | correction de bug | `fix/cli-commentaires` |
| `refactor/` | refactorisation | `refactor/phase-0-socle` |
| `docs/` | documentation seule | `docs/registre-spec` |
| `chore/` | outillage, CI, dépendances | `chore/repo-governance` |
| `release/` | préparation de release | `release/v0.6.0` |

### Une PR par grosse refactorisation

Le plan de refacto (`plan/`) est découpé en 8 phases. **Chaque phase = une
branche `refactor/phase-N-slug` = une PR.** Pas de méga-PR multi-phases : une
phase se review, se teste et se merge isolément — c'est la condition pour que
le filet baseline reste lisible (cf. `plan/contrat-et-risques.md`).

## Flux de travail

1. Créer la branche depuis `main` à jour.
2. Commits conventionnels (voir ci-dessous).
3. Pousser, ouvrir une PR — le template se remplit tout seul.
4. La CI doit être verte (`CI Success` agrège lint + test + build).
5. **Squash-merge** dans `main`. La branche est supprimée automatiquement.

L'historique de `main` est **linéaire** (pas de merge commits).

## Commits

Format conventionnel, **une ligne**, pas d'emoji :

```
feat(n2b-scanners): scanner docker-compose
fix(n2b-cli): --fix ignore les lignes commentées
chore(ci): ajoute la matrice Windows
```

Scopes = nom du crate/package ou domaine (`n2b-core`, `n2b-cli`, `ci`,
`baselines`…). Jamais de `Co-Authored-By`, jamais de `Generated with`.

## Contrat externe gelé — à ne jamais casser silencieusement

Certaines surfaces sont consommées par des outils tiers via subprocess
(cf. `CLAUDE.md` § « Contrat externe gelé »). Toute PR qui les touche doit :

- **soit** ajouter une nouvelle règle / un nouveau champ (additif, non-breaking) ;
- **soit** justifier le breaking dans la description **et** régénérer les
  baselines dans la même PR.

Surfaces gelées : Rule IDs, schéma JSON `schema/v2.json`, codes de sortie
`0`/`1`/`2`, flags CLI, ABI cdylib v1.

## Tests locaux avant de pousser

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bun run typecheck
bun run codegen:schema:check
bash tests/compare-baseline.sh
```

La CI rejoue tout ça sur Linux, macOS et Windows.

## License

En contribuant, vous acceptez que votre code soit distribué sous license MIT
(cf. [`LICENSE`](LICENSE)).
