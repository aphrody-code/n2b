mod analyze;
mod app_cmd;
mod bin_cmd;
mod bunpp_cmd;
mod cli;
mod commands;
mod linux_cmd;
mod patch;
#[cfg(test)]
mod schema_test;
mod subprocess;
mod wasm_cmd;
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
