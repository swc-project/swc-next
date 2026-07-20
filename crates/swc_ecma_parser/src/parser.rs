use std::str;

use swc_experimental_allocator::wtf8::Wtf8;
use swc_experimental_ecma_ast::*;

use crate::{
    error::Error,
    lexer::{Lexer, LexerMode, LexicalError, lexical_error_help, lexical_error_message},
    token::{Precedence, Token, TokenTag},
    util::{decode_identifier_escapes, decode_string_escapes},
};

type ParseResult<T> = Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum CommentMode {
    None,
    #[default]
    Flat,
    Attached,
    Both,
}

impl CommentMode {
    pub fn collects(self) -> bool {
        self != Self::None
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Lang {
    #[default]
    Js,
    Ts,
    Jsx,
    Tsx,
    Dts,
}

impl Lang {
    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".d.ts") || path.ends_with(".d.mts") || path.ends_with(".d.cts") {
            Self::Dts
        } else if path.ends_with(".tsx") {
            Self::Tsx
        } else if path.ends_with(".ts") || path.ends_with(".mts") || path.ends_with(".cts") {
            Self::Ts
        } else if path.ends_with(".jsx") {
            Self::Jsx
        } else {
            Self::Js
        }
    }

    pub fn is_typescript(self) -> bool {
        matches!(self, Self::Ts | Self::Tsx | Self::Dts)
    }

    pub fn is_jsx(self) -> bool {
        matches!(self, Self::Jsx | Self::Tsx)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Options {
    pub source_type: SourceType,
    pub lang: Lang,
    pub preserve_parens: bool,
    pub allow_return_outside_function: bool,
    pub comments: CommentMode,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            source_type: SourceType::Module,
            lang: Lang::Js,
            preserve_parens: true,
            allow_return_outside_function: false,
            comments: CommentMode::Flat,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Context {
    allow_in: bool,
    allow_await: bool,
    allow_yield: bool,
    allow_return: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ParserState {
    cover_has_trailing_comma: Option<u32>,
    cover_has_init_name: bool,
}

#[derive(Clone, Copy)]
enum PropertyOrMethodKind {
    Init,
    Get,
    Set,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum PatternContext {
    Binding,
    Assignment,
}

impl From<PropertyKind> for PropertyOrMethodKind {
    fn from(kind: PropertyKind) -> Self {
        match kind {
            PropertyKind::Init => Self::Init,
            PropertyKind::Get => Self::Get,
            PropertyKind::Set => Self::Set,
        }
    }
}

impl From<MethodDefinitionKind> for PropertyOrMethodKind {
    fn from(kind: MethodDefinitionKind) -> Self {
        match kind {
            MethodDefinitionKind::Get => Self::Get,
            MethodDefinitionKind::Set => Self::Set,
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => Self::Init,
        }
    }
}

impl Context {
    fn top_level(options: Options) -> Self {
        Self {
            allow_in: true,
            allow_await: options.source_type == SourceType::Module,
            allow_yield: false,
            allow_return: options.allow_return_outside_function,
        }
    }
}

/// Parser state and typed AST writer.
pub struct Parser<'parser, 'allocator, 'source> {
    ast: &'parser mut Ast<'allocator>,
    source: &'source [u8],
    options: Options,
    lexer: Lexer<'source>,
    current_token: Token,
    previous_end: u32,
    context: Context,
    state: ParserState,
}

impl<'parser, 'allocator, 'source> Parser<'parser, 'allocator, 'source> {
    pub fn new(ast: &'parser mut Ast<'allocator>, source: &'source [u8], options: Options) -> Self {
        assert!(source.len() <= u32::MAX as usize);
        let lexer = Lexer::init(source, options.source_type, options.comments.collects());
        Self {
            ast,
            source,
            options,
            lexer,
            current_token: Token::eof(0),
            previous_end: 0,
            context: Context::top_level(options),
            state: ParserState::default(),
        }
    }

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        self.advance()?;
        let mut body = Vec::new();
        let mut directive_prologue = true;
        while self.current_token.tag != TokenTag::Eof {
            let item = self.parse_statement_list_item(directive_prologue)?;
            directive_prologue = directive_prologue && item.is_directive(self.ast);
            body.push(item);
        }
        let end = self.current_token.span.end;
        let body = self.ast.add_typed_sub_range(body);
        let hashbang = self.lexer.hashbang.map(|hashbang| {
            let start = hashbang.start;
            let end = start + u32::from(hashbang.len);
            let value = self.atom_from_span(Span::new(start, end));
            self.ast.hashbang(Span::new(0, end), value)
        });
        Ok(self
            .ast
            .program(Span::new(0, end), self.options.source_type, body, hashbang))
    }

    pub fn comments(&self) -> Vec<Comment<'allocator>> {
        self.lexer
            .comments
            .iter()
            .map(|comment| {
                let bytes = &self.source[comment.value.start as usize..comment.value.end as usize];
                let value = String::from_utf8_lossy(bytes);
                Comment {
                    comment_type: comment.comment_type,
                    value: self.ast.add_atom(&value),
                    span: comment.span,
                }
            })
            .collect()
    }

    pub fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_expression_with_precedence(Precedence::COMMA)
    }

    fn parse_assignment_expression_allow_in(&mut self) -> ParseResult<Expr> {
        let old_allow_in = self.context.allow_in;
        self.context.allow_in = true;
        let result = self.parse_expression_with_precedence(Precedence::ASSIGNMENT);
        self.context.allow_in = old_allow_in;
        result
    }

    fn parse_assignment_expression_in_cover(&mut self) -> ParseResult<Expr> {
        let old_allow_in = self.context.allow_in;
        self.context.allow_in = true;
        let result = self.parse_expression_with_precedence_context(Precedence::ASSIGNMENT, true);
        self.context.allow_in = old_allow_in;
        result
    }

    fn parse_expression_allow_in(&mut self) -> ParseResult<Expr> {
        let old_allow_in = self.context.allow_in;
        self.context.allow_in = true;
        let result = self.parse_expression();
        self.context.allow_in = old_allow_in;
        result
    }

    fn parse_statement_list_item(
        &mut self,
        directive_prologue: bool,
    ) -> ParseResult<StatementListItem> {
        let statement = self.parse_statement()?;
        if directive_prologue
            && let StmtData::ExpressionStatement(expression_statement) =
                self.ast.stmt_data(statement)
        {
            let expression = expression_statement.expression(self.ast);
            if let ExprData::StringLiteral(string_literal) = self.ast.expr_data(expression) {
                let directive = self.ast.directive(
                    statement.span(self.ast),
                    string_literal,
                    string_literal.value(self.ast),
                );
                return Ok(StatementListItem::Directive(directive));
            }
        }
        Ok(StatementListItem::Stmt(statement))
    }

    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        let is_async_function =
            self.current_token.tag == TokenTag::Async && self.peek_tag()? == TokenTag::Function;
        let is_import_declaration = self.current_token.tag == TokenTag::Import
            && !matches!(self.peek_tag()?, TokenTag::LeftParen | TokenTag::Dot);
        let is_type_alias =
            self.options.lang.is_typescript() && self.current_token.tag == TokenTag::Type;
        let is_await_using = if self.current_token.tag == TokenTag::Await {
            let using = self.peek_token()?;
            if using.tag == TokenTag::Using && !using.has_line_terminator_before() {
                let binding = self.peek_second_token()?;
                !binding.has_line_terminator_before() && binding.tag != TokenTag::LeftBracket
            } else {
                false
            }
        } else {
            false
        };
        let is_using_declaration = self.current_token.tag == TokenTag::Using && {
            let next = self.peek_token()?;
            !next.has_line_terminator_before() && next.tag != TokenTag::LeftBracket
        };
        let let_starts_declaration = self.current_token.tag == TokenTag::Let && {
            let next = self.peek_token()?;
            !self.current_token.is_escaped()
                && if next.has_line_terminator_before() {
                    next.tag == TokenTag::LeftBracket
                        || next.tag == TokenTag::Let
                        || (next.tag == TokenTag::Await && self.context.allow_await)
                        || (next.tag == TokenTag::Yield && self.context.allow_yield)
                } else {
                    matches!(next.tag, TokenTag::LeftBracket | TokenTag::LeftBrace)
                        || (next.tag.is_identifier_like()
                            && !next.tag.is_unconditionally_reserved())
                }
        };
        match self.current_token.tag {
            TokenTag::LeftBrace => self.parse_block_statement(),
            TokenTag::Semicolon => self.parse_empty_statement(),
            TokenTag::If => self.parse_if_statement(),
            TokenTag::While => self.parse_while_statement(),
            TokenTag::Do => self.parse_do_while_statement(),
            TokenTag::Return => self.parse_return_statement(),
            TokenTag::Throw => self.parse_throw_statement(),
            TokenTag::Break => self.parse_break_statement(),
            TokenTag::Continue => self.parse_continue_statement(),
            TokenTag::Debugger => self.parse_debugger_statement(),
            TokenTag::Let if !let_starts_declaration => {
                self.parse_expression_or_labeled_statement()
            }
            TokenTag::Var | TokenTag::Let | TokenTag::Const => {
                self.parse_variable_declaration_statement()
            }
            TokenTag::Using if is_using_declaration => self.parse_variable_declaration_statement(),
            TokenTag::Await if is_await_using => self.parse_variable_declaration_statement(),
            TokenTag::Function => self.parse_function_declaration(false),
            TokenTag::Async if is_async_function => self.parse_function_declaration(true),
            TokenTag::For => self.parse_for_statement(),
            TokenTag::Class => self.parse_class_declaration(),
            TokenTag::At => self.parse_decorated_class_declaration(),
            TokenTag::Switch => self.parse_switch_statement(),
            TokenTag::Try => self.parse_try_statement(),
            TokenTag::With => self.parse_with_statement(),
            TokenTag::Import if is_import_declaration => self.parse_import_declaration(),
            TokenTag::Export => self.parse_export_declaration(),
            TokenTag::Type if is_type_alias => self.parse_ts_type_alias_declaration(),
            _ => self.parse_expression_or_labeled_statement(),
        }
    }

    fn parse_block_statement(&mut self) -> ParseResult<Stmt> {
        let block = self.parse_block()?;
        Ok(Stmt::BlockStatement(block))
    }

    fn parse_block(&mut self) -> ParseResult<BlockStatement> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let mut statements = Vec::new();
        while !matches!(self.current_token.tag, TokenTag::RightBrace | TokenTag::Eof) {
            statements.push(self.parse_statement()?);
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        let body = self.ast.add_typed_sub_range(statements);
        Ok(self.ast.block_statement(Span::new(start, end), body))
    }

