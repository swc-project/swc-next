use oxc::{
    ast::ast::{
        BreakStatement, ClassElement, ContinueStatement, DebuggerStatement, ExportAllDeclaration,
        ExportDefaultDeclaration, ExportNamedDeclaration, ExpressionStatement, ImportDeclaration,
        LabelIdentifier, Program, ReturnStatement, ThrowStatement, UpdateExpression,
        VariableDeclaration, YieldExpression,
    },
    ast_visit::{Visit, walk},
    parser::{Kind, ParseOptions, Parser, Token, config::TokensParserConfig},
    semantic::{Semantic, SemanticBuilder},
    span::{GetSpan, Ident, SourceType, Span},
};
use oxc_allocator::Allocator;
use rustc_hash::FxHashSet;

pub fn run(src: &'static str) {
    let allocator = Allocator::default();
    let source_type = SourceType::unambiguous();
    let (program, tokens) = run_parse(src, source_type, &allocator);
    let semantic = run_semantic(&program);
    let semi = run_collect_semicolons(&program, &tokens);
    let idents = run_scan_dependencies(&program);
    drop((semantic, semi, idents));
}

#[inline(never)]
fn run_parse<'a>(
    src: &'a str,
    source_type: SourceType,
    allocator: &'a Allocator,
) -> (Program<'a>, oxc_allocator::Vec<'a, Token>) {
    let parser_return = Parser::new(allocator, src, source_type)
        .with_options(ParseOptions {
            preserve_parens: false,
            ..Default::default()
        })
        .with_config(TokensParserConfig)
        .parse();
    assert!(parser_return.errors.is_empty());
    (parser_return.program, parser_return.tokens)
}

#[inline(never)]
fn run_semantic<'a>(program: &'a Program<'a>) -> Semantic<'a> {
    SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(program)
        .semantic
}

#[inline(never)]
fn run_collect_semicolons(program: &Program, tokens: &[Token]) -> FxHashSet<u32> {
    let mut semicolons = FxHashSet::default();
    InsertedSemicolons {
        semicolons: &mut semicolons,
        tokens,
    }
    .visit_program(program);
    semicolons
}

#[inline(never)]
fn run_scan_dependencies<'a>(program: &Program<'a>) -> Vec<Ident<'a>> {
    let mut parser = JavascriptParser { idents: Vec::new() };
    parser.visit_program(program);
    parser.idents
}

struct InsertedSemicolons<'a> {
    semicolons: &'a mut FxHashSet<u32>,
    tokens: &'a [Token],
}

impl InsertedSemicolons<'_> {
    #[inline]
    fn curr_token(&self, span: Span) -> Option<usize> {
        self.tokens
            .binary_search_by(|t| t.start().cmp(&span.start))
            .ok()
    }

    #[inline]
    fn next_token(&self, span: Span) -> Option<usize> {
        self.tokens
            .binary_search_by(|t| t.end().cmp(&span.end))
            .ok()
            .map(|i| i + 1)
    }

    #[inline]
    fn can_insert_semi(&self, token_index: usize) -> bool {
        if token_index == self.tokens.len() {
            return true;
        }
        let token = &self.tokens[token_index];
        token.kind() == Kind::RCurly || token.is_on_new_line()
    }

    #[inline]
    fn semi(&mut self, span: Span) {
        let Some(index) = self.curr_token(span) else {
            return;
        };
        if index > 0 {
            let prev = &self.tokens[index - 1];
            if prev.kind() != Kind::Semicolon && self.can_insert_semi(index) {
                self.semicolons.insert(prev.end());
            }
        }
    }

    #[inline]
    fn post_semi(&mut self, span: Span) {
        let Some(index) = self.next_token(span) else {
            return;
        };
        if index > 0 {
            let prev = &self.tokens[index - 1];
            if prev.kind() != Kind::Semicolon && self.can_insert_semi(index) {
                self.semicolons.insert(prev.end());
            }
        }
    }
}

impl<'a> Visit<'a> for InsertedSemicolons<'_> {
    fn visit_expression_statement(&mut self, it: &ExpressionStatement<'a>) {
        self.post_semi(it.span());
        walk::walk_expression_statement(self, it);
    }

    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        self.post_semi(it.span());
        walk::walk_variable_declaration(self, it);
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        self.semi(it.span());
        walk::walk_update_expression(self, it);
    }

    fn visit_continue_statement(&mut self, it: &ContinueStatement<'a>) {
        self.post_semi(it.span());
        walk::walk_continue_statement(self, it);
    }

    fn visit_break_statement(&mut self, it: &BreakStatement<'a>) {
        self.post_semi(it.span());
        walk::walk_break_statement(self, it);
    }

    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        self.post_semi(it.span());
        walk::walk_return_statement(self, it);
    }

    fn visit_throw_statement(&mut self, it: &ThrowStatement<'a>) {
        self.post_semi(it.span());
        walk::walk_throw_statement(self, it);
    }

    fn visit_yield_expression(&mut self, it: &YieldExpression<'a>) {
        self.post_semi(it.span());
        walk::walk_yield_expression(self, it);
    }

    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        self.post_semi(it.span());
        walk::walk_import_declaration(self, it);
    }

    fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
        self.post_semi(it.span());
        walk::walk_export_named_declaration(self, it);
    }

    fn visit_export_default_declaration(&mut self, it: &ExportDefaultDeclaration<'a>) {
        self.post_semi(it.span());
        walk::walk_export_default_declaration(self, it);
    }

    fn visit_export_all_declaration(&mut self, it: &ExportAllDeclaration<'a>) {
        self.post_semi(it.span());
        walk::walk_export_all_declaration(self, it);
    }

    fn visit_debugger_statement(&mut self, it: &DebuggerStatement) {
        self.post_semi(it.span());
        walk::walk_debugger_statement(self, it);
    }

    fn visit_class_element(&mut self, it: &ClassElement<'a>) {
        match it {
            ClassElement::PropertyDefinition(prop) => self.post_semi(prop.span()),
            ClassElement::AccessorProperty(prop) => self.post_semi(prop.span()),
            _ => {}
        }
        walk::walk_class_element(self, it);
    }
}

struct JavascriptParser<'a> {
    idents: Vec<Ident<'a>>,
}

impl<'a> Visit<'a> for JavascriptParser<'a> {
    fn visit_identifier_reference(&mut self, it: &oxc::ast::ast::IdentifierReference<'a>) {
        self.idents.push(it.name);
    }

    fn visit_binding_identifier(&mut self, it: &oxc::ast::ast::BindingIdentifier<'a>) {
        self.idents.push(it.name);
    }

    fn visit_label_identifier(&mut self, it: &LabelIdentifier<'a>) {
        self.idents.push(it.name);
    }
}
