//! `n2b wasm <subcommand>` — workflow Rust → WASM → Bun complet.
//!
//! Inspiré de :
//!   - https://rustwasm.github.io/book/game-of-life/setup.html
//!   - https://rustwasm.github.io/book/reference/tools.html
//!   - https://developer.mozilla.org/docs/WebAssembly/Guides/Rust_to_Wasm
//!   - https://rust-lang.org/what/wasm/

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum WasmTemplate {
    /// `wasm-pack-template` équivalent : lib.rs + greet() + `#[wasm_bindgen]`.
    Basic,
    /// Rustwasm book "Game of Life" : grille, tick(), canvas côté JS.
    GameOfLife,
    /// Rust+wgpu compute (WebGPU). Même contenu que `bin --flavor webgpu`.
    Wgpu,
}

impl WasmTemplate {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "basic" | "default" => Some(Self::Basic),
            "game-of-life" | "life" | "gol" => Some(Self::GameOfLife),
            "wgpu" | "webgpu" => Some(Self::Wgpu),
            _ => None,
        }
    }
}

pub enum WasmCmd {
    Init {
        name: String,
        template: WasmTemplate,
        dir: Option<PathBuf>,
        force: bool,
    },
    Doctor,
    Build {
        /// Répertoire du projet à builder (défaut: cwd).
        root: PathBuf,
        /// Target wasm-pack : web, bundler, nodejs, no-modules.
        target: String,
        /// Mode release (true) ou dev.
        release: bool,
    },
    Opt {
        /// Chemin du .wasm à optimiser (écrit en place).
        path: PathBuf,
        /// Niveau d'optimisation (-Oz, -O3, …).
        level: String,
    },
    Size {
        /// Chemin du .wasm à analyser.
        path: PathBuf,
        /// Nombre de symboles top à afficher.
        top: usize,
    },
}

pub fn run(cmd: WasmCmd, quiet: bool) -> Result<()> {
    match cmd {
        WasmCmd::Init { name, template, dir, force } => init(name, template, dir, force, quiet),
        WasmCmd::Doctor => doctor(quiet),
        WasmCmd::Build { root, target, release } => build(root, target, release, quiet),
        WasmCmd::Opt { path, level } => opt(path, level, quiet),
        WasmCmd::Size { path, top } => size(path, top, quiet),
    }
}

fn init(
    name: String,
    template: WasmTemplate,
    dir: Option<PathBuf>,
    force: bool,
    quiet: bool,
) -> Result<()> {
    let target_dir = match dir {
        Some(d) => d.join(&name),
        None => PathBuf::from(&name),
    };
    if target_dir.exists() && !force {
        anyhow::bail!(
            "{} existe déjà — relancer avec --force pour écraser",
            target_dir.display()
        );
    }
    std::fs::create_dir_all(&target_dir)
        .with_context(|| format!("mkdir {}", target_dir.display()))?;

    match template {
        WasmTemplate::Basic => basic_template(&target_dir, &name, quiet)?,
        WasmTemplate::GameOfLife => game_of_life_template(&target_dir, &name, quiet)?,
        WasmTemplate::Wgpu => wgpu_template(&target_dir, &name, quiet)?,
    }

    if !quiet {
        eprintln!(
            "[wasm init] ✓ {name} scaffolded → cd {} && bun install && bun run build",
            target_dir.display()
        );
    }
    Ok(())
}

fn doctor(quiet: bool) -> Result<()> {
    let tools: &[(&str, &str, &str)] = &[
        ("wasm-pack",       "cargo install wasm-pack",       "build Rust→WASM standard"),
        ("cargo-generate",  "cargo install cargo-generate",  "scaffolds depuis templates git"),
        ("wasm-opt",        "apt install binaryen",          "optimise .wasm (binaryen)"),
        ("twiggy",          "cargo install twiggy",          "profiler taille .wasm"),
        ("wasm2wat",        "apt install wabt",              "disassembler .wasm ↔ .wat"),
        ("wasm-run",        "cargo install wasm-run",        "dev-server hot-reload pour Rust WASM"),
        ("trunk",           "cargo install trunk",           "build tool pour Rust WASM apps (Yew/Leptos)"),
        ("wasm-bindgen",    "cargo install wasm-bindgen-cli", "CLI wasm-bindgen"),
        ("wasm-snip",       "cargo install wasm-snip",       "remove panic paths from .wasm"),
    ];
    let mut missing: Vec<&str> = Vec::new();
    for (bin, install, desc) in tools {
        let ok = which(bin).is_ok();
        if !quiet {
            let mark = if ok { "✓" } else { "✗" };
            println!("  {mark} {:<16} {desc}", bin);
            if !ok {
                println!("       install: {install}");
            }
        }
        if !ok {
            missing.push(*bin);
        }
    }
    if !missing.is_empty() && !quiet {
        eprintln!("\n{} outil(s) manquant(s) : {}", missing.len(), missing.join(", "));
    }
    Ok(())
}

