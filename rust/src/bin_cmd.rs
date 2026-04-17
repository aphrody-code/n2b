use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

pub struct BinOpts {
    pub name: String,
    pub dir: Option<PathBuf>,
    pub flavor: BinFlavor,
    pub force: bool,
    pub quiet: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinFlavor {
    /// Plugin natif Bun (Rust → .node via NAPI, bun-native-plugin).
    NativePlugin,
    /// Exemple MDX → JSX (mdxjs-rs via plugin natif Bun).
    Mdx,
    /// Module WASM (wasm-pack / wasm-bindgen) invoqué depuis Bun.
    Wasm,
}

impl BinFlavor {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "native" | "native-plugin" | "plugin" => Some(Self::NativePlugin),
            "mdx" => Some(Self::Mdx),
            "wasm" => Some(Self::Wasm),
            _ => None,
        }
    }
}

pub fn run_bin(opts: BinOpts) -> Result<()> {
    let target_dir = match opts.dir {
        Some(d) => d.join(&opts.name),
        None => PathBuf::from(&opts.name),
    };

    if target_dir.exists() {
        if !opts.force {
            anyhow::bail!(
                "{} existe déjà — relancer avec --force pour écraser",
                target_dir.display()
            );
        }
        if !opts.quiet {
            eprintln!("[bin] écrasement de {}", target_dir.display());
        }
    }

    std::fs::create_dir_all(&target_dir)
        .with_context(|| format!("création de {}", target_dir.display()))?;

    match opts.flavor {
        BinFlavor::NativePlugin => scaffold_native_plugin(&target_dir, &opts.name, opts.quiet)?,
        BinFlavor::Mdx => scaffold_mdx(&target_dir, &opts.name, opts.quiet)?,
        BinFlavor::Wasm => scaffold_wasm(&target_dir, &opts.name, opts.quiet)?,
    }

    if !opts.quiet {
        eprintln!(
            "[bin] ✓ {} ({:?}) scaffolded → cd {} && bun install && bun run build",
            opts.name,
            opts.flavor,
            target_dir.display()
        );
    }
    Ok(())
}

fn scaffold_native_plugin(dir: &std::path::Path, name: &str, quiet: bool) -> Result<()> {
    write_file(
        dir.join("Cargo.toml"),
        &render_cargo_toml(name, false),
        quiet,
    )?;
    write_file(
        dir.join("src/lib.rs"),
        NATIVE_PLUGIN_LIB_RS,
        quiet,
    )?;
    write_file(
        dir.join("package.json"),
        &render_package_json(name, false),
        quiet,
    )?;
    write_file(dir.join("build.rs"), BUILD_RS, quiet)?;
    write_file(dir.join(".cargo/config.toml"), CARGO_CONFIG, quiet)?;
    write_file(dir.join("README.md"), &render_readme(name, "Plugin natif Bun (Rust → .node)"), quiet)?;
    write_file(dir.join("index.ts"), INDEX_TS_NATIVE, quiet)?;
    write_file(dir.join(".gitignore"), GITIGNORE, quiet)?;
    Ok(())
}

fn scaffold_mdx(dir: &std::path::Path, name: &str, quiet: bool) -> Result<()> {
    write_file(dir.join("Cargo.toml"), &render_cargo_toml(name, true), quiet)?;
    write_file(dir.join("src/lib.rs"), MDX_LIB_RS, quiet)?;
    write_file(dir.join("package.json"), &render_package_json(name, false), quiet)?;
    write_file(dir.join("build.rs"), BUILD_RS, quiet)?;
    write_file(dir.join(".cargo/config.toml"), CARGO_CONFIG, quiet)?;
    write_file(
        dir.join("README.md"),
        &render_readme(name, "Plugin MDX → JSX pour Bun.build (via mdxjs-rs)"),
        quiet,
    )?;
    write_file(dir.join("index.ts"), INDEX_TS_MDX, quiet)?;
    write_file(dir.join("example.mdx"), EXAMPLE_MDX, quiet)?;
    write_file(dir.join(".gitignore"), GITIGNORE, quiet)?;
    Ok(())
}

