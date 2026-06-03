// Copyright 2026 aphrody-code
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

// start.ts — runnable self-check entrypoint for @aphrody/n2b-core.
// Invoked by the root `check-self` script: `bun run --filter @aphrody/n2b-core start .`
// Runs a dry-run scan via the thin Rust-binary façade and prints a JSON summary.

import { scan } from "./cli";

if (import.meta.main) {
  const root = Bun.argv[2] ?? ".";
  const report = await scan(root, { mode: "check", quiet: true });
  console.log(
    JSON.stringify(
      {
        root: report.root,
        files_scanned: report.files_scanned,
        findings_total: report.findings_total,
      },
      null,
      2,
    ),
  );
}
