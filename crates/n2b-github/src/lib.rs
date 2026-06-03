// Copyright 2026 aphrody-code
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::Result;
use octocrab::Octocrab;

/// Builds an Octocrab client from `GH_TOKEN`/`GITHUB_TOKEN`, falling back to a
/// `.env` file discovered by walking up from the current working directory.
/// Returns an anonymous client (no token) if none is found — the public API
/// stays reachable at the unauthenticated rate limit of 60 req/h.
pub fn client() -> Result<Octocrab> {
    match resolve_token() {
        Ok(token) => Ok(Octocrab::builder().personal_token(token).build()?),
        Err(_) => Ok(Octocrab::builder().build()?),
    }
}

/// Token env var names, in precedence order.
const TOKEN_KEYS: [&str; 2] = ["GH_TOKEN", "GITHUB_TOKEN"];

fn resolve_token() -> Result<String> {
    for key in TOKEN_KEYS {
        if let Ok(t) = std::env::var(key) {
            if !t.is_empty() {
                return Ok(t);
            }
        }
    }
    if let Some(t) = token_from_dotenv() {
        return Ok(t);
    }
    anyhow::bail!("no GitHub token (set GH_TOKEN/GITHUB_TOKEN or add it to a .env file)")
}

/// Walk up from the current working directory looking for a `.env` that
/// declares `GH_TOKEN=` or `GITHUB_TOKEN=`. No hardcoded machine paths.
fn token_from_dotenv() -> Option<String> {
    let start = std::env::current_dir().ok()?;
    for dir in start.ancestors() {
        let env_path = dir.join(".env");
        let Ok(content) = std::fs::read_to_string(&env_path) else {
            continue;
        };
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim().trim_start_matches("export ").trim();
            if TOKEN_KEYS.contains(&key) {
                let token = value.trim().trim_matches('"').trim_matches('\'');
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }
    None
}
