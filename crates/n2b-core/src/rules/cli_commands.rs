use crate::types::{Finding, MakeFindingOpts};
use crate::util::{line_offsets, make_finding};
use once_cell::sync::Lazy;
use regex::Regex;

struct Mapping {
    re: Regex,
    replace: &'static str,
    rule_id: &'static str,
    message: &'static str,
}

macro_rules! m {
    ($pat:expr, $repl:expr, $id:expr, $msg:expr) => {
        Mapping {
            re: Regex::new($pat).unwrap_or_else(|e| {
                panic!(
                    "invariant: cli_commands mapping pattern '{}' is invalid: {}",
                    $pat, e
                )
            }),
            replace: $repl,
            rule_id: $id,
            message: $msg,
        }
    };
}

static MAPPINGS: Lazy<Vec<Mapping>> = Lazy::new(|| {
    vec![
        // npm
        m!(
            r"\bnpm\s+install\s+--save-dev\b",
            "bun add --dev",
            "cli/npm-install-dev",
            "npm install --save-dev → bun add --dev"
        ),
        m!(
            r"\bnpm\s+install\s+-D\b",
            "bun add --dev",
            "cli/npm-install-D",
            "npm install -D → bun add --dev"
        ),
        m!(
            r"\bnpm\s+install\s+--save\b",
            "bun add",
            "cli/npm-install-save",
            "npm install --save → bun add"
        ),
        m!(
            r"\bnpm\s+i\s+--save-dev\b",
            "bun add --dev",
            "cli/npm-i-dev",
            "npm i --save-dev → bun add --dev"
        ),
        m!(
            r"\bnpm\s+i\s+-D\b",
            "bun add --dev",
            "cli/npm-i-D",
            "npm i -D → bun add --dev"
        ),
        m!(
            r"\bnpm\s+install\b(?:\s+-g)?",
            "bun install",
            "cli/npm-install",
            "npm install → bun install"
        ),
        m!(
            r"\bnpm\s+i\b",
            "bun install",
            "cli/npm-i",
            "npm i → bun install"
        ),
        m!(
            r"\bnpm\s+ci\b",
            "bun install --frozen-lockfile",
            "cli/npm-ci",
            "npm ci → bun install --frozen-lockfile"
        ),
        m!(
            r"\bnpm\s+run\s+",
            "bun run ",
            "cli/npm-run",
            "npm run → bun run"
        ),
        m!(
            r"\bnpm\s+test\b",
            "bun test",
            "cli/npm-test",
            "npm test → bun test"
        ),
        m!(
            r"\bnpm\s+start\b",
            "bun start",
            "cli/npm-start",
            "npm start → bun start"
        ),
        m!(
            r"\bnpm\s+uninstall\b",
            "bun remove",
            "cli/npm-uninstall",
            "npm uninstall → bun remove"
        ),
        m!(
            r"\bnpm\s+rm\b",
            "bun remove",
            "cli/npm-rm",
            "npm rm → bun remove"
        ),
        m!(
            r"\bnpm\s+update\b",
            "bun update",
            "cli/npm-update",
            "npm update → bun update"
        ),
        m!(
            r"\bnpm\s+outdated\b",
            "bun outdated",
            "cli/npm-outdated",
            "npm outdated → bun outdated"
        ),
        m!(
            r"\bnpm\s+pack\b",
            "bun pm pack",
            "cli/npm-pack",
            "npm pack → bun pm pack"
        ),
        m!(
            r"\bnpm\s+publish\b",
            "bun publish",
            "cli/npm-publish",
            "npm publish → bun publish"
        ),
        m!(
            r"\bnpm\s+link\b",
            "bun link",
            "cli/npm-link",
            "npm link → bun link"
        ),
        m!(
            r"\bnpm\s+init\s+-y\b",
            "bun init -y",
            "cli/npm-init-y",
            "npm init -y → bun init -y"
        ),
        m!(
            r"\bnpm\s+init\b",
            "bun init",
            "cli/npm-init",
            "npm init → bun init"
        ),
        m!(
            r"\bnpm\s+exec\s+",
            "bunx ",
            "cli/npm-exec",
            "npm exec → bunx"
        ),
        m!(r"\bnpx\s+", "bunx ", "cli/npx", "npx → bunx"),
        // pnpm
        m!(
            r"\bpnpm\s+install\s+--save-dev\b",
            "bun add --dev",
            "cli/pnpm-install-dev",
            "pnpm install --save-dev → bun add --dev"
        ),
        m!(
            r"\bpnpm\s+add\s+--save-dev\b",
            "bun add --dev",
            "cli/pnpm-add-dev",
            "pnpm add --save-dev → bun add --dev"
        ),
        m!(
            r"\bpnpm\s+add\s+-D\b",
            "bun add --dev",
            "cli/pnpm-add-D",
            "pnpm add -D → bun add --dev"
        ),
        m!(
            r"\bpnpm\s+install\b",
            "bun install",
            "cli/pnpm-install",
            "pnpm install → bun install"
        ),
        m!(
            r"\bpnpm\s+i\b",
            "bun install",
            "cli/pnpm-i",
            "pnpm i → bun install"
        ),
        m!(
            r"\bpnpm\s+add\b",
            "bun add",
            "cli/pnpm-add",
            "pnpm add → bun add"
        ),
        m!(
            r"\bpnpm\s+remove\b",
            "bun remove",
            "cli/pnpm-remove",
            "pnpm remove → bun remove"
        ),
        m!(
            r"\bpnpm\s+rm\b",
            "bun remove",
            "cli/pnpm-rm",
            "pnpm rm → bun remove"
        ),
        m!(
            r"\bpnpm\s+run\s+",
            "bun run ",
            "cli/pnpm-run",
            "pnpm run → bun run"
        ),
        m!(
            r"\bpnpm\s+test\b",
            "bun test",
            "cli/pnpm-test",
            "pnpm test → bun test"
        ),
        m!(
            r"\bpnpm\s+start\b",
            "bun start",
            "cli/pnpm-start",
            "pnpm start → bun start"
        ),
        m!(
            r"\bpnpm\s+update\b",
            "bun update",
            "cli/pnpm-update",
            "pnpm update → bun update"
        ),
        m!(
            r"\bpnpm\s+dlx\s+",
            "bunx ",
            "cli/pnpm-dlx",
            "pnpm dlx → bunx"
        ),
        m!(
            r"\bpnpm\s+exec\s+",
            "bunx ",
            "cli/pnpm-exec",
            "pnpm exec → bunx"
        ),
        // yarn
        m!(
            r"\byarn\s+install\s+--frozen-lockfile\b",
            "bun install --frozen-lockfile",
            "cli/yarn-install-frozen",
            "yarn install --frozen-lockfile → bun install --frozen-lockfile"
        ),
        m!(
            r"\byarn\s+add\s+--dev\b",
            "bun add --dev",
            "cli/yarn-add-dev",
            "yarn add --dev → bun add --dev"
        ),
        m!(
            r"\byarn\s+add\s+-D\b",
            "bun add --dev",
            "cli/yarn-add-D",
            "yarn add -D → bun add --dev"
        ),
        m!(
            r"\byarn\s+add\b",
            "bun add",
            "cli/yarn-add",
            "yarn add → bun add"
        ),
        m!(
            r"\byarn\s+remove\b",
            "bun remove",
            "cli/yarn-remove",
            "yarn remove → bun remove"
        ),
        m!(
            r"\byarn\s+install\b",
            "bun install",
            "cli/yarn-install",
            "yarn install → bun install"
        ),
        m!(
            r"\byarn\s+run\s+",
            "bun run ",
            "cli/yarn-run",
            "yarn run → bun run"
        ),
        m!(
            r"\byarn\s+test\b",
            "bun test",
            "cli/yarn-test",
            "yarn test → bun test"
        ),
        m!(
            r"\byarn\s+start\b",
            "bun start",
            "cli/yarn-start",
            "yarn start → bun start"
        ),
        m!(
            r"\byarn\s+dlx\s+",
            "bunx ",
            "cli/yarn-dlx",
            "yarn dlx → bunx"
        ),
        // bare yarn at start of line → bun install (preserve leading ws)
        m!(
            r"(?m)^(\s*)yarn(\s|$)",
            "${1}bun install${2}",
            "cli/yarn-bare",
            "bare `yarn` → bun install"
        ),
    ]
});

