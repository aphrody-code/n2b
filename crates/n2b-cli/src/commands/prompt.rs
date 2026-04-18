/// Commande `n2b prompt` — génère un prompt markdown pour LLM.
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;

use n2b_core::types::{Mode, Report, RunOptions, Severity};
use n2b_core::{report, run};

pub fn run_prompt(
    root: PathBuf,
    max_findings: usize,
    include_info: bool,
    ignore: Vec<String>,
    agent: bool,
) -> Result<ExitCode> {
    let root = root.canonicalize().unwrap_or(root);
    let opts = RunOptions {
        root,
        mode: Mode::Check,
        report: Report::Text,
        quiet: true,
        ignore,
        agent,
        dry_run: false,
    };
    let mut fixes = run::run(&opts)?;
    if !include_info {
        for f in &mut fixes {
            f.findings.retain(|x| x.severity != Severity::Info);
        }
        fixes.retain(|f| !f.findings.is_empty());
    }
    print!("{}", report::render_prompt(&fixes, &opts, max_findings));
    Ok(ExitCode::SUCCESS)
}
