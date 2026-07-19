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
            TokenTag::Var | TokenTag::Let | TokenTag::Const => {
                self.parse_variable_declaration_statement()
            }
            TokenTag::Function => self.parse_function_declaration(false),
            TokenTag::Async if is_async_function => self.parse_function_declaration(true),
            TokenTag::For => self.parse_for_statement(),
            TokenTag::Class => self.parse_class_declaration(),
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
        let end = self.eat_semicolon()?;
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
            _ => return self.error("expected variable declaration"),
        };
        self.advance()?;
        let mut declarators = Vec::new();
        loop {
            let start = self.current_token.span.start;
            let id = self.parse_binding_pattern(false)?;
            let init = if self.eat(TokenTag::Assign)? {
                Some(self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?)
            } else {
                None
            };
            if kind == VariableKind::Const && init.is_none() && !in_for_head {
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
            let right = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
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
                let expression = self.parse_expression()?;
                self.expect(TokenTag::RightBracket)?;
                (PropertyKey::Expr(expression), true)
            } else {
                (self.parse_property_key()?, false)
            };
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
                    let right = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
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
        let id = if self.current_token.tag.is_identifier_like() {
            Some(self.parse_binding_identifier()?)
        } else {
            None
        };
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

    fn parse_class_expression(&mut self) -> ParseResult<Expr> {
        let class = self.parse_class(false)?;
        Ok(Expr::Class(class))
    }

    fn parse_class(&mut self, declaration: bool) -> ParseResult<Class> {
        let start = self.expect(TokenTag::Class)?.span.start;
        let id = if self.current_token.tag.is_identifier_like() {
            Some(self.parse_binding_identifier()?)
        } else if declaration {
            return self.error("class declarations require a name");
        } else {
            None
        };
        let super_class = if self.eat(TokenTag::Extends)? {
            Some(self.parse_expression_with_precedence(Precedence::RELATIONAL)?)
        } else {
            None
        };
        let body = self.parse_class_body()?;
        let end = body.span(self.ast).end;
        let decorators = self.empty_range();
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
            let element_start = self.current_token.span.start;
            let is_static = self.current_token.tag == TokenTag::Static
                && self.peek_tag()? != TokenTag::LeftParen;
            if is_static {
                self.advance()?;
                if self.current_token.tag == TokenTag::LeftBrace {
                    let block = self.parse_block()?;
                    let body = block.body(self.ast);
                    let element = self
                        .ast
                        .static_block(Span::new(element_start, block.span(self.ast).end), body);
                    elements.push(ClassElement::StaticBlock(element));
                    continue;
                }
            }
            let key_token = self.current_token;
            let key = self.parse_property_key()?;
            if self.current_token.tag == TokenTag::LeftParen {
                let params = self.parse_formal_parameters()?;
                let return_type = if self.options.lang.is_typescript()
                    && self.current_token.tag == TokenTag::Colon
                {
                    Some(self.parse_ts_type_annotation()?)
                } else {
                    None
                };
                let old_context = self.context;
                self.context.allow_return = true;
                let body = self.parse_function_body();
                self.context = old_context;
                let body = body?;
                let end = body.span(self.ast).end;
                let function = self.ast.function(
                    Span::new(element_start, end),
                    FunctionType::FunctionExpression,
                    None,
                    false,
                    false,
                    false,
                    params,
                    Some(body),
                    None,
                    return_type,
                );
                let decorators = self.empty_range();
                let kind = if key_token.tag == TokenTag::Constructor {
                    MethodDefinitionKind::Constructor
                } else {
                    MethodDefinitionKind::Method
                };
                let method = self.ast.method_definition(
                    Span::new(element_start, end),
                    decorators,
                    key,
                    function,
                    kind,
                    false,
                    is_static,
                    false,
                    false,
                    false,
                    Accessibility::None,
                );
                elements.push(ClassElement::MethodDefinition(method));
            } else {
                let value = if self.eat(TokenTag::Assign)? {
                    Some(self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?)
                } else {
                    None
                };
                let end = self.eat_semicolon()?;
                let decorators = self.empty_range();
                let property = self.ast.property_definition(
                    Span::new(element_start, end),
                    decorators,
                    key,
                    value,
                    false,
                    is_static,
                    false,
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
        Ok(self.ast.formal_parameters(
            Span::new(start, end),
            items,
            rest,
            FormalParameterKind::FormalParameters,
        ))
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
        let end = self.eat_semicolon()?;
        let specifiers = self.ast.add_typed_sub_range(specifiers);
        let attributes = self.empty_range();
        Ok(self.ast.stmt_import_declaration(
            Span::new(start, end),
            specifiers,
            source,
            attributes,
            None,
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
                (ExportDefaultDeclarationKind::Function(function), false)
            } else if self.current_token.tag == TokenTag::Async
                && self.peek_tag()? == TokenTag::Function
            {
                let expression = self.parse_function_expression(true)?;
                let ExprData::Function(function) = self.ast.expr_data(expression) else {
                    unreachable!()
                };
                (ExportDefaultDeclarationKind::Function(function), false)
            } else if self.current_token.tag == TokenTag::Class {
                (
                    ExportDefaultDeclarationKind::Class(self.parse_class(false)?),
                    false,
                )
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
            self.expect(TokenTag::From)?;
            let source = self.parse_string_literal_node()?;
            let end = self.eat_semicolon()?;
            let attributes = self.empty_range();
            return Ok(self.ast.stmt_export_all_declaration(
                Span::new(start, end),
                None,
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
        let end = self.eat_semicolon()?;
        let specifiers = self.ast.add_typed_sub_range(specifiers);
        let attributes = self.empty_range();
        Ok(self.ast.stmt_export_named_declaration(
            Span::new(start, end),
            None,
            specifiers,
            source,
            attributes,
            ImportOrExportKind::Value,
        ))
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
        self.expect(TokenTag::LeftParen)?;
        let old_allow_in = self.context.allow_in;
        self.context.allow_in = false;
        let init = if matches!(
            self.current_token.tag,
            TokenTag::Var | TokenTag::Let | TokenTag::Const
        ) {
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
            let init = init.ok_or_else(|| {
                Error::new(
                    self.current_token.span,
                    "for-in/of requires a left-hand side",
                )
            })?;
            let left = match self.ast.for_statement_init_data(init) {
                ForStatementInitData::VariableDeclaration(declaration) => {
                    ForStatementLeft::VariableDeclaration(declaration)
                }
                ForStatementInitData::Expr(expression) => ForStatementLeft::AssignmentTarget(
                    self.expression_to_assignment_target(expression)?,
                ),
            };
            self.advance()?;
            let right = self.parse_expression()?;
            self.expect(TokenTag::RightParen)?;
            let body = self.parse_statement()?;
            let end = body.span(self.ast).end;
            return if operator == TokenTag::In {
                Ok(self
                    .ast
                    .stmt_for_in_statement(Span::new(start, end), left, right, body))
            } else {
                Ok(self
                    .ast
                    .stmt_for_of_statement(Span::new(start, end), left, right, body, false))
            };
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
        let mut left = self.parse_prefix_expression()?;
        loop {
            let token = self.current_token;
            if token.tag == TokenTag::In && !self.context.allow_in {
                break;
            }
            if matches!(
                token.tag,
                TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead
            ) {
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

    fn parse_prefix_expression(&mut self) -> ParseResult<Expr> {
        let token = self.current_token;
        match token.tag {
            TokenTag::Async if self.peek_tag()? == TokenTag::Function => {
                self.parse_function_expression(true)
            }
            TokenTag::Async if self.is_async_arrow_head()? => self.parse_async_arrow_expression(),
            TokenTag::Await if self.context.allow_await => self.parse_await_expression(),
            TokenTag::Yield if self.context.allow_yield => self.parse_yield_expression(),
            tag if tag.is_identifier_like() && !tag.is_reserved() => {
                let name = self.identifier_atom(token)?;
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
                self.advance()?;
                Ok(self
                    .ast
                    .expr_boolean_literal(token.span, token.tag == TokenTag::True))
            }
            TokenTag::NullLiteral => {
                self.advance()?;
                Ok(self.ast.expr_null_literal(token.span))
            }
            tag if tag.is_numeric_literal() => self.parse_numeric_literal(),
            TokenTag::StringLiteral => self.parse_string_literal(),
            TokenTag::NoSubstitutionTemplate | TokenTag::TemplateHead => {
                self.parse_template_literal()
            }
            TokenTag::Slash | TokenTag::SlashAssign => self.parse_regex_literal(),
            TokenTag::LeftParen => self.parse_parenthesized_or_arrow_expression(),
            TokenTag::LeftBracket => self.parse_array_expression(),
            TokenTag::LeftBrace => self.parse_object_expression(),
            TokenTag::Function => self.parse_function_expression(false),
            TokenTag::New => self.parse_new_expression(),
            TokenTag::Class => self.parse_class_expression(),
            TokenTag::LessThan if self.options.lang.is_jsx() => self.parse_jsx_element(),
            TokenTag::Import => {
                if self.peek_tag()? == TokenTag::Dot {
                    self.parse_meta_property()
                } else {
                    self.parse_import_expression()
                }
            }
            TokenTag::Increment | TokenTag::Decrement => {
                self.advance()?;
                let argument = self.parse_prefix_expression()?;
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
        let pattern = self.atom_from_span(result.pattern);
        let flags = self.atom_from_span(result.flags);
        let expression = self.ast.expr_reg_exp_literal(result.span, pattern, flags);
        self.current_token = Token::eof(result.span.end);
        self.advance()?;
        Ok(expression)
    }

    fn parse_template_literal(&mut self) -> ParseResult<Expr> {
        Ok(Expr::TemplateLiteral(self.parse_template_literal_node()?))
    }

    fn parse_template_literal_node(&mut self) -> ParseResult<TemplateLiteral> {
        let start = self.current_token.span.start;
        let mut quasis = Vec::new();
        let mut expressions = Vec::new();
        if self.current_token.tag == TokenTag::NoSubstitutionTemplate {
            let token = self.current_token;
            quasis.push(self.add_template_element(token, true)?);
            self.advance()?;
            let quasis = self.ast.add_typed_sub_range(quasis);
            let expressions = self.ast.add_typed_sub_range(expressions);
            return Ok(self.ast.template_literal(token.span, quasis, expressions));
        }
        let head = self.current_token;
        quasis.push(self.add_template_element(head, false)?);
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
            quasis.push(self.add_template_element(continuation, tail)?);
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
        let quasi = self.parse_template_literal_node()?;
        Ok(self.ast.expr_tagged_template_expression(
            Span::new(start, quasi.span(self.ast).end),
            tag,
            quasi,
            None,
        ))
    }

    fn add_template_element(&mut self, token: Token, tail: bool) -> ParseResult<TemplateElement> {
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
        if self.current_token.tag == TokenTag::RightParen {
            self.advance()?;
            self.expect(TokenTag::Arrow)?;
            return self.parse_arrow_body(arrow_start, Vec::new(), async_function);
        }
        let first = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
        if self.current_token.tag == TokenTag::Comma {
            let mut expressions = vec![first];
            while self.eat(TokenTag::Comma)? {
                expressions.push(self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?);
            }
            let end = self.expect(TokenTag::RightParen)?.span.end;
            if self.eat(TokenTag::Arrow)? {
                let parameters = expressions
                    .into_iter()
                    .map(|expression| self.expression_to_parameter(expression))
                    .collect::<ParseResult<Vec<_>>>()?;
                return self.parse_arrow_body(arrow_start, parameters, async_function);
            }
            let expressions = self.ast.add_typed_sub_range(expressions);
            let sequence = self
                .ast
                .expr_sequence_expression(Span::new(start + 1, end - 1), expressions);
            return if self.options.preserve_parens {
                Ok(self
                    .ast
                    .expr_parenthesized_expression(Span::new(start, end), sequence))
            } else {
                Ok(sequence)
            };
        }
        let end = self.expect(TokenTag::RightParen)?.span.end;
        if self.eat(TokenTag::Arrow)? {
            let parameter = self.expression_to_parameter(first)?;
            return self.parse_arrow_body(arrow_start, vec![parameter], async_function);
        }
        if self.options.preserve_parens {
            Ok(self
                .ast
                .expr_parenthesized_expression(Span::new(start, end), first))
        } else {
            Ok(first)
        }
    }

    fn parse_async_arrow_expression(&mut self) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::Async)?.span.start;
        if self.current_token.tag == TokenTag::LeftParen {
            return self.parse_parenthesized_or_arrow_expression_with(Some(start), true);
        }
        let token = self.current_token;
        let name = self.identifier_atom(token)?;
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
        let parameter_end = self.previous_end;
        let parameters = self.ast.add_typed_sub_range(parameters);
        let parameters = self.ast.formal_parameters(
            Span::new(start, parameter_end),
            parameters,
            None,
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
        match self.ast.expr_data(expression) {
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
            ExprData::AssignmentExpression(assignment)
                if assignment.operator(self.ast) == AssignmentOperator::Assign =>
            {
                let left = self.assignment_target_to_binding_pattern(assignment.left(self.ast))?;
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
                            elements.push(Some(self.expression_to_binding_pattern(expression)?));
                        }
                        ArgumentData::SpreadElement(spread) => {
                            if index + 1 != source_elements_len {
                                return self.error_at(
                                    spread.span(self.ast),
                                    "a rest element must be last",
                                );
                            }
                            let argument =
                                self.expression_to_binding_pattern(spread.argument(self.ast))?;
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
                            let value =
                                self.expression_to_binding_pattern(property.value(self.ast))?;
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
                            let argument =
                                self.expression_to_binding_pattern(spread.argument(self.ast))?;
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
                self.expression_to_binding_pattern(parenthesized.expression(self.ast))
            }
            _ => self.error_at(expression.span(self.ast), "invalid arrow parameter"),
        }
    }

    fn assignment_target_to_binding_pattern(
        &mut self,
        target: AssignmentTarget,
    ) -> ParseResult<BindingPattern> {
        match self.ast.assignment_target_data(target) {
            AssignmentTargetData::SimpleAssignmentTarget(target) => {
                match self.ast.simple_assignment_target_data(target) {
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

    fn parse_array_expression(&mut self) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::LeftBracket)?.span.start;
        let mut elements = Vec::new();
        while self.current_token.tag != TokenTag::RightBracket {
            if self.eat(TokenTag::Comma)? {
                elements.push(None);
                continue;
            }
            let argument = if self.eat(TokenTag::Spread)? {
                let spread_start = self.previous_end - 3;
                let expression = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
                let span = Span::new(spread_start, expression.span(self.ast).end);
                Argument::SpreadElement(self.ast.spread_element(span, expression))
            } else {
                Argument::Expr(self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?)
            };
            elements.push(Some(argument));
            if !self.eat(TokenTag::Comma)? {
                break;
            }
        }
        let end = self.expect(TokenTag::RightBracket)?.span.end;
        let elements = self.ast.add_typed_sub_range(elements);
        Ok(self
            .ast
            .expr_array_expression(Span::new(start, end), elements))
    }

    fn parse_object_expression(&mut self) -> ParseResult<Expr> {
        let start = self.expect(TokenTag::LeftBrace)?.span.start;
        let mut properties = Vec::new();
        while self.current_token.tag != TokenTag::RightBrace {
            if self.eat(TokenTag::Spread)? {
                let spread_start = self.previous_end - 3;
                let expression = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
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
        }
        let end = self.expect(TokenTag::RightBrace)?.span.end;
        let properties = self.ast.add_typed_sub_range(properties);
        Ok(self
            .ast
            .expr_object_expression(Span::new(start, end), properties))
    }

    fn parse_object_property(&mut self) -> ParseResult<ObjectPropertyKind> {
        let token = self.current_token;
        let key = self.parse_property_key()?;
        if self.current_token.tag == TokenTag::LeftParen {
            let params = self.parse_formal_parameters()?;
            let return_type =
                if self.options.lang.is_typescript() && self.current_token.tag == TokenTag::Colon {
                    Some(self.parse_ts_type_annotation()?)
                } else {
                    None
                };
            let old_context = self.context;
            self.context.allow_return = true;
            let body = self.parse_function_body();
            self.context = old_context;
            let body = body?;
            let span = Span::new(token.span.start, body.span(self.ast).end);
            let function = self.ast.function(
                span,
                FunctionType::FunctionExpression,
                None,
                false,
                false,
                false,
                params,
                Some(body),
                None,
                return_type,
            );
            let value = Expr::Function(function);
            return Ok(ObjectPropertyKind::ObjectProperty(
                self.ast
                    .object_property(span, key, value, PropertyKind::Init, true, false, false),
            ));
        }
        let (value, shorthand) = if self.eat(TokenTag::Colon)? {
            (
                self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?,
                false,
            )
        } else {
            let PropertyKeyData::IdentifierName(identifier) = self.ast.property_key_data(key)
            else {
                return self.error_at(key.span(self.ast), "expected ':' after property name");
            };
            let identifier = self.ast.expr_identifier_reference(
                identifier.span(self.ast),
                identifier.name(self.ast),
                None,
            );
            let value = if self.eat(TokenTag::Assign)? {
                let right = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
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
        let span = Span::new(token.span.start, value.span(self.ast).end);
        let property = self.ast.object_property(
            span,
            key,
            value,
            PropertyKind::Init,
            false,
            shorthand,
            false,
        );
        Ok(ObjectPropertyKind::ObjectProperty(property))
    }

    fn parse_property_key(&mut self) -> ParseResult<PropertyKey> {
        let token = self.current_token;
        match token.tag {
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
            let expression = self.parse_expression()?;
            let property = PropertyKey::Expr(expression);
            let end = self.expect(TokenTag::RightBracket)?.span.end;
            (property, true, end)
        } else {
            if !optional {
                self.expect(TokenTag::Dot)?;
            }
            let token = self.current_token;
            let name = self.identifier_atom(token)?;
            self.advance()?;
            (
                self.ast.property_key_identifier_name(token.span, name),
                false,
                token.span.end,
            )
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
            let argument = if self.eat(TokenTag::Spread)? {
                let start = self.previous_end - 3;
                let expression = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
                Argument::SpreadElement(
                    self.ast.spread_element(
                        Span::new(start, expression.span(self.ast).end),
                        expression,
                    ),
                )
            } else {
                Argument::Expr(self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?)
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
        let callee = self.parse_expression_with_precedence(Precedence::NEW)?;
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
        let start = self.expect(TokenTag::Import)?.span.start;
        self.expect(TokenTag::LeftParen)?;
        let source = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
        let options = if self.eat(TokenTag::Comma)? {
            Some(self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?)
        } else {
            None
        };
        let end = self.expect(TokenTag::RightParen)?.span.end;
        Ok(self
            .ast
            .expr_import_expression(Span::new(start, end), source, options, None))
    }

    fn parse_meta_property(&mut self) -> ParseResult<Expr> {
        let meta_token = self.current_token;
        let meta_name = self.identifier_atom(meta_token)?;
        self.advance()?;
        self.expect(TokenTag::Dot)?;
        let property_token = self.current_token;
        if !property_token.tag.is_identifier_like() {
            return self.error("expected a meta-property name");
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
        let delegate = self.eat(TokenTag::Star)?;
        let argument = if self.current_token.has_line_terminator_before()
            || matches!(
                self.current_token.tag,
                TokenTag::Semicolon | TokenTag::RightBrace | TokenTag::Eof
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
        self.advance()?;
        let right = self.parse_expression_with_precedence(precedence + 1)?;
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
        let consequent = self.parse_expression_with_precedence(Precedence::ASSIGNMENT)?;
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
            ExprData::MemberExpression(node) => Ok(SimpleAssignmentTarget::MemberExpression(node)),
            _ => Err(Error::new(
                expression.span(self.ast),
                "invalid assignment target",
            )),
        }
    }

    fn expression_to_assignment_target(&self, expression: Expr) -> ParseResult<AssignmentTarget> {
        Ok(AssignmentTarget::SimpleAssignmentTarget(
            self.expression_to_simple_assignment_target(expression)?,
        ))
    }

    fn parse_binding_identifier(&mut self) -> ParseResult<BindingIdentifier> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() || token.tag.is_unconditionally_reserved() {
            return self.error("expected an identifier");
        }
        let name = self.identifier_atom(token)?;
        self.advance()?;
        let decorators = self.empty_range();
        Ok(self
            .ast
            .binding_identifier(token.span, name, decorators, None, false, None))
    }

    fn parse_label_identifier(&mut self) -> ParseResult<LabelIdentifier> {
        let token = self.current_token;
        if !token.tag.is_identifier_like() {
            return self.error("expected a label");
        }
        let name = self.identifier_atom(token)?;
        self.advance()?;
        Ok(self.ast.label_identifier(token.span, name))
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
            .map(|()| token.tag)
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
        self.advance()?;
        Ok(token)
    }

    fn eat(&mut self, tag: TokenTag) -> ParseResult<bool> {
        if self.current_token.tag != tag {
            return Ok(false);
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
