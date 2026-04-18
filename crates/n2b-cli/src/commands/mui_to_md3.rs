//! `n2b mui-to-md3` — migre un projet MUI v9 → @md3-ui/core.
//!
//! v1 : rewrites ciblés via regex (imports + props simples). Les transformations
//! structurelles (Dialog anatomie, Tabs, Grid→div) sont flaggées `manual` dans
//! le rapport sans être appliquées.
//!
//! Règles : `n2b/rules/mui-to-md3.yaml` (embarquées via `include_str!`).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use n2b_core::types::Report;

// Règles embarquées au build — pas de read runtime si pas `--rules path`.
const BUNDLED_RULES_YAML: &str = include_str!("../../../../rules/mui-to-md3.yaml");

#[allow(dead_code)]
pub struct MuiToMd3Opts {
    pub root: PathBuf,
    pub write: bool,
    pub stage_atomic: bool,
    pub only: Vec<String>,
    pub rewrite_sx: bool,
    pub ignore: Vec<String>,
    pub rules: Option<PathBuf>,
    pub report: Report,
    pub quiet: bool,
    pub agent: bool,
}

pub fn run(opts: MuiToMd3Opts) -> Result<()> {
    let rules = load_rules(opts.rules.as_deref())?;

    let root = opts
        .root
        .canonicalize()
        .unwrap_or_else(|_| opts.root.clone());
    if !root.is_dir() {
        anyhow::bail!(
            "{} n'est pas un dossier — pas de projet à migrer",
            root.display()
        );
    }

    // Collect fichiers .tsx / .ts (évite node_modules, .next, etc.)
    let files = collect_source_files(&root, &opts.ignore, &rules.ignore);
    if !opts.quiet && !opts.agent {
        eprintln!(
            "{} scan {} fichiers sous {}",
            "[mui-to-md3]".bold().cyan(),
            files.len(),
            root.display()
        );
    }

    let only: std::collections::HashSet<String> = opts
        .only
        .iter()
        .map(|s| s.to_ascii_lowercase())
        .collect();

    let mut report = MigrationReport::default();

    // Filtre les règles actives + matchant --only
    let active_rules: Vec<&Rule> = rules
        .rules
        .iter()
        .filter(|r| r.status == "active")
        .filter(|r| {
            if only.is_empty() {
                true
            } else {
                r.from.imports.iter().any(|imp| {
                    only.contains(&imp.to_ascii_lowercase())
                        || only.contains(&r.id.replace("mui-", "").to_ascii_lowercase())
                })
            }
        })
        .collect();

    for file in &files {
        let src = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut rewritten = src.clone();
        let mut file_changes: Vec<RuleHit> = Vec::new();

        for rule in &active_rules {
            let hits = apply_import_rewrite(&mut rewritten, rule);
            if hits > 0 {
                file_changes.push(RuleHit {
                    rule_id: rule.id.clone(),
                    from_package: rule.from.package.clone(),
                    to_package: rule
                        .to
                        .as_ref()
                        .map(|t| t.package.clone())
                        .unwrap_or_else(|| "<REMOVED>".into()),
                    imports: rule.from.imports.clone(),
                    count: hits,
                });
                // Applique prop-transforms seulement si on a matché un import.
                for pt in &rule.prop_transforms {
                    apply_prop_transform(&mut rewritten, rule, pt);
                }
            }
        }

        if rewritten != src {
            let rel = file.strip_prefix(&root).unwrap_or(file).to_path_buf();
            report.files.push(FileReport {
                path: rel.display().to_string(),
                rules_applied: file_changes.clone(),
                bytes_before: src.len(),
                bytes_after: rewritten.len(),
            });
            for hit in &file_changes {
                *report
                    .by_rule
                    .entry(hit.rule_id.clone())
                    .or_insert(0) += hit.count;
            }

            if opts.write {
                fs::write(file, &rewritten)
                    .with_context(|| format!("write {}", file.display()))?;
                if opts.stage_atomic {
                    // Regroupe le commit par règle majoritaire du fichier
                    if let Some(top) = file_changes
                        .iter()
                        .max_by_key(|h| h.count)
                        .map(|h| h.rule_id.clone())
                    {
                        git_stage_file(&root, &rel, opts.quiet)?;
                        git_commit(
                            &root,
                            &format!("migrate({}): {} to md3-ui", top, top.replace("mui-", "")),
                            opts.quiet,
                        )?;
                    }
                }
            }
        }
    }

    render_report(&report, &opts);

    Ok(())
}

