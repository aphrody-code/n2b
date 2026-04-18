use n2b_core::scanners::source::scan_source;
use n2b_core::types::{Mode, Report, RunOptions};
use proptest::prelude::*;
use std::path::PathBuf;

fn opts() -> RunOptions {
    RunOptions {
        root: PathBuf::from("."),
        mode: Mode::Check,
        report: Report::Text,
        quiet: true,
        ignore: Vec::new(),
        agent: false,
        dry_run: false,
    }
}

proptest! {
    #[test]
    fn scanner_never_panics_on_arbitrary_source(s in "[\\x20-\\x7E\\n\\t]{0,4096}") {
        // The scanner must either produce findings or return cleanly.
        // It must never panic on arbitrary printable ASCII + whitespace input.
        let _ = scan_source("test.ts", &s, &opts());
    }
}
