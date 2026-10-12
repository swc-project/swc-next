use swc_next_ecma_ast::{
    Ast, AstUtf8, DUMMY_SP, Decl, Expr, GetSpan, JsxAttributeItem, JsxAttributeNameData,
    JsxAttributeValue, JsxExpression, JsxOpeningElement, ObjectPropertyKind, Program, PropertyKey,
    PropertyKind, Stmt, SubRange, VariableDeclaration, VariableKind, VisitMut, VisitMutWith,
};
use swc_next_ecma_semantic::Semantic;

use crate::uid::UidGenerator;

use super::{self_context::JsxSelfContext, source::SourceLocations};

pub(crate) fn transform<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    jsx_self: bool,
    jsx_source: bool,
    source_path: Option<&str>,
) -> Option<VariableDeclaration> {
    let uid = UidGenerator::new(ast, semantic);
    let self_context = jsx_self.then(|| JsxSelfContext::collect(program, ast));
    let mut visitor = DevelopmentVisitor {
        ast,
        self_context,
        jsx_self,
        jsx_source,
        source_locations: SourceLocations::default(),
        source_path: source_path.unwrap_or_default(),
        uid,
        filename: None,
    };
    program.visit_mut_with(&mut visitor);
    visitor.insert_filename(program)
}

struct DevelopmentVisitor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    self_context: Option<JsxSelfContext>,
    jsx_self: bool,
    jsx_source: bool,
    source_locations: SourceLocations,
    source_path: &'visit str,
    uid: UidGenerator<'ast>,
    filename: Option<AstUtf8>,
}

impl<'ast> VisitMut<'ast> for DevelopmentVisitor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_jsx_opening_element(
        &mut self,
        element: JsxOpeningElement,
    ) -> Option<JsxOpeningElement> {
        element.visit_mut_children_with(self);
        let existing = element.attributes(self.ast);
        let add_self = self.jsx_self
            && !has_attribute(existing, "__self", self.ast)
            && self
                .self_context
                .as_ref()
                .is_some_and(|context| context.can_add_self(element.node_id()));
        let add_source = self.jsx_source
            && element.span(self.ast) != DUMMY_SP
            && !has_attribute(existing, "__source", self.ast);
        if !add_self && !add_source {
            return None;
        }
        let mut attributes =
            Vec::with_capacity(existing.len() + usize::from(add_self) + usize::from(add_source));
        attributes.extend(existing.iter().map(|id| self.ast.get_node_in_sub_range(id)));
        if add_self {
            let expression = self.ast.jsx_expression_expr_this_expression(DUMMY_SP);
            attributes.push(expression_attribute(self.ast, "__self", expression));
        }
        if add_source {
            let source = self.source_object(element.span(self.ast).start);
            attributes.push(expression_attribute(
                self.ast,
                "__source",
                JsxExpression::Expr(source),
            ));
        }
        let attributes = self.ast.add_sub_range(attributes);
        element.set_attributes(self.ast, attributes);
        None
    }
}

impl DevelopmentVisitor<'_, '_> {
    fn source_object(&mut self, offset: u32) -> Expr {
        let filename = if let Some(filename) = self.filename {
            filename
        } else {
            let filename = self.uid.fresh("_jsxFileName", self.ast);
            self.filename = Some(filename);
            filename
        };
        let (line, column) = self.source_locations.line_column(self.ast, offset);
        let filename = self.ast.expr_identifier_reference(DUMMY_SP, filename);
        let line = self
            .ast
            .expr_numeric_literal(DUMMY_SP, f64::from(line), None);
        let column = self
            .ast
            .expr_numeric_literal(DUMMY_SP, f64::from(column), None);
        let filename = object_property(self.ast, "fileName", filename);
        let line = object_property(self.ast, "lineNumber", line);
        let column = object_property(self.ast, "columnNumber", column);
        let properties = self.ast.add_sub_range([filename, line, column]);
        self.ast.expr_object_expression(DUMMY_SP, properties)
    }

    fn insert_filename(&mut self, program: Program) -> Option<VariableDeclaration> {
        let filename = self.filename?;
        let source_path = self.ast.add_utf8(self.source_path);
        let init = self
            .ast
            .expr_string_literal(DUMMY_SP, source_path.into(), AstUtf8::EMPTY);
        let id = self
            .ast
            .binding_pattern_binding_identifier(DUMMY_SP, filename);
        let declarator = self
            .ast
            .variable_declarator(DUMMY_SP, id, None, Some(init), false);
        let declarators = self.ast.add_sub_range([declarator]);
        let declaration =
            self.ast
                .variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false);
        let statement = Stmt::Declaration(Decl::VariableDeclaration(declaration));
        let existing = program.body(self.ast);
        let mut body = Vec::with_capacity(existing.len() + 1);
        body.push(statement);
        body.extend(existing.iter().map(|id| self.ast.get_node_in_sub_range(id)));
        let body = self.ast.add_sub_range(body);
        program.set_body(self.ast, body);
        Some(declaration)
    }
}

fn has_attribute(attributes: SubRange<JsxAttributeItem>, name: &str, ast: &Ast<'_>) -> bool {
    attributes.iter().any(|id| {
        let attribute = ast.get_node_in_sub_range(id);
        let Some(attribute) = attribute.as_jsx_attribute(ast) else {
            return false;
        };
        let JsxAttributeNameData::JsxIdentifier(identifier) =
            ast.jsx_attribute_name_data(attribute.name(ast))
        else {
            return false;
        };
        ast.get_utf8(identifier.name(ast)) == name
    })
}

fn expression_attribute(
    ast: &mut Ast<'_>,
    name: &str,
    expression: JsxExpression,
) -> JsxAttributeItem {
    let name = ast.add_utf8(name);
    let name = ast.jsx_attribute_name_jsx_identifier(DUMMY_SP, name);
    let value = JsxAttributeValue::JsxExpressionContainer(
        ast.jsx_expression_container(DUMMY_SP, expression),
    );
    ast.jsx_attribute_item_jsx_attribute(DUMMY_SP, name, Some(value))
}

fn object_property(ast: &mut Ast<'_>, name: &str, value: Expr) -> ObjectPropertyKind {
    let name = ast.add_utf8(name);
    let key = PropertyKey::IdentifierName(ast.identifier_name(DUMMY_SP, name));
    ObjectPropertyKind::ObjectProperty(ast.object_property(
        DUMMY_SP,
        key,
        value,
        PropertyKind::Init,
        false,
        false,
        false,
    ))
}