    fn parse_empty_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.expect(TokenTag::Semicolon)?.span;
        Ok(self.ast.stmt_empty_statement(span))
    }

    fn parse_if_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::If)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let test = self.parse_expression()?;
        self.expect(TokenTag::RightParen)?;
        let consequent = self.parse_statement()?;
        let alternate = if self.eat(TokenTag::Else)? {
            Some(self.parse_statement()?)
        } else {
            None
        };
        let end = alternate.map_or_else(
            || consequent.span(self.ast).end,
            |stmt| stmt.span(self.ast).end,
        );
        Ok(self
            .ast
            .stmt_if_statement(Span::new(start, end), test, consequent, alternate))
    }

    fn parse_while_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::While)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let test = self.parse_expression()?;
        self.expect(TokenTag::RightParen)?;
        let body = self.parse_statement()?;
        let end = body.span(self.ast).end;
        Ok(self
            .ast
            .stmt_while_statement(Span::new(start, end), test, body))
    }

    fn parse_do_while_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Do)?.span.start;
        let body = self.parse_statement()?;
        self.expect(TokenTag::While)?;
        self.expect(TokenTag::LeftParen)?;
        let test = self.parse_expression()?;
        self.expect(TokenTag::RightParen)?;
        self.eat(TokenTag::Semicolon)?;
        let end = self.previous_end;
        Ok(self
            .ast
            .stmt_do_while_statement(Span::new(start, end), body, test))
    }

    fn parse_return_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Return)?.span.start;
        if !self.context.allow_return {
            return self.error_at(
                Span::new(start, self.previous_end),
                "return is not allowed outside a function",
            );
        }
        let argument = if self.current_token.has_line_terminator_before()
            || matches!(
                self.current_token.tag,
                TokenTag::Semicolon | TokenTag::RightBrace | TokenTag::Eof
            ) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        let end = self.eat_semicolon()?;
        Ok(self
            .ast
            .stmt_return_statement(Span::new(start, end), argument))
    }

    fn parse_throw_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Throw)?.span.start;
        if self.current_token.has_line_terminator_before() {
            return self.error("line terminator is not allowed after throw");
        }
        let argument = self.parse_expression()?;
        let end = self.eat_semicolon()?;
        Ok(self
            .ast
            .stmt_throw_statement(Span::new(start, end), argument))
    }

    fn parse_break_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Break)?.span.start;
        let label = if self.current_token.has_line_terminator_before()
            || !self.current_token.tag.is_identifier_like()
        {
            None
        } else {
            Some(self.parse_label_identifier()?)
        };
        let end = self.eat_semicolon()?;
        Ok(self.ast.stmt_break_statement(Span::new(start, end), label))
    }

    fn parse_continue_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Continue)?.span.start;
        let label = if self.current_token.has_line_terminator_before()
            || !self.current_token.tag.is_identifier_like()
        {
            None
        } else {
            Some(self.parse_label_identifier()?)
        };
        let end = self.eat_semicolon()?;
        Ok(self
            .ast
            .stmt_continue_statement(Span::new(start, end), label))
    }

    fn parse_debugger_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Debugger)?.span.start;
        let end = self.eat_semicolon()?;
        Ok(self.ast.stmt_debugger_statement(Span::new(start, end)))
    }

    fn parse_with_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::With)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let object = self.parse_expression()?;
        self.expect(TokenTag::RightParen)?;
        let body = self.parse_statement()?;
        let end = body.span(self.ast).end;
        Ok(self
            .ast
            .stmt_with_statement(Span::new(start, end), object, body))
    }

    fn parse_expression_or_labeled_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.current_token.span.start;
        if self.current_token.tag.is_identifier_like() && self.peek_tag()? == TokenTag::Colon {
            let label = self.parse_label_identifier()?;
            self.expect(TokenTag::Colon)?;
            let body = self.parse_statement()?;
            let end = body.span(self.ast).end;
            return Ok(self
                .ast
                .stmt_labeled_statement(Span::new(start, end), label, body));
        }
        let expression = self.parse_expression()?;
        let end = self.eat_semicolon()?;
        Ok(self
            .ast
            .stmt_expression_statement(Span::new(start, end), expression))
    }

    fn parse_variable_declaration_statement(&mut self) -> ParseResult<Stmt> {
        let declaration = self.parse_variable_declaration(false)?;
        let span = declaration.span(self.ast);
        let end = self.eat_semicolon()?;
        declaration.set_span(self.ast, Span::new(span.start, end));
        Ok(Stmt::Declaration(Decl::VariableDeclaration(declaration)))
    }

    fn parse_variable_declaration(
        &mut self,
        in_for_head: bool,
    ) -> ParseResult<VariableDeclaration> {
        let token = self.current_token;
        let kind = match token.tag {
            TokenTag::Var => VariableKind::Var,
            TokenTag::Let => VariableKind::Let,
            TokenTag::Const => VariableKind::Const,
            TokenTag::Using => VariableKind::Using,
            TokenTag::Await => {
                self.advance()?;
                if self.current_token.has_line_terminator_before() {
                    return self.error("'await using' may not contain a line terminator");
                }
                self.expect(TokenTag::Using)?;
                VariableKind::AwaitUsing
            }
            _ => return self.error("expected variable declaration"),
        };
        if token.tag != TokenTag::Await {
            self.advance()?;
        }
        let mut declarators = Vec::new();
        loop {
            let start = self.current_token.span.start;
            let id = self.parse_binding_pattern(false)?;
            if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing)
                && !matches!(
                    self.ast.binding_pattern_data(id),
                    BindingPatternData::BindingIdentifier(_)
                )
            {
                return self.error_at(
                    id.span(self.ast),
                    "using declarations require binding identifiers",
                );
            }
            let init = if self.eat(TokenTag::Assign)? {
                Some(self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?)
            } else {
                None
            };
            if matches!(
                kind,
                VariableKind::Const | VariableKind::Using | VariableKind::AwaitUsing
            ) && init.is_none()
                && !in_for_head
            {
                return self.error_at(
                    id.span(self.ast),
                    "const declarations require an initializer",
                );
            }
            let end = init.map_or_else(|| id.span(self.ast).end, |expr| expr.span(self.ast).end);
            declarators.push(
                self.ast
                    .variable_declarator(Span::new(start, end), id, init, false),
            );
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        let end = declarators
            .last()
            .expect("a variable declaration has at least one declarator")
            .span(self.ast)
            .end;
        let declarators = self.ast.add_typed_sub_range(declarators);
        Ok(self.ast.variable_declaration(
            Span::new(token.span.start, end),
            kind,
            declarators,
            false,
        ))
    }

    fn parse_binding_pattern(&mut self, allow_default: bool) -> ParseResult<BindingPattern> {
        let pattern = self.parse_binding_pattern_base()?;
        if allow_default && self.eat(TokenTag::Assign)? {
            let right = self.parse_assignment_expression_allow_in()?;
            let span = Span::new(pattern.span(self.ast).start, right.span(self.ast).end);
            let decorators = self.empty_range();
            return Ok(self.ast.binding_pattern_assignment_pattern(
                span, pattern, right, decorators, None, false,
            ));
        }
        Ok(pattern)
    }

    fn parse_binding_pattern_base(&mut self) -> ParseResult<BindingPattern> {
        match self.current_token.tag {
            tag if tag.is_identifier_like() && !tag.is_unconditionally_reserved() => {
                let token = self.current_token;
                let name = self.identifier_atom(token)?;
                self.validate_identifier(token, name.as_str(), "a binding identifier")?;
                self.advance()?;
                let (optional, type_annotation, end) =
                    self.parse_binding_type_suffix(token.span.end)?;
                let decorators = self.empty_range();
                Ok(self.ast.binding_pattern_binding_identifier(
                    Span::new(token.span.start, end),
                    name,
                    decorators,
                    type_annotation,
                    optional,
                    None,
                ))
            }
            TokenTag::LeftBracket => self.parse_array_binding_pattern(),
            TokenTag::LeftBrace => self.parse_object_binding_pattern(),
            _ => self.error("expected a binding pattern"),
        }
    }

    fn parse_array_binding_pattern(&mut self) -> ParseResult<BindingPattern> {
        let start = self.expect(TokenTag::LeftBracket)?.span.start;
        let mut elements = Vec::new();
        let mut rest = None;
        while self.current_token.tag != TokenTag::RightBracket {
            if self.eat(TokenTag::Comma)? {
                elements.push(None);
                continue;
            }
            if self.current_token.tag == TokenTag::Spread {
                let rest_start = self.current_token.span.start;
                self.advance()?;
                let argument = self.parse_binding_pattern(false)?;
                let span = Span::new(rest_start, argument.span(self.ast).end);
                let decorators = self.empty_range();
                rest = Some(
                    self.ast
                        .binding_rest_element(span, argument, decorators, None, false),
                );
                if self.eat(TokenTag::Comma)? {
                    return self.error("a rest element must be last in a binding pattern");
                }
                break;
            }
            elements.push(Some(self.parse_binding_pattern(true)?));
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        let close_end = self.expect(TokenTag::RightBracket)?.span.end;
        let (optional, type_annotation, end) = self.parse_binding_type_suffix(close_end)?;
        let elements = self.ast.add_typed_sub_range(elements);
        let decorators = self.empty_range();
        Ok(self.ast.binding_pattern_array_pattern(
            Span::new(start, end),
            elements,
            rest,
            decorators,
            type_annotation,
            optional,
        ))
    }

    fn parse_object_binding_pattern(&mut self) -> ParseResult<BindingPattern> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let mut properties = Vec::new();
        let mut rest = None;
        while self.current_token.tag != TokenTag::RightBrace {
            if self.current_token.tag == TokenTag::Spread {
                let rest_start = self.current_token.span.start;
                self.advance()?;
                let argument = self.parse_binding_pattern(false)?;
                let span = Span::new(rest_start, argument.span(self.ast).end);
                let decorators = self.empty_range();
                rest = Some(
                    self.ast
                        .binding_rest_element(span, argument, decorators, None, false),
                );
                if self.eat(TokenTag::Comma)? {
                    return self.error("a rest element must be last in a binding pattern");
                }
                break;
            }

            let property_start = self.current_token.span.start;
            let (key, computed) = if self.eat(TokenTag::LeftBracket)? {
                let expression = self.parse_assignment_expression_allow_in()?;
                self.expect(TokenTag::RightBracket)?;
                (PropertyKey::Expr(expression), true)
            } else {
                (self.parse_property_key()?, false)
            };
            if matches!(
                self.ast.property_key_data(key),
                PropertyKeyData::PrivateIdentifier(_)
            ) {
                return self.error_at(
                    key.span(self.ast),
                    "private names are not allowed in binding patterns",
                );
            }
            let (value, shorthand) = if self.eat(TokenTag::Colon)? {
                (self.parse_binding_pattern(true)?, false)
            } else {
                let PropertyKeyData::IdentifierName(identifier) = self.ast.property_key_data(key)
                else {
                    return self.error_at(
                        key.span(self.ast),
                        "expected ':' after binding property name",
                    );
                };
                self.validate_identifier_name(
                    identifier.span(self.ast),
                    identifier.name(self.ast).as_str(),
                    "a binding identifier",
                )?;
                let decorators = self.empty_range();
                let value = self.ast.binding_pattern_binding_identifier(
                    identifier.span(self.ast),
                    identifier.name(self.ast),
                    decorators,
                    None,
                    false,
                    None,
                );
                let value = if self.eat(TokenTag::Assign)? {
                    let right = self.parse_assignment_expression_allow_in()?;
                    let span = Span::new(value.span(self.ast).start, right.span(self.ast).end);
                    let decorators = self.empty_range();
                    self.ast.binding_pattern_assignment_pattern(
                        span, value, right, decorators, None, false,
                    )
                } else {
                    value
                };
                (value, true)
            };
            let span = Span::new(property_start, value.span(self.ast).end);
            properties.push(
                self.ast
                    .binding_property(span, key, value, shorthand, computed),
            );
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        let close_end = self.expect(TokenTag::RightBrace)?.span.end;
        let (optional, type_annotation, end) = self.parse_binding_type_suffix(close_end)?;
        let properties = self.ast.add_typed_sub_range(properties);
        let decorators = self.empty_range();
        Ok(self.ast.binding_pattern_object_pattern(
            Span::new(start, end),
            properties,
            rest,
            decorators,
            type_annotation,
            optional,
        ))
    }

    fn parse_binding_type_suffix(
        &mut self,
        initial_end: u32,
    ) -> ParseResult<(bool, Option<TsTypeAnnotation>, u32)> {
        let optional = self.options.lang.is_typescript() && self.eat(TokenTag::Question)?;
        let type_annotation =
            if self.options.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                Some(self.parse_ts_type_annotation()?)
            } else {
                None
            };
        let end = type_annotation.map_or_else(
            || {
                if optional {
                    self.previous_end
                } else {
                    initial_end
                }
            },
            |node| node.span(self.ast).end,
        );
        Ok((optional, type_annotation, end))
    }

    fn parse_function_declaration(&mut self, async_prefix: bool) -> ParseResult<Stmt> {
        let start = self.current_token.span.start;
        if async_prefix {
            self.expect(TokenTag::Async)?;
        }
        self.expect(TokenTag::Function)?;
        let generator = self.eat(TokenTag::Star)?;
        let id = Some(self.parse_binding_identifier()?);
        let (params, body, return_type) = self.parse_function_tail(generator, async_prefix)?;
        let end = body.span(self.ast).end;
        let function = self.ast.function(
            Span::new(start, end),
            FunctionType::FunctionDeclaration,
            id,
            generator,
            async_prefix,
            false,
            params,
            Some(body),
            None,
            return_type,
        );
        Ok(Stmt::Declaration(Decl::Function(function)))
    }

    fn parse_function_expression(&mut self, async_prefix: bool) -> ParseResult<Expr> {
        let start = self.current_token.span.start;
        if async_prefix {
            self.expect(TokenTag::Async)?;
        }
        self.expect(TokenTag::Function)?;
        let generator = self.eat(TokenTag::Star)?;
        let old_context = self.context;
        self.context.allow_yield = generator;
        self.context.allow_await = async_prefix;
        let id = if self.current_token.tag.is_identifier_like() {
            Some(self.parse_binding_identifier()?)
        } else {
            None
        };
        self.context = old_context;
        let (params, body, return_type) = self.parse_function_tail(generator, async_prefix)?;
        let end = body.span(self.ast).end;
        let function = self.ast.function(
            Span::new(start, end),
            FunctionType::FunctionExpression,
            id,
            generator,
            async_prefix,
            false,
            params,
            Some(body),
            None,
            return_type,
        );
        Ok(Expr::Function(function))
    }

    fn parse_class_declaration(&mut self) -> ParseResult<Stmt> {
        let class = self.parse_class(true)?;
        Ok(Stmt::Declaration(Decl::Class(class)))
    }

    fn parse_decorated_class_declaration(&mut self) -> ParseResult<Stmt> {
        let (start, decorators) = self.parse_decorators()?;
        let class = self.parse_class_with_decorators(true, start, decorators)?;
        Ok(Stmt::Declaration(Decl::Class(class)))
    }

    fn parse_class_expression(&mut self) -> ParseResult<Expr> {
        let class = self.parse_class(false)?;
        Ok(Expr::Class(class))
    }

    fn parse_class(&mut self, declaration: bool) -> ParseResult<Class> {
        let start = self.current_token.span.start;
        let decorators = self.empty_range();
        self.parse_class_with_decorators(declaration, start, decorators)
    }

    fn parse_class_with_decorators(
        &mut self,
        declaration: bool,
        start: u32,
        decorators: TypedSubRange<Decorator>,
    ) -> ParseResult<Class> {
        self.expect(TokenTag::Class)?;
        let id = if self.current_token.tag.is_identifier_like()
            && self.current_token.tag != TokenTag::Extends
        {
            Some(self.parse_binding_identifier()?)
        } else if declaration {
            return self.error("class declarations require a name");
        } else {
            None
        };
        let super_class = if self.eat(TokenTag::Extends)? {
            let expression = self.parse_expression_with_precedence(Precedence::RELATIONAL)?;
            if matches!(
                self.ast.expr_data(expression),
                ExprData::ArrowFunctionExpression(_)
                    | ExprData::AssignmentExpression(_)
                    | ExprData::ConditionalExpression(_)
                    | ExprData::SequenceExpression(_)
                    | ExprData::YieldExpression(_)
            ) {
                return self.error_at(
                    expression.span(self.ast),
                    "class heritage must be a left-hand-side expression",
                );
            }
            Some(expression)
        } else {
            None
        };
        let body = self.parse_class_body()?;
        let end = body.span(self.ast).end;
        let implements = self.empty_range();
        Ok(self.ast.class(
            Span::new(start, end),
            if declaration {
                ClassType::ClassDeclaration
            } else {
                ClassType::ClassExpression
            },
            decorators,
            id,
            super_class,
            body,
            None,
            None,
            implements,
            false,
            false,
        ))
    }

    fn parse_class_body(&mut self) -> ParseResult<ClassBody> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let mut elements = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace {
            if self.eat(TokenTag::Semicolon)? {
                continue;
            }
            let (element_start, decorators) = if self.current_token.tag == TokenTag::At {
                self.parse_decorators()?
            } else {
                (self.current_token.span.start, self.empty_range())
            };
            let mut is_static = false;
            let mut async_function = false;
            let mut generator = false;
            let mut accessor = false;
            let mut method_kind = MethodDefinitionKind::Method;
            let mut key = None;

            if self.current_token.tag == TokenTag::Static {
                let token = self.current_token;
                self.advance()?;
                if self.current_token.tag == TokenTag::LeftBrace {
                    if !decorators.is_empty() {
                        return self.error_at(
                            Span::new(element_start, token.span.end),
                            "a static block may not have decorators",
                        );
                    }
                    let old_context = self.context;
                    self.context.allow_return = false;
                    self.context.allow_yield = false;
                    self.context.allow_await = true;
                    let block = self.parse_block()?;
                    self.context = old_context;
                    let body = block.body(self.ast);
                    let element = self
                        .ast
                        .static_block(Span::new(element_start, block.span(self.ast).end), body);
                    elements.push(ClassElement::StaticBlock(element));
                    continue;
                }
                if Self::is_property_key_start(self.current_token.tag) {
                    if token.is_escaped() {
                        return self
                            .error_at(token.span, "keywords may not contain escape sequences");
                    }
                    is_static = true;
                } else {
                    let name = self.identifier_atom(token)?;
                    key = Some(self.ast.property_key_identifier_name(token.span, name));
                }
            }

            if key.is_none() && self.current_token.tag == TokenTag::Async {
                let token = self.current_token;
                self.advance()?;
                if !self.current_token.has_line_terminator_before()
                    && Self::is_property_key_start(self.current_token.tag)
                {
                    if token.is_escaped() {
                        return self
                            .error_at(token.span, "keywords may not contain escape sequences");
                    }
                    async_function = true;
                } else {
                    let name = self.identifier_atom(token)?;
                    key = Some(self.ast.property_key_identifier_name(token.span, name));
                }
            }

            if key.is_none() && self.current_token.tag == TokenTag::Accessor {
                let token = self.current_token;
                self.advance()?;
                if !self.current_token.has_line_terminator_before()
                    && Self::is_property_key_start(self.current_token.tag)
                {
                    if token.is_escaped() {
                        return self
                            .error_at(token.span, "keywords may not contain escape sequences");
                    }
                    accessor = true;
                } else {
                    let name = self.identifier_atom(token)?;
                    key = Some(self.ast.property_key_identifier_name(token.span, name));
                }
            }

            if key.is_none() && self.eat(TokenTag::Star)? {
                generator = true;
            }

            if key.is_none()
                && !async_function
                && !generator
                && matches!(self.current_token.tag, TokenTag::Get | TokenTag::Set)
            {
                let token = self.current_token;
                self.advance()?;
                if !self.current_token.has_line_terminator_before()
                    && Self::is_property_key_start(self.current_token.tag)
                {
                    if token.is_escaped() {
                        return self
                            .error_at(token.span, "keywords may not contain escape sequences");
                    }
                    method_kind = if token.tag == TokenTag::Get {
                        MethodDefinitionKind::Get
                    } else {
                        MethodDefinitionKind::Set
                    };
                } else {
                    let name = self.identifier_atom(token)?;
                    key = Some(self.ast.property_key_identifier_name(token.span, name));
                }
            }

            let computed = key.is_none() && self.current_token.tag == TokenTag::LeftBracket;
            let key = match key {
                Some(key) => key,
                None => self.parse_property_key()?,
            };

            if matches!(
                self.ast.property_key_data(key),
                PropertyKeyData::PrivateIdentifier(private)
                    if private.name(self.ast).as_str() == "constructor"
            ) {
                return self.error_at(
                    key.span(self.ast),
                    "a class may not declare a private '#constructor' member",
                );
            }

            if !is_static
                && !computed
                && method_kind == MethodDefinitionKind::Method
                && self.property_key_name_is(key, "constructor")
            {
                method_kind = MethodDefinitionKind::Constructor;
            }

            if self.current_token.tag == TokenTag::LeftParen {
                if is_static && !computed && self.property_key_name_is(key, "prototype") {
                    return self.error_at(
                        key.span(self.ast),
                        "a static class method may not be named 'prototype'",
                    );
                }
                if !is_static
                    && !computed
                    && matches!(
                        method_kind,
                        MethodDefinitionKind::Get | MethodDefinitionKind::Set
                    )
                    && self.property_key_name_is(key, "constructor")
                {
                    return self
                        .error_at(key.span(self.ast), "a constructor may not be an accessor");
                }
                if method_kind == MethodDefinitionKind::Constructor && (async_function || generator)
                {
                    return self.error_at(
                        key.span(self.ast),
                        "a constructor may not be async or a generator",
                    );
                }
                elements.push(self.parse_class_method(
                    element_start,
                    decorators,
                    key,
                    computed,
                    is_static,
                    async_function,
                    generator,
                    method_kind,
                )?);
            } else {
                if async_function || generator || method_kind != MethodDefinitionKind::Method {
                    return self.error("expected '(' after method name");
                }
                if !computed
                    && ((!is_static && self.property_key_name_is(key, "constructor"))
                        || (is_static
                            && (self.property_key_name_is(key, "constructor")
                                || self.property_key_name_is(key, "prototype"))))
                {
                    return self.error_at(key.span(self.ast), "invalid class field name");
                }
                let old_context = self.context;
                self.context.allow_await = false;
                self.context.allow_yield = false;
                let value = if self.eat(TokenTag::Assign)? {
                    Some(self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?)
                } else {
                    None
                };
                self.context = old_context;
                let end = self.eat_semicolon()?;
                let property = self.ast.property_definition(
                    Span::new(element_start, end),
                    decorators,
                    key,
                    value,
                    computed,
                    is_static,
                    accessor,
                    None,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    Accessibility::None,
                );
                elements.push(ClassElement::PropertyDefinition(property));
            }
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        let elements = self.ast.add_typed_sub_range(elements);
        Ok(self.ast.class_body(Span::new(start, end), elements))
    }

    fn parse_class_method(
        &mut self,
        start: u32,
        decorators: TypedSubRange<Decorator>,
        key: PropertyKey,
        computed: bool,
        is_static: bool,
        async_function: bool,
        generator: bool,
        kind: MethodDefinitionKind,
    ) -> ParseResult<ClassElement> {
        let old_context = self.context;
        self.context.allow_return = true;
        self.context.allow_await = async_function;
        self.context.allow_yield = generator;
        let result = (|| {
            let params =
                self.parse_formal_parameters_kind(FormalParameterKind::UniqueFormalParameters)?;
            self.check_accessor_arity(kind, params)?;
            let return_type =
                if self.options.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                    Some(self.parse_ts_type_annotation()?)
                } else {
                    None
                };
            let body = self.parse_function_body()?;
            Ok((params, return_type, body))
        })();
        self.context = old_context;
        let (params, return_type, body) = result?;
        let end = body.span(self.ast).end;
        let function = self.ast.function(
            Span::new(start, end),
            FunctionType::FunctionExpression,
            None,
            generator,
            async_function,
            false,
            params,
            Some(body),
            None,
            return_type,
        );
        let method = self.ast.method_definition(
            Span::new(start, end),
            decorators,
            key,
            function,
            kind,
            computed,
            is_static,
            false,
            false,
            false,
            Accessibility::None,
        );
        Ok(ClassElement::MethodDefinition(method))
    }

    fn parse_decorators(&mut self) -> ParseResult<(u32, TypedSubRange<Decorator>)> {
        let start = self.current_token.span.start;
        let mut decorators = Vec::new();
        while self.eat(TokenTag::At)? {
            let decorator_start = self.previous_end - 1;
            let expression = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
            decorators.push(self.ast.decorator(
                Span::new(decorator_start, expression.span(self.ast).end),
                expression,
            ));
        }
        Ok((start, self.ast.add_typed_sub_range(decorators)))
    }

    fn property_key_name_is(&self, key: PropertyKey, expected: &str) -> bool {
        match self.ast.property_key_data(key) {
            PropertyKeyData::IdentifierName(identifier) => {
                identifier.name(self.ast).as_str() == expected
            }
            PropertyKeyData::StringLiteral(string) => {
                string.value(self.ast).as_wtf8().as_str() == Some(expected)
            }
            _ => false,
        }
    }

    fn check_accessor_arity(
        &self,
        kind: impl Into<PropertyOrMethodKind>,
        params: FormalParameters,
    ) -> ParseResult<()> {
        let kind = kind.into();
        let count = params.items(self.ast).len();
        let has_rest = params.rest(self.ast).is_some();
        match kind {
            PropertyOrMethodKind::Get if count != 0 || has_rest => {
                self.error_at(params.span(self.ast), "a getter must have no parameters")
            }
            PropertyOrMethodKind::Set if count != 1 || has_rest => self.error_at(
                params.span(self.ast),
                "a setter must have exactly one parameter",
            ),
            _ => Ok(()),
        }
    }

    fn parse_function_tail(
        &mut self,
        generator: bool,
        async_function: bool,
    ) -> ParseResult<(FormalParameters, FunctionBody, Option<TsTypeAnnotation>)> {
        let old_context = self.context;
        self.context.allow_return = true;
        self.context.allow_yield = generator;
        self.context.allow_await = async_function;
        let params = self.parse_formal_parameters()?;
        let return_type =
            if self.options.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                Some(self.parse_ts_type_annotation()?)
            } else {
                None
            };
        let body = self.parse_function_body();
        self.context = old_context;
        Ok((params, body?, return_type))
    }

    fn parse_formal_parameters(&mut self) -> ParseResult<FormalParameters> {
        self.parse_formal_parameters_kind(FormalParameterKind::FormalParameters)
    }

    fn parse_formal_parameters_kind(
        &mut self,
        kind: FormalParameterKind,
    ) -> ParseResult<FormalParameters> {
        let start = self.expect(TokenTag::LeftParen)?.span.start;
        let mut items = Vec::new();
        let mut rest = None;
        while self.current_token.tag != TokenTag::RightParen {
            if self.current_token.tag == TokenTag::Spread {
                let rest_start = self.current_token.span.start;
                self.advance()?;
                let argument = self.parse_binding_pattern(false)?;
                let span = Span::new(rest_start, argument.span(self.ast).end);
                let decorators = self.empty_range();
                rest = Some(
                    self.ast
                        .binding_rest_element(span, argument, decorators, None, false),
                );
                if self.eat(TokenTag::Comma)? {
                    return self.error("a rest parameter must be last");
                }
                break;
            }
            let pattern = self.parse_binding_pattern(true)?;
            let span = pattern.span(self.ast);
            let parameter_pattern = FormalParameterPattern::BindingPattern(pattern);
            let parameter = self.ast.formal_parameter(span, parameter_pattern);
            items.push(FormalParameterItem::FormalParameter(parameter));
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        let end = self.expect(TokenTag::RightParen)?.span.end;
        let items = self.ast.add_typed_sub_range(items);
        Ok(self
            .ast
            .formal_parameters(Span::new(start, end), items, rest, kind))
    }

    fn parse_function_body(&mut self) -> ParseResult<FunctionBody> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let mut body = Vec::new();
        let mut directive_prologue = true;
        while !matches!(self.current_token.tag, TokenTag::RightBrace | TokenTag::Eof) {
            let item = self.parse_statement_list_item(directive_prologue)?;
            directive_prologue = directive_prologue && item.is_directive(self.ast);
            body.push(item);
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        let body = self.ast.add_typed_sub_range(body);
        Ok(self.ast.function_body(Span::new(start, end), body))
    }

    fn parse_import_declaration(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Import)?.span.start;
        let mut specifiers = Vec::new();
        let phase =
            if self.current_token.tag == TokenTag::Defer && self.peek_tag()? == TokenTag::Star {
                self.advance()?;
                Some(ImportPhase::Defer)
            } else {
                None
            };
        let source = if self.current_token.tag == TokenTag::StringLiteral {
            self.parse_string_literal_node()?
        } else {
            if self.current_token.tag.is_identifier_like() {
                let local = self.parse_binding_identifier()?;
                let span = local.span(self.ast);
                let specifier = self.ast.import_default_specifier(span, local);
                specifiers.push(ImportDeclarationSpecifier::ImportDefaultSpecifier(
                    specifier,
                ));
                self.eat(TokenTag::Comma)?;
            }
            if self.eat(TokenTag::Star)? {
                let namespace_start = self.previous_end - 1;
                self.expect(TokenTag::As)?;
                let local = self.parse_binding_identifier()?;
                let span = Span::new(namespace_start, local.span(self.ast).end);
                let specifier = self.ast.import_namespace_specifier(span, local);
                specifiers.push(ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                    specifier,
                ));
            } else if self.eat(TokenTag::LeftBrace)? {
                while self.current_token.tag != TokenTag::RightBrace {
                    let imported = self.parse_module_export_name()?;
                    let imported_span = imported.span(self.ast);
                    let local = if self.eat(TokenTag::As)? {
                        self.parse_binding_identifier()?
                    } else {
                        let ModuleExportNameData::IdentifierName(identifier) =
                            self.ast.module_export_name_data(imported)
                        else {
                            return self.error_at(
                                imported_span,
                                "string import names require a local alias",
                            );
                        };
                        let decorators = self.empty_range();
                        self.ast.binding_identifier(
                            imported_span,
                            identifier.name(self.ast),
                            decorators,
                            None,
                            false,
                            None,
                        )
                    };
                    let span = Span::new(imported_span.start, local.span(self.ast).end);
                    let specifier =
                        self.ast
                            .import_specifier(span, imported, local, ImportOrExportKind::Value);
                    specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(specifier));
                    if !self.eat(TokenTag::Comma)? {
                        break;
                    }
                }
                self.expect(TokenTag::RightBrace)?;
            }
            self.expect(TokenTag::From)?;
            self.parse_string_literal_node()?
        };
        let attributes = self.parse_import_attributes()?;
        let end = self.eat_semicolon()?;
        let specifiers = self.ast.add_typed_sub_range(specifiers);
        Ok(self.ast.stmt_import_declaration(
            Span::new(start, end),
            specifiers,
            source,
            attributes,
            phase,
            ImportOrExportKind::Value,
        ))
    }

    fn parse_export_declaration(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Export)?.span.start;
        if self.eat(TokenTag::Default)? {
            let (declaration, needs_semicolon) = if self.current_token.tag == TokenTag::Function {
                let expression = self.parse_function_expression(false)?;
                let ExprData::Function(function) = self.ast.expr_data(expression) else {
                    unreachable!()
                };
                function.set_function_type(self.ast, FunctionType::FunctionDeclaration);
                (ExportDefaultDeclarationKind::Function(function), false)
            } else if self.current_token.tag == TokenTag::Async
                && self.peek_tag()? == TokenTag::Function
            {
                let expression = self.parse_function_expression(true)?;
                let ExprData::Function(function) = self.ast.expr_data(expression) else {
                    unreachable!()
                };
                function.set_function_type(self.ast, FunctionType::FunctionDeclaration);
                (ExportDefaultDeclarationKind::Function(function), false)
            } else if self.current_token.tag == TokenTag::Class {
                let class = self.parse_class(false)?;
                class.set_class_type(self.ast, ClassType::ClassDeclaration);
                (ExportDefaultDeclarationKind::Class(class), false)
            } else {
                (
                    ExportDefaultDeclarationKind::Expr(
                        self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?,
                    ),
                    true,
                )
            };
            let end = if needs_semicolon {
                self.eat_semicolon()?
            } else {
                declaration.span(self.ast).end
            };
            return Ok(self
                .ast
                .stmt_export_default_declaration(Span::new(start, end), declaration));
        }
        if self.eat(TokenTag::Star)? {
            let exported = if self.eat(TokenTag::As)? {
                Some(self.parse_module_export_name()?)
            } else {
                None
            };
            self.expect(TokenTag::From)?;
            let source = self.parse_string_literal_node()?;
            let attributes = self.parse_import_attributes()?;
            let end = self.eat_semicolon()?;
            return Ok(self.ast.stmt_export_all_declaration(
                Span::new(start, end),
                exported,
                source,
                attributes,
                ImportOrExportKind::Value,
            ));
        }
        if matches!(
            self.current_token.tag,
            TokenTag::Var | TokenTag::Let | TokenTag::Const
        ) {
            let declaration = self.parse_variable_declaration(false)?;
            let end = self.eat_semicolon()?;
            declaration.set_span(self.ast, Span::new(declaration.span(self.ast).start, end));
            let specifiers = self.empty_range();
            let attributes = self.empty_range();
            return Ok(self.ast.stmt_export_named_declaration(
                Span::new(start, end),
                Some(Decl::VariableDeclaration(declaration)),
                specifiers,
                None,
                attributes,
                ImportOrExportKind::Value,
            ));
        }
        if self.current_token.tag == TokenTag::Function
            || (self.current_token.tag == TokenTag::Async && self.peek_tag()? == TokenTag::Function)
            || self.current_token.tag == TokenTag::Class
        {
            let statement = if self.current_token.tag == TokenTag::Class {
                self.parse_class_declaration()?
            } else {
                let async_prefix = self.current_token.tag == TokenTag::Async;
                self.parse_function_declaration(async_prefix)?
            };
            let StmtData::Declaration(declaration) = self.ast.stmt_data(statement) else {
                unreachable!()
            };
            let end = statement.span(self.ast).end;
            let specifiers = self.empty_range();
            let attributes = self.empty_range();
            return Ok(self.ast.stmt_export_named_declaration(
                Span::new(start, end),
                Some(declaration),
                specifiers,
                None,
                attributes,
                ImportOrExportKind::Value,
            ));
        }
        self.expect(TokenTag::LeftBrace)?;
        let mut specifiers = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace {
            let local = self.parse_module_export_name()?;
            let local_span = local.span(self.ast);
            let exported = if self.eat(TokenTag::As)? {
                self.parse_module_export_name()?
            } else {
                local
            };
            let span = Span::new(local_span.start, exported.span(self.ast).end);
            specifiers.push(self.ast.export_specifier(
                span,
                local,
                exported,
                ImportOrExportKind::Value,
            ));
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        self.expect(TokenTag::RightBrace)?;
        let source = if self.eat(TokenTag::From)? {
            Some(self.parse_string_literal_node()?)
        } else {
            None
        };
        let attributes = self.parse_import_attributes()?;
        let end = self.eat_semicolon()?;
        let specifiers = self.ast.add_typed_sub_range(specifiers);
        Ok(self.ast.stmt_export_named_declaration(
            Span::new(start, end),
            None,
            specifiers,
            source,
            attributes,
            ImportOrExportKind::Value,
        ))
    }

    fn parse_import_attributes(&mut self) -> ParseResult<TypedSubRange<ImportAttribute>> {
        if !self.eat(TokenTag::With)? {
            return Ok(self.empty_range());
        }
        self.expect(TokenTag::LeftBrace)?;
        let mut attributes = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace {
            let start = self.current_token.span.start;
            let key = self.parse_module_export_name()?;
            self.expect(TokenTag::Colon)?;
            let value = self.parse_string_literal_node()?;
            attributes.push(self.ast.import_attribute(
                Span::new(start, value.span(self.ast).end),
                key,
                value,
            ));
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        self.expect(TokenTag::RightBrace)?;
        Ok(self.ast.add_typed_sub_range(attributes))
    }

    fn parse_module_export_name(&mut self) -> ParseResult<ModuleExportName> {
        let token = self.current_token;
        if token.tag == TokenTag::StringLiteral {
            let raw = self.atom_from_span(token.span);
            let value = self.string_wtf8_atom(token)?;
            self.advance()?;
            return Ok(self
                .ast
                .module_export_name_string_literal(token.span, value, raw));
        }
        if token.tag.is_identifier_like() {
            let name = self.identifier_atom(token)?;
            self.advance()?;
            return Ok(self
                .ast
                .module_export_name_identifier_name(token.span, name));
        }
        self.error("expected an import or export name")
    }

    fn parse_ts_type_alias_declaration(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Type)?.span.start;
        let id = self.parse_binding_identifier()?;
        self.expect(TokenTag::Assign)?;
        let type_annotation = self.parse_ts_type()?;
        let end = self.eat_semicolon()?;
        let declaration = self.ast.ts_type_alias_declaration(
            Span::new(start, end),
            id,
            None,
            type_annotation,
            false,
        );
        Ok(Stmt::Declaration(Decl::TsTypeAliasDeclaration(declaration)))
    }

    fn parse_ts_type_annotation(&mut self) -> ParseResult<TsTypeAnnotation> {
        let start = self.expect(TokenTag::Colon)?.span.start;
        let type_annotation = self.parse_ts_type()?;
        let end = type_annotation.span(self.ast).end;
        Ok(self
            .ast
            .ts_type_annotation(Span::new(start, end), type_annotation))
    }

    fn parse_ts_type(&mut self) -> ParseResult<TsType> {
        let start = self.current_token.span.start;
        let mut types = vec![self.parse_ts_intersection_type()?];
        while self.eat(TokenTag::BitwiseOr)? {
            types.push(self.parse_ts_intersection_type()?);
        }
        if types.len() == 1 {
            return Ok(types[0]);
        }
        let end = types.last().unwrap().span(self.ast).end;
        let types = self.ast.add_typed_sub_range(types);
        Ok(TsType::TsUnionType(
            self.ast.ts_union_type(Span::new(start, end), types),
        ))
    }

    fn parse_ts_intersection_type(&mut self) -> ParseResult<TsType> {
        let start = self.current_token.span.start;
        let mut types = vec![self.parse_ts_postfix_type()?];
        while self.eat(TokenTag::BitwiseAnd)? {
            types.push(self.parse_ts_postfix_type()?);
        }
        if types.len() == 1 {
            return Ok(types[0]);
        }
        let end = types.last().unwrap().span(self.ast).end;
        let types = self.ast.add_typed_sub_range(types);
        Ok(TsType::TsIntersectionType(
            self.ast.ts_intersection_type(Span::new(start, end), types),
        ))
    }

    fn parse_ts_postfix_type(&mut self) -> ParseResult<TsType> {
        let mut type_annotation = self.parse_ts_primary_type()?;
        while self.current_token.tag == TokenTag::LeftBracket
            && self.peek_tag()? == TokenTag::RightBracket
        {
            let start = type_annotation.span(self.ast).start;
            self.advance()?;
            let end = self.expect(TokenTag::RightBracket)?.span.end;
            type_annotation = TsType::TsArrayType(
                self.ast
                    .ts_array_type(Span::new(start, end), type_annotation),
            );
        }
        Ok(type_annotation)
    }

    fn parse_ts_primary_type(&mut self) -> ParseResult<TsType> {
        let token = self.current_token;
        let keyword = match token.tag {
            TokenTag::Any => Some(self.ast.ts_type_ts_any_keyword(token.span)),
            TokenTag::Unknown => Some(self.ast.ts_type_ts_unknown_keyword(token.span)),
            TokenTag::Never => Some(self.ast.ts_type_ts_never_keyword(token.span)),
            TokenTag::Void => Some(self.ast.ts_type_ts_void_keyword(token.span)),
            TokenTag::NullLiteral => Some(self.ast.ts_type_ts_null_keyword(token.span)),
            TokenTag::Undefined => Some(self.ast.ts_type_ts_undefined_keyword(token.span)),
            TokenTag::String => Some(self.ast.ts_type_ts_string_keyword(token.span)),
            TokenTag::Number => Some(self.ast.ts_type_ts_number_keyword(token.span)),
            TokenTag::Bigint => Some(self.ast.ts_type_ts_big_int_keyword(token.span)),
            TokenTag::Boolean => Some(self.ast.ts_type_ts_boolean_keyword(token.span)),
            TokenTag::Symbol => Some(self.ast.ts_type_ts_symbol_keyword(token.span)),
            TokenTag::Object => Some(self.ast.ts_type_ts_object_keyword(token.span)),
            TokenTag::Intrinsic => Some(self.ast.ts_type_ts_intrinsic_keyword(token.span)),
            TokenTag::This => Some(self.ast.ts_type_ts_this_type(token.span)),
            _ => None,
        };
        if let Some(keyword) = keyword {
            self.advance()?;
            return Ok(keyword);
        }
        if self.eat(TokenTag::LeftParen)? {
            let start = token.span.start;
            let type_annotation = self.parse_ts_type()?;
            let end = self.expect(TokenTag::RightParen)?.span.end;
            return Ok(TsType::TsParenthesizedType(
                self.ast
                    .ts_parenthesized_type(Span::new(start, end), type_annotation),
            ));
        }
        if self.eat(TokenTag::LeftBracket)? {
            let start = token.span.start;
            let mut elements = Vec::new();
            while self.current_token.tag != TokenTag::RightBracket {
                elements.push(TsTupleElement::TsType(self.parse_ts_type()?));
                if !self.eat(TokenTag::Comma)? {
                    break;
                }
            }
            let end = self.expect(TokenTag::RightBracket)?.span.end;
            let elements = self.ast.add_typed_sub_range(elements);
            return Ok(TsType::TsTupleType(
                self.ast.ts_tuple_type(Span::new(start, end), elements),
            ));
        }
        if token.tag.is_identifier_like() {
            let name = self.identifier_atom(token)?;
            self.advance()?;
            let type_name = self
                .ast
                .ts_type_name_identifier_reference(token.span, name, None);
            let type_arguments = if self.current_token.tag == TokenTag::LessThan {
                Some(self.parse_ts_type_arguments()?)
            } else {
                None
            };
            let end = type_arguments.map_or(token.span.end, |args| args.span(self.ast).end);
            return Ok(self.ast.ts_type_ts_type_reference(
                Span::new(token.span.start, end),
                type_name,
                type_arguments,
            ));
        }
        self.error("expected a TypeScript type")
    }

    fn parse_ts_type_arguments(&mut self) -> ParseResult<TsTypeParameterInstantiation> {
        let start = self.expect(TokenTag::LessThan)?.span.start;
        let mut parameters = Vec::new();
        loop {
            parameters.push(self.parse_ts_type()?);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        let end = self.expect(TokenTag::GreaterThan)?.span.end;
        let parameters = self.ast.add_typed_sub_range(parameters);
        Ok(self
            .ast
            .ts_type_parameter_instantiation(Span::new(start, end), parameters))
    }

    fn parse_for_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::For)?.span.start;
        let is_for_await = if self.current_token.tag == TokenTag::Await {
            if !self.context.allow_await {
                return self.error("for await is only allowed in async contexts");
            }
            self.advance()?;
            true
        } else {
            false
        };
        self.expect(TokenTag::LeftParen)?;
        let old_allow_in = self.context.allow_in;
        self.context.allow_in = false;
        let let_starts_declaration = if self.current_token.tag == TokenTag::Let {
            let next = self.peek_tag()?;
            matches!(next, TokenTag::LeftBracket | TokenTag::LeftBrace)
                || (next.is_identifier_like() && !next.is_unconditionally_reserved())
        } else {
            false
        };
        let init = if matches!(
            self.current_token.tag,
            TokenTag::Var | TokenTag::Const | TokenTag::Using
        ) || let_starts_declaration
            || (self.current_token.tag == TokenTag::Await
                && self.peek_tag()? == TokenTag::Using
                && {
                    let binding = self.peek_second_token()?;
                    !binding.has_line_terminator_before() && binding.tag != TokenTag::LeftBracket
                }) {
            Some(ForStatementInit::VariableDeclaration(
                self.parse_variable_declaration(true)?,
            ))
        } else if self.current_token.tag == TokenTag::Semicolon {
            None
        } else {
            Some(ForStatementInit::Expr(
                self.parse_expression_with_precedence(Precedence::COMMA)?,
            ))
        };
        self.context.allow_in = old_allow_in;
        if matches!(self.current_token.tag, TokenTag::In | TokenTag::Of) {
            let operator = self.current_token.tag;
            if self.current_token.is_escaped() {
                return self.error_at(
                    self.current_token.span,
                    "for-in/of keywords may not contain escape sequences",
                );
            }
            if is_for_await && operator != TokenTag::Of {
                return self.error("for await requires 'of'");
            }
            let init = init.ok_or_else(|| {
                Error::new(
                    self.current_token.span,
                    "for-in/of requires a left-hand side",
                )
            })?;
            let invalid_async_for_of = operator == TokenTag::Of
                && !is_for_await
                && matches!(
                    self.ast.for_statement_init_data(init),
                    ForStatementInitData::Expr(expression)
                        if matches!(
                            self.ast.expr_data(expression),
                            ExprData::IdentifierReference(identifier)
                                if identifier.name(self.ast) == "async"
                                    && &self.source[identifier.span(self.ast).start as usize
                                        ..identifier.span(self.ast).end as usize] == b"async"
                        )
                );
            let left = match self.ast.for_statement_init_data(init) {
                ForStatementInitData::VariableDeclaration(declaration) => {
                    ForStatementLeft::VariableDeclaration(declaration)
                }
                ForStatementInitData::Expr(expression) => ForStatementLeft::AssignmentTarget(
                    self.expression_to_assignment_target(expression)?,
                ),
            };
            if operator == TokenTag::In
                && matches!(
                    self.ast.for_statement_left_data(left),
                    ForStatementLeftData::VariableDeclaration(declaration)
                        if matches!(
                            declaration.kind(self.ast),
                            VariableKind::Using | VariableKind::AwaitUsing
                        )
                )
            {
                return self.error_at(
                    left.span(self.ast),
                    "using declarations are not allowed in for-in statements",
                );
            }
            if invalid_async_for_of {
                return self.error_at(
                    left.span(self.ast),
                    "'async' may not be used as the left side of a for-of statement",
                );
            }
            self.advance()?;
            let right = if operator == TokenTag::Of {
                self.parse_assignment_expression_allow_in()?
            } else {
                self.parse_expression_allow_in()?
            };
            self.expect(TokenTag::RightParen)?;
            let body = self.parse_statement()?;
            let end = body.span(self.ast).end;
            return if operator == TokenTag::In {
                Ok(self
                    .ast
                    .stmt_for_in_statement(Span::new(start, end), left, right, body))
            } else {
                Ok(self.ast.stmt_for_of_statement(
                    Span::new(start, end),
                    left,
                    right,
                    body,
                    is_for_await,
                ))
            };
        }
        if is_for_await {
            return self.error("for await requires 'of'");
        }
        self.expect(TokenTag::Semicolon)?;
        let test = if self.current_token.tag == TokenTag::Semicolon {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenTag::Semicolon)?;
        let update = if self.current_token.tag == TokenTag::RightParen {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenTag::RightParen)?;
        let body = self.parse_statement()?;
        let end = body.span(self.ast).end;
        Ok(self
            .ast
            .stmt_for_statement(Span::new(start, end), init, test, update, body))
    }

    fn parse_switch_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Switch)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let discriminant = self.parse_expression()?;
        self.expect(TokenTag::RightParen)?;
        self.expect(TokenTag::LeftBrace)?;
        let mut cases = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace {
            let case_start = self.current_token.span.start;
            let test = if self.eat(TokenTag::Case)? {
                let test = self.parse_expression()?;
                self.expect(TokenTag::Colon)?;
                Some(test)
            } else {
                self.expect(TokenTag::Default)?;
                self.expect(TokenTag::Colon)?;
                None
            };
            let mut consequent = Vec::new();
            while !matches!(
                self.current_token.tag,
                TokenTag::Case | TokenTag::Default | TokenTag::RightBrace | TokenTag::Eof
            ) {
                consequent.push(self.parse_statement()?);
            }
            let case_end = consequent
                .last()
                .map_or(self.previous_end, |stmt| stmt.span(self.ast).end);
            let consequent = self.ast.add_typed_sub_range(consequent);
            cases.push(
                self.ast
                    .switch_case(Span::new(case_start, case_end), test, consequent),
            );
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        let cases = self.ast.add_typed_sub_range(cases);
        Ok(self
            .ast
            .stmt_switch_statement(Span::new(start, end), discriminant, cases))
    }

    fn parse_try_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.expect(TokenTag::Try)?.span.start;
        let block = self.parse_block()?;
        let handler = if self.eat(TokenTag::Catch)? {
            let catch_start = self.previous_end - 5;
            let param = if self.eat(TokenTag::LeftParen)? {
                let param = self.parse_binding_pattern(false)?;
                self.expect(TokenTag::RightParen)?;
                Some(param)
            } else {
                None
            };
            let body = self.parse_block()?;
            Some(self.ast.catch_clause(
                Span::new(catch_start, body.span(self.ast).end),
                param,
                body,
            ))
        } else {
            None
        };
        let finalizer = if self.eat(TokenTag::Finally)? {
            Some(self.parse_block()?)
        } else {
            None
        };
        if handler.is_none() && finalizer.is_none() {
            return self.error("try requires catch or finally");
        }
        let end = finalizer.map_or_else(
            || handler.map_or(block.span(self.ast).end, |node| node.span(self.ast).end),
            |node| node.span(self.ast).end,
        );
        Ok(self
            .ast
            .stmt_try_statement(Span::new(start, end), block, handler, finalizer))
    }

    fn parse_expression_with_precedence(&mut self, minimum: u8) -> ParseResult<Expr> {
        self.parse_expression_with_precedence_context(minimum, false)
    }

    fn parse_expression_with_precedence_context(
        &mut self,
        minimum: u8,
        in_cover: bool,
    ) -> ParseResult<Expr> {
        let mut left = self.parse_prefix_expression(in_cover)?;
        if minimum > Precedence::ASSIGNMENT
            && matches!(self.ast.expr_data(left), ExprData::YieldExpression(_))
        {
            return self.error_at(
                left.span(self.ast),
                "A yield expression is not valid in this expression position",
            );
        }
        loop {
            if matches!(self.ast.expr_data(left), ExprData::YieldExpression(_))
                && self.current_token.tag != TokenTag::Comma
            {
                break;
            }
            let token = self.current_token;
            if token.tag == TokenTag::In && !self.context.allow_in {
                break;
            }
            if matches!(
                token.tag,
                TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead
            ) {
                if expression_contains_optional_chain(left, self.ast) {
                    return self.error_at(
                        left.span(self.ast),
                        "An optional chain cannot be used as a tagged template tag",
                    );
                }
                left = self.parse_tagged_template_expression(left)?;
                continue;
            }
            let precedence = token.tag.precedence();
            if precedence < minimum || precedence == Precedence::LOWEST {
                break;
            }
            left = match token.tag {
                TokenTag::Dot | TokenTag::LeftBracket => self.parse_member_expression(left)?,
                TokenTag::OptionalChaining => {
                    self.advance()?;
                    if matches!(
                        self.current_token.tag,
                        TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead
                    ) {
                        return self
                            .error("An optional chain cannot be followed by a template literal");
                    }
                    if self.current_token.tag == TokenTag::LeftParen {
                        self.parse_call_expression(left, true)?
                    } else {
                        self.parse_member_expression_tail(left, true)?
                    }
                }
                TokenTag::LeftParen => self.parse_call_expression(left, false)?,
                TokenTag::Increment | TokenTag::Decrement
                    if !token.has_line_terminator_before() =>
                {
                    self.parse_update_expression(left, false)?
                }
                TokenTag::Question => self.parse_conditional_expression(left)?,
                tag if tag.is_assignment_operator() => {
                    self.parse_assignment_expression(left, tag)?
                }
                tag if tag.is_binary_operator() => self.parse_binary_expression(left, tag)?,
                tag if tag.is_logical_operator() => self.parse_logical_expression(left, tag)?,
                TokenTag::Comma => self.parse_sequence_expression(left)?,
                _ => break,
            };
        }
        Ok(left)
    }

    fn parse_prefix_expression(&mut self, in_cover: bool) -> ParseResult<Expr> {
        let token = self.current_token;
        match token.tag {
            TokenTag::Async if self.peek_tag()? == TokenTag::Function => {
                self.parse_function_expression(true)
            }
            TokenTag::Async if self.is_async_arrow_head()? => self.parse_async_arrow_expression(),
            TokenTag::Await if self.context.allow_await => self.parse_await_expression(),
            TokenTag::Yield if self.context.allow_yield => self.parse_yield_expression(),
            TokenTag::PrivateIdentifier => {
                let name = self.identifier_atom(token)?;
                let private = self.ast.private_identifier(token.span, name);
                self.advance()?;
                if self.current_token.tag != TokenTag::In {
                    return self.error_at(
                        token.span,
                        "Private names are only valid in property accesses or 'in' expressions",
                    );
                }
                Ok(Expr::PrivateIdentifier(private))
            }
            tag if tag.is_identifier_like() && !tag.is_unconditionally_reserved() => {
                let name = self.identifier_atom(token)?;
                self.validate_identifier(token, name.as_str(), "an identifier")?;
                self.advance()?;
                if self.current_token.tag == TokenTag::Arrow {
                    self.parse_single_parameter_arrow(token.span, name, false)
                } else {
                    Ok(self.ast.expr_identifier_reference(token.span, name, None))
                }
            }
            TokenTag::This => {
                self.advance()?;
                Ok(self.ast.expr_this_expression(token.span))
            }
            TokenTag::Super => {
                self.advance()?;
                Ok(self.ast.expr_super(token.span))
            }
            TokenTag::True | TokenTag::False => {
                if token.is_escaped() {
                    return self.error_at(token.span, "keywords may not contain escape sequences");
                }
                self.advance()?;
                Ok(self
                    .ast
                    .expr_boolean_literal(token.span, token.tag == TokenTag::True))
            }
            TokenTag::NullLiteral => {
                if token.is_escaped() {
                    return self.error_at(token.span, "keywords may not contain escape sequences");
                }
                self.advance()?;
                Ok(self.ast.expr_null_literal(token.span))
            }
            tag if tag.is_numeric_literal() => self.parse_numeric_literal(),
            TokenTag::StringLiteral => self.parse_string_literal(),
            TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                self.parse_template_literal(false)
            }
            TokenTag::Slash | TokenTag::SlashAssign => self.parse_regex_literal(),
            TokenTag::LeftParen => self.parse_parenthesized_or_arrow_expression(),
            TokenTag::LeftBracket => self.parse_array_expression(in_cover),
            TokenTag::LeftBrace => self.parse_object_expression(in_cover),
            TokenTag::Function => self.parse_function_expression(false),
            TokenTag::New => self.parse_new_expression(),
            TokenTag::Class => self.parse_class_expression(),
            TokenTag::At => {
                let (start, decorators) = self.parse_decorators()?;
                Ok(Expr::Class(
                    self.parse_class_with_decorators(false, start, decorators)?,
                ))
            }
            TokenTag::LessThan if self.options.lang.is_jsx() => self.parse_jsx_element(),
            TokenTag::Import => self.parse_import_expression(),
            TokenTag::Increment | TokenTag::Decrement => {
                self.advance()?;
                let argument = self.parse_expression_with_precedence(Precedence::NEW)?;
                self.parse_update_expression_from(start_of(token), argument, true, token.tag)
            }
            tag if tag.is_unary_operator() => self.parse_unary_expression(),
            _ => self.error("expected an expression"),
        }
    }

    fn parse_numeric_literal(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        let kind = match token.tag {
            TokenTag::NumericLiteral => NumericLiteralKind::Decimal,
            TokenTag::HexLiteral => NumericLiteralKind::Hex,
            TokenTag::OctalLiteral => NumericLiteralKind::Octal,
            TokenTag::BinaryLiteral => NumericLiteralKind::Binary,
            TokenTag::BigIntLiteral => {
                let raw = self.atom_from_span(token.span);
                self.advance()?;
                return Ok(self.ast.expr_big_int_literal(token.span, raw));
            }
            _ => unreachable!(),
        };
        let raw = self.atom_from_span(token.span);
        self.advance()?;
        Ok(self.ast.expr_numeric_literal(token.span, kind, raw))
    }

    fn parse_string_literal(&mut self) -> ParseResult<Expr> {
        Ok(Expr::StringLiteral(self.parse_string_literal_node()?))
    }

    fn parse_string_literal_node(&mut self) -> ParseResult<StringLiteral> {
        let token = self.current_token;
        if token.tag != TokenTag::StringLiteral {
            return self.error("expected a string literal");
        }
        let raw = self.atom_from_span(token.span);
        let value = self.string_wtf8_atom(token)?;
        self.advance()?;
        Ok(self.ast.string_literal(token.span, value, raw))
    }

    fn parse_regex_literal(&mut self) -> ParseResult<Expr> {
        let result = self
            .lexer
            .re_scan_as_regex(self.current_token.span.start)
            .map_err(|error| self.lexical_error(error))?;
        let pattern_text = str::from_utf8(
            &self.source[result.pattern.start as usize..result.pattern.end as usize],
        )
        .map_err(|_| Error::new(result.pattern, "regular expression pattern is not UTF-8"))?;
        let flags_text =
            str::from_utf8(&self.source[result.flags.start as usize..result.flags.end as usize])
                .map_err(|_| Error::new(result.flags, "regular expression flags are not UTF-8"))?;
        let allocator = oxc_allocator::Allocator::default();
        if let Err(error) = oxc_regular_expression::LiteralParser::new(
            &allocator,
            pattern_text,
            Some(flags_text),
            oxc_regular_expression::Options {
                pattern_span_offset: result.pattern.start,
                flags_span_offset: result.flags.start,
            },
        )
        .parse()
        {
            return self.error_at(result.span, error.to_string());
        }
        let pattern = self.atom_from_span(result.pattern);
        let flags = self.atom_from_span(result.flags);
        let expression = self.ast.expr_reg_exp_literal(result.span, pattern, flags);
        self.current_token = Token::eof(result.span.end);
        self.advance()?;
        Ok(expression)
    }

    fn parse_template_literal(&mut self, tagged: bool) -> ParseResult<Expr> {
        Ok(Expr::TemplateLiteral(
            self.parse_template_literal_node(tagged)?,
        ))
    }

    fn parse_template_literal_node(&mut self, tagged: bool) -> ParseResult<TemplateLiteral> {
        let start = self.current_token.span.start;
        let mut quasis = Vec::new();
        let mut expressions = Vec::new();
        if self.current_token.tag == TokenTag::NoSubstitutionTemplate {
            let token = self.current_token;
            quasis.push(self.add_template_element(token, true, tagged)?);
            self.advance()?;
            let quasis = self.ast.add_typed_sub_range(quasis);
            let expressions = self.ast.add_typed_sub_range(expressions);
            return Ok(self.ast.template_literal(token.span, quasis, expressions));
        }
        let head = self.current_token;
        quasis.push(self.add_template_element(head, false, tagged)?);
        self.advance()?;
        let end = loop {
            expressions.push(self.parse_expression()?);
            if self.current_token.tag != TokenTag::RightBrace {
                return self.error("expected '}' in template literal");
            }
            let continuation = self
                .lexer
                .re_scan_template_continuation(self.current_token.span.start)
                .map_err(|error| self.lexical_error(error))?;
            let tail = continuation.tag == TokenTag::TemplateTail;
            quasis.push(self.add_template_element(continuation, tail, tagged)?);
            let end = continuation.span.end;
            self.current_token = continuation;
            self.advance()?;
            if tail {
                break end;
            }
        };
        let quasis = self.ast.add_typed_sub_range(quasis);
        let expressions = self.ast.add_typed_sub_range(expressions);
        Ok(self
            .ast
            .template_literal(Span::new(start, end), quasis, expressions))
    }

    fn parse_tagged_template_expression(&mut self, tag: Expr) -> ParseResult<Expr> {
        let start = tag.span(self.ast).start;
        let quasi = self.parse_template_literal_node(true)?;
        Ok(self.ast.expr_tagged_template_expression(
            Span::new(start, quasi.span(self.ast).end),
            tag,
            quasi,
            None,
        ))
    }

    fn add_template_element(
        &mut self,
        token: Token,
        tail: bool,
        tagged: bool,
    ) -> ParseResult<TemplateElement> {
        let span = match token.tag {
            TokenTag::TemplateHead | TokenTag::TemplateMiddle => {
                Span::new(token.span.start + 1, token.span.end - 2)
            }
            TokenTag::TemplateTail | TokenTag::NoSubstitutionTemplate => {
                Span::new(token.span.start + 1, token.span.end - 1)
            }
            _ => unreachable!(),
        };
        let raw = self.atom_from_span(span);
        let is_cooked_undefined = token.has_invalid_escape();
        if is_cooked_undefined && !tagged {
            return self.error_at(span, "invalid escape sequence in template literal");
        }
        let cooked = if is_cooked_undefined {
            self.ast.empty_atom()
        } else if token.is_escaped() {
            let mut decoded = Vec::new();
            decode_string_escapes(
                &self.source[span.start as usize..span.end as usize],
                &mut decoded,
            );
            let value = str::from_utf8(&decoded)
                .map_err(|_| Error::new(span, "template value is not valid UTF-8"))?;
            self.ast.add_atom(value)
        } else {
            raw
        };
        Ok(self
            .ast
            .template_element(span, cooked, raw, tail, is_cooked_undefined))
    }

    fn parse_parenthesized_or_arrow_expression(&mut self) -> ParseResult<Expr> {
        self.parse_parenthesized_or_arrow_expression_with(None, false)
    }

    fn parse_parenthesized_or_arrow_expression_with(
        &mut self,
        arrow_start: Option<u32>,
        async_function: bool,
    ) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::LeftParen)?.span.start;
        let arrow_start = arrow_start.unwrap_or(start);
        let old_context = self.context;
        if async_function {
            self.context.allow_await = true;
        }
        if self.current_token.tag == TokenTag::RightParen {
            self.advance()?;
            if self.current_token.has_line_terminator_before() {
                return self.error("a line terminator is not allowed before '=>'");
            }
            self.expect(TokenTag::Arrow)?;
            self.context = old_context;
            return self.parse_arrow_body(arrow_start, Vec::new(), async_function);
        }
        let mut expressions = Vec::new();
        let mut rest_argument = None;
        let mut trailing_comma = false;
        loop {
            if self.eat(TokenTag::Spread)? {
                if rest_argument.is_some() {
                    return self.error("only one rest parameter is allowed");
                }
                rest_argument = Some(
                    self.parse_expression_with_precedence_context(Precedence::ASSIGNMENT, true)?,
                );
            } else {
                if rest_argument.is_some() {
                    return self.error("a rest parameter must be last");
                }
                expressions.push(
                    self.parse_expression_with_precedence_context(Precedence::ASSIGNMENT, true)?,
                );
            }
            if !self.eat(TokenTag::Comma)? {
                break;
            }
            if self.current_token.tag == TokenTag::RightParen {
                trailing_comma = true;
                break;
            }
        }
        let end = self.expect(TokenTag::RightParen)?.span.end;
        if self.current_token.tag == TokenTag::Arrow {
            if self.current_token.has_line_terminator_before() {
                return self.error("a line terminator is not allowed before '=>'");
            }
            self.advance()?;
            if trailing_comma && rest_argument.is_some() {
                return self.error("a rest parameter cannot have a trailing comma");
            }
            let parameters = expressions
                .into_iter()
                .map(|expression| self.expression_to_parameter(expression))
                .collect::<ParseResult<Vec<_>>>()?;
            let rest = if let Some(argument) = rest_argument {
                let argument = self.expression_to_binding_pattern(argument)?;
                if matches!(
                    self.ast.binding_pattern_data(argument),
                    BindingPatternData::AssignmentPattern(_)
                ) {
                    return self.error_at(
                        argument.span(self.ast),
                        "a rest parameter may not have an initializer",
                    );
                }
                let decorators = self.empty_range();
                Some(self.ast.binding_rest_element(
                    argument.span(self.ast),
                    argument,
                    decorators,
                    None,
                    false,
                ))
            } else {
                None
            };
            self.state.cover_has_init_name = false;
            self.context = old_context;
            return self.parse_arrow_body_with_rest(arrow_start, parameters, rest, async_function);
        }
        self.context = old_context;
        if rest_argument.is_some() {
            return self.error_at(Span::new(start, end), "spread is not allowed here");
        }
        if trailing_comma {
            return self.error_at(
                Span::new(start, end),
                "a parenthesized expression cannot have a trailing comma",
            );
        }
        let expression = if expressions.len() == 1 {
            expressions[0]
        } else {
            let expressions = self.ast.add_typed_sub_range(expressions);
            self.ast
                .expr_sequence_expression(Span::new(start + 1, end - 1), expressions)
        };
        if self.state.cover_has_init_name {
            self.state.cover_has_init_name = false;
            self.validate_no_cover_initialized_syntax(expression)?;
        }
        if self.options.preserve_parens {
            Ok(self
                .ast
                .expr_parenthesized_expression(Span::new(start, end), expression))
        } else {
            Ok(expression)
        }
    }

    fn parse_async_arrow_expression(&mut self) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::Async)?.span.start;
        if self.current_token.tag == TokenTag::LeftParen {
            return self.parse_parenthesized_or_arrow_expression_with(Some(start), true);
        }
        let token = self.current_token;
        let name = self.identifier_atom(token)?;
        if name == "await" {
            return self.error_at(
                token.span,
                "'await' is reserved in an async context and cannot be used as a binding identifier",
            );
        }
        self.advance()?;
        self.expect(TokenTag::Arrow)?;
        let decorators = self.empty_range();
        let pattern = self
            .ast
            .binding_pattern_binding_identifier(token.span, name, decorators, None, false, None);
        let parameter = self
            .ast
            .formal_parameter(token.span, FormalParameterPattern::BindingPattern(pattern));
        self.parse_arrow_body(
            start,
            vec![FormalParameterItem::FormalParameter(parameter)],
            true,
        )
    }

    fn parse_single_parameter_arrow(
        &mut self,
        span: Span,
        name: Atom<'allocator>,
        async_function: bool,
    ) -> ParseResult<Expr> {
        if self.current_token.has_line_terminator_before() {
            return self.error("a line terminator is not allowed before '=>'");
        }
        self.expect(TokenTag::Arrow)?;
        let decorators = self.empty_range();
        let pattern = self
            .ast
            .binding_pattern_binding_identifier(span, name, decorators, None, false, None);
        let parameter_pattern = FormalParameterPattern::BindingPattern(pattern);
        let parameter = self.ast.formal_parameter(span, parameter_pattern);
        self.parse_arrow_body(
            span.start,
            vec![FormalParameterItem::FormalParameter(parameter)],
            async_function,
        )
    }

    fn parse_arrow_body(
        &mut self,
        start: u32,
        parameters: Vec<FormalParameterItem>,
        async_function: bool,
    ) -> ParseResult<Expr> {
        self.parse_arrow_body_with_rest(start, parameters, None, async_function)
    }

    fn parse_arrow_body_with_rest(
        &mut self,
        start: u32,
        parameters: Vec<FormalParameterItem>,
        rest: Option<BindingRestElement>,
        async_function: bool,
    ) -> ParseResult<Expr> {
        let parameter_end = self.previous_end;
        let parameters = self.ast.add_typed_sub_range(parameters);
        let parameters = self.ast.formal_parameters(
            Span::new(start, parameter_end),
            parameters,
            rest,
            FormalParameterKind::ArrowFormalParameters,
        );
        let old_context = self.context;
        self.context.allow_return = true;
        self.context.allow_await = async_function;
        let (body, expression, end) = if self.current_token.tag == TokenTag::LeftBrace {
            let body = self.parse_function_body()?;
            let end = body.span(self.ast).end;
            (ArrowFunctionBody::FunctionBody(body), false, end)
        } else {
            let body = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
            let end = body.span(self.ast).end;
            (ArrowFunctionBody::Expr(body), true, end)
        };
        self.context = old_context;
        Ok(self.ast.expr_arrow_function_expression(
            Span::new(start, end),
            expression,
            async_function,
            parameters,
            body,
            None,
            None,
        ))
    }

    fn expression_to_parameter(&mut self, expression: Expr) -> ParseResult<FormalParameterItem> {
        let pattern = self.expression_to_binding_pattern(expression)?;
        let span = pattern.span(self.ast);
        let pattern = FormalParameterPattern::BindingPattern(pattern);
        let parameter = self.ast.formal_parameter(span, pattern);
        Ok(FormalParameterItem::FormalParameter(parameter))
    }

    fn expression_to_binding_pattern(&mut self, expression: Expr) -> ParseResult<BindingPattern> {
        self.expression_to_pattern(expression, PatternContext::Binding)
    }

    fn expression_to_assignment_pattern(
        &mut self,
        expression: Expr,
    ) -> ParseResult<BindingPattern> {
        self.expression_to_pattern(expression, PatternContext::Assignment)
    }

    fn expression_to_pattern(
        &mut self,
        expression: Expr,
        context: PatternContext,
    ) -> ParseResult<BindingPattern> {
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(identifier) if context == PatternContext::Assignment => {
                Ok(BindingPattern::SimpleAssignmentTarget(
                    SimpleAssignmentTarget::IdentifierReference(identifier),
                ))
            }
            ExprData::IdentifierReference(identifier) => {
                let decorators = self.empty_range();
                Ok(self.ast.binding_pattern_binding_identifier(
                    identifier.span(self.ast),
                    identifier.name(self.ast),
                    decorators,
                    None,
                    false,
                    None,
                ))
            }
            ExprData::MemberExpression(member)
                if context == PatternContext::Assignment
                    && !member_expression_is_optional(member, self.ast) =>
            {
                Ok(BindingPattern::SimpleAssignmentTarget(
                    SimpleAssignmentTarget::MemberExpression(member),
                ))
            }
            ExprData::AssignmentExpression(assignment)
                if assignment.operator(self.ast) == AssignmentOperator::Assign =>
            {
                let left = self.assignment_target_to_pattern(assignment.left(self.ast), context)?;
                let right = assignment.right(self.ast);
                let decorators = self.empty_range();
                Ok(self.ast.binding_pattern_assignment_pattern(
                    assignment.span(self.ast),
                    left,
                    right,
                    decorators,
                    None,
                    false,
                ))
            }
            ExprData::ArrayExpression(array) => {
                let source_elements = array.elements(self.ast);
                let source_elements = (0..source_elements.len())
                    .map(|index| source_elements.get_node(self.ast, index).unwrap())
                    .collect::<Vec<_>>();
                let source_elements_len = source_elements.len();
                let mut elements = Vec::new();
                let mut rest = None;
                for (index, argument) in source_elements.into_iter().enumerate() {
                    let Some(argument) = argument else {
                        elements.push(None);
                        continue;
                    };
                    match self.ast.argument_data(argument) {
                        ArgumentData::Expr(expression) => {
                            elements.push(Some(self.expression_to_pattern(expression, context)?));
                        }
                        ArgumentData::SpreadElement(spread) => {
                            if index + 1 != source_elements_len {
                                return self.error_at(
                                    spread.span(self.ast),
                                    "a rest element must be last",
                                );
                            }
                            if self.state.cover_has_trailing_comma
                                == Some(array.span(self.ast).start)
                            {
                                self.state.cover_has_trailing_comma = None;
                                return self.error_at(
                                    spread.span(self.ast),
                                    "a rest element may not have a trailing comma",
                                );
                            }
                            let argument =
                                self.expression_to_pattern(spread.argument(self.ast), context)?;
                            if matches!(
                                self.ast.binding_pattern_data(argument),
                                BindingPatternData::AssignmentPattern(_)
                            ) {
                                return self.error_at(
                                    argument.span(self.ast),
                                    "a rest element may not have an initializer",
                                );
                            }
                            let decorators = self.empty_range();
                            rest = Some(self.ast.binding_rest_element(
                                spread.span(self.ast),
                                argument,
                                decorators,
                                None,
                                false,
                            ));
                        }
                    }
                }
                let elements = self.ast.add_typed_sub_range(elements);
                let decorators = self.empty_range();
                Ok(self.ast.binding_pattern_array_pattern(
                    array.span(self.ast),
                    elements,
                    rest,
                    decorators,
                    None,
                    false,
                ))
            }
            ExprData::ObjectExpression(object) => {
                let source_properties = object.properties(self.ast);
                let source_properties = (0..source_properties.len())
                    .map(|index| source_properties.get_node(self.ast, index).unwrap())
                    .collect::<Vec<_>>();
                let source_properties_len = source_properties.len();
                let mut properties = Vec::new();
                let mut rest = None;
                for (index, property) in source_properties.into_iter().enumerate() {
                    match self.ast.object_property_kind_data(property) {
                        ObjectPropertyKindData::ObjectProperty(property) => {
                            if property.method(self.ast)
                                || property.kind(self.ast) != PropertyKind::Init
                            {
                                return self.error_at(
                                    property.span(self.ast),
                                    "methods are not allowed in patterns",
                                );
                            }
                            let value =
                                self.expression_to_pattern(property.value(self.ast), context)?;
                            properties.push(self.ast.binding_property(
                                property.span(self.ast),
                                property.key(self.ast),
                                value,
                                property.shorthand(self.ast),
                                property.computed(self.ast),
                            ));
                        }
                        ObjectPropertyKindData::SpreadElement(spread) => {
                            if index + 1 != source_properties_len {
                                return self.error_at(
                                    spread.span(self.ast),
                                    "a rest property must be last",
                                );
                            }
                            if self.state.cover_has_trailing_comma
                                == Some(object.span(self.ast).start)
                            {
                                self.state.cover_has_trailing_comma = None;
                                return self.error_at(
                                    spread.span(self.ast),
                                    "a rest property may not have a trailing comma",
                                );
                            }
                            let argument =
                                self.expression_to_pattern(spread.argument(self.ast), context)?;
                            let decorators = self.empty_range();
                            rest = Some(self.ast.binding_rest_element(
                                spread.span(self.ast),
                                argument,
                                decorators,
                                None,
                                false,
                            ));
                        }
                    }
                }
                let properties = self.ast.add_typed_sub_range(properties);
                let decorators = self.empty_range();
                Ok(self.ast.binding_pattern_object_pattern(
                    object.span(self.ast),
                    properties,
                    rest,
                    decorators,
                    None,
                    false,
                ))
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.expression_to_pattern(parenthesized.expression(self.ast), context)
            }
            _ => self.error_at(expression.span(self.ast), "invalid arrow parameter"),
        }
    }

    fn assignment_target_to_pattern(
        &mut self,
        target: AssignmentTarget,
        context: PatternContext,
    ) -> ParseResult<BindingPattern> {
        match self.ast.assignment_target_data(target) {
            AssignmentTargetData::SimpleAssignmentTarget(target) => {
                match self.ast.simple_assignment_target_data(target) {
                    SimpleAssignmentTargetData::IdentifierReference(_)
                        if context == PatternContext::Assignment =>
                    {
                        Ok(BindingPattern::SimpleAssignmentTarget(target))
                    }
                    SimpleAssignmentTargetData::IdentifierReference(identifier) => {
                        let decorators = self.empty_range();
                        Ok(self.ast.binding_pattern_binding_identifier(
                            identifier.span(self.ast),
                            identifier.name(self.ast),
                            decorators,
                            None,
                            false,
                            None,
                        ))
                    }
                    SimpleAssignmentTargetData::MemberExpression(_)
                        if context == PatternContext::Assignment =>
                    {
                        Ok(BindingPattern::SimpleAssignmentTarget(target))
                    }
                    _ => self.error_at(target.span(self.ast), "invalid binding target"),
                }
            }
            AssignmentTargetData::ArrayAssignmentTarget(pattern) => {
                Ok(BindingPattern::ArrayPattern(pattern))
            }
            AssignmentTargetData::ObjectAssignmentTarget(pattern) => {
                Ok(BindingPattern::ObjectPattern(pattern))
            }
        }
    }

    fn finish_cover_expression(&mut self, expression: Expr, in_cover: bool) -> ParseResult<Expr> {
        let needs_validation = !in_cover
            && self.state.cover_has_init_name
            && !matches!(
                self.current_token.tag,
                TokenTag::Assign | TokenTag::In | TokenTag::Of
            );
        if !in_cover {
            self.state.cover_has_init_name = false;
        }
        if needs_validation {
            self.validate_no_cover_initialized_syntax(expression)?;
        }
        Ok(expression)
    }

    fn validate_no_cover_initialized_syntax(&self, expression: Expr) -> ParseResult<()> {
        match self.ast.expr_data(expression) {
            ExprData::ObjectExpression(object) => {
                for id in object.properties(self.ast).iter() {
                    let property = self.ast.get_node_in_sub_range(id);
                    match self.ast.object_property_kind_data(property) {
                        ObjectPropertyKindData::ObjectProperty(property) => {
                            let value = property.value(self.ast);
                            if property.shorthand(self.ast)
                                && matches!(
                                    self.ast.expr_data(value),
                                    ExprData::AssignmentExpression(assignment)
                                        if assignment.operator(self.ast)
                                            == AssignmentOperator::Assign
                                )
                            {
                                return Err(Error::new(
                                    property.span(self.ast),
                                    "A shorthand property cannot have a default value in an object expression",
                                ));
                            }
                            self.validate_no_cover_initialized_syntax(value)?;
                        }
                        ObjectPropertyKindData::SpreadElement(spread) => {
                            self.validate_no_cover_initialized_syntax(spread.argument(self.ast))?;
                        }
                    }
                }
            }
            ExprData::ArrayExpression(array) => {
                for index in 0..array.elements(self.ast).len() {
                    let Some(argument) =
                        array.elements(self.ast).get_node(self.ast, index).unwrap()
                    else {
                        continue;
                    };
                    let expression = match self.ast.argument_data(argument) {
                        ArgumentData::Expr(expression) => expression,
                        ArgumentData::SpreadElement(spread) => spread.argument(self.ast),
                    };
                    self.validate_no_cover_initialized_syntax(expression)?;
                }
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.validate_no_cover_initialized_syntax(parenthesized.expression(self.ast))?;
            }
            ExprData::SequenceExpression(sequence) => {
                for index in 0..sequence.expressions(self.ast).len() {
                    let expression = sequence
                        .expressions(self.ast)
                        .get_node(self.ast, index)
                        .unwrap();
                    self.validate_no_cover_initialized_syntax(expression)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn parse_array_expression(&mut self, in_cover: bool) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::LeftBracket)?.span.start;
        let mut elements = Vec::new();
        while self.current_token.tag != TokenTag::RightBracket {
            if self.eat(TokenTag::Comma)? {
                elements.push(None);
                continue;
            }
            let argument = if self.eat(TokenTag::Spread)? {
                let spread_start = self.previous_end - 3;
                let expression = self.parse_assignment_expression_in_cover()?;
                let span = Span::new(spread_start, expression.span(self.ast).end);
                Argument::SpreadElement(self.ast.spread_element(span, expression))
            } else {
                Argument::Expr(self.parse_assignment_expression_in_cover()?)
            };
            elements.push(Some(argument));
            if !self.eat(TokenTag::Comma)? {
                break;
            }
            if self.current_token.tag == TokenTag::RightBracket {
                self.state.cover_has_trailing_comma = Some(start);
            }
        }
        let end = self.expect(TokenTag::RightBracket)?.span.end;
        let elements = self.ast.add_typed_sub_range(elements);
        let expression = self
            .ast
            .expr_array_expression(Span::new(start, end), elements);
        self.finish_cover_expression(expression, in_cover)
    }

    fn parse_object_expression(&mut self, in_cover: bool) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let mut properties = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace {
            if self.eat(TokenTag::Spread)? {
                let spread_start = self.previous_end - 3;
                let expression = self.parse_assignment_expression_in_cover()?;
                let spread = self.ast.spread_element(
                    Span::new(spread_start, expression.span(self.ast).end),
                    expression,
                );
                properties.push(ObjectPropertyKind::SpreadElement(spread));
            } else {
                properties.push(self.parse_object_property()?);
            }
            if !self.eat(TokenTag::Comma)? {
                break;
            }
            if self.current_token.tag == TokenTag::RightBrace {
                self.state.cover_has_trailing_comma = Some(start);
            }
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        let properties = self.ast.add_typed_sub_range(properties);
        let expression = self
            .ast
            .expr_object_expression(Span::new(start, end), properties);
        self.finish_cover_expression(expression, in_cover)
    }

    fn parse_object_property(&mut self) -> ParseResult<ObjectPropertyKind> {
        let start = self.current_token.span.start;
        let mut async_function = false;
        let mut generator = false;
        let mut kind = PropertyKind::Init;
        let mut key = None;

        if self.current_token.tag == TokenTag::Async {
            let token = self.current_token;
            self.advance()?;
            if !self.current_token.has_line_terminator_before()
                && Self::is_property_key_start(self.current_token.tag)
            {
                if token.is_escaped() {
                    return self.error_at(token.span, "keywords may not contain escape sequences");
                }
                async_function = true;
            } else {
                let name = self.identifier_atom(token)?;
                key = Some(self.ast.property_key_identifier_name(token.span, name));
            }
        }

        if key.is_none() && self.eat(TokenTag::Star)? {
            generator = true;
        }

        if key.is_none()
            && !async_function
            && !generator
            && matches!(self.current_token.tag, TokenTag::Get | TokenTag::Set)
        {
            let token = self.current_token;
            self.advance()?;
            if !self.current_token.has_line_terminator_before()
                && Self::is_property_key_start(self.current_token.tag)
            {
                if token.is_escaped() {
                    return self.error_at(token.span, "keywords may not contain escape sequences");
                }
                kind = if token.tag == TokenTag::Get {
                    PropertyKind::Get
                } else {
                    PropertyKind::Set
                };
            } else {
                let name = self.identifier_atom(token)?;
                key = Some(self.ast.property_key_identifier_name(token.span, name));
            }
        }

        let computed = self.current_token.tag == TokenTag::LeftBracket && key.is_none();
        let key = match key {
            Some(key) => key,
            None => self.parse_property_key()?,
        };
        if matches!(
            self.ast.property_key_data(key),
            PropertyKeyData::PrivateIdentifier(_)
        ) {
            return self.error_at(
                key.span(self.ast),
                "private names are not allowed in object literals",
            );
        }

        if self.current_token.tag == TokenTag::LeftParen {
            return self.parse_object_method(start, key, computed, kind, async_function, generator);
        }

        if async_function || generator || kind != PropertyKind::Init {
            return self.error("expected '(' after method name");
        }

        let (value, shorthand) = if self.eat(TokenTag::Colon)? {
            (self.parse_assignment_expression_in_cover()?, false)
        } else {
            let PropertyKeyData::IdentifierName(identifier) = self.ast.property_key_data(key)
            else {
                return self.error_at(key.span(self.ast), "expected ':' after property name");
            };
            if is_unconditionally_reserved_word(identifier.name(self.ast).as_str()) {
                return self.error_at(
                    identifier.span(self.ast),
                    "reserved words may not be used as shorthand properties",
                );
            }
            self.validate_identifier_name(
                identifier.span(self.ast),
                identifier.name(self.ast).as_str(),
                "an identifier",
            )?;
            let identifier = self.ast.expr_identifier_reference(
                identifier.span(self.ast),
                identifier.name(self.ast),
                None,
            );
            let value = if self.eat(TokenTag::Assign)? {
                self.state.cover_has_init_name = true;
                let right = self.parse_assignment_expression_in_cover()?;
                let target = self.expression_to_assignment_target(identifier)?;
                self.ast.expr_assignment_expression(
                    Span::new(identifier.span(self.ast).start, right.span(self.ast).end),
                    target,
                    right,
                    AssignmentOperator::Assign,
                )
            } else {
                identifier
            };
            (value, true)
        };
        let span = Span::new(start, value.span(self.ast).end);
        let property = self.ast.object_property(
            span,
            key,
            value,
            PropertyKind::Init,
            false,
            shorthand,
            computed,
        );
        Ok(ObjectPropertyKind::ObjectProperty(property))
    }

    fn parse_object_method(
        &mut self,
        start: u32,
        key: PropertyKey,
        computed: bool,
        kind: PropertyKind,
        async_function: bool,
        generator: bool,
    ) -> ParseResult<ObjectPropertyKind> {
        let old_context = self.context;
        self.context.allow_return = true;
        self.context.allow_await = async_function;
        self.context.allow_yield = generator;
        let result = (|| {
            let params =
                self.parse_formal_parameters_kind(FormalParameterKind::UniqueFormalParameters)?;
            self.check_accessor_arity(kind, params)?;
            let return_type =
                if self.options.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                    Some(self.parse_ts_type_annotation()?)
                } else {
                    None
                };
            let body = self.parse_function_body()?;
            Ok((params, return_type, body))
        })();
        self.context = old_context;
        let (params, return_type, body) = result?;
        let span = Span::new(start, body.span(self.ast).end);
        let function = self.ast.function(
            span,
            FunctionType::FunctionExpression,
            None,
            generator,
            async_function,
            false,
            params,
            Some(body),
            None,
            return_type,
        );
        let value = Expr::Function(function);
        Ok(ObjectPropertyKind::ObjectProperty(
            self.ast.object_property(
                span,
                key,
                value,
                kind,
                kind == PropertyKind::Init,
                false,
                computed,
            ),
        ))
    }

    fn is_property_key_start(tag: TokenTag) -> bool {
        matches!(
            tag,
            TokenTag::Star
                | TokenTag::LeftBracket
                | TokenTag::StringLiteral
                | TokenTag::PrivateIdentifier
        ) || tag.is_numeric_literal()
            || tag.is_identifier_like()
    }

    fn parse_property_key(&mut self) -> ParseResult<PropertyKey> {
        let token = self.current_token;
        match token.tag {
            TokenTag::LeftBracket => {
                self.advance()?;
                let expression = self.parse_assignment_expression_allow_in()?;
                self.expect(TokenTag::RightBracket)?;
                Ok(PropertyKey::Expr(expression))
            }
            TokenTag::PrivateIdentifier => {
                let name = self.identifier_atom(token)?;
                self.advance()?;
                Ok(self.ast.property_key_private_identifier(token.span, name))
            }
            TokenTag::StringLiteral => {
                let raw = self.atom_from_span(token.span);
                let value = self.string_wtf8_atom(token)?;
                self.advance()?;
                Ok(self.ast.property_key_string_literal(token.span, value, raw))
            }
            tag if tag.is_numeric_literal() => {
                let kind = match tag {
                    TokenTag::HexLiteral => NumericLiteralKind::Hex,
                    TokenTag::OctalLiteral => NumericLiteralKind::Octal,
                    TokenTag::BinaryLiteral => NumericLiteralKind::Binary,
                    _ => NumericLiteralKind::Decimal,
                };
                let raw = self.atom_from_span(token.span);
                self.advance()?;
                Ok(self.ast.property_key_numeric_literal(token.span, kind, raw))
            }
            tag if tag.is_identifier_like() => {
                let name = self.identifier_atom(token)?;
                self.advance()?;
                Ok(self.ast.property_key_identifier_name(token.span, name))
            }
            _ => self.error("expected a property name"),
        }
    }

    fn parse_member_expression(&mut self, object: Expr) -> ParseResult<Expr> {
        self.parse_member_expression_tail(object, false)
    }

    fn parse_member_expression_tail(&mut self, object: Expr, optional: bool) -> ParseResult<Expr> {
        let start = object.span(self.ast).start;
        let (property, computed, end) = if self.eat(TokenTag::LeftBracket)? {
            let expression = self.parse_expression_allow_in()?;
            let property = PropertyKey::Expr(expression);
            let end = self.expect(TokenTag::RightBracket)?.span.end;
            (property, true, end)
        } else {
            if !optional {
                self.expect(TokenTag::Dot)?;
            }
            let token = self.current_token;
            if !token.tag.is_identifier_like() && token.tag != TokenTag::PrivateIdentifier {
                return self.error("expected an identifier name after '.'");
            }
            let name = self.identifier_atom(token)?;
            self.advance()?;
            let property = if token.tag == TokenTag::PrivateIdentifier {
                self.ast.property_key_private_identifier(token.span, name)
            } else {
                self.ast.property_key_identifier_name(token.span, name)
            };
            (property, false, token.span.end)
        };
        Ok(self.ast.expr_member_expression(
            Span::new(start, end),
            object,
            property,
            computed,
            optional,
        ))
    }

    fn parse_call_expression(&mut self, callee: Expr, optional: bool) -> ParseResult<Expr> {
        let start = callee.span(self.ast).start;
        let arguments = self.parse_arguments()?;
        let end = self.previous_end;
        Ok(self
            .ast
            .expr_call_expression(Span::new(start, end), callee, arguments, optional, None))
    }

    fn parse_arguments(&mut self) -> ParseResult<TypedSubRange<Argument>> {
        self.expect(TokenTag::LeftParen)?;
        let mut arguments = Vec::new();
        while self.current_token.tag != TokenTag::RightParen {
            let argument =
                if self.eat(TokenTag::Spread)? {
                    let start = self.previous_end - 3;
                    let expression = self.parse_assignment_expression_allow_in()?;
                    Argument::SpreadElement(self.ast.spread_element(
                        Span::new(start, expression.span(self.ast).end),
                        expression,
                    ))
                } else {
                    Argument::Expr(self.parse_assignment_expression_allow_in()?)
                };
            arguments.push(argument);
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        self.expect(TokenTag::RightParen)?;
        Ok(self.ast.add_typed_sub_range(arguments))
    }

    fn parse_new_expression(&mut self) -> ParseResult<Expr> {
        if self.peek_tag()? == TokenTag::Dot {
            return self.parse_meta_property();
        }
        let start = self.expect(TokenTag::New)?.span.start;
        let mut callee = self.parse_prefix_expression(false)?;
        while matches!(
            self.current_token.tag,
            TokenTag::Dot | TokenTag::LeftBracket | TokenTag::OptionalChaining
        ) {
            if self.current_token.tag == TokenTag::OptionalChaining {
                self.advance()?;
                if self.current_token.tag == TokenTag::LeftParen {
                    break;
                }
                callee = self.parse_member_expression_tail(callee, true)?;
            } else {
                callee = self.parse_member_expression(callee)?;
            }
        }
        if matches!(self.ast.expr_data(callee), ExprData::ImportExpression(_)) {
            return self.error_at(callee.span(self.ast), "import() cannot be used with new");
        }
        let arguments = if self.current_token.tag == TokenTag::LeftParen {
            self.parse_arguments()?
        } else {
            self.empty_range()
        };
        let end = self.previous_end.max(callee.span(self.ast).end);
        Ok(self
            .ast
            .expr_new_expression(Span::new(start, end), callee, arguments, None))
    }

    fn parse_import_expression(&mut self) -> ParseResult<Expr> {
        let import_token = self.expect(TokenTag::Import)?;
        let start = import_token.span.start;
        let phase = if self.eat(TokenTag::Dot)? {
            let property = self.current_token;
            if property.tag == TokenTag::Identifier
                && self
                    .source
                    .get(property.span.start as usize..property.span.end as usize)
                    == Some(b"meta")
            {
                let meta_name = self.identifier_atom(import_token)?;
                self.advance()?;
                let property_name = self.identifier_atom(property)?;
                let meta = self.ast.identifier_name(import_token.span, meta_name);
                let property = self.ast.identifier_name(property.span, property_name);
                return Ok(self.ast.expr_meta_property(
                    Span::new(start, property.span(self.ast).end),
                    meta,
                    property,
                ));
            }
            let phase = match property.tag {
                TokenTag::Source => ImportPhase::Source,
                TokenTag::Defer => ImportPhase::Defer,
                _ => return self.error("expected 'meta', 'source', or 'defer' after import."),
            };
            self.advance()?;
            Some(phase)
        } else {
            None
        };
        self.expect(TokenTag::LeftParen)?;
        let source = self.parse_assignment_expression_allow_in()?;
        let options = if phase.is_none() && self.eat(TokenTag::Comma)? {
            let options = if self.current_token.tag == TokenTag::RightParen {
                None
            } else {
                Some(self.parse_assignment_expression_allow_in()?)
            };
            self.eat(TokenTag::Comma)?;
            options
        } else {
            None
        };
        let end = self.expect(TokenTag::RightParen)?.span.end;
        Ok(self
            .ast
            .expr_import_expression(Span::new(start, end), source, options, phase))
    }

    fn parse_meta_property(&mut self) -> ParseResult<Expr> {
        let meta_token = self.current_token;
        if meta_token.is_escaped() {
            return self.error_at(
                meta_token.span,
                "meta-property keywords may not contain escape sequences",
            );
        }
        let meta_name = self.identifier_atom(meta_token)?;
        self.advance()?;
        self.expect(TokenTag::Dot)?;
        let property_token = self.current_token;
        if !property_token.tag.is_identifier_like() {
            return self.error("expected a meta-property name");
        }
        if property_token.is_escaped() {
            return self.error_at(
                property_token.span,
                "meta-property names may not contain escape sequences",
            );
        }
        let property_name = self.identifier_atom(property_token)?;
        self.advance()?;
        let meta = self.ast.identifier_name(meta_token.span, meta_name);
        let property = self.ast.identifier_name(property_token.span, property_name);
        Ok(self.ast.expr_meta_property(
            Span::new(meta_token.span.start, property_token.span.end),
            meta,
            property,
        ))
    }

    fn parse_jsx_element(&mut self) -> ParseResult<Expr> {
        let start = self.current_token.span.start;
        self.lexer.mode = LexerMode::JsxTag;
        self.advance()?;
        if self.current_token.tag == TokenTag::GreaterThan {
            return self.parse_jsx_fragment(start);
        }
        let opening_name = self.parse_jsx_element_name()?;
        let opening_name_span = opening_name.span(self.ast);
        let mut attributes = Vec::new();
        while matches!(
            self.current_token.tag,
            TokenTag::JsxIdentifier | TokenTag::LeftBrace
        ) {
            attributes.push(self.parse_jsx_attribute()?);
        }
        let self_closing = self.eat(TokenTag::Slash)?;
        let opening_end = if self.current_token.tag == TokenTag::GreaterThan {
            self.current_token.span.end
        } else {
            return self.error("expected '>' to close JSX opening element");
        };
        let attributes = self.ast.add_typed_sub_range(attributes);
        let opening = self.ast.jsx_opening_element(
            Span::new(start, opening_end),
            opening_name,
            attributes,
            self_closing,
            None,
        );
        if self_closing {
            self.lexer.mode = LexerMode::Normal;
            self.advance()?;
            let children = self.empty_range();
            return Ok(Expr::JsxElement(self.ast.jsx_element(
                Span::new(start, opening_end),
                opening,
                children,
                None,
            )));
        }
        let children = self.parse_jsx_children(opening_end)?;
        let closing_start = self.current_token.span.start;
        self.lexer.mode = LexerMode::JsxTag;
        self.advance()?;
        self.expect(TokenTag::Slash)?;
        let closing_name = self.parse_jsx_element_name()?;
        let closing_name_span = closing_name.span(self.ast);
        if self.source[opening_name_span.start as usize..opening_name_span.end as usize]
            != self.source[closing_name_span.start as usize..closing_name_span.end as usize]
        {
            return self.error_at(closing_name_span, "JSX closing tag does not match");
        }
        let closing_end = if self.current_token.tag == TokenTag::GreaterThan {
            self.current_token.span.end
        } else {
            return self.error("expected '>' to close JSX closing element");
        };
        let closing = self
            .ast
            .jsx_closing_element(Span::new(closing_start, closing_end), closing_name);
        self.lexer.mode = LexerMode::Normal;
        self.advance()?;
        Ok(Expr::JsxElement(self.ast.jsx_element(
            Span::new(start, closing_end),
            opening,
            children,
            Some(closing),
        )))
    }

    fn parse_jsx_fragment(&mut self, start: u32) -> ParseResult<Expr> {
        let opening_end = self.current_token.span.end;
        let opening = self.ast.jsx_opening_fragment(Span::new(start, opening_end));
        let children = self.parse_jsx_children(opening_end)?;
        let closing_start = self.current_token.span.start;
        self.lexer.mode = LexerMode::JsxTag;
        self.advance()?;
        self.expect(TokenTag::Slash)?;
        if self.current_token.tag != TokenTag::GreaterThan {
            return self.error("expected '>' to close JSX fragment");
        }
        let closing_end = self.current_token.span.end;
        let closing = self
            .ast
            .jsx_closing_fragment(Span::new(closing_start, closing_end));
        self.lexer.mode = LexerMode::Normal;
        self.advance()?;
        Ok(self
            .ast
            .expr_jsx_fragment(Span::new(start, closing_end), opening, children, closing))
    }

    fn parse_jsx_children(&mut self, mut scan_from: u32) -> ParseResult<TypedSubRange<JsxChild>> {
        let mut children = Vec::new();
        self.lexer.mode = LexerMode::Normal;
        loop {
            let text = self.lexer.re_scan_jsx_text(scan_from);
            if text.span.start < text.span.end {
                let bytes = &self.source[text.span.start as usize..text.span.end as usize];
                let value = Wtf8::from_bytes(bytes)
                    .map_err(|_| Error::new(text.span, "JSX text is not valid WTF-8"))?;
                children.push(
                    self.ast
                        .jsx_child_jsx_text(text.span, self.ast.add_wtf8_atom(value)),
                );
            }
            self.current_token = text;
            self.advance()?;
            match self.current_token.tag {
                TokenTag::LessThan => {
                    if self.peek_tag()? == TokenTag::Slash {
                        break;
                    }
                    let child = self.parse_jsx_element()?;
                    scan_from = child.span(self.ast).end;
                    match self.ast.expr_data(child) {
                        ExprData::JsxElement(element) => {
                            children.push(JsxChild::JsxElement(element));
                        }
                        ExprData::JsxFragment(fragment) => {
                            children.push(JsxChild::JsxFragment(fragment));
                        }
                        _ => unreachable!(),
                    }
                }
                TokenTag::LeftBrace => {
                    let child = self.parse_jsx_expression_container_child()?;
                    scan_from = child.span(self.ast).end;
                    children.push(child);
                }
                _ => return self.error("expected a JSX child or closing tag"),
            }
        }
        Ok(self.ast.add_typed_sub_range(children))
    }

    fn parse_jsx_expression_container_child(&mut self) -> ParseResult<JsxChild> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        if self.current_token.tag == TokenTag::RightBrace {
            let empty_span = Span::new(start + 1, self.current_token.span.start);
            let empty = self.ast.jsx_empty_expression(empty_span);
            let expression = JsxExpression::JsxEmptyExpression(empty);
            let end = self.current_token.span.end;
            self.advance()?;
            return Ok(self
                .ast
                .jsx_child_jsx_expression_container(Span::new(start, end), expression));
        }
        if self.eat(TokenTag::Spread)? {
            let expression = self.parse_expression()?;
            let end = self.current_token.span.end;
            self.expect(TokenTag::RightBrace)?;
            return Ok(self
                .ast
                .jsx_child_jsx_spread_child(Span::new(start, end), expression));
        }
        let expression = self.parse_expression()?;
        let end = self.current_token.span.end;
        self.expect(TokenTag::RightBrace)?;
        Ok(self.ast.jsx_child_jsx_expression_container(
            Span::new(start, end),
            JsxExpression::Expr(expression),
        ))
    }

    fn parse_jsx_element_name(&mut self) -> ParseResult<JsxElementName> {
        let token = self.current_token;
        if !matches!(token.tag, TokenTag::JsxIdentifier | TokenTag::Identifier)
            && !token.tag.is_identifier_like()
        {
            return self.error("expected a JSX element name");
        }
        let name = self.identifier_atom(token)?;
        self.advance()?;
        Ok(JsxElementName::JsxIdentifier(
            self.ast.jsx_identifier(token.span, name),
        ))
    }

    fn parse_jsx_attribute(&mut self) -> ParseResult<JsxAttributeItem> {
        if self.current_token.tag == TokenTag::LeftBrace {
            let start = self.expect(TokenTag::LeftBrace)?.span.start;
            self.lexer.mode = LexerMode::Normal;
            self.expect(TokenTag::Spread)?;
            let argument = self.parse_expression()?;
            if self.current_token.tag != TokenTag::RightBrace {
                return self.error("expected '}' after JSX spread attribute");
            }
            let end = self.current_token.span.end;
            self.lexer.mode = LexerMode::JsxTag;
            self.advance()?;
            return Ok(JsxAttributeItem::JsxSpreadAttribute(
                self.ast
                    .jsx_spread_attribute(Span::new(start, end), argument),
            ));
        }
        let token = self.current_token;
        let name = self.identifier_atom(token)?;
        self.advance()?;
        let name = JsxAttributeName::JsxIdentifier(self.ast.jsx_identifier(token.span, name));
        let value = if self.eat(TokenTag::Assign)? {
            match self.current_token.tag {
                TokenTag::StringLiteral => Some(JsxAttributeValue::StringLiteral(
                    self.parse_string_literal_node()?,
                )),
                TokenTag::LeftBrace => {
                    let start = self.current_token.span.start;
                    self.lexer.mode = LexerMode::Normal;
                    self.advance()?;
                    let expression = self.parse_expression()?;
                    if self.current_token.tag != TokenTag::RightBrace {
                        return self.error("expected '}' after JSX attribute expression");
                    }
                    let end = self.current_token.span.end;
                    self.lexer.mode = LexerMode::JsxTag;
                    self.advance()?;
                    let container = self.ast.jsx_expression_container(
                        Span::new(start, end),
                        JsxExpression::Expr(expression),
                    );
                    Some(JsxAttributeValue::JsxExpressionContainer(container))
                }
                _ => return self.error("expected a JSX attribute value"),
            }
        } else {
            None
        };
        let end = value.map_or(token.span.end, |value| value.span(self.ast).end);
        Ok(JsxAttributeItem::JsxAttribute(self.ast.jsx_attribute(
            Span::new(token.span.start, end),
            name,
            value,
        )))
    }

    fn parse_await_expression(&mut self) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::Await)?.span.start;
        let argument = self.parse_expression_with_precedence(Precedence::UNARY)?;
        Ok(self
            .ast
            .expr_await_expression(Span::new(start, argument.span(self.ast).end), argument))
    }

    fn parse_yield_expression(&mut self) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::Yield)?.span.start;
        let delegate = if self.current_token.tag == TokenTag::Star
            && !self.current_token.has_line_terminator_before()
        {
            self.advance()?;
            true
        } else {
            false
        };
        let argument = if self.current_token.has_line_terminator_before()
            || matches!(
                self.current_token.tag,
                TokenTag::Semicolon
                    | TokenTag::RightBrace
                    | TokenTag::RightParen
                    | TokenTag::RightBracket
                    | TokenTag::Colon
                    | TokenTag::Comma
                    | TokenTag::Eof
            ) {
            None
        } else {
            Some(self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?)
        };
        let end = argument.map_or(self.previous_end, |expr| expr.span(self.ast).end);
        Ok(self
            .ast
            .expr_yield_expression(Span::new(start, end), argument, delegate))
    }

    fn parse_unary_expression(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        self.advance()?;
        let argument = self.parse_expression_with_precedence(Precedence::UNARY)?;
        let operator = match token.tag {
            TokenTag::Minus => UnaryOperator::Negate,
            TokenTag::Plus => UnaryOperator::Positive,
            TokenTag::LogicalNot => UnaryOperator::LogicalNot,
            TokenTag::BitwiseNot => UnaryOperator::BitwiseNot,
            TokenTag::Typeof => UnaryOperator::Typeof,
            TokenTag::Void => UnaryOperator::Void,
            TokenTag::Delete => UnaryOperator::Delete,
            _ => unreachable!(),
        };
        Ok(self.ast.expr_unary_expression(
            Span::new(token.span.start, argument.span(self.ast).end),
            argument,
            operator,
        ))
    }

    fn parse_update_expression(&mut self, argument: Expr, prefix: bool) -> ParseResult<Expr> {
        let token = self.current_token;
        self.advance()?;
        self.parse_update_expression_from(
            argument.span(self.ast).start,
            argument,
            prefix,
            token.tag,
        )
    }

    fn parse_update_expression_from(
        &mut self,
        start: u32,
        argument: Expr,
        prefix: bool,
        tag: TokenTag,
    ) -> ParseResult<Expr> {
        let target = self.expression_to_simple_assignment_target(argument)?;
        let operator = if tag == TokenTag::Increment {
            UpdateOperator::Increment
        } else {
            UpdateOperator::Decrement
        };
        let end = if prefix {
            argument.span(self.ast).end
        } else {
            self.previous_end
        };
        Ok(self
            .ast
            .expr_update_expression(Span::new(start, end), target, operator, prefix))
    }

    fn parse_binary_expression(&mut self, left: Expr, tag: TokenTag) -> ParseResult<Expr> {
        let start = left.span(self.ast).start;
        let precedence = tag.precedence();
        if tag == TokenTag::Exponent
            && matches!(self.ast.expr_data(left), ExprData::UnaryExpression(_))
        {
            return self.error_at(
                left.span(self.ast),
                "A unary expression cannot be the left operand of exponentiation",
            );
        }
        self.advance()?;
        let right = self.parse_expression_with_precedence(if tag == TokenTag::Exponent {
            precedence
        } else {
            precedence + 1
        })?;
        if matches!(
            self.ast.expr_data(right),
            ExprData::PrivateIdentifier(_) | ExprData::ArrowFunctionExpression(_)
        ) {
            return self.error_at(
                right.span(self.ast),
                "Private names are only valid on the left of an 'in' expression",
            );
        }
        let operator = binary_operator(tag);
        Ok(self.ast.expr_binary_expression(
            Span::new(start, right.span(self.ast).end),
            left,
            right,
            operator,
        ))
    }

    fn parse_logical_expression(&mut self, left: Expr, tag: TokenTag) -> ParseResult<Expr> {
        let start = left.span(self.ast).start;
        let precedence = tag.precedence();
        self.advance()?;
        let right = self.parse_expression_with_precedence(precedence + 1)?;
        let operator = match tag {
            TokenTag::LogicalAnd => LogicalOperator::And,
            TokenTag::LogicalOr => LogicalOperator::Or,
            TokenTag::NullishCoalescing => LogicalOperator::NullishCoalescing,
            _ => unreachable!(),
        };
        if logical_expression_mixes_coalesce(left, operator, self.ast)
            || logical_expression_mixes_coalesce(right, operator, self.ast)
        {
            return self.error_at(
                Span::new(start, right.span(self.ast).end),
                "Nullish coalescing cannot be mixed with logical AND or OR without parentheses",
            );
        }
        Ok(self.ast.expr_logical_expression(
            Span::new(start, right.span(self.ast).end),
            left,
            right,
            operator,
        ))
    }

    fn parse_assignment_expression(&mut self, left: Expr, tag: TokenTag) -> ParseResult<Expr> {
        let start = left.span(self.ast).start;
        let target = self.expression_to_assignment_target(left)?;
        if tag != TokenTag::Assign
            && !matches!(
                self.ast.assignment_target_data(target),
                AssignmentTargetData::SimpleAssignmentTarget(_)
            )
        {
            return self.error_at(
                target.span(self.ast),
                "A compound assignment requires a simple assignment target",
            );
        }
        if matches!(
            tag,
            TokenTag::LogicalAndAssign | TokenTag::LogicalOrAssign | TokenTag::NullishAssign
        ) && assignment_target_is_call_expression(target, self.ast)
        {
            return self.error_at(
                target.span(self.ast),
                "A call expression cannot be used as a logical assignment target",
            );
        }
        self.advance()?;
        let right = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
        Ok(self.ast.expr_assignment_expression(
            Span::new(start, right.span(self.ast).end),
            target,
            right,
            assignment_operator(tag),
        ))
    }

    fn parse_conditional_expression(&mut self, test: Expr) -> ParseResult<Expr> {
        let start = test.span(self.ast).start;
        self.expect(TokenTag::Question)?;
        let consequent = self.parse_assignment_expression_allow_in()?;
        self.expect(TokenTag::Colon)?;
        let alternate = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
        Ok(self.ast.expr_conditional_expression(
            Span::new(start, alternate.span(self.ast).end),
            test,
            consequent,
            alternate,
        ))
    }

    fn parse_sequence_expression(&mut self, left: Expr) -> ParseResult<Expr> {
        let start = left.span(self.ast).start;
        let mut expressions = vec![left];
        while self.eat(TokenTag::Comma)? {
            expressions.push(self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?);
        }
        let end = expressions.last().unwrap().span(self.ast).end;
        let expressions = self.ast.add_typed_sub_range(expressions);
        Ok(self
            .ast
            .expr_sequence_expression(Span::new(start, end), expressions))
    }

    fn expression_to_simple_assignment_target(
        &self,
        expression: Expr,
    ) -> ParseResult<SimpleAssignmentTarget> {
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(node) => {
                Ok(SimpleAssignmentTarget::IdentifierReference(node))
            }
            ExprData::MemberExpression(node) if !member_expression_is_optional(node, self.ast) => {
                Ok(SimpleAssignmentTarget::MemberExpression(node))
            }
            ExprData::CallExpression(node) => Ok(SimpleAssignmentTarget::CallExpression(node)),
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.expression_to_simple_assignment_target(parenthesized.expression(self.ast))
            }
            _ => Err(Error::new(
                expression.span(self.ast),
                "invalid assignment target",
            )),
        }
    }

    fn expression_to_assignment_target(
        &mut self,
        expression: Expr,
    ) -> ParseResult<AssignmentTarget> {
        match self.ast.expr_data(expression) {
            ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_) => {
                let pattern = self.expression_to_assignment_pattern(expression)?;
                match self.ast.binding_pattern_data(pattern) {
                    BindingPatternData::ArrayPattern(pattern) => {
                        Ok(AssignmentTarget::ArrayAssignmentTarget(pattern))
                    }
                    BindingPatternData::ObjectPattern(pattern) => {
                        Ok(AssignmentTarget::ObjectAssignmentTarget(pattern))
                    }
                    _ => unreachable!("array and object expressions convert to matching patterns"),
                }
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                let inner = parenthesized.expression(self.ast);
                if matches!(
                    self.ast.expr_data(inner),
                    ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_)
                ) {
                    return self.error_at(
                        expression.span(self.ast),
                        "A parenthesized destructuring pattern is not an assignment target",
                    );
                }
                self.expression_to_assignment_target(inner)
            }
            _ => Ok(AssignmentTarget::SimpleAssignmentTarget(
                self.expression_to_simple_assignment_target(expression)?,
            )),
        }
    }

    fn parse_binding_identifier(&mut self) -> ParseResult<BindingIdentifier> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() || token.tag.is_unconditionally_reserved() {
            return self.error("expected an identifier");
        }
        let name = self.identifier_atom(token)?;
        self.validate_identifier(token, name.as_str(), "a binding identifier")?;
        self.advance()?;
        let decorators = self.empty_range();
        Ok(self
            .ast
            .binding_identifier(token.span, name, decorators, None, false, None))
    }

    fn parse_label_identifier(&mut self) -> ParseResult<LabelIdentifier> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() || token.tag.is_unconditionally_reserved() {
            return self.error("expected a label");
        }
        let name = self.identifier_atom(token)?;
        self.validate_identifier(token, name.as_str(), "a label")?;
        self.advance()?;
        Ok(self.ast.label_identifier(token.span, name))
    }

    fn validate_identifier(&self, token: Token, name: &str, as_what: &str) -> ParseResult<()> {
        self.validate_identifier_name(token.span, name, as_what)
    }

    fn validate_identifier_name(&self, span: Span, name: &str, as_what: &str) -> ParseResult<()> {
        if name == "yield" && self.context.allow_yield {
            return Err(Error::new(
                span,
                format!(
                    "'yield' is reserved in a generator context and cannot be used as {as_what}"
                ),
            ));
        }
        if name == "await"
            && (self.context.allow_await || self.options.source_type == SourceType::Module)
        {
            return Err(Error::new(
                span,
                format!(
                    "'await' is reserved in an async/module context and cannot be used as {as_what}"
                ),
            ));
        }
        Ok(())
    }

    fn identifier_atom(&self, token: Token) -> ParseResult<Atom<'allocator>> {
        let start = token.span.start + u32::from(token.tag == TokenTag::PrivateIdentifier);
        if token.is_escaped() {
            let mut buffer = [0u8; 256];
            let bytes = decode_identifier_escapes(
                &self.source[start as usize..token.span.end as usize],
                &mut buffer,
            );
            let value = str::from_utf8(bytes)
                .map_err(|_| Error::new(token.span, "identifier is not valid UTF-8"))?;
            return Ok(self.ast.add_atom(value));
        }
        let bytes = &self.source[start as usize..token.span.end as usize];
        let value = str::from_utf8(bytes)
            .map_err(|_| Error::new(token.span, "identifier is not valid UTF-8"))?;
        Ok(self.ast.add_atom(value))
    }

    fn string_wtf8_atom(&self, token: Token) -> ParseResult<Wtf8Atom<'allocator>> {
        let raw = &self.source[token.span.start as usize + 1..token.span.end as usize - 1];
        if token.is_escaped() {
            let mut decoded = Vec::new();
            decode_string_escapes(raw, &mut decoded);
            let value = Wtf8::from_bytes(&decoded)
                .map_err(|_| Error::new(token.span, "string is not valid WTF-8"))?;
            Ok(self.ast.add_wtf8_atom(value))
        } else {
            let value = Wtf8::from_bytes(raw)
                .map_err(|_| Error::new(token.span, "string is not valid WTF-8"))?;
            Ok(self.ast.add_wtf8_atom(value))
        }
    }

    fn atom_from_span(&self, span: Span) -> Atom<'allocator> {
        let bytes = &self.source[span.start as usize..span.end as usize];
        let value = str::from_utf8(bytes).expect("the lexer only accepts valid UTF-8 text");
        self.ast.add_atom(value)
    }

    fn empty_range<T: ExtraDataCompact<'allocator>>(&mut self) -> TypedSubRange<T> {
        self.ast.add_typed_sub_range(std::iter::empty())
    }

    fn advance(&mut self) -> ParseResult<()> {
        self.previous_end = self.current_token.span.end;
        self.lexer
            .next_token(&mut self.current_token)
            .map_err(|error| self.lexical_error(error))
    }

    fn peek_tag(&mut self) -> ParseResult<TokenTag> {
        self.peek_token().map(|token| token.tag)
    }

    fn peek_token(&mut self) -> ParseResult<Token> {
        let cursor = self.lexer.cursor;
        let state = self.lexer.state;
        let mode = self.lexer.mode;
        let comments_len = self.lexer.comments.len();
        let mut token = Token::eof(cursor);
        let result = self.lexer.next_token(&mut token);
        self.lexer.cursor = cursor;
        self.lexer.state = state;
        self.lexer.mode = mode;
        self.lexer.comments.truncate(comments_len);
        result
            .map(|()| token)
            .map_err(|error| self.lexical_error(error))
    }

    fn peek_second_token(&mut self) -> ParseResult<Token> {
        let cursor = self.lexer.cursor;
        let state = self.lexer.state;
        let mode = self.lexer.mode;
        let comments_len = self.lexer.comments.len();
        let mut token = Token::eof(cursor);
        let result = self
            .lexer
            .next_token(&mut token)
            .and_then(|()| self.lexer.next_token(&mut token));
        self.lexer.cursor = cursor;
        self.lexer.state = state;
        self.lexer.mode = mode;
        self.lexer.comments.truncate(comments_len);
        result
            .map(|()| token)
            .map_err(|error| self.lexical_error(error))
    }

    fn is_async_arrow_head(&mut self) -> ParseResult<bool> {
        let cursor = self.lexer.cursor;
        let state = self.lexer.state;
        let mode = self.lexer.mode;
        let comments_len = self.lexer.comments.len();
        let result = (|| {
            let mut token = Token::eof(cursor);
            self.lexer.next_token(&mut token)?;
            if token.has_line_terminator_before() {
                return Ok(false);
            }
            if token.tag.is_identifier_like() && !token.tag.is_reserved() {
                self.lexer.next_token(&mut token)?;
                return Ok(token.tag == TokenTag::Arrow);
            }
            if token.tag != TokenTag::LeftParen {
                return Ok(false);
            }
            let mut depth = 1u32;
            while depth > 0 {
                self.lexer.next_token(&mut token)?;
                match token.tag {
                    TokenTag::LeftParen => depth += 1,
                    TokenTag::RightParen => depth -= 1,
                    TokenTag::Eof => return Ok(false),
                    _ => {}
                }
            }
            self.lexer.next_token(&mut token)?;
            Ok(token.tag == TokenTag::Arrow)
        })();
        self.lexer.cursor = cursor;
        self.lexer.state = state;
        self.lexer.mode = mode;
        self.lexer.comments.truncate(comments_len);
        result.map_err(|error| self.lexical_error(error))
    }

    fn expect(&mut self, tag: TokenTag) -> ParseResult<Token> {
        if self.current_token.tag != tag {
            return self.error(format!(
                "expected '{}', found {}",
                tag.to_str().unwrap_or("token"),
                self.describe_current()
            ));
        }
        let token = self.current_token;
        if tag.is_keyword() && token.is_escaped() {
            return self.error_at(token.span, "keywords may not contain escape sequences");
        }
        self.advance()?;
        Ok(token)
    }

    fn eat(&mut self, tag: TokenTag) -> ParseResult<bool> {
        if self.current_token.tag != tag {
            return Ok(false);
        }
        if tag.is_keyword() && self.current_token.is_escaped() {
            return self.error_at(
                self.current_token.span,
                "keywords may not contain escape sequences",
            );
        }
        self.advance()?;
        Ok(true)
    }

    fn eat_semicolon(&mut self) -> ParseResult<u32> {
        if self.current_token.tag == TokenTag::Semicolon {
            return Ok(self.expect(TokenTag::Semicolon)?.span.end);
        }
        if self.current_token.tag == TokenTag::RightBrace
            || self.current_token.tag == TokenTag::Eof
            || self.current_token.has_line_terminator_before()
        {
            return Ok(self.previous_end);
        }
        self.error("expected ';'")
    }

    fn describe_current(&self) -> String {
        self.current_token.tag.to_str().map_or_else(
            || {
                String::from_utf8_lossy(
                    &self.source[self.current_token.span.start as usize
                        ..self.current_token.span.end as usize],
                )
                .into_owned()
            },
            str::to_owned,
        )
    }

    fn lexical_error(&self, error: LexicalError) -> Error {
        Error::new(
            Span::new(self.previous_end, self.lexer.cursor),
            format!(
                "{}. {}",
                lexical_error_message(error),
                lexical_error_help(error)
            ),
        )
    }

    fn error<T>(&self, message: impl Into<String>) -> ParseResult<T> {
        Err(Error::new(self.current_token.span, message))
    }

    fn error_at<T>(&self, span: Span, message: impl Into<String>) -> ParseResult<T> {
        Err(Error::new(span, message))
    }
}

