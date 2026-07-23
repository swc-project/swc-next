#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, ExtraData};
use crate::{ast::*, node_id::*};
use oxc_index::Idx;
impl<'a> ExtraDataCompact<'a> for Program {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Stmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Decl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Expr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BindingPattern {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AssignmentTarget {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SimpleAssignmentTarget {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for PropertyKey {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ModuleExportName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Argument {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectPropertyKind {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ForStatementInit {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ForStatementLeft {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ArrowFunctionBody {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for FormalParameterItem {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for FormalParameterPattern {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ClassElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportDeclarationSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultDeclarationKind {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Super {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ThisExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for StringLiteral {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for NumericLiteral {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BigIntLiteral {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BooleanLiteral {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for NullLiteral {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for RegExpLiteral {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TemplateLiteral {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TemplateElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for IdentifierReference {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for PrivateIdentifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BindingIdentifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for IdentifierName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for LabelIdentifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AssignmentPattern {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BindingRestElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ArrayPattern {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectPattern {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BindingProperty {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BinaryExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for LogicalExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ConditionalExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for UnaryExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for UpdateExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AssignmentExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for VariableDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for VariableDeclarator {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for IfStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SwitchStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SwitchCase {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ForStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ForInStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ForOfStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TryStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for CatchClause {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BreakStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ContinueStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for LabeledStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExpressionStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ReturnStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ThrowStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for DebuggerStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for EmptyStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Directive {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Decorator {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Class {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ClassBody {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for MethodDefinition {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for PropertyDefinition {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for StaticBlock {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Function {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for FunctionBody {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BlockStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for WhileStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for DoWhileStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for WithStatement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for FormalParameters {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for FormalParameter {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ArrayExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SpreadElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectProperty {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ParenthesizedExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ArrowFunctionExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SequenceExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for MemberExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for CallExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ChainExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TaggedTemplateExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for NewExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AwaitExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for YieldExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for MetaProperty {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportDefaultSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportNamespaceSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportAttribute {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportNamedDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportAllDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Hashbang {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsQualifiedNameLeft {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsImportTypeQualifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeQueryExprName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsLiteral {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTupleElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypePredicateName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsEnumMemberName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsModuleName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsModuleBody {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsModuleReference {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeAnnotation {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsAnyKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsUnknownKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsNeverKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsVoidKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsNullKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsUndefinedKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsStringKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsNumberKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsBigIntKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsBooleanKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsSymbolKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsObjectKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsIntrinsicKeyword {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsThisType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeReference {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsQualifiedName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeQuery {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsImportType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeParameter {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeParameterDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeParameterInstantiation {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsLiteralType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTemplateLiteralType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsArrayType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsIndexedAccessType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTupleType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsNamedTupleMember {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsOptionalType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsRestType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsJsDocNullableType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsJsDocNonNullableType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsJsDocUnknownType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsUnionType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsIntersectionType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsConditionalType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsInferType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeOperator {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsParenthesizedType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsFunctionType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsConstructorType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypePredicate {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeLiteral {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsMappedType {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsPropertySignature {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsMethodSignature {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsCallSignatureDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsConstructSignatureDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsIndexSignature {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeAliasDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsInterfaceDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsInterfaceBody {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsInterfaceHeritage {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsClassImplements {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsEnumDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsEnumBody {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsEnumMember {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsModuleDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsModuleBlock {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsGlobalDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsParameterProperty {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsThisParameter {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsAsExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsSatisfiesExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsTypeAssertion {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsNonNullExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsInstantiationExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsExportAssignment {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsNamespaceExportDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsImportEqualsDeclaration {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TsExternalModuleReference {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxElementName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxMemberExpressionObject {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxAttributeName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxAttributeItem {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxAttributeValue {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxChild {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxOpeningElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxClosingElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxFragment {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxOpeningFragment {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxClosingFragment {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxIdentifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxNamespacedName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxMemberExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxAttribute {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxSpreadAttribute {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxExpressionContainer {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxEmptyExpression {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxText {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JsxSpreadChild {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Hashbang> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(Hashbang(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<TsTypeAnnotation> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(TsTypeAnnotation(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<BindingPattern> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(BindingPattern(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<BindingRestElement> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(BindingRestElement(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Expr> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(Expr(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Stmt> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(Stmt(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<ForStatementInit> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(ForStatementInit(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<CatchClause> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(CatchClause(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<BlockStatement> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(BlockStatement(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<LabelIdentifier> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(LabelIdentifier(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<BindingIdentifier> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(BindingIdentifier(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<TsTypeParameterDeclaration> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(TsTypeParameterDeclaration(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<TsTypeParameterInstantiation> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(TsTypeParameterInstantiation(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<FunctionBody> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(FunctionBody(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Argument> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(Argument(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Decl> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(Decl(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<StringLiteral> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(StringLiteral(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<ModuleExportName> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(ModuleExportName(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<TsImportTypeQualifier> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(TsImportTypeQualifier(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<TsType> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(TsType(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<TsModuleBody> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(TsModuleBody(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<JsxClosingElement> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(JsxClosingElement(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<JsxAttributeValue> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then_some(JsxAttributeValue(node_id))
    }
}
