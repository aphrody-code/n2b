// Copyright 2026 Yohan Pierre
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::imports_ast;
use n2b_registry::{MODULES, PACKAGES};
use n2b_types::types::{Finding, MakeFindingOpts};
use n2b_util::{Edit, apply_edits, line_offsets, make_finding};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

/// Builtins Node `node:*` — alimenté par `crates/n2b-registry/registry/modules.toml`.
/// La liste inclut les sous-chemins shimmés par Bun (`fs/promises`, etc.).
static BUILTINS: Lazy<HashSet<String>> =
    Lazy::new(|| MODULES.iter().map(|m| m.module.clone()).collect());

struct BunReplacement {
    replacement: String,
    note: String,
    aggressive: bool,
}

static BUN_REPLACEMENTS: Lazy<HashMap<String, BunReplacement>> = Lazy::new(|| {
    PACKAGES
        .iter()
        .map(|p| {
            (
                p.package.clone(),
                BunReplacement {
                    replacement: p.replacement.clone(),
                    note: p.note.clone(),
                    aggressive: p.aggressive,
                },
            )
        })
        .collect()
});

pub fn apply_node_import_rules(
    path: &str,
    source: &str,
    aggressive: bool,
) -> (Vec<Finding>, String) {
    let offsets = line_offsets(source);
    let mut findings: Vec<Finding> = Vec::new();
    let mut edits: Vec<Edit> = Vec::new();

    let mut seen_builtin_finding: HashSet<String> = HashSet::new();
    let mut seen_repl_finding: HashSet<String> = HashSet::new();

    let specifiers = imports_ast::extract_specifiers(path, source);

    for s in &specifiers {
        let pos = s.inner_start as usize;
        let len = s.inner_len as usize;
        let spec = &s.value;

        if BUILTINS.contains(spec.as_str()) {
            edits.push(Edit {
                index: pos,
                len,
                replacement: format!("node:{spec}"),
            });
            if seen_builtin_finding.insert(spec.clone()) {
                findings.push(make_finding(
                    path,
                    &offsets,
                    pos,
                    "imports/node-prefix",
                    format!("préfixer '{spec}' avec 'node:' (recommandé)"),
                    spec.clone(),
                    Some(format!("node:{spec}")),
                    MakeFindingOpts {
                        autofix: Some(true),
                        ..Default::default()
                    },
                ));
            }
            continue;
        }

        if let Some(r) = BUN_REPLACEMENTS.get(spec.as_str()) {
            if seen_repl_finding.insert(spec.clone()) {
                findings.push(make_finding(
                    path,
                    &offsets,
                    pos,
                    "imports/bun-native",
                    format!("remplacer '{}' par {} — {}", spec, r.replacement, r.note),
                    spec.clone(),
                    Some(r.replacement.to_string()),
                    MakeFindingOpts {
                        autofix: Some(false),
                        aggressive: Some(true),
                        ..Default::default()
                    },
                ));
            }
            if aggressive
                && r.aggressive
                && (r.replacement.starts_with("bun:") || r.replacement.starts_with("node:"))
            {
                edits.push(Edit {
                    index: pos,
                    len,
                    replacement: r.replacement.to_string(),
                });
            }
        }
    }

    let out = apply_edits(source, edits);
    (findings, out)
}
