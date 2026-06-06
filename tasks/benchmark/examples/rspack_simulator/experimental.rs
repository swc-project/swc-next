use rustc_hash::FxHashSet;
use swc_core::atoms::Atom as LegacyAtom;
use swc_experimental_allocator::{Allocator, atom::Atom};
use swc_experimental_ecma_ast::{
    BreakStmt, ClassMember, ContinueStmt, DebuggerStmt, ExportAll, ExportDefaultExpr, ExprStmt,
    GetSpan, ImportDecl, NamedExport, Program, ReturnStmt, Span, ThrowStmt, UpdateExpr, VarDecl,
    Visit, VisitWith, YieldExpr,
};
use swc_experimental_ecma_ast_compat::{AstCompat, UnsafeArenaAstCompat};
use swc_experimental_ecma_parser::{
    Lexer, Parser, StringSource, Syntax,
    unstable::{Capturing, Token, TokenAndSpan},
};
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};
use swc_experimental_ecma_transforms_base::remove_paren::remove_paren;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatMode {
    Safe,
    Unsafe,
}

pub fn run(src: &'static str, compat: Option<CompatMode>) {
    let allocator = Allocator::new();
    let (mut program, tokens) = run_parse(&allocator, src);
    run_remove_paren(&mut program, &allocator);
    let semantic = run_resolver(&program);
    let _semi = run_collect_semiconlons(&program, &tokens);
    match compat {
        Some(compat) => run_compat_and_scan(program, &semantic, compat),
        None => run_scan_dependencies(&program),
    }
}

#[inline(never)]
fn run_parse<'a>(allocator: &'a Allocator, src: &'a str) -> (Program<'a>, Vec<TokenAndSpan>) {
    let (program, tokens) = {
        let parser_lexer = Lexer::new(
            allocator,
            Syntax::Es(Default::default()),
            Default::default(),
            StringSource::new(src),
            None,
        );

        // Empirically, 1/8 of the source length is a good capacity.
        let lexer = Capturing::with_capacity(parser_lexer, src.len() / 8);
        let mut parser = Parser::new_from(allocator, lexer);

        let program = parser.parse_program().unwrap();
        let tokens = Capturing::take(&mut parser.input_mut().iter);
        (program, tokens)
    };
    (program, tokens)
}

#[inline(never)]
fn run_remove_paren<'a>(root: &mut Program<'a>, allocator: &'a Allocator) {
    remove_paren(root, allocator, None)
}

#[inline(never)]
fn run_resolver(root: &Program<'_>) -> Semantic {
    resolver(root)
}

#[inline(never)]
fn run_scan_dependencies(root: &Program<'_>) {
    root.visit_with(&mut JavascriptParser { idents: Vec::new() });
}

#[inline(never)]
fn run_compat_and_scan(root: Program<'_>, semantic: &Semantic, compat: CompatMode) {
    match compat {
        CompatMode::Safe => {
            let program = AstCompat::new(semantic).compat_program(root);
            run_scan_dependencies_legacy(&program);
        }
        CompatMode::Unsafe => {
            let program = UnsafeArenaAstCompat::new(semantic).compat_program(root);
            program.with_ref(run_scan_dependencies_legacy);
        }
    }
}

#[inline(never)]
fn run_scan_dependencies_legacy(program: &swc_core::ecma::ast::Program) {
    swc_core::ecma::visit::VisitWith::visit_with(
        program,
        &mut JavascriptParserLegacy { idents: Vec::new() },
    );
}

#[inline(never)]
fn run_collect_semiconlons(root: &Program, tokens: &[TokenAndSpan]) -> FxHashSet<u32> {
    let mut semicolons_set = FxHashSet::default();
    let mut semicolons = InsertedSemicolons {
        semicolons: &mut semicolons_set,
        tokens,
    };
    semicolons.visit_program(root);
    semicolons_set
}

struct InsertedSemicolons<'a> {
    semicolons: &'a mut FxHashSet<u32>,
    tokens: &'a [TokenAndSpan],
}

impl InsertedSemicolons<'_> {
    /// Find the starting token of this span.
    /// Returns [None] if there's no token is found.
    /// This might be happen if there's an error in the lexer.
    #[inline]
    fn curr_token(&self, span: &Span) -> Option<usize> {
        self.tokens
            .binary_search_by(|t| t.span.start.cmp(&span.start))
            .ok()
    }

    /// Find the next token of this span.
    /// Returns [None] if there's no token is found.
    /// This might be happen if there's an error in the lexer.
    #[inline]
    fn next_token(&self, span: &Span) -> Option<usize> {
        self.tokens
            .binary_search_by(|t| t.span.end.cmp(&span.end))
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
                self.semicolons.insert(prev.span.end);
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
                self.semicolons.insert(prev.span.end);
            }
        }
    }
}

impl<'ast> Visit<'ast> for InsertedSemicolons<'_> {
    fn visit_expr_stmt(&mut self, n: &ExprStmt<'ast>) {
        self.post_semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_var_decl(&mut self, n: &VarDecl<'ast>) {
        self.post_semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_update_expr(&mut self, n: &UpdateExpr<'ast>) {
        self.semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_continue_stmt(&mut self, n: &ContinueStmt<'ast>) {
        self.post_semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_break_stmt(&mut self, n: &BreakStmt<'ast>) {
        self.post_semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_return_stmt(&mut self, n: &ReturnStmt<'ast>) {
        self.post_semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_throw_stmt(&mut self, n: &ThrowStmt<'ast>) {
        self.post_semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_yield_expr(&mut self, n: &YieldExpr<'ast>) {
        self.post_semi(&n.span());
        if let Some(arg) = &n.arg {
            arg.visit_children_with(self)
        }
    }

    fn visit_import_decl(&mut self, n: &ImportDecl<'ast>) {
        self.post_semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_named_export(&mut self, n: &NamedExport<'ast>) {
        self.post_semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_export_default_expr(&mut self, n: &ExportDefaultExpr<'ast>) {
        self.post_semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_export_all(&mut self, n: &ExportAll<'ast>) {
        self.post_semi(&n.span());
        n.visit_children_with(self)
    }

    fn visit_debugger_stmt(&mut self, n: &DebuggerStmt) {
        self.post_semi(&n.span());
        n.visit_children_with(self);
    }

    fn visit_class_member(&mut self, n: &ClassMember<'ast>) {
        match n {
            ClassMember::ClassProp(prop) => self.post_semi(&prop.span()),
            ClassMember::PrivateProp(prop) => self.post_semi(&prop.span()),
            _ => {}
        };
        n.visit_children_with(self);
    }
}

struct JavascriptParser<'ast> {
    idents: Vec<Atom<'ast>>,
}

impl<'ast> Visit<'ast> for JavascriptParser<'ast> {
    fn visit_ident(&mut self, node: &swc_experimental_ecma_ast::Ident<'ast>) {
        self.idents.push(node.sym);
    }
}

struct JavascriptParserLegacy {
    idents: Vec<LegacyAtom>,
}

impl swc_core::ecma::visit::Visit for JavascriptParserLegacy {
    fn visit_ident(&mut self, node: &swc_core::ecma::ast::Ident) {
        self.idents.push(node.sym.clone());
    }
}
