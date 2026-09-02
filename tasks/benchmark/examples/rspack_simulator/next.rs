use rustc_hash::FxHashSet;
use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{
    Ast, AstUtf8, BindingIdentifier, BreakStatement, ClassElement, ClassElementData,
    ContinueStatement, DebuggerStatement, ExportAllDeclaration, ExportDefaultDeclaration,
    ExportNamedDeclaration, ExpressionStatement, GetSpan, IdentifierReference, ImportDeclaration,
    LabelIdentifier, ReturnStatement, Span, ThrowStatement, UpdateExpression, VariableDeclaration,
    Visit, VisitWith, YieldExpression,
};
use swc_next_ecma_parser::{Options, Parser, Token, TokenParserConfig, TokenTag};
use swc_next_ecma_semantic::{AnalyzeOptions, Semantic, analyze};

pub fn run(src: &'static str) {
    let allocator = Allocator::new();
    let (mut ast, tokens) = run_parse(src, &allocator);
    let semantic = run_semantic(&mut ast);
    let semicolons = run_collect_semicolons(&ast, &tokens);
    let idents = run_scan_dependencies(&ast);
    drop((ast, semantic, tokens, semicolons, idents));
}

#[inline(never)]
fn run_parse<'a>(src: &'a str, allocator: &'a Allocator) -> (Ast<'a>, ArenaVec<'a, Token>) {
    let result = Parser::init(
        allocator,
        src,
        Options {
            preserve_parens: false,
            ..Options::default()
        },
        TokenParserConfig,
    )
    .parse();
    assert!(result.diagnostics.is_empty());
    (result.ast, result.tokens)
}

#[inline(never)]
fn run_semantic<'a>(ast: &mut Ast<'a>) -> Semantic<'a> {
    analyze(
        ast,
        AnalyzeOptions {
            check_syntax: true,
            build_module_record: false,
        },
    )
    .semantic
}

#[inline(never)]
fn run_scan_dependencies(ast: &Ast<'_>) -> Vec<AstUtf8> {
    let mut parser = JavascriptParser {
        ast,
        idents: Vec::new(),
    };
    ast.root_program().visit_with(&mut parser);
    parser.idents
}

#[inline(never)]
fn run_collect_semicolons(ast: &Ast<'_>, tokens: &[Token]) -> FxHashSet<u32> {
    let mut semicolons = FxHashSet::default();
    InsertedSemicolons {
        ast,
        semicolons: &mut semicolons,
        tokens,
    }
    .visit_program(ast.root_program());
    semicolons
}

struct InsertedSemicolons<'visit, 'ast, 'tokens> {
    ast: &'visit Ast<'ast>,
    semicolons: &'visit mut FxHashSet<u32>,
    tokens: &'tokens [Token],
}

impl InsertedSemicolons<'_, '_, '_> {
    /// Find the token starting at this span.
    #[inline]
    fn curr_token(&self, span: Span) -> Option<usize> {
        self.tokens
            .binary_search_by(|token| token.span.start.cmp(&span.start))
            .ok()
    }

    /// Find the token immediately after this span.
    #[inline]
    fn next_token(&self, span: Span) -> Option<usize> {
        self.tokens
            .binary_search_by(|token| token.span.end.cmp(&span.end))
            .ok()
            .map(|index| index + 1)
    }

    #[inline]
    fn can_insert_semi(&self, token_index: usize) -> bool {
        if token_index == self.tokens.len() {
            return true;
        }
        let token = self.tokens[token_index];
        token.tag == TokenTag::RightBrace || token.has_line_terminator_before()
    }

    #[inline]
    fn semi(&mut self, span: Span) {
        let Some(index) = self.curr_token(span) else {
            return;
        };
        if index > 0 {
            let previous = self.tokens[index - 1];
            if previous.tag != TokenTag::Semicolon && self.can_insert_semi(index) {
                self.semicolons.insert(previous.span.end);
            }
        }
    }

    #[inline]
    fn post_semi(&mut self, span: Span) {
        let Some(index) = self.next_token(span) else {
            return;
        };
        if index > 0 {
            let previous = self.tokens[index - 1];
            if previous.tag != TokenTag::Semicolon && self.can_insert_semi(index) {
                self.semicolons.insert(previous.span.end);
            }
        }
    }
}

impl<'ast> Visit<'ast> for InsertedSemicolons<'_, 'ast, '_> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_expression_statement(&mut self, node: ExpressionStatement) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_variable_declaration(&mut self, node: VariableDeclaration) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_update_expression(&mut self, node: UpdateExpression) {
        self.semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_continue_statement(&mut self, node: ContinueStatement) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_break_statement(&mut self, node: BreakStatement) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_return_statement(&mut self, node: ReturnStatement) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_throw_statement(&mut self, node: ThrowStatement) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_yield_expression(&mut self, node: YieldExpression) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_import_declaration(&mut self, node: ImportDeclaration) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_export_default_declaration(&mut self, node: ExportDefaultDeclaration) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_export_all_declaration(&mut self, node: ExportAllDeclaration) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_debugger_statement(&mut self, node: DebuggerStatement) {
        self.post_semi(node.span(self.ast));
        node.visit_children_with(self);
    }

    fn visit_class_element(&mut self, node: ClassElement) {
        if let ClassElementData::PropertyDefinition(property) = self.ast.class_element_data(node) {
            self.post_semi(property.span(self.ast));
        }
        node.visit_children_with(self);
    }
}

struct JavascriptParser<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    idents: Vec<AstUtf8>,
}

impl<'ast> Visit<'ast> for JavascriptParser<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        self.idents.push(node.name(self.ast));
    }

    fn visit_binding_identifier(&mut self, node: BindingIdentifier) {
        self.idents.push(node.name(self.ast));
    }

    fn visit_label_identifier(&mut self, node: LabelIdentifier) {
        self.idents.push(node.name(self.ast));
    }
}