fn start_of(token: Token) -> u32 {
    token.span.start
}

fn member_expression_is_optional(member: MemberExpression, ast: &Ast<'_>) -> bool {
    if member.optional(ast) {
        return true;
    }
    match ast.expr_data(member.object(ast)) {
        ExprData::MemberExpression(object) => member_expression_is_optional(object, ast),
        ExprData::CallExpression(call) => call.optional(ast),
        ExprData::ChainExpression(_) => true,
        ExprData::ParenthesizedExpression(parenthesized) => {
            matches!(
                ast.expr_data(parenthesized.expression(ast)),
                ExprData::ChainExpression(_)
            )
        }
        _ => false,
    }
}

fn expression_contains_optional_chain(expression: Expr, ast: &Ast<'_>) -> bool {
    match ast.expr_data(expression) {
        ExprData::MemberExpression(member) => member_expression_is_optional(member, ast),
        ExprData::CallExpression(call) => {
            call.optional(ast) || expression_contains_optional_chain(call.callee(ast), ast)
        }
        ExprData::ChainExpression(_) => true,
        ExprData::ParenthesizedExpression(_) => false,
        _ => false,
    }
}

fn logical_expression_mixes_coalesce(
    expression: Expr,
    parent_operator: LogicalOperator,
    ast: &Ast<'_>,
) -> bool {
    let ExprData::LogicalExpression(logical) = ast.expr_data(expression) else {
        return false;
    };
    let child_operator = logical.operator(ast);
    (parent_operator == LogicalOperator::NullishCoalescing)
        != (child_operator == LogicalOperator::NullishCoalescing)
}

