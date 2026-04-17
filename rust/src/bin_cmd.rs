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
    /// Module Rust+wgpu compilé en WASM, compute shader WebGPU côté Bun.
    WebGpu,
    /// Plugin Bun natif qui compile/minifie CSS via lightningcss.
    /// Utile pour Tailwind v4, CSS nesting, transpilation color-mix, etc.
    LightningCss,
    /// Plugin Bun JS wrappant @tailwindcss/postcss (Tailwind v4 Oxide via N-API).
    /// Zéro compile Rust — utilise le binding npm officiel.
    Tailwind,
}

impl BinFlavor {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "native" | "native-plugin" | "plugin" => Some(Self::NativePlugin),
            "mdx" => Some(Self::Mdx),
            "wasm" => Some(Self::Wasm),
            "webgpu" | "wgpu" | "gpu" => Some(Self::WebGpu),
            "lightningcss" | "css" | "css-lightning" => Some(Self::LightningCss),
            "tailwind" | "tailwindcss" | "tw" => Some(Self::Tailwind),
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
        BinFlavor::WebGpu => scaffold_webgpu(&target_dir, &opts.name, opts.quiet)?,
        BinFlavor::LightningCss => scaffold_lightningcss(&target_dir, &opts.name, opts.quiet)?,
        BinFlavor::Tailwind => scaffold_tailwind(&target_dir, &opts.name, opts.quiet)?,
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

fn scaffold_tailwind(dir: &std::path::Path, name: &str, quiet: bool) -> Result<()> {
    write_file(
        dir.join("package.json"),
        &render_tailwind_package_json(name),
        quiet,
    )?;
    write_file(dir.join("plugin.ts"), TAILWIND_PLUGIN_TS, quiet)?;
    write_file(dir.join("build.ts"), TAILWIND_BUILD_TS, quiet)?;
    write_file(dir.join("postcss.config.mjs"), TAILWIND_POSTCSS_CONFIG, quiet)?;
    write_file(dir.join("src/app.css"), TAILWIND_APP_CSS, quiet)?;
    write_file(dir.join("src/index.html"), TAILWIND_INDEX_HTML, quiet)?;
    write_file(
        dir.join("README.md"),
        &render_readme(
            name,
            "Plugin Bun JS wrapping @tailwindcss/postcss (Tailwind v4 Oxide via N-API)",
        ),
        quiet,
    )?;
    write_file(dir.join(".gitignore"), GITIGNORE_WASM, quiet)?;
    Ok(())
}

fn scaffold_lightningcss(dir: &std::path::Path, name: &str, quiet: bool) -> Result<()> {
    write_file(
        dir.join("Cargo.toml"),
        &render_lightningcss_cargo_toml(name),
        quiet,
    )?;
    write_file(dir.join("src/lib.rs"), LIGHTNINGCSS_LIB_RS, quiet)?;
    write_file(
        dir.join("package.json"),
        &render_package_json(name, false),
        quiet,
    )?;
    write_file(dir.join("build.rs"), BUILD_RS, quiet)?;
    write_file(dir.join(".cargo/config.toml"), CARGO_CONFIG, quiet)?;
    write_file(
        dir.join("README.md"),
        &render_readme(
            name,
            "Plugin Bun natif : minify/transpile CSS via lightningcss (Rust)",
        ),
        quiet,
    )?;
    write_file(dir.join("index.ts"), INDEX_TS_LIGHTNINGCSS, quiet)?;
    write_file(dir.join("example.css"), EXAMPLE_CSS, quiet)?;
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

fn scaffold_webgpu(dir: &std::path::Path, name: &str, quiet: bool) -> Result<()> {
    write_file(
        dir.join("Cargo.toml"),
        &render_webgpu_cargo_toml(name),
        quiet,
    )?;
    write_file(dir.join("src/lib.rs"), WEBGPU_LIB_RS, quiet)?;
    write_file(
        dir.join("src/compute.wgsl"),
        COMPUTE_WGSL,
        quiet,
    )?;
    write_file(
        dir.join("package.json"),
        &render_webgpu_package_json(name),
        quiet,
    )?;
    write_file(dir.join("index.ts"), INDEX_TS_WEBGPU, quiet)?;
    write_file(
        dir.join("README.md"),
        &render_readme(
            name,
            "Rust + wgpu → WASM → Bun/WebGPU compute pipeline (WGSL inclus)",
        ),
        quiet,
    )?;
    write_file(dir.join(".gitignore"), GITIGNORE_WASM, quiet)?;
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

fn render_webgpu_cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
description = "wgpu (WebGPU) compute module for Bun via WASM"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wgpu = {{ version = "29", features = ["webgpu"] }}
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
bytemuck = {{ version = "1", features = ["derive"] }}
futures-channel = "0.3"
log = "0.4"

[target.'cfg(target_arch = "wasm32")'.dependencies]
web-sys = {{ version = "0.3", features = ["Window", "Document", "Navigator", "Gpu"] }}
console_error_panic_hook = "0.1"

[profile.release]
lto = true
codegen-units = 1
opt-level = "s"
"#,
    )
}

fn render_webgpu_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "description": "wgpu compute WASM for Bun",
  "type": "module",
  "main": "index.ts",
  "scripts": {{
    "build": "wasm-pack build --target web --out-dir pkg -- --features webgpu",
    "build:bundler": "wasm-pack build --target bundler --out-dir pkg-bundler",
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

const WEBGPU_LIB_RS: &str = r#"//! Rust + wgpu → WASM compute pipeline for Bun.
//!
//! Shader : src/compute.wgsl (WGSL — voir https://gpuweb.github.io/gpuweb/wgsl/)
//! Runtime: navigator.gpu côté Bun (WebGPU est exposé dans Bun et Electrobun).
//!
//! Docs :
//!   - https://docs.rs/wgpu
//!   - https://gpuweb.github.io/gpuweb/
//!   - https://blackboard.sh/electrobun/docs/apis/webgpu/

use wasm_bindgen::prelude::*;

const SHADER: &str = include_str!("compute.wgsl");

#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let _ = wasm_bindgen::throw_str;
    }
    let _ = log::set_max_level;
}

/// Double chaque élément du buffer d'entrée via un compute shader WGSL.
/// Retourne un `Vec<u32>` de même taille.
#[wasm_bindgen]
pub async fn double_u32(input: Vec<u32>) -> Result<Vec<u32>, JsValue> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok_or_else(|| JsValue::from_str("no wgpu adapter"))?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .map_err(|e| JsValue::from_str(&format!("request_device: {e:?}")))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("double.wgsl"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });

    let size = (input.len() * std::mem::size_of::<u32>()) as u64;

    let storage = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("storage"),
        size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    queue.write_buffer(&storage, 0, bytemuck::cast_slice(&input));

    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage.as_entire_binding(),
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(input.len() as u32, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&storage, 0, &staging, 0, size);
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = futures_channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.await
        .map_err(|e| JsValue::from_str(&format!("map canceled: {e}")))?
        .map_err(|e| JsValue::from_str(&format!("map: {e:?}")))?;

    let data = slice.get_mapped_range();
    let out: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging.unmap();
    Ok(out)
}
"#;

