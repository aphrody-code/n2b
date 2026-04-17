use crate::rules::cli_commands::apply_cli_rules;
use crate::types::{Finding, MakeFindingOpts, Severity};
use crate::util::make_finding;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

const REDUNDANT_DEPS: &[&str] = &[
    "node-fetch", "isomorphic-fetch", "cross-fetch",
    "dotenv", "dotenv-cli",
    "rimraf", "mkdirp",
    "better-sqlite3", "sqlite3",
    "uuid", "nanoid",
    "ts-node", "tsx", "ts-node-esm",
    "concurrently",
];

static JEST_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bjest(?:\s|$)").unwrap());
static TSUP_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\btsup(?:\s|$)").unwrap());

pub fn scan_package_json(path: &str, content: &str) -> (Vec<Finding>, String) {
    let mut findings: Vec<Finding> = Vec::new();

    let mut parsed: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(e) => {
            findings.push(make_finding(
                path,
                &[],
                0,
                "pkg/parse",
                format!("package.json invalide : {e}"),
                content.chars().take(40).collect::<String>(),
                None,
                MakeFindingOpts {
                    severity: Some(Severity::Error),
                    autofix: Some(false),
                    ..Default::default()
                },
            ));
            return (findings, content.to_string());
        }
    };

    let mut mutated = false;

    // 1. Rewrite scripts.
    if let Some(scripts) = parsed.get_mut("scripts").and_then(|v| v.as_object_mut()) {
        let keys: Vec<String> = scripts.keys().cloned().collect();
        for name in keys {
            let raw = match scripts.get(&name).and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let (script_findings, rewritten) =
                apply_cli_rules(&format!("{path} [scripts.{name}]"), &raw);
            findings.extend(script_findings);
            if rewritten != raw {
                scripts.insert(name.clone(), Value::String(rewritten.clone()));
                mutated = true;
            }

            // pkg/jest-script — détecte "jest ..." dans les scripts
            if JEST_RE.is_match(&rewritten) {
                findings.push(make_finding(
                    path,
                    &[],
                    0,
                    "pkg/jest-script",
                    format!(
                        "script {name:?}='{rewritten}' utilise jest — préférer 'bun test' (compatible describe/test/expect ; utiliser --preload reflect-metadata pour les décorateurs)"
                    ),
                    rewritten.clone(),
                    Some("bun test".into()),
                    MakeFindingOpts {
                        autofix: Some(false),
                        aggressive: Some(true),
                        severity: Some(Severity::Warn),
                    },
                ));
            }

            // pkg/tsup-bun-external — détecte tsup sans --external bun
            // (heuristique : si la racine contient un fichier source avec await import("bun"))
            if TSUP_RE.is_match(&rewritten) && !rewritten.contains("--external bun") {
                findings.push(make_finding(
                    path,
                    &[],
                    0,
                    "pkg/tsup-bun-external",
                    format!(
                        "script {name:?} utilise tsup — ajouter '--external bun' si le code fait 'await import(\"bun\")' (sinon esbuild échoue au bundle-time)"
                    ),
                    rewritten.clone(),
                    Some(format!("{} --external bun", rewritten.trim_end())),
                    MakeFindingOpts {
                        autofix: Some(false),
                        aggressive: Some(true),
                        severity: Some(Severity::Info),
                    },
                ));
            }
        }
    }

    // 2. packageManager
    if let Some(pm) = parsed.get("packageManager").and_then(|v| v.as_str()) {
        if !pm.starts_with("bun@") {
            findings.push(make_finding(
                path,
                &[],
                0,
                "pkg/package-manager",
                format!("packageManager='{pm}' — remplacer par 'bun@<version>' ou supprimer"),
                pm.to_string(),
                None,
                MakeFindingOpts { autofix: Some(false), ..Default::default() },
            ));
        }
    }

    // 3. engines.{npm,pnpm,yarn}
    if let Some(engines) = parsed.get("engines").and_then(|v| v.as_object()) {
        if engines.contains_key("npm") || engines.contains_key("pnpm") || engines.contains_key("yarn") {
            findings.push(make_finding(
                path,
                &[],
                0,
                "pkg/engines-pm",
                "engines.{npm,pnpm,yarn} est superflu avec Bun — utiliser 'engines.bun'",
                serde_json::to_string(engines).unwrap_or_default(),
                None,
                MakeFindingOpts { autofix: Some(false), ..Default::default() },
            ));
        }
    }

    // 4. Redundant deps
    let dep_keys = ["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"];
    for key in dep_keys {
        if let Some(deps) = parsed.get(key).and_then(|v| v.as_object()) {
            for dep in deps.keys() {
                if REDUNDANT_DEPS.contains(&dep.as_str()) {
                    findings.push(make_finding(
                        path,
                        &[],
                        0,
                        "pkg/redundant-dep",
                        format!("dépendance '{dep}' redondante avec Bun (voir Bun.file / Bun.env / fetch global / bun:sqlite / bun test)"),
                        dep.clone(),
                        None,
                        MakeFindingOpts {
                            autofix: Some(false),
                            aggressive: Some(true),
                            ..Default::default()
                        },
                    ));
                }
            }
        }
    }

    // 5. Root-only checks : pnpm-workspace.yaml à côté → workspace/root-missing
    //    + @types/bun manquant quand code utilise Bun.*
    let is_root_pkg = !path.contains('/');
    if is_root_pkg {
        let dir = std::path::Path::new(path).parent().unwrap_or_else(|| std::path::Path::new("."));
        let pnpm_ws = dir.join("pnpm-workspace.yaml");
        if pnpm_ws.exists() && parsed.get("workspaces").is_none() {
            findings.push(make_finding(
                path,
                &[],
                0,
                "workspace/root-missing",
                "pnpm-workspace.yaml présent mais package.json racine n'a pas de champ \"workspaces\" — requis par Bun".to_string(),
                "workspaces".to_string(),
                Some(r#""workspaces": ["packages/*"]"#.to_string()),
                MakeFindingOpts {
                    autofix: Some(false),
                    severity: Some(Severity::Warn),
                    ..Default::default()
                },
            ));
        }

        // onlyBuiltDependencies dans pnpm-workspace.yaml → trustedDependencies
        if pnpm_ws.exists() && parsed.get("trustedDependencies").is_none() {
            if let Ok(content) = std::fs::read_to_string(&pnpm_ws) {
                if let Some(info) = crate::scanners::pnpm_workspace::parse_pnpm_workspace(&content) {
                    if !info.only_built.is_empty() {
                        findings.push(make_finding(
                            path,
                            &[],
                            0,
                            "workspace/trusted-deps-missing",
                            format!(
                                "onlyBuiltDependencies de pnpm-workspace.yaml ({} pkg) non portées vers \"trustedDependencies\"",
                                info.only_built.len()
                            ),
                            "trustedDependencies".to_string(),
                            Some(format!(
                                r#""trustedDependencies": {}"#,
                                serde_json::to_string(&info.only_built).unwrap_or_default()
                            )),
                            MakeFindingOpts {
                                autofix: Some(false),
                                severity: Some(Severity::Warn),
                                ..Default::default()
                            },
                        ));
                    }
                }
            }
        }
    }

    // 6. "type":"module" + main .cjs
    if parsed.get("type").and_then(|v| v.as_str()) == Some("module") {
        if let Some(main) = parsed.get("main").and_then(|v| v.as_str()) {
            if main.ends_with(".cjs") {
                findings.push(make_finding(
                    path,
                    &[],
                    0,
                    "pkg/main-mismatch",
                    r#""type":"module" mais main pointe sur un fichier .cjs"#.to_string(),
                    main.to_string(),
                    None,
                    MakeFindingOpts { autofix: Some(false), ..Default::default() },
                ));
            }
        }
    }

    let new_content = if mutated {
        let mut s = serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| content.to_string());
        if content.ends_with('\n') && !s.ends_with('\n') {
            s.push('\n');
        }
        s
    } else {
        content.to_string()
    };
    (findings, new_content)
}
