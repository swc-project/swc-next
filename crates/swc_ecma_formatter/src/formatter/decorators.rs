//! AST-selected decorator lists and their distinct source/core envelopes.

use swc_next_ecma_ast::{
    Ast, Class, ClassElement, ClassElementData, Comment, DeclData, Decorator,
    ExportDefaultDeclarationKindData, ExprData, GetSpan, PropertyKeyData, Span, Stmt, StmtData,
    SubRange,
};

use super::Formatter;
use super::comments::{CommentBinding, CommentRunLayout, NodeHead};
use super::diagnostics::FlatSeparator;
use super::expressions::{ExpressionContext, ExpressionPosition};
use super::source::is_source_whitespace;
use super::suppression::ContentMode;
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
pub(super) struct DecoratorSequence {
    first: Decorator,
    remaining: SubRange<Decorator>,
    physical: Span,
}

impl DecoratorSequence {
    fn new(ast: &Ast<'_>, mut nodes: SubRange<Decorator>) -> Option<Self> {
        let first = ast.get_node_in_sub_range(nodes.first()?);
        let remaining = nodes.split_off(1);
        let mut physical = first.span(ast);
        for id in remaining.iter() {
            let span = ast.get_node_in_sub_range(id).span(ast);
            physical.start = physical.start.min(span.start);
            physical.end = physical.end.max(span.end);
        }
        Some(Self {
            first,
            remaining,
            physical,
        })
    }

    fn envelope(self, native: Span) -> Span {
        Span::new(
            native.start.min(self.physical.start),
            native.end.max(self.physical.end),
        )
    }

    pub(super) fn with_envelope(self, envelope: Span) -> DecoratorPrefix {
        DecoratorPrefix {
            first: self.first,
            remaining: self.remaining,
            physical: self.physical,
            envelope,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct DecoratorPrefix {
    pub(super) first: Decorator,
    pub(super) remaining: SubRange<Decorator>,
    pub(super) envelope: Span,
    pub(super) physical: Span,
}

impl DecoratorPrefix {
    pub(super) fn new(ast: &Ast<'_>, nodes: SubRange<Decorator>, native: Span) -> Option<Self> {
        let sequence = DecoratorSequence::new(ast, nodes)?;
        let envelope = sequence.envelope(native);
        Some(sequence.with_envelope(envelope))
    }
}

#[derive(Clone, Copy)]
pub(super) enum StatementDecorators {
    Class {
        node: Class,
        sequence: DecoratorSequence,
    },
    Export {
        node: Class,
        sequence: DecoratorSequence,
        native: Span,
        default: bool,
    },
}

#[derive(Clone, Copy)]
pub(super) enum DecoratorLayout {
    Class,
    Member,
    Parameter,
}

pub(super) struct RemovedDecoratorHead<'ast> {
    comments: &'ast [Comment],
    core_start: u32,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn source_starts_decorator(&self, span: Span) -> bool {
        self.ast.source().as_bytes().get(span.start as usize) == Some(&b'@')
    }

    pub(super) fn removed_decorator_head(
        &self,
        native: Span,
        core_start: u32,
    ) -> Result<Option<RemovedDecoratorHead<'ast>>, FormatError> {
        if core_start == native.start || self.ast.comments().is_empty() {
            return Ok(None);
        }
        let comments = self.comments.in_range(Span::new(native.start, core_start));
        if comments.is_empty() {
            return Ok(None);
        }
        let binding = self.comment_binding(comments, Span::new(native.start, core_start), None);
        if !binding.allows_relocation(None) {
            let construct = match binding {
                CommentBinding::BoundTo(_) => "removed decorator bound source target",
                CommentBinding::SourceBound => "removed decorator unknown source binding",
                CommentBinding::Unbound => "removed decorator source binding",
            };
            return Err(FormatError::unsupported(construct, None, native));
        }
        Ok(Some(RemovedDecoratorHead {
            comments,
            core_start,
        }))
    }

    pub(super) fn format_removed_decorator_head(
        &mut self,
        head: RemovedDecoratorHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(head.comments)?;
        self.format_comment_run(
            head.comments,
            CommentRunLayout::Leading(head.core_start),
            writer,
        )
    }

    /// Only the source geometry exception is scanned; canonical empty lists return directly.
    pub(super) fn removed_member_core_start(&self, native: Span, key_start: u32) -> u32 {
        let comments = self.comments.in_range(Span::new(native.start, key_start));
        let mut end = key_start;
        let mut core_start = key_start;
        loop {
            let source = self.source(Span::new(native.start, end));
            end = native.start + source.trim_end_matches(is_source_whitespace).len() as u32;
            if let Some(comment) = comments
                .iter()
                .rev()
                .find(|comment| comment.span.end == end)
            {
                end = comment.span.start;
                continue;
            }
            let bytes = self.ast.source().as_bytes();
            if end > native.start && matches!(bytes[end as usize - 1], b'[' | b'*') {
                end -= 1;
                core_start = end;
                continue;
            }
            let mut start = end;
            while start > native.start && bytes[start as usize - 1].is_ascii_alphabetic() {
                start -= 1;
            }
            if start == end
                || start > native.start && matches!(bytes[start as usize - 1], b'@' | b'.')
            {
                return core_start;
            }
            match self.source(Span::new(start, end)) {
                "public" | "private" | "protected" | "static" | "declare" | "abstract"
                | "override" | "readonly" | "accessor" | "async" | "get" | "set" | "const"
                | "export" => {
                    end = start;
                    core_start = start;
                }
                _ => return core_start,
            }
        }
    }

    pub(super) fn removed_class_core_start(
        &self,
        class: Class,
        limit: u32,
    ) -> Result<u32, FormatError> {
        let ast = self.ast;
        let native = class.span(ast);
        let span = Span::new(native.start, limit);
        let comments = self.comments.in_range(span);
        let mut end = limit;
        let mut remaining = comments;
        let mut heritage = if class
            .super_class(ast)
            .is_some_and(|node| node.span(ast).start == limit)
        {
            Some("extends")
        } else if class
            .implements(ast)
            .first()
            .is_some_and(|id| ast.get_node_in_sub_range(id).span(ast).start == limit)
        {
            Some("implements")
        } else {
            None
        };
        let keyword = loop {
            let source = self.source(Span::new(native.start, end));
            end = native.start + source.trim_end_matches(is_source_whitespace).len() as u32;
            if let Some(comment) = remaining.last()
                && comment.span.end == end
            {
                end = comment.span.start;
                remaining = &remaining[..remaining.len() - 1];
                continue;
            }
            let source = self.source(Span::new(native.start, end));
            if let Some(keyword) = heritage
                && source.ends_with(keyword)
            {
                end -= keyword.len() as u32;
                heritage = None;
                continue;
            }
            if source.ends_with("class") {
                break end - 5;
            }
            return Err(self.unsupported("removed decorator class boundary", class));
        };
        if class.r#abstract(ast) || class.declare(ast) {
            Ok(self.removed_member_core_start(native, keyword))
        } else {
            Ok(keyword)
        }
    }

