//! `n2b llmstxt <url>` — générateur llms.txt / llms-full.txt.
//!
//! Migré depuis le crate autonome `llmstxt-rs` : orchestre `siteone-crawler`
//! pour rapatrier un site entier en Markdown, puis parse/regroupe et produit
//! les 2 fichiers spec officiels :
//!   - https://llmstxt.org/ (index hiérarchique)
//!   - llms-full.txt (concat complet)
//!
//! Feature `js` (rendering SPA via Chromium) non portée — si besoin,
//! réutiliser le crate `llmstxt-rs` autonome avec `--features js`.

pub mod crawl;
pub mod generate;
pub mod parse;
pub mod summarize;

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;

/// Options du sous-commande `n2b llmstxt`. Re-typé ici au lieu de dépendre
/// directement de la struct clap pour que les sous-modules restent portables.
#[derive(Debug, Clone)]
pub struct LlmstxtOpts {
    pub url: String,
    pub out: PathBuf,
    pub max_depth: usize,
    pub max_pages: usize,
    pub rps: u32,
    pub concurrency: u32,
    pub user_agent: Option<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub full: bool,
    pub summarize: bool,
    pub model: String,
    pub keep_intermediate: bool,
    pub skip_crawl: bool,
    pub quiet: bool,
}

pub fn run(opts: &LlmstxtOpts) -> Result<()> {
    std::fs::create_dir_all(&opts.out)
        .with_context(|| format!("mkdir {}", opts.out.display()))?;
    let md_dir = opts.out.join("_siteone_md");
    let md_single = opts.out.join("_siteone_single.md");

    if !opts.skip_crawl {
        eprintln!(
            "{} siteone-crawler → {}",
            "[1/3]".cyan().bold(),
            md_dir.display()
        );
        crawl::run_siteone(opts, &md_dir, &md_single)?;
    } else {
        eprintln!("{} skip crawl", "[1/3]".dimmed());
    }

    eprintln!("{} parse markdown export", "[2/3]".cyan().bold());
    let site = parse::scan_export(&md_dir, &opts.url)?;
    eprintln!(
        "       {} pages détectées, {} sections",
        site.pages.len(),
        site.sections().len()
    );

    if opts.summarize {
        eprintln!(
            "{} summarize via Claude {}",
            "[2b]".cyan().bold(),
            opts.model
        );
        summarize::run(&site, &opts.model)?;
    }

    eprintln!(
        "{} génération llms.txt + llms-full.txt",
        "[3/3]".cyan().bold()
    );
    let index_path = opts.out.join("llms.txt");
    generate::write_index(&site, &index_path)?;
    eprintln!(
        "       {} llms.txt ({} bytes)",
        "✓".green(),
        std::fs::metadata(&index_path)
            .map(|m| m.len())
            .unwrap_or(0)
    );

    if opts.full {
        let full_path = opts.out.join("llms-full.txt");
        generate::write_full(&site, &md_single, &full_path)?;
        eprintln!(
            "       {} llms-full.txt ({} bytes)",
            "✓".green(),
            std::fs::metadata(&full_path)
                .map(|m| m.len())
                .unwrap_or(0)
        );
    }

    if !opts.keep_intermediate {
        let _ = std::fs::remove_dir_all(&md_dir);
        let _ = std::fs::remove_file(&md_single);
    }

    eprintln!("{} fait → {}", "✓".green().bold(), opts.out.display());
    Ok(())
}
