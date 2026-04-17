//! Scanner Cargo.toml : détecte les crates Rust de l'écosystème WASM/Bun
//! et émet des findings `ecosystem/*` avec liens vers leurs guides/docs.
//!
//! Pertinent quand Bun sert de runtime JS hôte pour un module Rust→WASM.

use crate::types::{Finding, MakeFindingOpts, Severity};
use crate::util::make_finding;

/// (crate_name, rule_suffix, docs_url, label).
const RUST_ECOSYSTEM: &[(&str, &str, &str, &str)] = &[
    // Web frameworks Rust → WASM
    ("yew",            "yew",            "https://yew.rs/",                       "Yew (React-like, Rust → WASM)"),
    ("leptos",         "leptos",         "https://leptos.dev/",                   "Leptos (fine-grained reactivity)"),
    ("dioxus",         "dioxus",         "https://dioxuslabs.com/",               "Dioxus (cross-platform GUI)"),
    ("sycamore",       "sycamore",       "https://sycamore.dev/",                 "Sycamore (Solid-like)"),
    ("seed",           "seed",           "https://seed-rs.org/",                  "Seed (Elm-like SPA)"),
    // GPU / graphics
    ("wgpu",           "wgpu",           "https://wgpu.rs/",                      "wgpu (WebGPU, compute + render)"),
    ("naga",           "naga",           "https://github.com/gfx-rs/wgpu/tree/trunk/naga", "naga (shader translator)"),
    // WASM tooling
    ("wasm-bindgen",   "wasm-bindgen",   "https://rustwasm.github.io/wasm-bindgen/", "wasm-bindgen"),
    ("js-sys",         "js-sys",         "https://rustwasm.github.io/wasm-bindgen/api/js_sys/", "js-sys"),
    ("web-sys",        "web-sys",        "https://rustwasm.github.io/wasm-bindgen/api/web_sys/", "web-sys"),
    ("wasm-bindgen-futures", "wasm-bindgen-futures", "https://rustwasm.github.io/wasm-bindgen/", "wasm-bindgen-futures"),
    ("console_error_panic_hook", "panic-hook", "https://github.com/rustwasm/console_error_panic_hook", "panic hook → console"),
    ("wee_alloc",      "wee-alloc",      "https://github.com/rustwasm/wee_alloc", "wee_alloc (small WASM allocator)"),
    // Bun / napi bindings
    ("napi",           "napi-rs",        "https://napi.rs/",                      "napi-rs (Node/Bun addon)"),
    ("napi-derive",    "napi-rs",        "https://napi.rs/",                      "napi-rs derive macros"),
    ("bun-native-plugin", "bun-native-plugin", "https://docs.rs/bun-native-plugin", "bun-native-plugin-rs"),
    // Serialization
    ("serde-wasm-bindgen", "serde-wasm", "https://github.com/RReverser/serde-wasm-bindgen", "serde → WASM"),
    ("gloo",           "gloo",           "https://gloo-rs.web.app/",              "gloo (toolkit Rust+WASM)"),
    // App frameworks
    ("tauri",          "tauri-rs",       "https://tauri.app/",                    "Tauri (Rust + WebView)"),
    ("bevy",           "bevy",           "https://bevyengine.org/",               "Bevy (game engine, WASM-capable)"),
    // MDX / build helpers
    ("mdxjs",          "mdxjs-rs",       "https://github.com/wooorm/mdxjs-rs",    "mdxjs-rs"),
];

pub fn scan_cargo_toml(path: &str, content: &str) -> (Vec<Finding>, String) {
    let mut findings: Vec<Finding> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Extrait le nom avant `=` ou `.`. Supporte `name = "..."`, `name.workspace = true`,
        // `name = { version = "..." }`, `[dependencies.name]`.
        let name = if let Some(rest) = trimmed.strip_prefix('[') {
            // [dependencies.yew] → yew
            let rest = rest.trim_end_matches(']');
            rest.split('.').nth(1).unwrap_or("").trim().to_string()
        } else if let Some(eq) = trimmed.find('=') {
            let k = trimmed[..eq].trim();
            // Strip optional `.workspace`, `.version`, `.features`, etc.
            k.split('.').next().unwrap_or("").trim().to_string()
        } else {
            continue;
        };

        if name.is_empty() {
            continue;
        }

        for (crate_name, suffix, url, label) in RUST_ECOSYSTEM {
            if *crate_name == name.as_str() && seen.insert(*suffix) {
                let rule_id = format!("ecosystem/{suffix}");
                findings.push(make_finding(
                    path,
                    &[],
                    0,
                    &rule_id,
                    format!(
                        "crate Rust `{crate_name}` détecté ({label}) — doc : {url}"
                    ),
                    (*crate_name).to_string(),
                    Some((*url).to_string()),
                    MakeFindingOpts {
                        autofix: Some(false),
                        severity: Some(Severity::Info),
                        ..Default::default()
                    },
                ));
            }
        }
    }

    (findings, content.to_string())
}

pub fn is_cargo_toml(name: &str) -> bool {
    name == "Cargo.toml"
}
