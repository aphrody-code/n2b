## Type de changement

- [ ] `feat` — nouvelle fonctionnalité
- [ ] `fix` — correction de bug
- [ ] `refactor` — refactorisation (préciser la phase `plan/` si applicable)
- [ ] `docs` — documentation
- [ ] `chore` — outillage / CI / dépendances

## Description

<!-- Quoi et pourquoi. Le « comment » est dans le diff. -->

## Contrat externe gelé

- [ ] Ne touche **aucune** surface gelée (Rule IDs, `schema/v2.json`, codes de sortie, flags CLI, ABI cdylib).
- [ ] Touche une surface gelée — le changement est **additif** (nouvelle règle / champ optionnel).
- [ ] Introduit un **breaking** assumé — justifié ci-dessus + baselines régénérées dans cette PR.

## Checklist

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `bun run typecheck` + `bun run codegen:schema:check`
- [ ] `bash tests/compare-baseline.sh`
- [ ] `CHANGELOG.md` mis à jour si la sortie change
