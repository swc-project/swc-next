//! Real for-header owners and the independent punctuation between them.

use swc_next_ecma_ast::{
    AssignmentTargetData, BindingPattern, Comment, CommentKind, Expr, ForInStatement,
    ForOfStatement, ForStatement, ForStatementInitData, ForStatementLeftData, GetSpan,
    ParenthesizedExpression, Span, Stmt,
};

use super::{
    ControlledBodyEnd, ControlledBodyRole, EmptyBlock, Formatter, PrintedStatement,
    StatementPunctuation,
};
use crate::formatter::comments::{
    CommentBinding, CommentCursor, CommentGap, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::targets::SourceTarget;
use crate::formatter::expressions::{
    ExpressionContext, ExpressionPosition, ForInitializerGrammar, TailOwner,
};
use crate::formatter::suppression::{ContentMode, DelimitedFollowing, SuppressionPosition};
use crate::formatter::variables::VariableContext;
use crate::{FormatError, document::Writer};

pub(in crate::formatter::statements) enum IterationStatement {
    In(ForInStatement),
    Of(ForOfStatement),
}

struct HeaderNode<'ast> {
    span: Span,
    data: HeaderData<'ast>,
}

enum HeaderData<'ast> {
    Expression {
        node: Expr,
        context: ExpressionContext,
    },
    SourceExpression {
        node: Expr,
        enclosure: Option<ParenthesizedExpression>,
        context: ExpressionContext,
        leading: &'ast [Comment],
        following: LineConstraint,
    },
    Variable {
        node: swc_next_ecma_ast::VariableDeclaration,
        context: VariableContext,
    },
    Target {
        node: SourceTarget,
        context: ExpressionContext,
    },
    Pattern(BindingPattern),
}

#[derive(Clone, Copy)]
struct HeaderToken {
    span: Span,
    text: &'static str,
    space: bool,
    // An omitted slot has no trailing expression owner for a same-line packet.
    ends_node: bool,
    // Absent init/test nodes still own a break before their semicolon.
    empty_slot: Option<FlatSeparator>,
}

struct HeaderSyntax<'ast> {
    comments: CommentCursor<'ast>,
    keyword_end: u32,
    opening: Span,
    closing: Span,
    tokens: [Option<HeaderToken>; 4],
    classic: bool,
}

#[derive(Clone, Copy)]
enum HeaderRelation {
    First,
    Items(Span),
    Body(Span),
    Empty,
}

#[derive(Clone, Copy)]
enum HeaderGapOwner {
    Syntax,
    Node,
    Body,
}

struct PreparedHeaderToken<'ast, 'header> {
    token: &'header HeaderToken,
    before: &'ast [Comment],
    trailing: &'ast [Comment],
    following: CommentGap,
}

struct HeaderGap<'ast> {
    span: Span,
    comments: &'ast [Comment],
    tokens: std::ops::Range<usize>,
    leading: &'ast [Comment],
    source: bool,
    owner: HeaderGapOwner,
    head_gap: Option<LineConstraint>,
    preceding: ContentMode,
    following: ContentMode,
}

struct IterationGap {
    operator: HeaderToken,
    right: Expr,
}

enum HeaderGapOutput<'ast> {
    Canonical {
        start: u32,
        leading: &'ast [Comment],
        following: CommentGap,
        head_gap: Option<LineConstraint>,
        head_layout: CommentRunLayout,
    },
    Source {
        following: LineConstraint,
        comment_end: bool,
    },
}

struct PreparedHeaderItem<'ast, 'header> {
    node: &'header HeaderNode<'ast>,
    before: HeaderGap<'ast>,
    mode: ContentMode,
}

