//! Construct helper AST nodes through TransformCtx so their bindings and references
//! are registered before the generated import or require statement is attached.
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
    /// and external declaration on first use. Later calls reuse that binding.
    /// The declaration stays cached until `inject_helpers` queues it.
    ///
    /// Example callees (assuming the generated names are available):
    /// ```js
    /// _set_function_name              // SetFunctionName: default export
    /// _swc_helpers._to_property_key   // ToPropertyKey: namespace member
    /// ```
    pub(crate) fn helper_callee(&mut self, helper: HelperKind, scope: ScopeId) -> Expr {
        let bound_identifier = match self.state.helpers.bound_identifier(helper) {
            Some(bound_identifier) => bound_identifier,
            None => {
                let prepared = self.prepare_helper(helper);
                let bound_identifier = prepared.bound_identifier;
                self.state.helpers.insert(helper, prepared);
                bound_identifier
            }
        };
        match helper {
            HelperKind::ToPropertyKey => {
                self.external_member(bound_identifier, "_to_property_key", scope)
            }
            HelperKind::SetFunctionName => {
                Expr::IdentifierReference(bound_identifier.reference_read(self, scope, DUMMY_SP))
            }
        }
    }

    /// Allocate a collision-free name and register its binding in program scope,
    /// then build the detached import or require declaration. This allocates AST
    /// nodes and semantic information without attaching the declaration.
    ///
    /// For example, if `_swc_helpers` already exists, prepare:
    /// ```js
    /// import * as _swc_helpers2 from "@swc/helpers";
    /// ```
    fn prepare_helper(&mut self, helper: HelperKind) -> PreparedHelper {
        let base = match helper {
            HelperKind::SetFunctionName => "_set_function_name",
            HelperKind::ToPropertyKey => "_swc_helpers",
        };
        let flags = if self.ast.is_module() {
            SymbolFlags::IMPORT
        } else {
            SymbolFlags::FUNCTION_SCOPED_VAR
        };
        let bound_identifier = BoundIdentifier::named(self, self.program_scope(), base, flags);
        let statement = self.external_import(bound_identifier, helper);
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
    /// import _set_function_name from "@babel/runtime/helpers/setFunctionName";
    /// const C = _set_function_name(class {}, "C");
    /// ```
    pub(crate) fn inject_helpers(&mut self, program: Program) {
        let statements = self.state.helpers.take_statements();
        for statement in statements.into_iter().flatten() {
            self.inject_statement(program.node_id(), statement, Anchor::First);
        }
    }

    /// Build a call that gives `target` the supplied runtime name, using the
    /// supplied evaluation scope. Reuse or prepare the external helper binding.
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
    /// Reuse or prepare the helper namespace; the argument appears once.
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
    ///
    /// For a `require` callee and one string argument, build:
    /// ```js
    /// require("@swc/helpers")
    /// ```
    fn call<const N: usize>(&mut self, callee: Expr, arguments: [Argument; N]) -> Expr {
        let arguments = self.ast.add_sub_range(arguments);
        Expr::CallExpression(
            self.add_node(|ast| {
                ast.call_expression(DUMMY_SP, callee, None, arguments, false, false)
            }),
        )
    }

    /// Build a noncomputed member access on a generated helper namespace and
    /// register its namespace reference in `scope`. This builds the callee only.
    ///
    /// For `namespace = _swc_helpers` and `export = "_to_property_key"`, build:
    /// ```js
    /// _swc_helpers._to_property_key
    /// ```
    fn external_member(
        &mut self,
        namespace: BoundIdentifier,
        export: &str,
        scope: ScopeId,
    ) -> Expr {
        let object = Expr::IdentifierReference(namespace.reference_read(self, scope, DUMMY_SP));
        let property = self.ast.add_utf8(export);
        let property = PropertyKey::IdentifierName(
            self.add_node(|ast| ast.identifier_name(DUMMY_SP, property)),
        );
        Expr::MemberExpression(
            self.add_node(|ast| ast.member_expression(DUMMY_SP, object, property, false, false)),
        )
    }

    /// Build a detached external declaration using the already registered helper
    /// binding. Modules use a default or namespace import according to helper kind;
    /// scripts use a `var` initialized by `require`. Attachment happens later.
    ///
    /// Module forms:
    /// ```js
    /// import _set_function_name from "@babel/runtime/helpers/setFunctionName";
    /// import * as _swc_helpers from "@swc/helpers";
    /// ```
    /// Corresponding script forms:
    /// ```js
    /// var _set_function_name = require("@babel/runtime/helpers/setFunctionName");
    /// var _swc_helpers = require("@swc/helpers");
    /// ```
    fn external_import(&mut self, local: BoundIdentifier, helper: HelperKind) -> Stmt {
        let source = match helper {
            HelperKind::SetFunctionName => "@babel/runtime/helpers/setFunctionName",
            HelperKind::ToPropertyKey => "@swc/helpers",
        };
        let scope = self.program_scope();
        if !self.ast.is_module() {
            let name = self.state.helpers.require(self.ast);
            let require = self.ast.identifier_reference(DUMMY_SP, name);
            let target = self.semantic().binding(scope, b"require");
            self.register_reference(scope, require, target, Default::default());
            let require = Expr::IdentifierReference(require);
            let source = self.ast.add_wtf8(source.into());
            let source = Expr::StringLiteral(
                self.add_node(|ast| ast.string_literal(DUMMY_SP, source, ast.empty_utf8())),
            );
            let initializer = self.call(require, [Argument::Expr(source)]);
            return self.helper_var(local.identifier, initializer);
        }
        let specifier = match helper {
            HelperKind::SetFunctionName => ImportDeclarationSpecifier::ImportDefaultSpecifier(
                self.add_node(|ast| ast.import_default_specifier(DUMMY_SP, local.identifier)),
            ),
            HelperKind::ToPropertyKey => ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                self.add_node(|ast| ast.import_namespace_specifier(DUMMY_SP, local.identifier)),
            ),
        };
        let specifiers = self.ast.add_sub_range([specifier]);
        let source = self.ast.add_wtf8(source.into());
        let source = self.add_node(|ast| ast.string_literal(DUMMY_SP, source, ast.empty_utf8()));
        Stmt::ImportDeclaration(self.add_node(|ast| {
            ast.import_declaration(
                DUMMY_SP,
                specifiers,
                source,
                SubRange::empty(),
                None,
                ImportOrExportKind::Value,
            )
        }))
    }

    /// Wrap an existing helper binding and initializer in a detached `var`
    /// declaration, retaining the registered binding identifier and its symbol.
    ///
    /// For the `_swc_helpers` binding and a require initializer, build:
    /// ```js
    /// var _swc_helpers = require("@swc/helpers");
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
