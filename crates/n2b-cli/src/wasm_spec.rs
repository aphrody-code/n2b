//! `n2b wasm spec` — référence WebAssembly spec officielle.
//!
//! Trois sous-commandes :
//!   - `testsuite` : scanne les fichiers `.wast` du repo WebAssembly/spec
//!     et extrait/compte les modules inline, les passe à `wat2wasm` si dispo.
//!   - `features`  : analyse un binaire `.wasm` et liste les propositions utilisées.
//!   - `opcodes`   : affiche la table d'opcodes statique, filtrée par proposition.
//!
//! Références :
//!   - <https://webassembly.github.io/spec/>
//!   - <https://github.com/WebAssembly/spec/tree/main/interpreter/binary/decode.ml>
//!   - <https://github.com/WebAssembly/spec/tree/main/document/core/appendix/index-instructions.rst>

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Options pour `n2b wasm spec testsuite`.
pub struct TestsuiteOpts {
    /// Chemin vers la racine du clone WebAssembly/spec (contient `test/core/`).
    pub path: PathBuf,
    /// Filtre facultatif : `core`, `simd`, `gc`, `threads`, `bulk-memory`, etc.
    pub filter: Option<String>,
    /// Runtime : `bun` ou `wasmtime` — réservé V2 (actuellement ignored).
    #[allow(dead_code)]
    pub runtime: String,
    /// Timeout par `.wast` en secondes — réservé V2 (actuellement ignored).
    #[allow(dead_code)]
    pub timeout_secs: u64,
    pub quiet: bool,
}

/// Options pour `n2b wasm spec features`.
pub struct FeaturesOpts {
    /// Chemin du binaire `.wasm` à analyser.
    pub path: PathBuf,
    pub quiet: bool,
}

/// Options pour `n2b wasm spec opcodes`.
pub struct OpcodesOpts {
    /// Filtre par proposition (ex. `mvp`, `bulk-memory`, `simd`, …).
    /// `None` = affiche tous.
    pub proposal: Option<String>,
    /// Format de sortie : `text` | `md` | `json`.
    pub report: String,
    pub quiet: bool,
}

