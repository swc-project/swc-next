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
mod tests;
