use rustc_hash::FxHashSet;
use swc_core::common::{BytePos, comments::SingleThreadedComments};
use swc_experimental_ecma_ast::{Ast, NodeKind, Program, Span};
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
    let _semi = run_collect_semiconlons(&ast, &tokens);
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
fn run_collect_semiconlons(ast: &Ast, tokens: &[TokenAndSpan]) -> FxHashSet<BytePos> {
    let mut semicolons_set = FxHashSet::default();
    let mut semicolons = InsertedSemicolons {
        semicolons: &mut semicolons_set,
        tokens,
    };
    for (_, node) in ast.nodes() {
        match node.kind() {
            NodeKind::ExprStmt
            | NodeKind::VarDecl
            | NodeKind::ContinueStmt
            | NodeKind::BreakStmt
            | NodeKind::ReturnStmt
            | NodeKind::ThrowStmt
            | NodeKind::YieldExpr
            | NodeKind::ImportDecl
            | NodeKind::NamedExport
            | NodeKind::ExportDefaultExpr
            | NodeKind::ExportAll
            | NodeKind::DebuggerStmt
            | NodeKind::ClassProp
            | NodeKind::PrivateProp => semicolons.post_semi(&node.span()),
            NodeKind::UpdateExpr => semicolons.semi(&node.span()),
            _ => {}
        }
    }

    semicolons_set
}

struct InsertedSemicolons<'a> {
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
