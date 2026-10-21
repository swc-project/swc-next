//! Declaration-surface type and value reachability analysis.

use std::{
    collections::VecDeque,
    mem::{replace, take},
};

use bitflags::bitflags;
use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    Accessibility, ArgumentData, ArrowFunctionBodyData, ArrowFunctionExpression, AssignmentPattern,
    Ast, AstNode, BindingPatternData, Class, Decl, DeclData, Diagnostic, ExportDefaultDeclaration,
    ExportDefaultDeclarationKindData, ExportNamedDeclaration, ExportSpecifier, Expr, ExprData,
    FormalParameter, FormalParameterItemData, FormalParameterRest, Function, FunctionBody, GetSpan,
    IdentifierReference, MethodDefinition, MethodDefinitionKind, NodeId, ObjectProperty,
    ObjectPropertyKindData, Program, PropertyDefinition, PropertyKey, PropertyKeyData,
    PropertyKind, Severity, Span, StaticBlock, Stmt, StmtData, SymbolId, TsExportAssignment,
    TsFunction, TsGlobalDeclaration, TsMethodDefinition, TsModuleBlock, TsModuleDeclaration,
    TsParameterProperty, VariableDeclarator, Visit, VisitWith,
};
use swc_next_ecma_semantic::{ReferenceSpace, Semantic};

use crate::surface::{
    AccessorKey, AccessorType, DeclContext, DeclarationSurface, ExportExpression, InferredType,
    class_accessors, export_expression, inferred_type, namespace_binding, object_accessors,
    property_identity, return_expression, supported_computed_key, supported_object_key,
};

