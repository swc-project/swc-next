use super::build::runtime_expression;
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, AstUtf8, CallExpression, Class, DUMMY_SP, Expr, ExprData,
    Function, GetSpan, IdentifierReference, NodeId, NodeKindData, PropertyKey, ScopeId, SymbolId,
    Visit, VisitWith,
};

use swc_next_ecma_semantic::{Semantic, scope::ScopeKind};

pub(super) struct HookSignature {
    pub(super) key: String,
    pub(super) force_reset: bool,
    pub(super) custom_hooks: Vec<CustomHook>,
}

pub(super) enum CustomHook {
    Identifier(IdentifierReference),
    Member {
        root: IdentifierReference,
        property: AstUtf8,
    },
    NestedMember {
        root: IdentifierReference,
        object_property: AstUtf8,
        property: AstUtf8,
    },
}

impl CustomHook {
    fn root(&self) -> IdentifierReference {
        match *self {
            Self::Identifier(root)
            | Self::Member { root, .. }
            | Self::NestedMember { root, .. } => root,
        }
    }

    pub(super) fn create_expression(&self, ast: &mut Ast<'_>) -> Expr {
        let name = self.root().name(ast);
        let mut expression = ast.expr_identifier_reference(DUMMY_SP, name);
        let property = match *self {
            Self::Identifier(_) => return expression,
            Self::Member { property, .. } => property,
            Self::NestedMember {
                object_property,
                property,
                ..
            } => {
                expression = static_member_expression(ast, expression, object_property);
                property
            }
        };
        static_member_expression(ast, expression, property)
    }
}

fn static_member_expression(ast: &mut Ast<'_>, object: Expr, name: AstUtf8) -> Expr {
    let property = ast.identifier_name(DUMMY_SP, name);
    ast.expr_member_expression(
        DUMMY_SP,
        object,
        PropertyKey::IdentifierName(property),
        false,
        false,
    )
}

pub(super) struct HookCollector<'ast_ref, 'ast> {
    ast: &'ast_ref Ast<'ast>,
    semantic: &'ast_ref Semantic<'ast>,
    boundary: NodeId,
    path: Vec<NodeId>,
    key: String,
    force_reset: bool,
    custom_hooks: Vec<CustomHook>,
}

impl<'ast_ref, 'ast> HookCollector<'ast_ref, 'ast> {
    pub(super) fn into_signature(self) -> Option<HookSignature> {
        (!self.key.is_empty()).then_some(HookSignature {
            key: self.key,
            force_reset: self.force_reset,
            custom_hooks: self.custom_hooks,
        })
    }

    pub(super) fn new(
        ast: &'ast_ref Ast<'ast>,
        semantic: &'ast_ref Semantic<'ast>,
        boundary: NodeId,
    ) -> Self {
        Self {
            ast,
            semantic,
            boundary,
            path: Vec::with_capacity(32),
            key: String::new(),
            force_reset: false,
            custom_hooks: Vec::new(),
        }
    }

    fn enclosing_function_scope(&self, mut scope: ScopeId) -> Option<ScopeId> {
        loop {
            let current = self.semantic.scope(scope);
            if current.kind == ScopeKind::Function {
                return Some(scope);
            }
            scope = current.parent?;
        }
    }

    fn symbol_is_local(&self, symbol: SymbolId, reference_scope: ScopeId) -> bool {
        let function_scope = self.enclosing_function_scope(reference_scope);
        for declaration in self.semantic.declarations(symbol) {
            if declaration.node == self.boundary {
                return true;
            }
            let Some(function_scope) = function_scope else {
                continue;
            };
            let mut scope = Some(declaration.scope);
            while let Some(current) = scope {
                if current == function_scope {
                    return true;
                }
                scope = self.semantic.scope(current).parent;
            }
        }
        false
    }

