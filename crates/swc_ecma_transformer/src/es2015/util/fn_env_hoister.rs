use std::collections::HashSet;

use swc_experimental_ecma_ast::{
    AssignmentOperator, Ast, Class, DUMMY_SP, Decl, Expr, Function, FunctionBody, GetSpan,
    IdentifierReference, Program, StatementListItem, StaticBlock, Stmt, String as AstString,
    TypedSubRange, VariableKind, VisitMut, VisitMutWith,
};
use swc_experimental_ecma_semantic::{ReferenceKind, Semantic};

#[derive(Default)]
pub struct CaptureFrame {
    this: Option<AstString>,
    arguments: Option<AstString>,
    new_target: Option<AstString>,
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
pub struct FnEnvHoister<'semantic> {
    semantic: &'semantic Semantic,
    frames: Vec<CaptureFrame>,
    used_names: HashSet<Vec<u8>>,
    in_arrow: bool,
}

impl<'semantic> FnEnvHoister<'semantic> {
    pub fn new(ast: &Ast<'_>, semantic: &'semantic Semantic) -> Self {
        let mut used_names = HashSet::new();
        for symbol_id in semantic.symbol_ids() {
            used_names.insert(ast.string(semantic.symbol(symbol_id).name).to_vec());
        }
        for (_, reference) in semantic.symbol_table().unresolved_references() {
            used_names.insert(ast.string(reference.name).to_vec());
        }

        Self {
            semantic,
            frames: vec![CaptureFrame::default()],
            used_names,
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

    fn fresh_name(&mut self, base: &[u8], ast: &mut Ast<'_>) -> AstString {
        let mut candidate = base.to_vec();
        let mut suffix = 2u32;
        while self.used_names.contains(&candidate) {
            candidate.clear();
            candidate.extend_from_slice(base);
            candidate.extend_from_slice(suffix.to_string().as_bytes());
            suffix += 1;
        }
        self.used_names.insert(candidate.clone());
        ast.add_string(&candidate)
    }

    fn capture_this(&mut self, ast: &mut Ast<'_>) -> AstString {
        if let Some(name) = self.frames.last().and_then(|frame| frame.this) {
            return name;
        }
        let name = self.fresh_name(b"_this", ast);
        self.frames.last_mut().expect("a capture frame exists").this = Some(name);
        name
    }

    fn capture_arguments(&mut self, ast: &mut Ast<'_>) -> AstString {
        if let Some(name) = self.frames.last().and_then(|frame| frame.arguments) {
            return name;
        }
        let name = self.fresh_name(b"_arguments", ast);
        self.frames
            .last_mut()
            .expect("a capture frame exists")
            .arguments = Some(name);
        name
    }

    fn capture_new_target(&mut self, ast: &mut Ast<'_>) -> AstString {
        if let Some(name) = self.frames.last().and_then(|frame| frame.new_target) {
            return name;
        }
        let name = self.fresh_name(b"_newtarget", ast);
        self.frames
            .last_mut()
            .expect("a capture frame exists")
            .new_target = Some(name);
        name
    }

    fn is_unresolved_arguments(&self, node: IdentifierReference, ast: &Ast<'_>) -> bool {
        if ast.string(node.name(ast)) != b"arguments" {
            return false;
        }
        let Some(reference_id) = self.semantic.node_reference(node.node_id()) else {
            return false;
        };
        let reference = self.semantic.reference(reference_id);
        reference.kind == ReferenceKind::Value && reference.symbol_id.is_none()
    }

    fn is_new_target(&self, node: swc_experimental_ecma_ast::MetaProperty, ast: &Ast<'_>) -> bool {
        let meta = node.meta(ast);
        let property = node.property(ast);
        ast.string(meta.name(ast)) == b"new" && ast.string(property.name(ast)) == b"target"
    }

    pub fn visit_this_expression(
        &mut self,
        node: swc_experimental_ecma_ast::ThisExpression,
        ast: &mut Ast<'_>,
    ) {
        if !self.in_arrow {
            return;
        }
        let name = self.capture_this(ast);
        let replacement = ast.identifier_reference(node.span(ast), name, None);
        ast.replace_data(node, replacement);
    }

    pub fn visit_identifier_reference(&mut self, node: IdentifierReference, ast: &mut Ast<'_>) {
        if self.in_arrow && self.is_unresolved_arguments(node, ast) {
            let name = self.capture_arguments(ast);
            node.set_name(ast, name);
            node.set_symbol_id(ast, None);
        }
    }

    pub fn visit_meta_property(
        &mut self,
        node: swc_experimental_ecma_ast::MetaProperty,
        ast: &mut Ast<'_>,
    ) {
        if self.in_arrow && self.is_new_target(node, ast) {
            let name = self.capture_new_target(ast);
            let replacement = ast.identifier_reference(node.span(ast), name, None);
            ast.replace_data(node, replacement);
        }
    }

    fn build_capture_declaration(
        &mut self,
        frame: CaptureFrame,
        derived_constructor: bool,
        ast: &mut Ast<'_>,
    ) -> Option<swc_experimental_ecma_ast::VariableDeclaration> {
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
                None,
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
                None,
            );
            let arguments_name = ast.add_string(b"arguments");
            let init = ast.expr_identifier_reference(DUMMY_SP, arguments_name, None);
            declarators.push(ast.variable_declarator(DUMMY_SP, id, Some(init), false));
        }
        if let Some(name) = frame.new_target {
            let id = ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name,
                TypedSubRange::empty(),
                None,
                false,
                None,
            );
            let new_name = ast.add_string(b"new");
            let target_name = ast.add_string(b"target");
            let meta = ast.identifier_name(DUMMY_SP, new_name);
            let property = ast.identifier_name(DUMMY_SP, target_name);
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
        let statement =
            StatementListItem::Stmt(Stmt::Declaration(Decl::VariableDeclaration(declaration)));
        let body = program.body(ast);
        let body = prepend_statement_list_item(body, statement, ast);
        program.set_body(ast, body);
    }

