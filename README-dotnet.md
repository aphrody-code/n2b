# n2b — `dotnet` branch

> Windows 11 + dotnet 10 + node-api-dotnet + WinClean-specific migration rules,
> isolated on a dedicated branch.

This branch extends `main` with four new rule modules tuned for the WinClean
toolchain (NativeAOT, PowerShell payload, Bun ⇄ C# interop via
node-api-dotnet). It does **not** modify any existing rule, scanner, or frozen
baseline — `main` stays consumable as-is by every downstream client (notably
`rpb-dashboard`).

## What this branch adds

Four new modules under `crates/n2b-rules/src/`:

| Module | Rule ID space | Targets | Aggressive-only rules |
|---|---|---|---|
| `dotnet.rs` | `DN001-DN008` | `.csproj`, `.cs`, `.ps1`, shell scripts referencing the dotnet toolchain | 3 |
| `windows.rs` | `WN001-WN010` | `.ps1`, `.psm1`, `.cmd`, `.bat`, `.ts`/`.js` invoking Windows APIs | 2 |
| `node_api_dotnet.rs` | `NA001-NA005` | `.ts`/`.js` loading `node-api-dotnet`, `.csproj` referencing `Microsoft.JavaScript.NodeApi` | 1 |
| `winclean.rs` | `WC001-WC006` | Anything in a WinClean checkout: IPC patterns, hardcoded paths, dangerous service edits | 2 |

**Total: 29 rules**, 8 of which require `--aggressive` (more invasive rewrites).

Every rule follows the same shape as `cli_commands.rs`:
- A `Mapping { re, replace, rule_id, message, aggressive }`.
- A `Lazy<Vec<Mapping>>` compiled once at first access.
- A public `apply_<module>_rules(path, source, aggressive) -> (Vec<Finding>, String)`.
- Comment-prefix detection (`#`, `//`, `<!--`) skips commented lines.

## Rule catalogue

### `dotnet.rs` — `DN0xx`

| ID | Description | Aggressive |
|---|---|---|
| `DN001` | `nuget.exe restore` → `dotnet restore` | no |
| `DN002` | `MSBuild.exe` → `dotnet msbuild` | no |
| `DN003` | csproj `Include="Newtonsoft.Json"` → `Include="System.Text.Json"` | yes |
| `DN004` | `JsonConvert.DeserializeObject<T>(` → `JsonSerializer.Deserialize<T>(` | yes |
| `DN005` | `JsonConvert.SerializeObject(` → `JsonSerializer.Serialize(` | yes |
| `DN006` | `<TargetFramework>net6.0</TargetFramework>` → `net10.0` | no |
| `DN007` | `<TargetFramework>net7.0</TargetFramework>` → `net10.0` | no |
| `DN008` | `dotnet publish <proj>` (no `-r`) → add `-r win-x64` for AOT | yes |

### `windows.rs` — `WN0xx`

| ID | Description | Aggressive |
|---|---|---|
| `WN001` | `Get-WmiObject` → `Get-CimInstance` | no |
| `WN002` | `Invoke-WmiMethod` → `Invoke-CimMethod` | no |
| `WN003` | `Get-EventLog` → `Get-WinEvent` | no |
| `WN004` | `powershell.exe` → `pwsh.exe` (PS 7+) | no |
| `WN005` | Hardcoded `C:\Users\<name>\` → `$env:USERPROFILE` | yes |
| `WN006` | PS `$arr += @(item)` → `[List[T]].Add(item)` (perf O(n²)) | no |
| `WN007` | `Get-ChildItem -Recurse` on system path → `Directory.EnumerateFiles` | yes |
| `WN008` | `HKEY_LOCAL_MACHINE\` → `HKLM:\` (PSDrive prefix) | no |
| `WN009` | `HKEY_CURRENT_USER\` → `HKCU:\` | no |
| `WN010` | `#!/usr/bin/env node` shebang → `bun` | no |

### `node_api_dotnet.rs` — `NA0xx`

| ID | Description | Aggressive |
|---|---|---|
| `NA001` | Static `import dotnet from 'node-api-dotnet/net10.0'` → use direct `process.dlopen` | yes |
| `NA002` | `require('node-api-dotnet/net10.0')` → downgrade to `net9.0` | no |
| `NA003` | `PackageReference Microsoft.JavaScript.NodeApi Version="0.10.x"` → `0.9.19` | no |
| `NA004` | `[JSExport]` method returning `object` → flag (use typed DTO class) | no |
| `NA005` | `Microsoft.JavaScript.NodeApi` without companion `Microsoft.JavaScript.NodeApi.Generator` | no |

### `winclean.rs` — `WC0xx`