fn build(root: PathBuf, target: String, release: bool, quiet: bool) -> Result<()> {
    if which("wasm-pack").is_err() {
        anyhow::bail!("wasm-pack introuvable — `cargo install wasm-pack` ou `n2b wasm doctor`");
    }
    let mut args: Vec<String> =
        vec!["build".into(), "--target".into(), target];
    if release {
        args.push("--release".into());
    } else {
        args.push("--dev".into());
    }
    if !quiet {
        eprintln!("[wasm build] wasm-pack {}", args.join(" "));
    }
    let status = Command::new("wasm-pack")
        .args(&args)
        .current_dir(&root)
        .status()
        .context("exécution wasm-pack")?;
    if !status.success() {
        anyhow::bail!("wasm-pack build a échoué (exit {})", status);
    }
    Ok(())
}

fn opt(path: PathBuf, level: String, quiet: bool) -> Result<()> {
    if which("wasm-opt").is_err() {
        anyhow::bail!("wasm-opt introuvable — `apt install binaryen` ou équivalent");
    }
    if !path.exists() {
        anyhow::bail!("{} introuvable", path.display());
    }
    let tmp = path.with_extension("wasm.tmp");
    if !quiet {
        eprintln!(
            "[wasm opt] wasm-opt {} {} -o {}",
            level,
            path.display(),
            tmp.display()
        );
    }
    let status = Command::new("wasm-opt")
        .arg(&level)
        .arg(&path)
        .arg("-o")
        .arg(&tmp)
        .status()
        .context("exécution wasm-opt")?;
    if !status.success() {
        anyhow::bail!("wasm-opt a échoué");
    }
    std::fs::rename(&tmp, &path).context("rename wasm-opt output")?;
    if !quiet {
        let size = std::fs::metadata(&path)?.len();
        eprintln!("[wasm opt] ✓ {} ({size} bytes)", path.display());
    }
    Ok(())
}

fn size(path: PathBuf, top: usize, quiet: bool) -> Result<()> {
    if which("twiggy").is_err() {
        anyhow::bail!("twiggy introuvable — `cargo install twiggy`");
    }
    if !path.exists() {
        anyhow::bail!("{} introuvable", path.display());
    }
    if !quiet {
        eprintln!("[wasm size] twiggy top {} -n {}", path.display(), top);
    }
    let status = Command::new("twiggy")
        .arg("top")
        .arg("-n")
        .arg(top.to_string())
        .arg(&path)
        .status()
        .context("exécution twiggy")?;
    if !status.success() {
        anyhow::bail!("twiggy a échoué");
    }
    Ok(())
}

// --- Templates ---

fn basic_template(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    write(
        dir.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"

[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
"#,
        ),
        quiet,
    )?;
    write(
        dir.join("src/lib.rs"),
        r#"use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}! (from Rust→WASM)")
}
"#,
        quiet,
    )?;
    write(
        dir.join("package.json"),
        &pkg_json(name, "basic WASM module for Bun"),
        quiet,
    )?;
    write(dir.join("index.ts"), BASIC_INDEX_TS, quiet)?;
    write(dir.join("README.md"), &readme(name, "Basic Rust → WASM"), quiet)?;
    write(dir.join(".gitignore"), GITIGNORE, quiet)?;
    Ok(())
}

