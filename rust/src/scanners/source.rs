use crate::rules::{bun_apis::apply_bun_api_rules, node_imports::apply_node_import_rules};
use crate::scanners::shebang::scan_shebang;
use crate::types::{Finding, Mode, RunOptions};

pub fn scan_source(path: &str, content: &str, opts: &RunOptions) -> (Vec<Finding>, String) {
    let mut all: Vec<Finding> = Vec::new();
    let aggressive = opts.mode == Mode::Aggressive;

    let (f, working) = scan_shebang(path, content);
    all.extend(f);
    let (f, working) = apply_node_import_rules(path, &working, aggressive);
    all.extend(f);
    let (f, working) = apply_bun_api_rules(path, &working, aggressive);
    all.extend(f);

    if opts.mode == Mode::Check {
        return (all, content.to_string());
    }
    (all, working)
}
