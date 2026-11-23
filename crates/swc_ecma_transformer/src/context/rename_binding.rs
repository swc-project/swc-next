//! Rename lexical bindings while keeping syntax and semantic indexes synchronized.

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, AssignmentPattern, Ast, AstUtf8, BindingIdentifier,
    BindingPattern, BindingPatternData, CallExpression, Class, ClassElementData, DUMMY_SP, Decl,
    DeclData, Diagnostic, Expr, ExprData, ImportOrExportKind, ModuleExportName, NodeId, NodeKind,
    NodeKindData, PropertyKeyData, ScopeId, Stmt, SubRange, SymbolId, VariableDeclarator, Visit,
    VisitWith,
};
use swc_next_ecma_semantic::{ReferenceSpace, ScopeModel, SymbolFlags, scope::ScopeKind};

mod names;

use super::{TransformCtx, unwrap_transparent_expr};

impl TransformCtx<'_, '_> {
    /// Rename bindings while preserving their semantic identities and observable names.
    ///
    /// Callers must provide distinct symbols. Unsafe bindings remain unchanged
    /// and produce diagnostics. Returns their symbols.
    pub(crate) fn rename_bindings(&mut self, bindings: Vec<SymbolId>) -> Vec<SymbolId> {
        if bindings.is_empty() {
            return Vec::new();
        }
        let program = self.ast.root_program();
        // Include expressions queued by earlier transforms in safety checks and rewrites.
        self.inject_statements_to(program.node_id());
        let is_global = self.semantic().scope(self.program_scope()).kind == ScopeKind::Global;
        let mut rejected = Vec::new();
        let mut pending = Vec::new();
        let mut candidates = FxHashSet::default();
        for symbol in bindings {
            let linked: Vec<_> = self.semantic().linked_symbols(symbol).collect();
            let original_text = self
                .ast
                .get_wtf8(self.semantic().symbol(symbol).name)
                .as_str()
                .expect("binding name");
            let reason = if is_global {
                Some("lookup from another script or indirect eval")
            } else if linked.iter().any(|&id| {
                self.semantic()
                    .symbol(id)
                    .flags
                    .contains(SymbolFlags::ANNEX_B_IF_FUNCTION)
            }) {
                Some("conditional function name")
            } else if linked.iter().any(|&id| {
                self.semantic()
                    .symbol(id)
                    .flags
                    .contains(SymbolFlags::FUNCTION)
                    && self.ast.node_kind(
                        self.semantic()
                            .scope(self.semantic().scope_of_symbol(id))
                            .node,
                    ) == NodeKind::SwitchStatement
            }) {
                Some("switch-scoped function name")
            } else {
                None
            };
            if let Some(reason) = reason {
                self.report_unsafe_rename(symbol, original_text, reason);
                rejected.extend(linked);
                continue;
            }
            candidates.extend(linked.iter().copied());
            pending.push((symbol, linked, original_text));
        }

        let mut renamed = FxHashMap::default();
        let mut updates = Vec::new();
        if !pending.is_empty() {
            let mut safety = SafetyCheck {
                ast: self.ast,
                semantic: self.semantic(),
                candidates: &candidates,
                check_eval: self
                    .state
                    .used_names
                    .as_ref()
                    .is_none_or(|names| names.contains(b"eval")),
                eval_scopes: FxHashSet::default(),
                observable_class_names: FxHashSet::default(),
            };
            program.visit_with(&mut safety);
            let SafetyCheck {
                eval_scopes,
                observable_class_names,
                ..
            } = safety;

            for (symbol, linked, original_text) in pending {
                let reason = self
                    .observable_local_binding(original_text, &linked, &eval_scopes)
                    .or_else(|| {
                        linked
                            .iter()
                            .any(|id| observable_class_names.contains(id))
                            .then_some("class initialization or static name member")
                    });
                if let Some(reason) = reason {
                    self.report_unsafe_rename(symbol, original_text, reason);
                    rejected.extend(linked);
                    continue;
                }
                let declaration = self
                    .semantic()
                    .declaration(self.semantic().symbol(symbol).declaration);
                let NodeKindData::BindingIdentifier(identifier) =
                    self.ast.kind_data(declaration.node)
                else {
                    panic!("renaming requires a binding identifier");
                };
                let original = identifier.name(self.ast);
                let name = self.generate_name(&format!("_{original_text}"), 2);
                for symbol in linked {
                    renamed.insert(symbol, original);
                    updates.push((symbol, name));
                }
            }
        }
        if !updates.is_empty() {
            for (symbol, name) in updates {
                self.rename_symbol(symbol, name);
            }
            self.preserve_binding_exports(&renamed);
            self.preserve_renamed_syntax(&renamed);
        }
        rejected
    }

