// contract.rs — CLI-as-API contract tests.
//
// This file rejoue the exact invocations used by consumers of n2b
// (primarily /home/ubuntu/rpb-dashboard — see
// tests/rpb-dashboard-baseline/INVOCATIONS.md) and verifies two things:
//
//  1. The binary's JSON output deserializes cleanly into the generated
//     n2b_core::schema::N2bReport type.
//  2. The JSON output conforms to schema/v2.json via the jsonschema crate.
//
// These tests are the canonical verrou against regression: if either
// fails, rpb-dashboard (and any other consumer parsing JSON v2) breaks.

use assert_cmd::Command;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // $CARGO_MANIFEST_DIR points at crates/n2b-cli/. Two parents up → repo root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR parent")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn fixture() -> PathBuf {
    repo_root().join("test").join("fixture")
}

fn schema_path() -> PathBuf {
    repo_root().join("schema").join("v2.json")
}

fn run_n2b(args: &[&str]) -> (String, String, i32) {
    let output = Command::cargo_bin("n2b")
        .expect("n2b binary must be buildable")
        .args(args)
        .output()
        .expect("n2b invocation must succeed");
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

// ─── Shape-level assertions ─────────────────────────────────────────

#[test]
fn json_report_deserializes_into_typed_schema() {
    let fixture = fixture().display().to_string();
    let (stdout, stderr, code) = run_n2b(&[&fixture, "--report=json"]);
    // Exit 0 (no findings) or 1 (findings in check mode) both acceptable.
    assert!(code == 0 || code == 1, "n2b exit {code}, stderr:\n{stderr}");
    let _report: n2b_core::schema::N2bReport = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("JSON did not match schema/v2.json types: {e}\nstdout was: {stdout}")
    });
}

#[test]
fn json_report_validates_against_schema_v2() {
    let fixture = fixture().display().to_string();
    let (stdout, _stderr, _code) = run_n2b(&[&fixture, "--report=json"]);
    let schema_text = std::fs::read_to_string(schema_path()).expect("schema/v2.json readable");
    let schema_json: serde_json::Value =
        serde_json::from_str(&schema_text).expect("schema/v2.json is valid JSON");
    let instance: serde_json::Value =
        serde_json::from_str(&stdout).expect("n2b stdout is valid JSON");
    let validator = jsonschema::validator_for(&schema_json).expect("schema compiles");
    let errors: Vec<_> = validator.iter_errors(&instance).collect();
    assert!(
        errors.is_empty(),
        "schema validation failed: {} error(s)\nfirst: {}",
        errors.len(),
        errors.first().map(|e| e.to_string()).unwrap_or_default()
    );
}

// ─── Invocation contracts (rpb-dashboard compat) ────────────────────

#[test]
fn rules_text_format_succeeds() {
    let (stdout, stderr, code) = run_n2b(&["rules"]);
    assert_eq!(code, 0, "n2b rules exit {code}, stderr:\n{stderr}");
    assert!(!stdout.trim().is_empty());
    // rpb-dashboard expects at least the rule_id column.
    assert!(stdout.contains("rule_id") || stdout.contains("cli/") || stdout.contains("api/"));
}

#[test]
fn rules_json_format_is_array() {
    let (stdout, stderr, code) = run_n2b(&["rules", "--report=json"]);
    assert_eq!(
        code, 0,
        "n2b rules --report=json exit {code}, stderr:\n{stderr}"
    );
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON output");
    assert!(v.is_array() || v.get("rules").is_some());
}

#[test]
fn jsonl_format_starts_with_meta_type() {
    let fixture = fixture().display().to_string();
    let (stdout, _stderr, _code) = run_n2b(&[&fixture, "--report=jsonl"]);
    let first_line = stdout.lines().next().expect("at least one line");
    let v: serde_json::Value = serde_json::from_str(first_line).expect("JSONL first line is JSON");
    assert_eq!(v.get("type").and_then(|t| t.as_str()), Some("meta"));
}

#[test]
fn sarif_format_has_version_2_1() {
    let fixture = fixture().display().to_string();
    let (stdout, _stderr, _code) = run_n2b(&[&fixture, "--report=sarif"]);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("SARIF is JSON");
    assert_eq!(v.get("version").and_then(|x| x.as_str()), Some("2.1.0"));
}

#[test]
fn markdown_format_has_heading() {
    let fixture = fixture().display().to_string();
    let (stdout, _stderr, _code) = run_n2b(&[&fixture, "--report=md"]);
    assert!(stdout.contains("# "), "markdown must start with a heading");
}

#[test]
fn md_and_markdown_are_equivalent() {
    let fixture = fixture().display().to_string();
    let (a, _, _) = run_n2b(&[&fixture, "--report=md"]);
    let (b, _, _) = run_n2b(&[&fixture, "--report=markdown"]);
    assert_eq!(a, b, "--report=md and --report=markdown must be equivalent");
}

#[test]
fn exit_code_2_on_invalid_flag() {
    let (_stdout, _stderr, code) = run_n2b(&["--definitely-not-a-flag"]);
    assert_eq!(code, 2, "invalid flag must exit 2");
}
