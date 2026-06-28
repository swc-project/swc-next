#![allow(unused)]
use crate::{
    codec::{AstDecode, NONE, NodeKind, decode::Decoder},
    *,
};
impl<'a> AstDecode<'a> for Program<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::Module as u8 => {
                Self::Module(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Script as u8 => {
                Self::Script(AstDecode::decode(decoder, id))
            }
            _ => panic!(
                "unexpected node kind {} for {}",
                node.kind,
                stringify!(Program)
            ),
        }
    }
}
impl<'a> AstDecode<'a> for Module<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Module as u8);
        Self {
            span: decoder.span(node),
            body: decoder.nodes(node.field0, node.field1),
            shebang: decoder.opt_atom(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for Script<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Script as u8);
        Self {
            span: decoder.span(node),
            body: decoder.nodes(node.field0, node.field1),
            shebang: decoder.opt_atom(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for ModuleItem<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::ImportDecl as u8 => {
                Self::ModuleDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExportDecl as u8 => {
                Self::ModuleDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::NamedExport as u8 => {
                Self::ModuleDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExportDefaultDecl as u8 => {
                Self::ModuleDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExportDefaultExpr as u8 => {
                Self::ModuleDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExportAll as u8 => {
                Self::ModuleDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BlockStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::EmptyStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::DebuggerStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::WithStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ReturnStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::LabeledStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BreakStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ContinueStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::IfStmt as u8 => Self::Stmt(AstDecode::decode(decoder, id)),
            value if value == NodeKind::SwitchStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ThrowStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::TryStmt as u8 => Self::Stmt(AstDecode::decode(decoder, id)),
            value if value == NodeKind::WhileStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::DoWhileStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ForStmt as u8 => Self::Stmt(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ForInStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ForOfStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassDecl as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnDecl as u8 => Self::Stmt(AstDecode::decode(decoder, id)),
            value if value == NodeKind::VarDecl as u8 => Self::Stmt(AstDecode::decode(decoder, id)),
            value if value == NodeKind::UsingDecl as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExprStmt as u8 => {
                Self::Stmt(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(ModuleItem)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for ModuleDecl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::ImportDecl as u8 => {
                Self::Import(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExportDecl as u8 => {
                Self::ExportDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::NamedExport as u8 => {
                Self::ExportNamed(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExportDefaultDecl as u8 => {
                Self::ExportDefaultDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExportDefaultExpr as u8 => {
                Self::ExportDefaultExpr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExportAll as u8 => {
                Self::ExportAll(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(ModuleDecl)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for ImportDecl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ImportDecl as u8);
        Self {
            span: decoder.span(node),
            specifiers: decoder.nodes(node.field0, node.field1),
            src: AstDecode::decode(decoder, node.field2),
            type_only: (node.flags & (1u16 << 0u16)) != 0,
            with: AstDecode::decode(decoder, node.field3),
            phase: match (node.flags >> 1u16) & 3u16 {
                0u16 => ImportPhase::Evaluation,
                1u16 => ImportPhase::Source,
                2u16 => ImportPhase::Defer,
                value => panic!("invalid {} flag value {value}", stringify!(ImportPhase)),
            },
        }
    }
}
impl<'a> AstDecode<'a> for ImportSpecifier<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::ImportNamedSpecifier as u8 => {
                Self::Named(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ImportDefaultSpecifier as u8 => {
                Self::Default(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ImportStarAsSpecifier as u8 => {
                Self::Namespace(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(ImportSpecifier)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for ImportNamedSpecifier<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ImportNamedSpecifier as u8);
        Self {
            span: decoder.span(node),
            local: AstDecode::decode(decoder, node.field0),
            imported: AstDecode::decode(decoder, node.field1),
            is_type_only: (node.flags & (1u16 << 0u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for ImportDefaultSpecifier<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ImportDefaultSpecifier as u8);
        Self {
            span: decoder.span(node),
            local: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for ImportStarAsSpecifier<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ImportStarAsSpecifier as u8);
        Self {
            span: decoder.span(node),
            local: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for ExportDecl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ExportDecl as u8);
        Self {
            span: decoder.span(node),
            decl: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for NamedExport<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::NamedExport as u8);
        Self {
            span: decoder.span(node),
            specifiers: decoder.nodes(node.field0, node.field1),
            src: AstDecode::decode(decoder, node.field2),
            type_only: (node.flags & (1u16 << 0u16)) != 0,
            with: AstDecode::decode(decoder, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for ExportSpecifier<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::ExportNamespaceSpecifier as u8 => {
                Self::Namespace(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExportDefaultSpecifier as u8 => {
                Self::Default(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExportNamedSpecifier as u8 => {
                Self::Named(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(ExportSpecifier)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for ExportNamespaceSpecifier<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ExportNamespaceSpecifier as u8);
        Self {
            span: decoder.span(node),
            name: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for ModuleExportName<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::Ident as u8 => Self::Ident(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Str as u8 => Self::Str(AstDecode::decode(decoder, id)),
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(ModuleExportName)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for ExportDefaultSpecifier<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ExportDefaultSpecifier as u8);
        Self {
            exported: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for ExportNamedSpecifier<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ExportNamedSpecifier as u8);
        Self {
            span: decoder.span(node),
            orig: AstDecode::decode(decoder, node.field0),
            exported: AstDecode::decode(decoder, node.field1),
            is_type_only: (node.flags & (1u16 << 0u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for ExportDefaultDecl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ExportDefaultDecl as u8);
        Self {
            span: decoder.span(node),
            decl: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for DefaultDecl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::ClassExpr as u8 => {
                Self::Class(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnExpr as u8 => Self::Fn(AstDecode::decode(decoder, id)),
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(DefaultDecl)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for ExportDefaultExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ExportDefaultExpr as u8);
        Self {
            span: decoder.span(node),
            expr: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for ExportAll<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ExportAll as u8);
        Self {
            span: decoder.span(node),
            src: AstDecode::decode(decoder, node.field0),
            type_only: (node.flags & (1u16 << 0u16)) != 0,
            with: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for BlockStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::BlockStmt as u8);
        Self {
            span: decoder.span(node),
            stmts: decoder.nodes(node.field0, node.field1),
            scope_id: std::cell::Cell::new(None),
        }
    }
}
impl<'a> AstDecode<'a> for Stmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::BlockStmt as u8 => {
                Self::Block(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::EmptyStmt as u8 => {
                Self::Empty(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::DebuggerStmt as u8 => {
                Self::Debugger(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::WithStmt as u8 => {
                Self::With(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ReturnStmt as u8 => {
                Self::Return(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::LabeledStmt as u8 => {
                Self::Labeled(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BreakStmt as u8 => {
                Self::Break(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ContinueStmt as u8 => {
                Self::Continue(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::IfStmt as u8 => Self::If(AstDecode::decode(decoder, id)),
            value if value == NodeKind::SwitchStmt as u8 => {
                Self::Switch(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ThrowStmt as u8 => {
                Self::Throw(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::TryStmt as u8 => Self::Try(AstDecode::decode(decoder, id)),
            value if value == NodeKind::WhileStmt as u8 => {
                Self::While(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::DoWhileStmt as u8 => {
                Self::DoWhile(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ForStmt as u8 => Self::For(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ForInStmt as u8 => {
                Self::ForIn(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ForOfStmt as u8 => {
                Self::ForOf(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassDecl as u8 => {
                Self::Decl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnDecl as u8 => Self::Decl(AstDecode::decode(decoder, id)),
            value if value == NodeKind::VarDecl as u8 => Self::Decl(AstDecode::decode(decoder, id)),
            value if value == NodeKind::UsingDecl as u8 => {
                Self::Decl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ExprStmt as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            _ => panic!(
                "unexpected node kind {} for {}",
                node.kind,
                stringify!(Stmt)
            ),
        }
    }
}
impl<'a> AstDecode<'a> for ExprStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ExprStmt as u8);
        Self {
            span: decoder.span(node),
            expr: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for EmptyStmt {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::EmptyStmt as u8);
        Self {
            span: decoder.span(node),
        }
    }
}
impl<'a> AstDecode<'a> for DebuggerStmt {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::DebuggerStmt as u8);
        Self {
            span: decoder.span(node),
        }
    }
}
impl<'a> AstDecode<'a> for WithStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::WithStmt as u8);
        Self {
            span: decoder.span(node),
            obj: AstDecode::decode(decoder, node.field0),
            body: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for ReturnStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ReturnStmt as u8);
        Self {
            span: decoder.span(node),
            arg: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for LabeledStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::LabeledStmt as u8);
        Self {
            span: decoder.span(node),
            label: AstDecode::decode(decoder, node.field0),
            body: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for BreakStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::BreakStmt as u8);
        Self {
            span: decoder.span(node),
            label: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for ContinueStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ContinueStmt as u8);
        Self {
            span: decoder.span(node),
            label: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for IfStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::IfStmt as u8);
        Self {
            span: decoder.span(node),
            test: AstDecode::decode(decoder, node.field0),
            cons: AstDecode::decode(decoder, node.field1),
            alt: AstDecode::decode(decoder, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for SwitchStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::SwitchStmt as u8);
        Self {
            span: decoder.span(node),
            discriminant: AstDecode::decode(decoder, node.field0),
            cases: decoder.nodes(node.field1, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for ThrowStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ThrowStmt as u8);
        Self {
            span: decoder.span(node),
            arg: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for TryStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::TryStmt as u8);
        Self {
            span: decoder.span(node),
            block: AstDecode::decode(decoder, node.field0),
            handler: AstDecode::decode(decoder, node.field1),
            finalizer: AstDecode::decode(decoder, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for WhileStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::WhileStmt as u8);
        Self {
            span: decoder.span(node),
            test: AstDecode::decode(decoder, node.field0),
            body: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for DoWhileStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::DoWhileStmt as u8);
        Self {
            span: decoder.span(node),
            test: AstDecode::decode(decoder, node.field0),
            body: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for ForStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ForStmt as u8);
        Self {
            span: decoder.span(node),
            init: AstDecode::decode(decoder, node.field0),
            test: AstDecode::decode(decoder, node.field1),
            update: AstDecode::decode(decoder, node.field2),
            body: AstDecode::decode(decoder, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for ForInStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ForInStmt as u8);
        Self {
            span: decoder.span(node),
            left: AstDecode::decode(decoder, node.field0),
            right: AstDecode::decode(decoder, node.field1),
            body: AstDecode::decode(decoder, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for ForOfStmt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ForOfStmt as u8);
        Self {
            span: decoder.span(node),
            is_await: (node.flags & (1u16 << 0u16)) != 0,
            left: AstDecode::decode(decoder, node.field0),
            right: AstDecode::decode(decoder, node.field1),
            body: AstDecode::decode(decoder, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for SwitchCase<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::SwitchCase as u8);
        Self {
            span: decoder.span(node),
            test: AstDecode::decode(decoder, node.field0),
            cons: decoder.nodes(node.field1, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for CatchClause<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::CatchClause as u8);
        Self {
            span: decoder.span(node),
            param: AstDecode::decode(decoder, node.field0),
            body: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for ForHead<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::VarDecl as u8 => {
                Self::VarDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::UsingDecl as u8 => {
                Self::UsingDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BindingIdent as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrayPat as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::RestPat as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ObjectPat as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AssignPat as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ThisExpr as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ArrayLit as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ObjectLit as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnExpr as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::UnaryExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::UpdateExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BinExpr as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::AssignExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MemberExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SuperPropExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CondExpr as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ImportExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CallExpr as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::NewExpr as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::SeqExpr as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Ident as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Str as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Bool as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Null as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Number as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::BigInt as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Regex as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Tpl as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::TaggedTpl as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrowExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::YieldExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MetaPropExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AwaitExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ParenExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXMemberExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXNamespacedName as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXEmptyExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXElement as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXFragment as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::PrivateName as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::OptChainExpr as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            _ => panic!(
                "unexpected node kind {} for {}",
                node.kind,
                stringify!(ForHead)
            ),
        }
    }
}
impl<'a> AstDecode<'a> for VarDeclOrExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::VarDecl as u8 => {
                Self::VarDecl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ThisExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrayLit as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ObjectLit as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::UnaryExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::UpdateExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BinExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::AssignExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MemberExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SuperPropExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CondExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ImportExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CallExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::NewExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::SeqExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Ident as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Str as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Bool as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Null as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Number as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::BigInt as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Regex as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Tpl as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::TaggedTpl as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrowExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::YieldExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MetaPropExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AwaitExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ParenExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXMemberExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXNamespacedName as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXEmptyExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXElement as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXFragment as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::PrivateName as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::OptChainExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(VarDeclOrExpr)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for Decl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::ClassDecl as u8 => {
                Self::Class(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnDecl as u8 => Self::Fn(AstDecode::decode(decoder, id)),
            value if value == NodeKind::VarDecl as u8 => Self::Var(AstDecode::decode(decoder, id)),
            value if value == NodeKind::UsingDecl as u8 => {
                Self::Using(AstDecode::decode(decoder, id))
            }
            _ => panic!(
                "unexpected node kind {} for {}",
                node.kind,
                stringify!(Decl)
            ),
        }
    }
}
impl<'a> AstDecode<'a> for FnDecl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::FnDecl as u8);
        Self {
            ident: AstDecode::decode(decoder, node.field0),
            declare: (node.flags & (1u16 << 0u16)) != 0,
            function: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for ClassDecl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ClassDecl as u8);
        Self {
            ident: AstDecode::decode(decoder, node.field0),
            declare: (node.flags & (1u16 << 0u16)) != 0,
            class: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for VarDecl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::VarDecl as u8);
        Self {
            span: decoder.span(node),
            kind: match (node.flags >> 0u16) & 3u16 {
                0u16 => VarDeclKind::Var,
                1u16 => VarDeclKind::Let,
                2u16 => VarDeclKind::Const,
                value => panic!("invalid {} flag value {value}", stringify!(VarDeclKind)),
            },
            declare: (node.flags & (1u16 << 2u16)) != 0,
            decls: decoder.nodes(node.field0, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for VarDeclarator<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::VarDeclarator as u8);
        Self {
            span: decoder.span(node),
            name: AstDecode::decode(decoder, node.field0),
            init: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for UsingDecl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::UsingDecl as u8);
        Self {
            span: decoder.span(node),
            is_await: (node.flags & (1u16 << 0u16)) != 0,
            decls: decoder.nodes(node.field0, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for Expr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::ThisExpr as u8 => {
                Self::This(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrayLit as u8 => {
                Self::Array(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ObjectLit as u8 => {
                Self::Object(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnExpr as u8 => Self::Fn(AstDecode::decode(decoder, id)),
            value if value == NodeKind::UnaryExpr as u8 => {
                Self::Unary(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::UpdateExpr as u8 => {
                Self::Update(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BinExpr as u8 => Self::Bin(AstDecode::decode(decoder, id)),
            value if value == NodeKind::AssignExpr as u8 => {
                Self::Assign(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MemberExpr as u8 => {
                Self::Member(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SuperPropExpr as u8 => {
                Self::SuperProp(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CondExpr as u8 => {
                Self::Cond(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ImportExpr as u8 => {
                Self::Import(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CallExpr as u8 => {
                Self::Call(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::NewExpr as u8 => Self::New(AstDecode::decode(decoder, id)),
            value if value == NodeKind::SeqExpr as u8 => Self::Seq(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Ident as u8 => Self::Ident(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Str as u8 => Self::Lit(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Bool as u8 => Self::Lit(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Null as u8 => Self::Lit(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Number as u8 => Self::Lit(AstDecode::decode(decoder, id)),
            value if value == NodeKind::BigInt as u8 => Self::Lit(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Regex as u8 => Self::Lit(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Tpl as u8 => Self::Tpl(AstDecode::decode(decoder, id)),
            value if value == NodeKind::TaggedTpl as u8 => {
                Self::TaggedTpl(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrowExpr as u8 => {
                Self::Arrow(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassExpr as u8 => {
                Self::Class(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::YieldExpr as u8 => {
                Self::Yield(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MetaPropExpr as u8 => {
                Self::MetaProp(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AwaitExpr as u8 => {
                Self::Await(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ParenExpr as u8 => {
                Self::Paren(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXMemberExpr as u8 => {
                Self::JSXMember(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXNamespacedName as u8 => {
                Self::JSXNamespacedName(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXEmptyExpr as u8 => {
                Self::JSXEmpty(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXElement as u8 => {
                Self::JSXElement(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXFragment as u8 => {
                Self::JSXFragment(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::PrivateName as u8 => {
                Self::PrivateName(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::OptChainExpr as u8 => {
                Self::OptChain(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => {
                Self::Invalid(AstDecode::decode(decoder, id))
            }
            _ => panic!(
                "unexpected node kind {} for {}",
                node.kind,
                stringify!(Expr)
            ),
        }
    }
}
impl<'a> AstDecode<'a> for ThisExpr {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ThisExpr as u8);
        Self {
            span: decoder.span(node),
        }
    }
}
impl<'a> AstDecode<'a> for ArrayLit<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ArrayLit as u8);
        Self {
            span: decoder.span(node),
            elems: decoder.nodes(node.field0, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for ObjectLit<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ObjectLit as u8);
        Self {
            span: decoder.span(node),
            props: decoder.nodes(node.field0, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for PropOrSpread<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::SpreadElement as u8 => {
                Self::Spread(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Ident as u8 => Self::Prop(AstDecode::decode(decoder, id)),
            value if value == NodeKind::KeyValueProp as u8 => {
                Self::Prop(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AssignProp as u8 => {
                Self::Prop(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::GetterProp as u8 => {
                Self::Prop(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SetterProp as u8 => {
                Self::Prop(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MethodProp as u8 => {
                Self::Prop(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(PropOrSpread)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for SpreadElement<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::SpreadElement as u8);
        Self {
            dot3_token: decoder.span(node),
            expr: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for UnaryExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::UnaryExpr as u8);
        Self {
            span: decoder.span(node),
            op: match (node.flags >> 0u16) & 7u16 {
                0u16 => UnaryOp::Minus,
                1u16 => UnaryOp::Plus,
                2u16 => UnaryOp::Bang,
                3u16 => UnaryOp::Tilde,
                4u16 => UnaryOp::TypeOf,
                5u16 => UnaryOp::Void,
                6u16 => UnaryOp::Delete,
                value => panic!("invalid {} flag value {value}", stringify!(UnaryOp)),
            },
            arg: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for UpdateExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::UpdateExpr as u8);
        Self {
            span: decoder.span(node),
            op: match (node.flags >> 0u16) & 1u16 {
                0u16 => UpdateOp::PlusPlus,
                1u16 => UpdateOp::MinusMinus,
                value => panic!("invalid {} flag value {value}", stringify!(UpdateOp)),
            },
            prefix: (node.flags & (1u16 << 1u16)) != 0,
            arg: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for BinExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::BinExpr as u8);
        Self {
            span: decoder.span(node),
            op: match (node.flags >> 0u16) & 31u16 {
                0u16 => BinaryOp::EqEq,
                1u16 => BinaryOp::NotEq,
                2u16 => BinaryOp::EqEqEq,
                3u16 => BinaryOp::NotEqEq,
                4u16 => BinaryOp::Lt,
                5u16 => BinaryOp::LtEq,
                6u16 => BinaryOp::Gt,
                7u16 => BinaryOp::GtEq,
                8u16 => BinaryOp::LShift,
                9u16 => BinaryOp::RShift,
                10u16 => BinaryOp::ZeroFillRShift,
                11u16 => BinaryOp::Add,
                12u16 => BinaryOp::Sub,
                13u16 => BinaryOp::Mul,
                14u16 => BinaryOp::Div,
                15u16 => BinaryOp::Mod,
                16u16 => BinaryOp::BitOr,
                17u16 => BinaryOp::BitXor,
                18u16 => BinaryOp::BitAnd,
                19u16 => BinaryOp::LogicalOr,
                20u16 => BinaryOp::LogicalAnd,
                21u16 => BinaryOp::In,
                22u16 => BinaryOp::InstanceOf,
                23u16 => BinaryOp::Exp,
                24u16 => BinaryOp::NullishCoalescing,
                value => panic!("invalid {} flag value {value}", stringify!(BinaryOp)),
            },
            left: AstDecode::decode(decoder, node.field0),
            right: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for FnExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::FnExpr as u8);
        Self {
            ident: AstDecode::decode(decoder, node.field0),
            function: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for ClassExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ClassExpr as u8);
        Self {
            ident: AstDecode::decode(decoder, node.field0),
            class: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for AssignExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::AssignExpr as u8);
        Self {
            span: decoder.span(node),
            op: match (node.flags >> 0u16) & 31u16 {
                0u16 => AssignOp::Assign,
                1u16 => AssignOp::AddAssign,
                2u16 => AssignOp::SubAssign,
                3u16 => AssignOp::MulAssign,
                4u16 => AssignOp::DivAssign,
                5u16 => AssignOp::ModAssign,
                6u16 => AssignOp::LShiftAssign,
                7u16 => AssignOp::RShiftAssign,
                8u16 => AssignOp::ZeroFillRShiftAssign,
                9u16 => AssignOp::BitOrAssign,
                10u16 => AssignOp::BitXorAssign,
                11u16 => AssignOp::BitAndAssign,
                12u16 => AssignOp::ExpAssign,
                13u16 => AssignOp::AndAssign,
                14u16 => AssignOp::OrAssign,
                15u16 => AssignOp::NullishAssign,
                value => panic!("invalid {} flag value {value}", stringify!(AssignOp)),
            },
            left: AstDecode::decode(decoder, node.field0),
            right: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for MemberExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::MemberExpr as u8);
        Self {
            span: decoder.span(node),
            obj: AstDecode::decode(decoder, node.field0),
            prop: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for MemberProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::IdentName as u8 => {
                Self::Ident(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::PrivateName as u8 => {
                Self::PrivateName(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ComputedPropName as u8 => {
                Self::Computed(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(MemberProp)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for SuperPropExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::SuperPropExpr as u8);
        Self {
            span: decoder.span(node),
            obj: AstDecode::decode(decoder, node.field0),
            prop: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for SuperProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::IdentName as u8 => {
                Self::Ident(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ComputedPropName as u8 => {
                Self::Computed(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(SuperProp)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for CondExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::CondExpr as u8);
        Self {
            span: decoder.span(node),
            test: AstDecode::decode(decoder, node.field0),
            cons: AstDecode::decode(decoder, node.field1),
            alt: AstDecode::decode(decoder, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for ImportExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ImportExpr as u8);
        Self {
            span: decoder.span(node),
            source: AstDecode::decode(decoder, node.field0),
            options: AstDecode::decode(decoder, node.field1),
            phase: match (node.flags >> 0u16) & 3u16 {
                0u16 => ImportPhase::Evaluation,
                1u16 => ImportPhase::Source,
                2u16 => ImportPhase::Defer,
                value => panic!("invalid {} flag value {value}", stringify!(ImportPhase)),
            },
        }
    }
}
impl<'a> AstDecode<'a> for CallExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::CallExpr as u8);
        Self {
            span: decoder.span(node),
            callee: AstDecode::decode(decoder, node.field0),
            args: decoder.nodes(node.field1, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for NewExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::NewExpr as u8);
        Self {
            span: decoder.span(node),
            callee: AstDecode::decode(decoder, node.field0),
            args: decoder.nodes(node.field1, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for SeqExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::SeqExpr as u8);
        Self {
            span: decoder.span(node),
            exprs: decoder.nodes(node.field0, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for ArrowExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ArrowExpr as u8);
        Self {
            span: decoder.span(node),
            params: decoder.nodes(node.field0, node.field1),
            body: AstDecode::decode(decoder, node.field2),
            is_async: (node.flags & (1u16 << 0u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for YieldExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::YieldExpr as u8);
        Self {
            span: decoder.span(node),
            arg: AstDecode::decode(decoder, node.field0),
            delegate: (node.flags & (1u16 << 0u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for MetaPropExpr {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::MetaPropExpr as u8);
        Self {
            span: decoder.span(node),
            kind: match (node.flags >> 0u16) & 1u16 {
                0u16 => MetaPropKind::NewTarget,
                1u16 => MetaPropKind::ImportMeta,
                value => {
                    panic!("invalid {} flag value {value}", stringify!(MetaPropKind))
                }
            },
        }
    }
}
impl<'a> AstDecode<'a> for AwaitExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::AwaitExpr as u8);
        Self {
            span: decoder.span(node),
            arg: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for Tpl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Tpl as u8);
        Self {
            span: decoder.span(node),
            exprs: decoder.nodes(node.field0, node.field1),
            quasis: decoder.nodes(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for TaggedTpl<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::TaggedTpl as u8);
        Self {
            span: decoder.span(node),
            tag: AstDecode::decode(decoder, node.field0),
            tpl: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for TplElement<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::TplElement as u8);
        Self {
            span: decoder.span(node),
            tail: (node.flags & (1u16 << 0u16)) != 0,
            cooked: decoder.opt_wtf8_atom(node.field0, node.field1),
            raw: decoder.atom(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for ParenExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ParenExpr as u8);
        Self {
            span: decoder.span(node),
            expr: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for Callee<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::Super as u8 => Self::Super(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ThisExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrayLit as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ObjectLit as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::UnaryExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::UpdateExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BinExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::AssignExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MemberExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SuperPropExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CondExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ImportExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CallExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::NewExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::SeqExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Ident as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Str as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Bool as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Null as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Number as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::BigInt as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Regex as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Tpl as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::TaggedTpl as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrowExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::YieldExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MetaPropExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AwaitExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ParenExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXMemberExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXNamespacedName as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXEmptyExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXElement as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXFragment as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::PrivateName as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::OptChainExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            _ => panic!(
                "unexpected node kind {} for {}",
                node.kind,
                stringify!(Callee)
            ),
        }
    }
}
impl<'a> AstDecode<'a> for Super {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Super as u8);
        Self {
            span: decoder.span(node),
        }
    }
}
impl<'a> AstDecode<'a> for ExprOrSpread<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ExprOrSpread as u8);
        Self {
            spread: if node.field0 == NONE {
                None
            } else {
                Some(Span {
                    start: node.field0,
                    end: node.field1,
                })
            },
            expr: AstDecode::decode(decoder, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for BlockStmtOrExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::BlockStmt as u8 => {
                Self::BlockStmt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ThisExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrayLit as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ObjectLit as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::UnaryExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::UpdateExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BinExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::AssignExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MemberExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SuperPropExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CondExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ImportExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CallExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::NewExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::SeqExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Ident as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Str as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Bool as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Null as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Number as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::BigInt as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Regex as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Tpl as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::TaggedTpl as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrowExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::YieldExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MetaPropExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AwaitExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ParenExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXMemberExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXNamespacedName as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXEmptyExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXElement as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXFragment as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::PrivateName as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::OptChainExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(BlockStmtOrExpr)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for AssignTarget<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::BindingIdent as u8 => {
                Self::Simple(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MemberExpr as u8 => {
                Self::Simple(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SuperPropExpr as u8 => {
                Self::Simple(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ParenExpr as u8 => {
                Self::Simple(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::OptChainExpr as u8 => {
                Self::Simple(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => {
                Self::Simple(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrayPat as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ObjectPat as u8 => {
                Self::Pat(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => Self::Pat(AstDecode::decode(decoder, id)),
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(AssignTarget)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for AssignTargetPat<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::ArrayPat as u8 => {
                Self::Array(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ObjectPat as u8 => {
                Self::Object(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => {
                Self::Invalid(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(AssignTargetPat)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for SimpleAssignTarget<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::BindingIdent as u8 => {
                Self::Ident(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MemberExpr as u8 => {
                Self::Member(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SuperPropExpr as u8 => {
                Self::SuperProp(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ParenExpr as u8 => {
                Self::Paren(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::OptChainExpr as u8 => {
                Self::OptChain(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => {
                Self::Invalid(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(SimpleAssignTarget)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for OptChainExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::OptChainExpr as u8);
        Self {
            span: decoder.span(node),
            optional: (node.flags & (1u16 << 0u16)) != 0,
            base: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for OptChainBase<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::MemberExpr as u8 => {
                Self::Member(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::OptCall as u8 => Self::Call(AstDecode::decode(decoder, id)),
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(OptChainBase)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for OptCall<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::OptCall as u8);
        Self {
            span: decoder.span(node),
            callee: AstDecode::decode(decoder, node.field0),
            args: decoder.nodes(node.field1, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for Invalid {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Invalid as u8);
        Self {}
    }
}
impl<'a> AstDecode<'a> for Function<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Function as u8);
        Self {
            span: decoder.span(node),
            params: decoder.nodes(node.field0, node.field1),
            decorators: decoder.nodes(node.field2, node.field3),
            body: AstDecode::decode(decoder, node.field4),
            is_generator: (node.flags & (1u16 << 0u16)) != 0,
            is_async: (node.flags & (1u16 << 1u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for Param<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Param as u8);
        Self {
            span: decoder.span(node),
            decorators: decoder.nodes(node.field0, node.field1),
            pat: AstDecode::decode(decoder, node.field2),
        }
    }
}
impl<'a> AstDecode<'a> for ParamOrTsParamProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::Param as u8 => Self::Param(AstDecode::decode(decoder, id)),
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(ParamOrTsParamProp)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for Class<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Class as u8);
        Self {
            span: decoder.span(node),
            decorators: decoder.nodes(node.field0, node.field1),
            body: decoder.nodes(node.field2, node.field3),
            super_class: AstDecode::decode(decoder, node.field4),
            is_abstract: (node.flags & (1u16 << 0u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for ClassMember<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::Constructor as u8 => {
                Self::Constructor(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassMethod as u8 => {
                Self::Method(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::PrivateMethod as u8 => {
                Self::PrivateMethod(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassProp as u8 => {
                Self::ClassProp(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::PrivateProp as u8 => {
                Self::PrivateProp(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::EmptyStmt as u8 => {
                Self::Empty(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::StaticBlock as u8 => {
                Self::StaticBlock(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AutoAccessor as u8 => {
                Self::AutoAccessor(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(ClassMember)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for ClassProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ClassProp as u8);
        Self {
            span: decoder.span(node),
            key: AstDecode::decode(decoder, node.field0),
            value: AstDecode::decode(decoder, node.field1),
            is_static: (node.flags & (1u16 << 0u16)) != 0,
            decorators: decoder.nodes(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for PrivateProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::PrivateProp as u8);
        Self {
            span: decoder.span(node),
            key: AstDecode::decode(decoder, node.field0),
            value: AstDecode::decode(decoder, node.field1),
            is_static: (node.flags & (1u16 << 0u16)) != 0,
            decorators: decoder.nodes(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for ClassMethod<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ClassMethod as u8);
        Self {
            span: decoder.span(node),
            key: AstDecode::decode(decoder, node.field0),
            function: AstDecode::decode(decoder, node.field1),
            kind: match (node.flags >> 0u16) & 3u16 {
                0u16 => MethodKind::Method,
                1u16 => MethodKind::Getter,
                2u16 => MethodKind::Setter,
                value => panic!("invalid {} flag value {value}", stringify!(MethodKind)),
            },
            is_static: (node.flags & (1u16 << 2u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for PrivateMethod<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::PrivateMethod as u8);
        Self {
            span: decoder.span(node),
            key: AstDecode::decode(decoder, node.field0),
            function: AstDecode::decode(decoder, node.field1),
            kind: match (node.flags >> 0u16) & 3u16 {
                0u16 => MethodKind::Method,
                1u16 => MethodKind::Getter,
                2u16 => MethodKind::Setter,
                value => panic!("invalid {} flag value {value}", stringify!(MethodKind)),
            },
            is_static: (node.flags & (1u16 << 2u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for Constructor<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Constructor as u8);
        Self {
            span: decoder.span(node),
            key: AstDecode::decode(decoder, node.field0),
            params: decoder.nodes(node.field1, node.field2),
            body: AstDecode::decode(decoder, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for Decorator<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Decorator as u8);
        Self {
            span: decoder.span(node),
            expr: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for StaticBlock<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::StaticBlock as u8);
        Self {
            span: decoder.span(node),
            body: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for Key<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::PrivateName as u8 => {
                Self::Private(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::IdentName as u8 => {
                Self::Public(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Str as u8 => Self::Public(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Number as u8 => {
                Self::Public(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ComputedPropName as u8 => {
                Self::Public(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BigInt as u8 => {
                Self::Public(AstDecode::decode(decoder, id))
            }
            _ => panic!("unexpected node kind {} for {}", node.kind, stringify!(Key)),
        }
    }
}
impl<'a> AstDecode<'a> for AutoAccessor<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::AutoAccessor as u8);
        Self {
            span: decoder.span(node),
            key: AstDecode::decode(decoder, node.field0),
            value: AstDecode::decode(decoder, node.field1),
            is_static: (node.flags & (1u16 << 0u16)) != 0,
            decorators: decoder.nodes(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for Prop<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::Ident as u8 => {
                Self::Shorthand(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::KeyValueProp as u8 => {
                Self::KeyValue(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AssignProp as u8 => {
                Self::Assign(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::GetterProp as u8 => {
                Self::Getter(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SetterProp as u8 => {
                Self::Setter(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MethodProp as u8 => {
                Self::Method(AstDecode::decode(decoder, id))
            }
            _ => panic!(
                "unexpected node kind {} for {}",
                node.kind,
                stringify!(Prop)
            ),
        }
    }
}
impl<'a> AstDecode<'a> for KeyValueProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::KeyValueProp as u8);
        Self {
            key: AstDecode::decode(decoder, node.field0),
            value: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for AssignProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::AssignProp as u8);
        Self {
            span: decoder.span(node),
            key: AstDecode::decode(decoder, node.field0),
            value: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for GetterProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::GetterProp as u8);
        Self {
            span: decoder.span(node),
            key: AstDecode::decode(decoder, node.field0),
            body: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for SetterProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::SetterProp as u8);
        Self {
            span: decoder.span(node),
            key: AstDecode::decode(decoder, node.field0),
            this_param: AstDecode::decode(decoder, node.field1),
            param: AstDecode::decode(decoder, node.field2),
            body: AstDecode::decode(decoder, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for MethodProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::MethodProp as u8);
        Self {
            key: AstDecode::decode(decoder, node.field0),
            function: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for PropName<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::IdentName as u8 => {
                Self::Ident(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Str as u8 => Self::Str(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Number as u8 => Self::Num(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ComputedPropName as u8 => {
                Self::Computed(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BigInt as u8 => {
                Self::BigInt(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(PropName)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for ComputedPropName<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ComputedPropName as u8);
        Self {
            span: decoder.span(node),
            expr: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for Pat<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::BindingIdent as u8 => {
                Self::Ident(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrayPat as u8 => {
                Self::Array(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::RestPat as u8 => Self::Rest(AstDecode::decode(decoder, id)),
            value if value == NodeKind::ObjectPat as u8 => {
                Self::Object(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AssignPat as u8 => {
                Self::Assign(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => {
                Self::Invalid(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ThisExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrayLit as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ObjectLit as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::UnaryExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::UpdateExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BinExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::AssignExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MemberExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SuperPropExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CondExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ImportExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CallExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::NewExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::SeqExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Ident as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Str as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Bool as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Null as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Number as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::BigInt as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Regex as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Tpl as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::TaggedTpl as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrowExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::YieldExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MetaPropExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AwaitExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ParenExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXMemberExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXNamespacedName as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXEmptyExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXElement as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXFragment as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::PrivateName as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::OptChainExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            _ => panic!("unexpected node kind {} for {}", node.kind, stringify!(Pat)),
        }
    }
}
impl<'a> AstDecode<'a> for ArrayPat<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ArrayPat as u8);
        Self {
            span: decoder.span(node),
            elems: decoder.nodes(node.field0, node.field1),
            optional: (node.flags & (1u16 << 0u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for ObjectPat<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::ObjectPat as u8);
        Self {
            span: decoder.span(node),
            props: decoder.nodes(node.field0, node.field1),
            optional: (node.flags & (1u16 << 0u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for AssignPat<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::AssignPat as u8);
        Self {
            span: decoder.span(node),
            left: AstDecode::decode(decoder, node.field0),
            right: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for RestPat<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::RestPat as u8);
        Self {
            span: decoder.span(node),
            dot3_token: decoder.span(node),
            arg: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for ObjectPatProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::KeyValuePatProp as u8 => {
                Self::KeyValue(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AssignPatProp as u8 => {
                Self::Assign(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::RestPat as u8 => Self::Rest(AstDecode::decode(decoder, id)),
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(ObjectPatProp)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for KeyValuePatProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::KeyValuePatProp as u8);
        Self {
            key: AstDecode::decode(decoder, node.field0),
            value: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for AssignPatProp<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::AssignPatProp as u8);
        Self {
            span: decoder.span(node),
            key: AstDecode::decode(decoder, node.field0),
            value: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for Ident<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Ident as u8);
        Self {
            span: decoder.span(node),
            sym: decoder.atom(node.field0, node.field1),
            symbol_id: std::cell::Cell::new(None),
        }
    }
}
impl<'a> AstDecode<'a> for IdentName<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::IdentName as u8);
        Self {
            span: decoder.span(node),
            sym: decoder.atom(node.field0, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for PrivateName<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::PrivateName as u8);
        Self {
            span: decoder.span(node),
            name: decoder.atom(node.field0, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for BindingIdent<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::BindingIdent as u8);
        Self {
            id: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for Lit<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::Str as u8 => Self::Str(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Bool as u8 => Self::Bool(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Null as u8 => Self::Null(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Number as u8 => Self::Num(AstDecode::decode(decoder, id)),
            value if value == NodeKind::BigInt as u8 => {
                Self::BigInt(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Regex as u8 => Self::Regex(AstDecode::decode(decoder, id)),
            _ => panic!("unexpected node kind {} for {}", node.kind, stringify!(Lit)),
        }
    }
}
impl<'a> AstDecode<'a> for Str<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Str as u8);
        Self {
            span: decoder.span(node),
            value: decoder.wtf8_atom(node.field0, node.field1),
            raw: decoder.opt_atom(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for Bool {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Bool as u8);
        Self {
            span: decoder.span(node),
            value: (node.flags & (1u16 << 0u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for Null {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Null as u8);
        Self {
            span: decoder.span(node),
        }
    }
}
impl<'a> AstDecode<'a> for Number<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Number as u8);
        Self {
            span: decoder.span(node),
            value: f64::from_bits((node.field0 as u64) | ((node.field1 as u64) << 32)),
            raw: decoder.opt_atom(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for BigInt<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::BigInt as u8);
        Self {
            span: decoder.span(node),
            value: decoder.atom(node.field0, node.field1),
            raw: decoder.opt_atom(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for Regex<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::Regex as u8);
        Self {
            span: decoder.span(node),
            exp: decoder.atom(node.field0, node.field1),
            flags: decoder.atom(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for JSXObject<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::JSXMemberExpr as u8 => {
                Self::JSXMemberExpr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Ident as u8 => Self::Ident(AstDecode::decode(decoder, id)),
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(JSXObject)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for JSXMemberExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXMemberExpr as u8);
        Self {
            span: decoder.span(node),
            obj: AstDecode::decode(decoder, node.field0),
            prop: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for JSXNamespacedName<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXNamespacedName as u8);
        Self {
            span: decoder.span(node),
            ns: AstDecode::decode(decoder, node.field0),
            name: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for JSXEmptyExpr {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXEmptyExpr as u8);
        Self {
            span: decoder.span(node),
        }
    }
}
impl<'a> AstDecode<'a> for JSXExprContainer<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXExprContainer as u8);
        Self {
            span: decoder.span(node),
            expr: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for JSXExpr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::JSXEmptyExpr as u8 => {
                Self::JSXEmptyExpr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ThisExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrayLit as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ObjectLit as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::FnExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::UnaryExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::UpdateExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::BinExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::AssignExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MemberExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SuperPropExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CondExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ImportExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::CallExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::NewExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::SeqExpr as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Ident as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Str as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Bool as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Null as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Number as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::BigInt as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Regex as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::Tpl as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            value if value == NodeKind::TaggedTpl as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ArrowExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ClassExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::YieldExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::MetaPropExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::AwaitExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::ParenExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXMemberExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXNamespacedName as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXEmptyExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXElement as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXFragment as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::PrivateName as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::OptChainExpr as u8 => {
                Self::Expr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::Invalid as u8 => Self::Expr(AstDecode::decode(decoder, id)),
            _ => panic!(
                "unexpected node kind {} for {}",
                node.kind,
                stringify!(JSXExpr)
            ),
        }
    }
}
impl<'a> AstDecode<'a> for JSXSpreadChild<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXSpreadChild as u8);
        Self {
            span: decoder.span(node),
            expr: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for JSXElementName<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::Ident as u8 => Self::Ident(AstDecode::decode(decoder, id)),
            value if value == NodeKind::JSXMemberExpr as u8 => {
                Self::JSXMemberExpr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXNamespacedName as u8 => {
                Self::JSXNamespacedName(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(JSXElementName)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for JSXOpeningElement<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXOpeningElement as u8);
        Self {
            span: decoder.span(node),
            name: AstDecode::decode(decoder, node.field0),
            attrs: decoder.nodes(node.field1, node.field2),
            self_closing: (node.flags & (1u16 << 0u16)) != 0,
        }
    }
}
impl<'a> AstDecode<'a> for JSXAttrOrSpread<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::JSXAttr as u8 => {
                Self::JSXAttr(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::SpreadElement as u8 => {
                Self::SpreadElement(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(JSXAttrOrSpread)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for JSXClosingElement<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXClosingElement as u8);
        Self {
            span: decoder.span(node),
            name: AstDecode::decode(decoder, node.field0),
        }
    }
}
impl<'a> AstDecode<'a> for JSXAttr<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXAttr as u8);
        Self {
            span: decoder.span(node),
            name: AstDecode::decode(decoder, node.field0),
            value: AstDecode::decode(decoder, node.field1),
        }
    }
}
impl<'a> AstDecode<'a> for JSXAttrName<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::IdentName as u8 => {
                Self::Ident(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXNamespacedName as u8 => {
                Self::JSXNamespacedName(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(JSXAttrName)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for JSXAttrValue<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::Str as u8 => Self::Str(AstDecode::decode(decoder, id)),
            value if value == NodeKind::JSXExprContainer as u8 => {
                Self::JSXExprContainer(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXElement as u8 => {
                Self::JSXElement(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXFragment as u8 => {
                Self::JSXFragment(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(JSXAttrValue)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for JSXText<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXText as u8);
        Self {
            span: decoder.span(node),
            value: decoder.atom(node.field0, node.field1),
            raw: decoder.atom(node.field2, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for JSXElement<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXElement as u8);
        Self {
            span: decoder.span(node),
            opening: AstDecode::decode(decoder, node.field0),
            children: decoder.nodes(node.field1, node.field2),
            closing: AstDecode::decode(decoder, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for JSXElementChild<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        match node.kind {
            value if value == NodeKind::JSXText as u8 => {
                Self::JSXText(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXExprContainer as u8 => {
                Self::JSXExprContainer(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXSpreadChild as u8 => {
                Self::JSXSpreadChild(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXElement as u8 => {
                Self::JSXElement(AstDecode::decode(decoder, id))
            }
            value if value == NodeKind::JSXFragment as u8 => {
                Self::JSXFragment(AstDecode::decode(decoder, id))
            }
            _ => {
                panic!(
                    "unexpected node kind {} for {}",
                    node.kind,
                    stringify!(JSXElementChild)
                )
            }
        }
    }
}
impl<'a> AstDecode<'a> for JSXFragment<'a> {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXFragment as u8);
        Self {
            span: decoder.span(node),
            opening: AstDecode::decode(decoder, node.field0),
            children: decoder.nodes(node.field1, node.field2),
            closing: AstDecode::decode(decoder, node.field3),
        }
    }
}
impl<'a> AstDecode<'a> for JSXOpeningFragment {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXOpeningFragment as u8);
        Self {
            span: decoder.span(node),
        }
    }
}
impl<'a> AstDecode<'a> for JSXClosingFragment {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        let node = decoder.node(id);
        debug_assert_eq!(node.kind, NodeKind::JSXClosingFragment as u8);
        Self {
            span: decoder.span(node),
        }
    }
}
