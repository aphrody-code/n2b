// @n2b/core — TypeScript façade thin pour le binaire Rust `n2b`.
//
// Architecture Turborepo-style — packages frères :
//   - @n2b/core    : wrapper CLI (scan, rules, promptMarkdown).
//   - @n2b/types   : types TS auto-générés depuis schema/v2.json.
//   - @n2b/plugin  : Bun.plugin() + bindings FFI — importe @n2b/core et @n2b/types.
//   - @n2b/shims   : polyfills Bun-natifs des APIs Node détectées par n2b.

export { scan, rules, promptMarkdown, binaryVersion } from "./cli";
export type { ScanOptions, N2BError } from "./cli";

export type { N2BReport, FileFix, Finding, Context, Mode, Severity } from "@n2b/types";
