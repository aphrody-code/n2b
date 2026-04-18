/// Applique les side-effects du mode `--migrate` :
///  1. Migre pnpm-workspace.yaml → `workspaces` + `trustedDependencies` dans package.json
///  2. Retire pnpm-lock.yaml / yarn.lock / package-lock.json
///  3. Exécute `bun install` pour reconstruire bun.lock
///  4. Ajoute `@types/bun` en devDep si source utilise `Bun.*`
///
/// Utilise `BackupGuard` pour un rollback transactionnel : si `bun install`
/// échoue, les fichiers modifiés sont restaurés depuis leurs `.n2b-bak`.
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::subprocess::bun::{self, BackupGuard};
use crate::types::FileFix;

pub fn run_migrate_side_effects(
    root: &PathBuf,
    fixes: &[FileFix],
    quiet: bool,
) -> Result<()> {
    let log = |msg: &str| {
        if !quiet {
            eprintln!("[migrate] {msg}");
        }
    };

    let pnpm_ws = root.join("pnpm-workspace.yaml");
    let root_pkg = root.join("package.json");

    // Collect rival lockfiles that exist.
    let rivals: Vec<PathBuf> = crate::scanners::lockfile::RIVAL_LOCKFILES
        .iter()
        .map(|name| root.join(name))
        .filter(|p| p.exists())
        .collect();

    // --- BackupGuard setup ---
    let mut guard = BackupGuard::new();
    guard.backup(&root_pkg)?;
    guard.backup(&pnpm_ws)?;
    for rival in &rivals {
        guard.backup(rival)?;
    }

    // 1. pnpm-workspace.yaml → package.json
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
    for rival in &rivals {
        let name = rival.file_name().unwrap_or_default().to_string_lossy();
        std::fs::remove_file(rival)?;
        log(&format!("  - {name} supprimé"));
    }

    // 3. bun install (reconstruit bun.lock)
    log("  → bun install");
    match bun::install(root) {
        Ok(()) => {
            log("  ✓ bun install OK");
        }
        Err(e) => {
            guard.restore_all();
            return Err(e).context("bun install a échoué ; fichiers restaurés depuis .n2b-bak");
        }
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
                    if let Err(e) = bun::add_dev(root, "@types/bun") {
                        log(&format!("  ✗ bun add -d @types/bun: {e}"));
                    } else {
                        log("  ✓ @types/bun ajouté");
                    }
                }
            }
        }
    }

    guard.commit()?;
    Ok(())
}