// Prepared owners stay borrowed while printing consumes only the comment cursor.
struct PreparedHeader<'ast, 'header> {
    items: [Option<PreparedHeaderItem<'ast, 'header>>; 3],
    tokens: [Option<PreparedHeaderToken<'ast, 'header>>; 4],
    trailing: HeaderGap<'ast>,
    final_tail: Option<&'ast [Comment]>,
    iteration: Option<IterationGap>,
    closing: Span,
    classic: bool,
    empty: bool,
    body_mode: ContentMode,
    body_head_end: u32,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter::statements) fn format_for_statement(
        &mut self,
        node: ForStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        self.with_for_initializer(None, |formatter| {
            formatter.format_classic_for(node, punctuation, writer)
        })
    }

    fn format_classic_for(
        &mut self,
        node: ForStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let body = node.body(ast);
        let body_start = body.span(ast).start;
        let init = node.init(ast).map(|init| {
            let span = init.span(ast);
            let data = match ast.for_statement_init_data(init) {
                ForStatementInitData::VariableDeclaration(node) => HeaderData::Variable {
                    node,
                    context: VariableContext::ForInitializer,
                },
                ForStatementInitData::Expr(node) => self.prepare_for_initializer_expression(node),
            };
            HeaderNode { span, data }
        });
        let test = node
            .test(ast)
            .map(|node| self.for_expression_node(node, ExpressionPosition::ForTest));
        let update = node
            .update(ast)
            .map(|node| self.for_expression_node(node, ExpressionPosition::ForHead));
        let keyword_end = node.span(ast).start + 3;
        let comments =
            CommentCursor::new(self.comments.in_range(Span::new(keyword_end, body_start)));
        let token = |span: Span, text: &'static str| {
            self.find_token(span, comments.in_range(span), text)
                .map(|start| HeaderToken {
                    span: Span::new(start, start + 1),
                    text,
                    space: false,
                    ends_node: false,
                    empty_slot: None,
                })
                .ok_or_else(|| self.unsupported("for header boundary", node))
        };
        let mut first = token(
            Span::new(
                init.as_ref().map_or(keyword_end, |node| node.span.end),
                test.as_ref()
                    .or(update.as_ref())
                    .map_or(body_start, |node| node.span.start),
            ),
            ";",
        )?;
        let mut second = token(
            Span::new(
                test.as_ref().map_or(first.span.end, |node| node.span.end),
                update.as_ref().map_or(body_start, |node| node.span.start),
            ),
            ";",
        )?;
        let closing = token(
            Span::new(
                update
                    .as_ref()
                    .map_or(second.span.end, |node| node.span.end),
                body_start,
            ),
            ")",
        )?;
        let opening = token(
            Span::new(
                keyword_end,
                init.as_ref()
                    .map_or(first.span.start, |node| node.span.start),
            ),
            "(",
        )?;
        first.ends_node = init.is_some();
        second.ends_node = test.is_some();
        if init.is_some() || test.is_some() || update.is_some() {
            first.empty_slot = init.is_none().then_some(FlatSeparator::Empty);
            second.empty_slot = test.is_none().then_some(FlatSeparator::Space);
        }
        let syntax = HeaderSyntax {
            comments,
            keyword_end,
            opening: opening.span,
            closing: closing.span,
            tokens: [Some(opening), Some(first), Some(second), None],
            classic: true,
        };
        let nodes = [init, test, update];
        let mut header = self.prepare_for_header(&syntax, &nodes, body);
        header.final_tail = self.prepare_classic_final_tail(&header)?;
        self.format_for_with_body(&header, body, node.span(ast).end, punctuation, writer)
    }

    pub(in crate::formatter::statements) fn format_iteration_statement(
        &mut self,
        statement: IterationStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        self.with_for_initializer(None, |formatter| {
            formatter.format_iteration_for(statement, punctuation, writer)
        })
    }

    fn format_iteration_for(
        &mut self,
        statement: IterationStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let (span, left, right, body, of, awaited) = match statement {
            IterationStatement::In(node) => (
                node.span(ast),
                node.left(ast),
                node.right(ast),
                node.body(ast),
                false,
                false,
            ),
            IterationStatement::Of(node) => (
                node.span(ast),
                node.left(ast),
                node.right(ast),
                node.body(ast),
                true,
                node.r#await(ast),
            ),
        };
        let keyword_end = span.start + 3;
        let body_start = body.span(ast).start;
        let comments =
            CommentCursor::new(self.comments.in_range(Span::new(keyword_end, body_start)));
        let left_span = left.span(ast);
        let right_span = right.span(ast);
        let opening_gap = Span::new(keyword_end, left_span.start);
        let opening_comments = comments.in_range(opening_gap);
        let opening_start = self
            .find_token(opening_gap, opening_comments, "(")
            .ok_or_else(|| self.unsupported("iteration opening boundary", left))?;
        let opening = HeaderToken {
            span: Span::new(opening_start, opening_start + 1),
            text: "(",
            space: true,
            ends_node: false,
            empty_slot: None,
        };
        let operator_gap = Span::new(left_span.end, right_span.start);
        let operator_text = if of { "of" } else { "in" };
        let operator_start = self
            .find_token(operator_gap, comments.in_range(operator_gap), operator_text)
            .ok_or_else(|| self.unsupported("iteration operator boundary", left))?;
        let operator = HeaderToken {
            span: Span::new(operator_start, operator_start + 2),
            text: operator_text,
            space: true,
            ends_node: false,
            empty_slot: None,
        };
        let closing_gap = Span::new(right_span.end, body_start);
        let closing_start = self
            .find_token(closing_gap, comments.in_range(closing_gap), ")")
            .ok_or_else(|| self.unsupported("iteration closing boundary", right))?;
        let closing = Span::new(closing_start, closing_start + 1);
        let await_token = if awaited {
            let start = self
                .find_token(opening_gap, opening_comments, "await")
                .ok_or_else(|| self.unsupported("for-await keyword boundary", left))?;
            Some(HeaderToken {
                span: Span::new(start, start + 5),
                text: "await",
                space: true,
                ends_node: false,
                empty_slot: None,
            })
        } else {
            None
        };
        let left = match ast.for_statement_left_data(left) {
            ForStatementLeftData::VariableDeclaration(node) => HeaderNode {
                span: left_span,
                data: HeaderData::Variable {
                    node,
                    context: if of {
                        VariableContext::Iteration
                    } else {
                        VariableContext::ForIn
                    },
                },
            },
            ForStatementLeftData::AssignmentTarget(target) => {
                match ast.assignment_target_data(target) {
                    AssignmentTargetData::SimpleAssignmentTarget(target) => {
                        let node = self.iteration_source_target(
                            target,
                            Span::new(opening.span.end, left_span.start),
                            operator.span,
                            of,
                        )?;
                        let context = self.iteration_left_context(&node, of, awaited)?;
                        HeaderNode {
                            span: node.span(),
                            data: HeaderData::Target { node, context },
                        }
                    }
                    AssignmentTargetData::ArrayAssignmentTarget(node) => HeaderNode {
                        span: left_span,
                        data: HeaderData::Pattern(BindingPattern::ArrayPattern(node)),
                    },
                    AssignmentTargetData::ObjectAssignmentTarget(node) => HeaderNode {
                        span: left_span,
                        data: HeaderData::Pattern(BindingPattern::ObjectPattern(node)),
                    },
                }
            }
        };
        let iteration = IterationGap { operator, right };
        let right = self.for_expression_node(
            right,
            if of {
                ExpressionPosition::Nested
            } else {
                ExpressionPosition::ForHead
            },
        );
        let syntax = HeaderSyntax {
            comments,
            keyword_end,
            opening: opening.span,
            closing,
            tokens: [await_token, Some(opening), Some(operator), None],
            classic: false,
        };
        let nodes = [Some(left), Some(right), None];
        let mut header = self.prepare_for_header(&syntax, &nodes, body);
        if header
            .items
            .get(1)
            .and_then(Option::as_ref)
            .is_some_and(|item| !item.before.comments.is_empty())
            && header
                .items
                .iter()
                .flatten()
                .all(|item| item.mode == ContentMode::Formatted)
        {
            header.iteration = Some(iteration);
        }
        self.format_for_with_body(&header, body, span.end, punctuation, writer)
    }

    fn prepare_classic_final_tail(
        &self,
        header: &PreparedHeader<'ast, '_>,
    ) -> Result<Option<&'ast [Comment]>, FormatError> {
        let gap = &header.trailing;
        if header.empty
            || gap.source
            || gap.preceding != ContentMode::Formatted
            || gap.following != ContentMode::Formatted
            || !header.tokens[gap.tokens.start..gap.tokens.end]
                .iter()
                .flatten()
                .any(|prepared| prepared.token.ends_node)
        {
            return Ok(None);
        }
        let comments = &gap.comments[gap
            .comments
            .partition_point(|comment| comment.span.start < gap.span.start)..];
        if comments.is_empty()
            || !self
                .control_flow_comment_binding(comments, gap.span, None, false)?
                .allows_relocation(None)
        {
            return Ok(None);
        }
        let mut previous_end = gap.span.start;
        for comment in comments {
            if self.line_breaks(previous_end, comment.span.start) != 0 {
                return Ok(None);
            }
            previous_end = comment.span.end;
        }
        Ok(Some(comments))
    }

    fn for_expression_node(&self, node: Expr, position: ExpressionPosition) -> HeaderNode<'ast> {
        HeaderNode {
            span: node.span(self.ast),
            data: HeaderData::Expression {
                node,
                context: ExpressionContext::at(position),
            },
        }
    }

    fn prepare_for_initializer_expression(&self, node: Expr) -> HeaderData<'ast> {
        let (context, ignored) = self.prepare_for_initializer_context(
            node,
            ForInitializerGrammar::Expression,
            TailOwner::ParentOperation,
        );
        let Some(ignored) = ignored else {
            return HeaderData::Expression { node, context };
        };
        let span = ignored.leading;
        let leading = self.comments.in_range(span);
        let end = leading
            .last()
            .map_or(span.start, |comment| comment.span.end);
        let enclosure = context
            .for_initializer_requires_enclosure()
            .then_some(ignored.parenthesized);
        HeaderData::SourceExpression {
            node: ignored.expression,
            enclosure,
            context,
            leading,
            following: self.source_line_constraint(end, ignored.expression.span(self.ast).start),
        }
    }

    fn iteration_left_context(
        &self,
        target: &SourceTarget,
        of: bool,
        awaited: bool,
    ) -> Result<ExpressionContext, FormatError> {
        let mut context = ExpressionContext::at(ExpressionPosition::Nested);
        if target.has_retained_parentheses() {
            return Ok(context);
        }
        let Some(target) = target.simple() else {
            return Ok(context);
        };
        let mut expression = target;
        let root_data = self.expression_content(expression).data;
        let bare = matches!(
            root_data,
            swc_next_ecma_ast::ExprData::IdentifierReference(_)
        );
        let mut data = root_data;
        loop {
            if self.expression_is_parenthesized(data, context)
                || matches!(
                    data,
                    swc_next_ecma_ast::ExprData::ParenthesizedExpression(_)
                )
                || context.is_verbatim()
            {
                return Ok(ExpressionContext::at(ExpressionPosition::Nested));
            }
            if let swc_next_ecma_ast::ExprData::IdentifierReference(node) = data {
                let name = self.ast.get_utf8(node.name(self.ast));
                let needs =
                    name == "let" && (of || !bare) || name == "async" && of && !awaited && bare;
                return Ok(ExpressionContext::at(ExpressionPosition::Nested)
                    .with_lexical_head(needs.then_some(node)));
            }
            let Some((child, child_context)) = self.expression_head_operand(data, context)? else {
                return Ok(ExpressionContext::at(ExpressionPosition::Nested));
            };
            expression = child;
            context = child_context;
            data = self.expression_content(expression).data;
        }
    }

    fn prepare_for_header<'header>(
        &self,
        syntax: &'header HeaderSyntax<'ast>,
        nodes: &'header [Option<HeaderNode<'ast>>; 3],
        body: Stmt,
    ) -> PreparedHeader<'ast, 'header> {
        let mut nodes = nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| node.as_ref().map(|node| (index, node)))
            .peekable();
        let body_span = body.span(self.ast);
        let mut tokens = [None, None, None, None];
        let first_start = nodes
            .peek()
            .map_or(syntax.closing.start, |(_, node)| node.span.start);
        let empty = nodes.peek().is_none();
        let initial_relation = if empty {
            HeaderRelation::Empty
        } else {
            HeaderRelation::First
        };
        let mut before = self.prepare_for_gap(
            Span::new(syntax.keyword_end, first_start),
            initial_relation,
            body_span.start,
            syntax,
            &mut tokens,
            syntax.keyword_end,
        );
        let target = nodes.peek().map_or(body_span, |(_, node)| node.span);
        let count = before
            .comments
            .partition_point(|comment| comment.span.end <= syntax.opening.start);
        let before_opening = &before.comments[..count];
        let relocate = !before_opening.is_empty()
            && self
                .for_prefix_binding(before.comments, before.span, syntax.opening, target)
                .allows_relocation(Some(target));
        let body_head_end = if relocate && empty {
            syntax.keyword_end
        } else {
            syntax.closing.end
        };
        if relocate {
            before.owner = if empty {
                HeaderGapOwner::Body
            } else {
                HeaderGapOwner::Node
            };
            before.source = false;
            if !empty && before.following == ContentMode::Verbatim {
                before.head_gap = before
                    .comments
                    .last()
                    .map(|comment| self.source_line_constraint(comment.span.end, target.start));
            }
        }
        let mut items = [None, None, None];
        while let Some((index, node)) = nodes.next() {
            let (end, relation) = if let Some((_, next)) = nodes.peek() {
                let separator = syntax
                    .tokens
                    .iter()
                    .flatten()
                    .rfind(|token| {
                        token.span.start >= node.span.end && token.span.end <= next.span.start
                    })
                    .map_or(Span::new(node.span.end, node.span.end), |token| token.span);
                (next.span.start, HeaderRelation::Items(separator))
            } else {
                (syntax.closing.start, HeaderRelation::Body(syntax.closing))
            };
            let after = self.prepare_for_gap(
                Span::new(node.span.end, end),
                relation,
                body_span.start,
                syntax,
                &mut tokens,
                node.span.start,
            );
            let mode = if before.following == ContentMode::Verbatim
                || after.preceding == ContentMode::Verbatim
            {
                ContentMode::Verbatim
            } else {
                ContentMode::Formatted
            };
            items[index] = Some(PreparedHeaderItem { node, before, mode });
            before = after;
        }
        let body_mode = before.following;
        PreparedHeader {
            items,
            tokens,
            trailing: before,
            final_tail: None,
            iteration: None,
            closing: syntax.closing,
            classic: syntax.classic,
            empty,
            body_mode,
            body_head_end,
        }
    }

    fn for_prefix_binding(
        &self,
        comments: &[Comment],
        boundary: Span,
        opening: Span,
        target: Span,
    ) -> CommentBinding {
        if self.diagnostic_range_is_sensitive(boundary) {
            return CommentBinding::SourceBound;
        }
        self.comment_run_binding(comments.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.is_ordinary() {
                return CommentBinding::Unbound;
            }
            if semantics.suppression {
                return CommentBinding::BoundTo(target);
            }
            if comment.span.end <= opening.start
                && (semantics.optimizer || semantics.type_cast)
                && !semantics.diagnostic
                && !semantics.file_directive
            {
                // The opening parenthesis is a statement delimiter, so this
                // prefix has no expression subject for a target annotation.
                CommentBinding::Unbound
            } else {
                CommentBinding::SourceBound
            }
        })
    }

    fn prepare_for_gap<'header>(
        &self,
        span: Span,
        relation: HeaderRelation,
        body_start: u32,
        syntax: &'header HeaderSyntax<'ast>,
        tokens: &mut [Option<PreparedHeaderToken<'ast, 'header>>; 4],
        owner_start: u32,
    ) -> HeaderGap<'ast> {
        let mut syntax_tokens = syntax
            .tokens
            .iter()
            .enumerate()
            .filter_map(|(index, token)| token.as_ref().map(|token| (index, token)))
            .filter(|(_, token)| token.span.start >= span.start && token.span.end <= span.end)
            .peekable();
        let token_start = syntax_tokens.peek().map_or(0, |(index, _)| *index);
        let mut token_end = token_start;
        let comments = syntax.comments.in_range(Span::new(owner_start, span.end));
        let mut remaining = comments;
        while let Some((index, token)) = syntax_tokens.next() {
            let end = syntax_tokens
                .peek()
                .map_or(span.end, |(_, next)| next.span.start);
            let before_end =
                remaining.partition_point(|comment| comment.span.end <= token.span.start);
            let (before, after) = remaining.split_at(before_end);
            let after_end = after.partition_point(|comment| comment.span.end <= end);
            let (trailing, leading) = if token.ends_node {
                self.split_separator_comments(&after[..after_end], 0, token.span.end, Some(end))
            } else {
                (&after[..0], &after[..after_end])
            };
            let next = leading.first().map_or(end, |comment| comment.span.start);
            let boundary = self.prepare_comment_boundary(trailing, token.span.end, next);
            remaining = &after[trailing.len()..];
            tokens[index] = Some(PreparedHeaderToken {
                token,
                before,
                trailing,
                following: boundary.gap(),
            });
            token_end = index + 1;
        }
        let leading = if token_start == token_end {
            &comments[comments.partition_point(|comment| comment.span.end <= span.start)..]
        } else {
            remaining
        };
        let mut gap = HeaderGap {
            span,
            comments,
            tokens: token_start..token_end,
            leading,
            source: false,
            owner: HeaderGapOwner::Syntax,
            head_gap: None,
            preceding: ContentMode::Formatted,
            following: ContentMode::Formatted,
        };
        if !self.has_suppression {
            return gap;
        }
        let end = if matches!(relation, HeaderRelation::Body(_) | HeaderRelation::Empty) {
            body_start
        } else {
            span.end
        };
        let comments = syntax.comments.in_range(Span::new(span.start, end));
        for (run, position) in self.suppressed_comment_runs(comments, end) {
            let Some(last) = run.last() else {
                continue;
            };
            gap.source |= run.iter().any(|comment| {
                comment.span.end <= span.end
                    && crate::formatter::suppression::is_suppression_comment(self.ast, *comment)
            });
            let preceding = match relation {
                HeaderRelation::First | HeaderRelation::Empty => false,
                HeaderRelation::Items(separator) => match position {
                    SuppressionPosition::OwnLine => false,
                    SuppressionPosition::EndOfLine => true,
                    SuppressionPosition::Inline => last.span.end <= separator.start,
                },
                HeaderRelation::Body(closing) => {
                    last.span.end <= closing.start
                        && !matches!(position, SuppressionPosition::OwnLine)
                }
            };
            if preceding {
                gap.preceding = ContentMode::Verbatim;
            } else {
                gap.following = ContentMode::Verbatim;
            }
        }
        gap
    }

    fn format_for_with_body(
        &mut self,
        header: &PreparedHeader<'ast, '_>,
        statement: Stmt,
        end: u32,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let body = self
            .prepare_controlled_body(
                statement,
                header.body_head_end,
                ControlledBodyEnd::Enclosing(end),
                ControlledBodyRole::Body,
                header.body_mode,
            )
            .with_empty_block(if header.classic {
                EmptyBlock::Compact
            } else {
                EmptyBlock::Expanded
            })
            .with_enclosing_punctuation(punctuation);
        writer.group(false, |writer| {
            if header.classic {
                writer.group(false, |writer| self.format_for_header(header, writer))?;
            } else {
                self.format_for_header(header, writer)?;
            }
            self.format_controlled_body(body, writer)
        })
    }

    fn format_for_header(
        &mut self,
        header: &PreparedHeader<'ast, '_>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let classic = header.classic;
        let empty = header.empty;
        let closing = header.closing;
        writer.text("for");
        let mut write = |writer: &mut Writer<'ast>| {
            let mut pending = None;
            for (index, item) in header.items.iter().enumerate() {
                let Some(item) = item else {
                    continue;
                };
                let output = if index == 1
                    && let Some(iteration) = &header.iteration
                {
                    self.format_iteration_gap(
                        &item.before,
                        &header.tokens,
                        iteration,
                        pending,
                        writer,
                    )?
                } else {
                    self.format_for_gap(&item.before, &header.tokens, pending, writer)?
                };
                let first = index == 0;
                let flat = if first {
                    FlatSeparator::Empty
                } else {
                    FlatSeparator::Space
                };
                let default = |writer: &mut Writer<'ast>| {
                    if classic {
                        if first {
                            writer.soft_line();
                        } else {
                            writer.line();
                        }
                    } else if !first {
                        writer.space();
                    }
                };
                let mut layout = CommentRunLayout::GroupedLeading(item.node.span.start);
                let comments = match output {
                    HeaderGapOutput::Canonical {
                        leading,
                        following,
                        head_gap,
                        head_layout,
                        ..
                    } => {
                        layout = head_layout;
                        following.write(writer, flat, default);
                        if let Some(constraint) = head_gap {
                            self.comments.claim(leading)?;
                            self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
                            constraint.write(writer, FlatSeparator::Space, |_| {});
                            &[]
                        } else {
                            leading
                        }
                    }
                    HeaderGapOutput::Source {
                        following,
                        comment_end,
                    } => {
                        let flat = if comment_end {
                            FlatSeparator::Space
                        } else {
                            flat
                        };
                        following.write(writer, flat, default);
                        &[]
                    }
                };
                let head = NodeHead {
                    comments,
                    content_start: item.node.span.start,
                    layout,
                    gap_end: None,
                };
                pending = self.format_for_header_item(item.node, item.mode, head, writer)?;
            }
            self.format_for_final_gap(header, pending, writer)
        };
        let tail = if classic && !empty {
            writer.indent(write)?
        } else {
            write(writer)?
        };
        match tail {
            HeaderGapOutput::Source { following, .. } => {
                following.write(writer, FlatSeparator::Empty, |writer| {
                    if classic && !empty {
                        writer.soft_line();
                    }
                });
            }
            HeaderGapOutput::Canonical {
                start,
                leading,
                following,
                ..
            } => {
                self.comments.claim(leading)?;
                self.format_comment_run(leading, CommentRunLayout::SourceAfter(start), writer)?;
                let closing_gap = if leading.is_empty() {
                    following
                } else {
                    self.prepare_comment_boundary(leading, start, closing.start)
                        .gap()
                };
                closing_gap.write(writer, FlatSeparator::Empty, |writer| {
                    if leading
                        .last()
                        .is_some_and(|comment| comment.kind == swc_next_ecma_ast::CommentKind::Line)
                    {
                        writer.hard_line();
                    } else if classic && !empty {
                        writer.soft_line();
                    }
                });
            }
        }
        writer.text(")");
        Ok(())
    }

    fn format_for_final_gap(
        &mut self,
        header: &PreparedHeader<'ast, '_>,
        pending: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<HeaderGapOutput<'ast>, FormatError> {
        let gap = &header.trailing;
        let final_tail = header.final_tail.filter(|_| {
            // A child line suffix and the following packet need one source
            // boundary so a later comment cannot overtake the leased tail.
            pending.as_ref().is_none_or(|tail| {
                !tail
                    .comments
                    .iter()
                    .any(|comment| comment.kind == swc_next_ecma_ast::CommentKind::Line)
            })
        });
        if let Some(comments) = final_tail {
            self.format_expression_tail(pending, writer)?;
            self.comments.claim(comments)?;
            self.format_inline_trailing_comments(comments, writer)?;
            for prepared in header.tokens[gap.tokens.start..gap.tokens.end]
                .iter()
                .flatten()
            {
                if let Some(flat) = prepared.token.empty_slot {
                    match flat {
                        FlatSeparator::Empty => writer.soft_line(),
                        FlatSeparator::Space => writer.line(),
                    }
                }
                writer.text(prepared.token.text);
            }
            return Ok(HeaderGapOutput::Canonical {
                start: gap.span.end,
                leading: &[],
                following: CommentGap::Source(LineConstraint::Unconstrained),
                head_gap: None,
                head_layout: CommentRunLayout::Source,
            });
        }
        self.format_for_gap(gap, &header.tokens, pending, writer)
    }

    fn format_iteration_gap(
        &mut self,
        gap: &HeaderGap<'ast>,
        tokens: &[Option<PreparedHeaderToken<'ast, '_>>; 4],
        iteration: &IterationGap,
        pending: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<HeaderGapOutput<'ast>, FormatError> {
        if gap.source
            || !matches!(gap.owner, HeaderGapOwner::Syntax)
            || gap.preceding != ContentMode::Formatted
            || gap.following != ContentMode::Formatted
        {
            return self.format_for_gap(gap, tokens, pending, writer);
        }
        let start = pending
            .as_ref()
            .map_or(gap.span.start, |tail| tail.source_end);
        let comments = self.comments.in_range(Span::new(start, gap.span.end));
        let head = comments
            .last()
            .is_some_and(|comment| comment.kind == CommentKind::Line)
            && !self.diagnostic_range_is_sensitive(Span::new(start, gap.span.end))
            && matches!(
                self.expression_head_binding(comments, iteration.right),
                CommentBinding::Unbound
            );
        if !head {
            return self.format_for_gap(gap, tokens, pending, writer);
        }
        // This complete packet has a RHS owner. Its final Line is emitted here,
        // so it cannot become a suffix that crosses a later statement comment.
        writer.space();
        writer.text(iteration.operator.text);
        Ok(HeaderGapOutput::Canonical {
            start: iteration.operator.span.end,
            leading: comments,
            following: CommentGap::Source(LineConstraint::Unconstrained),
            head_gap: None,
            head_layout: CommentRunLayout::FlatLeading(gap.span.end),
        })
    }

    fn format_for_gap(
        &mut self,
        gap: &HeaderGap<'ast>,
        tokens: &[Option<PreparedHeaderToken<'ast, '_>>; 4],
        pending: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<HeaderGapOutput<'ast>, FormatError> {
        if gap.source {
            self.format_expression_tail(pending, writer)?;
            let source =
                self.format_delimited_source_gap(gap.span, DelimitedFollowing::Node, writer)?;
            return Ok(HeaderGapOutput::Source {
                following: source.following,
                comment_end: gap
                    .comments
                    .last()
                    .is_some_and(|comment| comment.span.end == source.end),
            });
        }
        let mut start = pending
            .as_ref()
            .map_or(gap.span.start, |tail| tail.source_end);
        let mut following = None;
        for prepared in tokens[gap.tokens.start..gap.tokens.end].iter().flatten() {
            let PreparedHeaderToken {
                token,
                before,
                trailing,
                following: after,
            } = prepared;
            let before = if matches!(gap.owner, HeaderGapOwner::Syntax) {
                &before[before.partition_point(|comment| comment.span.end <= start)..]
            } else {
                &before[..0]
            };
            let canonical = if token.ends_node && !before.is_empty() {
                self.prepare_delimited_trailing_run(before, start, token.span.start, true, None)
                    .filter(|run| run.len() == before.len())
            } else {
                None
            };
            let riding = canonical.is_some();
            if let Some(run) = canonical {
                self.claim_prepared_trailing_run(run, writer)?;
            } else if let Some(flat) = token.empty_slot {
                self.comments.claim(before)?;
                self.format_comment_run(before, CommentRunLayout::SourceAfter(start), writer)?;
                let constraint = match following.filter(|_| before.is_empty()) {
                    Some(gap) => gap,
                    None => self
                        .prepare_comment_boundary(before, start, token.span.start)
                        .gap(),
                };
                constraint.write(writer, flat, |writer| match flat {
                    FlatSeparator::Empty => writer.soft_line(),
                    FlatSeparator::Space => writer.line(),
                });
            } else if token.space || token.text == "(" {
                self.comments.claim(before)?;
                let constraint = match following.filter(|_| before.is_empty()) {
                    Some(gap) => gap,
                    None => {
                        let first = before
                            .first()
                            .map_or(token.span.start, |comment| comment.span.start);
                        CommentGap::Source(self.line_constraint(start, first))
                    }
                };
                constraint.write(writer, FlatSeparator::Space, Writer::space);
                self.format_comment_run(
                    before,
                    CommentRunLayout::FlatLeading(token.span.start),
                    writer,
                )?;
            } else {
                self.comments.claim(before)?;
                self.format_comment_run(before, CommentRunLayout::SourceAfter(start), writer)?;
                let constraint = match following.filter(|_| before.is_empty()) {
                    Some(gap) => gap,
                    None => self
                        .prepare_comment_boundary(before, start, token.span.start)
                        .gap(),
                };
                constraint.write(writer, FlatSeparator::Empty, |_| {});
            }
            writer.text(token.text);
            let trailing = if matches!(gap.owner, HeaderGapOwner::Syntax) {
                trailing
            } else {
                &trailing[..0]
            };
            self.comments.claim(trailing)?;
            let source_end = if riding {
                before
                    .last()
                    .map_or(token.span.end, |comment| comment.span.end)
            } else {
                token.span.end
            };
            self.format_comment_run(trailing, CommentRunLayout::SourceAfter(source_end), writer)?;
            start = trailing
                .last()
                .map_or(token.span.end, |comment| comment.span.end);
            following = Some(if riding && trailing.is_empty() {
                CommentGap::LineComment
            } else if matches!(gap.owner, HeaderGapOwner::Syntax) {
                *after
            } else {
                CommentGap::Source(LineConstraint::Unconstrained)
            });
        }
        let leading = match gap.owner {
            HeaderGapOwner::Body => &gap.comments[..0],
            HeaderGapOwner::Node => gap.comments,
            HeaderGapOwner::Syntax if following.is_some() => gap.leading,
            HeaderGapOwner::Syntax => {
                &gap.comments[gap
                    .comments
                    .partition_point(|comment| comment.span.end <= start)..]
            }
        };
        let following = match following {
            Some(gap) => gap,
            None => {
                let first = leading
                    .first()
                    .map_or(gap.span.end, |comment| comment.span.start);
                CommentGap::Source(self.line_constraint(start, first))
            }
        };
        Ok(HeaderGapOutput::Canonical {
            start,
            leading,
            following,
            head_gap: gap.head_gap,
            head_layout: CommentRunLayout::GroupedLeading(gap.span.end),
        })
    }

    fn format_for_header_item(
        &mut self,
        node: &HeaderNode<'ast>,
        mode: ContentMode,
        head: NodeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let initializer = match &node.data {
            HeaderData::Expression { context, .. }
            | HeaderData::SourceExpression { context, .. } => context.is_for_initializer(),
            HeaderData::Variable { context, .. } => {
                matches!(context, VariableContext::ForInitializer)
            }
            _ => false,
        };
        if initializer {
            return self.with_for_initializer(Some(node.span), |formatter| {
                formatter.format_for_header_content(node, mode, head, writer)
            });
        }
        self.format_for_header_content(node, mode, head, writer)
    }

    fn format_for_header_content(
        &mut self,
        node: &HeaderNode<'ast>,
        mode: ContentMode,
        head: NodeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.comments.claim(head.comments)?;
        if mode == ContentMode::Verbatim {
            self.format_comment_run(head.comments, head.layout, writer)?;
            self.format_verbatim(node.span, writer)?;
            return Ok(None);
        }
        match &node.data {
            HeaderData::Expression { node, context } => self
                .format_expression_with_leading_comments(
                    *node,
                    head,
                    context
                        .with_content_mode(mode)
                        .with_tail_owner(TailOwner::ParentOperation),
                    None,
                    writer,
                ),
            HeaderData::SourceExpression {
                node: expression,
                enclosure,
                context,
                leading,
                following,
            } => {
                self.format_comment_run(head.comments, head.layout, writer)?;
                if let Some(enclosure) = enclosure {
                    let span = enclosure.span(self.ast);
                    let before = &leading
                        [..leading.partition_point(|comment| comment.span.end <= span.start)];
                    self.comments.claim(before)?;
                    self.format_comment_run(before, CommentRunLayout::Leading(span.start), writer)?;
                    self.format_verbatim(span, writer)?;
                    return self.capture_expression_tail(Span::new(span.end, node.span.end), None);
                }
                self.comments.claim(leading)?;
                self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
                following.write(writer, FlatSeparator::Space, |_| {});
                let span = expression.span(self.ast);
                let tail = self.format_expression_with_leading_comments(
                    *expression,
                    NodeHead {
                        comments: &[],
                        content_start: span.start,
                        layout: CommentRunLayout::Source,
                        gap_end: None,
                    },
                    context
                        .with_content_mode(ContentMode::Verbatim)
                        .with_tail_owner(TailOwner::ParentOperation),
                    None,
                    writer,
                )?;
                self.capture_expression_tail(Span::new(span.end, node.span.end), tail)
            }
            HeaderData::Variable {
                node: declaration,
                context,
            } => {
                self.format_comment_run(head.comments, head.layout, writer)?;
                self.format_variable_declaration_with_context(*declaration, *context, writer)
                    .map(|(_, tail)| tail)
            }
            HeaderData::Target {
                node: target,
                context,
            } => {
                self.format_comment_run(head.comments, head.layout, writer)?;
                self.format_source_target(target, context.with_content_mode(mode), writer)
            }
            HeaderData::Pattern(pattern) => {
                self.format_comment_run(head.comments, head.layout, writer)?;
                self.format_declaration_binding(*pattern, mode, writer)
            }
        }
    }
}