    pub(super) fn statement_decorators(
        &self,
        statement: Stmt,
    ) -> Option<(Span, StatementDecorators)> {
        let ast = self.ast;
        let native = statement.span(ast);
        let (node, export) = match ast.stmt_data(statement) {
            StmtData::Declaration(declaration) => match ast.decl_data(declaration) {
                DeclData::Class(node) => (node, None),
                _ => return None,
            },
            StmtData::ExportNamedDeclaration(export) => {
                let declaration = export.declaration(ast)?;
                let DeclData::Class(node) = ast.decl_data(declaration) else {
                    return None;
                };
                (node, Some(false))
            }
            StmtData::ExportDefaultDeclaration(export) => {
                let ExportDefaultDeclarationKindData::Class(node) =
                    ast.export_default_declaration_kind_data(export.declaration(ast))
                else {
                    return None;
                };
                (node, Some(true))
            }
            _ => return None,
        };
        let sequence = DecoratorSequence::new(ast, node.decorators(ast))?;
        let envelope = sequence.envelope(native);
        let decorators = match export {
            Some(default) => StatementDecorators::Export {
                node,
                sequence,
                native,
                default,
            },
            None => StatementDecorators::Class { node, sequence },
        };
        Some((envelope, decorators))
    }

    pub(super) fn member_decorators(&self, node: ClassElement) -> Option<DecoratorPrefix> {
        let ast = self.ast;
        let decorators = match ast.class_element_data(node) {
            ClassElementData::MethodDefinition(node) => node.decorators(ast),
            ClassElementData::TsMethodDefinition(node) => node.decorators(ast),
            ClassElementData::PropertyDefinition(node) => node.decorators(ast),
            _ => return None,
        };
        DecoratorPrefix::new(ast, decorators, node.span(ast))
    }

    /// This bound excludes decorator expressions, not AST-selected modifiers.
    pub(super) fn decorator_core_start(
        &self,
        prefix: DecoratorPrefix,
        native: Span,
        limit: u32,
    ) -> Result<u32, FormatError> {
        let mut start = native.start.max(prefix.physical.end);
        if start > limit {
            return Err(FormatError::unsupported(
                "decorator core boundary",
                None,
                native,
            ));
        }
        if !self.ast.comments().is_empty() {
            for comment in self.comments.in_range(Span::new(start, limit)) {
                if !self
                    .source(Span::new(start, comment.span.start))
                    .chars()
                    .all(is_source_whitespace)
                {
                    break;
                }
                start = comment.span.end;
            }
        }
        let whitespace = self
            .source(Span::new(start, limit))
            .chars()
            .take_while(|&character| is_source_whitespace(character))
            .map(|character| character.len_utf8() as u32)
            .sum::<u32>();
        Ok(start + whitespace)
    }

