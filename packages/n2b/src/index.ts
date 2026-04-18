// @n2b/core — TypeScript façade for the Rust binary `n2b`.
//
// Three surfaces:
//
//  1. `scan()`, `rules()`, `promptMarkdown()` — spawn `n2b --report=json`
//     and return typed results. Use these when you want to invoke n2b
//     from automation code (CI, scripts, custom tools).
//
//  2. `n2bPlugin()` — Bun.plugin() integration. Runs a scan at build
//     start and surfaces per-file findings during module loading.
//
//  3. `shims/*` — Bun-native wrappers for Node patterns that n2b flags
//     most often (env access, __dirname equivalents, sync fs reads via
//     Bun.file, shell commands). These are drop-in replacements that
//     callers can adopt mechanically.
//
// The Rust-side types (schema/v2.json) are re-exported from `./schema`
// so consumers get the exact same shape n2b emits.

export { scan, rules, promptMarkdown, binaryVersion } from "./cli";
export type { ScanOptions, N2BError } from "./cli";

export { n2bPlugin } from "./plugin";
export type { N2BPluginOptions } from "./plugin";

export { computeLineOffsets, posFromOffsets, nativeAvailable } from "./ffi";

export type { N2BReport, FileFix, Finding, Context, Mode, Severity } from "./schema";
