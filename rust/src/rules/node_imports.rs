use crate::rules::imports_ast;
use crate::types::{Finding, MakeFindingOpts};
use crate::util::{line_offsets, make_finding};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

static BUILTINS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    // Liste dérivée de `bun/src/js/node` + builtins Node canoniques.
    // Inclut les sous-chemins /promises, /strict, /web, /posix, /win32
    // que Bun shimme (voir `bun/src/js/node/*.ts`).
    [
        // Top-level
        "assert", "async_hooks", "buffer", "child_process", "cluster", "console",
        "crypto", "dgram", "diagnostics_channel", "dns", "domain", "events", "fs",
        "http", "http2", "https", "inspector", "module", "net", "os", "path",
        "perf_hooks", "process", "punycode", "querystring", "readline", "repl",
        "stream", "string_decoder", "sys", "test", "timers", "tls", "trace_events",
        "tty", "url", "util", "v8", "vm", "wasi", "worker_threads", "zlib",
        // Sub-paths shimmés par Bun
        "assert/strict",
        "dns/promises",
        "fs/promises",
        "inspector/promises",
        "path/posix",
        "path/win32",
        "readline/promises",
        "stream/consumers",
        "stream/promises",
        "stream/web",
        "timers/promises",
    ]
    .into_iter()
    .collect()
});

struct BunReplacement {
    replacement: &'static str,
    note: &'static str,
    aggressive: bool,
}

fn repl(replacement: &'static str, note: &'static str, aggressive: bool) -> BunReplacement {
    BunReplacement { replacement, note, aggressive }
}

static BUN_REPLACEMENTS: Lazy<HashMap<&'static str, BunReplacement>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("sqlite3",         repl("bun:sqlite",           "préférer bun:sqlite (natif, synchrone, rapide)", true));
    m.insert("better-sqlite3",  repl("bun:sqlite",           "préférer bun:sqlite (API similaire)", true));
    m.insert("node-fetch",      repl("<global fetch>",       "fetch est global dans Bun — supprimer l'import", true));
    m.insert("isomorphic-fetch",repl("<global fetch>",       "fetch est global dans Bun — supprimer l'import", true));
    m.insert("cross-fetch",     repl("<global fetch>",       "fetch est global dans Bun — supprimer l'import", true));
    m.insert("dotenv",          repl("<auto>",               "Bun charge .env automatiquement, dotenv inutile", true));
    m.insert("dotenv/config",   repl("<auto>",               "Bun charge .env automatiquement, dotenv inutile", true));
    m.insert("pg",              repl("Bun.sql",              "Bun.sql est un client PostgreSQL natif", true));
    m.insert("postgres",        repl("Bun.sql",              "Bun.sql est un client PostgreSQL natif", true));
    m.insert("ioredis",         repl("Bun.redis",            "Bun.redis est un client Redis/Valkey natif", true));
    m.insert("redis",           repl("Bun.redis",            "Bun.redis est un client Redis/Valkey natif", true));
    m.insert("ws",              repl("WebSocket",            "WebSocket client/serveur est natif dans Bun", true));
    m.insert("uuid",            repl("Bun.randomUUIDv7",     "utiliser Bun.randomUUIDv7() ou crypto.randomUUID()", true));
    m.insert("minimist",        repl("util.parseArgs",       "Node fournit util.parseArgs", false));
    m.insert("chalk",           repl("<natif>",              "Bun supporte les codes ANSI nativement", false));
    m.insert("rimraf",          repl("fs.rm",                "fs.rm({ recursive: true }) suffit", false));
    m.insert("mkdirp",          repl("fs.mkdir",             "fs.mkdir({ recursive: true }) suffit", false));
    m.insert("node-cron",       repl("Bun.cron",             "Bun.cron (API native)", true));
    m.insert("axios",           repl("<global fetch>",       "fetch est global dans Bun — pas besoin d'axios", true));
    m.insert("got",             repl("<global fetch>",       "fetch est global dans Bun — pas besoin de got", true));
    m.insert("node-got",        repl("<global fetch>",       "fetch est global dans Bun — pas besoin de node-got", true));
    m.insert("superagent",      repl("<global fetch>",       "fetch est global dans Bun — pas besoin de superagent", true));
    m.insert("undici",          repl("<global fetch>",       "fetch/WebSocket sont natifs dans Bun — undici inutile", true));
    m.insert("make-fetch-happen", repl("<global fetch>",     "fetch est global dans Bun", true));
    m.insert("ky",              repl("<global fetch>",       "fetch est global dans Bun — ky est inutile", false));
    m.insert("bcrypt",          repl("Bun.password",         "Bun.password.hash/verify supporte bcrypt et argon2", true));
    m.insert("bcryptjs",        repl("Bun.password",         "Bun.password.hash/verify supporte bcrypt et argon2", true));
    m.insert("argon2",          repl("Bun.password",         "Bun.password.hash({ algorithm: 'argon2id' })", true));
    m.insert("form-data",       repl("FormData",             "FormData est global dans Bun — supprimer l'import", true));
    m.insert("abort-controller",repl("AbortController",      "AbortController est global dans Bun — supprimer l'import", true));
    m.insert("whatwg-url",      repl("URL",                  "URL/URLSearchParams sont globaux dans Bun", true));
    m.insert("node-blob",       repl("Blob",                 "Blob est global dans Bun", true));
    m.insert("web-streams-polyfill", repl("<global>",        "ReadableStream/WritableStream sont globaux dans Bun", true));
    m.insert("jest",            repl("bun:test",             "bun:test est compatible Jest (describe/test/expect/mock)", true));
    m.insert("mocha",           repl("bun:test",             "bun:test remplace mocha avec une API similaire", true));
    m.insert("vitest",          repl("bun:test",             "bun:test offre les mêmes fonctionnalités que vitest", true));
    m.insert("chai",            repl("bun:test",             "bun:test inclut expect() avec assertions Jest/Chai", false));
    m.insert("@jest/globals",   repl("bun:test",             "importer depuis bun:test à la place", true));
    m.insert("ts-jest",         repl("bun:test",             "bun:test supporte TypeScript nativement", true));
    m.insert("jest-circus",     repl("bun:test",             "bun:test est le runner natif Bun", true));
    m.insert("ts-node",         repl("bun run",              "bun exécute TypeScript nativement sans ts-node", true));
    m.insert("tsx",             repl("bun run",              "bun exécute TSX nativement — tsx CLI inutile", true));
    m.insert("esm",             repl("<natif>",              "Bun supporte ESM nativement", true));
    m.insert("nodemon",         repl("bun --watch",          "bun --watch ou bun --hot remplacent nodemon", true));
    m.insert("concurrently",    repl("<natif>",              "Bun.spawn permet de lancer des processus en parallèle", false));
    m.insert("@types/node",     repl("@types/bun",           "utiliser @types/bun pour les types Bun (ou bun-types)", false));
    m.insert("@iarna/toml",     repl("Bun.TOML",             "Bun.TOML.parse() est natif", true));
    m.insert("toml",            repl("Bun.TOML",             "Bun.TOML.parse() est natif", true));
    m.insert("smol-toml",       repl("Bun.TOML",             "Bun.TOML.parse() est natif", true));
    m.insert("dns-packet",      repl("bun dns",              "Bun expose dns.lookup/dns.resolve nativement", false));
    m.insert("mime",            repl("<natif>",              "Bun.file().type retourne le MIME type automatiquement", false));
    m.insert("mime-types",      repl("<natif>",              "Bun.file().type retourne le MIME type automatiquement", false));
    m.insert("glob",            repl("bun (Glob)",           "import { Glob } from 'bun' est natif (pas bun:glob)", true));
    m.insert("fast-glob",       repl("bun (Glob)",           "import { Glob } from 'bun' est natif (pas bun:glob)", true));
    m.insert("glob-parent",     repl("bun (Glob)",           "utiliser import { Glob } from 'bun'", false));
    m
});

