// shims — Bun-native implementations for the Node.js patterns n2b flags.
// Import by namespace (`import { env, fs, path, shell } from "@n2b/core/shims"`)
// or by scoped subpath (`import { readText } from "@n2b/core/shims/fs"`).

export * as env from "./env";
export * as fs from "./fs";
export * as path from "./path";
export * as shell from "./shell";

// Re-export EnvError for callers that want to instanceof-check.
export { EnvError } from "./env";
