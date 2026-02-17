use rustc_hash::FxHashSet;
use swc_core::common::{BytePos, comments::SingleThreadedComments};
use swc_experimental_ecma_ast::{
    Ast, BreakStmt, ClassMember, ContinueStmt, DebuggerStmt, ExportAll, ExportDefaultExpr,
    ExprStmt, GetSpan, ImportDecl, NamedExport, Program, ReturnStmt, Span, ThrowStmt, UpdateExpr,
    VarDecl, Visit, VisitWith, YieldExpr,
};
use swc_experimental_ecma_ast_compat::AstCompat;
use swc_experimental_ecma_parser::{
    Lexer, Parser, StringSource, Syntax,
    unstable::{Capturing, Token, TokenAndSpan},
};
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};
use swc_experimental_ecma_transforms_base::remove_paren::remove_paren;

pub fn run(src: &'static str, compat: bool) {
    let comments = SingleThreadedComments::default();
    let (program, mut ast, tokens) = run_parse(src, &comments);
    run_remove_paren(program, &mut ast, &comments);
    let semantic = run_resolver(program, &ast);
    let _semi = run_collect_semiconlons(&ast, program, &tokens);
    if compat {
        let _program = run_compat(program, &ast, &semantic);
    }
}

#[inline(never)]
fn run_parse(src: &str, comments: &SingleThreadedComments) -> (Program, Ast, Vec<TokenAndSpan>) {
    let parser_lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringSource::new(src),
        Some(comments),
    );

    // Empirically, 1/8 of the source length is a good capacity.
    let lexer = Capturing::with_capacity(parser_lexer, src.len() / 8);
    let parser = Parser::new_from(lexer);

    let mut ret = parser.parse_program().unwrap();
    let tokens = Capturing::take(&mut ret.input);
    (ret.root, ret.ast, tokens)
}

#[inline(never)]
fn run_remove_paren(root: Program, ast: &mut Ast, comments: &SingleThreadedComments) {
    remove_paren(root, ast, Some(comments));
}

#[inline(never)]
fn run_resolver(root: Program, ast: &Ast) -> Semantic {
    resolver(root, ast)
}

#[inline(never)]
fn run_compat(root: Program, ast: &Ast, semantic: &Semantic) -> swc_core::ecma::ast::Program {
    AstCompat::new(ast, semantic).compat_program(root)
}

#[inline(never)]
fn run_collect_semiconlons(
    ast: &Ast,
    root: Program,
    tokens: &[TokenAndSpan],
) -> FxHashSet<BytePos> {
    let mut semicolons_set = FxHashSet::default();
    let mut semicolons = InsertedSemicolons {
        ast,
        semicolons: &mut semicolons_set,
        tokens,
    };
    semicolons.visit_program(root);
    semicolons_set
}

struct InsertedSemicolons<'a> {
    ast: &'a Ast,
    semicolons: &'a mut FxHashSet<BytePos>,
    tokens: &'a [TokenAndSpan],
}

impl InsertedSemicolons<'_> {
    /// Find the starting token of this span.
    /// Returns [None] if there's no token is found.
    /// This might be happen if there's an error in the lexer.
    #[inline]
    fn curr_token(&self, span: &Span) -> Option<usize> {
        self.tokens
            .binary_search_by(|t| t.span.lo.cmp(&span.lo))
            .ok()
    }

    /// Find the next token of this span.
    /// Returns [None] if there's no token is found.
    /// This might be happen if there's an error in the lexer.
    #[inline]
    fn next_token(&self, span: &Span) -> Option<usize> {
        self.tokens
            .binary_search_by(|t| t.span.hi.cmp(&span.hi))
            .ok()
            .map(|i| i + 1)
    }

    #[inline]
    fn can_insert_semi(&self, token_index: usize) -> bool {
        if token_index == self.tokens.len() {
            // eof
            return true;
        }
        let token = &self.tokens[token_index];
        matches!(token.token, Token::RBrace) || token.had_line_break
    }

    #[inline]
    fn semi(&mut self, span: &Span) {
        let Some(index) = self.curr_token(span) else {
            return;
        };
        if index > 0 {
            let prev = &self.tokens[index - 1];
            if !matches!(prev.token, Token::Semi) && self.can_insert_semi(index) {
                self.semicolons.insert(prev.span.hi);
            }
        }
    }

    #[inline]
    fn post_semi(&mut self, span: &Span) {
        let Some(index) = self.next_token(span) else {
            return;
        };
        if index > 0 {
            let prev = &self.tokens[index - 1];
            if !matches!(prev.token, Token::Semi) && self.can_insert_semi(index) {
                self.semicolons.insert(prev.span.hi);
            }
        }
    }
}

impl<'ast> Visit for InsertedSemicolons<'ast> {
    fn ast(&self) -> &Ast {
        self.ast
    }

    fn visit_expr_stmt(&mut self, n: ExprStmt) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_var_decl(&mut self, n: VarDecl) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_update_expr(&mut self, n: UpdateExpr) {
        self.semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_continue_stmt(&mut self, n: ContinueStmt) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_break_stmt(&mut self, n: BreakStmt) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_return_stmt(&mut self, n: ReturnStmt) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_throw_stmt(&mut self, n: ThrowStmt) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_yield_expr(&mut self, n: YieldExpr) {
        self.post_semi(&n.span(self.ast));
        if let Some(arg) = &n.arg(self.ast) {
            arg.visit_children_with(self)
        }
    }

    fn visit_import_decl(&mut self, n: ImportDecl) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_named_export(&mut self, n: NamedExport) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_export_default_expr(&mut self, n: ExportDefaultExpr) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_export_all(&mut self, n: ExportAll) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self)
    }

    fn visit_debugger_stmt(&mut self, n: DebuggerStmt) {
        self.post_semi(&n.span(self.ast));
        n.visit_children_with(self);
    }

    fn visit_class_member(&mut self, n: ClassMember) {
        match n {
            ClassMember::ClassProp(prop) => self.post_semi(&prop.span(self.ast)),
            ClassMember::PrivateProp(prop) => self.post_semi(&prop.span(self.ast)),
            _ => {}
        };
        n.visit_children_with(self);
    }
}