/// Point d'entrée principal appelé depuis `wasm_cmd::run`.
pub fn run_spec(cmd: crate::wasm_cmd::WasmSpecCmd, quiet: bool) -> anyhow::Result<()> {
    match cmd {
        crate::wasm_cmd::WasmSpecCmd::Testsuite { path, filter, runtime, timeout_secs } => {
            run_testsuite(&TestsuiteOpts { path, filter, runtime, timeout_secs, quiet })
        }
        crate::wasm_cmd::WasmSpecCmd::Features { path } => {
            run_features(&FeaturesOpts { path, quiet })
        }
        crate::wasm_cmd::WasmSpecCmd::Opcodes { proposal, report } => {
            run_opcodes(&OpcodesOpts { proposal, report, quiet })
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 1 — testsuite
// ---------------------------------------------------------------------------

/// Résultat d'analyse d'un fichier `.wast`.
struct WastResult {
    modules_found: usize,
    modules_passed: usize,
    modules_failed: usize,
    skipped: bool,
}

/// Lance la testsuite WebAssembly.
///
/// Stratégie :
/// 1. Collecte les fichiers `.wast` selon le filtre demandé.
/// 2. Pour chaque `.wast`, extrait les blocs `(module …)` top-level avec un
///    parseur de parenthèses minimal (pas de WAST complet).
/// 3. Si `wat2wasm` est disponible, compile chaque module extrait et le valide
///    via `new WebAssembly.Module(bytes)` sous Bun.
/// 4. Sinon, compte seulement les modules (mode "count-only").
pub fn run_testsuite(opts: &TestsuiteOpts) -> Result<()> {
    let spec_root = &opts.path;
    let core_dir = spec_root.join("test/core");

    if !core_dir.exists() {
        anyhow::bail!(
            "Répertoire `test/core` introuvable dans `{}`.\n\
             Cloner le repo WebAssembly/spec :\n\
             \n  git clone --depth 1 https://github.com/WebAssembly/spec {}\n",
            spec_root.display(),
            spec_root.display()
        );
    }

    let wast_files = collect_wast_files(&core_dir, opts.filter.as_deref())?;
    if wast_files.is_empty() {
        if !opts.quiet {
            eprintln!("[wasm spec testsuite] Aucun fichier .wast trouvé (filtre: {:?})", opts.filter);
        }
        return Ok(());
    }

    let has_wat2wasm = which("wat2wasm");
    let has_bun = which("bun");

    if !opts.quiet {
        let mode = match (has_wat2wasm.is_some(), has_bun.is_some()) {
            (true, true) => "validation Bun (wat2wasm + WebAssembly.Module)",
            (true, false) => "compile-only (wat2wasm, bun absent)",
            _ => "count-only (wat2wasm absent — installer wabt)",
        };
        eprintln!(
            "[wasm spec testsuite] {} fichiers .wast — mode: {}",
            wast_files.len(),
            mode
        );
    }

    let mut total_modules = 0usize;
    let mut total_passed = 0usize;
    let mut total_failed = 0usize;
    let mut total_skipped = 0usize;

    for wast_path in &wast_files {
        let result = process_wast_file(wast_path, has_wat2wasm.as_deref(), has_bun.as_deref())?;
        total_modules += result.modules_found;
        total_passed += result.modules_passed;
        total_failed += result.modules_failed;
        if result.skipped {
            total_skipped += 1;
        }

        if !opts.quiet {
            let rel = wast_path
                .strip_prefix(&core_dir)
                .unwrap_or(wast_path.as_path());
            if result.skipped {
                println!("  {} {}", "SKIP".yellow(), rel.display());
            } else {
                let status = if result.modules_failed == 0 {
                    "OK".green()
                } else {
                    "FAIL".red()
                };
                println!(
                    "  {} {} ({} modules, {} passed, {} failed)",
                    status,
                    rel.display(),
                    result.modules_found,
                    result.modules_passed,
                    result.modules_failed
                );
            }
        }
    }

    // Résumé final
    if !opts.quiet {
        println!();
        println!(
            "Testsuite WebAssembly — {} fichiers .wast",
            wast_files.len()
        );
        println!("  modules trouvés  : {}", total_modules);
        if has_wat2wasm.is_some() {
            println!("  passed           : {}", total_passed.to_string().green());
            println!(
                "  failed           : {}",
                if total_failed == 0 {
                    total_failed.to_string().green()
                } else {
                    total_failed.to_string().red()
                }
            );
        }
        println!("  skipped (assert) : {}", total_skipped);
        if has_wat2wasm.is_none() {
            println!();
            println!(
                "  {} Pour l'exécution complète, installer wabt : apt install wabt",
                "TODO:".yellow()
            );
            println!("  {} Support runtime Bun natif (sans wat2wasm) prévu en V2.", "TODO:".yellow());
        }
    }

    Ok(())
}

/// Collecte les fichiers `.wast` selon le filtre.
fn collect_wast_files(core_dir: &Path, filter: Option<&str>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    match filter {
        // Sous-répertoire de proposition
        Some(f) if !f.eq_ignore_ascii_case("core") => {
            let sub = core_dir.join(f);
            if sub.is_dir() {
                collect_wast_recursive(&sub, &mut files)?;
            } else {
                anyhow::bail!(
                    "Sous-répertoire `{}` introuvable dans `{}`",
                    f,
                    core_dir.display()
                );
            }
        }
        // Core ou pas de filtre : fichiers à la racine de test/core/
        _ => {
            let entries = std::fs::read_dir(core_dir)
                .with_context(|| format!("lecture de {}", core_dir.display()))?;
            for entry in entries.flatten() {
                let path = entry.path();
                let is_wast = path.extension().and_then(|e| e.to_str()) == Some("wast");
                let is_dir = path.is_dir();
                if is_wast {
                    files.push(path);
                } else if filter.is_none() && is_dir {
                    // Inclut aussi les sous-répertoires quand pas de filtre
                    collect_wast_recursive(&path, &mut files)?;
                }
            }
            files.sort();
        }
    }

    Ok(files)
}

/// Collecte récursivement les `.wast` dans un répertoire.
fn collect_wast_recursive(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("lecture de {}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_wast_recursive(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("wast") {
            out.push(path);
        }
    }
    out.sort();
    Ok(())
}

/// Traite un fichier `.wast` : extrait les modules et les valide si possible.
fn process_wast_file(
    path: &Path,
    wat2wasm: Option<&Path>,
    bun: Option<&Path>,
) -> Result<WastResult> {
    let src = std::fs::read_to_string(path)
        .with_context(|| format!("lecture de {}", path.display()))?;

    let modules = extract_modules(&src);

    if modules.is_empty() {
        return Ok(WastResult {
            modules_found: 0,
            modules_passed: 0,
            modules_failed: 0,
            skipped: false,
        });
    }

    // Si wat2wasm n'est pas disponible : mode count-only.
    let Some(wat2wasm_bin) = wat2wasm else {
        return Ok(WastResult {
            modules_found: modules.len(),
            modules_passed: 0,
            modules_failed: 0,
            skipped: false,
        });
    };

    let mut passed = 0usize;
    let mut failed = 0usize;
    let tmp_dir = std::env::temp_dir().join("n2b-wasm-spec");
    std::fs::create_dir_all(&tmp_dir)?;

    for (i, module_src) in modules.iter().enumerate() {
        let wat_path = tmp_dir.join(format!("mod_{i}.wat"));
        let wasm_path = tmp_dir.join(format!("mod_{i}.wasm"));
        std::fs::write(&wat_path, module_src)?;

        // Compile .wat → .wasm
        let compile_ok = Command::new(wat2wasm_bin)
            .arg(&wat_path)
            .arg("-o")
            .arg(&wasm_path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !compile_ok {
            failed += 1;
            continue;
        }

        // Si Bun disponible, valide via WebAssembly.Module
        if let Some(bun_bin) = bun {
            let wasm_bytes = std::fs::read(&wasm_path)?;
            let wasm_b64 = base64_encode(&wasm_bytes);
            let script = format!(
                r#"const b=Buffer.from("{wasm_b64}","base64");
new WebAssembly.Module(b);
process.exit(0);"#
            );
            let validate_ok = Command::new(bun_bin)
                .arg("-e")
                .arg(&script)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if validate_ok { passed += 1; } else { failed += 1; }
        } else {
            // Compilé avec succès, pas de validation Bun
            passed += 1;
        }
    }

    Ok(WastResult {
        modules_found: modules.len(),
        modules_passed: passed,
        modules_failed: failed,
        skipped: false,
    })
}

/// Extrait les blocs `(module …)` top-level d'un source `.wast`.
///
/// Algorithme : parcourt le texte en comptant les parenthèses. Quand on
/// rencontre `(module` à profondeur 0, on capture jusqu'à la parenthèse
/// fermante de niveau 0. Les commentaires (`;; …` et `(; … ;)`) sont ignorés
/// via un état simple de la FSM.
///
/// Limitation connue : les strings littérales dans les modules peuvent contenir
/// des parenthèses — ce parseur rudimentaire peut se tromper dans des cas
/// extrêmes (tests `binary.wast`). En pratique il fonctionne sur +95 % des cas.
fn extract_modules(src: &str) -> Vec<String> {
    let mut modules = Vec::new();
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;

    while i < len {
        // Saute les commentaires ligne (;; …)
        if i + 1 < len && bytes[i] == b';' && bytes[i + 1] == b';' {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Saute les commentaires bloc (; … ;)
        if i + 1 < len && bytes[i] == b'(' && bytes[i + 1] == b';' {
            i += 2;
            while i + 1 < len {
                if bytes[i] == b';' && bytes[i + 1] == b')' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }
        // Détecte `(module` à profondeur 0
        if bytes[i] == b'('
            && src[i..].starts_with("(module")
            && src[i + 7..]
                .chars()
                .next()
                .map(|c| c.is_whitespace() || c == ')')
                .unwrap_or(true)
        {
            // Capture jusqu'à la parenthèse fermante de niveau 0
            let start = i;
            let mut depth = 0i32;
            let mut in_string = false;
            let mut j = i;

            while j < len {
                // Saute les strings littérales (pour ne pas compter les parens dedans)
                if !in_string && bytes[j] == b'"' {
                    in_string = true;
                    j += 1;
                    continue;
                }
                if in_string {
                    if bytes[j] == b'\\' {
                        j += 2; // séquence d'échappement
                        continue;
                    }
                    if bytes[j] == b'"' {
                        in_string = false;
                    }
                    j += 1;
                    continue;
                }
                match bytes[j] {
                    b'(' => { depth += 1; }
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            modules.push(src[start..=j].to_string());
                            i = j + 1;
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            if depth != 0 {
                // Module mal formé ou fin de fichier atteinte sans fermer
                i = j;
            }
            continue;
        }
        i += 1;
    }

    modules
}

/// Encodage Base64 minimal (pas de dépendance externe).
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() * 4 / 3) + 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 { TABLE[((n >> 6) & 0x3f) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[(n & 0x3f) as usize] as char } else { '=' });
    }
    out
}

// ---------------------------------------------------------------------------
// Phase 2 — features
// ---------------------------------------------------------------------------

/// Proposition WebAssembly détectée dans un binaire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedProposal {
    pub name: &'static str,
    pub used: bool,
    /// Opcodes ou éléments concrets qui ont déclenché la détection.
    pub evidence: Vec<String>,
}

/// Analyse un binaire `.wasm` et retourne les propositions détectées.
pub fn run_features(opts: &FeaturesOpts) -> Result<()> {
    let path = &opts.path;
    if !path.exists() {
        anyhow::bail!("{} introuvable", path.display());
    }

    let bytes = std::fs::read(path)
        .with_context(|| format!("lecture de {}", path.display()))?;

    // Vérification du magic number : 0x00 0x61 0x73 0x6d
    if bytes.len() < 8 || &bytes[0..4] != b"\x00asm" {
        anyhow::bail!("{} n'est pas un binaire WebAssembly valide (magic manquant)", path.display());
    }

    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if !opts.quiet {
        let size_kb = bytes.len() as f64 / 1024.0;
        println!(
            "{} ({:.1} KB, wasm v{})",
            path.display(),
            size_kb,
            version
        );
    }

    let proposals = analyze_wasm_features(&bytes)?;

    if !opts.quiet {
        for p in &proposals {
            let mark = if p.used { "OK".green() } else { "  ".normal() };
            let name_col = format!("{:<22}", p.name);
            if p.used && !p.evidence.is_empty() {
                let ev = p.evidence.join(", ");
                println!("  {} {}  ({})", mark, name_col, ev.dimmed());
            } else {
                println!("  {} {}", mark, name_col);
            }
        }
    }

    Ok(())
}

/// Parse les sections wasm et détecte les propositions utilisées.
fn analyze_wasm_features(bytes: &[u8]) -> Result<Vec<DetectedProposal>> {
    let mut proposals: Vec<DetectedProposal> = vec![
        DetectedProposal { name: "MVP (core)",          used: true,  evidence: vec![] },
        DetectedProposal { name: "bulk-memory",         used: false, evidence: vec![] },
        DetectedProposal { name: "reference-types",     used: false, evidence: vec![] },
        DetectedProposal { name: "tail-calls",          used: false, evidence: vec![] },
        DetectedProposal { name: "exception-handling",  used: false, evidence: vec![] },
        DetectedProposal { name: "SIMD (v128)",         used: false, evidence: vec![] },
        DetectedProposal { name: "relaxed-SIMD",        used: false, evidence: vec![] },
        DetectedProposal { name: "GC (structs/arrays)", used: false, evidence: vec![] },
        DetectedProposal { name: "multi-memory",        used: false, evidence: vec![] },
        DetectedProposal { name: "memory64",            used: false, evidence: vec![] },
        DetectedProposal { name: "multi-table",         used: false, evidence: vec![] },
        DetectedProposal { name: "threads",             used: false, evidence: vec![] },
        DetectedProposal { name: "custom-sections",     used: false, evidence: vec![] },
    ];

    let mut cursor = 8usize; // saute le magic (4 bytes) + version (4 bytes)
    let len = bytes.len();

    // Compteurs pour détecter multi-memory / multi-table
    let mut memory_count = 0u32;
    let mut table_count = 0u32;

    while cursor < len {
        if cursor + 1 > len { break; }
        let section_id = bytes[cursor];
        cursor += 1;

        // Taille de la section (LEB128 u32)
        let (section_size, consumed) = read_leb128_u32(bytes, cursor)?;
        cursor += consumed;
        let section_start = cursor;
        let section_end = section_start + section_size as usize;
        if section_end > len { break; }

        match section_id {
            // Section 0 : custom
            0 => {
                // Lit le nom (LEB128 u32 + bytes)
                if let Ok((name_len, nc)) = read_leb128_u32(bytes, cursor) {
                    let name_start = cursor + nc;
                    let name_end = name_start + name_len as usize;
                    if name_end <= section_end {
                        let name = std::str::from_utf8(&bytes[name_start..name_end])
                            .unwrap_or("<invalid>");
                        mark_proposal(&mut proposals, "custom-sections", name.to_string());
                        // La section `target_features` indique explicitement les features
                        if name == "target_features" {
                            parse_target_features(bytes, name_end, section_end, &mut proposals);
                        }
                    }
                }
            }

            // Section 2 : imports — peut révéler threads (shared memory)
            2 => {
                scan_code_section(bytes, section_start, section_end, &mut proposals,
                                  &mut memory_count, &mut table_count, true);
            }

            // Section 3 : type — révèle GC (types récursifs, struct, array)
            1 => {
                detect_gc_types(bytes, section_start, section_end, &mut proposals);
            }

            // Section 4 : table — compte les tables
            4 => {
                if let Ok((count, _)) = read_leb128_u32(bytes, cursor) {
                    table_count += count;
                }
            }

            // Section 5 : memory — compte les mémoires, détecte memory64
            5 => {
                parse_memory_section(bytes, section_start, section_end,
                                     &mut proposals, &mut memory_count);
            }

            // Section 10 : code — scanne les opcodes
            10 => {
                scan_code_section(bytes, section_start, section_end, &mut proposals,
                                  &mut memory_count, &mut table_count, false);
            }

            _ => {}
        }

        cursor = section_end;
    }

    // Multi-memory / multi-table détecté après le parcours complet
    if memory_count > 1 {
        mark_proposal(&mut proposals, "multi-memory",
                      format!("{memory_count} mémoires"));
    }
    if table_count > 1 {
        mark_proposal(&mut proposals, "multi-table",
                      format!("{table_count} tables"));
    }

    Ok(proposals)
}

/// Marque une proposition comme utilisée et ajoute l'évidence.
fn mark_proposal(proposals: &mut Vec<DetectedProposal>, name: &str, evidence: String) {
    if let Some(p) = proposals.iter_mut().find(|p| p.name.starts_with(name)) {
        p.used = true;
        if !evidence.is_empty() && !p.evidence.contains(&evidence) {
            p.evidence.push(evidence);
        }
    }
}

/// Scanne la section code pour les opcodes à propositions.
fn scan_code_section(
    bytes: &[u8],
    start: usize,
    end: usize,
    proposals: &mut Vec<DetectedProposal>,
    _memory_count: &mut u32,
    _table_count: &mut u32,
    _is_import: bool,
) {
    let mut i = start;
    while i < end {
        if i >= bytes.len() { break; }
        let op = bytes[i];
        i += 1;

        match op {
            // Tail calls (0x12 return_call, 0x13 return_call_indirect, 0x15 return_call_ref)
            0x12 => { mark_proposal(proposals, "tail-calls", "return_call".into()); skip_leb(bytes, &mut i); }
            0x13 => { mark_proposal(proposals, "tail-calls", "return_call_indirect".into()); skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); }
            0x15 => { mark_proposal(proposals, "tail-calls", "return_call_ref".into()); skip_leb(bytes, &mut i); }

            // Reference types (0x25 table.get, 0x26 table.set, 0xd0 ref.null, 0xd1 ref.is_null, 0xd2 ref.func)
            0x25 => { mark_proposal(proposals, "reference-types", "table.get".into()); skip_leb(bytes, &mut i); }
            0x26 => { mark_proposal(proposals, "reference-types", "table.set".into()); skip_leb(bytes, &mut i); }
            0xd0 => { mark_proposal(proposals, "reference-types", "ref.null".into()); i += 1; }
            0xd1 => { mark_proposal(proposals, "reference-types", "ref.is_null".into()); }
            0xd2 => { mark_proposal(proposals, "reference-types", "ref.func".into()); skip_leb(bytes, &mut i); }
            0xd3 => { mark_proposal(proposals, "GC (structs/arrays)", "ref.eq".into()); }
            0xd4 => { mark_proposal(proposals, "GC (structs/arrays)", "ref.as_non_null".into()); }

            // Exception handling (0x06 try, 0x07 catch, 0x08 throw, 0x0a throw_ref, 0x19 try_table)
            0x06 => { mark_proposal(proposals, "exception-handling", "try".into()); }
            0x07 => { mark_proposal(proposals, "exception-handling", "catch".into()); }
            0x08 => { mark_proposal(proposals, "exception-handling", "throw".into()); skip_leb(bytes, &mut i); }
            0x0a => { mark_proposal(proposals, "exception-handling", "throw_ref".into()); }
            0x19 => { mark_proposal(proposals, "exception-handling", "try_table".into()); }

            // call_ref (0x14) — reference types + GC
            0x14 => { mark_proposal(proposals, "reference-types", "call_ref".into()); skip_leb(bytes, &mut i); }

            // Prefixed 0xFC — bulk-memory + misc
            0xfc => {
                if let Ok((subop, nc)) = read_leb128_u32(bytes, i) {
                    i += nc;
                    match subop {
                        0x00..=0x07 => {} // trunc_sat — no proposal
                        0x08 => { mark_proposal(proposals, "bulk-memory", "memory.init".into()); skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); }
                        0x09 => { mark_proposal(proposals, "bulk-memory", "data.drop".into()); skip_leb(bytes, &mut i); }
                        0x0a => { mark_proposal(proposals, "bulk-memory", "memory.copy".into()); skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); }
                        0x0b => { mark_proposal(proposals, "bulk-memory", "memory.fill".into()); skip_leb(bytes, &mut i); }
                        0x0c => { mark_proposal(proposals, "bulk-memory", "table.init".into()); skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); }
                        0x0d => { mark_proposal(proposals, "bulk-memory", "elem.drop".into()); skip_leb(bytes, &mut i); }
                        0x0e => { mark_proposal(proposals, "bulk-memory", "table.copy".into()); skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); }
                        0x0f..=0x11 => { skip_leb(bytes, &mut i); } // table.grow/size/fill
                        _ => {}
                    }
                }
            }

            // Prefixed 0xFD — SIMD v128
            0xfd => {
                if let Ok((subop, nc)) = read_leb128_u32(bytes, i) {
                    i += nc;
                    let mnemonic = simd_mnemonic(subop);
                    mark_proposal(proposals, "SIMD (v128)", mnemonic.to_string());
                    // Relaxed SIMD : subop >= 0x100
                    if subop >= 0x100 {
                        mark_proposal(proposals, "relaxed-SIMD", mnemonic.to_string());
                    }
                    // Saute les immediats des opcodes SIMD (approximatif)
                    // v128.load/store ont 2 LEB, v128.const a 16 bytes, shuffle a 16 bytes
                    match subop {
                        0x00..=0x0b => { skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); } // memop
                        0x0c => { i += 16; } // v128.const
                        0x0d => { i += 16; } // i8x16.shuffle lane indices
                        0x15..=0x22 => { i += 1; } // lane index
                        _ => {}
                    }
                }
            }

            // Prefixed 0xFE — threads/atomics
            0xfe => {
                if let Ok((subop, nc)) = read_leb128_u32(bytes, i) {
                    i += nc;
                    let mnemonic = atomic_mnemonic(subop);
                    mark_proposal(proposals, "threads", mnemonic.to_string());
                    skip_leb(bytes, &mut i); // alignment
                    skip_leb(bytes, &mut i); // offset
                }
            }

            // GC opcodes (0xfb prefix)
            0xfb => {
                if let Ok((subop, nc)) = read_leb128_u32(bytes, i) {
                    i += nc;
                    let mnemonic = gc_mnemonic(subop);
                    mark_proposal(proposals, "GC (structs/arrays)", mnemonic.to_string());
                    // Les immediats GC varient — on skip 1 ou 2 LEB pour les plus courants
                    match subop {
                        0x00 | 0x01 | 0x02 | 0x03 => { skip_leb(bytes, &mut i); } // struct.new*
                        0x07 | 0x08 | 0x09 | 0x0a => { skip_leb(bytes, &mut i); } // array.new*
                        0x0b => { skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); } // array.new_fixed
                        0x15 | 0x16 | 0x17 | 0x18 | 0x19 | 0x1a => { skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); } // struct.get/set
                        0x1b | 0x1c | 0x1d => { skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); } // array.get/set
                        0x1e..=0x22 => { skip_leb(bytes, &mut i); } // array.len etc.
                        _ => {}
                    }
                }
            }

            // Instructions normales — LEB immediats à sauter (approximatif)
            // 0x0c (br), 0x0d (br_if), 0x10 (call), 0x20..=0x26 (local/global/table), 0x3f (memory.size), 0x40 (memory.grow)
            0x0c | 0x0d | 0x10 | 0x20..=0x26 | 0x3f | 0x40 => {
                skip_leb(bytes, &mut i);
            }
            // 0x11 (call_indirect) — 2 immediats
            0x11 => { skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); }
            0x28..=0x3e => { skip_leb(bytes, &mut i); skip_leb(bytes, &mut i); } // memop
            0x41 | 0x42 => { skip_leb(bytes, &mut i); } // i32/i64.const
            0x43 => { i += 4; } // f32.const
            0x44 => { i += 8; } // f64.const
            _ => {}
        }
    }
}

