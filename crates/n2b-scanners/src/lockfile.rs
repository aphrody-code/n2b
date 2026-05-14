use n2b_types::types::{Finding, MakeFindingOpts, Severity};
use n2b_util::make_finding;

pub const RIVAL_LOCKFILES: &[&str] = &[
    "package-lock.json",
    "npm-shrinkwrap.json",
    "pnpm-lock.yaml",
    "yarn.lock",
];

pub fn check_lockfile(path: &str, name: &str) -> Option<Finding> {
    if !RIVAL_LOCKFILES.contains(&name) {
        return None;
    }
    Some(make_finding(
        path,
        &[],
        0,
        "lock/rival",
        format!(
            "lockfile concurrent '{name}' présent — exécuter 'bun install' puis supprimer ce fichier"
        ),
        name.to_string(),
        None,
        MakeFindingOpts {
            autofix: Some(false),
            severity: Some(Severity::Warn),
            ..Default::default()
        },
    ))
}
