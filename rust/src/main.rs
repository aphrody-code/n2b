mod ai;
mod analyze;
mod audit;
mod github;
mod patch;
mod report;
mod rules;
mod run;
mod scanners;
mod types;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

use crate::types::{Mode, Report, RunOptions, Severity};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ReportArg {
    Text,
    Json,
    Jsonl,
    Md,
    Markdown,
    Sarif,
}

impl From<ReportArg> for Report {
    fn from(r: ReportArg) -> Self {
        match r {
            ReportArg::Text => Report::Text,
            ReportArg::Json => Report::Json,
            ReportArg::Jsonl => Report::Jsonl,
            ReportArg::Md | ReportArg::Markdown => Report::Markdown,
            ReportArg::Sarif => Report::Sarif,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum StateArg {
    Open,
    Closed,
    All,
}

impl From<StateArg> for audit::ItemState {
    fn from(s: StateArg) -> Self {
        match s {
            StateArg::Open => audit::ItemState::Open,
            StateArg::Closed => audit::ItemState::Closed,
            StateArg::All => audit::ItemState::All,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "n2b", version, about = "n2b — analyse un package et corrige les incompatibilités avec Bun.")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,

    // Flags du scan par défaut (conservés au niveau racine pour compat).
    #[arg(default_value = ".")]
    root: PathBuf,

    #[arg(long)]
    fix: bool,

    #[arg(long)]
    aggressive: bool,

    /// Mode migration complète : applique --fix --aggressive ET exécute les
    /// side-effects (bun install, retrait pnpm-lock.yaml, migration
    /// pnpm-workspace.yaml → package.json, ajout @types/bun si requis).
    #[arg(long)]
    migrate: bool,

    #[arg(long, value_enum, default_value = "text")]
    report: ReportArg,

    #[arg(long)]
    ignore: Vec<String>,

    #[arg(long)]
    quiet: bool,

    /// Mode AI-agent : désactive les couleurs, logs sur stderr,
    /// stdout réservé au payload structuré. Implique --report=json si text.
    #[arg(long, global = true)]
    agent: bool,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Liste toutes les règles connues avec id, catégorie, docs_url.
    Rules {
        /// Format de sortie.
        #[arg(long, value_enum, default_value = "text")]
        report: ReportArg,
    },

    /// Génère un prompt markdown prêt à coller dans un LLM.
    Prompt {
        /// Racine du package (défaut: .)
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Nombre maximum de findings inclus dans le prompt.
        #[arg(long, default_value_t = 50)]
        max_findings: usize,

        /// Inclure les findings Info (par défaut ignorés).
        #[arg(long)]
        include_info: bool,

        /// Glob d'exclusion.
        #[arg(long)]
        ignore: Vec<String>,
    },

    /// Détecte le repo GitHub et scanne issues + PRs mentionnant bun/node.
    Audit {
        /// Racine du package (défaut: .)
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Termes à chercher (défaut: bun, node).
        #[arg(long = "term", short = 't', value_name = "TERM", num_args = 1..)]
        terms: Vec<String>,

        /// État des items : open, closed, all.
        #[arg(long, value_enum, default_value = "all")]
        state: StateArg,

        /// Nombre maximum d'items par catégorie.
        #[arg(long, default_value_t = 30)]
        limit: usize,

        /// Format du rapport.
        #[arg(long, value_enum, default_value = "text")]
        report: ReportArg,
    },

    /// Génère ou applique des patches. Deux modes :
    ///   · `n2b patch <pkg>`  → wrapper `bun patch` : applique les règles n2b
    ///                            sur node_modules/<pkg>, puis `bun patch --commit`.
    ///   · `n2b patch --self` → produit un diff unifié du repo courant
    ///                            (n2b.patch) sans modifier les fichiers.
    Patch {
        /// Nom (ou name@version) du package npm à patcher (mode A).
        package: Option<String>,

        /// Mode B : patch le repo courant au lieu d'une dep npm.
        #[arg(long = "self")]
        self_repo: bool,

        /// Racine du projet.
        #[arg(long, default_value = ".")]
        root: PathBuf,

        /// Applique aussi les règles marquées `aggressive`.
        #[arg(long)]
        aggressive: bool,

        /// Fichier de sortie pour le .patch (mode B).
        #[arg(long, default_value = "n2b.patch")]
        output: PathBuf,

        /// Dossier où `bun patch --commit` écrit les patch files (mode A).
        #[arg(long)]
        patches_dir: Option<PathBuf>,

        /// N'écrit rien (mode B imprime le patch sur stdout ; mode A skip
        /// `bun patch --commit`).
        #[arg(long)]
        dry_run: bool,

        /// Glob d'exclusion supplémentaire (cumulable).
        #[arg(long)]
        ignore: Vec<String>,
    },

    /// Scan + audit + crosslink ML (embeddings) sur un ou plusieurs repos.
    Analyze {
        /// Chemins à analyser (défaut: discord.js/discordx/nextjs détectés dans cwd).
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,

        /// Nombre max d'issues/PRs par repo.
        #[arg(long, default_value_t = 60)]
        issue_limit: usize,

        /// Top-K issues à croiser par finding.
        #[arg(long, default_value_t = 3)]
        top_k: usize,

        /// Seuil minimum de similarité cosinus pour considérer un lien.
        #[arg(long, default_value_t = 0.35)]
        threshold: f32,

        /// Format du rapport.
        #[arg(long, value_enum, default_value = "text")]
        report: ReportArg,

        /// Glob d'exclusion supplémentaire (cumulable).
        #[arg(long)]
        ignore: Vec<String>,

        /// Applique aussi les fixes (fix ou aggressive).
        #[arg(long, value_name = "MODE")]
        apply: Option<ApplyArg>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ApplyArg {
    Fix,
    Aggressive,
}

impl From<ApplyArg> for Mode {
    fn from(a: ApplyArg) -> Self {
        match a {
            ApplyArg::Fix => Mode::Fix,
            ApplyArg::Aggressive => Mode::Aggressive,
        }
    }
}

fn main() -> ExitCode {
    match real_main() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("n2b a échoué : {err:?}");
            ExitCode::from(2)
        }
    }
}

fn real_main() -> Result<ExitCode> {
    let cli = Cli::parse();

    // Mode agent : coupe les couleurs pour que stderr ne contienne pas d'ANSI.
    if cli.agent {
        colored::control::set_override(false);
    }

    match cli.cmd {
        Some(Cmd::Rules { report }) => {
            return run_rules(report.into());
        }
        Some(Cmd::Prompt { root, max_findings, include_info, ignore }) => {
            return run_prompt(root, max_findings, include_info, ignore, cli.agent);
        }
        Some(Cmd::Audit { root, terms, state, limit, report }) => {
            return run_audit(root, terms, state.into(), limit, report.into());
        }
        Some(Cmd::Patch {
            package,
            self_repo,
            root,
            aggressive,
            output,
            patches_dir,
            dry_run,
            ignore,
        }) => {
            patch::run_patch(patch::PatchOpts {
                package,
                self_repo,
                root,
                aggressive,
                output,
                patches_dir,
                dry_run,
                ignore,
                quiet: cli.quiet,
            })?;
            return Ok(ExitCode::SUCCESS);
        }
        Some(Cmd::Analyze { paths, issue_limit, top_k, threshold, report, ignore, apply }) => {
            let cwd = std::env::current_dir()?;
            let paths = if paths.is_empty() {
                analyze::resolve_default_paths(&cwd)
            } else {
                paths
            };
            if paths.is_empty() {
                anyhow::bail!(
                    "aucun chemin fourni et aucun candidat (discord.js/discordx/nextjs) trouvé dans {}",
                    cwd.display()
                );
            }
            analyze::run_analyze(analyze::AnalyzeOpts {
                paths,
                issue_limit,
                top_k,
                threshold,
                report: report.into(),
                apply: apply.map(Into::into),
                ignore,
            })?;
            return Ok(ExitCode::SUCCESS);
        }
        None => {}
    }

    // Scan (mode par défaut).
    let mode = if cli.migrate || cli.aggressive {
        Mode::Aggressive
    } else if cli.fix {
        Mode::Fix
    } else {
        Mode::Check
    };
    let root = cli.root.canonicalize().unwrap_or(cli.root.clone());
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
        ignore: cli.ignore,
        agent: cli.agent,
        dry_run: false,
    };

    let fixes = run::run(&opts)?;

    // Mode --migrate : applique les side-effects après le scan+fix.
    if cli.migrate {
        run_migrate_side_effects(&opts.root, &fixes, opts.quiet)?;
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

/// Applique les side-effects du mode `--migrate` :
///  1. Migre pnpm-workspace.yaml → `workspaces` + `trustedDependencies` dans package.json
///  2. Retire pnpm-lock.yaml / yarn.lock / package-lock.json
///  3. Exécute `bun install` pour reconstruire bun.lock
///  4. Ajoute `@types/bun` en devDep si source utilise `Bun.*`
fn run_migrate_side_effects(
    root: &PathBuf,
    fixes: &[crate::types::FileFix],
    quiet: bool,
) -> Result<()> {
    use std::process::Command;

    let log = |msg: &str| {
        if !quiet {
            eprintln!("[migrate] {msg}");
        }
    };

    // 1. pnpm-workspace.yaml → package.json
    let pnpm_ws = root.join("pnpm-workspace.yaml");
    let root_pkg = root.join("package.json");
    if pnpm_ws.exists() && root_pkg.exists() {
        let pnpm_content = std::fs::read_to_string(&pnpm_ws)?;
        if let Some(info) =
            crate::scanners::pnpm_workspace::parse_pnpm_workspace(&pnpm_content)
        {
            let pkg_content = std::fs::read_to_string(&root_pkg)?;
            let mut pkg: serde_json::Value = serde_json::from_str(&pkg_content)?;
            let mut mutated = false;
            if pkg.get("workspaces").is_none() && !info.packages.is_empty() {
                pkg["workspaces"] = serde_json::json!(info.packages);
                mutated = true;
                log("  + workspaces ajouté dans package.json");
            }
            if pkg.get("trustedDependencies").is_none() && !info.only_built.is_empty() {
                pkg["trustedDependencies"] = serde_json::json!(info.only_built);
                mutated = true;
                log("  + trustedDependencies ajouté dans package.json");
            }
            if mutated {
                let mut out = serde_json::to_string_pretty(&pkg)?;
                if pkg_content.ends_with('\n') && !out.ends_with('\n') {
                    out.push('\n');
                }
                std::fs::write(&root_pkg, out)?;
            }
            std::fs::remove_file(&pnpm_ws)?;
            log("  - pnpm-workspace.yaml supprimé");
        }
    }

    // 2. Retire les lockfiles concurrents
    for name in crate::scanners::lockfile::RIVAL_LOCKFILES.iter() {
        let p = root.join(name);
        if p.exists() {
            std::fs::remove_file(&p)?;
            log(&format!("  - {name} supprimé"));
        }
    }

    // 3. bun install (reconstruit bun.lock) — stdout/stderr capturés pour
    //    ne pas polluer le report JSON du parent.
    log("  → bun install");
    let out = Command::new("bun")
        .arg("install")
        .current_dir(root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();
    match out {
        Ok(o) if o.status.success() => log("  ✓ bun install OK"),
        Ok(o) => log(&format!(
            "  ✗ bun install exit={}\n{}",
            o.status,
            String::from_utf8_lossy(&o.stderr)
        )),
        Err(e) => log(&format!("  ✗ bun install error: {e}")),
    }

    // 4. @types/bun si usage Bun.* détecté et absent des deps
    let uses_bun_api = fixes.iter().any(|f| {
        f.file.ends_with(".ts") || f.file.ends_with(".tsx") || f.file.ends_with(".mts")
    }) && fixes.iter().any(|f| {
        f.after.contains("Bun.") || f.findings.iter().any(|x| x.rule_id.starts_with("api/"))
    });
    if uses_bun_api {
        if let Ok(pkg_content) = std::fs::read_to_string(&root_pkg) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&pkg_content) {
                let has_types = pkg
                    .get("devDependencies")
                    .and_then(|d| d.get("@types/bun"))
                    .is_some()
                    || pkg
                        .get("dependencies")
                        .and_then(|d| d.get("@types/bun"))
                        .is_some();
                if !has_types {
                    log("  → bun add -d @types/bun");
                    let out = Command::new("bun")
                        .args(["add", "-d", "@types/bun"])
                        .current_dir(root)
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .output();
                    match out {
                        Ok(o) if o.status.success() => log("  ✓ @types/bun ajouté"),
                        Ok(o) => log(&format!(
                            "  ✗ bun add exit={}\n{}",
                            o.status,
                            String::from_utf8_lossy(&o.stderr)
                        )),
                        Err(e) => log(&format!("  ✗ bun add error: {e}")),
                    }
                }
            }
        }
    }

    Ok(())
}

fn run_rules(report: Report) -> Result<ExitCode> {
    use colored::Colorize;
    use serde_json::json;
    // Liste statique des IDs de règles. Plus simple et stable qu'une
    // introspection runtime, et reflète ce que l'agent peut vraiment voir.
    let rules: &[(&str, &str)] = &[
        ("shebang/node", "shebang `node` → `bun`"),
        ("lock/rival", "lockfile concurrent (pnpm-lock / yarn.lock / package-lock)"),
        ("cli/npm-install", "npm install → bun install"),
        ("cli/npm-ci", "npm ci → bun install --frozen-lockfile"),
        ("cli/npm-run", "npm run → bun run"),
        ("cli/npx", "npx → bunx"),
        ("cli/pnpm-install", "pnpm install → bun install"),
        ("cli/pnpm-add", "pnpm add → bun add"),
        ("cli/pnpm-dlx", "pnpm dlx → bunx"),
        ("cli/yarn-add", "yarn add → bun add"),
        ("pkg/package-manager", "packageManager → bun@<version>"),
        ("pkg/engines-pm", "engines.{npm,pnpm,yarn} → engines.bun"),
        ("pkg/redundant-dep", "dépendance redondante avec API Bun native"),
        ("imports/node-prefix", "préfixer `fs` → `node:fs` etc."),
        ("imports/bun-native", "package npm → API Bun native (bun:sqlite, Bun.sql, etc.)"),
        ("api/fs-readFileSync", "fs.readFileSync(path, 'utf8') → await Bun.file(path).text()"),
        ("api/fs-writeFileSync", "fs.writeFileSync(path, data) → await Bun.write(path, data)"),
        ("api/fs-existsSync", "fs.existsSync(path) → await Bun.file(path).exists()"),
        ("api/fs-readFile-promise", "fsPromises.readFile → Bun.file(path).text()"),
        ("api/json-parse-readFileSync", "JSON.parse(fs.readFileSync) → Bun.file(path).json()"),
        ("api/dirname-esm", "__dirname ESM → import.meta.dir"),
        ("api/filename-esm", "__filename ESM → import.meta.path"),
        ("api/fileURLToPath", "fileURLToPath → Bun.fileURLToPath / import.meta.dir"),
        ("api/new-url-import-meta", "new URL(..., import.meta.url) → import.meta.dir"),
        ("api/path-join-dirname", "path.join(__dirname, ...) → path.join(import.meta.dir, ...)"),
        ("api/buffer-alloc", "Buffer.alloc(n) → new Uint8Array(n)"),
        ("api/buffer-concat", "Buffer.concat → concaténation Uint8Array"),
        ("api/buffer-from-string", "Buffer.from(str, 'utf8') → new TextEncoder().encode(str)"),
        ("api/buffer-from-base64", "Buffer.from(x, 'base64') → atob/btoa"),
        ("api/buffer-byteLength", "Buffer.byteLength → new TextEncoder().encode().length"),
        ("api/http-createServer", "http.createServer → Bun.serve()"),
        ("api/https-createServer", "https.createServer → Bun.serve({ tls })"),
        ("api/execSync", "child_process.execSync → shell Bun $`cmd`"),
        ("api/exec", "child_process.exec → Bun.spawn"),
        ("api/child-process-spawn", "spawn → Bun.spawn"),
        ("api/crypto-createHash", "crypto.createHash → Bun.hash / Bun.CryptoHasher"),
        ("api/util-inspect", "util.inspect → Bun.inspect"),
        ("api/util-promisify", "util.promisify (préférer APIs async natives)"),
        ("api/sleep-promise", "setTimeout Promise → Bun.sleep"),
        ("api/uuid-v4", "uuidv4() → crypto.randomUUID() / Bun.randomUUIDv7"),
        ("api/express-server", "express() → Bun.serve"),
        ("api/toml-parse", "TOML.parse → Bun.TOML.parse"),
        ("api/semver", "semver.* → Bun.semver.*"),
        ("api/require-resolve", "require.resolve → Bun.resolveSync"),
        ("api/set-immediate", "setImmediate → queueMicrotask / setTimeout(fn, 0)"),
        ("api/os-platform", "os.platform() → process.platform (info)"),
        ("api/os-homedir", "os.homedir() → Bun.env.HOME (info)"),
        ("api/process-env", "process.env.X → Bun.env.X (info, stylistique)"),
        ("api/process-stdout-write", "process.stdout.write → Bun.stdout.write"),
        ("api/process-stderr-write", "process.stderr.write → Bun.stderr.write"),
        ("api/performance-now", "performance.now → Bun.nanoseconds (info)"),
        ("ci/setup-node", "actions/setup-node → oven-sh/setup-bun@v2"),
        ("ci/node-version", "node-version → bun-version: latest"),
        ("docker/node-base", "FROM node:<tag> → FROM oven/bun:<tag>"),
        ("env/nvmrc", ".nvmrc / .node-version (info)"),
        ("tsconfig/bun-types", "compilerOptions.types : ajouter 'bun'"),
        ("tsconfig/module-resolution", "moduleResolution → bundler/nodenext"),
        ("workspace/pnpm-yaml", "pnpm-workspace.yaml → \"workspaces\" dans package.json racine"),
        ("workspace/only-built-deps", "onlyBuiltDependencies → trustedDependencies"),
        ("workspace/root-missing", "package.json racine sans \"workspaces\" alors que pnpm-workspace.yaml existe"),
        ("workspace/trusted-deps-missing", "trustedDependencies manquant (onlyBuiltDependencies de pnpm non porté)"),
        ("husky/pnpm-command", "hook husky 'pnpm X' → 'bun run X'"),
        ("husky/npm-command", "hook husky 'npm X' → 'bun run X'"),
        ("husky/yarn-command", "hook husky 'yarn X' → 'bun run X'"),
        ("husky/npx-command", "hook husky 'npx' → 'bunx --bun'"),
        ("husky/pnpm-dlx", "hook husky 'pnpm dlx' → 'bunx --bun'"),
        ("pkg/jest-script", "script 'jest' → 'bun test'"),
        ("pkg/tsup-bun-external", "script tsup + import('bun') → ajouter '--external bun'"),
        // --- Bun namespace APIs ---
        ("api/bcrypt-hash", "bcrypt.hash → Bun.password.hash"),
        ("api/bcrypt-compare", "bcrypt.compare → Bun.password.verify"),
        ("api/argon2-hash", "argon2.hash/verify → Bun.password"),
        ("api/yaml-parse", "yaml.load/parse → Bun.YAML.parse"),
        ("api/yaml-stringify", "yaml.dump/stringify → Bun.YAML.stringify"),
        ("api/json5-parse", "JSON5.parse → Bun.JSON5.parse"),
        ("api/json5-stringify", "JSON5.stringify → Bun.JSON5.stringify"),
        ("api/marked-call", "marked() → Bun.markdown.html"),
        ("api/marked-parse", "marked.parse → Bun.markdown.html"),
        ("api/escape-html", "escapeHtml / he.encode → Bun.escapeHTML"),
        ("api/strip-ansi", "stripAnsi → Bun.stripANSI"),
        ("api/string-width", "stringWidth → Bun.stringWidth"),
        ("api/slice-ansi", "sliceAnsi → Bun.sliceAnsi"),
        ("api/which-call", "which(pkg) → Bun.which(pkg)"),
        ("api/cron-schedule", "cron.schedule → Bun.cron"),
        ("api/cronjob-new", "new CronJob → Bun.cron"),
        ("api/fast-deep-equal", "fastDeepEqual → Bun.deepEquals"),
        ("api/pako-gzip", "pako.gzip/deflate → Bun.gzipSync / deflateSync"),
        ("api/pako-gunzip", "pako.ungzip/inflate → Bun.gunzipSync / inflateSync"),
        ("api/express-app", "express() → Bun.serve (info)"),
        ("api/fastify-app", "fastify() → Bun.serve (info)"),
        ("api/koa-new", "new Koa() → Bun.serve (info)"),
        ("api/http-request", "http.request → fetch"),
        ("api/https-request", "https.request → fetch"),
        ("api/crypto-randomBytes", "crypto.randomBytes → crypto.getRandomValues"),
        ("api/eventsource-new", "new EventSource → Bun.EventSource (déjà global, info)"),
        ("api/cookie-parse", "cookie.parse → new Bun.CookieMap"),
        ("api/cookie-serialize", "cookie.serialize → Bun.Cookie.toString"),
        ("api/aws-sdk-s3-client", "new S3Client → Bun.S3Client"),
        ("api/file-based-routing", "next-router → Bun.FileSystemRouter (info)"),
        ("api/chalk-call", "chalk.<color> → Bun.color / ANSI natif (info)"),
        ("api/zlib-gzipSync", "zlib.gzipSync → Bun.gzipSync (info)"),
        ("api/process-hrtime-bigint", "process.hrtime.bigint → Bun.nanoseconds (info)"),
        ("api/execa-call", "execa → Bun.$ / Bun.spawn"),
        // --- bunfig.toml ---
        ("bunfig/registry-npmjs", "registry npmjs par défaut (redondant)"),
        ("bunfig/option-note", "note sur une option bunfig (isolated, saveTextLockfile)"),
        ("bunfig/unknown-option", "option bunfig inconnue (legacy)"),
        // --- tsconfig étendu ---
        ("tsconfig/module-legacy", "module=CommonJS/AMD/UMD → ESNext/Preserve"),
        ("tsconfig/target-legacy", "target=ES2021 ou moins → ES2022+/ESNext"),
        ("tsconfig/module-detection", "moduleDetection absent → 'force'"),
        ("tsconfig/verbatim-module-syntax", "moduleResolution=bundler + verbatimModuleSyntax=true"),
        ("tsconfig/allow-ts-extensions", "Bun résout les .ts nativement → allowImportingTsExtensions"),
        ("tsconfig/no-emit", "bundler + noEmit=true (Bun émet, tsc type-check)"),
        ("tsconfig/duplicate-node-types", "types=['bun','node'] redondant — bun suffit"),
        // --- Next.js ---
        ("next/output-standalone", "next.config output:'standalone' (info, reste OK en Node)"),
        ("next/webpack-custom", "next.config webpack() custom (Turbopack est default Next 16)"),
        ("next/server-external-packages", "experimental.serverComponentsExternalPackages (auditer)"),
        ("next/turbopack-missing", "next.config : webpack custom sans turbopack: {}"),
        ("next/images-custom-loader", "next images.loader: 'custom'"),
        ("next/script-runtime", "script 'next dev/start' → préfixer par bunx --bun"),
        ("next/build-turbopack", "'next build' sans --turbopack (Next 16)"),
        ("next/custom-server-next-app", "next({ dev }) custom server → Bun.serve({ fetch })"),
        ("next/request-handler", "app.getRequestHandler() → wrapper pour Bun.serve"),
        // --- Ecosystem (info, guides) ---
        ("ecosystem/nextjs", "next détecté → guide Bun + Next.js"),
        ("ecosystem/nuxt", "nuxt détecté → guide Bun + Nuxt"),
        ("ecosystem/astro", "astro détecté → guide Bun + Astro"),
        ("ecosystem/remix", "@remix-run/react détecté → guide Bun + Remix"),
        ("ecosystem/sveltekit", "@sveltejs/kit détecté → guide Bun + SvelteKit"),
        ("ecosystem/tanstack-start", "@tanstack/start → guide Bun"),
        ("ecosystem/solid-start", "solid-start → guide Bun"),
        ("ecosystem/qwik", "@builder.io/qwik → guide Bun"),
        ("ecosystem/hono", "hono → guide Bun"),
        ("ecosystem/elysia", "elysia → guide Bun"),
        ("ecosystem/fastify", "fastify → guide Bun"),
        ("ecosystem/express", "express → guide Bun"),
        ("ecosystem/stric", "stric → guide Bun"),
        ("ecosystem/vite", "vite → guide Bun"),
        ("ecosystem/prisma", "prisma / @prisma/client → guide Bun"),
        ("ecosystem/drizzle", "drizzle-orm → guide Bun"),
        ("ecosystem/mongoose", "mongoose → guide Bun"),
        ("ecosystem/gel", "@edgedb/driver (Gel) → guide Bun"),
        ("ecosystem/pm2", "pm2 → guide daemon Bun"),
        ("ecosystem/sentry", "@sentry/node → guide Sentry + Bun"),
        ("ecosystem/discord-bot", "discord.js → guide Discord bot Bun"),
        // --- Top 10 awesome-bun packages ---
        ("ecosystem/graphql-yoga", "graphql-yoga → intégration Bun"),
        ("ecosystem/orama", "@orama/orama → search engine"),
        ("ecosystem/brisa", "brisa → full-stack framework"),
        ("ecosystem/kysely", "kysely → SQL query builder"),
        ("ecosystem/kysely-bun", "kysely-bun-sqlite → Kysely + bun:sqlite"),
        ("ecosystem/hattip", "@hattip/core → HTTP cross-runtime"),
        ("ecosystem/primate", "primate → web framework"),
        ("ecosystem/vixeny", "vixeny → functional web framework"),
        ("ecosystem/nbit", "nbit → zero-dep web framework"),
        ("ecosystem/bun-utilities", "bun-utilities → FS/shell helpers"),
        ("ecosystem/electrobun", "electrobun → desktop apps Bun+Zig"),
        ("ecosystem/electron-alt", "electron → envisager Electrobun (Bun+Zig)"),
        ("ecosystem/tauri", "tauri → compatible Bun frontend"),
        ("ecosystem/ink", "ink (React CLI) → tourne sous Bun"),
        ("ecosystem/bunli", "bunli → CLI framework Bun-native"),
        ("ecosystem/commander", "commander → compatible, ou Bunli"),
        ("ecosystem/yargs", "yargs → compatible, ou util.parseArgs/Bunli"),
        // --- .npmrc / .yarnrc / .pnpmrc → bunfig.toml ---
        ("npmrc/registry", ".npmrc registry → bunfig.toml [install].registry"),
        ("npmrc/auth-token", ".npmrc _authToken → bunfig.toml [install.scopes]"),
        ("npmrc/scoped-registry", "@scope:registry → bunfig.toml [install.scopes]"),
        ("npmrc/always-auth", "'always-auth' spécifique npm"),
        ("npmrc/save-prefix", "save-exact/save-prefix → bunfig.toml"),
        ("npmrc/node-linker", "node-linker → bunfig.toml [install].linker"),
        ("npmrc/engine-strict", "engine-strict (Bun lit engines.bun)"),
        ("npmrc/lockfile-flag", "options lockfile obsolètes avec bun.lock"),
    ];
    match report {
        Report::Json | Report::Jsonl => {
            let arr: Vec<_> = rules
                .iter()
                .map(|(id, desc)| {
                    json!({
                        "id": id,
                        "category": ai::category(id),
                        "description": desc,
                        "docs_url": ai::docs_url(id),
                        "confidence": ai::confidence(id, true),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
        }
        _ => {
            println!("{} {}", "n2b".bold(), format!("({} rules)", rules.len()).dimmed());
            let mut by_cat: std::collections::BTreeMap<&str, Vec<&(&str, &str)>> =
                Default::default();
            for r in rules {
                by_cat.entry(ai::category(r.0)).or_default().push(r);
            }
            for (cat, rs) in by_cat {
                println!("\n{}", cat.cyan().bold());
                for (id, desc) in rs {
                    println!("  {}  {}", id.yellow(), desc);
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn run_prompt(
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

fn run_audit(
    root: PathBuf,
    terms: Vec<String>,
    state: audit::ItemState,
    limit: usize,
    report: Report,
) -> Result<ExitCode> {
    let root = root.canonicalize().unwrap_or(root);
    let repo = audit::detect_repo(&root)?;
    let terms = if terms.is_empty() {
        vec!["bun".to_string(), "node".to_string()]
    } else {
        terms
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let result = rt.block_on(audit::run_audit(repo, &terms, state, limit))?;

    match report {
        Report::Json => println!("{}", audit::render_json(&result)),
        _ => print!("{}", audit::render_text(&result)),
    }
    Ok(ExitCode::SUCCESS)
}