const COMPUTE_WGSL: &str = r#"// WGSL compute shader — double chaque u32 du storage buffer.
// Spec : https://gpuweb.github.io/gpuweb/wgsl/

@group(0) @binding(0) var<storage, read_write> data: array<u32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let idx = gid.x;
  if (idx < arrayLength(&data)) {
    data[idx] = data[idx] * 2u;
  }
}
"#;

const INDEX_TS_WEBGPU: &str = r#"// Bun loader for the wgpu+WASM compute module.
// Requires Bun >= 1.2 (WebGPU exposed via navigator.gpu) or Electrobun.

import init, * as wasm from "./pkg/index.js";

if (typeof navigator === "undefined" || !("gpu" in navigator)) {
  throw new Error(
    "WebGPU not available — run with `bun --enable-webgpu` or use Electrobun.",
  );
}

await init();

/** Double each element of `buf` via a WGSL compute shader. */
export async function double(buf: Uint32Array): Promise<Uint32Array> {
  // @ts-expect-error — wasm-bindgen generates async signatures
  const out = await wasm.double_u32(buf);
  return new Uint32Array(out);
}
"#;

fn render_lightningcss_cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
description = "Bun native CSS plugin — lightningcss (minify, transpile, nesting)"

[lib]
crate-type = ["cdylib"]

[dependencies]
bun-native-plugin = "0.0.1"
napi = {{ version = "2", features = ["napi8"] }}
napi-derive = "2"
lightningcss = "1.0.0-alpha.71"
parcel_sourcemap = "2"

[build-dependencies]
napi-build = "2"

[profile.release]
lto = true
codegen-units = 1
strip = "symbols"
"#,
    )
}

const LIGHTNINGCSS_LIB_RS: &str = r#"//! Plugin Bun natif : minify + transpile CSS via lightningcss.
//!
//! Supporte :
//!   - CSS Nesting (spec moderne)
//!   - color-mix(), @layer, light-dark(), @container
//!   - Minification plus rapide qu'esbuild (pure Rust, SIMD-friendly)
//!   - Tailwind v4 (Tailwind Oxide utilise lightningcss en sortie)
//!
//! Docs :
//!   - https://lightningcss.dev/
//!   - https://docs.rs/lightningcss
//!   - https://bun.sh/docs/bundler/css

use bun_native_plugin::{sys, BunLoader, OnBeforeParse};
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::targets::Targets;
use napi_derive::napi;

#[napi]
pub fn register_bun_plugin() -> String {
    "lightningcss".to_string()
}