static COMMENT_PREFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(#|//)").expect("invariant: COMMENT_PREFIX regex literal is valid")
});

/// Reproduit applyCliRules : applique chaque mapping en séquence, en
/// ignorant les lignes commentées, et collecte findings + contenu modifié.
pub fn apply_cli_rules(path: &str, source: &str) -> (Vec<Finding>, String) {
    let mut out = source.to_string();
    let mut findings: Vec<Finding> = Vec::new();
    // line_offsets est recalculé seulement si le contenu change entre règles
    // (c'est rare : la plupart des règles ne matchent pas). Mémoïsation simple.
    let mut offsets = line_offsets(&out);
    let mut offsets_stale = false;

    for rule in MAPPINGS.iter() {
        if offsets_stale {
            offsets = line_offsets(&out);
            offsets_stale = false;
        }
        let hits: Vec<(usize, String)> = rule
            .re
            .find_iter(&out)
            .filter(|mat| {
                let line_start = out[..mat.start()].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let line_end = out[mat.start()..]
                    .find('\n')
                    .map(|p| mat.start() + p)
                    .unwrap_or(out.len());
                let line = &out[line_start..line_end];
                !COMMENT_PREFIX.is_match(line)
            })
            .map(|mat| (mat.start(), mat.as_str().to_string()))
            .collect();

        if hits.is_empty() {
            continue;
        }

        for (idx, text) in &hits {
            // Calcule la valeur de remplacement exacte en rejouant la regex
            // sur le hit pour conserver les back-refs ($1, $2).
            let rewritten = rule.re.replace(text, rule.replace).to_string();
            findings.push(make_finding(
                path,
                &offsets,
                *idx,
                rule.rule_id,
                rule.message,
                text.clone(),
                Some(rewritten),
                MakeFindingOpts {
                    autofix: Some(true),
                    ..Default::default()
                },
            ));
        }

        // Remplacement global (ignore les commentaires côté dommage : on
        // reproduit le comportement historique qui lui remplaçait globalement).
        let before = out.clone();
        out = rule.re.replace_all(&out, rule.replace).into_owned();
        if out != before {
            offsets_stale = true;
        }
    }

    (findings, out)
}
