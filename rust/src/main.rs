mod ai;
mod analyze;
mod app_cmd;
mod audit;
mod bin_cmd;
mod bunpp_cmd;
mod cli;
mod commands;
mod github;
mod linux_cmd;
mod llmstxt;
mod patch;
mod report;
mod rules;
mod run;
mod scanners;
mod schema;
#[cfg(test)]
mod schema_test;
mod subprocess;
mod types;
mod util;
mod wasm_cmd;
mod wasm_spec;
mod win32_cmd;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::dispatch::run_from_args() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("n2b a échoué : {err:?}");
            ExitCode::from(2)
        }
    }
}