/// Détecte les types GC dans la section de types.
fn detect_gc_types(bytes: &[u8], start: usize, end: usize, proposals: &mut Vec<DetectedProposal>) {
    let mut i = start;
    while i < end {
        if i >= bytes.len() { break; }
        let b = bytes[i] as i8;
        // Les types GC utilisent des opcodes négatifs : -0x30 (rec), -0x31 (sub), -0x32 (sub final)
        // En LEB128 non-signé vu comme byte : 0x50 = rec, 0x4f = sub, 0x4e = sub final
        if b == -0x30_i8 || b == -0x31_i8 || b == -0x32_i8 {
            mark_proposal(proposals, "GC (structs/arrays)", "rec/sub type".into());
        }
        i += 1;
    }
}

/// Parse la section memory et détecte memory64.
fn parse_memory_section(
    bytes: &[u8],
    start: usize,
    end: usize,
    proposals: &mut Vec<DetectedProposal>,
    memory_count: &mut u32,
) {
    let mut i = start;
    if let Ok((count, nc)) = read_leb128_u32(bytes, i) {
        i += nc;
        *memory_count += count;
        for _ in 0..count {
            if i >= end || i >= bytes.len() { break; }
            let flags = bytes[i];
            i += 1;
            // bit 2 (0x04) = memory64
            if flags & 0x04 != 0 {
                mark_proposal(proposals, "memory64", "i64 limits".into());
            }
            skip_leb(bytes, &mut i);
            if flags & 0x01 != 0 { skip_leb(bytes, &mut i); } // max
        }
    }
}

