use n2b_rules::cli_commands::apply_cli_rules;
use n2b_types::types::Finding;

pub fn scan_shell(path: &str, content: &str) -> (Vec<Finding>, String) {
    apply_cli_rules(path, content)
}
