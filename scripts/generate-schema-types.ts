#!/usr/bin/env bun
// generate-schema-types.ts — single entry point to regenerate Rust and
// TypeScript types from `schema/v2.json`. `schema/v2.json` is the source
// of truth for the CLI JSON output.
//
// Modes:
//   bun run scripts/generate-schema-types.ts            # write
//   bun run scripts/generate-schema-types.ts --check    # fail on drift
//
// Toolchain:
//   Rust: cargo-typify (cargo install cargo-typify).
//   TypeScript: json-schema-to-typescript (bunx, no install needed).
//
// Output paths:
//   Rust: crates/n2b-core/src/schema.rs
//   TS:   packages/n2b/src/schema.ts

import { $ } from "bun";
import { join, resolve } from "node:path";

const REPO = resolve(import.meta.dir, "..");
const SCHEMA = join(REPO, "schema", "v2.json");
const RUST_OUT = join(REPO, "crates", "n2b-core", "src", "schema.rs");
const TS_OUT = join(REPO, "packages", "n2b", "src", "schema.ts");

const HEADER_RUST = `// @generated — DO NOT EDIT MANUALLY.
// Regenerate with: bun run scripts/generate-schema-types.ts
// Source of truth: schema/v2.json
`;

const HEADER_TS = `// @generated — DO NOT EDIT MANUALLY.
// Regenerate with: bun run scripts/generate-schema-types.ts
// Source of truth: schema/v2.json
`;

const checkMode = process.argv.includes("--check");

async function genRust(): Promise<string> {
  const tmp = `/tmp/n2b-schema-${process.pid}.rs`;
  await $`cargo typify ${SCHEMA} -o ${tmp}`.quiet();
  const body = await Bun.file(tmp).text();
  await $`rm -f ${tmp}`.quiet();
  return HEADER_RUST + "\n" + body;
}

async function genTs(): Promise<string> {
  const proc = Bun.spawn(
    ["bunx", "--bun", "json-schema-to-typescript", SCHEMA, "--unreachableDefinitions"],
    { stdout: "pipe", stderr: "pipe" },
  );
  const out = await new Response(proc.stdout).text();
  const err = await new Response(proc.stderr).text();
  const code = await proc.exited;
  if (code !== 0) throw new Error(`json-schema-to-typescript failed: ${err}`);
  // Strip the generator's own header; we add our own.
  const body = out.replace(/^\/\* eslint-disable \*\/\n\/\*\*[\s\S]*?\*\/\n\n/, "");
  return HEADER_TS + "\n" + body;
}

async function writeOrCheck(path: string, content: string, label: string): Promise<boolean> {
  const existing = await Bun.file(path).exists() ? await Bun.file(path).text() : "";
  if (checkMode) {
    if (existing !== content) {
      console.error(`✗ ${label} drift detected: ${path}`);
      console.error(`  run \`bun run scripts/generate-schema-types.ts\` to regenerate`);
      return false;
    }
    console.log(`✓ ${label} up to date`);
    return true;
  }
  if (existing === content) {
    console.log(`= ${label} unchanged`);
    return true;
  }
  await Bun.write(path, content);
  console.log(`✓ ${label} written (${content.length} bytes)`);
  return true;
}

const [rustCode, tsCode] = await Promise.all([genRust(), genTs()]);
const rustOk = await writeOrCheck(RUST_OUT, rustCode, "crates/n2b-core/src/schema.rs");
const tsOk = await writeOrCheck(TS_OUT, tsCode, "packages/n2b/src/schema.ts");

if (!rustOk || !tsOk) process.exit(1);