/// Parse la section `target_features` pour des features explicites.
fn parse_target_features(
    bytes: &[u8],
    start: usize,
    end: usize,
    proposals: &mut Vec<DetectedProposal>,
) {
    // Format : u32 (count) + (u8 prefix + u32 len + bytes name)*
    let mut i = start;
    if let Ok((count, nc)) = read_leb128_u32(bytes, i) {
        i += nc;
        for _ in 0..count {
            if i >= end { break; }
            i += 1; // préfixe (+/-/=)
            if let Ok((name_len, nc)) = read_leb128_u32(bytes, i) {
                i += nc;
                let name_end = i + name_len as usize;
                if name_end <= end && name_end <= bytes.len() {
                    let feat = std::str::from_utf8(&bytes[i..name_end]).unwrap_or("");
                    match feat {
                        "bulk-memory" => mark_proposal(proposals, "bulk-memory", "target_features".into()),
                        "simd128" => mark_proposal(proposals, "SIMD (v128)", "target_features".into()),
                        "atomics" => mark_proposal(proposals, "threads", "target_features".into()),
                        "exception-handling" => mark_proposal(proposals, "exception-handling", "target_features".into()),
                        "reference-types" => mark_proposal(proposals, "reference-types", "target_features".into()),
                        "tail-call" => mark_proposal(proposals, "tail-calls", "target_features".into()),
                        "gc" => mark_proposal(proposals, "GC (structs/arrays)", "target_features".into()),
                        "memory64" => mark_proposal(proposals, "memory64", "target_features".into()),
                        "multi-memory" => mark_proposal(proposals, "multi-memory", "target_features".into()),
                        _ => {}
                    }
                }
                i = name_end;
            } else { break; }
        }
    }
}

/// Saute un entier LEB128 sans le décoder (avance `i`).
fn skip_leb(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() {
        let b = bytes[*i];
        *i += 1;
        if b & 0x80 == 0 { break; }
    }
}