// ─── Rules loader ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RulesFile {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub ignore: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Rule {
    pub id: String,
    #[serde(default = "default_effort")]
    pub effort: String,
    #[serde(default = "default_status")]
    pub status: String,
    pub from: RuleFrom,
    #[serde(default)]
    pub to: Option<RuleTo>,
    #[serde(default, rename = "prop-transforms")]
    pub prop_transforms: Vec<PropTransform>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuleFrom {
    pub package: String,
    #[serde(default)]
    pub imports: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct RuleTo {
    pub package: String,
    #[serde(default)]
    pub imports: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PropTransform {
    #[serde(default)]
    pub from: serde_yaml::Value,
    #[serde(default)]
    pub to: serde_yaml::Value,
}

fn default_effort() -> String {
    "low".into()
}
fn default_status() -> String {
    "active".into()
}

fn load_rules(path: Option<&Path>) -> Result<RulesFile> {
    let yaml = match path {
        Some(p) => fs::read_to_string(p)
            .with_context(|| format!("read rules {}", p.display()))?,
        None => BUNDLED_RULES_YAML.to_string(),
    };
    let rules: RulesFile =
        serde_yaml::from_str(&yaml).context("parse mui-to-md3.yaml")?;
    Ok(rules)
}

// ─── File collection ───────────────────────────────────────────────────────

fn collect_source_files(
    root: &Path,
    extra_ignore: &[String],
    rule_ignore: &[String],
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .build();
    let globs = build_glob_matcher(extra_ignore, rule_ignore);
    for entry in walker.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "tsx" && ext != "ts" && ext != "jsx" {
            continue;
        }
        if let Ok(rel) = p.strip_prefix(root) {
            if globs.is_match(rel) {
                continue;
            }
        }
        out.push(p.to_path_buf());
    }
    out
}

fn build_glob_matcher(a: &[String], b: &[String]) -> globset::GlobSet {
    let mut builder = globset::GlobSetBuilder::new();
    for g in a.iter().chain(b.iter()) {
        if let Ok(glob) = globset::Glob::new(g) {
            builder.add(glob);
        }
    }
    builder.build().unwrap_or_else(|_| globset::GlobSet::empty())
}

// ─── Rewriters (regex-based v1) ────────────────────────────────────────────

fn apply_import_rewrite(src: &mut String, rule: &Rule) -> usize {
    let from_pkg = &rule.from.package;
    if !src.contains(from_pkg) {
        return 0;
    }
    let Some(to) = &rule.to else {
        return count_imports_from(src, from_pkg, &rule.from.imports);
    };

    let mut count = 0;
    // 1. Rewrite `from "@mui/material"` → `from "@md3-ui/core/<kebab>"` (per-import split)
    // Strategy : split the named imports of the source package into one import
    // per md3 subpath. For a simple v1, we match the whole `import { ... } from "@mui/material"`
    // statement and rewrite it if all named imports are covered by this rule.
    let re = regex_for_named_import(from_pkg);
    let replacement = build_replacement(&re, src, rule, &to.package);
    if let Some((new, hits)) = replacement {
        *src = new;
        count += hits;
    }

    // 2. Default-import form : `import Button from "@mui/material/Button";`
    let default_re = regex_for_default_import(from_pkg);
    let (new2, hits2) = rewrite_default_import(&default_re, src, rule, &to.package);
    if hits2 > 0 {
        *src = new2;
        count += hits2;
    }

    count
}

fn count_imports_from(src: &str, pkg: &str, imports: &[String]) -> usize {
    let mut n = 0;
    for imp in imports {
        let pat = format!(
            r#"import\s*\{{[^}}]*\b{}\b[^}}]*\}}\s*from\s*["']{}["']"#,
            regex::escape(imp),
            regex::escape(pkg)
        );
        if let Ok(re) = regex::Regex::new(&pat) {
            n += re.find_iter(src).count();
        }
    }
    n
}

fn regex_for_named_import(pkg: &str) -> regex::Regex {
    let pat = format!(
        r#"import\s*\{{\s*([^}}]+?)\s*\}}\s*from\s*["']{}["']\s*;?"#,
        regex::escape(pkg)
    );
    regex::Regex::new(&pat).expect("import regex")
}

fn regex_for_default_import(pkg: &str) -> regex::Regex {
    // import Button from "@mui/material/Button";
    let pat = format!(
        r#"import\s+(\w+)\s+from\s+["']{}/(\w[\w\-]*)["']\s*;?"#,
        regex::escape(pkg)
    );
    regex::Regex::new(&pat).expect("default import regex")
}

fn build_replacement(
    re: &regex::Regex,
    src: &str,
    rule: &Rule,
    to_package: &str,
) -> Option<(String, usize)> {
    let mut out = String::with_capacity(src.len());
    let mut last = 0;
    let mut hits = 0;
    for m in re.captures_iter(src) {
        let whole = m.get(0).unwrap();
        let names_str = m.get(1).unwrap().as_str();
        let names: Vec<String> = names_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Partition names : matched par la règle vs non-matched.
        let mut matched: Vec<String> = Vec::new();
        let mut untouched: Vec<String> = Vec::new();
        for name in &names {
            let stripped = name.split(" as ").next().unwrap_or(name).trim();
            if rule.from.imports.iter().any(|imp| imp == stripped) {
                matched.push(name.clone());
            } else {
                untouched.push(name.clone());
            }
        }
        if matched.is_empty() {
            continue;
        }

        out.push_str(&src[last..whole.start()]);

        if to_package == "NONE" {
            // Suppression totale des imports matchés (le JSX rewriter gère le reste).
            if !untouched.is_empty() {
                let line = format!(
                    "import {{ {} }} from \"{}\";",
                    untouched.join(", "),
                    rule.from.package
                );
                out.push_str(&line);
            }
        } else {
            // Groupement par md3 subpath : si le `to.package` est global (ex @md3-ui/core/button),
            // on l'utilise tel quel. Sinon on kebab-case chaque nom.
            let import_line = format!(
                "import {{ {} }} from \"{}\";",
                matched.join(", "),
                to_package
            );
            out.push_str(&import_line);
            if !untouched.is_empty() {
                let line = format!(
                    "\nimport {{ {} }} from \"{}\";",
                    untouched.join(", "),
                    rule.from.package
                );
                out.push_str(&line);
            }
        }
        hits += matched.len();
        last = whole.end();
    }
    if hits == 0 {
        return None;
    }
    out.push_str(&src[last..]);
    Some((out, hits))
}

fn rewrite_default_import(
    re: &regex::Regex,
    src: &str,
    rule: &Rule,
    to_package: &str,
) -> (String, usize) {
    let mut hits = 0;
    let new = re
        .replace_all(src, |caps: &regex::Captures| {
            let ident = &caps[1];
            let mod_name = &caps[2];
            if rule
                .from
                .imports
                .iter()
                .any(|imp| imp.eq_ignore_ascii_case(ident))
            {
                hits += 1;
                if to_package == "NONE" {
                    String::new()
                } else {
                    format!("import {{ {} }} from \"{}\";", ident, to_package)
                }
            } else {
                format!(
                    "import {} from \"{}/{}\";",
                    ident, rule.from.package, mod_name
                )
            }
        })
        .into_owned();
    (new, hits)
}

fn apply_prop_transform(src: &mut String, rule: &Rule, pt: &PropTransform) {
    // V1 : prop-transform scopé au composant de la règle (évite qu'un
    // `variant="filled"` sur Button se fasse re-transformer par la règle Chip).
    //
    // Pattern : on matche seulement les props à l'intérieur du tag opening
    // de chaque composant de `rule.from.imports`. Ex pour Button :
    //   `<Button [^>]*variant="contained"[^>]*>`
    // et on rewrite la seule occurrence de la prop dans ce match.

    let (Some(from_map), Some(to_map)) = (pt.from.as_mapping(), pt.to.as_mapping()) else {
        return;
    };
    if from_map.len() != 1 {
        return;
    }
    let (from_key, from_val) = from_map.iter().next().unwrap();
    let Some(from_key_str) = from_key.as_str() else {
        return;
    };
    let from_val_str = match from_val {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        _ => return,
    };

    for component in &rule.from.imports {
        // Tag opening ouvert (match jusqu'au `>` ou `/>` non-quoté).
        // On évite DOTALL : un tag opening ne déborde pas sur plusieurs lignes
        // en pratique (limite acceptable v1).
        let tag_pat = format!(r#"<{}(\s+[^>]*?)?(/?>)"#, regex::escape(component));
        let Ok(tag_re) = regex::Regex::new(&tag_pat) else {
            continue;
        };

        // Replace inside each tag opening
        let mut out = String::with_capacity(src.len());
        let mut last = 0usize;
        let mut changed = false;

        for m in tag_re.find_iter(src) {
            out.push_str(&src[last..m.start()]);
            let tag = m.as_str();

            let new_tag = if to_map.is_empty() {
                // Suppression : `color="primary"` → rien
                let remove_pat = format!(
                    r#"\s+{}=["']{}["']"#,
                    regex::escape(from_key_str),
                    regex::escape(&from_val_str)
                );
                match regex::Regex::new(&remove_pat) {
                    Ok(re) => re.replace_all(tag, "").into_owned(),
                    Err(_) => tag.to_string(),
                }
            } else if to_map.len() == 1 {
                let (to_key, to_val) = to_map.iter().next().unwrap();
                let (Some(to_key_str), Some(to_val_str)) = (to_key.as_str(), to_val.as_str())
                else {
                    last = m.end();
                    out.push_str(tag);
                    continue;
                };
                let repl_pat = format!(
                    r#"{}=["']{}["']"#,
                    regex::escape(from_key_str),
                    regex::escape(&from_val_str)
                );
                match regex::Regex::new(&repl_pat) {
                    Ok(re) => re
                        .replace(tag, format!("{}=\"{}\"", to_key_str, to_val_str))
                        .into_owned(),
                    Err(_) => tag.to_string(),
                }
            } else {
                tag.to_string()
            };

            if new_tag != tag {
                changed = true;
            }
            out.push_str(&new_tag);
            last = m.end();
        }
        if changed {
            out.push_str(&src[last..]);
            *src = out;
        }
    }
}

// ─── Git stage helpers ─────────────────────────────────────────────────────

fn git_stage_file(root: &Path, rel: &Path, quiet: bool) -> Result<()> {
    let status = std::process::Command::new("git")
        .arg("add")
        .arg(rel)
        .current_dir(root)
        .status()?;
    if !quiet && !status.success() {
        eprintln!("{} git add {} failed", "[mui-to-md3]".red(), rel.display());
    }
    Ok(())
}

fn git_commit(root: &Path, msg: &str, quiet: bool) -> Result<()> {
    let status = std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(msg)
        .current_dir(root)
        .status()?;
    if !quiet {
        eprintln!(
            "{} git commit: {}",
            if status.success() { "✓".green() } else { "×".red() },
            msg
        );
    }
    Ok(())
}

// ─── Report ────────────────────────────────────────────────────────────────

#[derive(Default, Serialize)]
pub struct MigrationReport {
    pub files: Vec<FileReport>,
    pub by_rule: std::collections::BTreeMap<String, usize>,
}

#[derive(Serialize, Clone)]
pub struct RuleHit {
    pub rule_id: String,
    pub from_package: String,
    pub to_package: String,
    pub imports: Vec<String>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct FileReport {
    pub path: String,
    pub rules_applied: Vec<RuleHit>,
    pub bytes_before: usize,
    pub bytes_after: usize,
}

fn render_report(report: &MigrationReport, opts: &MuiToMd3Opts) {
    if opts.quiet {
        return;
    }
    match opts.report {
        Report::Json => {
            println!("{}", serde_json::to_string_pretty(report).unwrap_or_default());
        }
        Report::Markdown => render_md(report, opts),
        _ => render_text(report, opts),
    }
}

fn render_text(report: &MigrationReport, opts: &MuiToMd3Opts) {
    let total_hits: usize = report.by_rule.values().sum();
    eprintln!();
    eprintln!("{}", "─── MUI → md3-ui migration summary ───".bold());
    eprintln!(
        "{} fichiers touchés · {} imports rewritten · mode : {}",
        report.files.len().to_string().bold(),
        total_hits.to_string().bold(),
        if opts.write { "WRITE".red() } else { "DRY-RUN".yellow() },
    );
    eprintln!();
    if report.by_rule.is_empty() {
        eprintln!("{}", "Aucun import MUI détecté à migrer.".dimmed());
        return;
    }
    eprintln!("{}", "By rule:".bold());
    for (rule, count) in &report.by_rule {
        eprintln!("  {:30} {:>4}", rule.cyan(), count);
    }
    eprintln!();
    eprintln!("{}", "Top files:".bold());
    let mut sorted: Vec<&FileReport> = report.files.iter().collect();
    sorted.sort_by(|a, b| {
        let sa: usize = a.rules_applied.iter().map(|r| r.count).sum();
        let sb: usize = b.rules_applied.iter().map(|r| r.count).sum();
        sb.cmp(&sa)
    });
    for f in sorted.iter().take(20) {
        let sum: usize = f.rules_applied.iter().map(|r| r.count).sum();
        let rules: Vec<String> = f
            .rules_applied
            .iter()
            .map(|r| r.rule_id.replace("mui-", ""))
            .collect();
        eprintln!(
            "  {:>4} {} {}",
            sum.to_string().green(),
            f.path.bold(),
            format!("[{}]", rules.join(", ")).dimmed()
        );
    }
}

fn render_md(report: &MigrationReport, opts: &MuiToMd3Opts) {
    let total: usize = report.by_rule.values().sum();
    println!("# MUI → md3-ui migration report");
    println!();
    println!(
        "- Files changed: **{}**",
        report.files.len()
    );
    println!("- Imports rewritten: **{total}**");
    println!(
        "- Mode: **{}**",
        if opts.write { "WRITE" } else { "DRY-RUN" }
    );
    println!();
    println!("## By rule");
    println!();
    println!("| Rule | Hits |");
    println!("|---|--:|");
    for (rule, count) in &report.by_rule {
        println!("| `{rule}` | {count} |");
    }
    println!();
    println!("## Files");
    println!();
    for f in &report.files {
        let sum: usize = f.rules_applied.iter().map(|r| r.count).sum();
        println!("- `{}` — {sum} rewrites", f.path);
        for r in &f.rules_applied {
            println!("  - `{}` × {} ({} → {})", r.rule_id, r.count, r.from_package, r.to_package);
        }
    }
}
