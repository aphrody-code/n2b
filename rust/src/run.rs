use crate::scanners::{
    bunfig::scan_bunfig,
    cargo_toml::{is_cargo_toml, scan_cargo_toml},
    dockerfile::scan_dockerfile,
    husky::{is_husky_hook, scan_husky},
    lockfile::{check_lockfile, RIVAL_LOCKFILES},
    next_config::{is_next_config, scan_next_config},
    npmrc::{is_rc_file, scan_npmrc},
    nvmrc::scan_nvmrc,
    package_json::scan_package_json,
    pnpm_workspace::scan_pnpm_workspace,
    shell::scan_shell,
    source::scan_source,
    tsconfig::scan_tsconfig,
    workflows::scan_workflow,
};
use crate::types::{FileFix, Mode, RunOptions};
use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

static SOURCE_EXTS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec!["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"]
});
static SHELL_EXTS: Lazy<Vec<&'static str>> = Lazy::new(|| vec!["sh", "bash", "zsh"]);
const SHELL_NAMES: &[&str] = &["Dockerfile", "Makefile", "Justfile"];
const DEFAULT_IGNORE: &[&str] = &[
    "**/node_modules/**",
    "**/.git/**",
    "**/dist/**",
    "**/build/**",
    "**/out/**",
    "**/.next/**",
    "**/.turbo/**",
    "**/coverage/**",
    "**/.bun/**",
    "**/target/**",
];

fn is_workflow(rel: &str) -> bool {
    // .github/workflows/*.yml | *.yaml
    let normalized = rel.replace('\\', "/");
    static RE: Lazy<regex::Regex> = Lazy::new(|| {
        regex::Regex::new(r"\.github/workflows/.+\.ya?ml$").unwrap()
    });
    RE.is_match(&normalized)
}

pub fn run(opts: &RunOptions) -> Result<Vec<FileFix>> {
    // Build matcher for default + user ignore globs.
    let mut builder = GlobSetBuilder::new();
    for p in DEFAULT_IGNORE.iter().copied() {
        if let Ok(g) = Glob::new(p) {
            builder.add(g);
        }
    }
    for p in opts.ignore.iter() {
        if let Ok(g) = Glob::new(p) {
            builder.add(g);
        }
    }
    let ignore_set = builder.build()?;

    // Collect candidates.
    let mut candidates: Vec<(PathBuf, String)> = Vec::new();
    let walker = WalkBuilder::new(&opts.root)
        .hidden(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .parents(false)
        .build();
    for entry in walker.flatten() {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let abs = entry.into_path();
        let rel = abs
            .strip_prefix(&opts.root)
            .unwrap_or(&abs)
            .to_string_lossy()
            .into_owned();
        if ignore_set.is_match(&rel) {
            continue;
        }
        candidates.push((abs, rel));
    }

    // Process in parallel.
    let fixes: Vec<FileFix> = candidates
        .par_iter()
        .filter_map(|(abs, rel)| process_file(abs, rel, opts).transpose())
        .filter_map(|r| r.ok())
        .collect();

    Ok(fixes)
}

fn process_file(abs: &Path, rel: &str, opts: &RunOptions) -> Result<Option<FileFix>> {
    let name = abs.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let ext = abs
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    if RIVAL_LOCKFILES.contains(&name) {
        if let Some(f) = check_lockfile(rel, name) {
            return Ok(Some(FileFix {
                file: rel.to_string(),
                before: String::new(),
                after: String::new(),
                findings: vec![f],
            }));
        }
        return Ok(None);
    }

    let is_workflow = is_workflow(rel);
    let is_pkg = name == "package.json";
    let is_source = SOURCE_EXTS.contains(&ext.as_str());
    let is_shell = SHELL_EXTS.contains(&ext.as_str()) || SHELL_NAMES.contains(&name);
    let is_dockerfile = name == "Dockerfile"
        || name.starts_with("Dockerfile.")
        || name.ends_with(".dockerfile");
    let is_nvmrc = name == ".nvmrc" || name == ".node-version";
    let is_tsconfig = name == "tsconfig.json" || name.starts_with("tsconfig.");
    let is_husky = is_husky_hook(rel);
    let is_pnpm_ws = name == "pnpm-workspace.yaml";
    let is_bunfig = name == "bunfig.toml";
    let is_next = is_next_config(name);
    let is_rc = is_rc_file(name);
    let is_cargo = is_cargo_toml(name);

    if !is_pkg
        && !is_source
        && !is_shell
        && !is_workflow
        && !is_dockerfile
        && !is_nvmrc
        && !is_tsconfig
        && !is_husky
        && !is_pnpm_ws
        && !is_bunfig
        && !is_next
        && !is_rc
        && !is_cargo
    {
        return Ok(None);
    }

    // Size cap 2 MiB.
    let meta = match std::fs::metadata(abs) {
        Ok(m) => m,
        Err(_) => return Ok(None),
    };
    if meta.len() > 2 * 1024 * 1024 {
        return Ok(None);
    }
    let before = match std::fs::read_to_string(abs) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };

    let (findings, after) = if is_pkg {
        scan_package_json(rel, &before)
    } else if is_workflow {
        scan_workflow(rel, &before)
    } else if is_source {
        scan_source(rel, &before, opts)
    } else if is_dockerfile {
        scan_dockerfile(rel, &before)
    } else if is_nvmrc {
        scan_nvmrc(rel, &before)
    } else if is_tsconfig {
        scan_tsconfig(rel, &before)
    } else if is_husky {
        scan_husky(rel, &before)
    } else if is_pnpm_ws {
        (scan_pnpm_workspace(rel, &before), before.clone())
    } else if is_bunfig {
        scan_bunfig(rel, &before)
    } else if is_next {
        scan_next_config(rel, &before)
    } else if is_rc {
        scan_npmrc(rel, &before)
    } else if is_cargo {
        scan_cargo_toml(rel, &before)
    } else {
        scan_shell(rel, &before)
    };

    if findings.is_empty() && before == after {
        return Ok(None);
    }

    if opts.mode != Mode::Check && !opts.dry_run && after != before {
        std::fs::write(abs, &after)?;
    }

    Ok(Some(FileFix {
        file: rel.to_string(),
        before,
        after,
        findings,
    }))
}
