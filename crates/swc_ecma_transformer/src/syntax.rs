use rustc_hash::FxHashSet;
use swc_next_ecma_ast::SymbolId;
use swc_next_ecma_ast::{Ast, Diagnostic, Expr, Program, VariableDeclaration};
use swc_next_ecma_semantic::Semantic;

/// A syntax transform which runs after TypeScript has processed an expression.
///
/// TypeScript and JSX are separate logical transforms, but TSX can lower them
/// during the same mutable AST traversal by installing JSX as this continuation.
pub(crate) trait FollowingTransform<'ast> {
    /// Resolves implicit runtime dependencies before TypeScript plans emission.
    fn runtime_dependencies(&self, _semantic: &Semantic<'_>) -> FxHashSet<SymbolId> {
        FxHashSet::default()
    }

    fn exit_expression(&mut self, expression: Expr, ast: &mut Ast<'ast>) -> Option<Expr>;

    fn finish_program(&mut self, program: Program, ast: &mut Ast<'ast>);
}

/// Generated program state carried between syntax lowering and later passes.
#[derive(Default)]
pub(crate) struct SyntaxResult {
    pub(crate) diagnostics: Vec<Diagnostic<'static>>,
    /// A generated `var` declaration that may also hold program-level captures.
    pub(crate) generated_prelude: Option<VariableDeclaration>,
}
