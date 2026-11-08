use swc_next_allocator::Allocator;

use crate::{Ast, Comment, Expr, Lang, Program, SourceType, Stmt};

/// Owns an AST behind an explicit unsafe access boundary.
///
/// Root handles and metadata can be read safely. Accessing node-specific data
/// still requires an unsafe borrow of the underlying [`Ast`].
///
/// Use [`ast()`](Self::ast) for custom inspection and
/// [`ast_mut()`](Self::ast_mut) for custom edits. The safety requirements on
/// [`Ast`] apply to every subsequent use of the returned AST and its handles,
/// including calls to safe methods. A handle obtained before an edit may no
/// longer be valid afterward.
///
/// This type does not provide interior mutability. Mutable access requires an
/// exclusive borrow of the context.
pub struct AstCtx<'a> {
    ast: Ast<'a>,
}

impl<'a> AstCtx<'a> {
    /// Creates an empty AST backed by `source` and `allocator`.
    pub fn new(source: &'a str, allocator: &'a Allocator) -> Self {
        // SAFETY: The AST remains private until an unsafe accessor is used.
        Self {
            ast: unsafe { Ast::new(source, allocator) },
        }
    }

    /// Returns the AST's root program.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not a program.
    #[inline]
    pub fn root_program(&self) -> Program {
        self.ast.root_program()
    }

    /// Returns the AST's root expression.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not an expression.
    #[inline]
    pub fn root_expression(&self) -> Expr {
        self.ast.root_expression()
    }

    /// Returns the AST's root statement.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not a statement.
    #[inline]
    pub fn root_statement(&self) -> Stmt {
        self.ast.root_statement()
    }

    /// Returns the original source text.
    pub fn source(&self) -> &'a str {
        self.ast.source()
    }

    /// Returns the source's parsing mode.
    pub fn source_type(&self) -> SourceType {
        self.ast.source_type()
    }

    /// Returns the source language.
    pub fn lang(&self) -> Lang {
        self.ast.lang()
    }

    /// Returns whether the source language includes TypeScript.
    pub fn is_ts(&self) -> bool {
        self.ast.is_ts()
    }

    /// Returns whether the source language includes JSX.
    pub fn is_jsx(&self) -> bool {
        self.ast.is_jsx()
    }

    /// Returns whether the AST is a module.
    pub fn is_module(&self) -> bool {
        self.ast.is_module()
    }

    /// Returns attached comments in source order.
    pub fn comments(&self) -> &[Comment] {
        self.ast.comments()
    }

    /// Returns the number of stored nodes.
    pub fn node_count(&self) -> usize {
        self.ast.node_count()
    }

    /// Returns the number of out-of-line field entries.
    pub fn extra_data_count(&self) -> usize {
        self.ast.extra_data_count()
    }

    /// Borrows the underlying AST.
    ///
    /// # Safety
    ///
    /// The caller must uphold the handle invariants documented on [`Ast`] for
    /// all uses of the returned reference and any handles obtained through it.
    /// In particular, handles from different ASTs must not be mixed, and handles
    /// invalidated by mutation must not be reused, even if their indices are
    /// allocated again.
    #[inline]
    pub unsafe fn ast(&self) -> &Ast<'a> {
        &self.ast
    }

    /// Mutably borrows the underlying AST.
    ///
    /// # Safety
    ///
    /// The caller must uphold the handle invariants documented on [`Ast`] for
    /// all uses of the returned reference and its handles. All mutations must
    /// preserve those invariants for subsequent accesses, including after a
    /// panic unwinds. An AST must not acquire handles from another AST.
    #[inline]
    pub unsafe fn ast_mut(&mut self) -> &mut Ast<'a> {
        &mut self.ast
    }

    /// Consumes the ctx and returns its underlying AST.
    ///
    /// # Safety
    ///
    /// The caller must uphold the handle invariants documented on [`Ast`]
    /// throughout all subsequent uses of the returned AST and its handles.
    #[inline]
    pub unsafe fn into_ast(self) -> Ast<'a> {
        self.ast
    }
}

impl<'a> From<Ast<'a>> for AstCtx<'a> {
    fn from(ast: Ast<'a>) -> Self {
        Self { ast }
    }
}
