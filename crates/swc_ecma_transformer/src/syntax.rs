use swc_next_ecma_ast::{Diagnostic, VariableDeclaration};

/// Generated program state carried between syntax lowering and later passes.
#[derive(Default)]
pub(crate) struct SyntaxResult {
    pub(crate) diagnostics: Vec<Diagnostic<'static>>,
    /// A generated `var` declaration that may also hold program-level captures.
    pub(crate) generated_prelude: Option<VariableDeclaration>,
}