    fn collect_call(&mut self, call: CallExpression, parent: Option<NodeId>) {
        let Some((hook_name, custom_hook)) = hook_callee(call.callee(self.ast), self.ast) else {
            return;
        };
        if !is_builtin_hook(hook_name)
            && let Some(custom_hook) = custom_hook
        {
            let root = custom_hook.root();
            let reference = self
                .semantic
                .reference_of(root.node_id())
                .map(|reference| self.semantic.reference(reference));
            let binding = reference
                .and_then(|reference| reference.symbol.map(|symbol| (symbol, reference.scope)));
            match binding {
                Some((symbol, scope)) if !self.symbol_is_local(symbol, scope) => {
                    self.custom_hooks.push(custom_hook);
                }
                // The signature callback lives outside the component's scope.
                // Hooks defined inside it cannot be inspected during refresh.
                _ => self.force_reset = true,
            }
        }
        if !self.key.is_empty() {
            self.key.push_str("\\n");
        }
        self.key.push_str(hook_name);
        self.key.push('{');
        if let Some(parent) = parent
            && let NodeKindData::VariableDeclarator(declarator) = self.ast.kind_data(parent)
            && declarator
                .init(self.ast)
                .is_some_and(|init| init.node_id() == call.node_id())
        {
            let span = declarator.id(self.ast).span(self.ast);
            self.key.push_str(
                self.ast
                    .get_utf8(self.ast.source_slice(span.start, span.end)),
            );
        }
        let argument_index = if hook_name == "useState" {
            Some(0)
        } else if hook_name == "useReducer" {
            Some(1)
        } else {
            None
        };
        if let Some(argument_index) = argument_index
            && let Some(argument_id) = call.arguments(self.ast).iter().nth(argument_index)
        {
            let argument = self.ast.get_node_in_sub_range(argument_id);
            let span = argument.span(self.ast);
            self.key.push('(');
            self.key.push_str(
                self.ast
                    .get_utf8(self.ast.source_slice(span.start, span.end)),
            );
            self.key.push(')');
        }
        self.key.push('}');
    }
}

impl<'ast> Visit<'ast> for HookCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn enter_node(&mut self, node: NodeKindData) {
        self.path.push(node.node_id());
    }

    fn leave_node(&mut self, node: NodeKindData) {
        let current = self.path.pop();
        debug_assert_eq!(current, Some(node.node_id()));
    }

    fn visit_function(&mut self, _function: Function) {}

    fn visit_arrow_function_expression(&mut self, _arrow: ArrowFunctionExpression) {}

    fn visit_class(&mut self, _class: Class) {}

    fn visit_call_expression(&mut self, call: CallExpression) {
        let parent = self.path.last().copied();
        self.enter_node(NodeKindData::CallExpression(call));
        call.visit_children_with(self);
        self.leave_node(NodeKindData::CallExpression(call));
        self.collect_call(call, parent);
    }
}

fn hook_callee<'ast>(expression: Expr, ast: &Ast<'ast>) -> Option<(&'ast str, Option<CustomHook>)> {
    enum Properties {
        One(AstUtf8),
        Two(AstUtf8, AstUtf8),
        Unsupported,
    }

    let member = match ast.expr_data(runtime_expression(expression, ast)) {
        ExprData::IdentifierReference(identifier) => {
            let name = ast.get_utf8(identifier.name(ast));
            return is_use_hook_name(name)
                .then_some((name, Some(CustomHook::Identifier(identifier))));
        }
        ExprData::MemberExpression(member) if !member.computed(ast) => member,
        _ => return None,
    };
    let property = member.property(ast).as_identifier_name(ast)?.name(ast);
    let hook_name = ast.get_utf8(property);
    if !is_use_hook_name(hook_name) {
        return None;
    }

    let mut properties = Properties::One(property);
    let mut object = member.object(ast);
    loop {
        match ast.expr_data(runtime_expression(object, ast)) {
            ExprData::IdentifierReference(root) => {
                let custom_hook = match properties {
                    Properties::One(property) => Some(CustomHook::Member { root, property }),
                    Properties::Two(object_property, property) => Some(CustomHook::NestedMember {
                        root,
                        object_property,
                        property,
                    }),
                    // Match React Refresh's supported custom-hook member depth.
                    Properties::Unsupported => None,
                };
                return Some((hook_name, custom_hook));
            }
            ExprData::MemberExpression(member) if !member.computed(ast) => {
                let property = member.property(ast).as_identifier_name(ast)?.name(ast);
                properties = match properties {
                    Properties::One(outer) => Properties::Two(property, outer),
                    Properties::Two(_, _) | Properties::Unsupported => Properties::Unsupported,
                };
                object = member.object(ast);
            }
            _ => return None,
        }
    }
}

fn is_use_hook_name(name: &str) -> bool {
    name.starts_with("use") && name.as_bytes().get(3).is_some_and(u8::is_ascii_uppercase)
}

fn is_builtin_hook(name: &str) -> bool {
    matches!(
        name,
        "useState"
            | "useReducer"
            | "useEffect"
            | "useLayoutEffect"
            | "useMemo"
            | "useCallback"
            | "useRef"
            | "useContext"
            | "useImperativeHandle"
            | "useDebugValue"
            | "useId"
            | "useDeferredValue"
            | "useTransition"
            | "useInsertionEffect"
            | "useSyncExternalStore"
            | "useFormStatus"
            | "useFormState"
            | "useActionState"
            | "useOptimistic"
    )
}
