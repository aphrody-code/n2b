// n2b-core — moteur métier : scanners, règles, report, run, ai, github, audit.
// Re-export n2b-types (types + schema) pour préserver l'API publique consommée
// par n2b-cli et n2b-native (compat).

pub mod audit;
pub mod llmstxt;
pub mod run;

pub use n2b_ai as ai;
pub use n2b_github as github;
pub use n2b_report as report;
pub use n2b_rules as rules;
pub use n2b_scanners as scanners;
pub use n2b_types::{schema, types};
pub use n2b_util as util;
