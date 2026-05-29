use rustc_hash::FxHashSet;
use swc_core::{
    atoms::Atom,
    common::{BytePos, GLOBALS, Globals, Mark, Span, comments::SingleThreadedComments},
    ecma::{
        ast::{ClassMember, Ident, Program},
        parser::{
            Lexer, Parser, StringInput, Syntax,
            unstable::{Capturing, Token, TokenAndSpan},
        },
        transforms::base::{fixer::paren_remover, resolver},
        visit::{Visit, VisitMutWith, VisitWith},
    },
};

pub fn run(src: &'static str) {
    GLOBALS.set(&Globals::default(), || {
        let comments = SingleThreadedComments::default();
        let (mut program, tokens) = run_parse(src, &comments);
        run_remove_paren(&mut program, &comments);
        run_resolver(&mut program);
        let _semi = run_collect_semiconlons(&program, &tokens);
        run_scan_dependencies(&program);
    });
}

#[inline(never)]
fn run_parse(src: &str, comments: &SingleThreadedComments) -> (Program, Vec<TokenAndSpan>) {
    let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32));
    let lexer = Capturing::new(Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        input,
        Some(&comments),
    ));
    let mut parser = Parser::new_from(lexer);
    let program = parser.parse_program().unwrap();
    let tokens = parser.input_mut().iter_mut().take();
    (program, tokens)
}

#[inline(never)]
fn run_remove_paren(program: &mut Program, comments: &SingleThreadedComments) {
    program.visit_mut_with(&mut paren_remover(Some(&comments)));
}

#[inline(never)]
fn run_resolver(program: &mut Program) {
    program.visit_mut_with(&mut resolver(Mark::new(), Mark::new(), false));
}

#[inline(never)]
fn run_scan_dependencies(program: &Program) {
    program.visit_with(&mut JavascriptParser { idents: Vec::new() });
}

#[inline(never)]
fn run_collect_semiconlons(program: &Program, tokens: &Vec<TokenAndSpan>) -> FxHashSet<BytePos> {
    let mut semicolons = Default::default();
    program.visit_with(&mut InsertedSemicolons {
        semicolons: &mut semicolons,
        tokens,
    });
    semicolons
}

/// Auto inserted semicolon
/// See: https://262.ecma-international.org/7.0/#sec-rules-of-automatic-semicolon-insertion
struct InsertedSemicolons<'a> {
    semicolons: &'a mut FxHashSet<BytePos>,
    tokens: &'a Vec<TokenAndSpan>,
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

impl Visit for InsertedSemicolons<'_> {
    fn visit_expr_stmt(&mut self, n: &swc_core::ecma::ast::ExprStmt) {
        self.post_semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_var_decl(&mut self, n: &swc_core::ecma::ast::VarDecl) {
        self.post_semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_update_expr(&mut self, n: &swc_core::ecma::ast::UpdateExpr) {
        self.semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_continue_stmt(&mut self, n: &swc_core::ecma::ast::ContinueStmt) {
        self.post_semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_break_stmt(&mut self, n: &swc_core::ecma::ast::BreakStmt) {
        self.post_semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_return_stmt(&mut self, n: &swc_core::ecma::ast::ReturnStmt) {
        self.post_semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_throw_stmt(&mut self, n: &swc_core::ecma::ast::ThrowStmt) {
        self.post_semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_yield_expr(&mut self, n: &swc_core::ecma::ast::YieldExpr) {
        self.post_semi(&n.span);
        if let Some(arg) = &n.arg {
            arg.visit_children_with(self)
        }
    }

    fn visit_import_decl(&mut self, n: &swc_core::ecma::ast::ImportDecl) {
        self.post_semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_named_export(&mut self, n: &swc_core::ecma::ast::NamedExport) {
        self.post_semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_export_default_expr(&mut self, n: &swc_core::ecma::ast::ExportDefaultExpr) {
        self.post_semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_export_all(&mut self, n: &swc_core::ecma::ast::ExportAll) {
        self.post_semi(&n.span);
        n.visit_children_with(self)
    }

    fn visit_debugger_stmt(&mut self, n: &swc_core::ecma::ast::DebuggerStmt) {
        self.post_semi(&n.span);
        n.visit_children_with(self);
    }

    fn visit_class_member(&mut self, n: &swc_core::ecma::ast::ClassMember) {
        match n {
            ClassMember::ClassProp(prop) => self.post_semi(&prop.span),
            ClassMember::PrivateProp(prop) => self.post_semi(&prop.span),
            _ => {}
        };
        n.visit_children_with(self);
    }
}

struct JavascriptParser {
    idents: Vec<Atom>,
}

impl Visit for JavascriptParser {
    fn visit_ident(&mut self, node: &Ident) {
        self.idents.push(node.sym.clone());
    }
}