    pub fn inject_into_function(
        &mut self,
        function: Function,
        frame: CaptureFrame,
        derived_constructor: bool,
        ast: &mut Ast<'_>,
    ) {
        let Some(body) = function.body(ast) else {
            return;
        };

        if derived_constructor && let Some(this_name) = frame.this {
            rewrite_derived_super_calls(body, this_name, ast);
        }

        let Some(declaration) = self.build_capture_declaration(frame, derived_constructor, ast)
        else {
            return;
        };
        let statement =
            StatementListItem::Stmt(Stmt::Declaration(Decl::VariableDeclaration(declaration)));
        let statements = prepend_statement_list_item(body.body(ast), statement, ast);
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
        let old = block.body(ast);
        let mut statements = Vec::with_capacity(old.len() + 1);
        statements.push(statement);
        statements.extend(old.iter().map(|id| ast.get_node_in_sub_range(id)));
        let statements = ast.add_typed_sub_range(statements);
        block.set_body(ast, statements);
    }
}

fn prepend_statement_list_item(
    old: TypedSubRange<StatementListItem>,
    statement: StatementListItem,
    ast: &mut Ast<'_>,
) -> TypedSubRange<StatementListItem> {
    let mut items: Vec<_> = old.iter().map(|id| ast.get_node_in_sub_range(id)).collect();
    let insertion_index = items
        .iter()
        .position(|item| !item.is_directive(ast))
        .unwrap_or(items.len());
    items.insert(insertion_index, statement);
    ast.add_typed_sub_range(items)
}

fn rewrite_derived_super_calls(body: FunctionBody, this_name: AstString, ast: &mut Ast<'_>) {
    struct InitThis<'visit, 'ast> {
        ast: &'visit mut Ast<'ast>,
        this_name: AstString,
    }

    impl<'ast> VisitMut<'ast> for InitThis<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
            self.ast
        }

        fn visit_mut_function(&mut self, _node: Function) {}

        fn visit_mut_class(&mut self, _node: Class) {}

        fn visit_mut_expr(&mut self, node: Expr) {
            node.visit_mut_children_with(self);
            let Some(call) = node.as_call_expression(self.ast) else {
                return;
            };
            let callee = call.callee(self.ast);
            if !callee.is_super(self.ast) {
                return;
            }

            let call_copy = self.ast.expr_call_expression(
                call.span(self.ast),
                callee,
                call.arguments(self.ast),
                call.optional(self.ast),
                call.type_arguments(self.ast),
            );
            let left = self
                .ast
                .assignment_target_simple_assignment_target_identifier_reference(
                    DUMMY_SP,
                    self.this_name,
                    None,
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
            self.ast.replace_data(call, sequence);
        }
    }

    body.visit_mut_with(&mut InitThis { ast, this_name });
}
