//! SWC-compatible name resolution restored from the former `name_resolver`.

use swc_next_allocator::{Idx, IndexVec};
use swc_next_ecma_ast::*;

mod analyzer;
mod hygiene;
mod resolve;
mod scope;
mod utils;

pub(crate) use analyzer::Id;
use resolve::resolver;

/// Transformation-side context registration. Resolution itself stays in the
/// recovered resolver; `(name, ScopeId)` is SWC's `(Atom, SyntaxContext)` identity.
pub(crate) struct Resolver {
    pub(crate) node_scopes: IndexVec<NodeId, ScopeId>,
    top_level: ScopeId,
    unresolved: ScopeId,
    next_scope: usize,
}

impl Resolver {
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L136>.
    pub(crate) fn new(ast: &Ast<'_>) -> Self {
        let resolved = resolver(ast);
        Self {
            top_level: resolved.top_level_scope_id(),
            unresolved: resolved.unresolved_scope_id(),
            next_scope: resolved.next_scope,
            node_scopes: resolved.node_scopes,
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L427>.
    pub(crate) fn register(&mut self, ast: &Ast<'_>) {
        self.node_scopes.resize(ast.node_count(), self.unresolved);
    }

    /// Next AST adapter for SWC's `Ident::from(Id)` and `Ident::to_id`.
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_ast/src/ident.rs#L85>.
    pub(crate) fn binding(&mut self, ast: &mut Ast<'_>, id: &Id) -> BindingIdentifier {
        let name = ast.add_utf8(&id.name);
        let binding = ast.binding_identifier(DUMMY_SP, name);
        self.register(ast);
        self.node_scopes[binding.node_id()] = id.scope;
        binding
    }

    pub(crate) fn top_level(&self) -> ScopeId {
        self.top_level
    }
    pub(crate) fn unresolved(&self) -> ScopeId {
        self.unresolved
    }

    /// SWC explicit binding renames preserve import/export and shorthand names.
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/rename/mod.rs#L1>.
    pub(crate) fn rename(
        &mut self,
        ast: &mut Ast<'_>,
        renames: std::collections::HashMap<Id, AstUtf8>,
    ) {
        hygiene::rename(ast, self, renames);
    }

    /// SWC `private_ident!`: each generated binding receives a fresh context.
    ///
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/macros.rs#L3>.
    pub(crate) fn private_binding(&mut self, ast: &mut Ast<'_>, name: &str) -> BindingIdentifier {
        let name = ast.add_utf8(name);
        let binding = ast.binding_identifier(DUMMY_SP, name);
        self.register(ast);
        self.node_scopes[binding.node_id()] = ScopeId::from_usize(self.next_scope);
        self.next_scope += 1;
        binding
    }

    /// Copy a generated binding's syntax context onto its reference.
    ///
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L694>.
    pub(crate) fn reference(&mut self, ast: &mut Ast<'_>, binding: BindingIdentifier) -> Expr {
        let reference = ast.identifier_reference(DUMMY_SP, binding.name(ast));
        self.register(ast);
        self.node_scopes[reference.node_id()] = self.node_scopes[binding.node_id()];
        Expr::IdentifierReference(reference)
    }

    /// JSX component roots retain the context assigned by the recovered resolver.
    ///
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1459>.
    pub(crate) fn source_reference(
        &mut self,
        ast: &mut Ast<'_>,
        identifier: JsxIdentifier,
    ) -> Expr {
        let reference = ast.identifier_reference(identifier.span(ast), identifier.name(ast));
        self.register(ast);
        self.node_scopes[reference.node_id()] = self.node_scopes[identifier.node_id()];
        Expr::IdentifierReference(reference)
    }

    /// SWC marks pragma roots with the top-level mark and `require` with the
    /// unresolved mark; neither is resolved again at its insertion site.
    ///
    /// SWC pragma mark: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L180>.
    /// SWC unresolved require mark: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1400>.
    pub(crate) fn marked_reference(
        &mut self,
        ast: &mut Ast<'_>,
        name: &str,
        unresolved: bool,
    ) -> Expr {
        let name = ast.add_utf8(name);
        let reference = ast.identifier_reference(DUMMY_SP, name);
        self.register(ast);
        self.node_scopes[reference.node_id()] = if unresolved {
            self.unresolved
        } else {
            self.top_level
        };
        Expr::IdentifierReference(reference)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/hygiene/mod.rs#L68>.
    pub(crate) fn hygiene(&mut self, ast: &mut Ast<'_>) {
        hygiene::apply(ast, self);
    }
}