bitflags! {
    /// The declaration namespaces in which a symbol is required by declaration emit.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct SymbolUsage: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct UsedSymbols(Vec<u8>);

impl UsedSymbols {
    pub(crate) fn contains(&self, symbol: SymbolId) -> bool {
        self.usage(symbol) != SymbolUsage::empty()
    }

    pub(crate) fn contains_type(&self, symbol: SymbolId) -> bool {
        self.usage(symbol).contains(SymbolUsage::TYPE)
    }

    pub(crate) fn contains_value(&self, symbol: SymbolId) -> bool {
        self.usage(symbol).contains(SymbolUsage::VALUE)
    }

    fn usage(&self, symbol: SymbolId) -> SymbolUsage {
        let states = self.0.get(symbol.index()).copied().unwrap_or_default();
        let mut usage = SymbolUsage::empty();
        for candidate in [SymbolUsage::VALUE, SymbolUsage::TYPE, SymbolUsage::all()] {
            if states & state_for(candidate) != 0 {
                usage.insert(candidate);
            }
        }
        usage
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SymbolKey {
    symbol: SymbolId,
    usage: SymbolUsage,
}

impl SymbolKey {
    fn new(symbol: SymbolId, usage: SymbolUsage) -> Self {
        Self { symbol, usage }
    }
}

fn state_for(usage: SymbolUsage) -> u8 {
    1 << usage.bits()
}

/// Schedules each exact symbol/usage pair once, including the combined usage.
#[derive(Default)]
struct SymbolQueue {
    pending: VecDeque<SymbolKey>,
    seen: Vec<u8>,
}

impl SymbolQueue {
    fn push(&mut self, key: SymbolKey) {
        let index = key.symbol.index();
        if index >= self.seen.len() {
            self.seen.resize(index + 1, 0);
        }
        // TYPE, VALUE, and their combination are distinct graph states.
        let state = state_for(key.usage);
        if self.seen[index] & state == 0 {
            self.seen[index] |= state;
            self.pending.push_back(key);
        }
    }

    fn extend(&mut self, keys: impl IntoIterator<Item = SymbolKey>) {
        for key in keys {
            self.push(key);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Source {
    Root,
    Symbol(SymbolKey),
}

/// Builds the declaration dependency graph used to prune private declarations and imports.
///
/// References use semantic `SymbolId` identities to preserve shadowing and merged
/// declarations without rebuilding name resolution.
pub(crate) struct TypeUsageAnalyzer<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    semantic: &'visit Semantic<'ast>,
    edges: FxHashMap<SymbolKey, Vec<SymbolKey>>,
    pending_edges: Vec<SymbolKey>,
    roots: Vec<SymbolKey>,
    scope_source: Source,
    source: Option<Source>,
    diagnostics: Vec<(Source, Diagnostic<'static>)>,
    returns: Vec<(Span, Span)>,
    erased_function_names: Vec<SymbolId>,
    surface: DeclarationSurface,
    context: DeclContext,
    accessor_types: FxHashMap<NodeId, AccessorType>,
}

impl<'visit, 'ast> TypeUsageAnalyzer<'visit, 'ast> {
    pub(crate) fn analyze(
        ast: &'visit Ast<'ast>,
        semantic: &'visit Semantic<'ast>,
    ) -> (UsedSymbols, Vec<Diagnostic<'static>>, DeclarationSurface) {
        let mut analyzer = Self {
            ast,
            semantic,
            edges: FxHashMap::default(),
            pending_edges: Vec::new(),
            roots: Vec::new(),
            scope_source: Source::Root,
            source: None,
            diagnostics: Vec::new(),
            returns: Vec::new(),
            erased_function_names: Vec::new(),
            surface: DeclarationSurface::default(),
            context: DeclContext::for_program(ast.source_type()),
            accessor_types: FxHashMap::default(),
        };
        ast.root_program().visit_with(&mut analyzer);
        analyzer.finish()
    }

    fn finish(self) -> (UsedSymbols, Vec<Diagnostic<'static>>, DeclarationSurface) {
        let mut queue = SymbolQueue::default();
        queue.extend(self.roots);

        while let Some(key) = queue.pending.pop_front() {
            if let Some(targets) = self.edges.get(&key) {
                queue.extend(targets.iter().copied());
            }
            if key.usage == SymbolUsage::all() {
                for usage in [SymbolUsage::VALUE, SymbolUsage::TYPE] {
                    if let Some(targets) = self.edges.get(&SymbolKey::new(key.symbol, usage)) {
                        queue.extend(targets.iter().copied());
                    }
                }
            } else if let Some(targets) = self
                .edges
                .get(&SymbolKey::new(key.symbol, SymbolUsage::all()))
            {
                queue.extend(targets.iter().copied());
            }
        }

        let used = UsedSymbols(queue.seen);
        let diagnostics = self
            .diagnostics
            .into_iter()
            .filter_map(|(source, diagnostic)| {
                let reachable = match source {
                    Source::Root => true,
                    Source::Symbol(key) => used.usage(key.symbol).intersects(key.usage),
                };
                reachable.then_some(diagnostic)
            })
            .collect();
        (used, diagnostics, self.surface)
    }

    fn add_reference(&mut self, node: NodeId) {
        let semantic = self.semantic;
        let Some(id) = semantic.reference_of(node) else {
            return;
        };
        let reference = semantic.reference(id);
        let private_name = reference.resolution().symbol().is_some_and(|symbol| {
            self.erased_function_names.contains(&symbol)
                || self.returns.iter().any(|(body, retained)| {
                    semantic.declarations(symbol).all(|declaration| {
                        let span = self.ast.span(declaration.node);
                        body.start <= span.start
                            && span.end <= body.end
                            && !(retained.start <= span.start && span.end <= retained.end)
                    })
                })
        });
        if private_name && let Some(source) = self.source {
            let name = self.ast.get_utf8(reference.name);
            self.diagnostics.push((source, Diagnostic {
                severity: Severity::Error,
                message: format!("Type containing private name '{name}' can't be used with --isolatedDeclarations.").into(),
                span: self.ast.span(node),
                help: None,
                labels: Vec::new(),
            }));
        }

        let usage = match reference.flags.space {
            ReferenceSpace::Value | ReferenceSpace::Typeof => SymbolUsage::VALUE,
            ReferenceSpace::Type | ReferenceSpace::Namespace => SymbolUsage::TYPE,
            ReferenceSpace::Any => SymbolUsage::TYPE,
        };
        // A candidate is a dependency, not proof that this spelling will still
        // resolve to the same declaration after private declarations are pruned.
        for symbol in semantic.reference_candidates(id) {
            self.add_edge(SymbolKey::new(symbol, usage));
        }
        if reference.flags.space == ReferenceSpace::Any {
            for target in semantic.reference_value_targets(self.ast, id) {
                if let Some(symbol) = target.candidate() {
                    self.add_edge(SymbolKey::new(symbol, SymbolUsage::VALUE));
                }
            }
        }
        if reference.flags.is_uncertain()
            && let Some(source) = self.source
        {
            self.diagnostics.push((source, Diagnostic::error("The declaration depends on a reference whose target cannot be determined without type checking.", self.ast.span(node))));
        }
    }

    fn declaration_key(&self, node: impl AstNode, usage: SymbolUsage) -> Option<SymbolKey> {
        let symbol = self.semantic.symbol_of(node.node_id())?;
        Some(SymbolKey::new(symbol, usage))
    }

    fn add_edge(&mut self, target: SymbolKey) {
        match self.source {
            Some(Source::Root) => {
                self.roots.push(target);
            }
            Some(Source::Symbol(_)) => {
                self.pending_edges.push(target);
            }
            None => {}
        }
    }

    fn with_source(&mut self, source: Source, visit: impl FnOnce(&mut Self)) {
        let previous_source = self.source.replace(source);
        let previous_edges = take(&mut self.pending_edges);
        visit(self);
        if let Source::Symbol(key) = source
            && !self.pending_edges.is_empty()
        {
            self.edges
                .entry(key)
                .or_default()
                .append(&mut self.pending_edges);
        }
        self.pending_edges = previous_edges;
        self.source = previous_source;
    }

    fn with_scope_source(&mut self, visit: impl FnOnce(&mut Self)) {
        self.with_source(self.scope_source, visit);
    }

    fn with_declaration(
        &mut self,
        binding: impl AstNode,
        usage: SymbolUsage,
        visit: impl FnOnce(&mut Self),
    ) {
        let Some(key) = self.declaration_key(binding, usage) else {
            visit(self);
            return;
        };
        self.add_edge(key);
        self.with_source(Source::Symbol(key), visit);
    }

    fn visit_declaration_statement(&mut self, statement: Stmt) {
        if self
            .surface
            .hidden_implementations
            .contains(&statement.node_id())
        {
            return;
        }
        if self.context.retain_all
            && let StmtData::Declaration(declaration) = self.ast.stmt_data(statement)
        {
            self.with_scope_source(|analyzer| declaration.visit_with(analyzer));
            return;
        }
        match self.ast.stmt_data(statement) {
            StmtData::Declaration(declaration) => declaration.visit_with(self),
            StmtData::ExportNamedDeclaration(declaration) => declaration.visit_with(self),
            StmtData::ExportDefaultDeclaration(declaration) => declaration.visit_with(self),
            StmtData::TsExportAssignment(declaration) => declaration.visit_with(self),
            StmtData::ImportDeclaration(_)
            | StmtData::ExportAllDeclaration(_)
            | StmtData::TsNamespaceExportDeclaration(_) => {}
            _ => {}
        }
    }

    fn visit_function_signature(&mut self, function: Function) {
        function.type_parameters(self.ast).visit_with(self);
        function.this_param(self.ast).visit_with(self);
        function.params(self.ast).visit_with(self);
        if let Some(annotation) = function.return_type(self.ast) {
            annotation.visit_with(self);
        } else if !function.r#async(self.ast)
            && !function.generator(self.ast)
            && let Some(expression) = return_expression(function.body(self.ast), self.ast)
        {
            self.visit_return_expression(function.body(self.ast), expression);
        }
    }

    // Check body-local bindings while walking the original declaration surface.
    // Bindings inside the returned expression (e.g. a generic function's type
    // parameters) survive; nested inferred returns add their own erased body.
    fn visit_return_expression(&mut self, body: FunctionBody, expression: Expr) {
        self.returns
            .push((body.span(self.ast), expression.span(self.ast)));
        self.visit_inferred_expression(expression, false);
        self.returns.pop();
    }

    fn visit_export_expression(&mut self, expression: Expr) {
        match export_expression(expression, self.ast) {
            ExportExpression::Reference(reference) => self.add_reference(reference.node_id()),
            ExportExpression::Class(class) => class.visit_with(self),
            ExportExpression::Inferred(expression) => {
                self.visit_inferred_expression(expression, false)
            }
        }
    }

    fn visit_inferred_expression(&mut self, expression: Expr, is_const: bool) {
        match inferred_type(expression, is_const, self.ast, self.semantic, &self.surface) {
            InferredType::Annotation(annotation) | InferredType::OptionalParameter(annotation) => {
                annotation.visit_with(self)
            }
            InferredType::Function(function) => self.visit_inferred_function_signature(function),
            InferredType::Arrow(function) => self.visit_arrow_function_expression(function),
            InferredType::Object(object, is_const) => {
                let pairs = object_accessors(object, self.ast, self.semantic);
                for property in object.properties(self.ast).iter() {
                    let ObjectPropertyKindData::ObjectProperty(property) = self
                        .ast
                        .object_property_kind_data(self.ast.get_node_in_sub_range(property))
                    else {
                        continue;
                    };
                    if property.shorthand(self.ast)
                        || (property.computed(self.ast)
                            && !supported_object_key(property.key(self.ast), self.ast))
                    {
                        continue;
                    }
                    property.key(self.ast).visit_with(self);
                    if matches!(
                        property.kind(self.ast),
                        PropertyKind::Get | PropertyKind::Set
                    ) && let Some(name) =
                        property_identity(property.key(self.ast), self.ast, self.semantic)
                        && let Some(pair) = pairs.get(&AccessorKey {
                            name,
                            r#static: false,
                        })
                    {
                        let selected = if property.kind(self.ast) == PropertyKind::Get {
                            pair.read_type(self.ast)
                        } else {
                            pair.write_type(self.ast)
                        };
                        if let Some(selected) = selected {
                            self.visit_accessor_type(selected);
                        }
                    } else {
                        self.visit_inferred_expression(property.value(self.ast), is_const);
                    }
                }
            }
            InferredType::Array(array) => {
                for element in array.elements(self.ast).iter() {
                    if let Some(element) = self.ast.get_node_in_sub_range(element)
                        && let ArgumentData::Expr(expression) = self.ast.argument_data(element)
                    {
                        self.visit_inferred_expression(expression, true);
                    }
                }
            }
            InferredType::Literal(_)
            | InferredType::Template(_)
            | InferredType::String
            | InferredType::Number
            | InferredType::BigInt
            | InferredType::Boolean
            | InferredType::Null
            | InferredType::Undefined
            | InferredType::InvalidArray
            | InferredType::InvalidClass
            | InferredType::Unsupported => {}
        }
    }

    fn visit_inferred_function_signature(&mut self, function: Function) {
        let Some(binding) = function.id(self.ast) else {
            self.visit_function_signature(function);
            return;
        };
        let Some(symbol) = self.semantic.symbol_of(binding.node_id()) else {
            self.visit_function_signature(function);
            return;
        };

        self.erased_function_names.push(symbol);
        self.visit_function_signature(function);
        self.erased_function_names.pop();
    }

    fn visit_accessor_type(&mut self, selected: AccessorType) {
        match selected {
            AccessorType::Annotation(annotation) => annotation.visit_with(self),
            AccessorType::Inferred(function) => {
                if let Some(expression) = return_expression(function.body(self.ast), self.ast) {
                    self.visit_return_expression(function.body(self.ast), expression);
                }
            }
        }
    }

    fn visit_ts_function_signature(&mut self, function: TsFunction) {
        function.type_parameters(self.ast).visit_with(self);
        function.this_param(self.ast).visit_with(self);
        function.params(self.ast).visit_with(self);
        function.return_type(self.ast).visit_with(self);
    }

    fn visit_variable_declarator(&mut self, declarator: VariableDeclarator) {
        let pattern = declarator.id(self.ast);
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingIdentifier(binding) => {
                self.with_declaration(binding, SymbolUsage::VALUE, |analyzer| {
                    if let Some(annotation) = declarator.type_annotation(analyzer.ast) {
                        annotation.visit_with(analyzer);
                    } else if let Some(expression) = declarator.init(analyzer.ast) {
                        analyzer.visit_inferred_expression(expression, false);
                    }
                });
            }
            _ => pattern.visit_with(self),
        }
    }

    fn computed_key_is_declaration_safe(&self, key: PropertyKey) -> bool {
        supported_computed_key(key, self.ast, self.semantic)
    }

    fn property_key_is_private(key: PropertyKey, ast: &Ast<'_>) -> bool {
        matches!(
            ast.property_key_data(key),
            PropertyKeyData::Expr(expression)
                if matches!(ast.expr_data(expression), ExprData::PrivateIdentifier(_))
        )
    }
}

impl<'ast> Visit<'ast> for TypeUsageAnalyzer<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_program(&mut self, program: Program) {
        self.surface
            .record_statement_overloads(program.body(self.ast), self.ast, self.semantic);
        for statement in program.body(self.ast).iter() {
            let statement = self.ast.get_node_in_sub_range(statement);
            self.visit_declaration_statement(statement);
        }
    }

    fn visit_decl(&mut self, declaration: Decl) {
        match self.ast.decl_data(declaration) {
            DeclData::VariableDeclaration(declaration) => {
                for declarator in declaration.declarators(self.ast).iter() {
                    let declarator = self.ast.get_node_in_sub_range(declarator);
                    self.visit_variable_declarator(declarator);
                }
            }
            DeclData::Function(function) => {
                if let Some(binding) = function.id(self.ast) {
                    self.with_declaration(binding, SymbolUsage::VALUE, |analyzer| {
                        analyzer.visit_function_signature(function);
                    });
                } else {
                    self.visit_function_signature(function);
                }
            }
            DeclData::Class(class) => {
                if let Some(binding) = class.id(self.ast) {
                    self.with_declaration(binding, SymbolUsage::all(), |analyzer| {
                        class.visit_with(analyzer);
                    });
                } else {
                    class.visit_with(self);
                }
            }
            DeclData::TsFunction(function) => {
                if let Some(binding) = function.id(self.ast) {
                    self.with_declaration(binding, SymbolUsage::VALUE, |analyzer| {
                        analyzer.visit_ts_function_signature(function);
                    });
                } else {
                    self.visit_ts_function_signature(function);
                }
            }
            DeclData::TsTypeAliasDeclaration(declaration) => {
                let binding = declaration.id(self.ast);
                self.with_declaration(binding, SymbolUsage::TYPE, |analyzer| {
                    declaration
                        .type_parameters(analyzer.ast)
                        .visit_with(analyzer);
                    declaration
                        .type_annotation(analyzer.ast)
                        .visit_with(analyzer);
                });
            }
            DeclData::TsInterfaceDeclaration(declaration) => {
                let binding = declaration.id(self.ast);
                self.with_declaration(binding, SymbolUsage::TYPE, |analyzer| {
                    declaration
                        .type_parameters(analyzer.ast)
                        .visit_with(analyzer);
                    declaration.extends(analyzer.ast).visit_with(analyzer);
                    declaration.body(analyzer.ast).visit_with(analyzer);
                });
            }
            DeclData::TsEnumDeclaration(declaration) => {
                let binding = declaration.id(self.ast);
                self.with_declaration(binding, SymbolUsage::all(), |analyzer| {
                    declaration.body(analyzer.ast).visit_with(analyzer);
                });
            }
            DeclData::TsModuleDeclaration(declaration) => {
                declaration.visit_with(self);
            }
            DeclData::TsGlobalDeclaration(declaration) => declaration.visit_with(self),
            DeclData::TsImportEqualsDeclaration(declaration) => {
                let binding = declaration.id(self.ast);
                self.with_declaration(binding, SymbolUsage::all(), |analyzer| {
                    declaration
                        .module_reference(analyzer.ast)
                        .visit_with(analyzer);
                });
            }
        }
    }

    fn visit_export_named_declaration(&mut self, declaration: ExportNamedDeclaration) {
        if declaration.source(self.ast).is_some() {
            return;
        }
        self.with_scope_source(|analyzer| {
            declaration.declaration(analyzer.ast).visit_with(analyzer);
            declaration.specifiers(analyzer.ast).visit_with(analyzer);
        });
    }

    fn visit_export_default_declaration(&mut self, declaration: ExportDefaultDeclaration) {
        self.with_scope_source(|analyzer| {
            let declaration = declaration.declaration(analyzer.ast);
            if let ExportDefaultDeclarationKindData::Expr(expression) = analyzer
                .ast
                .export_default_declaration_kind_data(declaration)
            {
                analyzer.visit_export_expression(expression);
            } else {
                declaration.visit_with(analyzer);
            }
        });
    }

    fn visit_export_specifier(&mut self, specifier: ExportSpecifier) {
        self.add_reference(specifier.local(self.ast).node_id());
    }

    fn visit_ts_export_assignment(&mut self, declaration: TsExportAssignment) {
        self.with_scope_source(|analyzer| {
            analyzer.visit_export_expression(declaration.expression(analyzer.ast));
        });
    }

    fn visit_ts_module_declaration(&mut self, declaration: TsModuleDeclaration) {
        let context = self.context.namespace(declaration, self.ast);
        let previous_context = replace(&mut self.context, context);
        let id = declaration.id(self.ast);
        if let Some(binding) = namespace_binding(id, self.ast) {
            if let Some(key) = self.declaration_key(binding, SymbolUsage::all()) {
                self.add_edge(key);
                let previous_scope = replace(&mut self.scope_source, Source::Symbol(key));
                let previous = self.source.take();
                declaration.body(self.ast).visit_with(self);
                self.source = previous;
                self.scope_source = previous_scope;
            } else {
                declaration.body(self.ast).visit_with(self);
            }
        } else {
            self.with_scope_source(|analyzer| {
                declaration.body(analyzer.ast).visit_with(analyzer);
            });
        }
        self.context = previous_context;
    }

    fn visit_ts_module_block(&mut self, block: TsModuleBlock) {
        self.surface
            .record_statement_overloads(block.body(self.ast), self.ast, self.semantic);
        for statement in block.body(self.ast).iter() {
            let statement = self.ast.get_node_in_sub_range(statement);
            self.visit_declaration_statement(statement);
        }
    }

    fn visit_ts_global_declaration(&mut self, declaration: TsGlobalDeclaration) {
        let previous = replace(&mut self.context, DeclContext::AMBIENT_MODULE);
        self.with_source(Source::Root, |analyzer| {
            declaration.body(analyzer.ast).visit_with(analyzer);
        });
        self.context = previous;
    }

    fn visit_identifier_reference(&mut self, reference: IdentifierReference) {
        self.add_reference(reference.node_id());
    }

    fn visit_function(&mut self, function: Function) {
        self.visit_function_signature(function);
    }

    fn visit_ts_function(&mut self, function: TsFunction) {
        self.visit_ts_function_signature(function);
    }

    fn visit_arrow_function_expression(&mut self, function: ArrowFunctionExpression) {
        function.type_parameters(self.ast).visit_with(self);
        function.params(self.ast).visit_with(self);
        if let Some(annotation) = function.return_type(self.ast) {
            annotation.visit_with(self);
        } else if !function.r#async(self.ast) {
            let expression = match self.ast.arrow_function_body_data(function.body(self.ast)) {
                ArrowFunctionBodyData::Expr(expression) => Some(expression),
                ArrowFunctionBodyData::FunctionBody(body) => {
                    if let Some(expression) = return_expression(body, self.ast) {
                        self.visit_return_expression(body, expression);
                    }
                    None
                }
            };
            if let Some(expression) = expression {
                self.visit_inferred_expression(expression, false);
            }
        }
    }

    fn visit_class(&mut self, class: Class) {
        self.surface
            .record_class_overloads(class, self.ast, self.semantic);
        let elements = class
            .body(self.ast)
            .body(self.ast)
            .iter()
            .map(|element| self.ast.get_node_in_sub_range(element));
        for pair in class_accessors(elements, self.ast, self.semantic).into_values() {
            if let Some(getter) = pair.getter
                && let Some(selected) = pair.read_type(self.ast)
            {
                self.accessor_types.insert(getter.node_id(), selected);
            }
            if let Some(setter) = pair.setter
                && let Some(selected) = pair.write_type(self.ast)
            {
                self.accessor_types.insert(setter.node_id(), selected);
            }
        }
        class.id(self.ast).visit_with(self);
        class.type_parameters(self.ast).visit_with(self);
        class.super_class(self.ast).visit_with(self);
        class.super_type_arguments(self.ast).visit_with(self);
        class.implements(self.ast).visit_with(self);
        class.body(self.ast).visit_with(self);
    }

    fn visit_formal_parameter(&mut self, parameter: FormalParameter) {
        self.surface
            .record_annotation(parameter, self.ast, self.semantic);
        parameter.pattern(self.ast).visit_with(self);
        if let Some(annotation) = parameter.type_annotation(self.ast) {
            annotation.visit_with(self);
        } else if let Some(expression) = parameter.initializer(self.ast) {
            self.visit_inferred_expression(expression, false);
        }
    }

    fn visit_formal_parameter_rest(&mut self, parameter: FormalParameterRest) {
        parameter.argument(self.ast).visit_with(self);
        parameter.type_annotation(self.ast).visit_with(self);
    }

    fn visit_ts_parameter_property(&mut self, property: TsParameterProperty) {
        property.parameter(self.ast).visit_with(self);
    }

    fn visit_property_definition(&mut self, property: PropertyDefinition) {
        if Self::property_key_is_private(property.key(self.ast), self.ast) {
            return;
        }
        if !property.computed(self.ast)
            || self.computed_key_is_declaration_safe(property.key(self.ast))
        {
            property.key(self.ast).visit_with(self);
        }
        if property.accessibility(self.ast) == Accessibility::Private {
            return;
        }
        if let Some(annotation) = property.type_annotation(self.ast) {
            annotation.visit_with(self);
        } else if let Some(expression) = property.value(self.ast) {
            self.visit_inferred_expression(expression, false);
        }
    }

    fn visit_method_definition(&mut self, method: MethodDefinition) {
        let hidden = self
            .surface
            .hidden_implementations
            .contains(&method.node_id());
        if hidden && method.kind(self.ast) != MethodDefinitionKind::Constructor {
            return;
        }
        if Self::property_key_is_private(method.key(self.ast), self.ast) {
            return;
        }
        if !method.computed(self.ast) || self.computed_key_is_declaration_safe(method.key(self.ast))
        {
            method.key(self.ast).visit_with(self);
        }
        if hidden || method.accessibility(self.ast) == Accessibility::Private {
            if method.kind(self.ast) == MethodDefinitionKind::Constructor {
                let params = method.value(self.ast).params(self.ast);
                for item in params.items(self.ast).iter() {
                    let item = self.ast.get_node_in_sub_range(item);
                    let FormalParameterItemData::TsParameterProperty(property) =
                        self.ast.formal_parameter_item_data(item)
                    else {
                        continue;
                    };
                    if property.accessibility(self.ast) != Accessibility::Private {
                        property.parameter(self.ast).visit_with(self);
                    }
                }
            }
            return;
        }
        let function = method.value(self.ast);
        if let Some(selected) = self.accessor_types.get(&function.node_id()).copied() {
            function.this_param(self.ast).visit_with(self);
            self.visit_accessor_type(selected);
        } else {
            function.visit_with(self);
        }
    }

    fn visit_ts_method_definition(&mut self, method: TsMethodDefinition) {
        if Self::property_key_is_private(method.key(self.ast), self.ast) {
            return;
        }
        if !method.computed(self.ast) || self.computed_key_is_declaration_safe(method.key(self.ast))
        {
            method.key(self.ast).visit_with(self);
        }
        if method.accessibility(self.ast) == Accessibility::Private {
            return;
        }
        let function = method.value(self.ast);
        if let Some(selected) = self.accessor_types.get(&function.node_id()).copied() {
            function.this_param(self.ast).visit_with(self);
            self.visit_accessor_type(selected);
        } else {
            self.visit_ts_function_signature(function);
        }
    }

    fn visit_assignment_pattern(&mut self, pattern: AssignmentPattern) {
        pattern.left(self.ast).visit_with(self);
    }

    fn visit_object_property(&mut self, property: ObjectProperty) {
        if !property.computed(self.ast)
            || self.computed_key_is_declaration_safe(property.key(self.ast))
        {
            property.key(self.ast).visit_with(self);
        }
        property.value(self.ast).visit_with(self);
    }

    fn visit_static_block(&mut self, _block: StaticBlock) {}
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Lang, ScopeId, SourceType};
    use swc_next_ecma_codegen::{Options as PrintOptions, print};
    use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, ReferenceSpace, SyntaxChecks, analyze};

    use super::TypeUsageAnalyzer;
    use crate::transform;

    fn declaration_output(source: &str) -> (String, usize) {
        declaration_output_with_source_type(source, SourceType::Module)
    }

    fn declaration_output_with_source_type(
        source: &str,
        source_type: SourceType,
    ) -> (String, usize) {
        let allocator = Allocator::new();
        let options = Options {
            lang: Lang::Ts,
            source_type,
            comments: CommentMode::Flat,
            ..Options::default()
        };
        let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
        assert!(parsed.diagnostics.is_empty());
        let mut ast = parsed.ast;
        let result = analyze(
            &ast,
            AnalyzeOptions {
                check_syntax: SyntaxChecks::all(),
                ..AnalyzeOptions::default()
            },
        );
        assert!(result.diagnostics.is_empty());
        let diagnostics = transform(&mut ast, &result.semantic);
        let code = print(
            &ast,
            PrintOptions {
                comments: false,
                ..PrintOptions::default()
            },
        )
        .code;
        (code, diagnostics.len())
    }

    fn assert_export_dependencies(source: &str) {
        let allocator = Allocator::new();
        let options = Options {
            lang: Lang::Ts,
            source_type: SourceType::Module,
            ..Options::default()
        };
        let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
        assert!(parsed.diagnostics.is_empty());
        let ast = parsed.ast;
        let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
        let scope = ScopeId::MODULE;
        let local = semantic.lookup(scope, b"X", ReferenceSpace::Type).unwrap();
        let exported = semantic.lookup(scope, b"X", ReferenceSpace::Value).unwrap();
        assert_ne!(local, exported);
        let (used, diagnostics, _) = TypeUsageAnalyzer::analyze(&ast, &semantic);
        assert!(diagnostics.is_empty());
        assert!(used.contains_type(local));
        assert!(!used.contains_value(local));
        assert!(used.contains_value(exported));
    }

    #[test]
    fn export_dependencies() {
        let (code, diagnostics) = declaration_output(
            "import type { T } from 'dual' with { 'resolution-mode': 'require' }; export type Public = T;",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("with {"));
        assert!(code.contains("resolution-mode"));

        let (code, diagnostics) = declaration_output(
            "export type { T } from 'dual' with { 'resolution-mode': 'require' };",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("resolution-mode"));

        let (code, diagnostics) =
            declaration_output("export type * from 'dual' with { 'resolution-mode': 'import' };");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("resolution-mode"));

        assert_export_dependencies("interface X {} export const X = 1; export default X;");

        assert_export_dependencies("export const X = 1; interface X {} export { X as Y };");

        let (code, diagnostics) =
            declaration_output("interface X {} export const X = 1; export default X;");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("interface X"));
        assert!(code.contains("export declare const X = 1;"));
        assert!(code.contains("export default X;"));

        let (code, diagnostics) = declaration_output("export default 1 as typeof _default;");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("const _default_1: typeof _default"));
        assert!(code.contains("export default _default_1"));

        let (code, diagnostics) = declaration_output(
            "declare const _default_1: number; export default 1 as typeof _default;",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("const _default_2: typeof _default"));

        let (code, diagnostics) =
            declaration_output("const hidden = [1, 2]; export default hidden as number[];");
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("hidden"));
        assert!(code.contains("const _default: number[]"));

        let (code, diagnostics) =
            declaration_output("const hidden = [1, 2]; export = hidden as number[];");
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("hidden"));
        assert!(code.contains("export = _default"));
    }

    #[test]
    fn exported_alias_declarations() {
        let (code, diagnostics) =
            declaration_output("namespace N { export interface T {} } export import A = N.T;");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("namespace N"));
        assert!(code.contains("interface T"));
        assert!(code.contains("export import A = N.T;"));

        let (code, diagnostics) = declaration_output(
            "namespace N { export interface T {} } export namespace Use { export import A = N.T; }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("namespace N"));
        assert!(code.contains("namespace Use"));
        assert!(code.contains("export import A = N.T;"));
    }

    #[test]
    fn expando_type_dependencies() {
        let (code, diagnostics) = declaration_output(
            "type Alias = number; function Root(): void {} namespace Root { export function child(): void {} export namespace child { export interface Marker {} } } Root.child.value = 1; export namespace Use { import Alias = Root.child.value; export type T = Alias; }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("type Alias = number;"));
        assert!(!code.contains("import Alias"));
        assert!(!code.contains("namespace Root"));
    }

    #[test]
    fn uncertain_declaration_dependencies() {
        let (code, diagnostics) = declaration_output(
            "import * as Holder from 'pkg'; type Alias = number; export namespace Use { import Alias = Holder.Member; export type T = Alias; }",
        );
        assert_eq!(diagnostics, 1);
        assert!(code.contains("type Alias = number;"));
        assert!(code.contains("import Alias = Holder.Member;"));

        let (code, diagnostics) = declaration_output(
            "import * as Holder from 'pkg'; type Alias = number; namespace Private { import Alias = Holder.Member; export type T = Alias; } export {};",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("namespace Private"));
    }

    #[test]
    fn inferred_declaration_dependencies() {
        let (code, diagnostics) = declaration_output(
            "type Hidden = { x: number }; export class C { value = ({} as Hidden); }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("type Hidden"));
        assert!(code.contains("value: Hidden"));

        let (code, diagnostics) = declaration_output(
            "type Hidden = { x: number }; export function f(x = ({} as Hidden)): void {}",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("type Hidden"));
        assert!(code.contains("x?: Hidden"));

        let (code, diagnostics) = declaration_output(
            "type Hidden = { x: number }; export function f() { return ({} as Hidden); }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("type Hidden"));
        assert!(code.contains("f(): Hidden"));

        let (code, diagnostics) = declaration_output(
            "function runtime() { return [1]; } export const value: number = runtime()[0];",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("runtime"));
        assert!(code.contains("value: number"));
    }

    #[test]
    fn namespace_value_dependencies() {
        let (code, diagnostics) = declaration_output(
            "type Hidden = { x: number }; namespace N { export const x: Hidden = { x: 1 }; } export type T = typeof N;",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("namespace N"));
        assert!(code.contains("type Hidden"));
        assert!(code.contains("x: Hidden"));
        let (code, diagnostics) = declaration_output(
            "import type { T } from './dep'; export declare namespace API { namespace Nested { function use(value: T): void; } }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("import type { T }"));
        assert!(code.contains("use(value: T)"));

        let (code, diagnostics) = declaration_output_with_source_type(
            "namespace API { interface T { value: string } export function use(value: T): void {} }",
            SourceType::Script,
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("interface T"));
        assert!(code.contains("use(value: T)"));
    }

    #[test]
    fn method_overload_dependencies() {
        let (code, diagnostics) = declaration_output(
            "const hidden = (() => ({ value: 1 }))(); export function f(x: string): void; export function f(x: typeof hidden | string): void {}",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("hidden"));

        let (code, diagnostics) = declaration_output(
            "function compute(): number { return 1; } export class C { constructor(x?: number); constructor(x = compute()) {} }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("compute"));
        assert_eq!(code.matches("constructor(").count(), 1);

        let (code, diagnostics) = declaration_output(
            "type T = { value: number }; export class C { constructor(value: T); constructor(public value: T) {} }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("type T"));
        assert!(code.contains("value: T;"));
        assert_eq!(code.matches("constructor(").count(), 1);

        let (code, diagnostics) = declaration_output(
            "export default function f(x: number): void; export default function f(x = compute()): void {} declare function compute(): number;",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("compute"));

        let (code, diagnostics) = declaration_output(
            "export namespace N { const hidden = (() => 1)(); export function f(x: string): void; export function f(x: typeof hidden | string): void {} }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("hidden"));

        let (code, diagnostics) = declaration_output(
            "type Hidden = { value: string }; export declare class C { private m(x: Hidden): Hidden; }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("private m;"));
        assert!(!code.contains("Hidden"));

        let (code, diagnostics) = declaration_output(
            "type Hidden = { value: string }; export class C { private m(x: Hidden): Hidden; private m(x: string): string; private m(x: Hidden | string): Hidden | string { return x; } }",
        );
        assert_eq!(diagnostics, 0);
        assert_eq!(code.matches("private m;").count(), 1);
        assert!(!code.contains("Hidden"));

        let (code, diagnostics) = declaration_output(
            "type Hidden = number[]; export class C { m(x: string): string; m(x: number): number; m(x = [] as Hidden) { return [x]; } }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("Hidden"));
        assert!(code.contains("m(x: string): string"));
        assert!(code.contains("m(x: number): number"));
        assert_eq!(code.matches("m(").count(), 2);

        let (code, diagnostics) = declaration_output(
            "export default class { m(x: string): string; m(x = []) { return [x]; } }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("m(x: string): string"));
        assert_eq!(code.matches("m(").count(), 1);
    }

    #[test]
    fn module_contexts() {
        let (code, diagnostics) = declaration_output(
            "import type { Unused } from 'pkg'; declare global { interface Window { value: string } }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("import"));
        assert!(code.contains("declare global"));
        assert!(code.contains("export {}"));

        let common_js = SourceType::from_path("input.cts");
        let (code, diagnostics) =
            declaration_output_with_source_type("const hidden = 1;", common_js);
        assert_eq!(diagnostics, 0);
        assert_eq!(code, "export {};\n");

        let (code, diagnostics) =
            declaration_output_with_source_type("function hidden() { return [1]; }", common_js);
        assert_eq!(diagnostics, 0);
        assert_eq!(code, "export {};\n");

        let (code, diagnostics) = declaration_output_with_source_type(
            "type T = string; const visible: T = 'x'; const hidden = 1; export = visible;",
            common_js,
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("type T = string;"));
        assert!(code.contains("declare const visible: T;"));
        assert!(code.contains("export = visible;"));
        assert!(!code.contains("hidden"));

        let (code, diagnostics) = declaration_output_with_source_type(
            "declare global { interface Window { value: string } }",
            common_js,
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("declare global"));
        assert!(code.contains("export {}"));

        let (code, diagnostics) =
            declaration_output_with_source_type("const hidden = 1;", SourceType::Script);
        assert_eq!(diagnostics, 0);
        assert_eq!(code, "declare const hidden = 1;\n");
    }

    #[test]
    fn transparent_expression_parentheses() {
        let (plain, plain_diagnostics) =
            declaration_output("export const value = [-1, 2, `text`] as const;");
        let (wrapped, wrapped_diagnostics) =
            declaration_output("export const value = ((([-(1), ((2)), (`text`)])) as const);");
        assert_eq!(plain_diagnostics, 0);
        assert_eq!(wrapped_diagnostics, 0);
        assert_eq!(wrapped, plain);

        let (plain, plain_diagnostics) = declaration_output("export const value = -1;");
        let (wrapped, wrapped_diagnostics) = declaration_output("export const value = (-(1));");
        assert_eq!(plain_diagnostics, 0);
        assert_eq!(wrapped_diagnostics, 0);
        assert_eq!(wrapped, plain);

        let (plain, plain_diagnostics) = declaration_output(
            "export const value = { get ['x'](): number { return 1; }, set x(value: number) {} };",
        );
        let (wrapped, wrapped_diagnostics) = declaration_output(
            "export const value = { get [(('x'))](): number { return 1; }, set x(value: number) {} };",
        );
        assert_eq!(plain_diagnostics, 0);
        assert_eq!(wrapped_diagnostics, 0);
        assert_eq!(wrapped, plain);

        let (plain, plain_diagnostics) =
            declaration_output("export function f(): void {} f.x = 1;");
        let (wrapped, wrapped_diagnostics) =
            declaration_output("export function f(): void {} (((f).x = 1));");
        assert_eq!(plain_diagnostics, 1);
        assert_eq!(wrapped_diagnostics, plain_diagnostics);
        assert_eq!(wrapped, plain);

        let (plain, plain_diagnostics) =
            declaration_output("export const f = (): void => {}; f.x = 1;");
        let (wrapped, wrapped_diagnostics) =
            declaration_output("export const f = ((((): void => {}))); f.x = 1;");
        assert_eq!(plain_diagnostics, 1);
        assert_eq!(wrapped_diagnostics, plain_diagnostics);
        assert_eq!(wrapped, plain);

        let (plain, plain_diagnostics) =
            declaration_output("export const f = function(): void {}; f.x = 1;");
        let (wrapped, wrapped_diagnostics) =
            declaration_output("export const f = (((function(): void {}))); f.x = 1;");
        assert_eq!(plain_diagnostics, 1);
        assert_eq!(wrapped_diagnostics, plain_diagnostics);
        assert_eq!(wrapped, plain);

        let (plain, plain_diagnostics) = declaration_output(
            "export function f(): void {} export namespace f { export const x: number = 0; } f.x = 1;",
        );
        let (wrapped, wrapped_diagnostics) = declaration_output(
            "export function f(): void {} export namespace f { export const x: number = 0; } (((f).x = 1));",
        );
        assert_eq!(plain_diagnostics, 0);
        assert_eq!(wrapped_diagnostics, plain_diagnostics);
        assert_eq!(wrapped, plain);

        let (plain, plain_diagnostics) =
            declaration_output("export const value = { n: +1 } as const;");
        let (wrapped, wrapped_diagnostics) =
            declaration_output("export const value = { n: +((1)) } as const;");
        assert_eq!(plain_diagnostics, 0);
        assert_eq!(wrapped_diagnostics, plain_diagnostics);
        assert_eq!(wrapped, plain);

        let (plain, plain_diagnostics) = declaration_output("export const value = +1;");
        let (wrapped, wrapped_diagnostics) = declaration_output("export const value = +((1));");
        assert_eq!(plain_diagnostics, 0);
        assert_eq!(wrapped_diagnostics, plain_diagnostics);
        assert_eq!(wrapped, plain);

        let (_, plain_diagnostics) = declaration_output("export class C extends null {}");
        let (_, wrapped_diagnostics) = declaration_output("export class C extends ((null)) {}");
        assert_eq!(plain_diagnostics, 0);
        assert_eq!(wrapped_diagnostics, plain_diagnostics);
    }

    #[test]
    fn inferred_reference_identity() {
        let (code, diagnostics) = declaration_output("export function f(x?: number) { return x; }");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("f(x?: number): number | undefined"));

        let (code, diagnostics) =
            declaration_output("export function f(undefined?: number) { return (undefined); }");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("f(undefined?: number): number | undefined"));

        let (code, diagnostics) =
            declaration_output("export function f(x: number = 1) { return x; }");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("f(x?: number): number;"));

        let (code, diagnostics) = declaration_output(
            "type T = { value: string }; export function f(x?: T) { return x; }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("type T"));
        assert!(code.contains("f(x?: T): T | undefined"));

        let (code, diagnostics) =
            declaration_output("export declare namespace N { enum E { A = 5, B, C } }");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("A = 5"));
        assert!(!code.contains("B ="));
        assert!(!code.contains("C ="));

        let (code, diagnostics) = declaration_output("export declare const enum E { A = 5, B }");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("B = 6"));

        let (code, diagnostics) =
            declaration_output("export function f(undefined: number) { return (undefined); }");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("f(undefined: number): number"));

        let (code, diagnostics) =
            declaration_output("export enum E { Infinity = 2, A = Infinity, NaN = 3, B = NaN }");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("A = 2"));
        assert!(code.contains("B = 3"));

        let (code, diagnostics) =
            declaration_output("export enum E { A = '' + 1e21, B = '' + 1e-7 }");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("\"1e+21\""));
        assert!(code.contains("\"1e-7\""));

        let (code, diagnostics) = declaration_output(
            "export const value = { get '\\uD800'(): number { return 1; }, set '\\uD801'(value: string) {} };",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("readonly \"\\uD800\": number"));
        assert!(code.contains("\"\\uD801\": string"));
        let (code, diagnostics) =
            declaration_output("export enum E { A = +\"1\", B = -\"2\", C = ~\"3\" }");
        assert_eq!(diagnostics, 0);
        assert!(!code.contains('='));
    }

    #[test]
    fn computed_object_dependencies() {
        let (code, diagnostics) = declaration_output(
            "declare const key: unique symbol; export const o = { get [key](): number { return 1; }, set key(v: string) {} };",
        );
        assert_eq!(diagnostics, 1);
        assert!(!code.contains("declare const key"));
        assert!(code.contains("key: string"));
    }

    #[test]
    fn accessor_declaration_dependencies() {
        let (code, diagnostics) = declaration_output(
            "type Hidden = { x: number }; type Public = object; export const obj = { get value() { return {} as Hidden; }, set value(v: Public) {} };",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("Hidden"));
        assert!(code.contains("type Public"));
        assert!(code.contains("value: Public"));

        let (code, diagnostics) = declaration_output(
            "type Hidden = { x: number }; type Public = object; export class C { get value() { return {} as Hidden; } set value(v: Public) {} }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("Hidden"));
        assert!(code.contains("type Public"));
        assert!(code.contains("get value(): Public"));
        assert!(code.contains("set value(v: Public)"));

        let (code, diagnostics) = declaration_output(
            "type Read = { x: number }; type Write = object; export const obj = { get value(): Read { return { x: 1 }; }, set value(v: Write) {} };",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("type Read"));
        assert!(code.contains("type Write"));
        assert!(code.contains("get value(): Read"));
        assert!(code.contains("set value(v: Write)"));

        let (code, diagnostics) = declaration_output(
            "type Read = { x: number }; type Write = object; export const obj = { set value(v: Write) {}, get value(): Read { return { x: 1 }; } };",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("type Read"));
        assert!(code.contains("type Write"));
        assert!(code.contains("get value(): Read"));
        assert!(code.contains("set value(v: Write)"));
        let (code, diagnostics) = declaration_output(
            "import type { T } from './dep'; export declare class C { get value(): T; set value(v); }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("import type { T }"));
        assert!(code.contains("set value(v: T)"));

        let (code, diagnostics) = declaration_output(
            "export abstract class C { abstract set value(v); abstract get value(): string; }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("set value(v: string)"));
    }

    #[test]
    fn numeric_member_identity() {
        let (code, diagnostics) = declaration_output(
            "export const obj = { get [-1](): number { return 1; }, set ['-1'](v: string) {} };",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("get [-1](): number"));
        assert!(code.contains("set [\"-1\"](v: string)"));

        let (code, diagnostics) = declaration_output(
            "export class C { [-1](x: string): string; ['-1'](x: number): number; [-(1)](x: string | number): string | number { return x; } }",
        );
        assert_eq!(diagnostics, 0);
        assert_eq!(code.matches("(x:").count(), 2);
        assert!(!code.contains("string | number"));

        let (code, diagnostics) = declaration_output(
            "export const obj = { get [-(0)](): number { return 1; }, set [0](v: string) {} };",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("get [0](): number"));
        assert!(code.contains("set [0](v: string)"));

        let (code, diagnostics) = declaration_output(
            "export class C { get [-0](): number { return 1; } set [(0)](v: string) {} }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("get [0](): number"));
        assert!(code.contains("set [0](v: string)"));
    }

    #[test]
    fn private_setter_type_dependencies() {
        let (code, diagnostics) = declaration_output(
            "type Public = object; type Hidden = { x: number }; export class C { get value() { return {} as Hidden; } private set value(v: Public) {} }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("Hidden"));
        assert!(code.contains("type Public"));
        assert!(code.contains("get value(): Public"));
        assert!(code.contains("private set value(value)"));

        let (code, diagnostics) = declaration_output(
            "type Public = object; type Hidden = { x: number }; export class C { private set value(v: Public) {} get value() { return {} as Hidden; } }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("Hidden"));
        assert!(code.contains("get value(): Public"));

        let (code, diagnostics) = declaration_output(
            "type Hidden = object; export class C { private set value(v: Hidden) {} }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("Hidden"));

        let (code, diagnostics) = declaration_output(
            "interface Hidden { x: string } export declare class C { private get value(): Hidden; private set value(v: Hidden); }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("Hidden"));
        assert!(code.contains("private get value()"));
        assert!(code.contains("private set value(value)"));

        let (code, diagnostics) = declaration_output(
            "interface Hidden { x: string } export declare class C { private get value(): Hidden; }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("Hidden"));

        let (code, diagnostics) = declaration_output(
            "interface Hidden { x: string } export declare class C { private set value(v: Hidden); }",
        );
        assert_eq!(diagnostics, 0);
        assert!(!code.contains("Hidden"));

        let (code, diagnostics) = declaration_output(
            "interface Hidden { x: string } export declare class C { get value(); private set value(v: Hidden); }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("interface Hidden"));
        assert!(code.contains("get value(): Hidden"));
        assert!(code.contains("private set value(value)"));
    }

    #[test]
    fn parameter_properties() {
        let (code, diagnostics) = declaration_output(
            "export class C { constructor(public value = undefined as number | undefined) {} }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("value: number | undefined;"));
        assert!(code.contains("constructor(value?: number | undefined)"));

        let (code, diagnostics) = declaration_output(
            "export class C { constructor(public value = 1, required: string) {} }",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("value: number;"));
        assert!(code.contains("value: number | undefined, required: string"));

        let (_, diagnostics) = declaration_output(
            "declare function compute(): number; export class C { constructor(public value = compute()) {} }",
        );
        assert_eq!(diagnostics, 1);
    }

    #[test]
    fn declaration_diagnostics() {
        let (code, diagnostics) =
            declaration_output("export = class C { clone(): C { return this; } };");
        assert!(diagnostics > 0);
        assert!(!code.contains("clone(): C"));

        let (code, diagnostics) =
            declaration_output("export = (class C { clone(): C { return this; } });");
        assert!(diagnostics > 0);
        assert!(!code.contains("clone(): C"));

        let (code, diagnostics) =
            declaration_output("export = class C { m(): void { new C(); } };");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("m(): void"));

        let (code, diagnostics) = declaration_output("export = (class { x: number = 1; });");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("x: number"));

        let (code, diagnostics) =
            declaration_output("export default class C { clone(): C { return this; } }");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("clone(): C"));

        let (code, diagnostics) = declaration_output(
            "interface C { value: string }; export const value = {} as C; export default (class C { x: number = 1; });",
        );
        assert_eq!(diagnostics, 0);
        assert!(code.contains("interface C"));
        assert!(code.contains("x: number"));

        let (_, diagnostics) = declaration_output(
            "export function f() { type Local = string; return 'hello' as Local; }",
        );
        assert_eq!(diagnostics, 1);

        let (_, diagnostics) = declaration_output(
            "export function f() { const value = 1; return 1 as typeof value; }",
        );
        assert_eq!(diagnostics, 1);

        let (_, diagnostics) = declaration_output(
            "export function f() { type Local = string; return { get value(): Local { return ''; }, set value(v) {} }; }",
        );
        assert_eq!(diagnostics, 2);

        let (code, diagnostics) =
            declaration_output("export function f<T>(value: T) { return value as T; }");
        assert_eq!(diagnostics, 0);
        assert!(code.contains("f<T>(value: T): T"));

        let (_, diagnostics) =
            declaration_output("export function f() { return <T>(value: T): T => value; }");
        assert_eq!(diagnostics, 0);

        let (_, diagnostics) = declaration_output(
            "function hidden() { type Local = string; return '' as Local; } export const value = 1;",
        );
        assert_eq!(diagnostics, 0);

        let (_, diagnostics) = declaration_output("export let value;");
        assert_eq!(diagnostics, 1);

        let (_, diagnostics) = declaration_output("export class C { value; }");
        assert_eq!(diagnostics, 1);
    }
}
