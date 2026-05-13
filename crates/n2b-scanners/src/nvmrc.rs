use n2b_types::types::{Finding, MakeFindingOpts, Severity};
use n2b_util::make_finding;

/// `.nvmrc` / `.node-version` : advisory. Bun n'utilise pas ces fichiers.
pub fn scan_nvmrc(path: &str, content: &str) -> (Vec<Finding>, String) {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return (Vec::new(), content.to_string());
    }
    let finding = make_finding(
        path,
        &[],
        0,
        "env/nvmrc",
        format!(
            "`.nvmrc`/`.node-version` ({trimmed}) n'est pas utilisé par Bun — tu peux le conserver pour les devs qui restent sur Node, ou le supprimer si l'équipe migre complètement"
        ),
        trimmed.to_string(),
        None,
        MakeFindingOpts {
            autofix: Some(false),
            severity: Some(Severity::Info),
            ..Default::default()
        },
    );
    (vec![finding], content.to_string())
}
