//! `n2b win32 <sub>` — scaffolde des projets Bun + Win32 bas-niveau.
//!
//! Miroir de `linux_cmd.rs` pour l'écosystème Microsoft Windows :
//!   - `ffi`    : Rust cdylib (`windows` crate Microsoft) → Bun `dlopen` .dll
//!   - `cc`     : inline C avec `<windows.h>` compilé par TinyCC à runtime
//!   - `pwsh`   : scripts Bun Shell invoquant PowerShell 7 pour ops système
//!   - `init`   : projet complet mêlant les 3 approches + build.ps1 + build.sh (cross)
//!   - `doctor` : vérifie rustc+windows-msvc, cl.exe, link.exe, clang-cl,
//!                cmake, ninja, pwsh, scoop, mingw-w64 (cross-compile)
//!
//! Refs :
//!   - https://github.com/microsoft/windows-rs
//!   - https://bun.com/docs/runtime/ffi
//!   - https://bun.com/docs/runtime/c-compiler
//!   - https://bun.com/docs/runtime/shell
//!   - https://bun.com/docs/runtime/node-api
//!   - https://bun.com/docs/project/building-windows
//!   - https://github.com/marlersoft/zigwin32
//!   - https://www.npmjs.com/package/win32-api
//!   - https://learn.microsoft.com/fr-fr/windows/win32/api/
//!   - https://learn.microsoft.com/fr-fr/windows/dev-environment/rust/rust-for-windows

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

pub enum Win32Cmd {
    Init {
        name: String,
        dir: Option<PathBuf>,
        force: bool,
    },
    Ffi {
        name: String,
        dir: Option<PathBuf>,
        force: bool,
    },
    Cc {
        name: String,
        dir: Option<PathBuf>,
        force: bool,
    },
    Pwsh {
        name: String,
        dir: Option<PathBuf>,
        force: bool,
    },
    Doctor,
}

pub fn run(cmd: Win32Cmd, quiet: bool) -> Result<()> {
    match cmd {
        Win32Cmd::Init { name, dir, force } => init_all(name, dir, force, quiet),
        Win32Cmd::Ffi { name, dir, force } => scaffold(name, dir, force, quiet, Flavor::Ffi),
        Win32Cmd::Cc { name, dir, force } => scaffold(name, dir, force, quiet, Flavor::Cc),
        Win32Cmd::Pwsh { name, dir, force } => scaffold(name, dir, force, quiet, Flavor::Pwsh),
        Win32Cmd::Doctor => doctor(quiet),
    }
}

enum Flavor {
    Ffi,
    Cc,
    Pwsh,
}

fn scaffold(
    name: String,
    dir: Option<PathBuf>,
    force: bool,
    quiet: bool,
    flavor: Flavor,
) -> Result<()> {
    let target = match dir {
        Some(d) => d.join(&name),
        None => PathBuf::from(&name),
    };
    if target.exists() && !force {
        anyhow::bail!(
            "{} existe déjà — relancer avec --force",
            target.display()
        );
    }
    std::fs::create_dir_all(&target)?;
    match flavor {
        Flavor::Ffi => scaffold_ffi(&target, &name, quiet)?,
        Flavor::Cc => scaffold_cc(&target, &name, quiet)?,
        Flavor::Pwsh => scaffold_pwsh(&target, &name, quiet)?,
    }
    if !quiet {
        eprintln!(
            "[win32] ✓ {name} scaffolded → cd {} && bun install && bun run build",
            target.display()
        );
    }
    Ok(())
}