/// Hook onBeforeParse sur les .css : parse → minify → transpile → retourne le CSS.
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
            handle.log_error("lightningcss: failed to read source");
            return;
        }
    };

    let mut stylesheet = match StyleSheet::parse(&source, ParserOptions::default()) {
        Ok(ss) => ss,
        Err(e) => {
            handle.log_error(&format!("lightningcss parse: {e}"));
            return;
        }
    };

    if let Err(e) = stylesheet.minify(MinifyOptions {
        targets: Targets::default(),
        ..Default::default()
    }) {
        handle.log_error(&format!("lightningcss minify: {e}"));
        return;
    }

    match stylesheet.to_css(PrinterOptions {
        minify: true,
        ..Default::default()
    }) {
        Ok(out) => handle.set_output_source_code(out.code, BunLoader::Css),
        Err(e) => handle.log_error(&format!("lightningcss print: {e}")),
    }
}
"#;

const INDEX_TS_LIGHTNINGCSS: &str = r#"// Register the lightningcss plugin with Bun.build.

import { type BunPlugin } from "bun";

// @ts-expect-error — generated by napi at build time
import native from "./index.node";

export const plugin: BunPlugin = {
  name: "lightningcss",
  async setup(build) {
    build.onLoad({ filter: /\.css$/ }, async (args) => {
      const input = await Bun.file(args.path).text();
      const { code } = native.onBeforeParse(input);
      return { contents: code, loader: "css" };
    });
  },
};

// Usage :
//   await Bun.build({
//     entrypoints: ["./src/app.css"],
//     plugins: [plugin],
//     outdir: "./dist",
//   });
"#;

const EXAMPLE_CSS: &str = r#"/* Modern CSS that lightningcss will transpile/minify. */

@layer base {
  :root {
    --brand: light-dark(oklch(0.5 0.2 250), oklch(0.7 0.2 250));
  }

  body {
    color: var(--brand);

    /* Native nesting */
    & a {
      color: color-mix(in oklab, var(--brand), white 20%);
    }

    & .card {
      container-type: inline-size;

      @container (min-width: 400px) {
        padding: 2rem;
      }
    }
  }
}
"#;

fn render_tailwind_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "description": "Bun.build + Tailwind v4 integration",
  "type": "module",
  "scripts": {{
    "build": "bun run build.ts",
    "dev": "bun --watch build.ts"
  }},
  "dependencies": {{
    "tailwindcss": "^4.0.0",
    "@tailwindcss/postcss": "^4.0.0",
    "postcss": "^8.4.0"
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

const TAILWIND_PLUGIN_TS: &str = r#"// Bun.build plugin : applique @tailwindcss/postcss sur chaque .css import.
// Utilise le binding N-API officiel Tailwind v4 Oxide (Rust under the hood).
//
// Refs :
//   - https://tailwindcss.com/docs/installation/using-postcss
//   - https://bun.sh/docs/bundler/plugins

import { type BunPlugin } from "bun";
import postcss from "postcss";
import tailwindcss from "@tailwindcss/postcss";

const processor = postcss([tailwindcss()]);

export const plugin: BunPlugin = {
  name: "tailwindcss",
  async setup(build) {
    build.onLoad({ filter: /\.css$/ }, async (args) => {
      const source = await Bun.file(args.path).text();
      const result = await processor.process(source, {
        from: args.path,
        to: args.path,
      });
      return { contents: result.css, loader: "css" };
    });
  },
};
"#;

const TAILWIND_BUILD_TS: &str = r#"// Minimal build script using Bun.build + Tailwind v4 plugin.
// Run : `bun run build`.

import { plugin } from "./plugin.ts";

await Bun.build({
  entrypoints: ["./src/app.css", "./src/index.html"],
  outdir: "./dist",
  plugins: [plugin],
  minify: true,
});

console.log("✓ built to ./dist");
"#;

const TAILWIND_POSTCSS_CONFIG: &str = r#"// PostCSS config for Tailwind v4.
// Loaded automatically if you run `postcss` CLI; the plugin.ts above imports
// @tailwindcss/postcss directly so this file is optional.

export default {
  plugins: {
    "@tailwindcss/postcss": {},
  },
};
"#;

const TAILWIND_APP_CSS: &str = r#"/* Tailwind v4 : single import, Oxide-powered. */
@import "tailwindcss";

/* Custom layer on top */
@layer components {
  .btn {
    @apply px-4 py-2 rounded-md bg-blue-500 text-white hover:bg-blue-600;
  }
}
"#;

const TAILWIND_INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Tailwind v4 + Bun</title>
    <link rel="stylesheet" href="./app.css" />
  </head>
  <body class="bg-gray-50 p-8">
    <h1 class="text-3xl font-bold">Hello Tailwind v4</h1>
    <button class="btn mt-4">Click</button>
  </body>
</html>
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
