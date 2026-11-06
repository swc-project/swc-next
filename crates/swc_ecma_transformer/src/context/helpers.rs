//! Construct helper AST nodes through TransformCtx so their bindings and references
//! are registered before the generated import or require statement is attached.
//! The first use prepares and caches the declaration; Program exit queues it,
//! and statement injection attaches it to the program body.

use super::TransformCtx;
use crate::{
    context::{semantic::BoundIdentifier, statement_injector::Anchor},
    helpers::{HelperKind, HelperMode, PreparedHelper},
};
use swc_next_ecma_ast::{
    Argument, BindingIdentifier, BindingPattern, DUMMY_SP, Decl, Expr, ImportDeclarationSpecifier,
    ImportOrExportKind, ModuleExportName, Program, PropertyKey, ScopeId, Stmt, SubRange,
    VariableKind,
};
use swc_next_ecma_semantic::SymbolFlags;

impl TransformCtx<'_, '_> {
    /// Build a callee reference in `scope`, preparing its program-level binding
    /// and external declaration on first use. Later calls reuse that binding.
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
                let prepared = match self.state.helpers.mode {
                    HelperMode::External => self.prepare_helper(helper),
                    HelperMode::Inline => self.prepare_inline_helper(helper),
                };
                let bound_identifier = prepared.bound_identifier;
                self.state.helpers.insert(helper, prepared);
                self.state.helpers.preparing[helper as usize] = false;
                bound_identifier
            }
        }
    }

    /// Allocate a collision-free name and register its binding in program scope,
    /// then build the detached import or require declaration. This allocates AST
    /// nodes and semantic information without attaching the declaration.
    ///
    /// For example, if `_to_property_key` already exists, prepare:
    /// ```js
    /// import { _to_property_key as _to_property_key2 } from "@swc-next/helpers";
    /// ```
    fn prepare_helper(&mut self, helper: HelperKind) -> PreparedHelper {
        let flags = if self.ast.is_module() {
            SymbolFlags::IMPORT
        } else {
            SymbolFlags::FUNCTION_SCOPED_VAR
        };
        let bound_identifier =
            BoundIdentifier::named(self, self.program_scope(), helper.name(), flags);
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
    /// import { _set_function_name } from "@swc-next/helpers";
    /// const C = _set_function_name(class {}, "C");
    /// ```
    pub(crate) fn inject_helpers(&mut self, program: Program) {
        self.protect_helper_globals();
        let statements: Vec<_> = self.state.helpers.take_statements().collect();
        for statement in statements {
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
    ///
    /// For a `require` callee and one string argument, build:
    /// ```js
    /// require("@swc-next/helpers")
    /// ```
    pub(super) fn call<const N: usize>(&mut self, callee: Expr, arguments: [Argument; N]) -> Expr {
        let arguments = self.ast.add_sub_range(arguments);
        Expr::CallExpression(
            self.add_node(|ast| {
                ast.call_expression(DUMMY_SP, callee, None, arguments, false, false)
            }),
        )
    }

    /// Build a detached external declaration using the already registered helper
    /// binding. Modules import the helper by name; scripts read the named export
    /// from `require`. Attachment happens later.
    ///
    /// Module forms:
    /// ```js
    /// import { _set_function_name } from "@swc-next/helpers";
    /// ```
    /// Corresponding script forms:
    /// ```js
    /// var _set_function_name = require("@swc-next/helpers")._set_function_name;
    /// ```
    fn external_import(&mut self, local: BoundIdentifier, helper: HelperKind) -> Stmt {
        let source = swc_next_helpers::MODULE;
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
            let export = self.ast.add_utf8(helper.name());
            let property = PropertyKey::IdentifierName(
                self.add_node(|ast| ast.identifier_name(DUMMY_SP, export)),
            );
            let initializer = Expr::MemberExpression(self.add_node(|ast| {
                ast.member_expression(DUMMY_SP, initializer, property, false, false)
            }));
            return self.helper_var(local.identifier, initializer);
        }
        let export = self.ast.add_utf8(helper.name());
        let imported = ModuleExportName::IdentifierName(
            self.add_node(|ast| ast.identifier_name(DUMMY_SP, export)),
        );
        let specifier = ImportDeclarationSpecifier::ImportSpecifier(self.add_node(|ast| {
            ast.import_specifier(
                DUMMY_SP,
                imported,
                local.identifier,
                ImportOrExportKind::Value,
            )
        }));
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
    /// For the `_to_property_key` binding and a require initializer, build:
    /// ```js
    /// var _to_property_key = require("@swc-next/helpers")._to_property_key;
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
