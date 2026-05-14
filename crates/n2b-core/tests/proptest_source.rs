// Copyright 2026 Yohan Pierre
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
