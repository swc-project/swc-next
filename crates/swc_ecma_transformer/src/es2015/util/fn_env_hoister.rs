use swc_next_ecma_ast::{
    AssignmentOperator, Ast, AstUtf8, Class, DUMMY_SP, Decl, Expr, Function, FunctionBody, GetSpan,
    IdentifierReference, Program, StaticBlock, Stmt, TypedSubRange, VariableKind, VisitMut,
    VisitMutWith,
};
use swc_next_ecma_semantic::{ReferenceSpace, Semantic, UsedNameState};

#[derive(Default)]
pub struct CaptureFrame {
    this: Option<AstUtf8>,
    arguments: Option<AstUtf8>,
    new_target: Option<AstUtf8>,
}

impl CaptureFrame {
    fn is_empty(&self) -> bool {
        self.this.is_none() && self.arguments.is_none() && self.new_target.is_none()
    }
}

/// Hoists the lexical function environment used by converted arrow functions.
///
/// This is the flat-AST equivalent of `swc_ecma_utils::function::FnEnvHoister`.
/// It owns capture state, while callers retain ownership of AST traversal.
pub struct FnEnvHoister<'semantic, 'semantic_data, 'ast> {
    semantic: &'semantic Semantic<'semantic_data>,
    frames: Vec<CaptureFrame>,
    used_names: UsedNameState<'ast>,
    in_arrow: bool,
}

impl<'semantic, 'semantic_data, 'ast> FnEnvHoister<'semantic, 'semantic_data, 'ast> {
    pub fn new(ast: &mut Ast<'ast>, semantic: &'semantic Semantic<'semantic_data>) -> Self {
        Self {
            semantic,
            frames: vec![CaptureFrame::default()],
            used_names: UsedNameState::new(ast.allocator()),
            in_arrow: false,
        }
    }

    pub fn set_in_arrow(&mut self, in_arrow: bool) -> bool {
        std::mem::replace(&mut self.in_arrow, in_arrow)
    }

    pub fn push_frame(&mut self) {
        self.frames.push(CaptureFrame::default());
    }

    pub fn take_frame(&mut self) -> CaptureFrame {
        self.frames
            .pop()
            .expect("a function environment frame exists")
    }

    fn fresh_name(&mut self, base: &str, ast: &mut Ast<'_>) -> AstUtf8 {
        self.used_names.next_name(ast, base, 2).into()
    }

    fn capture_this(&mut self, ast: &mut Ast<'_>) -> AstUtf8 {
        if let Some(name) = self.frames.last().and_then(|frame| frame.this) {
            return name;
        }
        let name = self.fresh_name("_this", ast);
        self.frames.last_mut().expect("a capture frame exists").this = Some(name);
        name
    }

    fn capture_arguments(&mut self, ast: &mut Ast<'_>) -> AstUtf8 {
        if let Some(name) = self.frames.last().and_then(|frame| frame.arguments) {
            return name;
        }
        let name = self.fresh_name("_arguments", ast);
        self.frames
            .last_mut()
            .expect("a capture frame exists")
            .arguments = Some(name);
        name
    }

    fn capture_new_target(&mut self, ast: &mut Ast<'_>) -> AstUtf8 {
        if let Some(name) = self.frames.last().and_then(|frame| frame.new_target) {
            return name;
        }
        let name = self.fresh_name("_newtarget", ast);
        self.frames
            .last_mut()
            .expect("a capture frame exists")
            .new_target = Some(name);
        name
    }

    fn is_unresolved_arguments(&self, node: IdentifierReference, ast: &Ast<'_>) -> bool {
        if !ast.utf8_eq_bytes(node.name(ast), b"arguments") {
            return false;
        }
        let Some(reference_id) = self.semantic.reference_of(node.node_id()) else {
            return false;
        };
        let reference = self.semantic.reference(reference_id);
        reference.flags.space == ReferenceSpace::Value && reference.symbol.is_none()
    }

    fn is_new_target(&self, node: swc_next_ecma_ast::MetaProperty, ast: &Ast<'_>) -> bool {
        let meta = node.meta(ast);
        let property = node.property(ast);
        ast.utf8_eq_bytes(meta.name(ast), b"new")
            && ast.utf8_eq_bytes(property.name(ast), b"target")
    }

    pub fn transform_this_expression(
        &mut self,
        node: swc_next_ecma_ast::ThisExpression,
        ast: &mut Ast<'_>,
    ) -> Expr {
        if !self.in_arrow {
            return Expr::ThisExpression(node);
        }
        let name = self.capture_this(ast);
        let replacement = ast.identifier_reference(node.span(ast), name);
        Expr::IdentifierReference(replacement)
    }

    pub fn visit_identifier_reference(&mut self, node: IdentifierReference, ast: &mut Ast<'_>) {
        if self.in_arrow && self.is_unresolved_arguments(node, ast) {
            let name = self.capture_arguments(ast);
            node.set_name(ast, name);
        }
    }

    pub fn transform_meta_property(
        &mut self,
        node: swc_next_ecma_ast::MetaProperty,
        ast: &mut Ast<'_>,
    ) -> Expr {
        if self.in_arrow && self.is_new_target(node, ast) {
            let name = self.capture_new_target(ast);
            let replacement = ast.identifier_reference(node.span(ast), name);
            Expr::IdentifierReference(replacement)
        } else {
            Expr::MetaProperty(node)
        }
    }