fn game_of_life_template(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    write(
        dir.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"

[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
"#,
        ),
        quiet,
    )?;
    write(dir.join("src/lib.rs"), GAME_OF_LIFE_LIB, quiet)?;
    write(
        dir.join("package.json"),
        &pkg_json(name, "Conway's Game of Life — Rust+WASM + Bun"),
        quiet,
    )?;
    write(dir.join("index.ts"), GAME_OF_LIFE_TS, quiet)?;
    write(dir.join("README.md"), &readme(name, "Game of Life (Rustwasm book)"), quiet)?;
    write(dir.join(".gitignore"), GITIGNORE, quiet)?;
    Ok(())
}

fn wgpu_template(dir: &Path, _name: &str, _quiet: bool) -> Result<()> {
    // Déléguer à bin_cmd::scaffold_webgpu pour ne pas dupliquer le code.
    crate::bin_cmd::run_bin(crate::bin_cmd::BinOpts {
        name: dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("wasm-wgpu")
            .to_string(),
        flavor: crate::bin_cmd::BinFlavor::WebGpu,
        dir: dir.parent().map(|p| p.to_path_buf()),
        force: true,
        quiet: true,
    })
}

fn pkg_json(name: &str, desc: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "description": "{desc}",
  "type": "module",
  "main": "index.ts",
  "scripts": {{
    "build": "wasm-pack build --target web --out-dir pkg --release",
    "build:dev": "wasm-pack build --target web --out-dir pkg --dev",
    "test": "bun test"
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

fn readme(name: &str, title: &str) -> String {
    format!(
        r#"# {name}

{title}

Scaffolded by `n2b wasm init`.

## Build

```bash
bun install
bun run build       # wasm-pack release → pkg/
```

## Use from Bun

```ts
import {{ greet }} from "./index.ts";
console.log(await greet("World"));
```

## References

- https://rustwasm.github.io/book/
- https://developer.mozilla.org/docs/WebAssembly/Guides/Rust_to_Wasm
"#,
    )
}

fn write(path: PathBuf, content: &str, quiet: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("mkdir {}", parent.display()))?;
    }
    std::fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    if !quiet {
        eprintln!("[wasm]   + {}", path.display());
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
    }
    Err(anyhow!("binaire `{name}` introuvable dans PATH"))
}

// --- Template contents ---

const BASIC_INDEX_TS: &str = r#"import init, { greet } from "./pkg/index.js";
await init();
export { greet };
"#;

const GAME_OF_LIFE_LIB: &str = r#"//! Conway's Game of Life — port minimal du tutoriel rustwasm-book.
//!
//! Doc : https://rustwasm.github.io/book/game-of-life/implementing.html

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        let cells = (0..width * height)
            .map(|i| if i % 2 == 0 || i % 7 == 0 { Cell::Alive } else { Cell::Dead })
            .collect();
        Universe { width, height, cells }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn cells_ptr(&self) -> *const Cell { self.cells.as_ptr() }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row * self.width + col) as usize;
                let live = self.live_neighbors(row, col);
                next[idx] = match (self.cells[idx], live) {
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Dead, 3) => Cell::Alive,
                    _ => Cell::Dead,
                };
            }
        }
        self.cells = next;
    }

    fn live_neighbors(&self, row: u32, col: u32) -> u8 {
        let mut count = 0;
        for dr in [self.height - 1, 0, 1].iter().copied() {
            for dc in [self.width - 1, 0, 1].iter().copied() {
                if dr == 0 && dc == 0 { continue; }
                let r = (row + dr) % self.height;
                let c = (col + dc) % self.width;
                let idx = (r * self.width + c) as usize;
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}
"#;

const GAME_OF_LIFE_TS: &str = r#"import init, { Universe, Cell } from "./pkg/index.js";

await init();

const WIDTH = 64;
const HEIGHT = 64;
const uni = Universe.new(WIDTH, HEIGHT);

export function tick(): void {
  uni.tick();
}

export function dump(): string {
  const w = uni.width();
  const h = uni.height();
  let s = "";
  // Best-effort : on liste juste les dimensions. Pour le rendu pixel, lire
  // le buffer WASM via `uni.cells_ptr()` + `wasm.memory.buffer`.
  s += `${w}x${h}\n`;
  return s;
}
"#;

const GITIGNORE: &str = r#"target/
node_modules/
pkg/
pkg-bundler/
pkg-node/
.DS_Store
"#;