fn init_all(name: String, dir: Option<PathBuf>, force: bool, quiet: bool) -> Result<()> {
    let target = match dir {
        Some(d) => d.join(&name),
        None => PathBuf::from(&name),
    };
    if target.exists() && !force {
        anyhow::bail!(
            "{} existe déjà — relancer avec --force",
            target.display()
        );
    }
    std::fs::create_dir_all(&target)?;

    // FFI : crate Rust avec windows-rs
    write(
        target.join("crates/ffi/Cargo.toml"),
        &render_ffi_cargo_toml(&format!("{name}_ffi"), true),
        quiet,
    )?;
    write(
        target.join("crates/ffi/src/lib.rs"),
        FFI_LIB_RS_COMPLETE,
        quiet,
    )?;

    // CC : fichiers C inline
    write(target.join("c/hello.c"), CC_HELLO_C, quiet)?;
    write(target.join("c/winapi.c"), CC_WINAPI_C, quiet)?;

    // PowerShell scripts Bun
    write(target.join("scripts/sysinfo.ts"), PWSH_SYSINFO_TS, quiet)?;
    write(target.join("scripts/ops.ts"), PWSH_OPS_TS, quiet)?;

    // Entrypoints TS
    write(target.join("src/ffi.ts"), &render_ffi_index_ts(&name), quiet)?;
    write(target.join("src/cc.ts"), CC_INDEX_TS, quiet)?;
    write(target.join("src/pwsh.ts"), PWSH_INDEX_TS, quiet)?;
    write(target.join("src/main.ts"), &render_main_ts(&name), quiet)?;

    // Package + scripts de build
    write(target.join("package.json"), &render_all_package_json(&name), quiet)?;
    write(target.join("README.md"), &render_all_readme(&name), quiet)?;
    write(target.join(".gitignore"), GITIGNORE, quiet)?;
    write(target.join("build.ps1"), &render_build_ps1(&name), quiet)?;
    write(target.join("build.sh"), &render_build_sh_cross(&name), quiet)?;
    write(target.join("cross-compile.md"), CROSS_COMPILE_MD, quiet)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for f in ["build.sh", "build.ps1"] {
            if let Ok(meta) = std::fs::metadata(target.join(f)) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(target.join(f), perms);
            }
        }
    }

    if !quiet {
        eprintln!(
            "[win32 init] ✓ {name} complet → cd {} && ./build.ps1 (Windows) ou ./build.sh (cross)",
            target.display()
        );
    }
    Ok(())
}

fn scaffold_ffi(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    write(
        dir.join("Cargo.toml"),
        &render_ffi_cargo_toml(name, false),
        quiet,
    )?;
    write(dir.join("src/lib.rs"), FFI_LIB_RS_SIMPLE, quiet)?;
    write(
        dir.join("package.json"),
        &render_ffi_package_json(name),
        quiet,
    )?;
    write(dir.join("index.ts"), &render_ffi_index_ts_simple(name), quiet)?;
    write(
        dir.join("README.md"),
        &readme(name, "Rust cdylib (windows-rs) → Bun FFI dlopen .dll"),
        quiet,
    )?;
    write(dir.join(".gitignore"), GITIGNORE, quiet)?;
    write(dir.join("build.ps1"), &render_build_ps1_simple(name), quiet)?;
    write(dir.join("build.sh"), &render_build_sh_simple(name), quiet)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for f in ["build.sh", "build.ps1"] {
            if let Ok(meta) = std::fs::metadata(dir.join(f)) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(dir.join(f), perms);
            }
        }
    }
    Ok(())
}

fn scaffold_cc(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    write(dir.join("hello.c"), CC_HELLO_C, quiet)?;
    write(dir.join("winapi.c"), CC_WINAPI_C, quiet)?;
    write(
        dir.join("package.json"),
        &render_cc_package_json(name),
        quiet,
    )?;
    write(dir.join("index.ts"), CC_INDEX_TS, quiet)?;
    write(
        dir.join("README.md"),
        &readme(
            name,
            "Inline C avec <windows.h> compilé par TinyCC (bun:ffi `cc`)",
        ),
        quiet,
    )?;
    write(dir.join(".gitignore"), GITIGNORE, quiet)?;
    Ok(())
}

fn scaffold_pwsh(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    write(
        dir.join("package.json"),
        &render_pwsh_package_json(name),
        quiet,
    )?;
    write(dir.join("sysinfo.ts"), PWSH_SYSINFO_TS, quiet)?;
    write(dir.join("ops.ts"), PWSH_OPS_TS, quiet)?;
    write(
        dir.join("README.md"),
        &readme(name, "Ops système via Bun Shell + PowerShell 7 (`$\\`pwsh ...\\``)"),
        quiet,
    )?;
    write(dir.join(".gitignore"), GITIGNORE, quiet)?;
    Ok(())
}