    fn report_unsafe_rename(&mut self, symbol: SymbolId, original_text: &str, reason: &str) {
        let declaration = self
            .semantic()
            .declaration(self.semantic().symbol(symbol).declaration);
        let mut diagnostic = Diagnostic::error(
            "Cannot safely rename binding",
            self.ast.span(declaration.node),
        );
        diagnostic.help = Some(format!("Renaming `{original_text}` could change {reason}").into());
        self.state.diagnostics.push(diagnostic);
    }

    fn observable_local_binding(
        &self,
        name: &str,
        linked: &[SymbolId],
        eval_scopes: &FxHashSet<ScopeId>,
    ) -> Option<&'static str> {
        let semantic = self.semantic();
        let ancestors = |scope| std::iter::successors(Some(scope), |&id| semantic.scope(id).parent);
        if eval_scopes.iter().any(|&scope| {
            ancestors(scope)
                .filter_map(|id| semantic.binding(id, name.as_bytes()))
                .find(|&symbol| {
                    semantic
                        .symbol(symbol)
                        .flags
                        .visible_in(ReferenceSpace::Value)
                })
                .is_some_and(|symbol| linked.contains(&symbol))
        }) {
            return Some("lookup from direct eval");
        }
        if linked.iter().any(|&symbol| {
            let binding_scope = semantic.scope_of_symbol(symbol);
            semantic.uses(symbol).iter().any(|&id| {
                let reference = semantic.reference(id);
                if reference.flags.space != ReferenceSpace::Value {
                    return false;
                }
                ancestors(reference.scope)
                    .take_while(|&scope| scope != binding_scope)
                    .any(|scope| semantic.scope(scope).kind == ScopeKind::With)
            })
        }) {
            return Some("lookup through with");
        }
        None
    }

    fn rename_symbol(&mut self, symbol: SymbolId, name: AstUtf8) {
        let semantic = &self.state.semantic;
        let old_text = self.ast.get_wtf8(semantic.symbol(symbol).name).as_bytes();
        for declaration in semantic.declarations(symbol) {
            match self.ast.kind_data(declaration.node) {
                NodeKindData::BindingIdentifier(identifier) => identifier.set_name(self.ast, name),
                _ => panic!("renaming requires a binding identifier"),
            }
        }
        for &reference in semantic.uses(symbol) {
            let reference = semantic.reference(reference);
            if reference.symbol == Some(symbol) {
                match self.ast.kind_data(reference.node) {
                    NodeKindData::IdentifierReference(identifier) => {
                        identifier.set_name(self.ast, name)
                    }
                    NodeKindData::IdentifierName(identifier) => identifier.set_name(self.ast, name),
                    NodeKindData::JsxIdentifier(identifier) => identifier.set_name(self.ast, name),
                    _ => panic!("renaming requires an identifier reference"),
                }
            }
        }
        let text = self.ast.get_utf8(name);
        self.state.semantic.rename_symbol(symbol, old_text, name, text);
    }

    fn preserve_binding_exports(&mut self, renamed: &FxHashMap<SymbolId, AstUtf8>) {
        if !self.ast.is_module() {
            return;
        }
        let needs_update = renamed.keys().any(|&symbol| {
            self.semantic()
                .symbol(symbol)
                .flags
                .contains(SymbolFlags::EXPORTED)
                || self.semantic().uses(symbol).iter().any(|&id| {
                    matches!(
                        self.ast.kind_data(self.semantic().reference(id).node),
                        NodeKindData::IdentifierName(_)
                    )
                })
        });
        if !needs_update {
            return;
        }
        let program = self.ast.root_program();
        let scope = self.program_scope();
        let mut statements = Vec::new();
        let mut split = false;
        let mut seen = FxHashSet::default();
        let mut bindings = Vec::new();
        for slot in program.body(self.ast).iter() {
            let statement = self.ast.get_node_in_sub_range(slot);
            if let Some(export) = statement.as_export_named_declaration(self.ast) {
                if let Some(declaration) = export.declaration(self.ast) {
                    bindings.clear();
                    collect_export_decl_bindings(self.ast, declaration, &mut bindings);
                    let exported_bindings: Vec<_> = bindings
                        .iter()
                        .filter_map(|&identifier| {
                            let symbol = self.semantic().symbol_of(identifier.node_id())?;
                            let id = self.semantic().declaration_ids(symbol).find(|&id| {
                                self.semantic().declaration(id).node == identifier.node_id()
                            })?;
                            let decl = self.semantic().declaration(id);
                            (decl.scope == scope && decl.flags.contains(SymbolFlags::EXPORTED))
                                .then_some((id, symbol, identifier))
                        })
                        .collect();
                    if exported_bindings
                        .iter()
                        .any(|&(_, symbol, _)| renamed.contains_key(&symbol))
                    {
                        split = true;
                        let mut specifiers = Vec::new();
                        for (id, symbol, identifier) in exported_bindings {
                            self.state.semantic.clear_direct_export(id);
                            if !seen.insert(symbol) {
                                continue;
                            }
                            let name = renamed
                                .get(&symbol)
                                .copied()
                                .unwrap_or(identifier.name(self.ast));
                            let exported = ModuleExportName::IdentifierName(
                                self.add_node(|ast| ast.identifier_name(DUMMY_SP, name)),
                            );
                            let local_name = identifier.name(self.ast);
                            let local =
                                self.add_node(|ast| ast.identifier_name(DUMMY_SP, local_name));
                            self.register_export_reference(scope, local, symbol);
                            let local = ModuleExportName::IdentifierName(local);
                            specifiers.push(self.add_node(|ast| {
                                ast.export_specifier(
                                    DUMMY_SP,
                                    local,
                                    exported,
                                    ImportOrExportKind::Value,
                                )
                            }));
                        }
                        let decl_stmt = Stmt::Declaration(declaration);
                        self.inherits_comments(decl_stmt, statement);
                        statements.push(decl_stmt);
                        if !specifiers.is_empty() {
                            let specifiers = self.ast.add_sub_range(specifiers);
                            let export = self.add_node(|ast| {
                                ast.stmt_export_named_declaration(
                                    DUMMY_SP,
                                    None,
                                    specifiers,
                                    None,
                                    SubRange::empty(),
                                    ImportOrExportKind::Value,
                                )
                            });
                            statements.push(export);
                        }
                        continue;
                    }
                } else {
                    for slot in export.specifiers(self.ast).iter() {
                        let specifier = self.ast.get_node_in_sub_range(slot);
                        let local = specifier.local(self.ast).node_id();
                        // Parser shorthand exports can share the local/exported node.
                        if local == specifier.exported(self.ast).node_id()
                            && let Some(reference) = self.semantic().reference_of(local)
                            && let Some(symbol) = self.semantic().reference(reference).symbol
                            && let Some(&name) = renamed.get(&symbol)
                        {
                            let exported = ModuleExportName::IdentifierName(
                                self.add_node(|ast| ast.identifier_name(DUMMY_SP, name)),
                            );
                            specifier.set_exported(self.ast, exported);
                        }
                    }
                }
            }
            statements.push(statement);
        }
        if split {
            let body = self.ast.add_sub_range(statements);
            program.set_body(self.ast, body);
        }
    }
}

