// Extraction des specifiers d'import/require via AST oxc.
//
// Remplace la regex IMPORT_RE utilisée historiquement :
//   - pas de faux positifs (commentaires, strings littérales dans du code)
//   - couvre import static, dynamic import(), require(), export-from
//   - retourne l'offset byte exact du string literal pour patching précis

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

struct Collect {
    out: Vec<Specifier>,
}

impl Collect {
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

impl<'a> Visit<'a> for Collect {
    fn visit_import_declaration(&mut self, it: &oxc_ast::ast::ImportDeclaration<'a>) {
        self.push_string_literal(&it.source);
    }

    fn visit_import_expression(&mut self, it: &oxc_ast::ast::ImportExpression<'a>) {
        if let Expression::StringLiteral(lit) = &it.source {
            self.push_string_literal(lit);
        }
        // visite les enfants aussi
        oxc_ast_visit::walk::walk_import_expression(self, it);
    }

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

    fn visit_export_named_declaration(
        &mut self,
        it: &oxc_ast::ast::ExportNamedDeclaration<'a>,
    ) {
        if let Some(src) = &it.source {
            self.push_string_literal(src);
        }
        oxc_ast_visit::walk::walk_export_named_declaration(self, it);
    }

    fn visit_export_all_declaration(
        &mut self,
        it: &oxc_ast::ast::ExportAllDeclaration<'a>,
    ) {
        self.push_string_literal(&it.source);
    }
}

fn source_type_from_path(path: &str) -> SourceType {
    SourceType::from_path(path).unwrap_or_else(|_| SourceType::default())
}

/// Parse `source` en tant que JS/TS (détection via extension du `path`)
/// et retourne tous les specifiers d'imports/requires/exports-from.
pub fn extract_specifiers(path: &str, source: &str) -> Vec<Specifier> {
    let allocator = Allocator::default();
    let st = source_type_from_path(path);
    // Pour les shebangs, oxc gère bien (il ignore la première ligne).
    let ret = Parser::new(&allocator, source, st).parse();
    // ret.errors contient les erreurs de parsing mais le programme est
    // toujours construit (parsing best-effort). On continue même en cas
    // d'erreur syntaxique.
    let mut collector = Collect { out: Vec::new() };
    for stmt in &ret.program.body {
        walk_statement(&mut collector, stmt);
    }
    collector.out
}

fn walk_statement<'a>(c: &mut Collect, stmt: &Statement<'a>) {
    use oxc_ast::ast::Statement as S;
    match stmt {
        S::ImportDeclaration(it) => c.visit_import_declaration(it),
        S::ExportNamedDeclaration(it) => c.visit_export_named_declaration(it),
        S::ExportAllDeclaration(it) => c.visit_export_all_declaration(it),
        _ => {
            // Pour le reste (expressions, fonctions, etc.), on utilise le
            // walker générique pour attraper require() et import() imbriqués.
            oxc_ast_visit::walk::walk_statement(c, stmt);
        }
    }
}
