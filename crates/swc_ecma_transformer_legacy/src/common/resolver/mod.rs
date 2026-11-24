//! Restored from swc-next before removal of the legacy name resolver.
//! Only the AST accessors, collection types and NodeId-indexed storage are adapted.
//! <https://github.com/swc-project/swc-next/blob/947716b618c452d85beb03a616c686ae0fd210e1/crates/swc_ecma_semantic/src/name_resolver/resolver.rs#L1-L992>

pub(crate) mod context;

use self::context::{NodeContext, Symbols};
use std::collections::HashSet;

use rustc_hash::FxHashMap;

use swc_next_allocator::{Idx, IndexVec};

use swc_next_ecma_ast::*;

use crate::common::build::for_each_binding_ident;

/// Resolve every value identifier in `ast` using the legacy resolver's
/// hoist-then-resolve algorithm.
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L136>.
fn resolver<'ast>(ast: &'ast Ast<'ast>) -> JsNameResolver {
    let mut scopes = IndexVec::new();
    let unresolved_scope_id = scopes.push(Scope::new(ScopeKind::Fn, None));
    let top_level_scope_id = scopes.push(Scope::new(ScopeKind::Fn, None));
    let label_scope_id = ScopeId::from_usize(ScopeId::MAX);

    let mut resolver = Resolver {
        ast,
        scopes,
        nodes: IndexVec::from_vec(vec![NodeContext::None; ast.node_count()]),
        symbols: Symbols::default(),
        top_level_scope_id,
        unresolved_scope_id,
        label_scope_id,
        current: top_level_scope_id,
        is_module: false,
        decl_kind: DeclKind::Lexical,
        strict_mode: false,
    };

    ast.root_program().visit_with(&mut resolver);

    JsNameResolver {
        top_level_scope_id,
        unresolved_scope_id,
        next_scope: resolver.scopes.len(),
        nodes: resolver.nodes,
        symbols: resolver.symbols,
    }
}

struct JsNameResolver {
    top_level_scope_id: ScopeId,
    unresolved_scope_id: ScopeId,
    nodes: IndexVec<NodeId, NodeContext>,
    symbols: Symbols,
    next_scope: usize,
}

#[derive(Debug)]
struct Scope<'ast> {
    parent: Option<ScopeId>,
    kind: ScopeKind,
    declared_symbols: FxHashMap<&'ast [u8], Binding>,
    declared_types: FxHashMap<&'ast [u8], SymbolId>,
}

#[derive(Clone, Copy, Debug)]
struct Binding {
    kind: DeclKind,
    // Bare enum member references use the unresolved context for TS lowering.
    symbol: Option<SymbolId>,
}

impl<'ast> Scope<'ast> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L193>.
    fn new(kind: ScopeKind, parent: Option<ScopeId>) -> Self {
        Self {
            parent,
            kind,
            declared_symbols: FxHashMap::default(),
            declared_types: FxHashMap::default(),
        }
    }
}

/// AST nodes that can own a scope created by the name resolver.
///
/// Keep this list closed so `with_child` cannot accidentally attach a scope to
/// an arbitrary AST node. `ImplicitBlockStatement` is used for the implicit
/// scope around a loop body, which may be any statement rather than only a
/// block statement.
enum ScopeOwner {
    Program(Program),
    ImplicitBlockStatement(Stmt),
    BlockStatement(BlockStatement),
    CatchClause(CatchClause),
    Function(Function),
    FunctionBody(FunctionBody),
    SetterBody(FunctionBody),
    ArrowFunctionExpression(ArrowFunctionExpression),
    Class(Class),
    StaticBlock(StaticBlock),
    ObjectExpression(ObjectExpression),
    ForStatement(ForStatement),
    ForInStatement(ForInStatement),
    ForOfStatement(ForOfStatement),
    SwitchStatement(SwitchStatement),
    TsModuleDeclaration(TsModuleDeclaration),
    TsEnumDeclaration(TsEnumDeclaration),
}