fn collect_export_decl_bindings(
    ast: &Ast<'_>,
    decl: Decl,
    bindings: &mut Vec<BindingIdentifier>,
) {
    match ast.decl_data(decl) {
        DeclData::Function(function) => bindings.extend(function.id(ast)),
        DeclData::Class(class) => bindings.extend(class.id(ast)),
        DeclData::VariableDeclaration(declaration) => {
            for slot in declaration.declarators(ast).iter() {
                let declarator = ast.get_node_in_sub_range(slot);
                collect_binding_pattern(ast, declarator.id(ast), bindings);
            }
        }
        _ => {}
    }
}

fn collect_binding_pattern(
    ast: &Ast<'_>,
    pattern: BindingPattern,
    bindings: &mut Vec<BindingIdentifier>,
) {
    match ast.binding_pattern_data(pattern) {
        BindingPatternData::BindingIdentifier(identifier) => bindings.push(identifier),
        BindingPatternData::SimpleAssignmentTarget(_) => {}
        BindingPatternData::AssignmentPattern(pattern) => {
            collect_binding_pattern(ast, pattern.left(ast), bindings);
        }
        BindingPatternData::BindingRestElement(rest) => {
            collect_binding_pattern(ast, rest.argument(ast), bindings);
        }
        BindingPatternData::ArrayPattern(pattern) => {
            for slot in pattern.elements(ast).iter() {
                if let Some(element) = ast.get_node_in_sub_range(slot) {
                    collect_binding_pattern(ast, element, bindings);
                }
            }
            if let Some(rest) = pattern.rest(ast) {
                collect_binding_pattern(ast, rest.argument(ast), bindings);
            }
        }
        BindingPatternData::ObjectPattern(pattern) => {
            for slot in pattern.properties(ast).iter() {
                let property = ast.get_node_in_sub_range(slot);
                collect_binding_pattern(ast, property.value(ast), bindings);
            }
            if let Some(rest) = pattern.rest(ast) {
                collect_binding_pattern(ast, rest.argument(ast), bindings);
            }
        }
    }
}