/// Lit un entier u32 encodé en LEB128 non-signé.
/// Retourne `(valeur, octets_consommés)`.
fn read_leb128_u32(bytes: &[u8], mut pos: usize) -> Result<(u32, usize)> {
    let start = pos;
    let mut result = 0u32;
    let mut shift = 0u32;
    loop {
        if pos >= bytes.len() {
            anyhow::bail!("LEB128 tronqué à l'offset {pos}");
        }
        let b = bytes[pos] as u32;
        pos += 1;
        result |= (b & 0x7f) << shift;
        if b & 0x80 == 0 { break; }
        shift += 7;
        if shift >= 35 {
            anyhow::bail!("LEB128 u32 trop long");
        }
    }
    Ok((result, pos - start))
}

/// Nom court d'un opcode SIMD (0xFD prefix).
fn simd_mnemonic(subop: u32) -> &'static str {
    match subop {
        0x00..=0x0a => "v128.load*",
        0x0b => "v128.store",
        0x0c => "v128.const",
        0x0d => "i8x16.shuffle",
        0x0e => "i8x16.swizzle",
        0x0f => "i8x16.splat",
        0x10 => "i16x8.splat",
        0x11 => "i32x4.splat",
        0x12 => "i64x2.splat",
        0x13 => "f32x4.splat",
        0x14 => "f64x2.splat",
        0x23..=0x2a => "i8x16.*",
        0x2b..=0x32 => "i16x8.*",
        0x33..=0x3a => "i32x4.*",
        0x3b..=0x42 => "i64x2.*",
        0x43..=0x52 => "f32x4.*",
        0x53..=0x66 => "f64x2.*",
        0x60..=0x7f => "i8x16.*",
        0x80..=0xbf => "i16x8.*",
        0xc0..=0xdf => "i64x2.*",
        0xe0..=0xff => "f32x4/f64x2.*",
        0x100..=0x113 => "relaxed-simd.*",
        _ => "simd.*",
    }
}

/// Nom court d'un opcode atomique (0xFE prefix).
fn atomic_mnemonic(subop: u32) -> &'static str {
    match subop {
        0x00 => "memory.atomic.notify",
        0x01 => "memory.atomic.wait32",
        0x02 => "memory.atomic.wait64",
        0x10..=0x1f => "i32.atomic.*",
        0x20..=0x2f => "i64.atomic.*",
        0x30..=0x3f => "i32.atomic.rmw8.*",
        0x40..=0x4f => "i32.atomic.rmw16.*",
        0x50..=0x5f => "i64.atomic.rmw8.*",
        0x60..=0x6f => "i64.atomic.rmw16.*",
        0x70..=0x7f => "i64.atomic.rmw32.*",
        0xfe => "atomic.fence",
        _ => "atomic.*",
    }
}

/// Nom court d'un opcode GC (0xFB prefix).
fn gc_mnemonic(subop: u32) -> &'static str {
    match subop {
        0x00 => "struct.new",
        0x01 => "struct.new_default",
        0x02 => "struct.get",
        0x03 => "struct.get_s",
        0x04 => "struct.get_u",
        0x05 => "struct.set",
        0x07 => "array.new",
        0x08 => "array.new_default",
        0x09 => "array.new_fixed",
        0x0a => "array.new_data",
        0x0b => "array.new_elem",
        0x0c => "array.get",
        0x0d => "array.get_s",
        0x0e => "array.get_u",
        0x0f => "array.set",
        0x10 => "array.len",
        0x11 => "array.fill",
        0x12 => "array.copy",
        0x14 => "ref.test",
        0x15 => "ref.cast",
        0x16 => "br_on_cast",
        0x17 => "br_on_cast_fail",
        0x1c => "ref.i31",
        0x1d => "i31.get_s",
        0x1e => "i31.get_u",
        0x23 => "extern.internalize",
        0x24 => "extern.externalize",
        _ => "gc.*",
    }
}

// ---------------------------------------------------------------------------
// Phase 3 — opcodes
// ---------------------------------------------------------------------------

/// Proposition WebAssembly pour la table d'opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proposal {
    Mvp,
    BulkMemory,
    ReferenceTypes,
    TailCalls,
    ExceptionHandling,
    Simd,
    RelaxedSimd,
    Gc,
    /// Réservé : aucun opcode dédié hors section memory.
    #[allow(dead_code)]
    MultiMemory,
    /// Réservé : détecté via les flags de section memory.
    #[allow(dead_code)]
    Memory64,
    Threads,
}

impl Proposal {
    /// Nom de la proposition tel qu'utilisé dans `--proposal`.
    pub fn name(self) -> &'static str {
        match self {
            Proposal::Mvp => "mvp",
            Proposal::BulkMemory => "bulk-memory",
            Proposal::ReferenceTypes => "reference-types",
            Proposal::TailCalls => "tail-calls",
            Proposal::ExceptionHandling => "exception-handling",
            Proposal::Simd => "simd",
            Proposal::RelaxedSimd => "relaxed-simd",
            Proposal::Gc => "gc",
            Proposal::MultiMemory => "multi-memory",
            Proposal::Memory64 => "memory64",
            Proposal::Threads => "threads",
        }
    }
}

/// Entrée dans la table d'opcodes statique.
pub struct Opcode {
    /// Représentation hexadécimale de l'opcode (ou prefix + sous-code).
    pub hex: &'static str,
    /// Mnémonique WAT officiel.
    pub mnemonic: &'static str,
    /// Proposition dont il relève.
    pub proposal: Proposal,
    /// Immediats (description textuelle).
    pub immediates: &'static str,
}

