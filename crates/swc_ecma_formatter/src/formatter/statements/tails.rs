//! Statement-owned suppression tails and their actual following relation.

use swc_next_ecma_ast::{Comment, DeclData, GetSpan, Span, StmtData};

use super::{Formatter, StatementItem, StatementSourceTerminator};
use crate::formatter::diagnostics::LineConstraint;
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};

#[derive(Clone, Copy)]
pub(super) enum StatementTailRelation {
    Terminal,
    Statement,
    Keyword,
}

#[derive(Clone, Copy)]
enum StatementTailOwner {
    Current,
    Following,
    Enclosing,
}

#[derive(Clone, Copy)]
pub(super) struct StatementTailModes {
    pub(super) current: ContentMode,
    pub(super) following: ContentMode,
    pub(super) enclosing: ContentMode,
}

#[derive(Clone, Copy)]
pub(super) struct StatementSourcePacket {
    pub(super) span: Span,
    pub(super) following: LineConstraint,
}

pub(super) struct StatementSuppressionTail {
    pub(super) content: StatementContent,
    pub(super) terminator: Option<StatementSourceTerminator>,
    pub(super) modes: StatementTailModes,
    pub(super) following_gap: Option<LineConstraint>,
    pub(super) source_packet: Option<StatementSourcePacket>,
}

#[derive(Clone, Copy)]
pub(super) enum StatementContent {
    Semicolon(u32),
    Complete(u32),
}

