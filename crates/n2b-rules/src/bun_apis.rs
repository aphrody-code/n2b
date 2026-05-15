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

//! Règles `api/*` et `next/*` — détection de patterns d'appel Node →
//! réécriture Bun. Phase 1.5 : les données vivent dans
//! `crates/n2b-registry/registry/apis.toml` ; cette couche fournit le
//! matching, les hacks contextuels (`is_member_exec_call`,
//! `looks_like_dir_context`) et l'application des édits.

use n2b_registry::APIS;
use n2b_types::types::{Finding, MakeFindingOpts, Severity};
use n2b_util::{Edit, apply_edits, line_offsets, make_finding};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

enum ReplaceKind {
    None,
    Static(String),
    Template(String), // utilise $1..$n
}

/// Vue runtime d'une `ApiEntry` du registre — la regex est compilée une
/// fois au premier accès via `Lazy`, comme avant le refactor data-driven.
struct ApiRule {
    id: String,
    re: Regex,
    message: String,
    replace: ReplaceKind,
    aggressive: bool,
    severity: Severity,
}

/// Construit les `ApiRule` runtime depuis le registre `APIS` au premier
/// accès. L'ordre d'itération du registre = ordre des `[[apis]]` dans
/// `apis.toml` = ordre des `rule(...)` dans l'ancien `vec![...]` —
/// préserve l'ordre des findings dans la sortie (zero-drift Phase 1).
static RULES: Lazy<Vec<ApiRule>> = Lazy::new(|| {
    APIS.iter()
        .map(|e| {
            let replace = match e.replace.as_str() {
                "none" => ReplaceKind::None,
                "static" => ReplaceKind::Static(
                    e.replacement
                        .clone()
                        .expect("invariant: replace=static requires replacement"),
                ),
                "template" => ReplaceKind::Template(
                    e.replacement
                        .clone()
                        .expect("invariant: replace=template requires replacement"),
                ),
                other => panic!("invariant: replace='{other}' inconnu pour {}", e.id),
            };
            ApiRule {
                id: e.id.clone(),
                re: Regex::new(&e.pattern).unwrap_or_else(|err| {
                    panic!(
                        "invariant: ApiRule pattern for rule '{}' is invalid: {err}",
                        e.id
                    )
                }),
                message: e.message.clone(),
                replace,
                aggressive: e.aggressive,
                severity: e.severity,
            }
        })
        .collect()
});

pub fn apply_bun_api_rules(path: &str, source: &str, aggressive: bool) -> (Vec<Finding>, String) {
    let offsets = line_offsets(source);
    let mut findings: Vec<Finding> = Vec::new();
    let mut edits: Vec<Edit> = Vec::new();

    for r in RULES.iter() {
        for mat in r.re.captures_iter(source) {
            let whole = mat
                .get(0)
                .expect("invariant: capture group 0 is always present in a match");
            let index = whole.start();

            // Bug fix : api/exec matche aussi regex.exec() et string.exec() (accès membre).
            // Ne garder que les appels child_process → soit `child_process.exec(` explicite,
            // soit `exec(` en position d'appel directe (pas précédé d'un `.`).
            // Early continue pour éviter toute allocation inutile.
            if (r.id == "api/exec" || r.id == "api/execSync") && is_member_exec_call(source, index)
            {
                continue;
            }

            let original = whole.as_str();
            let original_len = original.len();
            let replacement = match &r.replace {
                ReplaceKind::None => None,
                ReplaceKind::Static(s) => Some(s.clone()),
                ReplaceKind::Template(t) => Some(expand(&mat, t)),
            };

            // Bug fix : fs.existsSync(path) suivi dans les ~15 lignes par fs.mkdirSync(path, ...)
            // indique un contexte DOSSIER — Bun.file().exists() retourne toujours false pour un dir.
            // On dégrade l'autofix en simple warning non-appliqué.
            let skip_autofix = r.id == "api/fs-existsSync"
                && mat.get(1).is_some_and(|m| {
                    let arg = m.as_str().trim();
                    looks_like_dir_context(source, index, arg)
                });

            let has_repl = replacement.is_some() && !skip_autofix;
            let replacement_for_edit = if aggressive && r.aggressive && !skip_autofix {
                replacement.clone()
            } else {
                None
            };
            findings.push(make_finding(
                path,
                &offsets,
                index,
                &r.id,
                if skip_autofix {
                    "fs.existsSync(path) suivi d'un fs.mkdirSync(path) — chemin probablement un dossier, Bun.file().exists() inadapté (utiliser fs.mkdirSync(path, { recursive: true }))".to_string()
                } else {
                    r.message.clone()
                },
                original.to_string(),
                if skip_autofix { None } else { replacement },
                MakeFindingOpts {
                    autofix: Some(has_repl),
                    aggressive: if r.aggressive { Some(true) } else { None },
                    severity: Some(r.severity),
                },
            ));
            if let Some(repl) = replacement_for_edit {
                edits.push(Edit {
                    index,
                    len: original_len,
                    replacement: repl,
                });
            }
        }
    }

    // Résout les overlaps : quand deux règles matchent des ranges imbriqués
    // (ex. api/fs-readFileSync à l'intérieur de api/json-parse-readFileSync),
    // apply_edits garde le range le plus large à index égal.
    let out = apply_edits(source, edits);
    (findings, out)
}

fn expand(caps: &Captures, template: &str) -> String {
    let mut out = String::with_capacity(template.len());
    caps.expand(template, &mut out);
    out
}

/// Fenêtre de recherche (en octets) pour décider si un `fs.existsSync` vise un
/// dossier. Heuristique : portée d'un bloc `fs.mkdir` typiquement proche du test
/// d'existence (≈ 15 lignes).
/// TODO(phase-2) : supprimable une fois le matching AST en place — l'AST sait si
/// le même symbole est passé à fs.mkdir sans fenêtre arbitraire.
const DIR_CONTEXT_WINDOW_BYTES: usize = 600;

/// Retourne true si un fs.mkdirSync(<arg>, ...) apparaît dans la fenêtre
/// `DIR_CONTEXT_WINDOW_BYTES` qui suit la position `pos` — indicateur qu'il
/// s'agit d'un dossier et non d'un fichier.
fn looks_like_dir_context(source: &str, pos: usize, arg: &str) -> bool {
    let end = (pos + DIR_CONTEXT_WINDOW_BYTES).min(source.len());
    let window = &source[pos..end];
    let needle_sync = format!("fs.mkdirSync({arg}");
    let needle_async = format!("fs.mkdir({arg}");
    window.contains(&needle_sync) || window.contains(&needle_async)
}

/// Détecte les appels `.exec(` / `.execSync(` sur un objet (RegExp, etc.) plutôt
/// que child_process. Si le `exec` est précédé d'un `.` ET que ce qui précède
/// n'est pas `child_process`, alors c'est un appel membre → faux positif.
fn is_member_exec_call(source: &str, pos: usize) -> bool {
    if pos == 0 {
        return false;
    }
    let bytes = source.as_bytes();
    if bytes[pos - 1] != b'.' {
        return false;
    }
    // Si on est juste après "child_process." → c'est le vrai appel Node
    !source[..pos].ends_with("child_process.")
}
