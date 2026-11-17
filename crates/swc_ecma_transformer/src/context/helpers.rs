//! Construct helper AST nodes through TransformCtx so their bindings and references
//! are registered before the generated declarations are attached.
//! The first use prepares and caches the declaration; Program exit queues it,
//! and statement injection attaches it to the program body.

use super::TransformCtx;
use crate::{
    context::{semantic::BoundIdentifier, statement_injector::Anchor},
    helpers::{HelperKind, PreparedHelper},
};
use swc_next_ecma_ast::{
    Argument, BindingIdentifier, BindingPattern, DUMMY_SP, Decl, Expr, ImportDeclarationSpecifier,
    ImportOrExportKind, Program, PropertyKey, ScopeId, Stmt, SubRange, VariableKind,
};
use swc_next_ecma_semantic::SymbolFlags;

impl TransformCtx<'_, '_> {
    /// Build a callee reference in `scope`, preparing its program-level binding
    /// and declaration on first use. Later calls reuse that binding.
    /// The declaration stays cached until `inject_helpers` queues it.
    ///
    /// Example callees (assuming the generated names are available):
    /// ```js
    /// _swc_helpers._set_function_name
    /// _swc_helpers._to_property_key
    /// ```
    pub(crate) fn helper_callee(&mut self, helper: HelperKind, scope: ScopeId) -> Expr {
        let namespace = match self.state.helpers.bound_identifier() {
            Some(namespace) => namespace,
            None => {
                let prepared = self.prepare_external_helpers();
                let namespace = prepared.bound_identifier;
                self.state.helpers.insert(prepared);
                namespace
            }
        };
        let object = Expr::IdentifierReference(namespace.reference_read(self, scope, DUMMY_SP));
        let name = self.ast.add_utf8(helper.name());
        let property =
            PropertyKey::IdentifierName(self.add_node(|ast| ast.identifier_name(DUMMY_SP, name)));
        Expr::MemberExpression(
            self.add_node(|ast| ast.member_expression(DUMMY_SP, object, property, false, false)),
        )
    }

    /// Allocate a collision-free name and register its binding in program scope,
    /// then build the detached import declaration. This allocates AST
    /// nodes and semantic information without attaching the declaration.
    ///
    /// For example, if `_swc_helpers` already exists, prepare:
    /// ```js
    /// import * as _swc_helpers2 from "@swc-next/helpers";
    /// ```
    fn prepare_external_helpers(&mut self) -> PreparedHelper {
        let bound_identifier = BoundIdentifier::named(
            self,
            self.program_scope(),
            "_swc_helpers",
            if self.ast.is_module() {
                SymbolFlags::IMPORT
            } else {
                SymbolFlags::FUNCTION_SCOPED_VAR
            },
        );
        if !self.ast.is_module() {
            let scope = self.program_scope();
            let name = self.ast.add_utf8("require");
            let require = self.add_node(|ast| ast.identifier_reference(DUMMY_SP, name));
            let target = self.semantic().binding(scope, b"require");
            self.register_reference(scope, require, target, Default::default());
            let source = self.ast.add_wtf8(swc_next_helpers::MODULE.into());
            let source = Expr::StringLiteral(
                self.add_node(|ast| ast.string_literal(DUMMY_SP, source, ast.empty_utf8())),
            );
            let initializer =
                self.call(Expr::IdentifierReference(require), [Argument::Expr(source)]);
            let statement = self.helper_var(bound_identifier.identifier, initializer);
            return PreparedHelper {
                bound_identifier,
                statement,
            };
        }
        let specifier =
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(self.add_node(|ast| {
                ast.import_namespace_specifier(DUMMY_SP, bound_identifier.identifier)
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

    /// Move the cached namespace declaration into the program's statement queue.
    /// This queues attachment only; `flush_statements` later prepends them.
    /// Repeating this call without preparing more helpers queues nothing.
    ///
    /// Module output after flushing a used helper:
    /// ```js
    /// import * as _swc_helpers from "@swc-next/helpers";
    /// const C = _swc_helpers._set_function_name(class {}, "C");
    /// ```
    pub(crate) fn inject_helpers(&mut self, program: Program) {
        if let Some(statement) = self.state.helpers.take_statement() {
            self.inject_statement(program.node_id(), statement, Anchor::First);
        }
    }

    /// Build a call that gives `target` the supplied runtime name, using the
    /// supplied evaluation scope. Reuse or prepare the helper binding.
    /// Both arguments are existing expressions, retained in their original order.
    ///
    /// For `target = class {}` and `name = "C"`, build:
    /// ```js
    /// _swc_helpers._set_function_name(class {}, "C")
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
    /// _swc_helpers._to_property_key(getKey())
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
    /// Wrap an existing helper binding and initializer in a detached `var`
    /// declaration, retaining the registered binding identifier and its symbol.
    ///
    /// For the `_swc_helpers` binding and a require initializer, build:
    /// ```js
    /// var _swc_helpers = require("@swc-next/helpers");
    /// ```
    fn helper_var(&mut self, binding: BindingIdentifier, init: Expr) -> Stmt {
        let declarator = self.add_node(|ast| {
            ast.variable_declarator(
                DUMMY_SP,
                BindingPattern::BindingIdentifier(binding),
                None,
                Some(init),
                false,
            )
        });
        let declarators = self.ast.add_sub_range([declarator]);
        Stmt::Declaration(Decl::VariableDeclaration(self.add_node(|ast| {
            ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false)
        })))
    }
}
