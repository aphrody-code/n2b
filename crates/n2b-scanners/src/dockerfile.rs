use n2b_rules::cli_commands::apply_cli_rules;
use n2b_types::types::{Finding, MakeFindingOpts};
use n2b_util::{line_offsets, make_finding};
use once_cell::sync::Lazy;
use regex::Regex;

static FROM_NODE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*FROM\s+node:(\S+)")
        .expect("invariant: FROM_NODE_RE regex literal is valid")
});

pub fn scan_dockerfile(path: &str, content: &str) -> (Vec<Finding>, String) {
    let mut findings: Vec<Finding> = Vec::new();
    let mut out = content.to_string();
    let offsets = line_offsets(content);

    // FROM node:<tag>  →  FROM oven/bun:<tag>  si le tag n'est pas une version spécifique Node.
    // On laisse le tag tel quel (probablement alpine/bookworm), oven/bun publie des tags analogues.
    for cap in FROM_NODE_RE.captures_iter(content) {
        let whole = cap
            .get(0)
            .expect("invariant: capture group 0 is always present in a match");
        let tag = cap
            .get(1)
            .expect("invariant: capture group 1 is guaranteed by FROM_NODE_RE pattern")
            .as_str();
        let suggested = format!("FROM oven/bun:{tag}");
        findings.push(make_finding(
            path,
            &offsets,
            whole.start(),
            "docker/node-base",
            format!(
                "base image `node:{tag}` → `oven/bun:{tag}` (ou `oven/bun:latest`) évite l'installation de Node/npm"
            ),
            whole.as_str().to_string(),
            Some(suggested),
            MakeFindingOpts {
                autofix: Some(true),
                ..Default::default()
            },
        ));
    }
    out = FROM_NODE_RE
        .replace_all(&out, "FROM oven/bun:$1")
        .into_owned();

    // Applique aussi les règles CLI (npm/pnpm/yarn → bun) dans les RUN.
    let (cli_f, cli_out) = apply_cli_rules(path, &out);
    findings.extend(cli_f);
    (findings, cli_out)
}
