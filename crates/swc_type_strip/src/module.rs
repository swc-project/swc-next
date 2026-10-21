use swc_next_ecma_ast::{
    Ast, Decl, DeclData, ExportDefaultDeclarationKindData, Expr, GetSpan, ImportOrExportKind,
    NodeKindData, Span, Stmt, StmtData, TsGlobalDeclaration, TsModuleBlock, TsModuleDeclaration,
    Visit, VisitWith,
};

pub(super) fn check(module: TsModuleDeclaration, ast: &Ast<'_>) -> Option<Span> {
    check_declaration(module, ast, !module.declare(ast))
}

pub(super) fn check_global(global: TsGlobalDeclaration, ast: &Ast<'_>) -> Option<Span> {
    check_declaration(global, ast, !global.declare(ast))
}

fn check_declaration<'ast, N>(
    node: N,
    ast: &'ast Ast<'ast>,
    check_instantiation: bool,
) -> Option<Span>
where
    N: GetSpan + VisitWith<'ast, ModuleChecker<'ast>>,
{
    let span = node.span(ast);
    let mut checker = ModuleChecker {
        ast,
        instantiation: InstantiationChecker::default(),
        erasable: ErasableSyntaxChecker::default(),
        check_instantiation,
        statements: Vec::new(),
        expressions: Vec::new(),
    };
    node.visit_with(&mut checker);
    if checker.instantiation.instantiated {
        return Some(span);
    }

    // Only the namespace structure can override an inner diagnostic. Resume
    // deferred children in source order without visiting their statements again.
    checker.check_instantiation = false;
    for (statement, ambient_depth) in std::mem::take(&mut checker.statements) {
        if !checker.erasable.precedes_diagnostic(statement.span(ast)) {
            break;
        }
        checker.erasable.ambient_depth = ambient_depth;
        statement.visit_children_with(&mut checker);
    }
    while let Some((expression, ambient_depth)) = checker.expressions.pop() {
        if checker.erasable.precedes_diagnostic(expression.span(ast)) {
            checker.erasable.ambient_depth = ambient_depth;
            expression.visit_children_with(&mut checker);
        }
    }
    checker.erasable.diagnostic
}

/// Shares traversal between the two checks, keeping structural short circuits
/// ahead of syntax checks inside types and initializers.
struct ModuleChecker<'ast> {
    ast: &'ast Ast<'ast>,
    instantiation: InstantiationChecker,
    erasable: ErasableSyntaxChecker,
    check_instantiation: bool,
    statements: Vec<(Stmt, u32)>,
    expressions: Vec<(Expr, u32)>,
}

impl<'ast> Visit<'ast> for ModuleChecker<'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn enter_node(&mut self, node: NodeKindData) {
        self.erasable.enter_node(node, self.ast);
    }

    fn leave_node(&mut self, node: NodeKindData) {
        self.erasable.leave_node(node, self.ast);
    }

    fn visit_stmt(&mut self, node: Stmt) {
        if self.erasable.precedes_diagnostic(node.span(self.ast)) {
            node.visit_children_with(self);
        }
    }

    fn visit_ts_module_declaration(&mut self, node: TsModuleDeclaration) {
        if self.check_instantiation {
            self.instantiation.record_module(node, self.ast);
        }
        if !self.instantiation.instantiated {
            self.enter_node(NodeKindData::TsModuleDeclaration(node));
            node.visit_children_with(self);
            self.leave_node(NodeKindData::TsModuleDeclaration(node));
        }
    }

    fn visit_ts_module_block(&mut self, node: TsModuleBlock) {
        if !self.check_instantiation {
            node.visit_children_with(self);
            return;
        }

        self.instantiation.record_block(node, self.ast);
        for index in node.body(self.ast).iter() {
            if self.instantiation.instantiated {
                break;
            }
            let statement = self.ast.get_node_in_sub_range(index);
            match self.instantiation.record_statement(statement, self.ast) {
                StatementKind::Runtime => break,
                StatementKind::Namespace => statement.visit_children_with(self),
                StatementKind::Erased => {
                    self.statements
                        .push((statement, self.erasable.ambient_depth));
                }
            }
        }
    }

    fn visit_expr(&mut self, node: Expr) {
        // The parser accepts long expression chains. Defer expression children
        // so checking an erased initializer does not recurse down those chains.
        if self.erasable.precedes_diagnostic(node.span(self.ast)) {
            self.expressions.push((node, self.erasable.ambient_depth));
        }
    }
}

enum StatementKind {
    Namespace,
    Erased,
    Runtime,
}

/// Tracks whether the declaration needs a runtime namespace object. Nested
/// `declare` modifiers do not affect this structural classification.
#[derive(Default)]
struct InstantiationChecker {
    instantiated: bool,
}

impl InstantiationChecker {
    fn record_module(&mut self, module: TsModuleDeclaration, ast: &Ast<'_>) {
        self.instantiated |= module.body(ast).is_none();
    }