    pub(super) fn format_decorators(
        &mut self,
        prefix: DecoratorPrefix,
        layout: DecoratorLayout,
        core_start: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let source_break = self.line_breaks(prefix.physical.end, core_start) != 0
            || prefix.remaining.iter().any(|id| {
                let node = ast.get_node_in_sub_range(id);
                let before =
                    ast.source()[..node.span(ast).start as usize].trim_end_matches([' ', '\t']);
                before.ends_with(['\n', '\r'])
            });
        if matches!(layout, DecoratorLayout::Class) {
            // The complete class frame owns the mandatory decorator break.
            writer.break_parent();
        }
        writer.group(
            matches!(layout, DecoratorLayout::Class) || source_break,
            |writer| {
                let mut current = prefix.first;
                let mut remaining = prefix.remaining;
                loop {
                    let next = remaining.first().map(|id| ast.get_node_in_sub_range(id));
                    let span = current.span(ast);
                    let following = next.map_or(core_start, |node| node.span(ast).start);
                    if following < span.end {
                        return Err(FormatError::unsupported(
                            "decorator source order",
                            None,
                            prefix.envelope,
                        ));
                    }
                    let expression = current.expression(ast);
                    let marker_end = span.start + u32::from(self.source(span).starts_with('@'));
                    let expression_span = expression.span(ast);
                    let physical_head =
                        if expression_span.start >= marker_end && expression_span.end <= span.end {
                            Span::new(marker_end, expression_span.start)
                        } else {
                            Span::new(marker_end, span.end.max(marker_end))
                        };
                    let comments = self.comments.between(physical_head)?;
                    if expression_span.start < marker_end || expression_span.end > span.end {
                        let binding = self.comment_binding(comments, physical_head, None);
                        if !binding.allows_relocation(None) {
                            return Err(FormatError::unsupported(
                                "replaced decorator unknown source binding",
                                None,
                                span,
                            ));
                        }
                    }
                    let expression_start = physical_head.end;
                    let content = self.expression_content(expression);
                    let mode = if self.has_suppression {
                        self.content_mode(comments)
                    } else {
                        ContentMode::Formatted
                    };
                    self.comments.claim(comments)?;
                    writer.text("@");
                    let tail = self.format_prepared_expression_with_leading_comments(
                        expression,
                        content,
                        NodeHead {
                            comments,
                            content_start: expression_start,
                            layout: CommentRunLayout::Source,
                            gap_end: (!comments.is_empty()).then_some(expression_start),
                        },
                        ExpressionContext::at(ExpressionPosition::Decorator)
                            .with_content_mode(mode),
                        writer,
                    )?;
                    self.format_expression_tail(tail, writer)?;
                    let comments = self.comments.between(Span::new(span.end, following))?;
                    let boundary = self.prepare_comment_boundary(comments, span.end, following);
                    self.format_comment_boundary(&boundary, writer)?;
                    boundary
                        .gap()
                        .write(writer, FlatSeparator::Space, |writer| {
                            if matches!(layout, DecoratorLayout::Class) {
                                writer.hard_line();
                            } else {
                                writer.line();
                            }
                        });
                    let Some(node) = next else { break };
                    current = node;
                    remaining = remaining.split_off(1);
                }
                Ok(())
            },
        )
    }

    pub(super) fn decorator_expression_is_unparenthesized(&self, mut data: ExprData) -> bool {
        if let ExprData::ChainExpression(chain) = data {
            data = self.ast.expr_data(chain.expression(self.ast));
        }
        if let ExprData::CallExpression(call) = data {
            return !call.optional(self.ast)
                && self.decorator_member_expression(self.ast.expr_data(call.callee(self.ast)));
        }
        self.decorator_member_expression(data)
    }

    fn decorator_member_expression(&self, mut data: ExprData) -> bool {
        loop {
            match data {
                ExprData::IdentifierReference(_) => return true,
                ExprData::MemberExpression(member)
                    if !member.computed(self.ast)
                        && !member.optional(self.ast)
                        && matches!(
                            self.ast.property_key_data(member.property(self.ast)),
                            PropertyKeyData::IdentifierName(_)
                        ) =>
                {
                    data = self.ast.expr_data(member.object(self.ast));
                }
                _ => return false,
            }
        }
    }
}
