//! Orchestre `siteone-crawler` pour télécharger un site entier en Markdown.
//!
//! On invoque :
//!   siteone-crawler --url <url>
//!     --markdown-export-dir <md_dir>
//!     --markdown-export-single-file <md_single>
//!     --max-reqs-per-sec <rps>
//!     --workers <concurrency>
//!     --markdown-remove-links-and-images-from-single-file
//!     --offline-export-no-auto-redirect-html
//!     --include-regex / --ignore-regex

use super::LlmstxtOpts;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn run_siteone(cli: &LlmstxtOpts, md_dir: &Path, md_single: &Path) -> Result<()> {
    let bin = which::which("siteone-crawler")
        .context("siteone-crawler introuvable dans PATH — `cargo install siteone-crawler`")?;

    // Purge d'un éventuel run précédent pour éviter les pages orphelines.
    let _ = std::fs::remove_dir_all(md_dir);
    let _ = std::fs::remove_file(md_single);
    std::fs::create_dir_all(md_dir)?;

    let mut cmd = Command::new(&bin);
    cmd.arg(format!("--url={}", cli.url))
        .arg(format!("--markdown-export-dir={}", md_dir.display()))
        .arg(format!(
            "--markdown-export-single-file={}",
            md_single.display()
        ))
        .arg(format!("--max-reqs-per-sec={}", cli.rps))
        .arg(format!("--workers={}", cli.concurrency))
        .arg("--markdown-remove-links-and-images-from-single-file")
        .arg("--offline-export-no-auto-redirect-html")
        .arg("--no-color");

    if cli.max_depth > 0 {
        cmd.arg(format!("--max-depth={}", cli.max_depth));
    }
    if cli.max_pages > 0 {
        cmd.arg(format!("--max-visited-urls={}", cli.max_pages));
    }
    if let Some(ua) = &cli.user_agent {
        cmd.arg(format!("--user-agent={ua}"));
    }
    if cli.quiet {
        cmd.arg("--hide-progress-bar");
    }
    for pat in &cli.include {
        cmd.arg(format!("--include-regex={pat}"));
    }
    for pat in &cli.exclude {
        cmd.arg(format!("--ignore-regex={pat}"));
    }

    let status = cmd
        .status()
        .with_context(|| format!("exécution {}", bin.display()))?;
    if !status.success() {
        anyhow::bail!("siteone-crawler exit {status}");
    }
    Ok(())
}
