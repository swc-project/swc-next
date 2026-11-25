use swc_next_ecma_ast::*;

use crate::common::build;

/// SWC's top_level_this skips ordinary functions and visits only computed
/// method/field names. Arrow functions inherit their surrounding this.
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/top_level_this.rs#L9>.
pub(super) fn top_level_this(ast: &mut Ast<'_>) {
    ast.root_program().visit_mut_with(&mut TopLevelThis { ast });
}

struct TopLevelThis<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
}

impl<'ast> VisitMut<'ast> for TopLevelThis<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_function(&mut self, _: Function) -> Option<Function> {
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/top_level_this.rs#L26>.
    fn visit_mut_class_element(&mut self, element: ClassElement) -> Option<ClassElement> {
        match self.ast.class_element_data(element) {
            ClassElementData::MethodDefinition(method) if method.computed(self.ast) => {
                let key = method.key(self.ast);
                if let Some(key) = key.visit_mut_with(self) {
                    method.set_key(self.ast, key);
                }
            }
            ClassElementData::PropertyDefinition(field) if field.computed(self.ast) => {
                let key = field.key(self.ast);
                if let Some(key) = key.visit_mut_with(self) {
                    field.set_key(self.ast, key);
                }
            }
            _ => {}
        }
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/top_level_this.rs#L42>.
    fn visit_mut_object_property(&mut self, property: ObjectProperty) -> Option<ObjectProperty> {
        if property.method(self.ast) || property.kind(self.ast) != PropertyKind::Init {
            if property.computed(self.ast) {
                let key = property.key(self.ast);
                if let Some(key) = key.visit_mut_with(self) {
                    property.set_key(self.ast, key);
                }
            }
            return None;
        }
        property.visit_mut_children_with(self)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/top_level_this.rs#L63>.
    fn visit_mut_expr(&mut self, expr: Expr) -> Option<Expr> {
        if expr.is_this_expression(self.ast) {
            let replacement = build::undefined(self.ast);
            replacement.set_span(self.ast, expr.span(self.ast));
            return Some(replacement);
        }
        expr.visit_mut_children_with(self)
    }
}