fn doctor(quiet: bool) -> Result<()> {
    let tools: &[(&str, &str, &str)] = &[
        ("rustc",      "winget install Rustlang.Rust.MSVC",         "compilateur Rust (target x86_64-pc-windows-msvc)"),
        ("cargo",      "(installé avec rustup)",                    "package manager Rust"),
        ("cl",         "VS 2022 Community + workload C++",          "MSVC compiler (cl.exe)"),
        ("link",       "VS 2022 Community",                         "MSVC linker (link.exe)"),
        ("clang-cl",   "scoop install llvm",                        "clang drop-in pour MSVC ABI"),
        ("cmake",      "winget install Kitware.CMake",              "build system"),
        ("ninja",      "scoop install ninja",                       "ninja build backend"),
        ("pwsh",       "winget install Microsoft.PowerShell",       "PowerShell 7 (pwsh.exe)"),
        ("scoop",      "iwr -useb get.scoop.sh | iex",              "package manager Windows"),
        ("x86_64-w64-mingw32-gcc", "sudo apt install mingw-w64",    "cross-compile Linux→Windows (GNU)"),
        ("cargo-xwin", "cargo install cargo-xwin",                  "cross-compile Linux→Windows (MSVC)"),
    ];
    let mut missing = 0;
    for (bin, install, desc) in tools {
        let ok = which(bin).is_ok();
        if !quiet {
            let mark = if ok { "✓" } else { "✗" };
            println!("  {mark} {:<25} {desc}", bin);
            if !ok {
                println!("       install: {install}");
                missing += 1;
            }
        }
    }
    if !quiet {
        if missing > 0 {
            eprintln!("\n{missing} outil(s) manquant(s)");
        }
        eprintln!("\nRust target :");
        let _ = std::process::Command::new("rustup")
            .args(["target", "list", "--installed"])
            .status();
        eprintln!("\nPour ajouter les cibles Windows :");
        eprintln!("  rustup target add x86_64-pc-windows-msvc   # natif (Windows ou cargo-xwin)");
        eprintln!("  rustup target add x86_64-pc-windows-gnu    # mingw (Linux-friendly)");
        eprintln!("  rustup target add aarch64-pc-windows-msvc  # ARM64 Windows");
    }
    Ok(())
}

// --- Renderers ---

fn render_ffi_cargo_toml(name: &str, richer: bool) -> String {
    let features = if richer {
        r#"[
    "Win32_Foundation",
    "Win32_System_SystemInformation",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Storage_FileSystem",
]"#
    } else {
        r#"[
    "Win32_Foundation",
    "Win32_System_SystemInformation",
    "Win32_System_Threading",
    "Win32_UI_WindowsAndMessaging",
]"#
    };
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
windows = {{ version = "0.58", features = {features} }}

[profile.release]
lto = true
codegen-units = 1
strip = "symbols"
"#,
    )
}

fn render_ffi_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "build": "powershell -NoProfile -File ./build.ps1 || ./build.sh",
    "build:win": "powershell -NoProfile -File ./build.ps1",
    "build:cross": "./build.sh",
    "start": "bun run index.ts"
  }},
  "devDependencies": {{
    "@types/bun": "latest"
  }},
  "engines": {{
    "bun": ">=1.2.0"
  }}
}}
"#,
    )
}

fn render_cc_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "start": "bun run index.ts"
  }},
  "devDependencies": {{
    "@types/bun": "latest"
  }},
  "engines": {{
    "bun": ">=1.2.0"
  }}
}}
"#,
    )
}

fn render_pwsh_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "start": "bun run sysinfo.ts",
    "ops": "bun run ops.ts"
  }},
  "devDependencies": {{
    "@types/bun": "latest"
  }},
  "engines": {{
    "bun": ">=1.2.0"
  }}
}}
"#,
    )
}