fn assignment_target_is_call_expression(target: AssignmentTarget, ast: &Ast<'_>) -> bool {
    matches!(
        ast.assignment_target_data(target),
        AssignmentTargetData::SimpleAssignmentTarget(simple)
            if matches!(
                ast.simple_assignment_target_data(simple),
                SimpleAssignmentTargetData::CallExpression(_)
            )
    )
}

fn is_unconditionally_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
    )
}

fn binary_operator(tag: TokenTag) -> BinaryOperator {
    match tag {
        TokenTag::Equal => BinaryOperator::Equal,
        TokenTag::NotEqual => BinaryOperator::NotEqual,
        TokenTag::StrictEqual => BinaryOperator::StrictEqual,
        TokenTag::StrictNotEqual => BinaryOperator::StrictNotEqual,
        TokenTag::LessThan => BinaryOperator::LessThan,
        TokenTag::LessThanEqual => BinaryOperator::LessThanOrEqual,
        TokenTag::GreaterThan => BinaryOperator::GreaterThan,
        TokenTag::GreaterThanEqual => BinaryOperator::GreaterThanOrEqual,
        TokenTag::Plus => BinaryOperator::Add,
        TokenTag::Minus => BinaryOperator::Subtract,
        TokenTag::Star => BinaryOperator::Multiply,
        TokenTag::Slash => BinaryOperator::Divide,
        TokenTag::Percent => BinaryOperator::Modulo,
        TokenTag::Exponent => BinaryOperator::Exponent,
        TokenTag::BitwiseOr => BinaryOperator::BitwiseOr,
        TokenTag::BitwiseXor => BinaryOperator::BitwiseXor,
        TokenTag::BitwiseAnd => BinaryOperator::BitwiseAnd,
        TokenTag::LeftShift => BinaryOperator::LeftShift,
        TokenTag::RightShift => BinaryOperator::RightShift,
        TokenTag::UnsignedRightShift => BinaryOperator::UnsignedRightShift,
        TokenTag::In => BinaryOperator::In,
        TokenTag::Instanceof => BinaryOperator::Instanceof,
        _ => unreachable!(),
    }
}

