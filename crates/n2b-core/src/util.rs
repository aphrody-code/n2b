use crate::types::{Finding, MakeFindingOpts, Severity};
use memchr::memchr_iter;

/// Liste les offsets (en octets UTF-8) de chaque '\n' dans `source`.
pub fn line_offsets(source: &str) -> Vec<u32> {
    memchr_iter(b'\n', source.as_bytes())
        .map(|p| p as u32)
        .collect()
}

/// Convertit un index (offset octet UTF-8) en (line, col) 1-based.
/// Reproduit la sémantique de l'ancien posFromIndex JS ; la colonne
/// compte en octets ici (acceptable pour les règles courantes).
pub fn pos_from_index(offsets: &[u32], index: usize) -> (u32, u32) {
    let idx = index as u32;
    // Plus grand offset strictement < idx.
    let i = offsets.partition_point(|&o| o < idx);
    if i == 0 {
        (1, idx + 1)
    } else {
        let line = (i as u32) + 1;
        let col = idx - offsets[i - 1];
        (line, col)
    }
}

#[allow(clippy::too_many_arguments)] // preexisting: builder pattern would require API churn
pub fn make_finding(
    path: &str,
    offsets: &[u32],
    index: usize,
    rule_id: &str,
    message: impl Into<String>,
    original: impl Into<String>,
    replacement: Option<String>,
    opts: MakeFindingOpts,
) -> Finding {
    let (line, col) = pos_from_index(offsets, index);
    let autofix = opts.autofix.unwrap_or_else(|| replacement.is_some());
    Finding {
        file: path.to_string(),
        line,
        col,
        rule_id: rule_id.to_string(),
        severity: opts.severity.unwrap_or(Severity::Warn),
        message: message.into(),
        original: original.into(),
        replacement,
        autofix,
        aggressive: opts.aggressive,
    }
}
