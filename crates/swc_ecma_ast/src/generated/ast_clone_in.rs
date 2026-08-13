#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, GetSpan};
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
        let data = ast.stmt_data(*self);
        match data {
            StmtData::BlockStatement(it) => Self::BlockStatement(it.clone_in(ast)),
            StmtData::BreakStatement(it) => Self::BreakStatement(it.clone_in(ast)),
            StmtData::ContinueStatement(it) => Self::ContinueStatement(it.clone_in(ast)),
            StmtData::DebuggerStatement(it) => Self::DebuggerStatement(it.clone_in(ast)),
            StmtData::DoWhileStatement(it) => Self::DoWhileStatement(it.clone_in(ast)),
            StmtData::EmptyStatement(it) => Self::EmptyStatement(it.clone_in(ast)),
            StmtData::ExpressionStatement(it) => Self::ExpressionStatement(it.clone_in(ast)),
            StmtData::ForInStatement(it) => Self::ForInStatement(it.clone_in(ast)),
            StmtData::ForOfStatement(it) => Self::ForOfStatement(it.clone_in(ast)),
            StmtData::ForStatement(it) => Self::ForStatement(it.clone_in(ast)),
            StmtData::IfStatement(it) => Self::IfStatement(it.clone_in(ast)),
            StmtData::LabeledStatement(it) => Self::LabeledStatement(it.clone_in(ast)),
            StmtData::ReturnStatement(it) => Self::ReturnStatement(it.clone_in(ast)),
            StmtData::SwitchStatement(it) => Self::SwitchStatement(it.clone_in(ast)),
            StmtData::ThrowStatement(it) => Self::ThrowStatement(it.clone_in(ast)),
            StmtData::TryStatement(it) => Self::TryStatement(it.clone_in(ast)),
            StmtData::WhileStatement(it) => Self::WhileStatement(it.clone_in(ast)),
            StmtData::WithStatement(it) => Self::WithStatement(it.clone_in(ast)),
            StmtData::Declaration(it) => Self::Declaration(it.clone_in(ast)),
            StmtData::ImportDeclaration(it) => Self::ImportDeclaration(it.clone_in(ast)),
            StmtData::ExportNamedDeclaration(it) => Self::ExportNamedDeclaration(it.clone_in(ast)),
            StmtData::ExportDefaultDeclaration(it) => {
                Self::ExportDefaultDeclaration(it.clone_in(ast))
            }
            StmtData::ExportAllDeclaration(it) => Self::ExportAllDeclaration(it.clone_in(ast)),
            StmtData::TsExportAssignment(it) => Self::TsExportAssignment(it.clone_in(ast)),
            StmtData::TsNamespaceExportDeclaration(it) => {
                Self::TsNamespaceExportDeclaration(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for Decl {
    type Cloned = Decl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.decl_data(*self);
        match data {
            DeclData::VariableDeclaration(it) => Self::VariableDeclaration(it.clone_in(ast)),
            DeclData::Function(it) => Self::Function(it.clone_in(ast)),
            DeclData::Class(it) => Self::Class(it.clone_in(ast)),
            DeclData::TsFunction(it) => Self::TsFunction(it.clone_in(ast)),
            DeclData::TsTypeAliasDeclaration(it) => Self::TsTypeAliasDeclaration(it.clone_in(ast)),
            DeclData::TsInterfaceDeclaration(it) => Self::TsInterfaceDeclaration(it.clone_in(ast)),
            DeclData::TsEnumDeclaration(it) => Self::TsEnumDeclaration(it.clone_in(ast)),
            DeclData::TsModuleDeclaration(it) => Self::TsModuleDeclaration(it.clone_in(ast)),
            DeclData::TsGlobalDeclaration(it) => Self::TsGlobalDeclaration(it.clone_in(ast)),
            DeclData::TsImportEqualsDeclaration(it) => {
                Self::TsImportEqualsDeclaration(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for Expr {
    type Cloned = Expr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.expr_data(*self);
        match data {
            ExprData::IdentifierReference(it) => Self::IdentifierReference(it.clone_in(ast)),
            ExprData::ThisExpression(it) => Self::ThisExpression(it.clone_in(ast)),
            ExprData::Super(it) => Self::Super(it.clone_in(ast)),
            ExprData::MetaProperty(it) => Self::MetaProperty(it.clone_in(ast)),
            ExprData::StringLiteral(it) => Self::StringLiteral(it.clone_in(ast)),
            ExprData::NumericLiteral(it) => Self::NumericLiteral(it.clone_in(ast)),
            ExprData::BigIntLiteral(it) => Self::BigIntLiteral(it.clone_in(ast)),
            ExprData::BooleanLiteral(it) => Self::BooleanLiteral(it.clone_in(ast)),
            ExprData::NullLiteral(it) => Self::NullLiteral(it.clone_in(ast)),
            ExprData::RegExpLiteral(it) => Self::RegExpLiteral(it.clone_in(ast)),
            ExprData::TemplateLiteral(it) => Self::TemplateLiteral(it.clone_in(ast)),
            ExprData::BinaryExpression(it) => Self::BinaryExpression(it.clone_in(ast)),
            ExprData::LogicalExpression(it) => Self::LogicalExpression(it.clone_in(ast)),
            ExprData::UnaryExpression(it) => Self::UnaryExpression(it.clone_in(ast)),
            ExprData::UpdateExpression(it) => Self::UpdateExpression(it.clone_in(ast)),
            ExprData::AssignmentExpression(it) => Self::AssignmentExpression(it.clone_in(ast)),
            ExprData::ConditionalExpression(it) => Self::ConditionalExpression(it.clone_in(ast)),
            ExprData::SequenceExpression(it) => Self::SequenceExpression(it.clone_in(ast)),
            ExprData::MemberExpression(it) => Self::MemberExpression(it.clone_in(ast)),
            ExprData::CallExpression(it) => Self::CallExpression(it.clone_in(ast)),
            ExprData::ChainExpression(it) => Self::ChainExpression(it.clone_in(ast)),
            ExprData::NewExpression(it) => Self::NewExpression(it.clone_in(ast)),
            ExprData::TaggedTemplateExpression(it) => {
                Self::TaggedTemplateExpression(it.clone_in(ast))
            }
            ExprData::ArrowFunctionExpression(it) => {
                Self::ArrowFunctionExpression(it.clone_in(ast))
            }
            ExprData::ArrayExpression(it) => Self::ArrayExpression(it.clone_in(ast)),
            ExprData::ObjectExpression(it) => Self::ObjectExpression(it.clone_in(ast)),
            ExprData::ParenthesizedExpression(it) => {
                Self::ParenthesizedExpression(it.clone_in(ast))
            }
            ExprData::Function(it) => Self::Function(it.clone_in(ast)),
            ExprData::Class(it) => Self::Class(it.clone_in(ast)),
            ExprData::ImportExpression(it) => Self::ImportExpression(it.clone_in(ast)),
            ExprData::AwaitExpression(it) => Self::AwaitExpression(it.clone_in(ast)),
            ExprData::YieldExpression(it) => Self::YieldExpression(it.clone_in(ast)),
            ExprData::PrivateIdentifier(it) => Self::PrivateIdentifier(it.clone_in(ast)),
            ExprData::TsAsExpression(it) => Self::TsAsExpression(it.clone_in(ast)),
            ExprData::TsSatisfiesExpression(it) => Self::TsSatisfiesExpression(it.clone_in(ast)),
            ExprData::TsTypeAssertion(it) => Self::TsTypeAssertion(it.clone_in(ast)),
            ExprData::TsNonNullExpression(it) => Self::TsNonNullExpression(it.clone_in(ast)),
            ExprData::TsInstantiationExpression(it) => {
                Self::TsInstantiationExpression(it.clone_in(ast))
            }
            ExprData::JsxElement(it) => Self::JsxElement(it.clone_in(ast)),
            ExprData::JsxFragment(it) => Self::JsxFragment(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for BindingPattern {
    type Cloned = BindingPattern;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.binding_pattern_data(*self);
        match data {
            BindingPatternData::BindingIdentifier(it) => Self::BindingIdentifier(it.clone_in(ast)),
            BindingPatternData::SimpleAssignmentTarget(it) => {
                Self::SimpleAssignmentTarget(it.clone_in(ast))
            }
            BindingPatternData::AssignmentPattern(it) => Self::AssignmentPattern(it.clone_in(ast)),
            BindingPatternData::BindingRestElement(it) => {
                Self::BindingRestElement(it.clone_in(ast))
            }
            BindingPatternData::ArrayPattern(it) => Self::ArrayPattern(it.clone_in(ast)),
            BindingPatternData::ObjectPattern(it) => Self::ObjectPattern(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for AssignmentTarget {
    type Cloned = AssignmentTarget;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.assignment_target_data(*self);
        match data {
            AssignmentTargetData::SimpleAssignmentTarget(it) => {
                Self::SimpleAssignmentTarget(it.clone_in(ast))
            }
            AssignmentTargetData::ArrayAssignmentTarget(it) => {
                Self::ArrayAssignmentTarget(it.clone_in(ast))
            }
            AssignmentTargetData::ObjectAssignmentTarget(it) => {
                Self::ObjectAssignmentTarget(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for SimpleAssignmentTarget {
    type Cloned = SimpleAssignmentTarget;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.simple_assignment_target_data(*self);
        match data {
            SimpleAssignmentTargetData::IdentifierReference(it) => {
                Self::IdentifierReference(it.clone_in(ast))
            }
            SimpleAssignmentTargetData::MemberExpression(it) => {
                Self::MemberExpression(it.clone_in(ast))
            }
            SimpleAssignmentTargetData::TsAsExpression(it) => {
                Self::TsAsExpression(it.clone_in(ast))
            }
            SimpleAssignmentTargetData::TsSatisfiesExpression(it) => {
                Self::TsSatisfiesExpression(it.clone_in(ast))
            }
            SimpleAssignmentTargetData::TsTypeAssertion(it) => {
                Self::TsTypeAssertion(it.clone_in(ast))
            }
            SimpleAssignmentTargetData::TsNonNullExpression(it) => {
                Self::TsNonNullExpression(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for PropertyKey {
    type Cloned = PropertyKey;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.property_key_data(*self);
        match data {
            PropertyKeyData::IdentifierName(it) => Self::IdentifierName(it.clone_in(ast)),
            PropertyKeyData::PrivateIdentifier(it) => Self::PrivateIdentifier(it.clone_in(ast)),
            PropertyKeyData::StringLiteral(it) => Self::StringLiteral(it.clone_in(ast)),
            PropertyKeyData::NumericLiteral(it) => Self::NumericLiteral(it.clone_in(ast)),
            PropertyKeyData::BigIntLiteral(it) => Self::BigIntLiteral(it.clone_in(ast)),
            PropertyKeyData::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ModuleExportName {
    type Cloned = ModuleExportName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.module_export_name_data(*self);
        match data {
            ModuleExportNameData::IdentifierName(it) => Self::IdentifierName(it.clone_in(ast)),
            ModuleExportNameData::StringLiteral(it) => Self::StringLiteral(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for Argument {
    type Cloned = Argument;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.argument_data(*self);
        match data {
            ArgumentData::Expr(it) => Self::Expr(it.clone_in(ast)),
            ArgumentData::SpreadElement(it) => Self::SpreadElement(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ObjectPropertyKind {
    type Cloned = ObjectPropertyKind;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.object_property_kind_data(*self);
        match data {
            ObjectPropertyKindData::ObjectProperty(it) => Self::ObjectProperty(it.clone_in(ast)),
            ObjectPropertyKindData::SpreadElement(it) => Self::SpreadElement(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ForStatementInit {
    type Cloned = ForStatementInit;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.for_statement_init_data(*self);
        match data {
            ForStatementInitData::VariableDeclaration(it) => {
                Self::VariableDeclaration(it.clone_in(ast))
            }
            ForStatementInitData::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ForStatementLeft {
    type Cloned = ForStatementLeft;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.for_statement_left_data(*self);
        match data {
            ForStatementLeftData::VariableDeclaration(it) => {
                Self::VariableDeclaration(it.clone_in(ast))
            }
            ForStatementLeftData::AssignmentTarget(it) => Self::AssignmentTarget(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ArrowFunctionBody {
    type Cloned = ArrowFunctionBody;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.arrow_function_body_data(*self);
        match data {
            ArrowFunctionBodyData::FunctionBody(it) => Self::FunctionBody(it.clone_in(ast)),
            ArrowFunctionBodyData::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for FormalParameterItem {
    type Cloned = FormalParameterItem;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.formal_parameter_item_data(*self);
        match data {
            FormalParameterItemData::FormalParameter(it) => Self::FormalParameter(it.clone_in(ast)),
            FormalParameterItemData::TsParameterProperty(it) => {
                Self::TsParameterProperty(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for FormalParameterPattern {
    type Cloned = FormalParameterPattern;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.formal_parameter_pattern_data(*self);
        match data {
            FormalParameterPatternData::BindingPattern(it) => {
                Self::BindingPattern(it.clone_in(ast))
            }
            FormalParameterPatternData::TsThisParameter(it) => {
                Self::TsThisParameter(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for ClassElement {
    type Cloned = ClassElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.class_element_data(*self);
        match data {
            ClassElementData::MethodDefinition(it) => Self::MethodDefinition(it.clone_in(ast)),
            ClassElementData::PropertyDefinition(it) => Self::PropertyDefinition(it.clone_in(ast)),
            ClassElementData::StaticBlock(it) => Self::StaticBlock(it.clone_in(ast)),
            ClassElementData::TsMethodDefinition(it) => Self::TsMethodDefinition(it.clone_in(ast)),
            ClassElementData::TsIndexSignature(it) => Self::TsIndexSignature(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ImportDeclarationSpecifier {
    type Cloned = ImportDeclarationSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.import_declaration_specifier_data(*self);
        match data {
            ImportDeclarationSpecifierData::ImportSpecifier(it) => {
                Self::ImportSpecifier(it.clone_in(ast))
            }
            ImportDeclarationSpecifierData::ImportDefaultSpecifier(it) => {
                Self::ImportDefaultSpecifier(it.clone_in(ast))
            }
            ImportDeclarationSpecifierData::ImportNamespaceSpecifier(it) => {
                Self::ImportNamespaceSpecifier(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for ExportDefaultDeclarationKind {
    type Cloned = ExportDefaultDeclarationKind;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.export_default_declaration_kind_data(*self);
        match data {
            ExportDefaultDeclarationKindData::Expr(it) => Self::Expr(it.clone_in(ast)),
            ExportDefaultDeclarationKindData::Function(it) => Self::Function(it.clone_in(ast)),
            ExportDefaultDeclarationKindData::Class(it) => Self::Class(it.clone_in(ast)),
            ExportDefaultDeclarationKindData::TsFunction(it) => Self::TsFunction(it.clone_in(ast)),
            ExportDefaultDeclarationKindData::TsInterfaceDeclaration(it) => {
                Self::TsInterfaceDeclaration(it.clone_in(ast))
            }
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
        let kind = self.kind(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.numeric_literal(span, kind, raw)
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
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.assignment_pattern(span, left, right, decorators, type_annotation, optional)
    }
}
impl<'a> CloneIn<'a> for BindingRestElement {
    type Cloned = BindingRestElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.binding_rest_element(span, argument, decorators, type_annotation, optional)
    }
}
impl<'a> CloneIn<'a> for ArrayPattern {
    type Cloned = ArrayPattern;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let elements = self.elements(ast).clone_in(ast);
        let rest = self.rest(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.array_pattern(span, elements, rest, decorators, type_annotation, optional)
    }
}
impl<'a> CloneIn<'a> for ObjectPattern {
    type Cloned = ObjectPattern;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let properties = self.properties(ast).clone_in(ast);
        let rest = self.rest(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.object_pattern(
            span,
            properties,
            rest,
            decorators,
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
        let super_class = self.super_class(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let super_type_arguments = self.super_type_arguments(ast).clone_in(ast);
        let implements = self.implements(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.class(
            span,
            class_type,
            decorators,
            id,
            super_class,
            body,
            type_parameters,
            super_type_arguments,
            implements,
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
        let value = self.value(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let r#static = self.r#static(ast).clone_in(ast);
        let accessor = self.accessor(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
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
            value,
            computed,
            r#static,
            accessor,
            type_annotation,
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
        let generator = self.generator(ast).clone_in(ast);
        let r#async = self.r#async(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        ast.function(
            span,
            function_type,
            id,
            generator,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
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
        let params = self.params(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        ast.arrow_function_expression(
            span,
            expression,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
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
        let arguments = self.arguments(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.call_expression(span, callee, arguments, optional, type_arguments)
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
        let quasi = self.quasi(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.tagged_template_expression(span, tag, quasi, type_arguments)
    }
}
impl<'a> CloneIn<'a> for NewExpression {
    type Cloned = NewExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let arguments = self.arguments(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.new_expression(span, callee, arguments, type_arguments)
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
        ast.import_expression(span, source, options, phase)
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
        let data = ast.ts_type_data(*self);
        match data {
            TsTypeData::TsAnyKeyword(it) => Self::TsAnyKeyword(it.clone_in(ast)),
            TsTypeData::TsUnknownKeyword(it) => Self::TsUnknownKeyword(it.clone_in(ast)),
            TsTypeData::TsNeverKeyword(it) => Self::TsNeverKeyword(it.clone_in(ast)),
            TsTypeData::TsVoidKeyword(it) => Self::TsVoidKeyword(it.clone_in(ast)),
            TsTypeData::TsNullKeyword(it) => Self::TsNullKeyword(it.clone_in(ast)),
            TsTypeData::TsUndefinedKeyword(it) => Self::TsUndefinedKeyword(it.clone_in(ast)),
            TsTypeData::TsStringKeyword(it) => Self::TsStringKeyword(it.clone_in(ast)),
            TsTypeData::TsNumberKeyword(it) => Self::TsNumberKeyword(it.clone_in(ast)),
            TsTypeData::TsBigIntKeyword(it) => Self::TsBigIntKeyword(it.clone_in(ast)),
            TsTypeData::TsBooleanKeyword(it) => Self::TsBooleanKeyword(it.clone_in(ast)),
            TsTypeData::TsSymbolKeyword(it) => Self::TsSymbolKeyword(it.clone_in(ast)),
            TsTypeData::TsObjectKeyword(it) => Self::TsObjectKeyword(it.clone_in(ast)),
            TsTypeData::TsIntrinsicKeyword(it) => Self::TsIntrinsicKeyword(it.clone_in(ast)),
            TsTypeData::TsThisType(it) => Self::TsThisType(it.clone_in(ast)),
            TsTypeData::TsTypeReference(it) => Self::TsTypeReference(it.clone_in(ast)),
            TsTypeData::TsTypeQuery(it) => Self::TsTypeQuery(it.clone_in(ast)),
            TsTypeData::TsImportType(it) => Self::TsImportType(it.clone_in(ast)),
            TsTypeData::TsLiteralType(it) => Self::TsLiteralType(it.clone_in(ast)),
            TsTypeData::TsTemplateLiteralType(it) => Self::TsTemplateLiteralType(it.clone_in(ast)),
            TsTypeData::TsArrayType(it) => Self::TsArrayType(it.clone_in(ast)),
            TsTypeData::TsIndexedAccessType(it) => Self::TsIndexedAccessType(it.clone_in(ast)),
            TsTypeData::TsTupleType(it) => Self::TsTupleType(it.clone_in(ast)),
            TsTypeData::TsOptionalType(it) => Self::TsOptionalType(it.clone_in(ast)),
            TsTypeData::TsRestType(it) => Self::TsRestType(it.clone_in(ast)),
            TsTypeData::TsJsDocNullableType(it) => Self::TsJsDocNullableType(it.clone_in(ast)),
            TsTypeData::TsJsDocNonNullableType(it) => {
                Self::TsJsDocNonNullableType(it.clone_in(ast))
            }
            TsTypeData::TsJsDocUnknownType(it) => Self::TsJsDocUnknownType(it.clone_in(ast)),
            TsTypeData::TsUnionType(it) => Self::TsUnionType(it.clone_in(ast)),
            TsTypeData::TsIntersectionType(it) => Self::TsIntersectionType(it.clone_in(ast)),
            TsTypeData::TsConditionalType(it) => Self::TsConditionalType(it.clone_in(ast)),
            TsTypeData::TsInferType(it) => Self::TsInferType(it.clone_in(ast)),
            TsTypeData::TsTypeOperator(it) => Self::TsTypeOperator(it.clone_in(ast)),
            TsTypeData::TsParenthesizedType(it) => Self::TsParenthesizedType(it.clone_in(ast)),
            TsTypeData::TsFunctionType(it) => Self::TsFunctionType(it.clone_in(ast)),
            TsTypeData::TsConstructorType(it) => Self::TsConstructorType(it.clone_in(ast)),
            TsTypeData::TsTypePredicate(it) => Self::TsTypePredicate(it.clone_in(ast)),
            TsTypeData::TsTypeLiteral(it) => Self::TsTypeLiteral(it.clone_in(ast)),
            TsTypeData::TsMappedType(it) => Self::TsMappedType(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for TsTypeName {
    type Cloned = TsTypeName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_type_name_data(*self);
        match data {
            TsTypeNameData::IdentifierReference(it) => Self::IdentifierReference(it.clone_in(ast)),
            TsTypeNameData::TsQualifiedName(it) => Self::TsQualifiedName(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for TsQualifiedNameLeft {
    type Cloned = TsQualifiedNameLeft;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_qualified_name_left_data(*self);
        match data {
            TsQualifiedNameLeftData::IdentifierReference(it) => {
                Self::IdentifierReference(it.clone_in(ast))
            }
            TsQualifiedNameLeftData::IdentifierName(it) => Self::IdentifierName(it.clone_in(ast)),
            TsQualifiedNameLeftData::BindingIdentifier(it) => {
                Self::BindingIdentifier(it.clone_in(ast))
            }
            TsQualifiedNameLeftData::ThisExpression(it) => Self::ThisExpression(it.clone_in(ast)),
            TsQualifiedNameLeftData::TsQualifiedName(it) => Self::TsQualifiedName(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for TsImportTypeQualifier {
    type Cloned = TsImportTypeQualifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_import_type_qualifier_data(*self);
        match data {
            TsImportTypeQualifierData::IdentifierName(it) => Self::IdentifierName(it.clone_in(ast)),
            TsImportTypeQualifierData::TsQualifiedName(it) => {
                Self::TsQualifiedName(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for TsTypeQueryExprName {
    type Cloned = TsTypeQueryExprName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_type_query_expr_name_data(*self);
        match data {
            TsTypeQueryExprNameData::TsTypeName(it) => Self::TsTypeName(it.clone_in(ast)),
            TsTypeQueryExprNameData::ThisExpression(it) => Self::ThisExpression(it.clone_in(ast)),
            TsTypeQueryExprNameData::TsImportType(it) => Self::TsImportType(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for TsLiteral {
    type Cloned = TsLiteral;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_literal_data(*self);
        match data {
            TsLiteralData::StringLiteral(it) => Self::StringLiteral(it.clone_in(ast)),
            TsLiteralData::NumericLiteral(it) => Self::NumericLiteral(it.clone_in(ast)),
            TsLiteralData::BigIntLiteral(it) => Self::BigIntLiteral(it.clone_in(ast)),
            TsLiteralData::BooleanLiteral(it) => Self::BooleanLiteral(it.clone_in(ast)),
            TsLiteralData::TemplateLiteral(it) => Self::TemplateLiteral(it.clone_in(ast)),
            TsLiteralData::UnaryExpression(it) => Self::UnaryExpression(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for TsTupleElement {
    type Cloned = TsTupleElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_tuple_element_data(*self);
        match data {
            TsTupleElementData::TsType(it) => Self::TsType(it.clone_in(ast)),
            TsTupleElementData::TsNamedTupleMember(it) => {
                Self::TsNamedTupleMember(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for TsTypeElement {
    type Cloned = TsTypeElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_type_element_data(*self);
        match data {
            TsTypeElementData::TsPropertySignature(it) => {
                Self::TsPropertySignature(it.clone_in(ast))
            }
            TsTypeElementData::TsMethodSignature(it) => Self::TsMethodSignature(it.clone_in(ast)),
            TsTypeElementData::TsCallSignatureDeclaration(it) => {
                Self::TsCallSignatureDeclaration(it.clone_in(ast))
            }
            TsTypeElementData::TsConstructSignatureDeclaration(it) => {
                Self::TsConstructSignatureDeclaration(it.clone_in(ast))
            }
            TsTypeElementData::TsIndexSignature(it) => Self::TsIndexSignature(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for TsTypePredicateName {
    type Cloned = TsTypePredicateName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_type_predicate_name_data(*self);
        match data {
            TsTypePredicateNameData::IdentifierName(it) => Self::IdentifierName(it.clone_in(ast)),
            TsTypePredicateNameData::TsThisType(it) => Self::TsThisType(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for TsEnumMemberName {
    type Cloned = TsEnumMemberName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_enum_member_name_data(*self);
        match data {
            TsEnumMemberNameData::IdentifierName(it) => Self::IdentifierName(it.clone_in(ast)),
            TsEnumMemberNameData::StringLiteral(it) => Self::StringLiteral(it.clone_in(ast)),
            TsEnumMemberNameData::TemplateLiteral(it) => Self::TemplateLiteral(it.clone_in(ast)),
            TsEnumMemberNameData::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for TsModuleName {
    type Cloned = TsModuleName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_module_name_data(*self);
        match data {
            TsModuleNameData::BindingIdentifier(it) => Self::BindingIdentifier(it.clone_in(ast)),
            TsModuleNameData::StringLiteral(it) => Self::StringLiteral(it.clone_in(ast)),
            TsModuleNameData::TsQualifiedName(it) => Self::TsQualifiedName(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for TsModuleBody {
    type Cloned = TsModuleBody;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_module_body_data(*self);
        match data {
            TsModuleBodyData::TsModuleBlock(it) => Self::TsModuleBlock(it.clone_in(ast)),
            TsModuleBodyData::TsModuleDeclaration(it) => {
                Self::TsModuleDeclaration(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for TsModuleReference {
    type Cloned = TsModuleReference;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.ts_module_reference_data(*self);
        match data {
            TsModuleReferenceData::TsTypeName(it) => Self::TsTypeName(it.clone_in(ast)),
            TsModuleReferenceData::TsExternalModuleReference(it) => {
                Self::TsExternalModuleReference(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for TsFunction {
    type Cloned = TsFunction;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let generator = self.generator(ast).clone_in(ast);
        let r#async = self.r#async(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        ast.ts_function(
            span,
            id,
            generator,
            r#async,
            declare,
            params,
            type_parameters,
            return_type,
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
        let data = ast.jsx_element_name_data(*self);
        match data {
            JsxElementNameData::JsxIdentifier(it) => Self::JsxIdentifier(it.clone_in(ast)),
            JsxElementNameData::JsxNamespacedName(it) => Self::JsxNamespacedName(it.clone_in(ast)),
            JsxElementNameData::JsxMemberExpression(it) => {
                Self::JsxMemberExpression(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for JsxMemberExpressionObject {
    type Cloned = JsxMemberExpressionObject;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_member_expression_object_data(*self);
        match data {
            JsxMemberExpressionObjectData::JsxIdentifier(it) => {
                Self::JsxIdentifier(it.clone_in(ast))
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(it) => {
                Self::JsxMemberExpression(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for JsxAttributeName {
    type Cloned = JsxAttributeName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_attribute_name_data(*self);
        match data {
            JsxAttributeNameData::JsxIdentifier(it) => Self::JsxIdentifier(it.clone_in(ast)),
            JsxAttributeNameData::JsxNamespacedName(it) => {
                Self::JsxNamespacedName(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for JsxAttributeItem {
    type Cloned = JsxAttributeItem;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_attribute_item_data(*self);
        match data {
            JsxAttributeItemData::JsxAttribute(it) => Self::JsxAttribute(it.clone_in(ast)),
            JsxAttributeItemData::JsxSpreadAttribute(it) => {
                Self::JsxSpreadAttribute(it.clone_in(ast))
            }
        }
    }
}
impl<'a> CloneIn<'a> for JsxAttributeValue {
    type Cloned = JsxAttributeValue;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_attribute_value_data(*self);
        match data {
            JsxAttributeValueData::StringLiteral(it) => Self::StringLiteral(it.clone_in(ast)),
            JsxAttributeValueData::JsxExpressionContainer(it) => {
                Self::JsxExpressionContainer(it.clone_in(ast))
            }
            JsxAttributeValueData::JsxElement(it) => Self::JsxElement(it.clone_in(ast)),
            JsxAttributeValueData::JsxFragment(it) => Self::JsxFragment(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for JsxExpression {
    type Cloned = JsxExpression;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_expression_data(*self);
        match data {
            JsxExpressionData::Expr(it) => Self::Expr(it.clone_in(ast)),
            JsxExpressionData::JsxEmptyExpression(it) => Self::JsxEmptyExpression(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for JsxChild {
    type Cloned = JsxChild;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_child_data(*self);
        match data {
            JsxChildData::JsxText(it) => Self::JsxText(it.clone_in(ast)),
            JsxChildData::JsxExpressionContainer(it) => {
                Self::JsxExpressionContainer(it.clone_in(ast))
            }
            JsxChildData::JsxSpreadChild(it) => Self::JsxSpreadChild(it.clone_in(ast)),
            JsxChildData::JsxElement(it) => Self::JsxElement(it.clone_in(ast)),
            JsxChildData::JsxFragment(it) => Self::JsxFragment(it.clone_in(ast)),
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
        let attributes = self.attributes(ast).clone_in(ast);
        let self_closing = self.self_closing(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.jsx_opening_element(span, name, attributes, self_closing, type_arguments)
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