fn assignment_operator(tag: TokenTag) -> AssignmentOperator {
    match tag {
        TokenTag::Assign => AssignmentOperator::Assign,
        TokenTag::PlusAssign => AssignmentOperator::AddAssign,
        TokenTag::MinusAssign => AssignmentOperator::SubtractAssign,
        TokenTag::StarAssign => AssignmentOperator::MultiplyAssign,
        TokenTag::SlashAssign => AssignmentOperator::DivideAssign,
        TokenTag::PercentAssign => AssignmentOperator::ModuloAssign,
        TokenTag::ExponentAssign => AssignmentOperator::ExponentAssign,
        TokenTag::LeftShiftAssign => AssignmentOperator::LeftShiftAssign,
        TokenTag::RightShiftAssign => AssignmentOperator::RightShiftAssign,
        TokenTag::UnsignedRightShiftAssign => AssignmentOperator::UnsignedRightShiftAssign,
        TokenTag::BitwiseOrAssign => AssignmentOperator::BitwiseOrAssign,
        TokenTag::BitwiseXorAssign => AssignmentOperator::BitwiseXorAssign,
        TokenTag::BitwiseAndAssign => AssignmentOperator::BitwiseAndAssign,
        TokenTag::LogicalOrAssign => AssignmentOperator::LogicalOrAssign,
        TokenTag::LogicalAndAssign => AssignmentOperator::LogicalAndAssign,
        TokenTag::NullishAssign => AssignmentOperator::NullishAssign,
        _ => unreachable!(),
    }
}