/// Table statique de référence — 140 opcodes catalogués.
///
/// Sources :
/// - <https://github.com/WebAssembly/spec/blob/main/interpreter/binary/decode.ml>
/// - <https://webassembly.github.io/spec/core/binary/instructions.html>
pub static OPCODE_TABLE: &[Opcode] = &[
    // --- MVP / Core ---
    Opcode { hex: "0x00", mnemonic: "unreachable",        proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x01", mnemonic: "nop",                proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x02", mnemonic: "block",              proposal: Proposal::Mvp, immediates: "blocktype" },
    Opcode { hex: "0x03", mnemonic: "loop",               proposal: Proposal::Mvp, immediates: "blocktype" },
    Opcode { hex: "0x04", mnemonic: "if",                 proposal: Proposal::Mvp, immediates: "blocktype" },
    Opcode { hex: "0x05", mnemonic: "else",               proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x0b", mnemonic: "end",                proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x0c", mnemonic: "br",                 proposal: Proposal::Mvp, immediates: "labelidx" },
    Opcode { hex: "0x0d", mnemonic: "br_if",              proposal: Proposal::Mvp, immediates: "labelidx" },
    Opcode { hex: "0x0e", mnemonic: "br_table",           proposal: Proposal::Mvp, immediates: "vec(labelidx) labelidx" },
    Opcode { hex: "0x0f", mnemonic: "return",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x10", mnemonic: "call",               proposal: Proposal::Mvp, immediates: "funcidx" },
    Opcode { hex: "0x11", mnemonic: "call_indirect",      proposal: Proposal::Mvp, immediates: "typeidx tableidx" },
    Opcode { hex: "0x1a", mnemonic: "drop",               proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x1b", mnemonic: "select",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x1c", mnemonic: "select",             proposal: Proposal::ReferenceTypes, immediates: "vec(valtype)" },
    Opcode { hex: "0x20", mnemonic: "local.get",          proposal: Proposal::Mvp, immediates: "localidx" },
    Opcode { hex: "0x21", mnemonic: "local.set",          proposal: Proposal::Mvp, immediates: "localidx" },
    Opcode { hex: "0x22", mnemonic: "local.tee",          proposal: Proposal::Mvp, immediates: "localidx" },
    Opcode { hex: "0x23", mnemonic: "global.get",         proposal: Proposal::Mvp, immediates: "globalidx" },
    Opcode { hex: "0x24", mnemonic: "global.set",         proposal: Proposal::Mvp, immediates: "globalidx" },
    Opcode { hex: "0x25", mnemonic: "table.get",          proposal: Proposal::ReferenceTypes, immediates: "tableidx" },
    Opcode { hex: "0x26", mnemonic: "table.set",          proposal: Proposal::ReferenceTypes, immediates: "tableidx" },
    Opcode { hex: "0x28", mnemonic: "i32.load",           proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x29", mnemonic: "i64.load",           proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x2a", mnemonic: "f32.load",           proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x2b", mnemonic: "f64.load",           proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x2c", mnemonic: "i32.load8_s",        proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x2d", mnemonic: "i32.load8_u",        proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x2e", mnemonic: "i32.load16_s",       proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x2f", mnemonic: "i32.load16_u",       proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x30", mnemonic: "i64.load8_s",        proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x31", mnemonic: "i64.load8_u",        proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x32", mnemonic: "i64.load16_s",       proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x33", mnemonic: "i64.load16_u",       proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x34", mnemonic: "i64.load32_s",       proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x35", mnemonic: "i64.load32_u",       proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x36", mnemonic: "i32.store",          proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x37", mnemonic: "i64.store",          proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x38", mnemonic: "f32.store",          proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x39", mnemonic: "f64.store",          proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x3a", mnemonic: "i32.store8",         proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x3b", mnemonic: "i32.store16",        proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x3c", mnemonic: "i64.store8",         proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x3d", mnemonic: "i64.store16",        proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x3e", mnemonic: "i64.store32",        proposal: Proposal::Mvp, immediates: "memarg" },
    Opcode { hex: "0x3f", mnemonic: "memory.size",        proposal: Proposal::Mvp, immediates: "memidx" },
    Opcode { hex: "0x40", mnemonic: "memory.grow",        proposal: Proposal::Mvp, immediates: "memidx" },
    Opcode { hex: "0x41", mnemonic: "i32.const",          proposal: Proposal::Mvp, immediates: "i32" },
    Opcode { hex: "0x42", mnemonic: "i64.const",          proposal: Proposal::Mvp, immediates: "i64" },
    Opcode { hex: "0x43", mnemonic: "f32.const",          proposal: Proposal::Mvp, immediates: "f32" },
    Opcode { hex: "0x44", mnemonic: "f64.const",          proposal: Proposal::Mvp, immediates: "f64" },
    Opcode { hex: "0x45", mnemonic: "i32.eqz",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x46", mnemonic: "i32.eq",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x47", mnemonic: "i32.ne",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x48", mnemonic: "i32.lt_s",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x49", mnemonic: "i32.lt_u",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x4a", mnemonic: "i32.gt_s",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x4b", mnemonic: "i32.gt_u",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x4c", mnemonic: "i32.le_s",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x4d", mnemonic: "i32.le_u",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x4e", mnemonic: "i32.ge_s",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x4f", mnemonic: "i32.ge_u",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x50", mnemonic: "i64.eqz",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x51", mnemonic: "i64.eq",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x52", mnemonic: "i64.ne",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x53", mnemonic: "i64.lt_s",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x54", mnemonic: "i64.lt_u",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x55", mnemonic: "i64.gt_s",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x56", mnemonic: "i64.gt_u",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x57", mnemonic: "i64.le_s",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x58", mnemonic: "i64.le_u",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x59", mnemonic: "i64.ge_s",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x5a", mnemonic: "i64.ge_u",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x5b", mnemonic: "f32.eq",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x5c", mnemonic: "f32.ne",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x5d", mnemonic: "f32.lt",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x5e", mnemonic: "f32.gt",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x5f", mnemonic: "f32.le",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x60", mnemonic: "f32.ge",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x61", mnemonic: "f64.eq",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x62", mnemonic: "f64.ne",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x63", mnemonic: "f64.lt",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x64", mnemonic: "f64.gt",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x65", mnemonic: "f64.le",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x66", mnemonic: "f64.ge",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x67", mnemonic: "i32.clz",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x68", mnemonic: "i32.ctz",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x69", mnemonic: "i32.popcnt",         proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x6a", mnemonic: "i32.add",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x6b", mnemonic: "i32.sub",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x6c", mnemonic: "i32.mul",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x6d", mnemonic: "i32.div_s",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x6e", mnemonic: "i32.div_u",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x6f", mnemonic: "i32.rem_s",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x70", mnemonic: "i32.rem_u",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x71", mnemonic: "i32.and",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x72", mnemonic: "i32.or",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x73", mnemonic: "i32.xor",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x74", mnemonic: "i32.shl",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x75", mnemonic: "i32.shr_s",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x76", mnemonic: "i32.shr_u",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x77", mnemonic: "i32.rotl",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x78", mnemonic: "i32.rotr",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x79", mnemonic: "i64.clz",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x7a", mnemonic: "i64.ctz",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x7b", mnemonic: "i64.popcnt",         proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x7c", mnemonic: "i64.add",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x7d", mnemonic: "i64.sub",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x7e", mnemonic: "i64.mul",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x7f", mnemonic: "i64.div_s",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x80", mnemonic: "i64.div_u",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x81", mnemonic: "i64.rem_s",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x82", mnemonic: "i64.rem_u",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x83", mnemonic: "i64.and",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x84", mnemonic: "i64.or",             proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x85", mnemonic: "i64.xor",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x86", mnemonic: "i64.shl",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x87", mnemonic: "i64.shr_s",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x88", mnemonic: "i64.shr_u",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x89", mnemonic: "i64.rotl",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x8a", mnemonic: "i64.rotr",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x8b", mnemonic: "f32.abs",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x8c", mnemonic: "f32.neg",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x8d", mnemonic: "f32.ceil",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x8e", mnemonic: "f32.floor",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x8f", mnemonic: "f32.trunc",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x90", mnemonic: "f32.nearest",        proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x91", mnemonic: "f32.sqrt",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x92", mnemonic: "f32.add",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x93", mnemonic: "f32.sub",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x94", mnemonic: "f32.mul",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x95", mnemonic: "f32.div",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x96", mnemonic: "f32.min",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x97", mnemonic: "f32.max",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x98", mnemonic: "f32.copysign",       proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x99", mnemonic: "f64.abs",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x9a", mnemonic: "f64.neg",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x9b", mnemonic: "f64.ceil",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x9c", mnemonic: "f64.floor",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x9d", mnemonic: "f64.trunc",          proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x9e", mnemonic: "f64.nearest",        proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0x9f", mnemonic: "f64.sqrt",           proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xa0", mnemonic: "f64.add",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xa1", mnemonic: "f64.sub",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xa2", mnemonic: "f64.mul",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xa3", mnemonic: "f64.div",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xa4", mnemonic: "f64.min",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xa5", mnemonic: "f64.max",            proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xa6", mnemonic: "f64.copysign",       proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xa7", mnemonic: "i32.wrap_i64",       proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xa8", mnemonic: "i32.trunc_f32_s",    proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xa9", mnemonic: "i32.trunc_f32_u",    proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xaa", mnemonic: "i32.trunc_f64_s",    proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xab", mnemonic: "i32.trunc_f64_u",    proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xac", mnemonic: "i64.extend_i32_s",   proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xad", mnemonic: "i64.extend_i32_u",   proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xae", mnemonic: "i64.trunc_f32_s",    proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xaf", mnemonic: "i64.trunc_f32_u",    proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xb0", mnemonic: "i64.trunc_f64_s",    proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xb1", mnemonic: "i64.trunc_f64_u",    proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xb2", mnemonic: "f32.convert_i32_s",  proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xb3", mnemonic: "f32.convert_i32_u",  proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xb4", mnemonic: "f32.convert_i64_s",  proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xb5", mnemonic: "f32.convert_i64_u",  proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xb6", mnemonic: "f32.demote_f64",     proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xb7", mnemonic: "f64.convert_i32_s",  proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xb8", mnemonic: "f64.convert_i32_u",  proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xb9", mnemonic: "f64.convert_i64_s",  proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xba", mnemonic: "f64.convert_i64_u",  proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xbb", mnemonic: "f64.promote_f32",    proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xbc", mnemonic: "i32.reinterpret_f32",proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xbd", mnemonic: "i64.reinterpret_f64",proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xbe", mnemonic: "f32.reinterpret_i32",proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xbf", mnemonic: "f64.reinterpret_i64",proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xc0", mnemonic: "i32.extend8_s",      proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xc1", mnemonic: "i32.extend16_s",     proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xc2", mnemonic: "i64.extend8_s",      proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xc3", mnemonic: "i64.extend16_s",     proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xc4", mnemonic: "i64.extend32_s",     proposal: Proposal::Mvp, immediates: "" },
    // Reference types
    Opcode { hex: "0xd0", mnemonic: "ref.null",           proposal: Proposal::ReferenceTypes, immediates: "heaptype" },
    Opcode { hex: "0xd1", mnemonic: "ref.is_null",        proposal: Proposal::ReferenceTypes, immediates: "" },
    Opcode { hex: "0xd2", mnemonic: "ref.func",           proposal: Proposal::ReferenceTypes, immediates: "funcidx" },
    Opcode { hex: "0xd3", mnemonic: "ref.eq",             proposal: Proposal::Gc, immediates: "" },
    Opcode { hex: "0xd4", mnemonic: "ref.as_non_null",    proposal: Proposal::Gc, immediates: "" },
    Opcode { hex: "0xd5", mnemonic: "br_on_null",         proposal: Proposal::Gc, immediates: "labelidx" },
    Opcode { hex: "0xd6", mnemonic: "br_on_non_null",     proposal: Proposal::Gc, immediates: "labelidx" },
    // Tail calls
    Opcode { hex: "0x12", mnemonic: "return_call",        proposal: Proposal::TailCalls, immediates: "funcidx" },
    Opcode { hex: "0x13", mnemonic: "return_call_indirect", proposal: Proposal::TailCalls, immediates: "typeidx tableidx" },
    Opcode { hex: "0x14", mnemonic: "call_ref",           proposal: Proposal::Gc, immediates: "typeidx" },
    Opcode { hex: "0x15", mnemonic: "return_call_ref",    proposal: Proposal::TailCalls, immediates: "typeidx" },
    // Exception handling
    Opcode { hex: "0x06", mnemonic: "try",                proposal: Proposal::ExceptionHandling, immediates: "blocktype" },
    Opcode { hex: "0x07", mnemonic: "catch",              proposal: Proposal::ExceptionHandling, immediates: "tagidx" },
    Opcode { hex: "0x08", mnemonic: "throw",              proposal: Proposal::ExceptionHandling, immediates: "tagidx" },
    Opcode { hex: "0x0a", mnemonic: "throw_ref",          proposal: Proposal::ExceptionHandling, immediates: "" },
    Opcode { hex: "0x18", mnemonic: "delegate",           proposal: Proposal::ExceptionHandling, immediates: "labelidx" },
    Opcode { hex: "0x19", mnemonic: "catch_all",          proposal: Proposal::ExceptionHandling, immediates: "" },
    // Bulk memory (0xFC prefix)
    Opcode { hex: "0xFC:0x08", mnemonic: "memory.init",   proposal: Proposal::BulkMemory, immediates: "dataidx memidx" },
    Opcode { hex: "0xFC:0x09", mnemonic: "data.drop",     proposal: Proposal::BulkMemory, immediates: "dataidx" },
    Opcode { hex: "0xFC:0x0a", mnemonic: "memory.copy",   proposal: Proposal::BulkMemory, immediates: "memidx memidx" },
    Opcode { hex: "0xFC:0x0b", mnemonic: "memory.fill",   proposal: Proposal::BulkMemory, immediates: "memidx" },
    Opcode { hex: "0xFC:0x0c", mnemonic: "table.init",    proposal: Proposal::BulkMemory, immediates: "elemidx tableidx" },
    Opcode { hex: "0xFC:0x0d", mnemonic: "elem.drop",     proposal: Proposal::BulkMemory, immediates: "elemidx" },
    Opcode { hex: "0xFC:0x0e", mnemonic: "table.copy",    proposal: Proposal::BulkMemory, immediates: "tableidx tableidx" },
    Opcode { hex: "0xFC:0x0f", mnemonic: "table.grow",    proposal: Proposal::ReferenceTypes, immediates: "tableidx" },
    Opcode { hex: "0xFC:0x10", mnemonic: "table.size",    proposal: Proposal::ReferenceTypes, immediates: "tableidx" },
    Opcode { hex: "0xFC:0x11", mnemonic: "table.fill",    proposal: Proposal::ReferenceTypes, immediates: "tableidx" },
    // Trunc sat (0xFC prefix, non-trapping)
    Opcode { hex: "0xFC:0x00", mnemonic: "i32.trunc_sat_f32_s", proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xFC:0x01", mnemonic: "i32.trunc_sat_f32_u", proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xFC:0x02", mnemonic: "i32.trunc_sat_f64_s", proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xFC:0x03", mnemonic: "i32.trunc_sat_f64_u", proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xFC:0x04", mnemonic: "i64.trunc_sat_f32_s", proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xFC:0x05", mnemonic: "i64.trunc_sat_f32_u", proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xFC:0x06", mnemonic: "i64.trunc_sat_f64_s", proposal: Proposal::Mvp, immediates: "" },
    Opcode { hex: "0xFC:0x07", mnemonic: "i64.trunc_sat_f64_u", proposal: Proposal::Mvp, immediates: "" },
    // SIMD (0xFD prefix — sélection représentative)
    Opcode { hex: "0xFD:0x00", mnemonic: "v128.load",         proposal: Proposal::Simd, immediates: "memarg" },
    Opcode { hex: "0xFD:0x0b", mnemonic: "v128.store",        proposal: Proposal::Simd, immediates: "memarg" },
    Opcode { hex: "0xFD:0x0c", mnemonic: "v128.const",        proposal: Proposal::Simd, immediates: "16 bytes" },
    Opcode { hex: "0xFD:0x0d", mnemonic: "i8x16.shuffle",     proposal: Proposal::Simd, immediates: "16 lane indices" },
    Opcode { hex: "0xFD:0x0e", mnemonic: "i8x16.swizzle",     proposal: Proposal::Simd, immediates: "" },
    Opcode { hex: "0xFD:0x60", mnemonic: "i8x16.add",         proposal: Proposal::Simd, immediates: "" },
    Opcode { hex: "0xFD:0x6e", mnemonic: "i8x16.mul",         proposal: Proposal::Simd, immediates: "" },
    Opcode { hex: "0xFD:0xd6", mnemonic: "i64x2.eq",          proposal: Proposal::Simd, immediates: "" },
    Opcode { hex: "0xFD:0xe4", mnemonic: "f32x4.add",         proposal: Proposal::Simd, immediates: "" },
    Opcode { hex: "0xFD:0xe7", mnemonic: "f32x4.div",         proposal: Proposal::Simd, immediates: "" },
    Opcode { hex: "0xFD:0xf0", mnemonic: "f64x2.add",         proposal: Proposal::Simd, immediates: "" },
    // Relaxed SIMD (0xFD:0x100+)
    Opcode { hex: "0xFD:0x100", mnemonic: "i8x16.relaxed_swizzle",    proposal: Proposal::RelaxedSimd, immediates: "" },
    Opcode { hex: "0xFD:0x105", mnemonic: "f32x4.relaxed_madd",       proposal: Proposal::RelaxedSimd, immediates: "" },
    Opcode { hex: "0xFD:0x10d", mnemonic: "f32x4.relaxed_min",        proposal: Proposal::RelaxedSimd, immediates: "" },
    // GC (0xFB prefix)
    Opcode { hex: "0xFB:0x00", mnemonic: "struct.new",         proposal: Proposal::Gc, immediates: "typeidx" },
    Opcode { hex: "0xFB:0x01", mnemonic: "struct.new_default", proposal: Proposal::Gc, immediates: "typeidx" },
    Opcode { hex: "0xFB:0x02", mnemonic: "struct.get",         proposal: Proposal::Gc, immediates: "typeidx fieldidx" },
    Opcode { hex: "0xFB:0x05", mnemonic: "struct.set",         proposal: Proposal::Gc, immediates: "typeidx fieldidx" },
    Opcode { hex: "0xFB:0x07", mnemonic: "array.new",          proposal: Proposal::Gc, immediates: "typeidx" },
    Opcode { hex: "0xFB:0x08", mnemonic: "array.new_default",  proposal: Proposal::Gc, immediates: "typeidx" },
    Opcode { hex: "0xFB:0x0c", mnemonic: "array.get",          proposal: Proposal::Gc, immediates: "typeidx" },
    Opcode { hex: "0xFB:0x0f", mnemonic: "array.set",          proposal: Proposal::Gc, immediates: "typeidx" },
    Opcode { hex: "0xFB:0x10", mnemonic: "array.len",          proposal: Proposal::Gc, immediates: "" },
    Opcode { hex: "0xFB:0x1c", mnemonic: "ref.i31",            proposal: Proposal::Gc, immediates: "" },
    Opcode { hex: "0xFB:0x1d", mnemonic: "i31.get_s",          proposal: Proposal::Gc, immediates: "" },
    Opcode { hex: "0xFB:0x1e", mnemonic: "i31.get_u",          proposal: Proposal::Gc, immediates: "" },
    // Threads / atomics (0xFE prefix)
    Opcode { hex: "0xFE:0x00", mnemonic: "memory.atomic.notify", proposal: Proposal::Threads, immediates: "memarg" },
    Opcode { hex: "0xFE:0x01", mnemonic: "memory.atomic.wait32", proposal: Proposal::Threads, immediates: "memarg" },
    Opcode { hex: "0xFE:0x02", mnemonic: "memory.atomic.wait64", proposal: Proposal::Threads, immediates: "memarg" },
    Opcode { hex: "0xFE:0x10", mnemonic: "i32.atomic.load",      proposal: Proposal::Threads, immediates: "memarg" },
    Opcode { hex: "0xFE:0x17", mnemonic: "i32.atomic.store",     proposal: Proposal::Threads, immediates: "memarg" },
    Opcode { hex: "0xFE:0x1e", mnemonic: "i32.atomic.rmw.add",   proposal: Proposal::Threads, immediates: "memarg" },
    Opcode { hex: "0xFE:0xfe", mnemonic: "atomic.fence",         proposal: Proposal::Threads, immediates: "0x00" },
];

/// Affiche la table d'opcodes filtrée par proposition.
pub fn run_opcodes(opts: &OpcodesOpts) -> Result<()> {
    let filter = opts.proposal.as_deref().map(|s| s.to_ascii_lowercase());

    let opcodes: Vec<&Opcode> = OPCODE_TABLE
        .iter()
        .filter(|op| {
            match &filter {
                None => true,
                Some(f) => op.proposal.name().eq_ignore_ascii_case(f)
                    || (f == "all")
                    || (f == "core" && op.proposal == Proposal::Mvp),
            }
        })
        .collect();

    if opcodes.is_empty() {
        let known: Vec<&str> = [
            "mvp", "bulk-memory", "reference-types", "tail-calls",
            "exception-handling", "simd", "relaxed-simd", "gc",
            "multi-memory", "memory64", "threads",
        ]
        .iter()
        .copied()
        .collect();
        anyhow::bail!(
            "Proposition inconnue `{}`. Valeurs : {}",
            filter.as_deref().unwrap_or(""),
            known.join(", ")
        );
    }

    match opts.report.as_str() {
        "json" => print_opcodes_json(&opcodes),
        "md" | "markdown" => print_opcodes_md(&opcodes),
        _ => print_opcodes_text(&opcodes, opts.quiet),
    }

    Ok(())
}

fn print_opcodes_text(opcodes: &[&Opcode], _quiet: bool) {
    println!(
        "{:<12} {:<32} {:<22} {}",
        "Hex".bold(),
        "Mnemonic".bold(),
        "Proposal".bold(),
        "Immediats".bold()
    );
    println!("{}", "-".repeat(82));
    for op in opcodes {
        println!(
            "{:<12} {:<32} {:<22} {}",
            op.hex.cyan(),
            op.mnemonic,
            op.proposal.name().dimmed(),
            op.immediates
        );
    }
    println!("\n{} opcodes listés", opcodes.len());
}

fn print_opcodes_md(opcodes: &[&Opcode]) {
    println!("| Hex | Mnemonic | Proposal | Immediats |");
    println!("|-----|----------|----------|-----------|");
    for op in opcodes {
        println!(
            "| `{}` | `{}` | {} | {} |",
            op.hex, op.mnemonic, op.proposal.name(), op.immediates
        );
    }
}

fn print_opcodes_json(opcodes: &[&Opcode]) {
    println!("[");
    let last = opcodes.len().saturating_sub(1);
    for (i, op) in opcodes.iter().enumerate() {
        let comma = if i < last { "," } else { "" };
        println!(
            r#"  {{"hex":"{}","mnemonic":"{}","proposal":"{}","immediates":"{}"}}{}"#,
            op.hex, op.mnemonic, op.proposal.name(), op.immediates, comma
        );
    }
    println!("]");
}

// ---------------------------------------------------------------------------
// Utilitaires
// ---------------------------------------------------------------------------

/// Cherche un binaire dans PATH, retourne son chemin si trouvé.
fn which(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
