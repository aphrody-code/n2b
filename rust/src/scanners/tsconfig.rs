use crate::types::{Finding, MakeFindingOpts, Severity};
use crate::util::make_finding;
use serde_json::Value;

/// Vérifie les options tsconfig qui gagnent à être ajustées pour Bun :
///   - `types` doit inclure `"bun"` pour que `Bun.*` soit typé
///   - `moduleResolution: "bundler"` recommandé (match Bun / Next 16)
pub fn scan_tsconfig(path: &str, content: &str) -> (Vec<Finding>, String) {
    let mut findings: Vec<Finding> = Vec::new();

    // tsconfig accepte les commentaires JSON5 — parse best-effort.
    let parsed: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return (findings, content.to_string()),
    };
    let co = match parsed.get("compilerOptions").and_then(|v| v.as_object()) {
        Some(c) => c,
        None => return (findings, content.to_string()),
    };

    // types : ajouter "bun" si @types/bun est attendu
    if let Some(types) = co.get("types").and_then(|v| v.as_array()) {
        let has_bun = types.iter().any(|t| t.as_str() == Some("bun"));
        let has_node = types.iter().any(|t| t.as_str() == Some("node"));
        if has_node && !has_bun {
            findings.push(make_finding(
                path,
                &[],
                0,
                "tsconfig/bun-types",
                "compilerOptions.types inclut 'node' mais pas 'bun' — ajouter 'bun' pour typer Bun.*",
                format!("{:?}", types),
                None,
                MakeFindingOpts {
                    autofix: Some(false),
                    severity: Some(Severity::Info),
                    ..Default::default()
                },
            ));
        }
    }

    // moduleResolution : flag si classic/node (pas bundler)
    if let Some(mr) = co.get("moduleResolution").and_then(|v| v.as_str()) {
        let lower = mr.to_ascii_lowercase();
        if lower == "node" || lower == "classic" {
            findings.push(make_finding(
                path,
                &[],
                0,
                "tsconfig/module-resolution",
                format!(
                    "moduleResolution='{mr}' — 'bundler' ou 'nodenext' offre une meilleure compat Bun/Next (ESM first)"
                ),
                mr.to_string(),
                None,
                MakeFindingOpts {
                    autofix: Some(false),
                    severity: Some(Severity::Info),
                    ..Default::default()
                },
            ));
        }
    }

    (findings, content.to_string())
}