impl ScopeOwner {
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L427>.
    fn node_id(self) -> NodeId {
        match self {
            Self::Program(node) => node.node_id(),
            Self::ImplicitBlockStatement(node) => node.node_id(),
            Self::BlockStatement(node) => node.node_id(),
            Self::CatchClause(node) => node.node_id(),
            Self::Function(node) => node.node_id(),
            Self::FunctionBody(node) => node.node_id(),
            Self::SetterBody(node) => node.node_id(),
            Self::ArrowFunctionExpression(node) => node.node_id(),
            Self::Class(node) => node.node_id(),
            Self::StaticBlock(node) => node.node_id(),
            Self::ObjectExpression(node) => node.node_id(),
            Self::ForStatement(node) => node.node_id(),
            Self::ForInStatement(node) => node.node_id(),
            Self::ForOfStatement(node) => node.node_id(),
            Self::SwitchStatement(node) => node.node_id(),
            Self::TsModuleDeclaration(node) => node.node_id(),
            Self::TsEnumDeclaration(node) => node.node_id(),
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L427>.
    fn kind(&self) -> ScopeKind {
        match self {
            Self::Program(_)
            | Self::CatchClause(_)
            | Self::Function(_)
            | Self::FunctionBody(_)
            | Self::ArrowFunctionExpression(_)
            | Self::Class(_) => ScopeKind::Fn,
            Self::ImplicitBlockStatement(_)
            | Self::BlockStatement(_)
            | Self::SetterBody(_)
            | Self::StaticBlock(_)
            | Self::ObjectExpression(_)
            | Self::ForStatement(_)
            | Self::ForInStatement(_)
            | Self::ForOfStatement(_)
            | Self::SwitchStatement(_)
            | Self::TsModuleDeclaration(_)
            | Self::TsEnumDeclaration(_) => ScopeKind::Block,
        }
    }
}

/// The resolver has the same two phases as the legacy implementation:
/// declarations visible in a scope are hoisted first, then references are
/// resolved while walking the source in order.
struct Resolver<'ast> {
    ast: &'ast Ast<'ast>,
    scopes: IndexVec<ScopeId, Scope<'ast>>,
    nodes: IndexVec<NodeId, NodeContext>,
    symbols: Symbols,
    top_level_scope_id: ScopeId,
    unresolved_scope_id: ScopeId,
    label_scope_id: ScopeId,
    current: ScopeId,
    is_module: bool,
    decl_kind: DeclKind,
    strict_mode: bool,
}

impl<'ast> Resolver<'ast> {
    /// Next AST adapter for SWC's nested `TsNamespaceDecl` representation.
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1420>.
    fn visit_namespace_root(&mut self, name: TsQualifiedName) {
        match self.ast.ts_qualified_name_left_data(name.left(self.ast)) {
            TsQualifiedNameLeftData::BindingIdentifier(id) => self.modify(id, DeclKind::Lexical),
            TsQualifiedNameLeftData::TsQualifiedName(name) => self.visit_namespace_root(name),
            _ => {}
        }
    }