    fn build_capture_declaration(
        &mut self,
        frame: CaptureFrame,
        derived_constructor: bool,
        ast: &mut Ast<'_>,
    ) -> Option<swc_next_ecma_ast::VariableDeclaration> {
        if frame.is_empty() {
            return None;
        }

        let mut declarators = Vec::with_capacity(3);
        if let Some(name) = frame.this {
            let id = ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name,
                TypedSubRange::empty(),
                None,
                false,
            );
            let init = (!derived_constructor).then(|| ast.expr_this_expression(DUMMY_SP));
            declarators.push(ast.variable_declarator(DUMMY_SP, id, init, false));
        }
        if let Some(name) = frame.arguments {
            let id = ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name,
                TypedSubRange::empty(),
                None,
                false,
            );
            let arguments_name = ast.add_utf8("arguments");
            let init = ast.expr_identifier_reference(DUMMY_SP, arguments_name.into());
            declarators.push(ast.variable_declarator(DUMMY_SP, id, Some(init), false));
        }
        if let Some(name) = frame.new_target {
            let id = ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name,
                TypedSubRange::empty(),
                None,
                false,
            );
            let new_name = ast.add_utf8("new");
            let target_name = ast.add_utf8("target");
            let meta = ast.identifier_name(DUMMY_SP, new_name.into());
            let property = ast.identifier_name(DUMMY_SP, target_name.into());
            let init = ast.expr_meta_property(DUMMY_SP, meta, property);
            declarators.push(ast.variable_declarator(DUMMY_SP, id, Some(init), false));
        }

        let declarators = ast.add_typed_sub_range(declarators);
        Some(ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false))
    }

    pub fn inject_into_program(
        &mut self,
        program: Program,
        frame: CaptureFrame,
        ast: &mut Ast<'_>,
    ) {
        let Some(declaration) = self.build_capture_declaration(frame, false, ast) else {
            return;
        };
        let statement = Stmt::Declaration(Decl::VariableDeclaration(declaration));
        let body = prepend_statement(program.body(ast), statement, ast);
        program.set_body(ast, body);
    }

    pub fn inject_into_function(
        &mut self,
        function: Function,
        frame: CaptureFrame,
        derived_constructor: bool,
        ast: &mut Ast<'_>,
    ) {
        let body = function.body(ast);

        if derived_constructor && let Some(this_name) = frame.this {
            rewrite_derived_super_calls(body, this_name, ast);
        }

        let Some(declaration) = self.build_capture_declaration(frame, derived_constructor, ast)
        else {
            return;
        };
        let statement = Stmt::Declaration(Decl::VariableDeclaration(declaration));
        let statements = prepend_statement(body.body(ast), statement, ast);
        body.set_body(ast, statements);
    }

    pub fn inject_into_static_block(
        &mut self,
        block: StaticBlock,
        frame: CaptureFrame,
        ast: &mut Ast<'_>,
    ) {
        let Some(declaration) = self.build_capture_declaration(frame, false, ast) else {
            return;
        };
        let statement = Stmt::Declaration(Decl::VariableDeclaration(declaration));
        let statements = prepend_statement(block.body(ast), statement, ast);
        block.set_body(ast, statements);
    }
}

fn prepend_statement(
    old: TypedSubRange<Stmt>,
    statement: Stmt,
    ast: &mut Ast<'_>,
) -> TypedSubRange<Stmt> {
    let mut items = Vec::with_capacity(old.len() + 1);
    items.push(statement);
    items.extend(old.iter().map(|id| ast.get_node_in_sub_range(id)));
    ast.add_typed_sub_range(items)
}

fn rewrite_derived_super_calls(body: FunctionBody, this_name: AstUtf8, ast: &mut Ast<'_>) {
    struct InitThis<'visit, 'ast> {
        ast: &'visit mut Ast<'ast>,
        this_name: AstUtf8,
    }

    impl<'ast> VisitMut<'ast> for InitThis<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
            self.ast
        }

        fn visit_mut_function(&mut self, _node: Function) -> Option<Function> {
            None
        }

        fn visit_mut_class(&mut self, _node: Class) -> Option<Class> {
            None
        }

        fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
            let replacement = node.visit_mut_children_with(self);
            let node = replacement.unwrap_or(node);
            let Some(call) = node.as_call_expression(self.ast) else {
                return replacement;
            };
            let callee = call.callee(self.ast);
            if !callee.is_super(self.ast) {
                return replacement;
            }

            let call_copy = self.ast.expr_call_expression(
                call.span(self.ast),
                callee,
                call.type_arguments(self.ast),
                call.arguments(self.ast),
                call.optional(self.ast),
            );
            let left = self
                .ast
                .assignment_target_simple_assignment_target_identifier_reference(
                    DUMMY_SP,
                    self.this_name,
                );
            let this = self.ast.expr_this_expression(DUMMY_SP);
            let assignment = self.ast.expr_assignment_expression(
                DUMMY_SP,
                left,
                this,
                AssignmentOperator::Assign,
            );
            let expressions = self.ast.add_typed_sub_range([call_copy, assignment]);
            let sequence = self
                .ast
                .sequence_expression(call.span(self.ast), expressions);
            Some(Expr::SequenceExpression(sequence))
        }
    }

    body.visit_mut_with(&mut InitThis { ast, this_name });
}