fn scaffold_wasm(dir: &std::path::Path, name: &str, quiet: bool) -> Result<()> {
    write_file(dir.join("Cargo.toml"), &render_wasm_cargo_toml(name), quiet)?;
    write_file(dir.join("src/lib.rs"), WASM_LIB_RS, quiet)?;
    write_file(dir.join("package.json"), &render_wasm_package_json(name), quiet)?;
    write_file(
        dir.join("README.md"),
        &render_readme(
            name,
            "Module WASM (wasm-pack / wasm-bindgen) utilisable depuis Bun",
        ),
        quiet,
    )?;
    write_file(dir.join("index.ts"), INDEX_TS_WASM, quiet)?;
    write_file(dir.join(".gitignore"), GITIGNORE_WASM, quiet)?;
    Ok(())
}

fn write_file(path: PathBuf, content: &str, quiet: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("mkdir {}", parent.display()))?;
    }
    std::fs::write(&path, content).with_context(|| format!("écrire {}", path.display()))?;
    if !quiet {
        eprintln!("[bin]   + {}", path.display());
    }
    Ok(())
}

fn render_cargo_toml(name: &str, with_mdx: bool) -> String {
    let extra = if with_mdx {
        "mdxjs-rs = \"0.2\"\n"
    } else {
        ""
    };
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
description = "Bun native bundler plugin (Rust)"

[lib]
crate-type = ["cdylib"]

[dependencies]
bun-native-plugin = "0.0.1"
napi = {{ version = "2", features = ["napi8"] }}
napi-derive = "2"
{extra}
[build-dependencies]
napi-build = "2"

[profile.release]
lto = true
codegen-units = 1
strip = "symbols"
"#,
    )
}

fn render_wasm_cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
description = "WASM module for Bun"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"

[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
"#,
    )
}

fn render_package_json(name: &str, _with_mdx: bool) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "description": "Bun native bundler plugin",
  "main": "index.ts",
  "napi": {{
    "name": "{name}"
  }},
  "scripts": {{
    "build": "napi build --release",
    "build:debug": "napi build",
    "test": "bun test"
  }},
  "devDependencies": {{
    "@napi-rs/cli": "^3.0.0",
    "@types/bun": "latest"
  }},
  "engines": {{
    "bun": ">=1.2.0"
  }}
}}
"#,
    )
}

