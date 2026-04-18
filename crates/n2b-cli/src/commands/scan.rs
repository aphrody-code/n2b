/// Mode scan par défaut (check / fix / aggressive / migrate).
use std::process::ExitCode;

use anyhow::Result;

use crate::cli::args::{Cli, ReportArg};
use n2b_core::types::{Mode, Report, RunOptions, Severity};
use n2b_core::{report, run};

pub fn run(cli: &Cli) -> Result<ExitCode> {
    let mode = if cli.migrate || cli.aggressive {
        Mode::Aggressive
    } else if cli.fix {
        Mode::Fix
    } else {
        Mode::Check
    };
    let root = cli.root.canonicalize().unwrap_or_else(|_| cli.root.clone());
    // En mode agent, un format text implicite est promu en JSON pour garder
    // stdout parsable ; le user peut forcer jsonl/md via --report.
    let effective_report: Report = if cli.agent && matches!(cli.report, ReportArg::Text) {
        Report::Json
    } else {
        cli.report.into()
    };
    let opts = RunOptions {
        root,
        mode,
        report: effective_report,
        quiet: cli.quiet,
        ignore: cli.ignore.clone(),
        agent: cli.agent,
        dry_run: false,
    };

    let fixes = run::run(&opts)?;

    // Mode --migrate : applique les side-effects après le scan+fix.
    if cli.migrate {
        crate::commands::migrate::run_migrate_side_effects(&opts.root, &fixes, opts.quiet)?;
    }

    match opts.report {
        Report::Json => println!("{}", report::render_json(&fixes, &opts)),
        Report::Jsonl => print!("{}", report::render_jsonl(&fixes, &opts)),
        Report::Markdown => println!("{}", report::render_markdown(&fixes, &opts)),
        Report::Sarif => println!("{}", report::render_sarif(&fixes, &opts)),
        Report::Text if !opts.quiet => print!("{}", report::render_text(&fixes, &opts)),
        _ => {}
    }

    let has_errors = fixes
        .iter()
        .any(|f| f.findings.iter().any(|x| x.severity == Severity::Error));
    let has_findings = fixes.iter().any(|f| !f.findings.is_empty());

    Ok(if has_errors {
        ExitCode::from(2)
    } else if opts.mode == Mode::Check && has_findings {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}
