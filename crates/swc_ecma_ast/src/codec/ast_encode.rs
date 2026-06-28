#![allow(unused)]
use crate::{
    codec::{AstEncode, NONE, NodeKind, PackedNode},
    *,
};
use swc_experimental_allocator::boxed::Box;
impl<'a, E: AstEncode> AstEncode for Box<'a, E> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        (**self).encode(encoder)
    }
}
impl<E: AstEncode> AstEncode for Option<E> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Some(it) => it.encode(encoder),
            None => NONE,
        }
    }
}
impl<'a> AstEncode for Program<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Module(it) => it.encode(encoder),
            Self::Script(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for Module<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__body_start, __body_len) = encoder.nodes(&self.body);
        let (__shebang_start, __shebang_len) = encoder.opt_utf8(self.shebang);
        let field0 = __body_start;
        let field1 = __body_len;
        let field2 = __shebang_start;
        let field3 = __shebang_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Module,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Script<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__body_start, __body_len) = encoder.nodes(&self.body);
        let (__shebang_start, __shebang_len) = encoder.opt_utf8(self.shebang);
        let field0 = __body_start;
        let field1 = __body_len;
        let field2 = __shebang_start;
        let field3 = __shebang_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Script,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ModuleItem<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::ModuleDecl(it) => it.encode(encoder),
            Self::Stmt(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for ModuleDecl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Import(it) => it.encode(encoder),
            Self::ExportDecl(it) => it.encode(encoder),
            Self::ExportNamed(it) => it.encode(encoder),
            Self::ExportDefaultDecl(it) => it.encode(encoder),
            Self::ExportDefaultExpr(it) => it.encode(encoder),
            Self::ExportAll(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for ImportDecl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__specifiers_start, __specifiers_len) = encoder.nodes(&self.specifiers);
        let field0 = __specifiers_start;
        let field1 = __specifiers_len;
        let field2 = self.src.encode(encoder);
        let field3 = match &self.with {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = if self.type_only { 1u16 << 0u16 } else { 0 } | (self.phase as u16) << 1u16;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ImportDecl,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ImportSpecifier<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Named(it) => it.encode(encoder),
            Self::Default(it) => it.encode(encoder),
            Self::Namespace(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for ImportNamedSpecifier<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.local.encode(encoder);
        let field1 = self.imported.encode(encoder);
        let flags = if self.is_type_only { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ImportNamedSpecifier,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ImportDefaultSpecifier<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.local.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ImportDefaultSpecifier,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ImportStarAsSpecifier<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.local.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ImportStarAsSpecifier,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ExportDecl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.decl.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ExportDecl,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for NamedExport<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__specifiers_start, __specifiers_len) = encoder.nodes(&self.specifiers);
        let field0 = __specifiers_start;
        let field1 = __specifiers_len;
        let field2 = match &self.src {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field3 = match &self.with {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = if self.type_only { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::NamedExport,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ExportSpecifier<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Namespace(it) => it.encode(encoder),
            Self::Default(it) => it.encode(encoder),
            Self::Named(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for ExportNamespaceSpecifier<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.name.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ExportNamespaceSpecifier,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ModuleExportName<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Ident(it) => it.encode(encoder),
            Self::Str(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for ExportDefaultSpecifier<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.exported.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ExportDefaultSpecifier,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ExportNamedSpecifier<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.orig.encode(encoder);
        let field1 = self.exported.encode(encoder);
        let flags = if self.is_type_only { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ExportNamedSpecifier,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ExportDefaultDecl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.decl.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ExportDefaultDecl,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for DefaultDecl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Class(it) => it.encode(encoder),
            Self::Fn(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for ExportDefaultExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.expr.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ExportDefaultExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ExportAll<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.src.encode(encoder);
        let field1 = match &self.with {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = if self.type_only { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ExportAll,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for BlockStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__stmts_start, __stmts_len) = encoder.nodes(&self.stmts);
        let field0 = __stmts_start;
        let field1 = __stmts_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::BlockStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Stmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Block(it) => it.encode(encoder),
            Self::Empty(it) => it.encode(encoder),
            Self::Debugger(it) => it.encode(encoder),
            Self::With(it) => it.encode(encoder),
            Self::Return(it) => it.encode(encoder),
            Self::Labeled(it) => it.encode(encoder),
            Self::Break(it) => it.encode(encoder),
            Self::Continue(it) => it.encode(encoder),
            Self::If(it) => it.encode(encoder),
            Self::Switch(it) => it.encode(encoder),
            Self::Throw(it) => it.encode(encoder),
            Self::Try(it) => it.encode(encoder),
            Self::While(it) => it.encode(encoder),
            Self::DoWhile(it) => it.encode(encoder),
            Self::For(it) => it.encode(encoder),
            Self::ForIn(it) => it.encode(encoder),
            Self::ForOf(it) => it.encode(encoder),
            Self::Decl(it) => it.encode(encoder),
            Self::Expr(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for ExprStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.expr.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ExprStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl AstEncode for EmptyStmt {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::EmptyStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
impl AstEncode for DebuggerStmt {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::DebuggerStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for WithStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.obj.encode(encoder);
        let field1 = self.body.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::WithStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ReturnStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = match &self.arg {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ReturnStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for LabeledStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.label.encode(encoder);
        let field1 = self.body.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::LabeledStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for BreakStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = match &self.label {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::BreakStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ContinueStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = match &self.label {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ContinueStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for IfStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.test.encode(encoder);
        let field1 = self.cons.encode(encoder);
        let field2 = match &self.alt {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::IfStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for SwitchStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__cases_start, __cases_len) = encoder.nodes(&self.cases);
        let field0 = self.discriminant.encode(encoder);
        let field1 = __cases_start;
        let field2 = __cases_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::SwitchStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ThrowStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.arg.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ThrowStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for TryStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.block.encode(encoder);
        let field1 = match &self.handler {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field2 = match &self.finalizer {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::TryStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for WhileStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.test.encode(encoder);
        let field1 = self.body.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::WhileStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for DoWhileStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.test.encode(encoder);
        let field1 = self.body.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::DoWhileStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ForStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = match &self.init {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field1 = match &self.test {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field2 = match &self.update {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field3 = self.body.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ForStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ForInStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.left.encode(encoder);
        let field1 = self.right.encode(encoder);
        let field2 = self.body.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ForInStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ForOfStmt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.left.encode(encoder);
        let field1 = self.right.encode(encoder);
        let field2 = self.body.encode(encoder);
        let flags = if self.is_await { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ForOfStmt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for SwitchCase<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__cons_start, __cons_len) = encoder.nodes(&self.cons);
        let field0 = match &self.test {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field1 = __cons_start;
        let field2 = __cons_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::SwitchCase,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for CatchClause<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = match &self.param {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field1 = self.body.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::CatchClause,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ForHead<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::VarDecl(it) => it.encode(encoder),
            Self::UsingDecl(it) => it.encode(encoder),
            Self::Pat(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for VarDeclOrExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::VarDecl(it) => it.encode(encoder),
            Self::Expr(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for Decl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Class(it) => it.encode(encoder),
            Self::Fn(it) => it.encode(encoder),
            Self::Var(it) => it.encode(encoder),
            Self::Using(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for FnDecl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.ident.encode(encoder);
        let field1 = self.function.encode(encoder);
        let flags = if self.declare { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::FnDecl,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ClassDecl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.ident.encode(encoder);
        let field1 = self.class.encode(encoder);
        let flags = if self.declare { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ClassDecl,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for VarDecl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__decls_start, __decls_len) = encoder.nodes(&self.decls);
        let field0 = __decls_start;
        let field1 = __decls_len;
        let flags = (self.kind as u16) << 0u16 | if self.declare { 1u16 << 2u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::VarDecl,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for VarDeclarator<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.name.encode(encoder);
        let field1 = match &self.init {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::VarDeclarator,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for UsingDecl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__decls_start, __decls_len) = encoder.nodes(&self.decls);
        let field0 = __decls_start;
        let field1 = __decls_len;
        let flags = if self.is_await { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::UsingDecl,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Expr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::This(it) => it.encode(encoder),
            Self::Array(it) => it.encode(encoder),
            Self::Object(it) => it.encode(encoder),
            Self::Fn(it) => it.encode(encoder),
            Self::Unary(it) => it.encode(encoder),
            Self::Update(it) => it.encode(encoder),
            Self::Bin(it) => it.encode(encoder),
            Self::Assign(it) => it.encode(encoder),
            Self::Member(it) => it.encode(encoder),
            Self::SuperProp(it) => it.encode(encoder),
            Self::Cond(it) => it.encode(encoder),
            Self::Import(it) => it.encode(encoder),
            Self::Call(it) => it.encode(encoder),
            Self::New(it) => it.encode(encoder),
            Self::Seq(it) => it.encode(encoder),
            Self::Ident(it) => it.encode(encoder),
            Self::Lit(it) => it.encode(encoder),
            Self::Tpl(it) => it.encode(encoder),
            Self::TaggedTpl(it) => it.encode(encoder),
            Self::Arrow(it) => it.encode(encoder),
            Self::Class(it) => it.encode(encoder),
            Self::Yield(it) => it.encode(encoder),
            Self::MetaProp(it) => it.encode(encoder),
            Self::Await(it) => it.encode(encoder),
            Self::Paren(it) => it.encode(encoder),
            Self::JSXMember(it) => it.encode(encoder),
            Self::JSXNamespacedName(it) => it.encode(encoder),
            Self::JSXEmpty(it) => it.encode(encoder),
            Self::JSXElement(it) => it.encode(encoder),
            Self::JSXFragment(it) => it.encode(encoder),
            Self::PrivateName(it) => it.encode(encoder),
            Self::OptChain(it) => it.encode(encoder),
            Self::Invalid(it) => it.encode(encoder),
        }
    }
}
impl AstEncode for ThisExpr {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ThisExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ArrayLit<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__elems_start, __elems_len) = encoder.nodes(&self.elems);
        let field0 = __elems_start;
        let field1 = __elems_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ArrayLit,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ObjectLit<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__props_start, __props_len) = encoder.nodes(&self.props);
        let field0 = __props_start;
        let field1 = __props_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ObjectLit,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for PropOrSpread<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Spread(it) => it.encode(encoder),
            Self::Prop(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for SpreadElement<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.expr.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::SpreadElement,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for UnaryExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.arg.encode(encoder);
        let flags = (self.op as u16) << 0u16;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::UnaryExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for UpdateExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.arg.encode(encoder);
        let flags = (self.op as u16) << 0u16 | if self.prefix { 1u16 << 1u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::UpdateExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for BinExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.left.encode(encoder);
        let field1 = self.right.encode(encoder);
        let flags = (self.op as u16) << 0u16;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::BinExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for FnExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = match &self.ident {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field1 = self.function.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::FnExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ClassExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = match &self.ident {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field1 = self.class.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ClassExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for AssignExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.left.encode(encoder);
        let field1 = self.right.encode(encoder);
        let flags = (self.op as u16) << 0u16;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::AssignExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for MemberExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.obj.encode(encoder);
        let field1 = self.prop.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::MemberExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for MemberProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Ident(it) => it.encode(encoder),
            Self::PrivateName(it) => it.encode(encoder),
            Self::Computed(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for SuperPropExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.obj.encode(encoder);
        let field1 = self.prop.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::SuperPropExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for SuperProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Ident(it) => it.encode(encoder),
            Self::Computed(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for CondExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.test.encode(encoder);
        let field1 = self.cons.encode(encoder);
        let field2 = self.alt.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::CondExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ImportExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.source.encode(encoder);
        let field1 = match &self.options {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = (self.phase as u16) << 0u16;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ImportExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for CallExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__args_start, __args_len) = encoder.nodes(&self.args);
        let field0 = self.callee.encode(encoder);
        let field1 = __args_start;
        let field2 = __args_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::CallExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for NewExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__args_start, __args_len) = encoder.nodes(&self.args);
        let field0 = self.callee.encode(encoder);
        let field1 = __args_start;
        let field2 = __args_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::NewExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for SeqExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__exprs_start, __exprs_len) = encoder.nodes(&self.exprs);
        let field0 = __exprs_start;
        let field1 = __exprs_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::SeqExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ArrowExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__params_start, __params_len) = encoder.nodes(&self.params);
        let field0 = __params_start;
        let field1 = __params_len;
        let field2 = self.body.encode(encoder);
        let flags = if self.is_async { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ArrowExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for YieldExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = match &self.arg {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = if self.delegate { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::YieldExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl AstEncode for MetaPropExpr {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = (self.kind as u16) << 0u16;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::MetaPropExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for AwaitExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.arg.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::AwaitExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Tpl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__exprs_start, __exprs_len) = encoder.nodes(&self.exprs);
        let (__quasis_start, __quasis_len) = encoder.nodes(&self.quasis);
        let field0 = __exprs_start;
        let field1 = __exprs_len;
        let field2 = __quasis_start;
        let field3 = __quasis_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Tpl,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for TaggedTpl<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.tag.encode(encoder);
        let field1 = self.tpl.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::TaggedTpl,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for TplElement<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__cooked_start, __cooked_len) = encoder.opt_wtf8(self.cooked);
        let (__raw_start, __raw_len) = encoder.utf8(self.raw);
        let field0 = __cooked_start;
        let field1 = __cooked_len;
        let field2 = __raw_start;
        let field3 = __raw_len;
        let flags = if self.tail { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::TplElement,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ParenExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.expr.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ParenExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Callee<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Super(it) => it.encode(encoder),
            Self::Expr(it) => it.encode(encoder),
        }
    }
}
impl AstEncode for Super {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Super,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ExprOrSpread<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__spread_start, __spread_end) = encoder.opt_span(self.spread);
        let field0 = __spread_start;
        let field1 = __spread_end;
        let field2 = self.expr.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ExprOrSpread,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for BlockStmtOrExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::BlockStmt(it) => it.encode(encoder),
            Self::Expr(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for AssignTarget<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Simple(it) => it.encode(encoder),
            Self::Pat(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for AssignTargetPat<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Array(it) => it.encode(encoder),
            Self::Object(it) => it.encode(encoder),
            Self::Invalid(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for SimpleAssignTarget<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Ident(it) => it.encode(encoder),
            Self::Member(it) => it.encode(encoder),
            Self::SuperProp(it) => it.encode(encoder),
            Self::Paren(it) => it.encode(encoder),
            Self::OptChain(it) => it.encode(encoder),
            Self::Invalid(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for OptChainExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.base.encode(encoder);
        let flags = if self.optional { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::OptChainExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for OptChainBase<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Member(it) => it.encode(encoder),
            Self::Call(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for OptCall<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__args_start, __args_len) = encoder.nodes(&self.args);
        let field0 = self.callee.encode(encoder);
        let field1 = __args_start;
        let field2 = __args_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::OptCall,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl AstEncode for Invalid {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Invalid,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Function<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__params_start, __params_len) = encoder.nodes(&self.params);
        let (__decorators_start, __decorators_len) = encoder.nodes(&self.decorators);
        let field0 = __params_start;
        let field1 = __params_len;
        let field2 = __decorators_start;
        let field3 = __decorators_len;
        let field4 = self.body.encode(encoder);
        let flags = if self.is_generator { 1u16 << 0u16 } else { 0 }
            | if self.is_async { 1u16 << 1u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Function,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            field4,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Param<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__decorators_start, __decorators_len) = encoder.nodes(&self.decorators);
        let field0 = __decorators_start;
        let field1 = __decorators_len;
        let field2 = self.pat.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Param,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ParamOrTsParamProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Param(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for Class<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__decorators_start, __decorators_len) = encoder.nodes(&self.decorators);
        let (__body_start, __body_len) = encoder.nodes(&self.body);
        let field0 = __decorators_start;
        let field1 = __decorators_len;
        let field2 = __body_start;
        let field3 = __body_len;
        let field4 = match &self.super_class {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = if self.is_abstract { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Class,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            field4,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ClassMember<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Constructor(it) => it.encode(encoder),
            Self::Method(it) => it.encode(encoder),
            Self::PrivateMethod(it) => it.encode(encoder),
            Self::ClassProp(it) => it.encode(encoder),
            Self::PrivateProp(it) => it.encode(encoder),
            Self::Empty(it) => it.encode(encoder),
            Self::StaticBlock(it) => it.encode(encoder),
            Self::AutoAccessor(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for ClassProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__decorators_start, __decorators_len) = encoder.nodes(&self.decorators);
        let field0 = self.key.encode(encoder);
        let field1 = match &self.value {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field2 = __decorators_start;
        let field3 = __decorators_len;
        let flags = if self.is_static { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ClassProp,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for PrivateProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__decorators_start, __decorators_len) = encoder.nodes(&self.decorators);
        let field0 = self.key.encode(encoder);
        let field1 = match &self.value {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field2 = __decorators_start;
        let field3 = __decorators_len;
        let flags = if self.is_static { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::PrivateProp,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ClassMethod<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.key.encode(encoder);
        let field1 = self.function.encode(encoder);
        let flags = (self.kind as u16) << 0u16 | if self.is_static { 1u16 << 2u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ClassMethod,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for PrivateMethod<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.key.encode(encoder);
        let field1 = self.function.encode(encoder);
        let flags = (self.kind as u16) << 0u16 | if self.is_static { 1u16 << 2u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::PrivateMethod,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Constructor<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__params_start, __params_len) = encoder.nodes(&self.params);
        let field0 = self.key.encode(encoder);
        let field1 = __params_start;
        let field2 = __params_len;
        let field3 = match &self.body {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Constructor,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Decorator<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.expr.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Decorator,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for StaticBlock<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.body.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::StaticBlock,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Key<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Private(it) => it.encode(encoder),
            Self::Public(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for AutoAccessor<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__decorators_start, __decorators_len) = encoder.nodes(&self.decorators);
        let field0 = self.key.encode(encoder);
        let field1 = match &self.value {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field2 = __decorators_start;
        let field3 = __decorators_len;
        let flags = if self.is_static { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::AutoAccessor,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Prop<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Shorthand(it) => it.encode(encoder),
            Self::KeyValue(it) => it.encode(encoder),
            Self::Assign(it) => it.encode(encoder),
            Self::Getter(it) => it.encode(encoder),
            Self::Setter(it) => it.encode(encoder),
            Self::Method(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for KeyValueProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.key.encode(encoder);
        let field1 = self.value.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::KeyValueProp,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for AssignProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.key.encode(encoder);
        let field1 = self.value.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::AssignProp,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for GetterProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.key.encode(encoder);
        let field1 = match &self.body {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::GetterProp,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for SetterProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.key.encode(encoder);
        let field1 = match &self.this_param {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let field2 = self.param.encode(encoder);
        let field3 = match &self.body {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::SetterProp,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for MethodProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.key.encode(encoder);
        let field1 = self.function.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::MethodProp,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for PropName<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Ident(it) => it.encode(encoder),
            Self::Str(it) => it.encode(encoder),
            Self::Num(it) => it.encode(encoder),
            Self::Computed(it) => it.encode(encoder),
            Self::BigInt(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for ComputedPropName<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.expr.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ComputedPropName,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Pat<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Ident(it) => it.encode(encoder),
            Self::Array(it) => it.encode(encoder),
            Self::Rest(it) => it.encode(encoder),
            Self::Object(it) => it.encode(encoder),
            Self::Assign(it) => it.encode(encoder),
            Self::Invalid(it) => it.encode(encoder),
            Self::Expr(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for ArrayPat<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__elems_start, __elems_len) = encoder.nodes(&self.elems);
        let field0 = __elems_start;
        let field1 = __elems_len;
        let flags = if self.optional { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ArrayPat,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ObjectPat<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__props_start, __props_len) = encoder.nodes(&self.props);
        let field0 = __props_start;
        let field1 = __props_len;
        let flags = if self.optional { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::ObjectPat,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for AssignPat<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.left.encode(encoder);
        let field1 = self.right.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::AssignPat,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for RestPat<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.arg.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::RestPat,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for ObjectPatProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::KeyValue(it) => it.encode(encoder),
            Self::Assign(it) => it.encode(encoder),
            Self::Rest(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for KeyValuePatProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.key.encode(encoder);
        let field1 = self.value.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::KeyValuePatProp,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for AssignPatProp<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.key.encode(encoder);
        let field1 = match &self.value {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::AssignPatProp,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Ident<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__sym_start, __sym_len) = encoder.utf8(self.sym);
        let field0 = __sym_start;
        let field1 = __sym_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Ident,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for IdentName<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__sym_start, __sym_len) = encoder.utf8(self.sym);
        let field0 = __sym_start;
        let field1 = __sym_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::IdentName,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for PrivateName<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__name_start, __name_len) = encoder.utf8(self.name);
        let field0 = __name_start;
        let field1 = __name_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::PrivateName,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for BindingIdent<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.id.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::BindingIdent,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Lit<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Str(it) => it.encode(encoder),
            Self::Bool(it) => it.encode(encoder),
            Self::Null(it) => it.encode(encoder),
            Self::Num(it) => it.encode(encoder),
            Self::BigInt(it) => it.encode(encoder),
            Self::Regex(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for Str<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__value_start, __value_len) = encoder.wtf8(self.value);
        let (__raw_start, __raw_len) = encoder.opt_utf8(self.raw);
        let field0 = __value_start;
        let field1 = __value_len;
        let field2 = __raw_start;
        let field3 = __raw_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Str,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl AstEncode for Bool {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = if self.value { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Bool,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
impl AstEncode for Null {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Null,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Number<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__raw_start, __raw_len) = encoder.opt_utf8(self.raw);
        let field0 = self.value.to_bits() as u32;
        let field1 = (self.value.to_bits() >> 32) as u32;
        let field2 = __raw_start;
        let field3 = __raw_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Number,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for BigInt<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__value_start, __value_len) = encoder.utf8(self.value);
        let (__raw_start, __raw_len) = encoder.opt_utf8(self.raw);
        let field0 = __value_start;
        let field1 = __value_len;
        let field2 = __raw_start;
        let field3 = __raw_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::BigInt,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for Regex<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__exp_start, __exp_len) = encoder.utf8(self.exp);
        let (__flags_start, __flags_len) = encoder.utf8(self.flags);
        let field0 = __exp_start;
        let field1 = __exp_len;
        let field2 = __flags_start;
        let field3 = __flags_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::Regex,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for JSXObject<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::JSXMemberExpr(it) => it.encode(encoder),
            Self::Ident(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for JSXMemberExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.obj.encode(encoder);
        let field1 = self.prop.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXMemberExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for JSXNamespacedName<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.ns.encode(encoder);
        let field1 = self.name.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXNamespacedName,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl AstEncode for JSXEmptyExpr {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXEmptyExpr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for JSXExprContainer<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.expr.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXExprContainer,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for JSXExpr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::JSXEmptyExpr(it) => it.encode(encoder),
            Self::Expr(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for JSXSpreadChild<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.expr.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXSpreadChild,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for JSXElementName<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Ident(it) => it.encode(encoder),
            Self::JSXMemberExpr(it) => it.encode(encoder),
            Self::JSXNamespacedName(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for JSXOpeningElement<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__attrs_start, __attrs_len) = encoder.nodes(&self.attrs);
        let field0 = self.name.encode(encoder);
        let field1 = __attrs_start;
        let field2 = __attrs_len;
        let flags = if self.self_closing { 1u16 << 0u16 } else { 0 };
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXOpeningElement,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for JSXAttrOrSpread<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::JSXAttr(it) => it.encode(encoder),
            Self::SpreadElement(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for JSXClosingElement<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.name.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXClosingElement,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for JSXAttr<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let field0 = self.name.encode(encoder);
        let field1 = match &self.value {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXAttr,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for JSXAttrName<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Ident(it) => it.encode(encoder),
            Self::JSXNamespacedName(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for JSXAttrValue<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::Str(it) => it.encode(encoder),
            Self::JSXExprContainer(it) => it.encode(encoder),
            Self::JSXElement(it) => it.encode(encoder),
            Self::JSXFragment(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for JSXText<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__value_start, __value_len) = encoder.utf8(self.value);
        let (__raw_start, __raw_len) = encoder.utf8(self.raw);
        let field0 = __value_start;
        let field1 = __value_len;
        let field2 = __raw_start;
        let field3 = __raw_len;
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXText,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for JSXElement<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__children_start, __children_len) = encoder.nodes(&self.children);
        let field0 = self.opening.encode(encoder);
        let field1 = __children_start;
        let field2 = __children_len;
        let field3 = match &self.closing {
            Some(it) => it.encode(encoder),
            None => NONE,
        };
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXElement,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl<'a> AstEncode for JSXElementChild<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Self::JSXText(it) => it.encode(encoder),
            Self::JSXExprContainer(it) => it.encode(encoder),
            Self::JSXSpreadChild(it) => it.encode(encoder),
            Self::JSXElement(it) => it.encode(encoder),
            Self::JSXFragment(it) => it.encode(encoder),
        }
    }
}
impl<'a> AstEncode for JSXFragment<'a> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (__children_start, __children_len) = encoder.nodes(&self.children);
        let field0 = self.opening.encode(encoder);
        let field1 = __children_start;
        let field2 = __children_len;
        let field3 = self.closing.encode(encoder);
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXFragment,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}
impl AstEncode for JSXOpeningFragment {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXOpeningFragment,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
impl AstEncode for JSXClosingFragment {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let flags = 0;
        let (span_lo, span_hi) = encoder.span(self.span());
        encoder.packed_node(PackedNode {
            kind: NodeKind::JSXClosingFragment,
            flags,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            ..Default::default()
        })
    }
}
