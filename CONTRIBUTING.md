# Contributing to n2b

## Branching model — GitHub Flow

`main` is the only long-lived branch: always stable and releasable.
**No direct commits** — everything goes through a Pull Request whose CI
(`CI Success`) must be green before merge.

> Server-side enforcement of the direct-push block (branch protection /
> rulesets) requires GitHub Pro or a public repo. On a private repo on the
> free plan it is a **convention** applied by discipline — switchable on with
> one command the day the repo goes public or Pro. The merge settings
> (squash/rebase only, no merge commits, auto-delete the branch) are already
> active server-side.

### Working branches

Conventional prefix + short kebab-case slug:

| Prefix | Use | Example |
|---|---|---|
| `feat/` | new feature | `feat/scanner-docker-compose` |
| `fix/` | bug fix | `fix/cli-comments` |
| `refactor/` | refactoring | `refactor/phase-0-foundation` |
| `docs/` | documentation only | `docs/registry-spec` |
| `chore/` | tooling, CI, dependencies | `chore/repo-governance` |
| `release/` | release preparation | `release/v0.6.0` |

### One PR per large refactor

The refactor plan (`plan/`) is split into 8 phases. **Each phase = one
`refactor/phase-N-slug` branch = one PR.** No multi-phase mega-PRs: a phase is
reviewed, tested, and merged in isolation — that is the condition for keeping
the baseline safety net readable (see `plan/contrat-et-risques.md`).

## Workflow

1. Create the branch from an up-to-date `main`.
2. Conventional Commits (see below).
3. Push, open a PR — the template fills itself in.
4. CI must be green (`CI Success` aggregates lint + test + build).
5. **Squash-merge** into `main`. The branch is deleted automatically.

The `main` history is **linear** (no merge commits).

## Commits

Conventional format, **single line**, no emoji:

```
feat(n2b-scanners): docker-compose scanner
fix(n2b-cli): --fix ignores commented lines
chore(ci): add the Windows matrix
```

Scopes = crate/package name or domain (`n2b-core`, `n2b-cli`, `ci`,
`baselines`, ...). Never `Co-Authored-By`, never `Generated with`.

## Frozen external contract — never break silently

Some surfaces are consumed by third-party tools via subprocess (see
`CLAUDE.md`, "Frozen external contract" section). Any PR that touches them must
either:

- add a new rule / a new field (additive, non-breaking); or
- justify the breaking change in the description **and** regenerate the
  baselines in the same PR.

Frozen surfaces: Rule IDs, the `schema/v2.json` JSON schema, the `0`/`1`/`2`
exit codes, CLI flags, the cdylib v1 ABI.

## Local tests before pushing

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bun run typecheck
bun run codegen:schema:check
bash tests/compare-baseline.sh
```

CI replays all of this on Linux, macOS, and Windows.

## License

By contributing, you agree that your code is distributed under the Apache 2.0
license (see [`LICENSE`](LICENSE)).