    /// SWC type declarations are separate from runtime declarations. Only
    /// import/export classification needs their contexts after erasure.
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L334>.
    fn declare_type(&mut self, identifier: BindingIdentifier) {
        let name = identifier.name(self.ast);
        let symbol = self.symbols.intern(self.ast, self.current, name);
        let bytes = self.name(name);
        self.scopes[self.current]
            .declared_types
            .insert(bytes, symbol);
        self.set_symbol_once(identifier.node_id(), symbol);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L373>.
    fn resolve_export(&mut self, name: AstUtf8) -> SymbolId {
        let bytes = self.name(name);
        if let Some(symbol) = self.resolve_value(bytes) {
            return symbol;
        }
        let mut scope = Some(self.current);
        while let Some(id) = scope {
            if let Some(&symbol) = self.scopes[id].declared_types.get(bytes) {
                return symbol;
            }
            scope = self.scopes[id].parent;
        }
        self.symbols
            .intern(self.ast, self.unresolved_scope_id, name)
    }

    /// Next AST adapter for SWC's nested namespace declarations.
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1420>.
    fn declare_namespace_segments(&mut self, name: TsQualifiedName) {
        if let Some(left) = name.left(self.ast).as_ts_qualified_name(self.ast) {
            self.declare_namespace_segments(left);
        }
        let right = name.right(self.ast);
        let symbol = self.declare(right.name(self.ast), DeclKind::Lexical);
        self.set_symbol_once(right.node_id(), symbol);
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L427>.
    fn name(&self, name: AstUtf8) -> &'ast [u8] {
        self.ast.get_utf8(name).as_bytes()
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L219>.
    fn is_declared(&self, name: &[u8], start_scope: ScopeId) -> Option<DeclKind> {
        let mut scope = Some(start_scope);
        while let Some(scope_id) = scope {
            let current = &self.scopes[scope_id];
            if let Some(binding) = current.declared_symbols.get(name) {
                return Some(binding.kind);
            }
            scope = current.parent;
        }
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L284>.
    fn with_child<F>(&mut self, owner: ScopeOwner, op: F)
    where
        F: FnOnce(&mut Self),
    {
        let scope = self.current;
        let decl_kind = self.decl_kind;
        let strict_mode = self.strict_mode;

        let kind = owner.kind();
        let child_scope = self.scopes.push(Scope::new(kind, Some(scope)));
        self.current = child_scope;
        self.set_owner_scope_once(owner, child_scope);
        op(self);

        self.current = scope;
        self.decl_kind = decl_kind;
        self.strict_mode = strict_mode;
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L427>.
    fn set_symbol_once(&mut self, node: NodeId, symbol: SymbolId) {
        if matches!(self.nodes[node], NodeContext::None) {
            self.nodes[node] = NodeContext::Symbol(symbol);
        }
    }

    fn set_owner_scope_once(&mut self, owner: ScopeOwner, scope: ScopeId) {
        let slot = &mut self.nodes[owner.node_id()];
        if matches!(slot, NodeContext::None) {
            *slot = NodeContext::Scope(scope);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L366>.
    fn lookup_value(&self, name: &[u8], stop_at_fn_scope: bool) -> Option<(ScopeId, Binding)> {
        let mut scope = Some(self.current);
        while let Some(scope_id) = scope {
            let current = &self.scopes[scope_id];
            if let Some(&binding) = current.declared_symbols.get(name) {
                return Some((scope_id, binding));
            }
            if current.kind == ScopeKind::Fn && stop_at_fn_scope {
                return None;
            }
            scope = current.parent;
        }
        None
    }

    fn resolve_value(&self, name: &[u8]) -> Option<SymbolId> {
        let (scope, binding) = self.lookup_value(name, false)?;
        if scope == self.top_level_scope_id
            && !self.is_module
            && matches!(name, b"undefined" | b"NaN" | b"Infinity")
        {
            return None;
        }
        binding.symbol
    }

    fn resolve_ref(&mut self, name: AstUtf8) -> SymbolId {
        self.resolve_value(self.name(name)).unwrap_or_else(|| {
            self.symbols
                .intern(self.ast, self.unresolved_scope_id, name)
        })
    }

    fn declare(&mut self, name: AstUtf8, kind: DeclKind) -> SymbolId {
        let bytes = self.name(name);
        let binding = self.scopes[self.current]
            .declared_symbols
            .entry(bytes)
            .or_insert(Binding { kind, symbol: None });
        binding.kind = kind;
        *binding
            .symbol
            .get_or_insert_with(|| self.symbols.intern(self.ast, self.current, name))
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L427>.
    fn modify(&mut self, identifier: BindingIdentifier, kind: DeclKind) {
        let node_index = identifier.node_id();
        if !matches!(self.nodes[node_index], NodeContext::None) {
            return;
        }

        let symbol = self.declare(identifier.name(self.ast), kind);
        self.nodes[node_index] = NodeContext::Symbol(symbol);
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L988>.
    fn predeclare_parameters(&mut self, parameters: FormalParameters) {
        for id in parameters.items(self.ast).iter() {
            let item = self.ast.get_node_in_sub_range(id);
            let pattern = match self.ast.formal_parameter_item_data(item) {
                FormalParameterItemData::FormalParameter(parameter) => parameter.pattern(self.ast),
                FormalParameterItemData::TsParameterProperty(property) => {
                    property.parameter(self.ast).pattern(self.ast)
                }
            };
            let ast = self.ast;
            for_each_binding_ident(ast, pattern.into(), |identifier| {
                self.modify(identifier, DeclKind::Param);
            });
        }

        if let Some(rest) = parameters.rest(self.ast) {
            let ast = self.ast;
            for_each_binding_ident(ast, rest.argument(ast).into(), |identifier| {
                self.modify(identifier, DeclKind::Param);
            });
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1261>.
    fn has_use_strict(&self, directives: SubRange<Directive>) -> bool {
        directives.iter().any(|id| {
            let directive = self.ast.get_node_in_sub_range(id);
            self.name(directive.value(self.ast)) == b"use strict"
        })
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L988>.
    fn visit_function_contents(&mut self, function: Function) {
        let parameters = function.params(self.ast);
        self.predeclare_parameters(parameters);
        let old_kind = self.decl_kind;
        self.decl_kind = DeclKind::Param;
        parameters.visit_with(self);
        self.decl_kind = old_kind;

        let body = function.body(self.ast);
        self.set_owner_scope_once(ScopeOwner::FunctionBody(body), self.current);
        let old_strict_mode = self.strict_mode;
        if !self.strict_mode {
            self.strict_mode = self.has_use_strict(body.directives(self.ast));
        }
        body.body(self.ast).visit_with(self);
        self.strict_mode = old_strict_mode;
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L355>.
    fn visit_stmt_within_child_scope(&mut self, statement: Stmt) {
        self.with_child(ScopeOwner::ImplicitBlockStatement(statement), |child| {
            if let StmtData::BlockStatement(block) = child.ast.stmt_data(statement) {
                child.set_owner_scope_once(ScopeOwner::BlockStatement(block), child.current);
                block.body(child.ast).visit_with(child);
            } else {
                statement.visit_with(child);
            }
        });
    }
}

impl<'ast> Visit<'ast> for Resolver<'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1163>.
    fn visit_program(&mut self, program: Program) {
        self.set_owner_scope_once(ScopeOwner::Program(program), self.current);
        self.is_module = self.ast.is_module();
        self.strict_mode = self.is_module || self.has_use_strict(program.directives(self.ast));
        program.body(self.ast).visit_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1278>.
    fn visit_stmts(&mut self, statements: SubRange<Stmt>) {
        {
            let kind = self.decl_kind;
            let mut hoister = Hoister {
                resolver: self,
                kind,
                in_block: false,
                in_catch_body: false,
                excluded_from_catch: HashSet::default(),
                catch_param_decls: HashSet::default(),
            };
            statements.visit_with(&mut hoister);
        }
        statements.visit_children_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1055>.
    fn visit_identifier_reference(&mut self, reference: IdentifierReference) {
        let node_index = reference.node_id();
        if !matches!(self.nodes[node_index], NodeContext::None) {
            return;
        }
        let symbol = self.resolve_ref(reference.name(self.ast));
        self.nodes[node_index] = NodeContext::Symbol(symbol);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L684>.
    fn visit_binding_identifier(&mut self, identifier: BindingIdentifier) {
        if self.decl_kind != DeclKind::Param {
            self.modify(identifier, self.decl_kind);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1154>.
    fn visit_label_identifier(&mut self, label: LabelIdentifier) {
        let symbol = self
            .symbols
            .intern(self.ast, self.label_scope_id, label.name(self.ast));
        self.set_symbol_once(label.node_id(), symbol);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1031>.
    fn visit_jsx_element_name(&mut self, name: JsxElementName) {
        match self.ast.jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                let name = identifier.name(self.ast);
                let symbol = if self.name(name).first().is_some_and(u8::is_ascii_lowercase) {
                    self.symbols
                        .intern(self.ast, self.unresolved_scope_id, name)
                } else {
                    self.resolve_ref(name)
                };
                self.set_symbol_once(identifier.node_id(), symbol);
            }
            JsxElementNameData::JsxMemberExpression(member) => {
                member.object(self.ast).visit_with(self);
            }
            JsxElementNameData::JsxNamespacedName(_) => {}
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1031>.
    fn visit_jsx_member_expression_object(&mut self, object: JsxMemberExpressionObject) {
        match self.ast.jsx_member_expression_object_data(object) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                let symbol = self.resolve_ref(identifier.name(self.ast));
                self.set_symbol_once(identifier.node_id(), symbol);
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                member.object(self.ast).visit_with(self);
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L698>.
    fn visit_block_statement(&mut self, block: BlockStatement) {
        self.with_child(ScopeOwner::BlockStatement(block), |child| {
            block.body(child.ast).visit_with(child);
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L712>.
    fn visit_catch_clause(&mut self, catch: CatchClause) {
        let body = catch.body(self.ast);
        self.with_child(ScopeOwner::CatchClause(catch), |child| {
            child.set_owner_scope_once(ScopeOwner::BlockStatement(body), child.current);
            catch.param(child.ast).visit_with(child);
            body.body(child.ast).visit_with(child);
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L988>.
    fn visit_function(&mut self, function: Function) {
        match function.function_type(self.ast) {
            FunctionType::FunctionDeclaration => {
                let body = function.body(self.ast);
                self.with_child(ScopeOwner::FunctionBody(body), |child| {
                    child.visit_function_contents(function);
                });
            }
            FunctionType::FunctionExpression => {
                if let Some(identifier) = function.id(self.ast) {
                    self.with_child(ScopeOwner::Function(function), |name_scope| {
                        name_scope.modify(identifier, DeclKind::Function);
                        let body = function.body(name_scope.ast);
                        name_scope.with_child(ScopeOwner::FunctionBody(body), |child| {
                            child.visit_function_contents(function);
                        });
                    });
                } else {
                    let body = function.body(self.ast);
                    self.with_child(ScopeOwner::FunctionBody(body), |child| {
                        child.visit_function_contents(function);
                    });
                }
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L630>.
    fn visit_arrow_function_expression(&mut self, arrow: ArrowFunctionExpression) {
        self.with_child(ScopeOwner::ArrowFunctionExpression(arrow), |child| {
            let parameters = arrow.params(child.ast);
            child.predeclare_parameters(parameters);
            let old_kind = child.decl_kind;
            child.decl_kind = DeclKind::Param;
            parameters.visit_with(child);
            child.decl_kind = old_kind;

            match child.ast.arrow_function_body_data(arrow.body(child.ast)) {
                ArrowFunctionBodyData::FunctionBody(body) => {
                    child.set_owner_scope_once(ScopeOwner::FunctionBody(body), child.current);
                    let old_strict_mode = child.strict_mode;
                    if !child.strict_mode {
                        child.strict_mode = child.has_use_strict(body.directives(child.ast));
                    }
                    body.body(child.ast).visit_with(child);
                    child.strict_mode = old_strict_mode;
                }
                ArrowFunctionBodyData::Expr(expression) => expression.visit_with(child),
            }
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L750>.
    fn visit_class(&mut self, class: Class) {
        match class.class_type(self.ast) {
            ClassType::ClassDeclaration => {
                if let Some(identifier) = class.id(self.ast) {
                    self.modify(identifier, DeclKind::Lexical);
                }
                self.with_child(ScopeOwner::Class(class), |child| {
                    let old_strict_mode = child.strict_mode;
                    child.strict_mode = true;
                    class.decorators(child.ast).visit_with(child);
                    class.super_class(child.ast).visit_with(child);
                    class.body(child.ast).visit_with(child);
                    child.strict_mode = old_strict_mode;
                });
            }
            ClassType::ClassExpression => {
                class.super_class(self.ast).visit_with(self);
                self.with_child(ScopeOwner::Class(class), |child| {
                    if let Some(identifier) = class.id(child.ast) {
                        child.modify(identifier, DeclKind::Lexical);
                    }
                    let old_strict_mode = child.strict_mode;
                    child.strict_mode = true;
                    class.decorators(child.ast).visit_with(child);
                    class.super_class(child.ast).visit_with(child);
                    class.body(child.ast).visit_with(child);
                    child.strict_mode = old_strict_mode;
                });
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L784>.
    fn visit_method_definition(&mut self, method: MethodDefinition) {
        method.decorators(self.ast).visit_with(self);
        method.key(self.ast).visit_with(self);
        let function = method.value(self.ast);
        let body = function.body(self.ast);
        self.with_child(ScopeOwner::FunctionBody(body), |child| {
            child.visit_function_contents(function);
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L794>.
    fn visit_property_definition(&mut self, property: PropertyDefinition) {
        property.decorators(self.ast).visit_with(self);
        if property.computed(self.ast) {
            property.key(self.ast).visit_with(self);
        }
        property.value(self.ast).visit_with(self);
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L136>.
    fn visit_static_block(&mut self, block: StaticBlock) {
        self.with_child(ScopeOwner::StaticBlock(block), |child| {
            block.body(child.ast).visit_with(child);
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1211>.
    fn visit_object_expression(&mut self, object: ObjectExpression) {
        self.with_child(ScopeOwner::ObjectExpression(object), |child| {
            object.visit_children_with(child);
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1244>.
    fn visit_object_property(&mut self, property: ObjectProperty) {
        property.key(self.ast).visit_with(self);
        let value = property.value(self.ast);
        let ExprData::Function(function) = self.ast.expr_data(value) else {
            value.visit_with(self);
            return;
        };

        let body = function.body(self.ast);
        match property.kind(self.ast) {
            PropertyKind::Get => {
                self.with_child(ScopeOwner::FunctionBody(body), |child| {
                    child.visit_function_contents(function);
                });
            }
            PropertyKind::Set => {
                self.with_child(ScopeOwner::Function(function), |child| {
                    let parameters = function.params(child.ast);
                    child.predeclare_parameters(parameters);
                    let old_kind = child.decl_kind;
                    child.decl_kind = DeclKind::Param;
                    parameters.visit_with(child);
                    child.decl_kind = old_kind;
                    child.with_child(ScopeOwner::SetterBody(body), |body_child| {
                        let old_strict_mode = body_child.strict_mode;
                        if !body_child.strict_mode {
                            body_child.strict_mode =
                                body_child.has_use_strict(body.directives(body_child.ast));
                        }
                        body.body(body_child.ast).visit_with(body_child);
                        body_child.strict_mode = old_strict_mode;
                    });
                });
            }
            PropertyKind::Init if property.method(self.ast) => {
                self.with_child(ScopeOwner::FunctionBody(body), |child| {
                    child.visit_function_contents(function);
                });
            }
            PropertyKind::Init => value.visit_with(self),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L975>.
    fn visit_for_statement(&mut self, statement: ForStatement) {
        self.with_child(ScopeOwner::ForStatement(statement), |child| {
            statement.init(child.ast).visit_with(child);
            statement.test(child.ast).visit_with(child);
            statement.update(child.ast).visit_with(child);
            child.visit_stmt_within_child_scope(statement.body(child.ast));
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L957>.
    fn visit_for_in_statement(&mut self, statement: ForInStatement) {
        self.with_child(ScopeOwner::ForInStatement(statement), |child| {
            statement.left(child.ast).visit_with(child);
            statement.right(child.ast).visit_with(child);
            child.visit_stmt_within_child_scope(statement.body(child.ast));
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L966>.
    fn visit_for_of_statement(&mut self, statement: ForOfStatement) {
        self.with_child(ScopeOwner::ForOfStatement(statement), |child| {
            statement.left(child.ast).visit_with(child);
            statement.right(child.ast).visit_with(child);
            child.visit_stmt_within_child_scope(statement.body(child.ast));
        });
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L136>.
    fn visit_do_while_statement(&mut self, statement: DoWhileStatement) {
        // Preserve the legacy AST traversal order. Scope allocation is
        // observable through the resolver's scope ids even though the body is
        // evaluated before the test at runtime.
        statement.test(self.ast).visit_with(self);
        statement.body(self.ast).visit_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1316>.
    fn visit_switch_statement(&mut self, statement: SwitchStatement) {
        statement.discriminant(self.ast).visit_with(self);
        self.with_child(ScopeOwner::SwitchStatement(statement), |child| {
            statement.cases(child.ast).visit_with(child);
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1310>.
    fn visit_switch_case(&mut self, case: SwitchCase) {
        case.consequent(self.ast).visit_with(self);
        case.test(self.ast).visit_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1741>.
    fn visit_variable_declaration(&mut self, declaration: VariableDeclaration) {
        let old_kind = self.decl_kind;
        self.decl_kind = declaration.kind(self.ast).into();
        declaration.declarators(self.ast).visit_with(self);
        self.decl_kind = old_kind;
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1108>.
    fn visit_import_declaration(&mut self, declaration: ImportDeclaration) {
        declaration.specifiers(self.ast).visit_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1118>.
    fn visit_import_specifier(&mut self, specifier: ImportSpecifier) {
        self.modify(specifier.local(self.ast), DeclKind::Lexical);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2016>.
    fn visit_import_default_specifier(&mut self, specifier: ImportDefaultSpecifier) {
        self.modify(specifier.local(self.ast), DeclKind::Lexical);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2042>.
    fn visit_import_namespace_specifier(&mut self, specifier: ImportNamespaceSpecifier) {
        self.modify(specifier.local(self.ast), DeclKind::Lexical);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1199>.
    fn visit_export_named_declaration(&mut self, declaration: ExportNamedDeclaration) {
        if declaration.source(self.ast).is_some() {
            return;
        }
        declaration.declaration(self.ast).visit_with(self);
        declaration.specifiers(self.ast).visit_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L862>.
    fn visit_export_default_declaration(&mut self, declaration: ExportDefaultDeclaration) {
        match self
            .ast
            .export_default_declaration_kind_data(declaration.declaration(self.ast))
        {
            ExportDefaultDeclarationKindData::Expr(expression) => {
                expression.visit_with(self);
                if let Some(id) = expression.as_identifier_reference(self.ast) {
                    let symbol = self.resolve_export(id.name(self.ast));
                    self.nodes[id.node_id()] = NodeContext::Symbol(symbol);
                }
            }
            ExportDefaultDeclarationKindData::Function(function) => {
                if function.id(self.ast).is_some() {
                    let body = function.body(self.ast);
                    self.with_child(ScopeOwner::FunctionBody(body), |child| {
                        child.visit_function_contents(function);
                    });
                } else {
                    function.visit_with(self);
                }
            }
            ExportDefaultDeclarationKindData::Class(class) => {
                let old_strict_mode = self.strict_mode;
                self.strict_mode = true;
                class.decorators(self.ast).visit_with(self);
                class.super_class(self.ast).visit_with(self);
                class.body(self.ast).visit_with(self);
                self.strict_mode = old_strict_mode;
            }
            _ => {}
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L893>.
    fn visit_export_specifier(&mut self, specifier: ExportSpecifier) {
        let local = specifier.local(self.ast);
        if let ModuleExportNameData::IdentifierName(identifier) =
            self.ast.module_export_name_data(local)
        {
            let symbol = self.resolve_export(identifier.name(self.ast));
            self.set_symbol_once(identifier.node_id(), symbol);
        }
        let exported = specifier.exported(self.ast);
        if exported.span(self.ast) != local.span(self.ast)
            && let ModuleExportNameData::IdentifierName(identifier) =
                self.ast.module_export_name_data(exported)
        {
            let symbol = self.resolve_export(identifier.name(self.ast));
            self.set_symbol_once(identifier.node_id(), symbol);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L705>.
    fn visit_break_statement(&mut self, statement: BreakStatement) {
        statement.label(self.ast).visit_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L855>.
    fn visit_continue_statement(&mut self, statement: ContinueStatement) {
        statement.label(self.ast).visit_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1154>.
    fn visit_labeled_statement(&mut self, statement: LabeledStatement) {
        statement.label(self.ast).visit_with(self);
        statement.body(self.ast).visit_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1244>.
    fn visit_ts_enum_declaration(&mut self, declaration: TsEnumDeclaration) {
        self.modify(declaration.id(self.ast), DeclKind::Lexical);
        self.with_child(ScopeOwner::TsEnumDeclaration(declaration), |child| {
            for slot in declaration.body(child.ast).members(child.ast).iter() {
                let member = child.ast.get_node_in_sub_range(slot);
                let name = match child.ast.ts_enum_member_name_data(member.id(child.ast)) {
                    TsEnumMemberNameData::IdentifierName(id) => child.name(id.name(child.ast)),
                    TsEnumMemberNameData::StringLiteral(string) => {
                        let Some(name) = child.ast.get_wtf8(string.value(child.ast)).as_str()
                        else {
                            continue;
                        };
                        name.as_bytes()
                    }
                    TsEnumMemberNameData::TemplateLiteral(template) => {
                        let quasi = child.ast.get_node_in_sub_range(
                            template.quasis(child.ast).iter().next().unwrap(),
                        );
                        let Some(name) = child.ast.get_wtf8(quasi.cooked(child.ast)).as_str()
                        else {
                            continue;
                        };
                        name.as_bytes()
                    }
                };
                child.scopes[child.current].declared_symbols.insert(
                    name,
                    Binding {
                        kind: DeclKind::Lexical,
                        symbol: None,
                    },
                );
            }
            declaration.body(child.ast).visit_with(child);
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1399>.
    fn visit_ts_module_declaration(&mut self, declaration: TsModuleDeclaration) {
        match self.ast.ts_module_name_data(declaration.id(self.ast)) {
            TsModuleNameData::BindingIdentifier(id) => self.modify(id, DeclKind::Lexical),
            TsModuleNameData::TsQualifiedName(name) => self.visit_namespace_root(name),
            TsModuleNameData::StringLiteral(_) => {}
        }
        self.with_child(ScopeOwner::TsModuleDeclaration(declaration), |child| {
            if let Some(name) = declaration.id(child.ast).as_ts_qualified_name(child.ast) {
                child.declare_namespace_segments(name);
            }
            declaration.body(child.ast).visit_with(child);
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1326>.
    fn visit_ts_import_equals_declaration(&mut self, declaration: TsImportEqualsDeclaration) {
        self.modify(declaration.id(self.ast), DeclKind::Lexical);
        declaration.module_reference(self.ast).visit_with(self);
    }

    /// Type-only declarations participate in import/export classification but
    /// their annotations do not introduce runtime references.
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1481>.
    fn visit_ts_type_alias_declaration(&mut self, declaration: TsTypeAliasDeclaration) {
        self.declare_type(declaration.id(self.ast));
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1340>.
    fn visit_ts_interface_declaration(&mut self, declaration: TsInterfaceDeclaration) {
        self.declare_type(declaration.id(self.ast));
    }

    /// Body-less functions do not leak their erased parameter bindings.
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L840>.
    fn visit_ts_function(&mut self, function: TsFunction) {
        if let Some(id) = function.id(self.ast) {
            self.modify(id, DeclKind::Function);
        }
    }

    // Type annotations were deliberately disabled in the legacy resolver.
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L618>.
    fn visit_ts_type(&mut self, _: TsType) {}
}

struct Hoister<'resolver, 'ast> {
    resolver: &'resolver mut Resolver<'ast>,
    kind: DeclKind,
    in_block: bool,
    in_catch_body: bool,
    excluded_from_catch: HashSet<&'ast [u8]>,
    catch_param_decls: HashSet<&'ast [u8]>,
}

impl<'resolver, 'ast> Hoister<'resolver, 'ast> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1787>.
    fn add_binding(&mut self, identifier: BindingIdentifier) {
        let name = self.resolver.name(identifier.name(self.resolver.ast));
        if self.in_catch_body {
            if self.resolver.lookup_value(name, true).is_some()
                && self.catch_param_decls.contains(name)
            {
                return;
            }
            self.excluded_from_catch.insert(name);
        } else if self.catch_param_decls.contains(name) && !self.excluded_from_catch.contains(name)
        {
            return;
        }
        self.resolver.modify(identifier, self.kind);
    }
}

impl<'ast> Visit<'ast> for Hoister<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.resolver.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2058>.
    fn visit_binding_pattern(&mut self, pattern: BindingPattern) {
        if let BindingPatternData::BindingIdentifier(identifier) =
            self.resolver.ast.binding_pattern_data(pattern)
        {
            self.add_binding(identifier);
        } else {
            pattern.visit_children_with(self);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2210>.
    fn visit_stmts(&mut self, statements: SubRange<Stmt>) {
        for id in statements.iter() {
            let statement = self.resolver.ast.get_node_in_sub_range(id);
            if let StmtData::Declaration(declaration) = self.resolver.ast.stmt_data(statement)
                && matches!(
                    self.resolver.ast.decl_data(declaration),
                    DeclData::VariableDeclaration(_) | DeclData::Function(_)
                )
            {
                statement.visit_with(self);
            }
        }
        for id in statements.iter() {
            let statement = self.resolver.ast.get_node_in_sub_range(id);
            if !matches!(
                self.resolver.ast.stmt_data(statement),
                StmtData::Declaration(declaration)
                    if matches!(
                        self.resolver.ast.decl_data(declaration),
                        DeclData::VariableDeclaration(_) | DeclData::Function(_)
                    )
            ) {
                statement.visit_with(self);
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1891>.
    fn visit_ts_module_declaration(&mut self, declaration: TsModuleDeclaration) {
        if !self.in_block {
            match self
                .resolver
                .ast
                .ts_module_name_data(declaration.id(self.resolver.ast))
            {
                TsModuleNameData::BindingIdentifier(id) => {
                    self.resolver.modify(id, DeclKind::Lexical)
                }
                TsModuleNameData::TsQualifiedName(name) => self.resolver.visit_namespace_root(name),
                TsModuleNameData::StringLiteral(_) => {}
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1891>.
    fn visit_ts_enum_declaration(&mut self, declaration: TsEnumDeclaration) {
        if !self.in_block {
            self.resolver
                .modify(declaration.id(self.resolver.ast), DeclKind::Lexical);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1891>.
    fn visit_ts_import_equals_declaration(&mut self, declaration: TsImportEqualsDeclaration) {
        if !self.in_block {
            self.resolver
                .modify(declaration.id(self.resolver.ast), DeclKind::Lexical);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1713>.
    fn visit_ts_type_alias_declaration(&mut self, declaration: TsTypeAliasDeclaration) {
        self.resolver
            .declare_type(declaration.id(self.resolver.ast));
    }
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1713>.
    fn visit_ts_interface_declaration(&mut self, declaration: TsInterfaceDeclaration) {
        if !self.in_block {
            self.resolver
                .declare_type(declaration.id(self.resolver.ast));
        }
    }
    fn visit_ts_function(&mut self, function: TsFunction) {
        if !self.in_block
            && let Some(id) = function.id(self.resolver.ast)
        {
            self.resolver.modify(id, DeclKind::Function);
        }
    }
    fn visit_ts_global_declaration(&mut self, _: TsGlobalDeclaration) {}

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1985>.
    fn visit_expr(&mut self, _: Expr) {}

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1814>.
    fn visit_arrow_function_expression(&mut self, _: ArrowFunctionExpression) {}

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1987>.
    fn visit_function(&mut self, function: Function) {
        if function.function_type(self.resolver.ast) != FunctionType::FunctionDeclaration {
            return;
        }
        let Some(identifier) = function.id(self.resolver.ast) else {
            return;
        };
        let name = self.resolver.name(identifier.name(self.resolver.ast));
        if self.catch_param_decls.contains(name) {
            return;
        }
        if self.in_block {
            if self.resolver.strict_mode {
                return;
            }
            if matches!(
                self.resolver.is_declared(name, self.resolver.current),
                Some(DeclKind::Lexical | DeclKind::Param)
            ) {
                return;
            }
        }
        self.resolver.modify(identifier, DeclKind::Function);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1897>.
    fn visit_class(&mut self, class: Class) {
        if class.class_type(self.resolver.ast) != ClassType::ClassDeclaration || self.in_block {
            return;
        }
        if let Some(identifier) = class.id(self.resolver.ast) {
            self.resolver.modify(identifier, DeclKind::Lexical);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1960>.
    fn visit_export_default_declaration(&mut self, declaration: ExportDefaultDeclaration) {
        match self
            .resolver
            .ast
            .export_default_declaration_kind_data(declaration.declaration(self.resolver.ast))
        {
            ExportDefaultDeclarationKindData::Function(function) => {
                if let Some(identifier) = function.id(self.resolver.ast) {
                    self.resolver.modify(identifier, DeclKind::Var);
                }
                function.visit_with(self);
            }
            ExportDefaultDeclarationKindData::Class(class) => {
                if let Some(identifier) = class.id(self.resolver.ast) {
                    self.resolver.modify(identifier, DeclKind::Lexical);
                }
                class.visit_with(self);
            }
            _ => {}
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1822>.
    fn visit_block_statement(&mut self, block: BlockStatement) {
        let old_in_block = self.in_block;
        self.in_block = true;
        block.body(self.resolver.ast).visit_with(self);
        self.in_block = old_in_block;
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1863>.
    fn visit_catch_clause(&mut self, catch: CatchClause) {
        let old_excluded = self.excluded_from_catch.clone();
        self.excluded_from_catch.clear();
        let old_in_catch_body = self.in_catch_body;

        let original_params = self.catch_param_decls.clone();
        if let Some(param) = catch.param(self.resolver.ast) {
            let ast = self.resolver.ast;
            for_each_binding_ident(ast, param, |identifier| {
                let name = ast.get_utf8(identifier.name(ast)).as_bytes();
                self.catch_param_decls.insert(name);
            });
        }

        self.in_catch_body = true;
        catch.body(self.resolver.ast).visit_with(self);
        self.in_catch_body = false;

        catch.param(self.resolver.ast).visit_with(self);

        self.catch_param_decls = original_params;
        self.in_catch_body = old_in_catch_body;
        self.excluded_from_catch = old_excluded;
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2101>.
    fn visit_variable_declaration(&mut self, declaration: VariableDeclaration) {
        if self.in_block && declaration.kind(self.resolver.ast) != VariableKind::Var {
            return;
        }
        let old_kind = self.kind;
        self.kind = declaration.kind(self.resolver.ast).into();
        for id in declaration.declarators(self.resolver.ast).iter() {
            let declarator = self.resolver.ast.get_node_in_sub_range(id);
            declarator.id(self.resolver.ast).visit_with(self);
        }
        self.kind = old_kind;
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2029>.
    fn visit_import_specifier(&mut self, specifier: ImportSpecifier) {
        self.resolver
            .modify(specifier.local(self.resolver.ast), DeclKind::Lexical);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2016>.
    fn visit_import_default_specifier(&mut self, specifier: ImportDefaultSpecifier) {
        self.resolver
            .modify(specifier.local(self.resolver.ast), DeclKind::Lexical);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2042>.
    fn visit_import_namespace_specifier(&mut self, specifier: ImportNamespaceSpecifier) {
        self.resolver
            .modify(specifier.local(self.resolver.ast), DeclKind::Lexical);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2071>.
    fn visit_switch_statement(&mut self, statement: SwitchStatement) {
        let old_in_block = self.in_block;
        self.in_block = true;
        statement.cases(self.resolver.ast).visit_with(self);
        self.in_block = old_in_block;
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2121>.
    fn visit_for_statement_init(&mut self, init: ForStatementInit) {
        if let ForStatementInitData::VariableDeclaration(declaration) =
            self.resolver.ast.for_statement_init_data(init)
            && declaration.kind(self.resolver.ast) == VariableKind::Var
        {
            declaration.visit_with(self);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2137>.
    fn visit_for_statement_left(&mut self, left: ForStatementLeft) {
        if let ForStatementLeftData::VariableDeclaration(declaration) =
            self.resolver.ast.for_statement_left_data(left)
            && declaration.kind(self.resolver.ast) == VariableKind::Var
        {
            declaration.visit_with(self);
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L136>.
    fn visit_static_block(&mut self, _: StaticBlock) {}

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L2069>.
    fn visit_assignment_target(&mut self, _: AssignmentTarget) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ScopeKind {
    Block,
    #[default]
    Fn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclKind {
    Lexical,
    Param,
    Var,
    Function,
}

impl From<VariableKind> for DeclKind {
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L1741>.
    fn from(kind: VariableKind) -> Self {
        match kind {
            VariableKind::Var => Self::Var,
            VariableKind::Let
            | VariableKind::Const
            | VariableKind::Using
            | VariableKind::AwaitUsing => Self::Lexical,
        }
    }
}
