<!-- SPDX-License-Identifier: Apache-2.0 -->
# Security Policy

`n2b` is a developer codemod tool: it reads source trees and, with `--fix` /
`--aggressive` / `--migrate`, rewrites files in place. The main security
considerations are therefore (1) the integrity of the rewrites it applies and
(2) the safety of running it against untrusted repositories.

## Supported versions

Only the latest released `0.x` line receives security fixes. `n2b` is
pre-1.0; pin an exact version in CI if you depend on stable behaviour.

| Version | Supported |
| ------- | --------- |
| latest `0.x` | yes |
| older | no — upgrade |

## Reporting a vulnerability

Please **do not** open a public issue for a security problem.

- Preferred: open a private advisory via GitHub Security Advisories —
  <https://github.com/aphrody-code/n2b/security/advisories/new>.
- Alternative: email `security@aphrody.dev` with a description, affected
  version (`n2b --version`), and a minimal reproduction.

We aim to acknowledge within 72 hours and to ship a fix or mitigation for
confirmed issues in the next patch release.

## Scope and safe usage

- `n2b` writes to disk only under `--fix`, `--aggressive`, `--migrate`, or the
  `scaffold`/`app`/`win32`/`linux`/`wasm` generators. A bare scan (no flag) is
  read-only and emits a report on stdout.
- `--migrate` runs side effects (`bun install`, removes `pnpm-lock.yaml`,
  rewrites `pnpm-workspace.yaml`). Run it on a clean working tree under version
  control so every change is reviewable in `git diff`.
- Treat scanning an untrusted repository as you would running any tool over
  untrusted input: the file contents are parsed, not executed, but review the
  diff before committing autofixes.
- `n2b audit` and the GitHub analysis paths make network requests to the
  GitHub API. No credentials are required for public repos; a `GITHUB_TOKEN`,
  if provided via the environment, is used only for authenticated rate limits
  and is never written to disk or to the report.

## What is in scope

- Incorrect or unsafe autofixes that corrupt source or change runtime semantics.
- Path traversal / writing outside the scanned root.
- Leaking environment secrets or tokens into reports or generated files.

## What is out of scope

- Findings that require already-compromised local privileges.
- Bun or Node runtime vulnerabilities (report those upstream).