fn render_all_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "build": "powershell -NoProfile -File ./build.ps1 || ./build.sh",
    "build:win": "powershell -NoProfile -File ./build.ps1",
    "build:cross": "./build.sh",
    "start": "bun run src/main.ts",
    "ffi": "bun run src/ffi.ts",
    "cc": "bun run src/cc.ts",
    "pwsh": "bun run src/pwsh.ts",
    "sysinfo": "bun run scripts/sysinfo.ts",
    "ops": "bun run scripts/ops.ts"
  }},
  "devDependencies": {{
    "@types/bun": "latest"
  }},
  "engines": {{
    "bun": ">=1.2.0"
  }}
}}
"#,
    )
}

fn render_all_readme(name: &str) -> String {
    format!(
        r#"# {name}

Projet Windows-native complet : Bun + Rust FFI (windows-rs) + C inline
+ Bun Shell (PowerShell 7).

## Structure

```
{name}/
├── crates/ffi/         → Rust cdylib (windows 0.58, GetTickCount, MessageBoxW…)
├── c/                  → Fichiers C compilés à runtime par TinyCC (<windows.h>)
├── scripts/            → Scripts Bun Shell utilisant pwsh.exe
├── src/
│   ├── ffi.ts          → dlopen({name}_ffi.dll) via bun:ffi
│   ├── cc.ts           → cc() compile winapi.c à runtime
│   ├── pwsh.ts         → Wrapper typé PowerShell via $`pwsh -Command …`
│   └── main.ts         → Orchestration
├── build.ps1           → Build natif Windows (cargo build --release)
├── build.sh            → Cross-compile depuis Linux (mingw ou cargo-xwin)
└── cross-compile.md    → Guide cross-compilation
```

## Build

### Natif sur Windows
```powershell
./build.ps1
bun run src/main.ts
```

### Cross-compile depuis Linux / macOS
```bash
# Option 1 : MinGW (GNU ABI) — simple, pas de dep Windows
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64
./build.sh

# Option 2 : cargo-xwin (MSVC ABI) — compatible .dll Windows officielles
cargo install cargo-xwin
cargo xwin build --release --target x86_64-pc-windows-msvc
```

Voir `cross-compile.md` pour le détail.

## Tester sur Linux sans Windows

Le scaffold compile via `build.sh` et produit un `.dll`. Pour l'exécuter,
utilisez Wine :

```bash
wine64 bun.exe src/main.ts    # avec une version Windows de Bun
```

Ou copiez le projet + `.dll` vers une VM/machine Windows.

## Docs

- [bun:ffi](https://bun.com/docs/runtime/ffi)
- [bun:cc](https://bun.com/docs/runtime/c-compiler)
- [Bun Shell](https://bun.com/docs/runtime/shell)
- [Bun Node-API](https://bun.com/docs/runtime/node-api)
- [microsoft/windows-rs](https://github.com/microsoft/windows-rs)
- [Rust for Windows](https://learn.microsoft.com/fr-fr/windows/dev-environment/rust/rust-for-windows)
- [Win32 API reference](https://learn.microsoft.com/fr-fr/windows/win32/api/)
- [Building Bun on Windows](https://bun.com/docs/project/building-windows)
"#,
    )
}

fn render_build_ps1(name: &str) -> String {
    format!(
        r#"# Build natif Windows via cargo (MSVC).
# Requiert : VS 2022 Community + "Desktop Development with C++" + rustc.

$ErrorActionPreference = "Stop"

Push-Location "$PSScriptRoot/crates/ffi"
cargo build --release
Pop-Location

New-Item -ItemType Directory -Force -Path "$PSScriptRoot/lib" | Out-Null
Copy-Item "$PSScriptRoot/crates/ffi/target/release/{name}_ffi.dll" `
  -Destination "$PSScriptRoot/lib/{name}_ffi.dll" -Force

Write-Host "✓ lib/{name}_ffi.dll built"
"#,
    )
}

fn render_build_ps1_simple(name: &str) -> String {
    format!(
        r#"# Build natif Windows via cargo (MSVC).
$ErrorActionPreference = "Stop"
cargo build --release
Copy-Item "target/release/{name}.dll" -Destination "./{name}.dll" -Force
Write-Host "✓ {name}.dll built"
"#,
    )
}

fn render_build_sh_cross(name: &str) -> String {
    format!(
        r#"#!/usr/bin/env bash
# Cross-compile depuis Linux/macOS vers Windows.
# Essaie cargo-xwin (MSVC ABI) d'abord, fallback MinGW (GNU ABI).
set -euo pipefail

mkdir -p lib
cd crates/ffi

if command -v cargo-xwin >/dev/null 2>&1; then
    echo "▶ cargo-xwin build (MSVC ABI)"
    cargo xwin build --release --target x86_64-pc-windows-msvc
    cp target/x86_64-pc-windows-msvc/release/{name}_ffi.dll ../../lib/
elif rustup target list --installed | grep -q x86_64-pc-windows-gnu; then
    echo "▶ cargo build --target x86_64-pc-windows-gnu (MinGW ABI)"
    cargo build --release --target x86_64-pc-windows-gnu
    cp target/x86_64-pc-windows-gnu/release/{name}_ffi.dll ../../lib/
else
    echo "✗ Ni cargo-xwin ni la target x86_64-pc-windows-gnu ne sont disponibles."
    echo "  Installer :"
    echo "    cargo install cargo-xwin                              # MSVC ABI (recommandé)"
    echo "    rustup target add x86_64-pc-windows-gnu && apt install mingw-w64"
    exit 1
fi

echo "✓ lib/{name}_ffi.dll built"
"#,
    )
}

fn render_build_sh_simple(name: &str) -> String {
    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail
if command -v cargo-xwin >/dev/null 2>&1; then
    cargo xwin build --release --target x86_64-pc-windows-msvc
    cp target/x86_64-pc-windows-msvc/release/{name}.dll ./{name}.dll
else
    cargo build --release --target x86_64-pc-windows-gnu
    cp target/x86_64-pc-windows-gnu/release/{name}.dll ./{name}.dll
fi
echo "✓ {name}.dll built"
"#,
    )
}

fn render_ffi_index_ts(name: &str) -> String {
    format!(
        r#"// Charge le cdylib Windows via bun:ffi dlopen.
// La .dll est produite par ./build.ps1 (natif) ou ./build.sh (cross).

import {{ dlopen, FFIType, suffix }} from "bun:ffi";

const {{ u32, u32: U32 }} = FFIType;

// Sur Windows, suffix === "dll". Sur Linux (debug/test), .so est produit.
const libPath = `./lib/{name}_ffi.${{suffix}}`;

const lib = dlopen(libPath, {{
  add: {{ args: [u32, u32], returns: u32 }},
  tick_count: {{ args: [], returns: u32 }},
  process_id: {{ args: [], returns: u32 }},
  page_size: {{ args: [], returns: u32 }},
  number_of_processors: {{ args: [], returns: u32 }},
}});

export const add = (a: number, b: number) => lib.symbols.add(a, b);
export const tickCount = () => lib.symbols.tick_count();
export const processId = () => lib.symbols.process_id();
export const pageSize = () => lib.symbols.page_size();
export const numberOfProcessors = () => lib.symbols.number_of_processors();
"#,
    )
}

fn render_ffi_index_ts_simple(name: &str) -> String {
    format!(
        r#"import {{ dlopen, FFIType, suffix }} from "bun:ffi";
const {{ u32 }} = FFIType;

const lib = dlopen(`./{name}.${{suffix}}`, {{
  add: {{ args: [u32, u32], returns: u32 }},
  tick_count: {{ args: [], returns: u32 }},
  process_id: {{ args: [], returns: u32 }},
}});

export const add = (a: number, b: number) => lib.symbols.add(a, b);
export const tickCount = () => lib.symbols.tick_count();
export const processId = () => lib.symbols.process_id();
"#,
    )
}

fn render_main_ts(name: &str) -> String {
    format!(
        r#"// Orchestration Bun + Win32 : FFI cdylib + inline C + PowerShell.

import {{ add, tickCount, processId, numberOfProcessors }} from "./ffi.ts";
import {{ answer, currentPid }} from "./cc.ts";
import {{ osName, cpuName, ramGB }} from "./pwsh.ts";

console.log("[{name}]");
console.log("");
console.log("── Rust FFI (windows-rs) ──");
console.log("  add(2, 3)               =", add(2, 3));
console.log("  GetTickCount()          =", tickCount(), "ms since boot");
console.log("  GetCurrentProcessId()   =", processId());
console.log("  nb processors           =", numberOfProcessors());
console.log("");
console.log("── Inline C (TinyCC) ──");
console.log("  answer()                =", answer());
console.log("  GetCurrentProcessId()   =", currentPid());
console.log("");
console.log("── PowerShell via Bun Shell ──");
console.log("  OS                      =", await osName());
console.log("  CPU                     =", await cpuName());
console.log("  RAM                     =", await ramGB(), "GB");
"#,
    )
}

fn readme(name: &str, title: &str) -> String {
    format!(
        r#"# {name}

{title}

Scaffolded by `n2b win32`.

## Build & Run

### Natif sur Windows
```powershell
bun install
./build.ps1
bun run start
```

### Cross-compile depuis Linux / macOS
```bash
bun install
./build.sh
bun run start     # avec Bun for Windows (via Wine si besoin)
```

## Docs

- https://github.com/microsoft/windows-rs
- https://bun.com/docs/runtime/ffi
- https://bun.com/docs/runtime/c-compiler
- https://bun.com/docs/runtime/shell
- https://learn.microsoft.com/fr-fr/windows/win32/api/
"#,
    )
}

fn write(path: PathBuf, content: &str, quiet: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }
    std::fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    if !quiet {
        eprintln!("[win32]   + {}", path.display());
    }
    Ok(())
}

fn which(name: &str) -> Result<PathBuf> {
    let path = std::env::var_os("PATH").ok_or_else(|| anyhow!("PATH non défini"))?;
    for dir in std::env::split_paths(&path) {
        let p = dir.join(name);
        if p.is_file() {
            return Ok(p);
        }
        let p_exe = dir.join(format!("{name}.exe"));
        if p_exe.is_file() {
            return Ok(p_exe);
        }
    }
    Err(anyhow!("binaire `{name}` introuvable dans PATH"))
}

// --- Templates ---

const FFI_LIB_RS_SIMPLE: &str = r#"//! Rust cdylib Windows minimal : add + GetTickCount + GetCurrentProcessId.
//!
//! Compilation :
//!   Windows    : cargo build --release
//!   Linux/mac  : cargo xwin build --release --target x86_64-pc-windows-msvc
//!              : ou cargo build --release --target x86_64-pc-windows-gnu

use std::os::raw::c_uint;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::Threading::GetCurrentProcessId;

#[no_mangle]
pub extern "C" fn add(a: c_uint, b: c_uint) -> c_uint {
    a + b
}

#[no_mangle]
pub extern "C" fn tick_count() -> c_uint {
    unsafe { GetTickCount() }
}

#[no_mangle]
pub extern "C" fn process_id() -> c_uint {
    unsafe { GetCurrentProcessId() }
}
"#;

const FFI_LIB_RS_COMPLETE: &str = r#"//! Rust cdylib riche : Win32 exposé via la crate `windows` (Microsoft).
//!
//! Features Cargo activées :
//!   Win32_Foundation, Win32_System_{SystemInformation,Threading,ProcessStatus},
//!   Win32_UI_WindowsAndMessaging, Win32_Storage_FileSystem.

use std::os::raw::{c_char, c_uint};
use windows::core::PCWSTR;
use windows::Win32::System::SystemInformation::{
    GetSystemInfo, GetTickCount, SYSTEM_INFO,
};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

#[no_mangle]
pub extern "C" fn add(a: c_uint, b: c_uint) -> c_uint {
    a + b
}

#[no_mangle]
pub extern "C" fn tick_count() -> c_uint {
    unsafe { GetTickCount() }
}

#[no_mangle]
pub extern "C" fn process_id() -> c_uint {
    unsafe { GetCurrentProcessId() }
}

#[no_mangle]
pub extern "C" fn page_size() -> c_uint {
    let mut info = SYSTEM_INFO::default();
    unsafe { GetSystemInfo(&mut info) };
    info.dwPageSize
}

#[no_mangle]
pub extern "C" fn number_of_processors() -> c_uint {
    let mut info = SYSTEM_INFO::default();
    unsafe { GetSystemInfo(&mut info) };
    info.dwNumberOfProcessors
}

/// Affiche une MessageBox. `text` et `title` sont des pointeurs UTF-16 nul-terminés
/// côté appelant ; passer un pointeur Bun (via CString) nécessite une conversion
/// UTF-8 → UTF-16. Exposée à titre de démo ; prefer l'appeler depuis code Rust.
///
/// # Safety
/// `text` et `title` doivent être des pointeurs UTF-16 nul-terminés valides.
#[no_mangle]
pub unsafe extern "C" fn message_box_w(text: *const u16, title: *const u16) -> c_uint {
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(text),
            PCWSTR(title),
            MB_OK | MB_ICONINFORMATION,
        )
        .0 as c_uint
    }
}

// Évite un warning sur c_char inutilisé.
#[allow(dead_code)]
fn _unused(_: c_char) {}
"#;

const CC_HELLO_C: &str = r#"// Fichier C compilé à runtime par Bun (TinyCC).

int answer(void) {
    return 42;
}

int add(int a, int b) {
    return a + b;
}
"#;

const CC_WINAPI_C: &str = r#"// Appels Win32 depuis C compilé par TinyCC à runtime.
// Les headers <windows.h> sont fournis par le compilateur embarqué.

#include <windows.h>

unsigned long current_pid(void) {
    return GetCurrentProcessId();
}

unsigned long tick_count(void) {
    return GetTickCount();
}

// Exemple : ouvrir une MessageBox. Blocage UI — uniquement en démo.
int message_box(void) {
    return MessageBoxA(NULL, "Hello from inline C!", "n2b win32 cc", MB_OK);
}
"#;

const CC_INDEX_TS: &str = r#"// Compile du C Win32 à runtime et appelle les fonctions depuis Bun.

import { cc } from "bun:ffi";
import hello from "./hello.c" with { type: "file" };
import winapi from "./winapi.c" with { type: "file" };

const helloLib = cc({
  source: hello,
  symbols: {
    answer: { args: [], returns: "int" },
    add: { args: ["int", "int"], returns: "int" },
  },
});

const winLib = cc({
  source: winapi,
  symbols: {
    current_pid: { args: [], returns: "u32" },
    tick_count: { args: [], returns: "u32" },
    // message_box: { args: [], returns: "int" }, // décommente pour pop-up
  },
});

export const answer = () => helloLib.symbols.answer();
export const add = (a: number, b: number) => helloLib.symbols.add(a, b);
export const currentPid = () => winLib.symbols.current_pid();
export const tickCount = () => winLib.symbols.tick_count();
"#;

const PWSH_SYSINFO_TS: &str = r#"// Info système Windows via Bun Shell + PowerShell 7.
// Doc : https://bun.com/docs/runtime/shell

import { $ } from "bun";

const PS = "powershell";  // ou "pwsh" si PowerShell 7 installé

async function run(cmd: string): Promise<string> {
  return (await $`${PS} -NoProfile -NonInteractive -Command ${cmd}`.text()).trim();
}

console.log("── Windows system overview ──");
console.log("OS      :", await run("(Get-CimInstance Win32_OperatingSystem).Caption"));
console.log("Version :", await run("(Get-CimInstance Win32_OperatingSystem).Version"));
console.log("CPU     :", await run("(Get-CimInstance Win32_Processor).Name"));
console.log(
  "RAM     :",
  await run("[math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB,1)"),
  "GB",
);
console.log(
  "Disk C: :",
  await run(
    "[math]::Round((Get-PSDrive C).Used/1GB,1).ToString() + '/' + [math]::Round(((Get-PSDrive C).Used+(Get-PSDrive C).Free)/1GB,1).ToString()",
  ),
  "GB",
);
console.log("Uptime  :", await run("((Get-Date) - (Get-CimInstance Win32_OperatingSystem).LastBootUpTime).ToString()"));

console.log("\n── Top 5 processus par CPU ──");
await $`${PS} -NoProfile -Command "Get-Process | Sort-Object CPU -Descending | Select-Object -First 5 Name,Id,CPU | Format-Table"`;
"#;

const PWSH_OPS_TS: &str = r#"// Exemples d'opérations admin Windows via Bun Shell.

import { $ } from "bun";

// Liste les services en running
console.log("── Services actifs ──");
await $`powershell -NoProfile -Command "Get-Service | Where-Object Status -eq 'Running' | Select -First 10 Name,DisplayName"`;

// Événements récents (journal Application)
console.log("\n── Derniers événements Application (5) ──");
await $`powershell -NoProfile -Command "Get-EventLog -LogName Application -Newest 5 | Format-Table TimeGenerated,Source,EntryType -Auto"`;

// Variables d'environnement critiques
console.log("\n── Env ──");
console.log("USERPROFILE :", Bun.env.USERPROFILE);
console.log("PATH (head) :", (Bun.env.PATH ?? "").split(";").slice(0, 5).join(";"));
"#;

const PWSH_INDEX_TS: &str = r#"// Wrapper typé pour les 3 infos système basiques (utilisé par main.ts).

import { $ } from "bun";

const PS = "powershell";

async function run(cmd: string): Promise<string> {
  return (await $`${PS} -NoProfile -Command ${cmd}`.text()).trim();
}

export const osName   = () => run("(Get-CimInstance Win32_OperatingSystem).Caption");
export const cpuName  = () => run("(Get-CimInstance Win32_Processor).Name");
export const ramGB    = async () =>
  Number(await run("[math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB,1)"));
"#;

const CROSS_COMPILE_MD: &str = r#"# Cross-compilation Linux/macOS → Windows

Deux approches supportées par ce scaffold :

## Option 1 — cargo-xwin (MSVC ABI, recommandé)

Reproduit exactement ce que produit Visual Studio :

```bash
cargo install cargo-xwin
rustup target add x86_64-pc-windows-msvc
cargo xwin build --release --target x86_64-pc-windows-msvc
```

cargo-xwin télécharge automatiquement les headers Windows SDK + CRT MSVC
sous licence (accepte l'EULA au premier run).

**Avantages** : ABI MSVC native, compatible avec .dll Windows officielles,
linkable par code compilé avec `cl.exe`.

## Option 2 — MinGW-w64 (GNU ABI)

Plus simple, pas de téléchargement EULA :

```bash
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64   # Debian/Ubuntu
brew install mingw-w64       # macOS
cargo build --release --target x86_64-pc-windows-gnu
```

**Avantages** : setup trivial, pas d'accord EULA.
**Limitations** : ABI différente (GNU), quelques incompat rares avec crates
qui link contre MSVCRT.

## Tester sans machine Windows

1. **Wine64** (couvre 90% des cas FFI Bun) :
   ```bash
   apt install wine
   wine64 bun.exe src/main.ts
   ```

2. **VM / Docker Windows** : GitHub Actions `windows-latest` runner est
   gratuit pour les repos publics — utile pour CI.

3. **Remote** : Azure DevBox, GitHub Codespaces avec image Windows.

## Refs

- https://github.com/rust-cross/cargo-xwin
- https://rust-lang.github.io/rustup/cross-compilation.html
- https://bun.com/docs/project/building-windows
"#;

const GITIGNORE: &str = r#"target/
node_modules/
*.dll
*.so
*.dylib
lib/
dist/
.DS_Store
"#;