    fn record_block(&mut self, block: TsModuleBlock, ast: &Ast<'_>) {
        // Directives are expression statements, stored separately in this AST.
        self.instantiated |= !block.directives(ast).is_empty();
    }

    #[inline]
    fn record_statement(&mut self, statement: Stmt, ast: &Ast<'_>) -> StatementKind {
        let kind = match ast.stmt_data(statement) {
            StmtData::Declaration(declaration) => Self::declaration_kind(declaration, ast),
            StmtData::ImportDeclaration(_) => StatementKind::Erased,
            StmtData::ExportNamedDeclaration(export) => match export.declaration(ast) {
                Some(declaration) if declaration.is_ts_import_equals_declaration(ast) => {
                    StatementKind::Runtime
                }
                Some(declaration) => Self::declaration_kind(declaration, ast),
                // TypeScript instantiates namespaces containing external re-exports.
                None if export.source(ast).is_some() => StatementKind::Runtime,
                None if export.export_kind(ast) == ImportOrExportKind::Type
                    || export.specifiers(ast).iter().all(|specifier| {
                        ast.get_node_in_sub_range(specifier).export_kind(ast)
                            == ImportOrExportKind::Type
                    }) =>
                {
                    StatementKind::Erased
                }
                // Unmarked local exports require name resolution.
                None => StatementKind::Runtime,
            },
            StmtData::ExportDefaultDeclaration(export)
                if matches!(
                    ast.export_default_declaration_kind_data(export.declaration(ast)),
                    ExportDefaultDeclarationKindData::TsInterfaceDeclaration(_)
                ) =>
            {
                StatementKind::Erased
            }
            _ => StatementKind::Runtime,
        };
        self.instantiated |= matches!(kind, StatementKind::Runtime);
        kind
    }

    fn declaration_kind(declaration: Decl, ast: &Ast<'_>) -> StatementKind {
        match ast.decl_data(declaration) {
            DeclData::TsModuleDeclaration(_) | DeclData::TsGlobalDeclaration(_) => {
                StatementKind::Namespace
            }
            DeclData::TsInterfaceDeclaration(_)
            | DeclData::TsTypeAliasDeclaration(_)
            | DeclData::TsImportEqualsDeclaration(_) => StatementKind::Erased,
            // Const enums still need their own erasableSyntaxOnly check.
            DeclData::TsEnumDeclaration(enumeration) if enumeration.is_const(ast) => {
                StatementKind::Erased
            }
            _ => StatementKind::Runtime,
        }
    }
}

/// Checks erased members without creating edits inside the erased container.
#[derive(Default)]
struct ErasableSyntaxChecker {
    ambient_depth: u32,
    diagnostic: Option<Span>,
}

impl ErasableSyntaxChecker {
    fn precedes_diagnostic(&self, span: Span) -> bool {
        self.diagnostic
            .is_none_or(|diagnostic| span.start < diagnostic.start)
    }

    fn enter_node(&mut self, node: NodeKindData, ast: &Ast<'_>) {
        if declares_ambient_context(node, ast) {
            self.ambient_depth += 1;
        }
        let span = match node {
            NodeKindData::TsEnumDeclaration(enumeration) if self.ambient_depth == 0 => {
                enumeration.span(ast)
            }
            NodeKindData::TsImportEqualsDeclaration(import) if self.ambient_depth == 0 => {
                import.span(ast)
            }
            NodeKindData::TsExportAssignment(export) if self.ambient_depth == 0 => export.span(ast),
            NodeKindData::TsParameterProperty(parameter) => parameter.span(ast),
            NodeKindData::TsTypeAssertion(assertion) => Span::new(
                assertion.span(ast).start,
                assertion.expression(ast).span(ast).start,
            ),
            _ => return,
        };
        if self.precedes_diagnostic(span) {
            self.diagnostic = Some(span);
        }
    }

    fn leave_node(&mut self, node: NodeKindData, ast: &Ast<'_>) {
        if declares_ambient_context(node, ast) {
            self.ambient_depth -= 1;
        }
    }
}

fn declares_ambient_context(node: NodeKindData, ast: &Ast<'_>) -> bool {
    match node {
        NodeKindData::VariableDeclaration(declaration) => declaration.declare(ast),
        NodeKindData::Class(class) => class.declare(ast),
        NodeKindData::TsFunction(function) => function.declare(ast),
        NodeKindData::TsTypeAliasDeclaration(alias) => alias.declare(ast),
        NodeKindData::TsInterfaceDeclaration(interface) => interface.declare(ast),
        NodeKindData::TsEnumDeclaration(enumeration) => enumeration.declare(ast),
        NodeKindData::TsModuleDeclaration(module) => module.declare(ast),
        NodeKindData::TsGlobalDeclaration(global) => global.declare(ast),
        _ => false,
    }
}
