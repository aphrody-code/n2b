use anyhow::{Context, Result};
use octocrab::Octocrab;

/// Construit un client Octocrab depuis GH_TOKEN/GITHUB_TOKEN ou /home/ubuntu/gnu-rust/.env.
/// Retourne un client anonyme (sans token) si aucun n'est trouvé — l'API
/// publique reste accessible avec une limite de 60 req/h.
pub fn client() -> Result<Octocrab> {
    match resolve_token() {
        Ok(token) => Ok(Octocrab::builder().personal_token(token).build()?),
        Err(_) => Ok(Octocrab::builder().build()?),
    }
}

fn resolve_token() -> Result<String> {
    if let Ok(t) = std::env::var("GH_TOKEN").or_else(|_| std::env::var("GITHUB_TOKEN")) {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    let env_path = std::path::Path::new("/home/ubuntu/gnu-rust/.env");
    if env_path.exists() {
        let content = std::fs::read_to_string(env_path)
            .context("Lecture de /home/ubuntu/gnu-rust/.env")?;
        for line in content.lines() {
            let line = line.trim();
            let token = if let Some((_, v)) = line.split_once('=') {
                v.trim().trim_matches('"')
            } else if line.starts_with("ghp_") || line.starts_with("gho_") {
                line
            } else {
                continue;
            };
            if !token.is_empty() {
                return Ok(token.to_string());
            }
        }
    }
    anyhow::bail!("no GitHub token")
}
