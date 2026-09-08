#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, GetSpan, NodeKind};
use crate::{ast::*, node_id::*};
impl<'a> CloneIn<'a> for Program {
    type Cloned = Program;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let directives = self.directives(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let hashbang = self.hashbang(ast).clone_in(ast);
        ast.program(span, directives, body, hashbang)
    }
}
impl<'a> CloneIn<'a> for Stmt {
    type Cloned = Stmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::BlockStatement => Self::BlockStatement(BlockStatement(self.0).clone_in(ast)),
            NodeKind::BreakStatement => Self::BreakStatement(BreakStatement(self.0).clone_in(ast)),
            NodeKind::ContinueStatement => {
                Self::ContinueStatement(ContinueStatement(self.0).clone_in(ast))
            }
            NodeKind::DebuggerStatement => {
                Self::DebuggerStatement(DebuggerStatement(self.0).clone_in(ast))
            }
            NodeKind::DoWhileStatement => {
                Self::DoWhileStatement(DoWhileStatement(self.0).clone_in(ast))
            }
            NodeKind::EmptyStatement => Self::EmptyStatement(EmptyStatement(self.0).clone_in(ast)),
            NodeKind::ExpressionStatement => {
                Self::ExpressionStatement(ExpressionStatement(self.0).clone_in(ast))
            }
            NodeKind::ForInStatement => Self::ForInStatement(ForInStatement(self.0).clone_in(ast)),
            NodeKind::ForOfStatement => Self::ForOfStatement(ForOfStatement(self.0).clone_in(ast)),
            NodeKind::ForStatement => Self::ForStatement(ForStatement(self.0).clone_in(ast)),
            NodeKind::IfStatement => Self::IfStatement(IfStatement(self.0).clone_in(ast)),
            NodeKind::LabeledStatement => {
                Self::LabeledStatement(LabeledStatement(self.0).clone_in(ast))
            }
            NodeKind::ReturnStatement => {
                Self::ReturnStatement(ReturnStatement(self.0).clone_in(ast))
            }
            NodeKind::SwitchStatement => {
                Self::SwitchStatement(SwitchStatement(self.0).clone_in(ast))
            }
            NodeKind::ThrowStatement => Self::ThrowStatement(ThrowStatement(self.0).clone_in(ast)),
            NodeKind::TryStatement => Self::TryStatement(TryStatement(self.0).clone_in(ast)),
            NodeKind::WhileStatement => Self::WhileStatement(WhileStatement(self.0).clone_in(ast)),
            NodeKind::WithStatement => Self::WithStatement(WithStatement(self.0).clone_in(ast)),
            NodeKind::VariableDeclaration
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::TsFunction
            | NodeKind::TsTypeAliasDeclaration
            | NodeKind::TsInterfaceDeclaration
            | NodeKind::TsEnumDeclaration
            | NodeKind::TsModuleDeclaration
            | NodeKind::TsGlobalDeclaration
            | NodeKind::TsImportEqualsDeclaration => Self::Declaration(Decl(self.0).clone_in(ast)),
            NodeKind::ImportDeclaration => {
                Self::ImportDeclaration(ImportDeclaration(self.0).clone_in(ast))
            }
            NodeKind::ExportNamedDeclaration => {
                Self::ExportNamedDeclaration(ExportNamedDeclaration(self.0).clone_in(ast))
            }
            NodeKind::ExportDefaultDeclaration => {
                Self::ExportDefaultDeclaration(ExportDefaultDeclaration(self.0).clone_in(ast))
            }
            NodeKind::ExportAllDeclaration => {
                Self::ExportAllDeclaration(ExportAllDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsExportAssignment => {
                Self::TsExportAssignment(TsExportAssignment(self.0).clone_in(ast))
            }
            NodeKind::TsNamespaceExportDeclaration => Self::TsNamespaceExportDeclaration(
                TsNamespaceExportDeclaration(self.0).clone_in(ast),
            ),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for Decl {
    type Cloned = Decl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::VariableDeclaration => {
                Self::VariableDeclaration(VariableDeclaration(self.0).clone_in(ast))
            }
            NodeKind::Function => Self::Function(Function(self.0).clone_in(ast)),
            NodeKind::Class => Self::Class(Class(self.0).clone_in(ast)),
            NodeKind::TsFunction => Self::TsFunction(TsFunction(self.0).clone_in(ast)),
            NodeKind::TsTypeAliasDeclaration => {
                Self::TsTypeAliasDeclaration(TsTypeAliasDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsInterfaceDeclaration => {
                Self::TsInterfaceDeclaration(TsInterfaceDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsEnumDeclaration => {
                Self::TsEnumDeclaration(TsEnumDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsModuleDeclaration => {
                Self::TsModuleDeclaration(TsModuleDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsGlobalDeclaration => {
                Self::TsGlobalDeclaration(TsGlobalDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsImportEqualsDeclaration => {
                Self::TsImportEqualsDeclaration(TsImportEqualsDeclaration(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for Expr {
    type Cloned = Expr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierReference => {
                Self::IdentifierReference(IdentifierReference(self.0).clone_in(ast))
            }
            NodeKind::ThisExpression => Self::ThisExpression(ThisExpression(self.0).clone_in(ast)),
            NodeKind::Super => Self::Super(Super(self.0).clone_in(ast)),
            NodeKind::MetaProperty => Self::MetaProperty(MetaProperty(self.0).clone_in(ast)),
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::NumericLiteral => Self::NumericLiteral(NumericLiteral(self.0).clone_in(ast)),
            NodeKind::BigIntLiteral => Self::BigIntLiteral(BigIntLiteral(self.0).clone_in(ast)),
            NodeKind::BooleanLiteral => Self::BooleanLiteral(BooleanLiteral(self.0).clone_in(ast)),
            NodeKind::NullLiteral => Self::NullLiteral(NullLiteral(self.0).clone_in(ast)),
            NodeKind::RegExpLiteral => Self::RegExpLiteral(RegExpLiteral(self.0).clone_in(ast)),
            NodeKind::TemplateLiteral => {
                Self::TemplateLiteral(TemplateLiteral(self.0).clone_in(ast))
            }
            NodeKind::BinaryExpression => {
                Self::BinaryExpression(BinaryExpression(self.0).clone_in(ast))
            }
            NodeKind::LogicalExpression => {
                Self::LogicalExpression(LogicalExpression(self.0).clone_in(ast))
            }
            NodeKind::UnaryExpression => {
                Self::UnaryExpression(UnaryExpression(self.0).clone_in(ast))
            }
            NodeKind::UpdateExpression => {
                Self::UpdateExpression(UpdateExpression(self.0).clone_in(ast))
            }
            NodeKind::AssignmentExpression => {
                Self::AssignmentExpression(AssignmentExpression(self.0).clone_in(ast))
            }
            NodeKind::ConditionalExpression => {
                Self::ConditionalExpression(ConditionalExpression(self.0).clone_in(ast))
            }
            NodeKind::SequenceExpression => {
                Self::SequenceExpression(SequenceExpression(self.0).clone_in(ast))
            }
            NodeKind::MemberExpression => {
                Self::MemberExpression(MemberExpression(self.0).clone_in(ast))
            }
            NodeKind::CallExpression => Self::CallExpression(CallExpression(self.0).clone_in(ast)),
            NodeKind::ChainExpression => {
                Self::ChainExpression(ChainExpression(self.0).clone_in(ast))
            }
            NodeKind::NewExpression => Self::NewExpression(NewExpression(self.0).clone_in(ast)),
            NodeKind::TaggedTemplateExpression => {
                Self::TaggedTemplateExpression(TaggedTemplateExpression(self.0).clone_in(ast))
            }
            NodeKind::ArrowFunctionExpression => {
                Self::ArrowFunctionExpression(ArrowFunctionExpression(self.0).clone_in(ast))
            }
            NodeKind::ArrayExpression => {
                Self::ArrayExpression(ArrayExpression(self.0).clone_in(ast))
            }
            NodeKind::ObjectExpression => {
                Self::ObjectExpression(ObjectExpression(self.0).clone_in(ast))
            }
            NodeKind::ParenthesizedExpression => {
                Self::ParenthesizedExpression(ParenthesizedExpression(self.0).clone_in(ast))
            }
            NodeKind::Function => Self::Function(Function(self.0).clone_in(ast)),
            NodeKind::Class => Self::Class(Class(self.0).clone_in(ast)),
            NodeKind::ImportExpression => {
                Self::ImportExpression(ImportExpression(self.0).clone_in(ast))
            }
            NodeKind::AwaitExpression => {
                Self::AwaitExpression(AwaitExpression(self.0).clone_in(ast))
            }
            NodeKind::YieldExpression => {
                Self::YieldExpression(YieldExpression(self.0).clone_in(ast))
            }
            NodeKind::PrivateIdentifier => {
                Self::PrivateIdentifier(PrivateIdentifier(self.0).clone_in(ast))
            }
            NodeKind::TsAsExpression => Self::TsAsExpression(TsAsExpression(self.0).clone_in(ast)),
            NodeKind::TsSatisfiesExpression => {
                Self::TsSatisfiesExpression(TsSatisfiesExpression(self.0).clone_in(ast))
            }
            NodeKind::TsTypeAssertion => {
                Self::TsTypeAssertion(TsTypeAssertion(self.0).clone_in(ast))
            }
            NodeKind::TsNonNullExpression => {
                Self::TsNonNullExpression(TsNonNullExpression(self.0).clone_in(ast))
            }
            NodeKind::TsInstantiationExpression => {
                Self::TsInstantiationExpression(TsInstantiationExpression(self.0).clone_in(ast))
            }
            NodeKind::JsxElement => Self::JsxElement(JsxElement(self.0).clone_in(ast)),
            NodeKind::JsxFragment => Self::JsxFragment(JsxFragment(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for BindingPattern {
    type Cloned = BindingPattern;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::BindingIdentifier => {
                Self::BindingIdentifier(BindingIdentifier(self.0).clone_in(ast))
            }
            NodeKind::IdentifierReference
            | NodeKind::MemberExpression
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression => {
                Self::SimpleAssignmentTarget(SimpleAssignmentTarget(self.0).clone_in(ast))
            }
            NodeKind::AssignmentPattern => {
                Self::AssignmentPattern(AssignmentPattern(self.0).clone_in(ast))
            }
            NodeKind::BindingRestElement => {
                Self::BindingRestElement(BindingRestElement(self.0).clone_in(ast))
            }
            NodeKind::ArrayPattern => Self::ArrayPattern(ArrayPattern(self.0).clone_in(ast)),
            NodeKind::ObjectPattern => Self::ObjectPattern(ObjectPattern(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for AssignmentTarget {
    type Cloned = AssignmentTarget;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierReference
            | NodeKind::MemberExpression
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression => {
                Self::SimpleAssignmentTarget(SimpleAssignmentTarget(self.0).clone_in(ast))
            }
            NodeKind::ArrayPattern => {
                Self::ArrayAssignmentTarget(ArrayPattern(self.0).clone_in(ast))
            }
            NodeKind::ObjectPattern => {
                Self::ObjectAssignmentTarget(ObjectPattern(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for SimpleAssignmentTarget {
    type Cloned = SimpleAssignmentTarget;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierReference => {
                Self::IdentifierReference(IdentifierReference(self.0).clone_in(ast))
            }
            NodeKind::MemberExpression => {
                Self::MemberExpression(MemberExpression(self.0).clone_in(ast))
            }
            NodeKind::TsAsExpression => Self::TsAsExpression(TsAsExpression(self.0).clone_in(ast)),
            NodeKind::TsSatisfiesExpression => {
                Self::TsSatisfiesExpression(TsSatisfiesExpression(self.0).clone_in(ast))
            }
            NodeKind::TsTypeAssertion => {
                Self::TsTypeAssertion(TsTypeAssertion(self.0).clone_in(ast))
            }
            NodeKind::TsNonNullExpression => {
                Self::TsNonNullExpression(TsNonNullExpression(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for PropertyKey {
    type Cloned = PropertyKey;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::PrivateIdentifier => {
                Self::PrivateIdentifier(PrivateIdentifier(self.0).clone_in(ast))
            }
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::NumericLiteral => Self::NumericLiteral(NumericLiteral(self.0).clone_in(ast)),
            NodeKind::BigIntLiteral => Self::BigIntLiteral(BigIntLiteral(self.0).clone_in(ast)),
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for ModuleExportName {
    type Cloned = ModuleExportName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for Argument {
    type Cloned = Argument;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            NodeKind::SpreadElement => Self::SpreadElement(SpreadElement(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for ObjectPropertyKind {
    type Cloned = ObjectPropertyKind;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::ObjectProperty => Self::ObjectProperty(ObjectProperty(self.0).clone_in(ast)),
            NodeKind::SpreadElement => Self::SpreadElement(SpreadElement(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for ForStatementInit {
    type Cloned = ForStatementInit;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::VariableDeclaration => {
                Self::VariableDeclaration(VariableDeclaration(self.0).clone_in(ast))
            }
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for ForStatementLeft {
    type Cloned = ForStatementLeft;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::VariableDeclaration => {
                Self::VariableDeclaration(VariableDeclaration(self.0).clone_in(ast))
            }
            NodeKind::IdentifierReference
            | NodeKind::MemberExpression
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::ArrayPattern
            | NodeKind::ObjectPattern => {
                Self::AssignmentTarget(AssignmentTarget(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for ArrowFunctionBody {
    type Cloned = ArrowFunctionBody;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::FunctionBody => Self::FunctionBody(FunctionBody(self.0).clone_in(ast)),
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for FormalParameterItem {
    type Cloned = FormalParameterItem;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::FormalParameter => {
                Self::FormalParameter(FormalParameter(self.0).clone_in(ast))
            }
            NodeKind::TsParameterProperty => {
                Self::TsParameterProperty(TsParameterProperty(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for FormalParameterPattern {
    type Cloned = FormalParameterPattern;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::BindingIdentifier
            | NodeKind::IdentifierReference
            | NodeKind::MemberExpression
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::AssignmentPattern
            | NodeKind::BindingRestElement
            | NodeKind::ArrayPattern
            | NodeKind::ObjectPattern => Self::BindingPattern(BindingPattern(self.0).clone_in(ast)),
            NodeKind::TsThisParameter => {
                Self::TsThisParameter(TsThisParameter(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for ClassElement {
    type Cloned = ClassElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::MethodDefinition => {
                Self::MethodDefinition(MethodDefinition(self.0).clone_in(ast))
            }
            NodeKind::PropertyDefinition => {
                Self::PropertyDefinition(PropertyDefinition(self.0).clone_in(ast))
            }
            NodeKind::StaticBlock => Self::StaticBlock(StaticBlock(self.0).clone_in(ast)),
            NodeKind::TsMethodDefinition => {
                Self::TsMethodDefinition(TsMethodDefinition(self.0).clone_in(ast))
            }
            NodeKind::TsIndexSignature => {
                Self::TsIndexSignature(TsIndexSignature(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for ImportDeclarationSpecifier {
    type Cloned = ImportDeclarationSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::ImportSpecifier => {
                Self::ImportSpecifier(ImportSpecifier(self.0).clone_in(ast))
            }
            NodeKind::ImportDefaultSpecifier => {
                Self::ImportDefaultSpecifier(ImportDefaultSpecifier(self.0).clone_in(ast))
            }
            NodeKind::ImportNamespaceSpecifier => {
                Self::ImportNamespaceSpecifier(ImportNamespaceSpecifier(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for ExportDefaultDeclarationKind {
    type Cloned = ExportDefaultDeclarationKind;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::Function => {
                let concrete = Function(self.0);
                if concrete.function_type(ast) == FunctionType::FunctionDeclaration {
                    Self::Function(concrete.clone_in(ast))
                } else {
                    Self::Expr(Expr(self.0).clone_in(ast))
                }
            }
            NodeKind::Class => {
                let concrete = Class(self.0);
                if concrete.class_type(ast) == ClassType::ClassDeclaration {
                    Self::Class(concrete.clone_in(ast))
                } else {
                    Self::Expr(Expr(self.0).clone_in(ast))
                }
            }
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            NodeKind::TsFunction => Self::TsFunction(TsFunction(self.0).clone_in(ast)),
            NodeKind::TsInterfaceDeclaration => {
                Self::TsInterfaceDeclaration(TsInterfaceDeclaration(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for Super {
    type Cloned = Super;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.super_(span)
    }
}
impl<'a> CloneIn<'a> for ThisExpression {
    type Cloned = ThisExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.this_expression(span)
    }
}
impl<'a> CloneIn<'a> for StringLiteral {
    type Cloned = StringLiteral;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.string_literal(span, value, raw)
    }
}
impl<'a> CloneIn<'a> for NumericLiteral {
    type Cloned = NumericLiteral;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.numeric_literal(span, value, raw)
    }
}
impl<'a> CloneIn<'a> for BigIntLiteral {
    type Cloned = BigIntLiteral;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.big_int_literal(span, raw)
    }
}
impl<'a> CloneIn<'a> for BooleanLiteral {
    type Cloned = BooleanLiteral;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.boolean_literal(span, value)
    }
}
impl<'a> CloneIn<'a> for NullLiteral {
    type Cloned = NullLiteral;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.null_literal(span)
    }
}
impl<'a> CloneIn<'a> for RegExpLiteral {
    type Cloned = RegExpLiteral;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let pattern = self.pattern(ast).clone_in(ast);
        let flags = self.flags(ast).clone_in(ast);
        ast.reg_exp_literal(span, pattern, flags)
    }
}
impl<'a> CloneIn<'a> for TemplateLiteral {
    type Cloned = TemplateLiteral;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let quasis = self.quasis(ast).clone_in(ast);
        let expressions = self.expressions(ast).clone_in(ast);
        ast.template_literal(span, quasis, expressions)
    }
}
impl<'a> CloneIn<'a> for TemplateElement {
    type Cloned = TemplateElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let cooked = self.cooked(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        let tail = self.tail(ast).clone_in(ast);
        let is_cooked_undefined = self.is_cooked_undefined(ast).clone_in(ast);
        ast.template_element(span, cooked, raw, tail, is_cooked_undefined)
    }
}
impl<'a> CloneIn<'a> for IdentifierReference {
    type Cloned = IdentifierReference;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.identifier_reference(span, name)
    }
}
impl<'a> CloneIn<'a> for PrivateIdentifier {
    type Cloned = PrivateIdentifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.private_identifier(span, name)
    }
}
impl<'a> CloneIn<'a> for BindingIdentifier {
    type Cloned = BindingIdentifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.binding_identifier(span, name, decorators, type_annotation, optional)
    }
}
impl<'a> CloneIn<'a> for IdentifierName {
    type Cloned = IdentifierName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.identifier_name(span, name)
    }
}
impl<'a> CloneIn<'a> for LabelIdentifier {
    type Cloned = LabelIdentifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.label_identifier(span, name)
    }
}
impl<'a> CloneIn<'a> for AssignmentPattern {
    type Cloned = AssignmentPattern;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.assignment_pattern(span, decorators, left, type_annotation, right, optional)
    }
}
impl<'a> CloneIn<'a> for BindingRestElement {
    type Cloned = BindingRestElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.binding_rest_element(span, decorators, argument, type_annotation, optional)
    }
}
impl<'a> CloneIn<'a> for ArrayPattern {
    type Cloned = ArrayPattern;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let elements = self.elements(ast).clone_in(ast);
        let rest = self.rest(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.array_pattern(span, decorators, elements, rest, type_annotation, optional)
    }
}
impl<'a> CloneIn<'a> for ObjectPattern {
    type Cloned = ObjectPattern;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let properties = self.properties(ast).clone_in(ast);
        let rest = self.rest(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.object_pattern(
            span,
            decorators,
            properties,
            rest,
            type_annotation,
            optional,
        )
    }
}
impl<'a> CloneIn<'a> for BindingProperty {
    type Cloned = BindingProperty;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let shorthand = self.shorthand(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        ast.binding_property(span, key, value, shorthand, computed)
    }
}
impl<'a> CloneIn<'a> for BinaryExpression {
    type Cloned = BinaryExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        ast.binary_expression(span, left, right, operator)
    }
}
impl<'a> CloneIn<'a> for LogicalExpression {
    type Cloned = LogicalExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        ast.logical_expression(span, left, right, operator)
    }
}
impl<'a> CloneIn<'a> for ConditionalExpression {
    type Cloned = ConditionalExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let consequent = self.consequent(ast).clone_in(ast);
        let alternate = self.alternate(ast).clone_in(ast);
        ast.conditional_expression(span, test, consequent, alternate)
    }
}
impl<'a> CloneIn<'a> for UnaryExpression {
    type Cloned = UnaryExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        ast.unary_expression(span, argument, operator)
    }
}
impl<'a> CloneIn<'a> for UpdateExpression {
    type Cloned = UpdateExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        let prefix = self.prefix(ast).clone_in(ast);
        ast.update_expression(span, argument, operator, prefix)
    }
}
impl<'a> CloneIn<'a> for AssignmentExpression {
    type Cloned = AssignmentExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        ast.assignment_expression(span, left, right, operator)
    }
}
impl<'a> CloneIn<'a> for VariableDeclaration {
    type Cloned = VariableDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let declarators = self.declarators(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.variable_declaration(span, kind, declarators, declare)
    }
}
impl<'a> CloneIn<'a> for VariableDeclarator {
    type Cloned = VariableDeclarator;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let init = self.init(ast).clone_in(ast);
        let definite = self.definite(ast).clone_in(ast);
        ast.variable_declarator(span, id, init, definite)
    }
}
impl<'a> CloneIn<'a> for IfStatement {
    type Cloned = IfStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let consequent = self.consequent(ast).clone_in(ast);
        let alternate = self.alternate(ast).clone_in(ast);
        ast.if_statement(span, test, consequent, alternate)
    }
}
impl<'a> CloneIn<'a> for SwitchStatement {
    type Cloned = SwitchStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let discriminant = self.discriminant(ast).clone_in(ast);
        let cases = self.cases(ast).clone_in(ast);
        ast.switch_statement(span, discriminant, cases)
    }
}
impl<'a> CloneIn<'a> for SwitchCase {
    type Cloned = SwitchCase;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let consequent = self.consequent(ast).clone_in(ast);
        ast.switch_case(span, test, consequent)
    }
}
impl<'a> CloneIn<'a> for ForStatement {
    type Cloned = ForStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let init = self.init(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let update = self.update(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.for_statement(span, init, test, update, body)
    }
}
impl<'a> CloneIn<'a> for ForInStatement {
    type Cloned = ForInStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.for_in_statement(span, left, right, body)
    }
}
impl<'a> CloneIn<'a> for ForOfStatement {
    type Cloned = ForOfStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let r#await = self.r#await(ast).clone_in(ast);
        ast.for_of_statement(span, left, right, body, r#await)
    }
}
impl<'a> CloneIn<'a> for TryStatement {
    type Cloned = TryStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let block = self.block(ast).clone_in(ast);
        let handler = self.handler(ast).clone_in(ast);
        let finalizer = self.finalizer(ast).clone_in(ast);
        ast.try_statement(span, block, handler, finalizer)
    }
}
impl<'a> CloneIn<'a> for CatchClause {
    type Cloned = CatchClause;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let param = self.param(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.catch_clause(span, param, body)
    }
}
impl<'a> CloneIn<'a> for BreakStatement {
    type Cloned = BreakStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        ast.break_statement(span, label)
    }
}
impl<'a> CloneIn<'a> for ContinueStatement {
    type Cloned = ContinueStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        ast.continue_statement(span, label)
    }
}
impl<'a> CloneIn<'a> for LabeledStatement {
    type Cloned = LabeledStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.labeled_statement(span, label, body)
    }
}
impl<'a> CloneIn<'a> for ExpressionStatement {
    type Cloned = ExpressionStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.expression_statement(span, expression)
    }
}
impl<'a> CloneIn<'a> for ReturnStatement {
    type Cloned = ReturnStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.return_statement(span, argument)
    }
}
impl<'a> CloneIn<'a> for ThrowStatement {
    type Cloned = ThrowStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.throw_statement(span, argument)
    }
}
impl<'a> CloneIn<'a> for DebuggerStatement {
    type Cloned = DebuggerStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.debugger_statement(span)
    }
}
impl<'a> CloneIn<'a> for EmptyStatement {
    type Cloned = EmptyStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.empty_statement(span)
    }
}
impl<'a> CloneIn<'a> for Directive {
    type Cloned = Directive;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.directive(span, expression, value)
    }
}
impl<'a> CloneIn<'a> for Decorator {
    type Cloned = Decorator;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.decorator(span, expression)
    }
}
impl<'a> CloneIn<'a> for Class {
    type Cloned = Class;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let class_type = self.class_type(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let super_class = self.super_class(ast).clone_in(ast);
        let super_type_arguments = self.super_type_arguments(ast).clone_in(ast);
        let implements = self.implements(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.class(
            span,
            class_type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
        )
    }
}
impl<'a> CloneIn<'a> for ClassBody {
    type Cloned = ClassBody;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.class_body(span, body)
    }
}
impl<'a> CloneIn<'a> for MethodDefinition {
    type Cloned = MethodDefinition;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let r#static = self.r#static(ast).clone_in(ast);
        let r#override = self.r#override(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        let accessibility = self.accessibility(ast).clone_in(ast);
        ast.method_definition(
            span,
            decorators,
            key,
            value,
            kind,
            computed,
            r#static,
            r#override,
            optional,
            r#abstract,
            accessibility,
        )
    }
}
impl<'a> CloneIn<'a> for PropertyDefinition {
    type Cloned = PropertyDefinition;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let r#static = self.r#static(ast).clone_in(ast);
        let accessor = self.accessor(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let r#override = self.r#override(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let definite = self.definite(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        let accessibility = self.accessibility(ast).clone_in(ast);
        ast.property_definition(
            span,
            decorators,
            key,
            type_annotation,
            value,
            computed,
            r#static,
            accessor,
            declare,
            r#override,
            optional,
            definite,
            readonly,
            r#abstract,
            accessibility,
        )
    }
}
impl<'a> CloneIn<'a> for StaticBlock {
    type Cloned = StaticBlock;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.static_block(span, body)
    }
}
impl<'a> CloneIn<'a> for Function {
    type Cloned = Function;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let function_type = self.function_type(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let generator = self.generator(ast).clone_in(ast);
        let r#async = self.r#async(ast).clone_in(ast);
        ast.function(
            span,
            function_type,
            id,
            type_parameters,
            params,
            return_type,
            body,
            generator,
            r#async,
        )
    }
}
impl<'a> CloneIn<'a> for FunctionBody {
    type Cloned = FunctionBody;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let directives = self.directives(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.function_body(span, directives, body)
    }
}
impl<'a> CloneIn<'a> for BlockStatement {
    type Cloned = BlockStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.block_statement(span, body)
    }
}
impl<'a> CloneIn<'a> for WhileStatement {
    type Cloned = WhileStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.while_statement(span, test, body)
    }
}
impl<'a> CloneIn<'a> for DoWhileStatement {
    type Cloned = DoWhileStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        ast.do_while_statement(span, body, test)
    }
}
impl<'a> CloneIn<'a> for WithStatement {
    type Cloned = WithStatement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let object = self.object(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.with_statement(span, object, body)
    }
}
impl<'a> CloneIn<'a> for FormalParameters {
    type Cloned = FormalParameters;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let items = self.items(ast).clone_in(ast);
        let rest = self.rest(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        ast.formal_parameters(span, items, rest, kind)
    }
}
impl<'a> CloneIn<'a> for FormalParameter {
    type Cloned = FormalParameter;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let pattern = self.pattern(ast).clone_in(ast);
        ast.formal_parameter(span, pattern)
    }
}
impl<'a> CloneIn<'a> for ArrayExpression {
    type Cloned = ArrayExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let elements = self.elements(ast).clone_in(ast);
        ast.array_expression(span, elements)
    }
}
impl<'a> CloneIn<'a> for ObjectExpression {
    type Cloned = ObjectExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let properties = self.properties(ast).clone_in(ast);
        ast.object_expression(span, properties)
    }
}
impl<'a> CloneIn<'a> for SpreadElement {
    type Cloned = SpreadElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.spread_element(span, argument)
    }
}
impl<'a> CloneIn<'a> for ObjectProperty {
    type Cloned = ObjectProperty;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let method = self.method(ast).clone_in(ast);
        let shorthand = self.shorthand(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        ast.object_property(span, key, value, kind, method, shorthand, computed)
    }
}
impl<'a> CloneIn<'a> for ParenthesizedExpression {
    type Cloned = ParenthesizedExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.parenthesized_expression(span, expression)
    }
}
impl<'a> CloneIn<'a> for ArrowFunctionExpression {
    type Cloned = ArrowFunctionExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let r#async = self.r#async(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.arrow_function_expression(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
        )
    }
}
impl<'a> CloneIn<'a> for SequenceExpression {
    type Cloned = SequenceExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expressions = self.expressions(ast).clone_in(ast);
        ast.sequence_expression(span, expressions)
    }
}
impl<'a> CloneIn<'a> for MemberExpression {
    type Cloned = MemberExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let object = self.object(ast).clone_in(ast);
        let property = self.property(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.member_expression(span, object, property, computed, optional)
    }
}
impl<'a> CloneIn<'a> for CallExpression {
    type Cloned = CallExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        let arguments = self.arguments(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.call_expression(span, callee, type_arguments, arguments, optional)
    }
}
impl<'a> CloneIn<'a> for ChainExpression {
    type Cloned = ChainExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.chain_expression(span, expression)
    }
}
impl<'a> CloneIn<'a> for TaggedTemplateExpression {
    type Cloned = TaggedTemplateExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let tag = self.tag(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        let quasi = self.quasi(ast).clone_in(ast);
        ast.tagged_template_expression(span, tag, type_arguments, quasi)
    }
}
impl<'a> CloneIn<'a> for NewExpression {
    type Cloned = NewExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        let arguments = self.arguments(ast).clone_in(ast);
        ast.new_expression(span, callee, type_arguments, arguments)
    }
}
impl<'a> CloneIn<'a> for AwaitExpression {
    type Cloned = AwaitExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.await_expression(span, argument)
    }
}
impl<'a> CloneIn<'a> for YieldExpression {
    type Cloned = YieldExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        let delegate = self.delegate(ast).clone_in(ast);
        ast.yield_expression(span, argument, delegate)
    }
}
impl<'a> CloneIn<'a> for MetaProperty {
    type Cloned = MetaProperty;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let meta = self.meta(ast).clone_in(ast);
        let property = self.property(ast).clone_in(ast);
        ast.meta_property(span, meta, property)
    }
}
impl<'a> CloneIn<'a> for ImportExpression {
    type Cloned = ImportExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let options = self.options(ast).clone_in(ast);
        let phase = self.phase(ast).clone_in(ast);
        let sync = self.sync(ast).clone_in(ast);
        ast.import_expression(span, source, options, phase, sync)
    }
}
impl<'a> CloneIn<'a> for ImportDeclaration {
    type Cloned = ImportDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let specifiers = self.specifiers(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let attributes = self.attributes(ast).clone_in(ast);
        let phase = self.phase(ast).clone_in(ast);
        let import_kind = self.import_kind(ast).clone_in(ast);
        ast.import_declaration(span, specifiers, source, attributes, phase, import_kind)
    }
}
impl<'a> CloneIn<'a> for ImportSpecifier {
    type Cloned = ImportSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let imported = self.imported(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        let import_kind = self.import_kind(ast).clone_in(ast);
        ast.import_specifier(span, imported, local, import_kind)
    }
}
impl<'a> CloneIn<'a> for ImportDefaultSpecifier {
    type Cloned = ImportDefaultSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        ast.import_default_specifier(span, local)
    }
}
impl<'a> CloneIn<'a> for ImportNamespaceSpecifier {
    type Cloned = ImportNamespaceSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        ast.import_namespace_specifier(span, local)
    }
}
impl<'a> CloneIn<'a> for ImportAttribute {
    type Cloned = ImportAttribute;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.import_attribute(span, key, value)
    }
}
impl<'a> CloneIn<'a> for ExportNamedDeclaration {
    type Cloned = ExportNamedDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let declaration = self.declaration(ast).clone_in(ast);
        let specifiers = self.specifiers(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let attributes = self.attributes(ast).clone_in(ast);
        let export_kind = self.export_kind(ast).clone_in(ast);
        ast.export_named_declaration(
            span,
            declaration,
            specifiers,
            source,
            attributes,
            export_kind,
        )
    }
}
impl<'a> CloneIn<'a> for ExportDefaultDeclaration {
    type Cloned = ExportDefaultDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let declaration = self.declaration(ast).clone_in(ast);
        ast.export_default_declaration(span, declaration)
    }
}
impl<'a> CloneIn<'a> for ExportAllDeclaration {
    type Cloned = ExportAllDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let exported = self.exported(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let attributes = self.attributes(ast).clone_in(ast);
        let export_kind = self.export_kind(ast).clone_in(ast);
        ast.export_all_declaration(span, exported, source, attributes, export_kind)
    }
}
impl<'a> CloneIn<'a> for ExportSpecifier {
    type Cloned = ExportSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        let exported = self.exported(ast).clone_in(ast);
        let export_kind = self.export_kind(ast).clone_in(ast);
        ast.export_specifier(span, local, exported, export_kind)
    }
}
impl<'a> CloneIn<'a> for Hashbang {
    type Cloned = Hashbang;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.hashbang(span, value)
    }
}
impl<'a> CloneIn<'a> for TsType {
    type Cloned = TsType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::TsAnyKeyword => Self::TsAnyKeyword(TsAnyKeyword(self.0).clone_in(ast)),
            NodeKind::TsUnknownKeyword => {
                Self::TsUnknownKeyword(TsUnknownKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsNeverKeyword => Self::TsNeverKeyword(TsNeverKeyword(self.0).clone_in(ast)),
            NodeKind::TsVoidKeyword => Self::TsVoidKeyword(TsVoidKeyword(self.0).clone_in(ast)),
            NodeKind::TsNullKeyword => Self::TsNullKeyword(TsNullKeyword(self.0).clone_in(ast)),
            NodeKind::TsUndefinedKeyword => {
                Self::TsUndefinedKeyword(TsUndefinedKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsStringKeyword => {
                Self::TsStringKeyword(TsStringKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsNumberKeyword => {
                Self::TsNumberKeyword(TsNumberKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsBigIntKeyword => {
                Self::TsBigIntKeyword(TsBigIntKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsBooleanKeyword => {
                Self::TsBooleanKeyword(TsBooleanKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsSymbolKeyword => {
                Self::TsSymbolKeyword(TsSymbolKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsObjectKeyword => {
                Self::TsObjectKeyword(TsObjectKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsIntrinsicKeyword => {
                Self::TsIntrinsicKeyword(TsIntrinsicKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsThisType => Self::TsThisType(TsThisType(self.0).clone_in(ast)),
            NodeKind::TsTypeReference => {
                Self::TsTypeReference(TsTypeReference(self.0).clone_in(ast))
            }
            NodeKind::TsTypeQuery => Self::TsTypeQuery(TsTypeQuery(self.0).clone_in(ast)),
            NodeKind::TsImportType => Self::TsImportType(TsImportType(self.0).clone_in(ast)),
            NodeKind::TsLiteralType => Self::TsLiteralType(TsLiteralType(self.0).clone_in(ast)),
            NodeKind::TsTemplateLiteralType => {
                Self::TsTemplateLiteralType(TsTemplateLiteralType(self.0).clone_in(ast))
            }
            NodeKind::TsArrayType => Self::TsArrayType(TsArrayType(self.0).clone_in(ast)),
            NodeKind::TsIndexedAccessType => {
                Self::TsIndexedAccessType(TsIndexedAccessType(self.0).clone_in(ast))
            }
            NodeKind::TsTupleType => Self::TsTupleType(TsTupleType(self.0).clone_in(ast)),
            NodeKind::TsOptionalType => Self::TsOptionalType(TsOptionalType(self.0).clone_in(ast)),
            NodeKind::TsRestType => Self::TsRestType(TsRestType(self.0).clone_in(ast)),
            NodeKind::TsJsDocNullableType => {
                Self::TsJsDocNullableType(TsJsDocNullableType(self.0).clone_in(ast))
            }
            NodeKind::TsJsDocNonNullableType => {
                Self::TsJsDocNonNullableType(TsJsDocNonNullableType(self.0).clone_in(ast))
            }
            NodeKind::TsJsDocUnknownType => {
                Self::TsJsDocUnknownType(TsJsDocUnknownType(self.0).clone_in(ast))
            }
            NodeKind::TsUnionType => Self::TsUnionType(TsUnionType(self.0).clone_in(ast)),
            NodeKind::TsIntersectionType => {
                Self::TsIntersectionType(TsIntersectionType(self.0).clone_in(ast))
            }
            NodeKind::TsConditionalType => {
                Self::TsConditionalType(TsConditionalType(self.0).clone_in(ast))
            }
            NodeKind::TsInferType => Self::TsInferType(TsInferType(self.0).clone_in(ast)),
            NodeKind::TsTypeOperator => Self::TsTypeOperator(TsTypeOperator(self.0).clone_in(ast)),
            NodeKind::TsParenthesizedType => {
                Self::TsParenthesizedType(TsParenthesizedType(self.0).clone_in(ast))
            }
            NodeKind::TsFunctionType => Self::TsFunctionType(TsFunctionType(self.0).clone_in(ast)),
            NodeKind::TsConstructorType => {
                Self::TsConstructorType(TsConstructorType(self.0).clone_in(ast))
            }
            NodeKind::TsTypePredicate => {
                Self::TsTypePredicate(TsTypePredicate(self.0).clone_in(ast))
            }
            NodeKind::TsTypeLiteral => Self::TsTypeLiteral(TsTypeLiteral(self.0).clone_in(ast)),
            NodeKind::TsMappedType => Self::TsMappedType(TsMappedType(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsTypeName {
    type Cloned = TsTypeName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierReference => {
                Self::IdentifierReference(IdentifierReference(self.0).clone_in(ast))
            }
            NodeKind::TsQualifiedName => {
                Self::TsQualifiedName(TsQualifiedName(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsQualifiedNameLeft {
    type Cloned = TsQualifiedNameLeft;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierReference => {
                Self::IdentifierReference(IdentifierReference(self.0).clone_in(ast))
            }
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::BindingIdentifier => {
                Self::BindingIdentifier(BindingIdentifier(self.0).clone_in(ast))
            }
            NodeKind::ThisExpression => Self::ThisExpression(ThisExpression(self.0).clone_in(ast)),
            NodeKind::TsQualifiedName => {
                Self::TsQualifiedName(TsQualifiedName(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsImportTypeQualifier {
    type Cloned = TsImportTypeQualifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::TsQualifiedName => {
                Self::TsQualifiedName(TsQualifiedName(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsTypeQueryExprName {
    type Cloned = TsTypeQueryExprName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName => {
                Self::TsTypeName(TsTypeName(self.0).clone_in(ast))
            }
            NodeKind::ThisExpression => Self::ThisExpression(ThisExpression(self.0).clone_in(ast)),
            NodeKind::TsImportType => Self::TsImportType(TsImportType(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsLiteral {
    type Cloned = TsLiteral;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::NumericLiteral => Self::NumericLiteral(NumericLiteral(self.0).clone_in(ast)),
            NodeKind::BigIntLiteral => Self::BigIntLiteral(BigIntLiteral(self.0).clone_in(ast)),
            NodeKind::BooleanLiteral => Self::BooleanLiteral(BooleanLiteral(self.0).clone_in(ast)),
            NodeKind::TemplateLiteral => {
                Self::TemplateLiteral(TemplateLiteral(self.0).clone_in(ast))
            }
            NodeKind::UnaryExpression => {
                Self::UnaryExpression(UnaryExpression(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsTupleElement {
    type Cloned = TsTupleElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::TsAnyKeyword
            | NodeKind::TsUnknownKeyword
            | NodeKind::TsNeverKeyword
            | NodeKind::TsVoidKeyword
            | NodeKind::TsNullKeyword
            | NodeKind::TsUndefinedKeyword
            | NodeKind::TsStringKeyword
            | NodeKind::TsNumberKeyword
            | NodeKind::TsBigIntKeyword
            | NodeKind::TsBooleanKeyword
            | NodeKind::TsSymbolKeyword
            | NodeKind::TsObjectKeyword
            | NodeKind::TsIntrinsicKeyword
            | NodeKind::TsThisType
            | NodeKind::TsTypeReference
            | NodeKind::TsTypeQuery
            | NodeKind::TsImportType
            | NodeKind::TsLiteralType
            | NodeKind::TsTemplateLiteralType
            | NodeKind::TsArrayType
            | NodeKind::TsIndexedAccessType
            | NodeKind::TsTupleType
            | NodeKind::TsOptionalType
            | NodeKind::TsRestType
            | NodeKind::TsJsDocNullableType
            | NodeKind::TsJsDocNonNullableType
            | NodeKind::TsJsDocUnknownType
            | NodeKind::TsUnionType
            | NodeKind::TsIntersectionType
            | NodeKind::TsConditionalType
            | NodeKind::TsInferType
            | NodeKind::TsTypeOperator
            | NodeKind::TsParenthesizedType
            | NodeKind::TsFunctionType
            | NodeKind::TsConstructorType
            | NodeKind::TsTypePredicate
            | NodeKind::TsTypeLiteral
            | NodeKind::TsMappedType => Self::TsType(TsType(self.0).clone_in(ast)),
            NodeKind::TsNamedTupleMember => {
                Self::TsNamedTupleMember(TsNamedTupleMember(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsTypeElement {
    type Cloned = TsTypeElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::TsPropertySignature => {
                Self::TsPropertySignature(TsPropertySignature(self.0).clone_in(ast))
            }
            NodeKind::TsMethodSignature => {
                Self::TsMethodSignature(TsMethodSignature(self.0).clone_in(ast))
            }
            NodeKind::TsCallSignatureDeclaration => {
                Self::TsCallSignatureDeclaration(TsCallSignatureDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsConstructSignatureDeclaration => Self::TsConstructSignatureDeclaration(
                TsConstructSignatureDeclaration(self.0).clone_in(ast),
            ),
            NodeKind::TsIndexSignature => {
                Self::TsIndexSignature(TsIndexSignature(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsTypePredicateName {
    type Cloned = TsTypePredicateName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::TsThisType => Self::TsThisType(TsThisType(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsEnumMemberName {
    type Cloned = TsEnumMemberName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::TemplateLiteral => {
                Self::TemplateLiteral(TemplateLiteral(self.0).clone_in(ast))
            }
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsModuleName {
    type Cloned = TsModuleName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::BindingIdentifier => {
                Self::BindingIdentifier(BindingIdentifier(self.0).clone_in(ast))
            }
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::TsQualifiedName => {
                Self::TsQualifiedName(TsQualifiedName(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsModuleBody {
    type Cloned = TsModuleBody;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::TsModuleBlock => Self::TsModuleBlock(TsModuleBlock(self.0).clone_in(ast)),
            NodeKind::TsModuleDeclaration => {
                Self::TsModuleDeclaration(TsModuleDeclaration(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsModuleReference {
    type Cloned = TsModuleReference;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName => {
                Self::TsTypeName(TsTypeName(self.0).clone_in(ast))
            }
            NodeKind::TsExternalModuleReference => {
                Self::TsExternalModuleReference(TsExternalModuleReference(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for TsFunction {
    type Cloned = TsFunction;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        let generator = self.generator(ast).clone_in(ast);
        let r#async = self.r#async(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_function(
            span,
            id,
            type_parameters,
            params,
            return_type,
            generator,
            r#async,
            declare,
        )
    }
}
impl<'a> CloneIn<'a> for TsMethodDefinition {
    type Cloned = TsMethodDefinition;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let r#static = self.r#static(ast).clone_in(ast);
        let r#override = self.r#override(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        let accessibility = self.accessibility(ast).clone_in(ast);
        ast.ts_method_definition(
            span,
            decorators,
            key,
            value,
            kind,
            computed,
            r#static,
            r#override,
            optional,
            r#abstract,
            accessibility,
        )
    }
}
impl<'a> CloneIn<'a> for TsTypeAnnotation {
    type Cloned = TsTypeAnnotation;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_type_annotation(span, type_annotation)
    }
}
impl<'a> CloneIn<'a> for TsAnyKeyword {
    type Cloned = TsAnyKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_any_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsUnknownKeyword {
    type Cloned = TsUnknownKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_unknown_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsNeverKeyword {
    type Cloned = TsNeverKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_never_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsVoidKeyword {
    type Cloned = TsVoidKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_void_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsNullKeyword {
    type Cloned = TsNullKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_null_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsUndefinedKeyword {
    type Cloned = TsUndefinedKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_undefined_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsStringKeyword {
    type Cloned = TsStringKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_string_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsNumberKeyword {
    type Cloned = TsNumberKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_number_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsBigIntKeyword {
    type Cloned = TsBigIntKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_big_int_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsBooleanKeyword {
    type Cloned = TsBooleanKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_boolean_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsSymbolKeyword {
    type Cloned = TsSymbolKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_symbol_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsObjectKeyword {
    type Cloned = TsObjectKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_object_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsIntrinsicKeyword {
    type Cloned = TsIntrinsicKeyword;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_intrinsic_keyword(span)
    }
}
impl<'a> CloneIn<'a> for TsThisType {
    type Cloned = TsThisType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_this_type(span)
    }
}
impl<'a> CloneIn<'a> for TsTypeReference {
    type Cloned = TsTypeReference;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_name = self.type_name(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_type_reference(span, type_name, type_arguments)
    }
}
impl<'a> CloneIn<'a> for TsQualifiedName {
    type Cloned = TsQualifiedName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        ast.ts_qualified_name(span, left, right)
    }
}
impl<'a> CloneIn<'a> for TsTypeQuery {
    type Cloned = TsTypeQuery;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr_name = self.expr_name(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_type_query(span, expr_name, type_arguments)
    }
}
impl<'a> CloneIn<'a> for TsImportType {
    type Cloned = TsImportType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let options = self.options(ast).clone_in(ast);
        let qualifier = self.qualifier(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_import_type(span, source, options, qualifier, type_arguments)
    }
}
impl<'a> CloneIn<'a> for TsTypeParameter {
    type Cloned = TsTypeParameter;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let constraint = self.constraint(ast).clone_in(ast);
        let default = self.default(ast).clone_in(ast);
        let r#in = self.r#in(ast).clone_in(ast);
        let out = self.out(ast).clone_in(ast);
        let r#const = self.r#const(ast).clone_in(ast);
        ast.ts_type_parameter(span, name, constraint, default, r#in, out, r#const)
    }
}
impl<'a> CloneIn<'a> for TsTypeParameterDeclaration {
    type Cloned = TsTypeParameterDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        ast.ts_type_parameter_declaration(span, params)
    }
}
impl<'a> CloneIn<'a> for TsTypeParameterInstantiation {
    type Cloned = TsTypeParameterInstantiation;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        ast.ts_type_parameter_instantiation(span, params)
    }
}
impl<'a> CloneIn<'a> for TsLiteralType {
    type Cloned = TsLiteralType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let literal = self.literal(ast).clone_in(ast);
        ast.ts_literal_type(span, literal)
    }
}
impl<'a> CloneIn<'a> for TsTemplateLiteralType {
    type Cloned = TsTemplateLiteralType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let quasis = self.quasis(ast).clone_in(ast);
        let types = self.types(ast).clone_in(ast);
        ast.ts_template_literal_type(span, quasis, types)
    }
}
impl<'a> CloneIn<'a> for TsArrayType {
    type Cloned = TsArrayType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let element_type = self.element_type(ast).clone_in(ast);
        ast.ts_array_type(span, element_type)
    }
}
impl<'a> CloneIn<'a> for TsIndexedAccessType {
    type Cloned = TsIndexedAccessType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let object_type = self.object_type(ast).clone_in(ast);
        let index_type = self.index_type(ast).clone_in(ast);
        ast.ts_indexed_access_type(span, object_type, index_type)
    }
}
impl<'a> CloneIn<'a> for TsTupleType {
    type Cloned = TsTupleType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let element_types = self.element_types(ast).clone_in(ast);
        ast.ts_tuple_type(span, element_types)
    }
}
impl<'a> CloneIn<'a> for TsNamedTupleMember {
    type Cloned = TsNamedTupleMember;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        let element_type = self.element_type(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.ts_named_tuple_member(span, label, element_type, optional)
    }
}
impl<'a> CloneIn<'a> for TsOptionalType {
    type Cloned = TsOptionalType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_optional_type(span, type_annotation)
    }
}
impl<'a> CloneIn<'a> for TsRestType {
    type Cloned = TsRestType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_rest_type(span, type_annotation)
    }
}
impl<'a> CloneIn<'a> for TsJsDocNullableType {
    type Cloned = TsJsDocNullableType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let postfix = self.postfix(ast).clone_in(ast);
        ast.ts_js_doc_nullable_type(span, type_annotation, postfix)
    }
}
impl<'a> CloneIn<'a> for TsJsDocNonNullableType {
    type Cloned = TsJsDocNonNullableType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let postfix = self.postfix(ast).clone_in(ast);
        ast.ts_js_doc_non_nullable_type(span, type_annotation, postfix)
    }
}
impl<'a> CloneIn<'a> for TsJsDocUnknownType {
    type Cloned = TsJsDocUnknownType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_js_doc_unknown_type(span)
    }
}
impl<'a> CloneIn<'a> for TsUnionType {
    type Cloned = TsUnionType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let types = self.types(ast).clone_in(ast);
        ast.ts_union_type(span, types)
    }
}
impl<'a> CloneIn<'a> for TsIntersectionType {
    type Cloned = TsIntersectionType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let types = self.types(ast).clone_in(ast);
        ast.ts_intersection_type(span, types)
    }
}
impl<'a> CloneIn<'a> for TsConditionalType {
    type Cloned = TsConditionalType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let check_type = self.check_type(ast).clone_in(ast);
        let extends_type = self.extends_type(ast).clone_in(ast);
        let true_type = self.true_type(ast).clone_in(ast);
        let false_type = self.false_type(ast).clone_in(ast);
        ast.ts_conditional_type(span, check_type, extends_type, true_type, false_type)
    }
}
impl<'a> CloneIn<'a> for TsInferType {
    type Cloned = TsInferType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_parameter = self.type_parameter(ast).clone_in(ast);
        ast.ts_infer_type(span, type_parameter)
    }
}
impl<'a> CloneIn<'a> for TsTypeOperator {
    type Cloned = TsTypeOperator;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_type_operator(span, operator, type_annotation)
    }
}
impl<'a> CloneIn<'a> for TsParenthesizedType {
    type Cloned = TsParenthesizedType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_parenthesized_type(span, type_annotation)
    }
}
impl<'a> CloneIn<'a> for TsFunctionType {
    type Cloned = TsFunctionType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        ast.ts_function_type(span, type_parameters, params, return_type)
    }
}
impl<'a> CloneIn<'a> for TsConstructorType {
    type Cloned = TsConstructorType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        ast.ts_constructor_type(span, type_parameters, params, return_type, r#abstract)
    }
}
impl<'a> CloneIn<'a> for TsTypePredicate {
    type Cloned = TsTypePredicate;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let parameter_name = self.parameter_name(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let asserts = self.asserts(ast).clone_in(ast);
        ast.ts_type_predicate(span, parameter_name, type_annotation, asserts)
    }
}
impl<'a> CloneIn<'a> for TsTypeLiteral {
    type Cloned = TsTypeLiteral;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let members = self.members(ast).clone_in(ast);
        ast.ts_type_literal(span, members)
    }
}
impl<'a> CloneIn<'a> for TsMappedType {
    type Cloned = TsMappedType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let constraint = self.constraint(ast).clone_in(ast);
        let name_type = self.name_type(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        ast.ts_mapped_type(
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
        )
    }
}
impl<'a> CloneIn<'a> for TsPropertySignature {
    type Cloned = TsPropertySignature;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        ast.ts_property_signature(span, key, type_annotation, computed, optional, readonly)
    }
}
impl<'a> CloneIn<'a> for TsMethodSignature {
    type Cloned = TsMethodSignature;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.ts_method_signature(
            span,
            key,
            type_parameters,
            params,
            return_type,
            kind,
            computed,
            optional,
        )
    }
}
impl<'a> CloneIn<'a> for TsCallSignatureDeclaration {
    type Cloned = TsCallSignatureDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        ast.ts_call_signature_declaration(span, type_parameters, params, return_type)
    }
}
impl<'a> CloneIn<'a> for TsConstructSignatureDeclaration {
    type Cloned = TsConstructSignatureDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        ast.ts_construct_signature_declaration(span, type_parameters, params, return_type)
    }
}
impl<'a> CloneIn<'a> for TsIndexSignature {
    type Cloned = TsIndexSignature;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let parameters = self.parameters(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let r#static = self.r#static(ast).clone_in(ast);
        ast.ts_index_signature(span, parameters, type_annotation, readonly, r#static)
    }
}
impl<'a> CloneIn<'a> for TsTypeAliasDeclaration {
    type Cloned = TsTypeAliasDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare)
    }
}
impl<'a> CloneIn<'a> for TsInterfaceDeclaration {
    type Cloned = TsInterfaceDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let extends = self.extends(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_interface_declaration(span, id, type_parameters, extends, body, declare)
    }
}
impl<'a> CloneIn<'a> for TsInterfaceBody {
    type Cloned = TsInterfaceBody;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.ts_interface_body(span, body)
    }
}
impl<'a> CloneIn<'a> for TsInterfaceHeritage {
    type Cloned = TsInterfaceHeritage;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_interface_heritage(span, expression, type_arguments)
    }
}
impl<'a> CloneIn<'a> for TsClassImplements {
    type Cloned = TsClassImplements;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_class_implements(span, expression, type_arguments)
    }
}
impl<'a> CloneIn<'a> for TsEnumDeclaration {
    type Cloned = TsEnumDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let is_const = self.is_const(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_enum_declaration(span, id, body, is_const, declare)
    }
}
impl<'a> CloneIn<'a> for TsEnumBody {
    type Cloned = TsEnumBody;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let members = self.members(ast).clone_in(ast);
        ast.ts_enum_body(span, members)
    }
}
impl<'a> CloneIn<'a> for TsEnumMember {
    type Cloned = TsEnumMember;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let initializer = self.initializer(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        ast.ts_enum_member(span, id, initializer, computed)
    }
}
impl<'a> CloneIn<'a> for TsModuleDeclaration {
    type Cloned = TsModuleDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_module_declaration(span, id, body, kind, declare)
    }
}
impl<'a> CloneIn<'a> for TsModuleBlock {
    type Cloned = TsModuleBlock;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let directives = self.directives(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.ts_module_block(span, directives, body)
    }
}
impl<'a> CloneIn<'a> for TsGlobalDeclaration {
    type Cloned = TsGlobalDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_global_declaration(span, id, body, declare)
    }
}
impl<'a> CloneIn<'a> for TsParameterProperty {
    type Cloned = TsParameterProperty;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let parameter = self.parameter(ast).clone_in(ast);
        let r#override = self.r#override(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let accessibility = self.accessibility(ast).clone_in(ast);
        ast.ts_parameter_property(
            span,
            decorators,
            parameter,
            r#override,
            readonly,
            accessibility,
        )
    }
}
impl<'a> CloneIn<'a> for TsThisParameter {
    type Cloned = TsThisParameter;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_this_parameter(span, type_annotation)
    }
}
impl<'a> CloneIn<'a> for TsAsExpression {
    type Cloned = TsAsExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_as_expression(span, expression, type_annotation)
    }
}
impl<'a> CloneIn<'a> for TsSatisfiesExpression {
    type Cloned = TsSatisfiesExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_satisfies_expression(span, expression, type_annotation)
    }
}
impl<'a> CloneIn<'a> for TsTypeAssertion {
    type Cloned = TsTypeAssertion;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.ts_type_assertion(span, type_annotation, expression)
    }
}
impl<'a> CloneIn<'a> for TsNonNullExpression {
    type Cloned = TsNonNullExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.ts_non_null_expression(span, expression)
    }
}
impl<'a> CloneIn<'a> for TsInstantiationExpression {
    type Cloned = TsInstantiationExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_instantiation_expression(span, expression, type_arguments)
    }
}
impl<'a> CloneIn<'a> for TsExportAssignment {
    type Cloned = TsExportAssignment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.ts_export_assignment(span, expression)
    }
}
impl<'a> CloneIn<'a> for TsNamespaceExportDeclaration {
    type Cloned = TsNamespaceExportDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        ast.ts_namespace_export_declaration(span, id)
    }
}
impl<'a> CloneIn<'a> for TsImportEqualsDeclaration {
    type Cloned = TsImportEqualsDeclaration;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let module_reference = self.module_reference(ast).clone_in(ast);
        let import_kind = self.import_kind(ast).clone_in(ast);
        ast.ts_import_equals_declaration(span, id, module_reference, import_kind)
    }
}
impl<'a> CloneIn<'a> for TsExternalModuleReference {
    type Cloned = TsExternalModuleReference;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.ts_external_module_reference(span, expression)
    }
}
impl<'a> CloneIn<'a> for JsxElementName {
    type Cloned = JsxElementName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::JsxIdentifier => Self::JsxIdentifier(JsxIdentifier(self.0).clone_in(ast)),
            NodeKind::JsxNamespacedName => {
                Self::JsxNamespacedName(JsxNamespacedName(self.0).clone_in(ast))
            }
            NodeKind::JsxMemberExpression => {
                Self::JsxMemberExpression(JsxMemberExpression(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for JsxMemberExpressionObject {
    type Cloned = JsxMemberExpressionObject;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::JsxIdentifier => Self::JsxIdentifier(JsxIdentifier(self.0).clone_in(ast)),
            NodeKind::JsxMemberExpression => {
                Self::JsxMemberExpression(JsxMemberExpression(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for JsxAttributeName {
    type Cloned = JsxAttributeName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::JsxIdentifier => Self::JsxIdentifier(JsxIdentifier(self.0).clone_in(ast)),
            NodeKind::JsxNamespacedName => {
                Self::JsxNamespacedName(JsxNamespacedName(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for JsxAttributeItem {
    type Cloned = JsxAttributeItem;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::JsxAttribute => Self::JsxAttribute(JsxAttribute(self.0).clone_in(ast)),
            NodeKind::JsxSpreadAttribute => {
                Self::JsxSpreadAttribute(JsxSpreadAttribute(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for JsxAttributeValue {
    type Cloned = JsxAttributeValue;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::JsxExpressionContainer => {
                Self::JsxExpressionContainer(JsxExpressionContainer(self.0).clone_in(ast))
            }
            NodeKind::JsxElement => Self::JsxElement(JsxElement(self.0).clone_in(ast)),
            NodeKind::JsxFragment => Self::JsxFragment(JsxFragment(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for JsxExpression {
    type Cloned = JsxExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            NodeKind::JsxEmptyExpression => {
                Self::JsxEmptyExpression(JsxEmptyExpression(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for JsxChild {
    type Cloned = JsxChild;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::JsxText => Self::JsxText(JsxText(self.0).clone_in(ast)),
            NodeKind::JsxExpressionContainer => {
                Self::JsxExpressionContainer(JsxExpressionContainer(self.0).clone_in(ast))
            }
            NodeKind::JsxSpreadChild => Self::JsxSpreadChild(JsxSpreadChild(self.0).clone_in(ast)),
            NodeKind::JsxElement => Self::JsxElement(JsxElement(self.0).clone_in(ast)),
            NodeKind::JsxFragment => Self::JsxFragment(JsxFragment(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a> CloneIn<'a> for JsxElement {
    type Cloned = JsxElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let opening_element = self.opening_element(ast).clone_in(ast);
        let children = self.children(ast).clone_in(ast);
        let closing_element = self.closing_element(ast).clone_in(ast);
        ast.jsx_element(span, opening_element, children, closing_element)
    }
}
impl<'a> CloneIn<'a> for JsxOpeningElement {
    type Cloned = JsxOpeningElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        let attributes = self.attributes(ast).clone_in(ast);
        let self_closing = self.self_closing(ast).clone_in(ast);
        ast.jsx_opening_element(span, name, type_arguments, attributes, self_closing)
    }
}
impl<'a> CloneIn<'a> for JsxClosingElement {
    type Cloned = JsxClosingElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.jsx_closing_element(span, name)
    }
}
impl<'a> CloneIn<'a> for JsxFragment {
    type Cloned = JsxFragment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let opening_fragment = self.opening_fragment(ast).clone_in(ast);
        let children = self.children(ast).clone_in(ast);
        let closing_fragment = self.closing_fragment(ast).clone_in(ast);
        ast.jsx_fragment(span, opening_fragment, children, closing_fragment)
    }
}
impl<'a> CloneIn<'a> for JsxOpeningFragment {
    type Cloned = JsxOpeningFragment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_opening_fragment(span)
    }
}
impl<'a> CloneIn<'a> for JsxClosingFragment {
    type Cloned = JsxClosingFragment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_closing_fragment(span)
    }
}
impl<'a> CloneIn<'a> for JsxIdentifier {
    type Cloned = JsxIdentifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.jsx_identifier(span, name)
    }
}
impl<'a> CloneIn<'a> for JsxNamespacedName {
    type Cloned = JsxNamespacedName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let namespace = self.namespace(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.jsx_namespaced_name(span, namespace, name)
    }
}
impl<'a> CloneIn<'a> for JsxMemberExpression {
    type Cloned = JsxMemberExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let object = self.object(ast).clone_in(ast);
        let property = self.property(ast).clone_in(ast);
        ast.jsx_member_expression(span, object, property)
    }
}
impl<'a> CloneIn<'a> for JsxAttribute {
    type Cloned = JsxAttribute;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.jsx_attribute(span, name, value)
    }
}
impl<'a> CloneIn<'a> for JsxSpreadAttribute {
    type Cloned = JsxSpreadAttribute;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.jsx_spread_attribute(span, argument)
    }
}
impl<'a> CloneIn<'a> for JsxExpressionContainer {
    type Cloned = JsxExpressionContainer;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.jsx_expression_container(span, expression)
    }
}
impl<'a> CloneIn<'a> for JsxEmptyExpression {
    type Cloned = JsxEmptyExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_empty_expression(span)
    }
}
impl<'a> CloneIn<'a> for JsxText {
    type Cloned = JsxText;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.jsx_text(span, value)
    }
}
impl<'a> CloneIn<'a> for JsxSpreadChild {
    type Cloned = JsxSpreadChild;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.jsx_spread_child(span, expression)
    }
}
