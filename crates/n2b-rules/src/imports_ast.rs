// Extraction des specifiers d'import/require via AST oxc.
//
// Remplace la regex IMPORT_RE utilisée historiquement :
//   - pas de faux positifs (commentaires, strings littérales dans du code)
//   - couvre import static, dynamic import(), require(), export-from
//   - retourne l'offset byte exact du string literal pour patching précis
//
// Stratégie d'extraction (oxc 0.126) :
//
//   1. ESM statique (import … from, export … from) :
//      → `ParserReturn::module_record.requested_modules` (HashMap<Str, Vec<RequestedModule>>)
//        Chaque `RequestedModule::span` couvre le string literal avec ses quotes.
//        Avantage : pas de traversée AST, O(n) sur les entrées du module record.
//
//   2. Import dynamique `import(expr)` avec specifier string littéral :
//      → `module_record.dynamic_imports` (Vec<DynamicImport>)
//        `DynamicImport::module_request` est le span de l'expression. On vérifie
//        que le premier caractère de la source est une quote (' " `) avant
//        de l'accepter comme string littérale.
//
//   3. `require("x")` (CJS) :
//      → NON présent dans le module record (CJS ≠ ESM). On conserve un visitor
//        AST ciblé uniquement sur les CallExpression nommés "require".

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[derive(Debug, Clone)]
pub struct Specifier {
    pub value: String,
    /// Offset byte du premier char du specifier (après la quote).
    pub inner_start: u32,
    pub inner_len: u32,
}

// ---------------------------------------------------------------------------
// Visitor CJS — uniquement pour require()
// ---------------------------------------------------------------------------

struct RequireCollect {
    out: Vec<Specifier>,
}

impl RequireCollect {
    fn push_string_literal(&mut self, lit: &oxc_ast::ast::StringLiteral) {
        // span couvre les quotes incluses.
        let value = lit.value.as_str().to_string();
        // inner_start = après la quote ouvrante (1 char UTF-8 : ' " ou `)
        let inner_start = lit.span.start + 1;
        let inner_len = value.len() as u32;
        self.out.push(Specifier {
            value,
            inner_start,
            inner_len,
        });
    }
}

impl<'a> Visit<'a> for RequireCollect {
    fn visit_call_expression(&mut self, it: &oxc_ast::ast::CallExpression<'a>) {
        // require("x") : callee est Expression::Identifier named "require"
        let is_require = match &it.callee {
            Expression::Identifier(ident) => ident.name.as_str() == "require",
            _ => false,
        };
        if is_require {
            if let Some(oxc_ast::ast::Argument::StringLiteral(lit)) = it.arguments.first() {
                self.push_string_literal(lit);
            }
        }
        oxc_ast_visit::walk::walk_call_expression(self, it);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn source_type_from_path(path: &str) -> SourceType {
    SourceType::from_path(path).unwrap_or_else(|_| SourceType::default())
}

/// Renvoie `true` si le caractère est une quote de début de string JS/TS.
#[inline]
fn is_string_quote(b: u8) -> bool {
    matches!(b, b'\'' | b'"' | b'`')
}

// ---------------------------------------------------------------------------
// API publique
// ---------------------------------------------------------------------------

/// Parse `source` en tant que JS/TS (détection via extension du `path`)
/// et retourne tous les specifiers d'imports/requires/exports-from.
///
/// Pour les imports/exports ESM statiques on utilise directement
/// `ParserReturn::module_record` (pas de traversée AST).
/// Le visitor est conservé uniquement pour `require()` (CJS).
pub fn extract_specifiers(path: &str, source: &str) -> Vec<Specifier> {
    let allocator = Allocator::default();
    let st = source_type_from_path(path);
    let ret = Parser::new(&allocator, source, st).parse();
    // ret.errors : parsing best-effort, on continue même avec des erreurs.

    let module = &ret.module_record;
    let src_bytes = source.as_bytes();

    // Capacité estimée : entrées module + imports dynamiques
    let cap = module.requested_modules.len() + module.dynamic_imports.len() + 4;
    let mut out: Vec<Specifier> = Vec::with_capacity(cap);

    // ------------------------------------------------------------------
    // 1. Imports/exports ESM statiques via requested_modules
    //    (couvre : import … from, export { … } from, export * from,
    //              export * as ns from)
    // ------------------------------------------------------------------
    for (name_str, occurrences) in &module.requested_modules {
        let value = name_str.as_str().to_string();
        for rm in occurrences.iter() {
            // rm.span couvre le string literal avec ses quotes.
            let inner_start = rm.span.start + 1;
            // Longueur brute dans la source (sans les deux quotes).
            let raw_len = rm.span.end.saturating_sub(rm.span.start).saturating_sub(2);
            out.push(Specifier {
                value: value.clone(),
                inner_start,
                inner_len: raw_len,
            });
        }
    }

    // ------------------------------------------------------------------
    // 2. Imports dynamiques import(expr) — uniquement string littérales
    // ------------------------------------------------------------------
    for di in module.dynamic_imports.iter() {
        let span = di.module_request;
        // Vérifie que l'expression est un string littéral (commence par une quote).
        if span.start < span.end {
            let start = span.start as usize;
            if src_bytes.get(start).copied().is_some_and(is_string_quote) {
                let value_bytes = &src_bytes[start + 1..span.end as usize - 1];
                // Safety : oxc a déjà vérifié que la source est UTF-8 valide.
                let value = std::str::from_utf8(value_bytes).unwrap_or("").to_string();
                let inner_start = span.start + 1;
                let inner_len = span.end.saturating_sub(span.start).saturating_sub(2);
                out.push(Specifier {
                    value,
                    inner_start,
                    inner_len,
                });
            }
        }
    }

    // ------------------------------------------------------------------
    // 3. require() CJS — visitor ciblé sur les CallExpression
    // ------------------------------------------------------------------
    let mut cjs = RequireCollect { out: Vec::new() };
    for stmt in &ret.program.body {
        walk_statement_for_require(&mut cjs, stmt);
    }
    out.extend(cjs.out);

    out
}

fn walk_statement_for_require<'a>(c: &mut RequireCollect, stmt: &Statement<'a>) {
    // On délègue au walker générique pour trouver les require() imbriqués
    // dans n'importe quelle expression ou bloc.
    oxc_ast_visit::walk::walk_statement(c, stmt);
}
