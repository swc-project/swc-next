use swc_next_ecma_ast::*;

use crate::common::{build, resolver::context::Resolver};

use super::ir::LocalExports;

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/pattern.rs#L10>.
pub(super) fn replace_exported_pat(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    exports: &LocalExports,
    setters: BindingIdentifier,
    target: AssignmentTarget,
    changed: &mut bool,
) -> AssignmentTarget {
    let mut visitor = ExportedPattern {
        ast,
        resolver,
        exports,
        setters,
        changed,
    };
    target.visit_mut_with(&mut visitor).unwrap_or(target)
}

struct ExportedPattern<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    exports: &'a LocalExports,
    setters: BindingIdentifier,
    changed: &'a mut bool,
}

impl<'ast> VisitMut<'ast> for ExportedPattern<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, _: Expr) -> Option<Expr> {
        None
    }

    fn visit_mut_binding_property(&mut self, property: BindingProperty) -> Option<BindingProperty> {
        property.visit_mut_children_with(self);
        build::binding_shorthand(self.ast, property);
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/util.rs#L28>.
    fn visit_mut_simple_assignment_target(
        &mut self,
        target: SimpleAssignmentTarget,
    ) -> Option<SimpleAssignmentTarget> {
        let id = target.as_identifier_reference(self.ast)?;
        let symbol = self.resolver.symbol(id.node_id())?;
        if !self.exports.contains_key(&symbol) {
            return None;
        }

        *self.changed = true;
        let object = self.resolver.reference(self.ast, self.setters);
        let name = self.ast.get_utf8(self.resolver.symbol_name(symbol));
        let member = build::member(self.ast, object, name);
        Some(build::target(self.ast, member))
    }
}