fn render_wasm_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "description": "WASM module for Bun",
  "type": "module",
  "main": "index.ts",
  "scripts": {{
    "build": "wasm-pack build --target web --out-dir pkg",
    "build:node": "wasm-pack build --target nodejs --out-dir pkg-node",
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

fn render_readme(name: &str, desc: &str) -> String {
    format!(
        r#"# {name}

{desc}

Scaffolded by [n2b](https://github.com/aphrody-code/n2b).

## Build

```bash
bun install
bun run build
```

## Use

```ts
import {{ plugin }} from "./index.ts";

await Bun.build({{
  entrypoints: ["./src/main.ts"],
  plugins: [plugin],
}});
```
"#,
    )
}

const NATIVE_PLUGIN_LIB_RS: &str = r#"//! Bun native bundler plugin — generated by n2b bin.
//!
//! Docs :
//!   - https://bun.sh/docs/bundler/plugins
//!   - https://docs.rs/bun-native-plugin
//!   - https://github.com/oven-sh/bun/tree/main/packages/bun-native-plugin-rs

use bun_native_plugin::{sys, OnBeforeParse};
use napi_derive::napi;

/// Nom du plugin exposé côté JavaScript (napi-rs → .node).
#[napi]
pub fn register_bun_plugin() -> String {
    "n2b-native-plugin".to_string()
}

/// Hook onBeforeParse : appelé par Bun avant le parsing de chaque module.
///
/// SAFETY : appelé par Bun depuis plusieurs threads — tout état externe doit
/// être `Sync`. Ne jamais paniquer : le crash kille tout le bundler.
#[no_mangle]
pub extern "C" fn on_before_parse_plugin_impl(
    args: *const sys::OnBeforeParseArguments,
    result: *mut sys::OnBeforeParseResult,
) {
    let args = unsafe { &*args };
    let result = unsafe { &mut *result };

    let mut handle = match OnBeforeParse::from_raw(args, result) {
        Ok(h) => h,
        Err(_) => return,
    };

    let source = match handle.input_source_code() {
        Ok(s) => s,
        Err(_) => {
            handle.log_error("failed to read source");
            return;
        }
    };

    // Exemple minimal : remplace __FOO__ par "bar"
    let patched = source.replace("__FOO__", "bar");
    handle.set_output_source_code(patched, handle.output_loader());
}
"#;

const MDX_LIB_RS: &str = r#"//! MDX → JSX plugin for Bun.build — inspired by bun-build-mdx-rs.
//!
//! Docs :
//!   - https://github.com/oven-sh/bun/tree/main/packages/bun-build-mdx-rs
//!   - https://docs.rs/mdxjs

use bun_native_plugin::{sys, BunLoader, OnBeforeParse};
use mdxjs::{compile, Options};
use napi_derive::napi;

#[napi]
pub fn register_bun_plugin() -> String {
    "mdx-rs".to_string()
}

#[no_mangle]
pub extern "C" fn on_before_parse_plugin_impl(
    args: *const sys::OnBeforeParseArguments,
    result: *mut sys::OnBeforeParseResult,
) {
    let args = unsafe { &*args };
    let result = unsafe { &mut *result };

    let mut handle = match OnBeforeParse::from_raw(args, result) {
        Ok(h) => h,
        Err(_) => return,
    };

    let source = match handle.input_source_code() {
        Ok(s) => s,
        Err(_) => {
            handle.log_error("mdx: failed to read source");
            return;
        }
    };

    match compile(&source, &Options::default()) {
        Ok(code) => handle.set_output_source_code(code, BunLoader::Tsx),
        Err(e) => handle.log_error(&format!("mdx compile error: {e}")),
    }
}
"#;

const BUILD_RS: &str = r#"fn main() {
    napi_build::setup();
}
"#;

const CARGO_CONFIG: &str = r#"[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-feature=-crt-static"]

[target.aarch64-unknown-linux-gnu]
rustflags = ["-C", "target-feature=-crt-static"]
"#;

const INDEX_TS_NATIVE: &str = r#"// Register the native plugin in Bun.build.
// Replace "./index.node" with the path emitted by `napi build`.

import { type BunPlugin } from "bun";

// @ts-expect-error — generated by napi at build time
import native from "./index.node";

export const plugin: BunPlugin = {
  name: "native-plugin",
  async setup(build) {
    build.onLoad({ filter: /.*/ }, (args) => {
      return native.onBeforeParse(args);
    });
  },
};
"#;

const INDEX_TS_MDX: &str = r#"// MDX → JSX plugin for Bun.build.

import { type BunPlugin } from "bun";

// @ts-expect-error — generated by napi at build time
import native from "./index.node";

export const plugin: BunPlugin = {
  name: "mdx-rs",
  async setup(build) {
    build.onLoad({ filter: /\.mdx$/ }, async (args) => {
      const contents = await Bun.file(args.path).text();
      const { code } = native.mdxToJsx(contents);
      return { contents: code, loader: "tsx" };
    });
  },
};
"#;

const INDEX_TS_WASM: &str = r#"// Bun-side loader for the WASM module built by wasm-pack.
// The `pkg/` directory is created by `bun run build`.

import init, * as wasm from "./pkg/index.js";

await init();
export const { add, greet } = wasm as any;
"#;

const WASM_LIB_RS: &str = r#"//! WASM module for Bun — generated by n2b bin.
//!
//! Build: `bun run build` (runs wasm-pack).
//! Use from Bun : `import init, { add, greet } from "./pkg/<name>.js"`.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}! (from Rust→WASM)")
}
"#;

const EXAMPLE_MDX: &str = r#"# Hello from MDX

Compiled to JSX at build-time by the Rust plugin.

```ts
const answer = 42;
```

<Button>Click me</Button>
"#;

const GITIGNORE: &str = r#"target/
node_modules/
*.node
pkg/
pkg-node/
.DS_Store
"#;

const GITIGNORE_WASM: &str = r#"target/
node_modules/
pkg/
pkg-node/
.DS_Store
"#;

#[allow(dead_code)]
pub fn valid_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(anyhow!("nom vide"));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "nom '{s}' contient des caractères invalides (autorisés : a-z 0-9 - _)"
        ));
    }
    Ok(())
}
