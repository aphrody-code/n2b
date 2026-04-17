use crate::types::{Finding, MakeFindingOpts, Severity};
use crate::util::make_finding;
use once_cell::sync::Lazy;
use regex::Regex;

/// Détecte les options connues de next.config.{js,mjs,ts} qui interagissent
/// avec le portage Bun. On raisonne à la regex (pas d'AST) — le fichier n'est
/// pas critique à modifier, seulement à annoter.
///
/// Référence : https://bun.sh/guides/ecosystem/nextjs · https://nextjs.org/docs
pub fn scan_next_config(path: &str, content: &str) -> (Vec<Finding>, String) {
    let mut findings: Vec<Finding> = Vec::new();

    // 1. output: 'standalone' — build Node-oriented, pas directement exécutable par `bun run`.
    static OUTPUT_STANDALONE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"output\s*:\s*['"]standalone['"]"#).unwrap()
    });
    if OUTPUT_STANDALONE.is_match(content) {
        findings.push(make_finding(
            path,
            &[],
            0,
            "next/output-standalone",
            "output: 'standalone' produit un build Node-only (server.js) — reste exécutable via `bun run .next/standalone/server.js` mais n'exploite pas Bun.serve",
            "output: 'standalone'",
            None,
            MakeFindingOpts {
                autofix: Some(false),
                severity: Some(Severity::Info),
                ..Default::default()
            },
        ));
    }

    // 2. webpack() custom — Turbopack est le bundler par défaut Next 16.
    static WEBPACK_FN: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\bwebpack\s*(?:\(|:\s*(?:function|\())").unwrap()
    });
    if WEBPACK_FN.is_match(content) {
        findings.push(make_finding(
            path,
            &[],
            0,
            "next/webpack-custom",
            "Fonction webpack() custom détectée — Turbopack est le bundler par défaut Next 16 (option `turbopack: {}`). Vérifier si le hook webpack est toujours nécessaire",
            "webpack(",
            None,
            MakeFindingOpts {
                autofix: Some(false),
                severity: Some(Severity::Info),
                ..Default::default()
            },
        ));
    }

    // 3. experimental.serverComponentsExternalPackages
    static SERVER_EXTERNAL: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"serverComponentsExternalPackages\s*:").unwrap()
    });
    if SERVER_EXTERNAL.is_match(content) {
        findings.push(make_finding(
            path,
            &[],
            0,
            "next/server-external-packages",
            "experimental.serverComponentsExternalPackages : vérifier si ces packages ont un équivalent Bun-native (Bun.sql, Bun.redis, Bun.S3Client...) qui éliminerait le flag",
            "serverComponentsExternalPackages",
            None,
            MakeFindingOpts {
                autofix: Some(false),
                severity: Some(Severity::Info),
                ..Default::default()
            },
        ));
    }

    // 4. Absence de turbopack: (flag ou config) — Next 16 l'active par défaut,
    //    mais l'absence de mention explicite quand on porte est worth noting.
    //    On ne flaggue que si `webpack` custom est présent (sinon silencieux).
    if WEBPACK_FN.is_match(content)
        && !Regex::new(r"\bturbopack\s*:").unwrap().is_match(content)
    {
        findings.push(make_finding(
            path,
            &[],
            0,
            "next/turbopack-missing",
            "webpack() custom sans `turbopack: {}` — Next 16 préfère Turbopack ; ajouter `turbopack: {}` fige l'intention (même vide)",
            "turbopack",
            Some("turbopack: {}".to_string()),
            MakeFindingOpts {
                autofix: Some(false),
                severity: Some(Severity::Info),
                ..Default::default()
            },
        ));
    }

    // 4.5 next-rspack : withRspack() enveloppe la config
    static WITH_RSPACK: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\bwithRspack\s*\(").unwrap()
    });
    if WITH_RSPACK.is_match(content) {
        findings.push(make_finding(
            path,
            &[],
            0,
            "next/rspack-wrapper",
            "withRspack() détecté — Next.js backed by Rspack (next-rspack). Compatible Bun runtime ; voir https://rspack.rs/guide/tech/next",
            "withRspack(",
            None,
            MakeFindingOpts {
                autofix: Some(false),
                severity: Some(Severity::Info),
                ..Default::default()
            },
        ));
    }

    // 5. images.loader custom
    static IMAGES_CUSTOM: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"loader\s*:\s*['"]custom['"]"#).unwrap()
    });
    if IMAGES_CUSTOM.is_match(content) {
        findings.push(make_finding(
            path,
            &[],
            0,
            "next/images-custom-loader",
            "images.loader: 'custom' détecté — vérifier que loaderFile tourne sous Bun (fetch + sharp sont supportés nativement)",
            "loader: 'custom'",
            None,
            MakeFindingOpts {
                autofix: Some(false),
                severity: Some(Severity::Info),
                ..Default::default()
            },
        ));
    }

    (findings, content.to_string())
}

/// Détection d'un fichier next.config à la racine ou imbriqué.
pub fn is_next_config(name: &str) -> bool {
    matches!(
        name,
        "next.config.js" | "next.config.mjs" | "next.config.ts" | "next.config.cjs"
    )
}
