//! Rename lexical bindings while keeping syntax and semantic indexes synchronized.

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    Ast, AstUtf8, BindingIdentifier, BindingProperty, CallExpression, DUMMY_SP, Diagnostic,
    ExprData, ImportOrExportKind, ModuleExportName, NodeId, NodeKindData, ObjectProperty, ScopeId,
    Stmt, SymbolId, Visit, VisitMut, VisitMutWith, VisitWith,
};
use swc_next_ecma_semantic::{
    DeclarationId, ReferenceSpace, ScopeModel, SymbolFlags, scope::ScopeKind,
};

mod names;

use super::TransformCtx;

impl TransformCtx<'_, '_> {
    /// Rename bindings while preserving their semantic identities and observable names.
    ///
    /// Callers must provide fresh, collision-free names, including across this batch.
    /// Unsafe bindings remain unchanged and produce diagnostics. Returns their symbols.
    pub fn rename_bindings(
        &mut self,
        bindings: impl IntoIterator<Item = (SymbolId, AstUtf8)>,
    ) -> Vec<SymbolId> {
        let bindings: Vec<_> = bindings
            .into_iter()
            .filter(|&(symbol, name)| {
                self.ast
                    .get_wtf8(self.semantic().symbol(symbol).name)
                    .as_bytes()
                    != self.ast.get_utf8(name).as_bytes()
            })
            .collect();
        if bindings.is_empty() {
            return Vec::new();
        }
        let program = self.ast.root_program();
        // Include expressions queued by earlier transforms in safety checks and rewrites.
        self.inject_statements_to(program.node_id());
        let mut eval = DirectEval {
            ast: self.ast,
            semantic: self.semantic(),
            scopes: FxHashSet::default(),
        };
        program.visit_with(&mut eval);
        let eval_scopes = eval.scopes;
        let mut rejected = Vec::new();
        let mut renamed = FxHashMap::default();
        let mut updates = Vec::new();
        let mut seen = FxHashSet::default();
        for (symbol, name) in bindings {
            if !seen.insert(symbol) {
                continue;
            }
            let linked: Vec<_> = self.semantic().linked_symbols(symbol).collect();
            seen.extend(linked.iter().copied());
            let original = self.semantic().symbol(symbol).name;
            let original = self
                .ast
                .add_utf8(self.ast.get_wtf8(original).as_str().expect("binding name"));
            if let Some(reason) =
                self.observable_binding(self.ast.get_utf8(original), &linked, &eval_scopes)
            {
                let declaration = self
                    .semantic()
                    .declaration(self.semantic().symbol(symbol).declaration);
                let mut diagnostic = Diagnostic::error(
                    "Cannot safely rename binding",
                    self.ast.span(declaration.node),
                );
                diagnostic.help = Some(
                    format!(
                        "Renaming `{}` could change {reason}",
                        self.ast.get_utf8(original)
                    )
                    .into(),
                );
                self.state.diagnostics.push(diagnostic);
                rejected.extend(linked);
                continue;
            }
            for symbol in linked {
                renamed.insert(symbol, original);
                updates.push((symbol, name));
            }
        }
        if !updates.is_empty() {
            for (symbol, name) in updates {
                self.rename_symbol(symbol, name);
            }
            self.preserve_binding_exports(&renamed);
            program.visit_mut_with(&mut ExpandedShorthands { ast: self.ast });
            self.preserve_renamed_function_names(&renamed);
        }
        rejected
    }