pub fn apply_node_import_rules(
    path: &str,
    source: &str,
    aggressive: bool,
) -> (Vec<Finding>, String) {
    let offsets = line_offsets(source);
    let mut findings: Vec<Finding> = Vec::new();
    struct Edit { index: usize, len: usize, replacement: String }
    let mut edits: Vec<Edit> = Vec::new();

    let mut seen_builtin_finding: HashSet<String> = HashSet::new();
    let mut seen_repl_finding: HashSet<String> = HashSet::new();

    let specifiers = imports_ast::extract_specifiers(path, source);

    for s in &specifiers {
        let pos = s.inner_start as usize;
        let len = s.inner_len as usize;
        let spec = &s.value;

        if BUILTINS.contains(spec.as_str()) {
            edits.push(Edit {
                index: pos,
                len,
                replacement: format!("node:{spec}"),
            });
            if seen_builtin_finding.insert(spec.clone()) {
                findings.push(make_finding(
                    path,
                    &offsets,
                    pos,
                    "imports/node-prefix",
                    format!("préfixer '{spec}' avec 'node:' (recommandé)"),
                    spec.clone(),
                    Some(format!("node:{spec}")),
                    MakeFindingOpts { autofix: Some(true), ..Default::default() },
                ));
            }
            continue;
        }

        if let Some(r) = BUN_REPLACEMENTS.get(spec.as_str()) {
            if seen_repl_finding.insert(spec.clone()) {
                findings.push(make_finding(
                    path,
                    &offsets,
                    pos,
                    "imports/bun-native",
                    format!(
                        "remplacer '{}' par {} — {}",
                        spec, r.replacement, r.note
                    ),
                    spec.clone(),
                    Some(r.replacement.to_string()),
                    MakeFindingOpts {
                        autofix: Some(false),
                        aggressive: Some(true),
                        ..Default::default()
                    },
                ));
            }
            if aggressive
                && r.aggressive
                && (r.replacement.starts_with("bun:") || r.replacement.starts_with("node:"))
            {
                edits.push(Edit {
                    index: pos,
                    len,
                    replacement: r.replacement.to_string(),
                });
            }
        }
    }

    // Apply mode: mode != check handled by caller via aggressive; we always
    // apply node: prefix edits here because that's also a safe fix in --fix mode.
    let mut out = source.to_string();
    if !edits.is_empty() {
        edits.sort_by(|a, b| b.index.cmp(&a.index));
        for e in edits {
            out.replace_range(e.index..e.index + e.len, &e.replacement);
        }
    }
    (findings, out)
}
