//! Construct helper AST nodes through TransformCtx so their bindings and references
//! are registered before the generated declarations are attached.
//! The first use prepares and caches the declaration; Program exit queues it,
//! and statement injection attaches it to the program body.

use super::TransformCtx;
use crate::{
    context::{semantic::BoundIdentifier, statement_injector::Anchor},
    helpers::{HelperKind, HelperMode, PreparedHelper},
};
use swc_next_ecma_ast::{
    Argument, DUMMY_SP, Expr, ImportDeclarationSpecifier, ImportOrExportKind, ModuleExportName,
    Program, ScopeId, Stmt, SubRange,
};
use swc_next_ecma_semantic::SymbolFlags;

impl TransformCtx<'_, '_> {
    /// Build a callee reference in `scope`, preparing its program-level binding
    /// and declaration on first use. Later calls reuse that binding.
    /// The declaration stays cached until `inject_helpers` queues it.
    ///
    /// Example callees (assuming the generated names are available):
    /// ```js
    /// _set_function_name
    /// _to_property_key
    /// ```
    pub(crate) fn helper_callee(&mut self, helper: HelperKind, scope: ScopeId) -> Expr {
        let bound_identifier = self.ensure_helper(helper);
        Expr::IdentifierReference(bound_identifier.reference_read(self, scope, DUMMY_SP))
    }

    pub(super) fn ensure_helper(&mut self, helper: HelperKind) -> BoundIdentifier {
        match self.state.helpers.bound_identifier(helper) {
            Some(bound_identifier) => bound_identifier,
            None => {
                assert!(
                    !self.state.helpers.preparing[helper as usize],
                    "cyclic helper imports"
                );
                self.state.helpers.preparing[helper as usize] = true;
                let prepared =
                    if self.state.helpers.mode == HelperMode::External && self.ast.is_module() {
                        self.prepare_external_helper(helper)
                    } else {
                        self.prepare_inline_helper(helper)
                    };
                let bound_identifier = prepared.bound_identifier;
                self.state.helpers.insert(helper, prepared);
                self.state.helpers.preparing[helper as usize] = false;
                bound_identifier
            }
        }
    }

    /// Allocate a collision-free name and register its binding in program scope,
    /// then build the detached import declaration. This allocates AST
    /// nodes and semantic information without attaching the declaration.
    ///
    /// For example, if `_to_property_key` already exists, prepare:
    /// ```js
    /// import { _to_property_key as _to_property_key2 } from "@swc-next/helpers";
    /// ```
    fn prepare_external_helper(&mut self, helper: HelperKind) -> PreparedHelper {
        let bound_identifier = BoundIdentifier::named(
            self,
            self.program_scope(),
            helper.name(),
            SymbolFlags::IMPORT,
        );
        let export = self.ast.add_utf8(helper.name());
        let imported = ModuleExportName::IdentifierName(
            self.add_node(|ast| ast.identifier_name(DUMMY_SP, export)),
        );
        let specifier = ImportDeclarationSpecifier::ImportSpecifier(self.add_node(|ast| {
            ast.import_specifier(
                DUMMY_SP,
                imported,
                bound_identifier.identifier,
                ImportOrExportKind::Value,
            )
        }));
        let specifiers = self.ast.add_sub_range([specifier]);
        let source = self.ast.add_wtf8(swc_next_helpers::MODULE.into());
        let source = self.add_node(|ast| ast.string_literal(DUMMY_SP, source, ast.empty_utf8()));
        let statement = Stmt::ImportDeclaration(self.add_node(|ast| {
            ast.import_declaration(
                DUMMY_SP,
                specifiers,
                source,
                SubRange::empty(),
                None,
                ImportOrExportKind::Value,
            )
        }));
        PreparedHelper {
            bound_identifier,
            statement,
        }
    }

    /// Drain cached helper declarations into the program's statement queue.
    /// This queues attachment only; `flush_statements` later prepends them.
    /// Repeating this call without preparing more helpers queues nothing.
    ///
    /// Module output after flushing a used helper:
    /// ```js
    /// import { _set_function_name } from "@swc-next/helpers";
    /// const C = _set_function_name(class {}, "C");
    /// ```
    pub(crate) fn inject_helpers(&mut self, program: Program) {
        let statements: Vec<_> = self.state.helpers.take_statements().collect();
        for statement in statements {
            self.inject_statement(program.node_id(), statement, Anchor::First);
        }
    }

    /// Build a call that gives `target` the supplied runtime name, using the
    /// supplied evaluation scope. Reuse or prepare the helper binding.
    /// Both arguments are existing expressions, retained in their original order.
    ///
    /// For `target = class {}` and `name = "C"`, build:
    /// ```js
    /// _set_function_name(class {}, "C")
    /// ```
    pub(crate) fn call_set_function_name(
        &mut self,
        target: Expr,
        name: Expr,
        scope: ScopeId,
    ) -> Expr {
        let callee = self.helper_callee(HelperKind::SetFunctionName, scope);
        self.call(callee, [Argument::Expr(target), Argument::Expr(name)])
    }

    /// Build a property-key conversion call in the supplied evaluation scope.
    /// Reuse or prepare the helper binding; the argument appears once.
    ///
    /// For `argument = getKey()`, build:
    /// ```js
    /// _to_property_key(getKey())
    /// ```
    pub(crate) fn call_to_property_key(&mut self, argument: Expr, scope: ScopeId) -> Expr {
        let callee = self.helper_callee(HelperKind::ToPropertyKey, scope);
        self.call(callee, [Argument::Expr(argument)])
    }

    /// Wrap an existing callee and ordered arguments in a normal call expression,
    /// growing its identity indexes. The callee and
    /// arguments must already have their semantic information registered.
    pub(super) fn call<const N: usize>(&mut self, callee: Expr, arguments: [Argument; N]) -> Expr {
        let arguments = self.ast.add_sub_range(arguments);
        Expr::CallExpression(
            self.add_node(|ast| {
                ast.call_expression(DUMMY_SP, callee, None, arguments, false, false)
            }),
        )
    }
}