    fn observable_binding(
        &self,
        name: &str,
        linked: &[SymbolId],
        eval_scopes: &FxHashSet<ScopeId>,
    ) -> Option<&'static str> {
        let semantic = self.semantic();
        if linked.iter().any(|&symbol| {
            semantic.scope(semantic.scope_of_symbol(symbol)).kind == ScopeKind::Global
                && semantic
                    .symbol(symbol)
                    .flags
                    .visible_in(ReferenceSpace::Value)
        }) {
            return Some("lookup from another script or indirect eval");
        }
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
                if !reference.flags.is_read() && !reference.flags.is_write() {
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
        for declaration in self.semantic().declarations(symbol).collect::<Vec<_>>() {
            match self.ast.kind_data(declaration.node) {
                NodeKindData::BindingIdentifier(identifier) => identifier.set_name(self.ast, name),
                _ => panic!("renaming requires a binding identifier"),
            }
        }
        for reference in self.semantic().uses(symbol).to_vec() {
            let reference = self.semantic().reference(reference);
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
        self.state
            .semantic
            .rename_symbol(symbol, name, self.ast.get_utf8(name));
    }

    fn preserve_binding_exports(&mut self, renamed: &FxHashMap<SymbolId, AstUtf8>) {
        if !self.ast.is_module() {
            return;
        }
        let program = self.ast.root_program();
        let scope = self.program_scope();
        let declarations: FxHashMap<_, _> = self
            .semantic()
            .iter_declarations()
            .filter(|entry| {
                entry.declaration.scope == scope
                    && entry.declaration.flags.contains(SymbolFlags::EXPORTED)
            })
            .map(|entry| (entry.declaration.node, (entry.id, entry.declaration.symbol)))
            .collect();
        let mut statements = Vec::new();
        for slot in program.body(self.ast).iter() {
            let statement = self.ast.get_node_in_sub_range(slot);
            if let Some(export) = statement.as_export_named_declaration(self.ast) {
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
            if let Some(export) = statement.as_export_named_declaration(self.ast)
                && let Some(declaration) = export.declaration(self.ast)
            {
                let mut bindings = ExportBindings {
                    ast: self.ast,
                    declarations: &declarations,
                    bindings: Vec::new(),
                };
                declaration.visit_with(&mut bindings);
                let bindings = bindings.bindings;
                if bindings
                    .iter()
                    .any(|&(_, symbol, _)| renamed.contains_key(&symbol))
                {
                    let mut specifiers = Vec::new();
                    let mut seen = FxHashSet::default();
                    for (id, symbol, identifier) in bindings {
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
                        let local = self.add_node(|ast| ast.identifier_name(DUMMY_SP, local_name));
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
                    export.set_declaration(self.ast, None);
                    let specifiers = self.ast.add_sub_range(specifiers);
                    export.set_specifiers(self.ast, specifiers);
                    statements.push(Stmt::Declaration(declaration));
                }
            }
            statements.push(statement);
        }
        let body = self.ast.add_sub_range(statements);
        program.set_body(self.ast, body);
    }
}

struct ExportBindings<'s, 'a> {
    ast: &'s Ast<'a>,
    declarations: &'s FxHashMap<NodeId, (DeclarationId, SymbolId)>,
    bindings: Vec<(DeclarationId, SymbolId, BindingIdentifier)>,
}

impl<'a> Visit<'a> for ExportBindings<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }
    fn visit_binding_identifier(&mut self, node: BindingIdentifier) {
        if let Some(&(id, symbol)) = self.declarations.get(&node.node_id()) {
            self.bindings.push((id, symbol, node));
        }
    }
}

struct ExpandedShorthands<'s, 'a> {
    ast: &'s mut Ast<'a>,
}

impl<'a> VisitMut<'a> for ExpandedShorthands<'_, 'a> {
    fn ast(&mut self) -> &mut Ast<'a> {
        self.ast
    }

    fn visit_mut_object_property(&mut self, node: ObjectProperty) -> Option<ObjectProperty> {
        node.visit_mut_children_with(self);
        if node.shorthand(self.ast)
            && let Some(key) = node.key(self.ast).as_identifier_name(self.ast)
            && let Some(value) = node.value(self.ast).as_identifier_reference(self.ast)
            && self.ast.get_utf8(key.name(self.ast)) != self.ast.get_utf8(value.name(self.ast))
        {
            node.set_shorthand(self.ast, false);
        }
        None
    }

    fn visit_mut_binding_property(&mut self, node: BindingProperty) -> Option<BindingProperty> {
        node.visit_mut_children_with(self);
        if !node.shorthand(self.ast) {
            return None;
        }
        let value = node.value(self.ast);
        let target = value
            .as_assignment_pattern(self.ast)
            .map_or(value, |pattern| pattern.left(self.ast));
        let name = target
            .as_binding_identifier(self.ast)
            .map(|identifier| identifier.name(self.ast))
            .or_else(|| {
                target
                    .as_simple_assignment_target(self.ast)?
                    .as_identifier_reference(self.ast)
                    .map(|identifier| identifier.name(self.ast))
            });
        if let Some(key) = node.key(self.ast).as_identifier_name(self.ast)
            && let Some(name) = name
            && self.ast.get_utf8(key.name(self.ast)) != self.ast.get_utf8(name)
        {
            node.set_shorthand(self.ast, false);
        }
        None
    }
}

struct DirectEval<'s, 'a> {
    ast: &'s Ast<'a>,
    semantic: &'s ScopeModel<'a>,
    scopes: FxHashSet<ScopeId>,
}

impl<'a> Visit<'a> for DirectEval<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    fn visit_call_expression(&mut self, node: CallExpression) {
        if !node.optional(self.ast) {
            let mut callee = node.callee(self.ast);
            loop {
                callee = match self.ast.expr_data(callee) {
                    ExprData::ParenthesizedExpression(node) => node.expression(self.ast),
                    ExprData::TsAsExpression(node) => node.expression(self.ast),
                    ExprData::TsSatisfiesExpression(node) => node.expression(self.ast),
                    ExprData::TsNonNullExpression(node) => node.expression(self.ast),
                    ExprData::TsTypeAssertion(node) => node.expression(self.ast),
                    ExprData::TsInstantiationExpression(node) => node.expression(self.ast),
                    ExprData::IdentifierReference(node)
                        if self.ast.get_utf8(node.name(self.ast)) == "eval" =>
                    {
                        if let Some(id) = self.semantic.reference_of(node.node_id()) {
                            self.scopes.insert(self.semantic.reference(id).scope);
                        }
                        break;
                    }
                    _ => break,
                };
            }
        }
        node.visit_children_with(self);
    }
}