fn has_observable_class_name(ast: &Ast<'_>, class: Class) -> bool {
    class.body(ast).body(ast).iter().any(|slot| {
        match ast.class_element_data(ast.get_node_in_sub_range(slot)) {
            ClassElementData::StaticBlock(_) => true,
            ClassElementData::PropertyDefinition(property) => {
                property.r#static(ast) && !property.declare(ast) && !property.r#abstract(ast)
            }
            ClassElementData::MethodDefinition(method) => {
                method.r#static(ast)
                    && (method.computed(ast)
                        || match ast.property_key_data(method.key(ast)) {
                            PropertyKeyData::IdentifierName(name) => {
                                ast.get_utf8(name.name(ast)) == "name"
                            }
                            PropertyKeyData::Expr(expr) => expr.as_string_literal(ast).is_some_and(
                                |lit| ast.get_wtf8(lit.value(ast)).as_bytes() == b"name",
                            ),
                        })
            }
            ClassElementData::TsMethodDefinition(_) | ClassElementData::TsIndexSignature(_) => {
                false
            }
        }
    })
}

struct SafetyCheck<'s, 'a> {
    ast: &'s Ast<'a>,
    semantic: &'s ScopeModel<'a>,
    candidates: &'s FxHashSet<SymbolId>,
    check_eval: bool,
    eval_scopes: FxHashSet<ScopeId>,
    observable_class_names: FxHashSet<SymbolId>,
}

impl SafetyCheck<'_, '_> {
    fn check_inferred_class(&mut self, binding: NodeId, expression: Expr) {
        let Some(symbol) = self.semantic.symbol_of(binding).or_else(|| {
            let reference = self.semantic.reference_of(binding)?;
            self.semantic.reference(reference).symbol
        }) else {
            return;
        };
        if !self.candidates.contains(&symbol) {
            return;
        }
        let target = unwrap_transparent_expr(expression, self.ast);
        if let Some(class) = target.as_class(self.ast)
            && class.id(self.ast).is_none()
            && has_observable_class_name(self.ast, class)
        {
            self.observable_class_names.insert(symbol);
        }
    }
}

impl<'a> Visit<'a> for SafetyCheck<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    fn visit_call_expression(&mut self, node: CallExpression) {
        if self.check_eval
            && !node.optional(self.ast)
            && let ExprData::IdentifierReference(callee) = self
                .ast
                .expr_data(unwrap_transparent_expr(node.callee(self.ast), self.ast))
            && self.ast.get_utf8(callee.name(self.ast)) == "eval"
            && let Some(id) = self.semantic.reference_of(callee.node_id())
        {
            self.eval_scopes.insert(self.semantic.reference(id).scope);
        }
        node.visit_children_with(self);
    }

    fn visit_class(&mut self, node: Class) {
        if let Some(id) = node.id(self.ast)
            && let Some(symbol) = self.semantic.symbol_of(id.node_id())
            && self.candidates.contains(&symbol)
            && has_observable_class_name(self.ast, node)
        {
            self.observable_class_names.insert(symbol);
        }
        node.visit_children_with(self);
    }

    fn visit_variable_declarator(&mut self, node: VariableDeclarator) {
        if let Some(init) = node.init(self.ast) {
            self.check_inferred_class(node.id(self.ast).node_id(), init);
        }
        node.visit_children_with(self);
    }

    fn visit_assignment_pattern(&mut self, node: AssignmentPattern) {
        self.check_inferred_class(node.left(self.ast).node_id(), node.right(self.ast));
        node.visit_children_with(self);
    }

    fn visit_assignment_expression(&mut self, node: AssignmentExpression) {
        if matches!(
            node.operator(self.ast),
            AssignmentOperator::Assign
                | AssignmentOperator::LogicalAndAssign
                | AssignmentOperator::LogicalOrAssign
                | AssignmentOperator::NullishAssign
        ) {
            self.check_inferred_class(node.left(self.ast).node_id(), node.right(self.ast));
        }
        node.visit_children_with(self);
    }
}