impl StatementContent {
    pub(super) fn end(self) -> u32 {
        match self {
            Self::Semicolon(end) | Self::Complete(end) => end,
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    /// Probe the lexical tail before following a compound statement's final body.
    pub(super) fn prepare_statement_suppression_tail(
        &self,
        statement: StatementItem,
        limit: u32,
        relation: StatementTailRelation,
    ) -> Option<StatementSuppressionTail> {
        if !self.has_suppression {
            return None;
        }
        let span = statement.span(self.ast);
        self.prepare_statement_suppression_tail_at(statement, span, limit, relation)
    }

    pub(super) fn prepare_statement_suppression_tail_at(
        &self,
        statement: StatementItem,
        span: Span,
        limit: u32,
        relation: StatementTailRelation,
    ) -> Option<StatementSuppressionTail> {
        if !self.has_suppression {
            return None;
        }
        let comments = self.comments.in_range(Span::new(span.start, limit));
        let mut first = comments.len();
        let mut following = limit;
        let mut suppression = false;
        for (index, comment) in comments.iter().enumerate().rev() {
            let gap = self.source(Span::new(comment.span.end, following));
            if !gap
                .chars()
                .all(|character| is_source_whitespace(character) || character == ';')
            {
                break;
            }
            first = index;
            following = comment.span.start;
            suppression |= is_suppression_comment(self.ast, *comment);
        }
        if !suppression {
            return None;
        }
        let content = self.statement_content(statement);
        let content_end = content.end();
        let comments = &comments[first..];
        let first = comments.partition_point(|comment| comment.span.start < content_end);
        let comments = &comments[first..];
        if self.content_mode(comments) != ContentMode::Verbatim {
            return None;
        }
        let source_terminator = self.statement_source_terminator(span);
        let split = source_terminator.map_or(0, |terminator| {
            comments.partition_point(|comment| comment.span.end <= terminator.start)
        });
        let (before, after) = comments.split_at(split);
        let modes = self.statement_tail_modes(comments, limit, relation);
        let terminator =
            source_terminator.filter(|_| self.content_mode(before) == ContentMode::Verbatim);
        let following_gap = match (modes.following, after.is_empty(), terminator, relation) {
            (ContentMode::Verbatim, true, Some(terminator), StatementTailRelation::Statement) => {
                Some(match self.line_breaks(terminator.start + 1, limit) {
                    0 => LineConstraint::SameLine,
                    count => LineConstraint::NewLine(count),
                })
            }
            _ => None,
        };
        let source_packet = if modes.current == ContentMode::Verbatim {
            self.statement_source_packet(span, content_end, comments, limit)
        } else {
            None
        };
        Some(StatementSuppressionTail {
            content,
            terminator,
            modes,
            following_gap,
            source_packet,
        })
    }

    /// Empty sequence semicolons can separate distinct, already active ignore owners.
    fn statement_source_packet(
        &self,
        statement: Span,
        content_end: u32,
        comments: &[Comment],
        limit: u32,
    ) -> Option<StatementSourcePacket> {
        let mut previous = content_end;
        let mut empty_terminator = false;
        let mut preserve = false;
        for &comment in comments {
            let gap = self.source(Span::new(previous, comment.span.start));
            if !gap
                .chars()
                .all(|character| is_source_whitespace(character) || character == ';')
            {
                return None;
            }
            if comment.span.start >= statement.end {
                let start = previous.max(statement.end);
                empty_terminator |= self
                    .source(Span::new(start, comment.span.start))
                    .contains(';');
            }
            preserve |= empty_terminator && is_suppression_comment(self.ast, comment);
            previous = comment.span.end;
        }
        if !preserve {
            return None;
        }
        let remaining = self.source(Span::new(previous, limit));
        let end = remaining
            .rfind(';')
            .map_or(previous, |offset| previous + offset as u32 + 1);
        let following = match self.line_breaks(end, limit) {
            0 => LineConstraint::SameLine,
            count => LineConstraint::NewLine(count),
        };
        Some(StatementSourcePacket {
            span: Span::new(content_end, end),
            following,
        })
    }

    pub(super) fn statement_content_end(&self, item: StatementItem) -> u32 {
        self.statement_content(item).end()
    }

    pub(super) fn statement_content(&self, item: StatementItem) -> StatementContent {
        use StatementContent::{Complete, Semicolon};

        let mut statement = match item {
            StatementItem::Directive(node) => {
                return Semicolon(node.expression(self.ast).span(self.ast).end);
            }
            StatementItem::Statement(statement) => statement,
        };
        loop {
            match self.ast.stmt_data(statement) {
                StmtData::IfStatement(node) => {
                    statement = match node.alternate(self.ast) {
                        Some(alternate) => alternate,
                        None => node.consequent(self.ast),
                    };
                }
                StmtData::ForStatement(node) => statement = node.body(self.ast),
                StmtData::ForInStatement(node) => statement = node.body(self.ast),
                StmtData::ForOfStatement(node) => statement = node.body(self.ast),
                StmtData::WhileStatement(node) => {
                    statement = node.body(self.ast);
                }
                StmtData::DoWhileStatement(node) => {
                    let expression = node.test(self.ast).span(self.ast);
                    let end = node.span(self.ast).end;
                    let span = Span::new(expression.end, end);
                    let closing = self.find_token(span, self.comments.in_range(span), ")");
                    return Semicolon(closing.map_or(end, |closing| closing + 1));
                }
                StmtData::WithStatement(node) => {
                    statement = node.body(self.ast);
                }
                StmtData::LabeledStatement(node) => {
                    statement = node.body(self.ast);
                }
                StmtData::BreakStatement(node) => {
                    let end = node
                        .label(self.ast)
                        .map_or(node.span(self.ast).start + "break".len() as u32, |label| {
                            label.span(self.ast).end
                        });
                    return Semicolon(end);
                }
                StmtData::ContinueStatement(node) => {
                    let end = node.label(self.ast).map_or(
                        node.span(self.ast).start + "continue".len() as u32,
                        |label| label.span(self.ast).end,
                    );
                    return Semicolon(end);
                }
                StmtData::ExpressionStatement(node) => {
                    return Semicolon(node.expression(self.ast).span(self.ast).end);
                }
                StmtData::ReturnStatement(node) => {
                    let end = node.argument(self.ast).map_or(
                        node.span(self.ast).start + "return".len() as u32,
                        |argument| argument.span(self.ast).end,
                    );
                    return Semicolon(end);
                }
                StmtData::ThrowStatement(node) => {
                    return Semicolon(node.argument(self.ast).span(self.ast).end);
                }
                StmtData::Declaration(declaration) => {
                    let DeclData::VariableDeclaration(variable) = self.ast.decl_data(declaration)
                    else {
                        return Complete(declaration.span(self.ast).end);
                    };
                    let Some(last) = variable.declarators(self.ast).last() else {
                        return Complete(variable.span(self.ast).end);
                    };
                    return Semicolon(self.ast.get_node_in_sub_range(last).span(self.ast).end);
                }
                StmtData::DebuggerStatement(node) => {
                    return Semicolon(node.span(self.ast).start + "debugger".len() as u32);
                }
                _ => return Complete(statement.span(self.ast).end),
            }
        }
    }

    pub(in crate::formatter) fn statement_source_terminator(
        &self,
        span: Span,
    ) -> Option<StatementSourceTerminator> {
        let start = span.end.checked_sub(1)?;
        (self.ast.source().as_bytes().get(start as usize) == Some(&b';'))
            .then_some(StatementSourceTerminator { start })
    }

    /// Resolve each line and punctuation packet before its comments are claimed.
    pub(super) fn statement_annotation_tail_modes(
        &self,
        comments: &[Comment],
        following_start: u32,
    ) -> StatementTailModes {
        self.statement_tail_modes(comments, following_start, StatementTailRelation::Statement)
    }

    fn statement_tail_modes(
        &self,
        mut comments: &[Comment],
        following_start: u32,
        relation: StatementTailRelation,
    ) -> StatementTailModes {
        let mut modes = StatementTailModes {
            current: ContentMode::Formatted,
            following: ContentMode::Formatted,
            enclosing: ContentMode::Formatted,
        };
        if !self.has_suppression {
            return modes;
        }
        while !comments.is_empty() {
            let count = comments
                .windows(2)
                .position(|pair| {
                    self.line_breaks(pair[0].span.end, pair[1].span.start) != 0
                        || self
                            .source(Span::new(pair[0].span.end, pair[1].span.start))
                            .contains(';')
                })
                .map_or(comments.len(), |index| index + 1);
            let (run, remaining) = comments.split_at(count);
            let following = remaining
                .first()
                .map_or(following_start, |comment| comment.span.start);
            let position = self.suppression_position(run, following);
            let owner = match (position, relation) {
                (Some(SuppressionPosition::OwnLine), StatementTailRelation::Keyword) => {
                    StatementTailOwner::Enclosing
                }
                (Some(SuppressionPosition::OwnLine), StatementTailRelation::Statement) => {
                    StatementTailOwner::Following
                }
                (Some(SuppressionPosition::Inline), StatementTailRelation::Statement) => {
                    let end = run.last().map_or(following, |comment| comment.span.end);
                    if self.source(Span::new(end, following)).contains(';') {
                        StatementTailOwner::Current
                    } else {
                        StatementTailOwner::Following
                    }
                }
                (Some(_), _) => StatementTailOwner::Current,
                (None, _) => {
                    comments = remaining;
                    continue;
                }
            };
            match owner {
                StatementTailOwner::Current => modes.current = ContentMode::Verbatim,
                StatementTailOwner::Following => modes.following = ContentMode::Verbatim,
                StatementTailOwner::Enclosing => modes.enclosing = ContentMode::Verbatim,
            }
            comments = remaining;
        }
        modes
    }
}
