use rustc_hash::FxHashSet;
use swc_next_ecma_ast::SymbolId;
use swc_next_ecma_ast::{Ast, AstUtf8, Diagnostic, Expr, Program};
use swc_next_ecma_semantic::Semantic;

/// Projects a selected source binding through the active TypeScript namespaces.
pub(crate) type NamespaceParameter<'visit> =
    dyn Fn(&Ast<'_>, &[u8], SymbolId) -> Option<AstUtf8> + 'visit;

/// A syntax transform which runs after TypeScript has processed an expression.
///
/// TypeScript and JSX are separate logical transforms, but TSX can lower them
/// during the same mutable AST traversal by installing JSX as this continuation.
pub(crate) trait FollowingTransform<'ast> {
    /// Resolves implicit runtime dependencies before TypeScript plans emission.
    fn runtime_dependencies(&mut self, _semantic: &Semantic<'_>) -> FxHashSet<SymbolId> {
        FxHashSet::default()
    }

    fn exit_expression(
        &mut self,
        expression: Expr,
        ast: &mut Ast<'ast>,
        namespace_parameter: &NamespaceParameter<'_>,
    ) -> Option<Expr>;

    fn finish_program(&mut self, program: Program, ast: &mut Ast<'ast>);
}

/// Diagnostics from syntax lowering.
#[derive(Default)]
pub(crate) struct SyntaxResult {
    pub(crate) diagnostics: Vec<Diagnostic<'static>>,
}
