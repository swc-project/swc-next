//! Rename lexical bindings while keeping syntax and semantic indexes synchronized.

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    Ast, AstUtf8, BindingIdentifier, BindingPattern, BindingPatternData, CallExpression, DUMMY_SP,
    Decl, DeclData, Diagnostic, Expr, ExprData, ImportOrExportKind, ModuleExportName, NodeKindData,
    ScopeId, Stmt, SymbolId, TsModuleNameData, TsQualifiedNameLeftData, Visit, VisitWith,
};
use swc_next_ecma_semantic::{ReferenceSpace, ScopeModel, SymbolFlags, scope::ScopeKind};

mod names;

use super::TransformCtx;

impl TransformCtx<'_, '_> {
    /// Rename bindings while preserving their semantic identities and observable names.
    ///
    /// Callers must provide distinct symbols with fresh, collision-free names.
    /// Unsafe bindings remain unchanged and produce diagnostics. Returns their symbols.
    pub(crate) fn rename_bindings(&mut self, bindings: Vec<(SymbolId, AstUtf8)>) -> Vec<SymbolId> {
        if bindings.is_empty() {
            return Vec::new();
        }
        let program = self.ast.root_program();
        // Include expressions queued by earlier transforms in safety checks and rewrites.
        self.inject_statements_to(program.node_id());
        let mut eval_scopes = None;
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
            let original_text = self
                .ast
                .get_wtf8(self.semantic().symbol(symbol).name)
                .as_str()
                .expect("binding name");
            let reason = if self.is_global_binding(&linked) {
                Some("lookup from another script or indirect eval")
            } else {
                let eval_scopes = eval_scopes.get_or_insert_with(|| {
                    if self
                        .state
                        .used_names
                        .as_ref()
                        .is_some_and(|names| !names.contains(b"eval"))
                    {
                        return FxHashSet::default();
                    }
                    let mut eval = DirectEval {
                        ast: self.ast,
                        semantic: self.semantic(),
                        scopes: FxHashSet::default(),
                    };
                    program.visit_with(&mut eval);
                    eval.scopes
                });
                self.observable_local_binding(original_text, &linked, eval_scopes)
            };
            let declaration = self
                .semantic()
                .declaration(self.semantic().symbol(symbol).declaration);
            if let Some(reason) = reason {
                let mut diagnostic = Diagnostic::error(
                    "Cannot safely rename binding",
                    self.ast.span(declaration.node),
                );
                diagnostic.help =
                    Some(format!("Renaming `{original_text}` could change {reason}").into());
                self.state.diagnostics.push(diagnostic);
                rejected.extend(linked);
                continue;
            }
            let NodeKindData::BindingIdentifier(identifier) = self.ast.kind_data(declaration.node)
            else {
                panic!("renaming requires a binding identifier");
            };
            let original = identifier.name(self.ast);
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
            self.preserve_renamed_syntax(&renamed);
        }
        rejected
    }

    fn is_global_binding(&self, linked: &[SymbolId]) -> bool {
        let semantic = self.semantic();
        linked.iter().any(|&symbol| {
            semantic.scope(semantic.scope_of_symbol(symbol)).kind == ScopeKind::Global
                && semantic
                    .symbol(symbol)
                    .flags
                    .visible_in(ReferenceSpace::Value)
        })
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
        let old_text = self
            .ast
            .get_wtf8(self.semantic().symbol(symbol).name)
            .as_bytes();
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
            .rename_symbol(symbol, old_text, name, self.ast.get_utf8(name));
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
                    collect_decl_bindings(self.ast, declaration, &mut bindings);
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
                        statements.push(Stmt::Declaration(declaration));
                        if !specifiers.is_empty() {
                            export.set_declaration(self.ast, None);
                            let specifiers = self.ast.add_sub_range(specifiers);
                            export.set_specifiers(self.ast, specifiers);
                            statements.push(statement);
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

fn collect_decl_bindings(ast: &Ast<'_>, decl: Decl, out: &mut Vec<BindingIdentifier>) {
    match ast.decl_data(decl) {
        DeclData::VariableDeclaration(node) => {
            for slot in node.declarators(ast).iter() {
                let declarator = ast.get_node_in_sub_range(slot);
                collect_pattern_bindings(ast, declarator.id(ast), out);
            }
        }
        DeclData::Function(node) => out.extend(node.id(ast)),
        DeclData::Class(node) => out.extend(node.id(ast)),
        DeclData::TsEnumDeclaration(node) => out.push(node.id(ast)),
        DeclData::TsModuleDeclaration(node) => match ast.ts_module_name_data(node.id(ast)) {
            TsModuleNameData::BindingIdentifier(identifier) => out.push(identifier),
            TsModuleNameData::TsQualifiedName(mut qualified) => loop {
                match ast.ts_qualified_name_left_data(qualified.left(ast)) {
                    TsQualifiedNameLeftData::BindingIdentifier(identifier) => {
                        out.push(identifier);
                        break;
                    }
                    TsQualifiedNameLeftData::TsQualifiedName(left) => qualified = left,
                    TsQualifiedNameLeftData::IdentifierReference(_)
                    | TsQualifiedNameLeftData::IdentifierName(_)
                    | TsQualifiedNameLeftData::ThisExpression(_) => break,
                }
            },
            TsModuleNameData::StringLiteral(_) => {}
        },
        DeclData::TsImportEqualsDeclaration(node) => out.push(node.id(ast)),
        DeclData::TsFunction(_)
        | DeclData::TsTypeAliasDeclaration(_)
        | DeclData::TsInterfaceDeclaration(_)
        | DeclData::TsGlobalDeclaration(_) => {}
    }
}

fn collect_pattern_bindings(
    ast: &Ast<'_>,
    pattern: BindingPattern,
    out: &mut Vec<BindingIdentifier>,
) {
    match ast.binding_pattern_data(pattern) {
        BindingPatternData::BindingIdentifier(node) => out.push(node),
        BindingPatternData::AssignmentPattern(node) => {
            collect_pattern_bindings(ast, node.left(ast), out);
        }
        BindingPatternData::BindingRestElement(node) => {
            collect_pattern_bindings(ast, node.argument(ast), out);
        }
        BindingPatternData::ArrayPattern(node) => {
            for slot in node.elements(ast).iter() {
                if let Some(element) = ast.get_node_in_sub_range(slot) {
                    collect_pattern_bindings(ast, element, out);
                }
            }
            if let Some(rest) = node.rest(ast) {
                collect_pattern_bindings(ast, rest.argument(ast), out);
            }
        }
        BindingPatternData::ObjectPattern(node) => {
            for slot in node.properties(ast).iter() {
                let property = ast.get_node_in_sub_range(slot);
                collect_pattern_bindings(ast, property.value(ast), out);
            }
            if let Some(rest) = node.rest(ast) {
                collect_pattern_bindings(ast, rest.argument(ast), out);
            }
        }
        BindingPatternData::SimpleAssignmentTarget(_) => {}
    }
}

pub(super) fn unwrap_transparent_expr(mut expr: Expr, ast: &Ast<'_>) -> Expr {
    loop {
        expr = match ast.expr_data(expr) {
            ExprData::ParenthesizedExpression(node) => node.expression(ast),
            ExprData::TsAsExpression(node) => node.expression(ast),
            ExprData::TsSatisfiesExpression(node) => node.expression(ast),
            ExprData::TsNonNullExpression(node) => node.expression(ast),
            ExprData::TsTypeAssertion(node) => node.expression(ast),
            ExprData::TsInstantiationExpression(node) => node.expression(ast),
            _ => return expr,
        };
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
        if !node.optional(self.ast)
            && let ExprData::IdentifierReference(node) = self
                .ast
                .expr_data(unwrap_transparent_expr(node.callee(self.ast), self.ast))
            && self.ast.get_utf8(node.name(self.ast)) == "eval"
            && let Some(id) = self.semantic.reference_of(node.node_id())
        {
            self.scopes.insert(self.semantic.reference(id).scope);
        }
        node.visit_children_with(self);
    }
}
