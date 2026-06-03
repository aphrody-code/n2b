## Type of change

- [ ] `feat` — new feature
- [ ] `fix` — bug fix
- [ ] `refactor` — refactoring (state the `plan/` phase if applicable)
- [ ] `docs` — documentation
- [ ] `chore` — tooling / CI / dependencies

## Description

<!-- What and why. The "how" is in the diff. -->

## Frozen external contract

- [ ] Touches **no** frozen surface (Rule IDs, `schema/v2.json`, exit codes, CLI flags, cdylib ABI).
- [ ] Touches a frozen surface — the change is **additive** (new rule / optional field).
- [ ] Introduces an intentional **breaking** change — justified above + baselines regenerated in this PR.

## Checklist

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `bun run typecheck` + `bun run codegen:schema:check`
- [ ] `bash tests/compare-baseline.sh`
- [ ] `CHANGELOG.md` updated if the output changes