| ID | Description | Aggressive |
|---|---|---|
| `WC001` | `Bun.spawn(["Winclean.Mcp.exe"])` → prefer in-process Winclean.Bun call | no |
| `WC002` | Hardcoded `C:\winclean\…` path → `process.env.WINCLEAN_ROOT` | yes |
| `WC003` | `Stop-Process -Name X` without try/catch → wrap (PPL-protected services throw) | yes |
| `WC004` | `JsonSerializer.Serialize(obj)` without context → AOT violation (use `McpJsonContext`) | no |
| `WC005` | `Process.GetProcesses()` → `ProcessManager.ScanProcesses()` (1 syscall) | no |
| `WC006` | `Set-Service -StartupType Disabled` on never-disable list (`WdiSystemHost`, `hns`, `HvHost`, …) | no |

The never-disable list is derived from the WinClean memory file
`feedback_never-disable-windows-services.md` and the Microsoft IoT Enterprise
services guide.

## How to use these rules

Today the modules are **not** wired into the default `scan_source` pipeline
because doing so would change the frozen JSON/JSONL/SARIF baselines that
`rpb-dashboard` consumes. Three usage paths:

### 1. Call directly from custom Rust code

```rust
use n2b_rules::{dotnet, windows, node_api_dotnet, winclean};

let (findings, rewritten) = dotnet::apply_dotnet_rules("foo.csproj", source, aggressive);
let (findings2, rewritten) = windows::apply_windows_rules("bar.ps1", &rewritten, aggressive);
```

### 2. Add a future `--dotnet` CLI flag

Wire each module into a new branch of `crates/n2b-cli/src/commands/scan.rs`
under a flag, e.g.:

```rust
if opts.dotnet {
    let (f, w) = dotnet::apply_dotnet_rules(path, &working, aggressive);
    findings.extend(f);
    working = w;
    // … same for windows / node_api_dotnet / winclean
}
```

This keeps `main`'s default scanning behaviour byte-stable.

### 3. Manifest opt-in (`n2b.json`)

Add `"profile": "dotnet"` to `n2b.json` at the project root and let the
manifest resolver activate the four modules. Requires touching
`crates/n2b-core/src/manifest.rs` — a follow-up commit, not in this PR.

## Why these rules are sound for WinClean specifically

- **AOT-first**: WinClean's `Directory.Build.props` enforces `<PublishAot>true</PublishAot>`
  for every `src/Winclean.*` project. Rules `DN003`, `DN004`, `DN005`, `WC004` all
  push toward source-generated JSON which is the only AOT-safe path.
- **Untrusted-code awareness**: `WC003`, `WC006` come from the fact-check session
  of 2026-05-17 that identified `WdiSystemHost` / `hns` / `HvHost` as Microsoft
  "don't disable" services. `WC003` mirrors the `try/catch` pattern from
  `scripts/payload/Optimize-GamingServices.ps1`.
- **Bun ⇄ C# in-process**: `WC001` reflects the fact that `src/Winclean.Bun/` is
  now AOT-compiled to `bin/Winclean.Bun.node` and loaded by Bun via
  `process.dlopen` — bypassing the MCP stdio overhead used previously.
- **node-api-dotnet 0.9.x reality**: the official npm package ships `net8.0`/
  `net9.0` only. `NA001-NA003` keep callers off the unpublished `net10.0` TFM
  variant.

## CI

This branch adds `.github/workflows/dotnet.yml` running on `windows-2025` (and
nightly canary on `windows-latest`) which:

1. Installs Rust 1.95 (workspace MSRV).
2. `cargo test -p n2b-rules --tests` for the new modules.
3. Runs `n2b --dotnet` against `tests/fixtures/winclean-snippets/` (planned —
   add fixtures in a follow-up).
4. Verifies the four module-level `apply_*` functions are still exported
   (`cargo doc --no-deps -p n2b-rules` succeeds).

No `cargo test --workspace` here — that lives in `ci.yml` on `main` and would
re-run the frozen-baseline diff which is intentionally untouched on this branch.

## Roadmap

| Step | Status |
|---|---|
| 4 rule modules + `lib.rs` exports | done |
| `README-dotnet.md` (this file) | done |
| `.github/workflows/dotnet.yml` | done |
| Per-module unit tests (`#[test]` in each `.rs`) | TODO |
| CLI flag `--dotnet` wiring | TODO |
| Manifest profile `"profile": "dotnet"` | TODO |
| Migrate rules to data-driven TOML in `n2b-registry/registry/dotnet.toml` | TODO |
| Snapshot fixtures for WinClean code patterns | TODO |

## Merging back to `main`

The branch is intentionally narrow: no modification to existing rules, no
modification to the frozen schema, no modification to the existing scanner
dispatch. A merge to `main` only adds files (`*.rs`, `README-dotnet.md`,
`dotnet.yml`) plus four `pub mod` lines in `lib.rs`. The baselines should not
shift.
