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
}

impl BinFlavor {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "native" | "native-plugin" | "plugin" => Some(Self::NativePlugin),
            "mdx" => Some(Self::Mdx),
            "wasm" => Some(Self::Wasm),
            "webgpu" | "wgpu" | "gpu" => Some(Self::WebGpu),
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
