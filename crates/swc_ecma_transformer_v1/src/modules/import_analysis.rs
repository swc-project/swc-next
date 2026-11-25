use swc_next_ecma_ast::*;

use crate::common::TraverseCtx;

use super::{ModuleType, module_record::SourceModule};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/import_analysis.rs#L12>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/import_analysis.rs#L41>.
pub(super) fn import_analyzer(ast: &mut Ast<'_>, ctx: &mut TraverseCtx<'_>, format: ModuleType) {
    let source = SourceModule::collect(ast, ctx.resolver);
    for request in source.requested_modules.values() {
        if request.usage.has_star_export() {
            ctx.helpers.enable(ctx.resolver, "export_star");
        }
        if let Some(helper) = request.usage.interop_require_helper() {
            ctx.helpers.enable(ctx.resolver, helper);
        }
    }

    if matches!(format, ModuleType::CommonJs | ModuleType::Amd) {
        let mut visitor = ImportAnalyzer {
            ast,
            dynamic_import_found: false,
        };
        visitor.ast.root_program().visit_with(&mut visitor);
        if visitor.dynamic_import_found {
            ctx.helpers.enable(ctx.resolver, "interop_require_wildcard");
        }
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/import_analysis.rs#L21>.
struct ImportAnalyzer<'a, 'ast> {
    ast: &'a Ast<'ast>,
    dynamic_import_found: bool,
}

impl<'ast> Visit<'ast> for ImportAnalyzer<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/import_analysis.rs#L112>.
    fn visit_import_expression(&mut self, import: ImportExpression) {
        if import.phase(self.ast).is_none() && !import.sync(self.ast) {
            self.dynamic_import_found = true;
        } else if !self.dynamic_import_found {
            import.visit_children_with(self);
        }
    }
}
