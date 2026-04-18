use crate::types::{Finding, MakeFindingOpts};
use crate::util::{line_offsets, make_finding};
use once_cell::sync::Lazy;
use regex::Regex;

static SHEBANG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^#!\s*(?:/usr/bin/env\s+node|/usr/bin/node|node)(?:\s|$)")
        .expect("invariant: SHEBANG_RE regex literal is valid")
});

pub fn scan_shebang(path: &str, content: &str) -> (Vec<Finding>, String) {
    let mut findings = Vec::new();
    if !content.starts_with("#!") {
        return (findings, content.to_string());
    }
    let first_line_end = content.find('\n').unwrap_or(content.len());
    let first_line = &content[..first_line_end];
    if !SHEBANG_RE.is_match(first_line) {
        return (findings, content.to_string());
    }
    let replacement = "#!/usr/bin/env bun";
    let offsets = line_offsets(content);
    findings.push(make_finding(
        path,
        &offsets,
        0,
        "shebang/node",
        "shebang 'node' → 'bun'",
        first_line.to_string(),
        Some(replacement.to_string()),
        MakeFindingOpts {
            autofix: Some(true),
            ..Default::default()
        },
    ));
    let rest = &content[first_line.len()..];
    (findings, format!("{replacement}{rest}"))
}
