use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Check,
    Fix,
    Aggressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Report {
    Text,
    Json,
    Jsonl,
    Markdown,
    Sarif,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub file: String,
    pub line: u32,
    pub col: u32,
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub original: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    pub autofix: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggressive: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct FileFix {
    pub file: String,
    pub before: String,
    pub after: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub root: std::path::PathBuf,
    pub mode: Mode,
    pub report: Report,
    pub quiet: bool,
    pub ignore: Vec<String>,
    /// Mode agent : pas de couleurs, logs sur stderr uniquement, stdout réservé au payload.
    pub agent: bool,
    /// Dry-run : applique les transformations en mémoire mais n'écrit rien
    /// sur le disque. Utilisé par `n2b patch --self`.
    pub dry_run: bool,
}

#[derive(Default)]
pub struct MakeFindingOpts {
    pub severity: Option<Severity>,
    pub autofix: Option<bool>,
    pub aggressive: Option<bool>,
}
