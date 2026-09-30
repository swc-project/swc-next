//! Function headers and their actual parameter/body boundaries.
use super::Formatter;
use super::comments::{CommentBinding, CommentRunLayout};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::parameters::SourceParameters;
use super::statements::{IncomingStatement, StatementBodyLeading};
use super::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{
    BindingIdentifier, Comment, CommentKind, Function, FunctionBody, GetSpan, Span, Stmt, StmtData,
};
pub(super) struct SourceFunctionSignature<'ast> {
    span: Span,
    parameters: SourceParameters,
    body: SourceFunctionBody<'ast>,
    preceding: Option<BindingIdentifier>,
}
pub(super) struct PreparedFunctionSignature<'ast> {
    parameters: SourceParameters,
    body: PreparedFunctionBody<'ast>,
    preceding: Option<BindingIdentifier>,
}
pub(super) enum SignatureBoundaryRole {
    Function { has_preceding: bool },
    Arrow { has_parameters: bool },
}

pub(super) struct SignatureBodyModes {
    preceding: ContentMode,
    pub(super) body: ContentMode,
    pub(super) first_statement: Option<IncomingStatement>,
    pub(super) constraint: LineConstraint,
}

impl SourceFunctionSignature<'_> {
    fn set_leading_mode(&mut self, mode: ContentMode) {
        if self.parameters.has_parameters() {
            self.parameters.set_leading_mode(mode);
        } else if mode == ContentMode::Verbatim {
            self.body.mode = mode;
        }
    }
}
impl PreparedFunctionSignature<'_> {
    fn name_mode(&self, name: BindingIdentifier) -> ContentMode {
        if self.preceding == Some(name) {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        }
    }
}
pub(super) struct FunctionNameModes {
    pub(super) name: ContentMode,
    pub(super) following: ContentMode,
}

struct SourceFunctionBody<'a> {
    body: FunctionBody,
    signature_end: u32,
    comments: &'a [Comment],
    mode: ContentMode,
    first_statement: Option<IncomingStatement>,
    constraint: LineConstraint,
}

pub(super) struct PreparedFunctionBody<'ast> {
    body: FunctionBody,
    signature_end: u32,
    boundary: FunctionBodyBoundary<'ast>,
    leading: StatementBodyLeading<'ast>,
    mode: ContentMode,
    first_statement: Option<IncomingStatement>,
    constraint: LineConstraint,
}

enum FunctionBodyBoundary<'ast> {
    Source(&'ast [Comment]),
    Formatted(&'ast [Comment]),
}

pub(super) enum FunctionBodyAttachment {
    Function,
    Arrow,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_function(
        &mut self,
        function: Function,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let signature = self.source_function_signature(function)?;
        if signature.parameters.enclosing_mode() == ContentMode::Verbatim {
            return self.format_verbatim(function.span(self.ast), writer);
        }
        let body = self.format_function_head(function, signature, writer)?;
        self.format_function_body_boundary(body, writer)
    }
    fn format_function_head(
        &mut self,
        function: Function,
        mut signature: SourceFunctionSignature<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedFunctionBody<'ast>, FormatError> {
        let ast = self.ast;
        let header_start = signature.span.start;
        let start = function.span(ast).start;
        let name = function.id(ast);
        let head_end = name.map_or(header_start, |name| name.span(ast).start);
        let function_start = if function.r#async(ast) {
            let gap = Span::new(start + 5, head_end);
            let comments = self.comments.in_range(gap);
            let Some(token) = self.find_token(gap, comments, "function") else {
                return Err(self.unsupported("async function keyword boundary", function));
            };
            Some(token)
        } else {
            None
        };
        let keyword_end = function_start.unwrap_or(start) + 8;
        let star = if function.generator(ast) {
            let gap = Span::new(keyword_end, head_end);
            let comments = self.comments.in_range(gap);
            let Some(token) = self.find_token(gap, comments, "*") else {
                return Err(self.unsupported("generator star boundary", function));
            };
            Some(token)
        } else {
            None
        };
        let header_end = star.map_or(keyword_end, |star| star + 1);
        let mut name_modes = match name {
            Some(name) => self.function_name_modes(Span::new(name.span(ast).end, header_start)),
            None => FunctionNameModes {
                name: ContentMode::Formatted,
                following: if self.has_suppression {
                    self.content_mode(self.comments.in_range(Span::new(start, header_start)))
                } else {
                    ContentMode::Formatted
                },
            },
        };
        if self.has_suppression
            && let Some(name) = name
            && self.content_mode(
                self.comments
                    .in_range(Span::new(start, name.span(ast).start)),
            ) == ContentMode::Verbatim
        {
            name_modes.name = ContentMode::Verbatim;
        }
        signature.set_leading_mode(name_modes.following);
        let signature = self.prepare_function_signature(signature);
        if let Some(name) = name
            && signature.name_mode(name) == ContentMode::Verbatim
        {
            name_modes.name = ContentMode::Verbatim;
        }
        if let Some(function_start) = function_start {
            writer.text("async");
            self.format_source_token_gap(start + 5, function_start, true, writer)?;
        }
        writer.text("function");
        if let Some(star) = star {
            self.format_source_token_gap(keyword_end, star, false, writer)?;
            writer.text("*");
        }
        self.format_source_token_gap(header_end, head_end, true, writer)?;
        if let Some(name) = name {
            let span = name.span(ast);
            if name_modes.name == ContentMode::Verbatim {
                self.format_verbatim(span, writer)?;
            } else {
                writer.text(ast.get_utf8(name.name(ast)));
            }
            self.format_source_token_gap(span.end, header_start, false, writer)?;
        }
        writer.group(false, |writer| {
            self.format_source_parameters(signature.parameters, writer)
        })?;
        Ok(signature.body)
    }
    pub(super) fn function_name_modes(&self, span: Span) -> FunctionNameModes {
        let mut modes = FunctionNameModes {
            name: ContentMode::Formatted,
            following: ContentMode::Formatted,
        };
        if !self.has_suppression {
            return modes;
        }
        let comments = self.comments.in_range(span);
        for position in self.suppression_positions(comments, span.end) {
            match position {
                SuppressionPosition::OwnLine => modes.following = ContentMode::Verbatim,
                SuppressionPosition::Inline | SuppressionPosition::EndOfLine => {
                    modes.name = ContentMode::Verbatim;
                }
            }
        }
        modes
    }

    fn source_function_signature(
        &self,
        function: Function,
    ) -> Result<SourceFunctionSignature<'ast>, FormatError> {
        let ast = self.ast;
        if function.this_param(ast).is_some()
            || function.return_type(ast).is_some()
            || function.type_parameters(ast).is_some()
        {
            return Err(self.unsupported("typed function", function));
        }
        let params = function.params(ast);
        let body = function.body(ast);
        let mut parameters = self.source_function_parameters(params)?;
        let span = params.span(ast);
        let body_span = body.span(ast);
        let comments = self.comments.in_range(Span::new(span.end, body_span.start));
        let previous = if self.content_mode(comments) == ContentMode::Verbatim {
            function.id(ast)
        } else {
            None
        };
        let modes = self.signature_body_modes(
            body,
            comments,
            body_span.start,
            span.end,
            SignatureBoundaryRole::Function {
                has_preceding: parameters.has_parameters() || previous.is_some(),
            },
        );
        let preceding = if modes.preceding == ContentMode::Verbatim {
            if parameters.has_parameters() {
                parameters.suppress_last(ast)?;
                None
            } else {
                previous
            }
        } else {
            None
        };
        Ok(SourceFunctionSignature {
            span,
            parameters,
            body: SourceFunctionBody {
                body,
                signature_end: span.end,
                comments,
                mode: modes.body,
                first_statement: modes.first_statement,
                constraint: modes.constraint,
            },
            preceding,
        })
    }
    fn prepare_function_signature(
        &self,
        signature: SourceFunctionSignature<'ast>,
    ) -> PreparedFunctionSignature<'ast> {
        PreparedFunctionSignature {
            parameters: signature.parameters,
            body: self.prepare_function_body(signature.body),
            preceding: signature.preceding,
        }
    }
    fn prepare_function_body(
        &self,
        source: SourceFunctionBody<'ast>,
    ) -> PreparedFunctionBody<'ast> {
        let SourceFunctionBody {
            body,
            signature_end,
            comments,
            mode,
            first_statement,
            constraint,
        } = source;
        let mode = self.function_body_content_mode(body, mode);
        let gap = Span::new(signature_end, body.span(self.ast).start);
        let ordinary = !comments.is_empty()
            && mode == ContentMode::Formatted
            && first_statement.is_none()
            && matches!(
                self.comment_binding(comments, gap, None),
                CommentBinding::Unbound
            );
        let (boundary, leading) = if ordinary {
            self.prepare_function_body_leading(comments, body, FunctionBodyAttachment::Function)
        } else {
            (comments, StatementBodyLeading::empty())
        };
        let boundary = if ordinary
            && boundary
                .iter()
                .all(|comment| comment.kind == CommentKind::Block)
        {
            FunctionBodyBoundary::Formatted(boundary)
        } else {
            FunctionBodyBoundary::Source(boundary)
        };
        PreparedFunctionBody {
            body,
            signature_end,
            boundary,
            leading,
            mode,
            first_statement,
            constraint,
        }
    }

    pub(super) fn function_body_content_mode(
        &self,
        body: FunctionBody,
        mode: ContentMode,
    ) -> ContentMode {
        if mode == ContentMode::Verbatim
            || !self.has_suppression
            || self.function_body_requires_break(body)
        {
            return mode;
        }
        let span = body.span(self.ast);
        self.content_mode(
            self.comments
                .in_range(Span::new(span.start + 1, span.end - 1)),
        )
    }

    pub(super) fn prepare_function_body_leading(
        &self,
        comments: &'ast [Comment],
        body: FunctionBody,
        attachment: FunctionBodyAttachment,
    ) -> (&'ast [Comment], StatementBodyLeading<'ast>) {
        let Some(first_line) = comments
            .iter()
            .position(|comment| comment.kind == CommentKind::Line)
        else {
            return (comments, StatementBodyLeading::empty());
        };
        // A contiguous body suffix retains source order even when other packets
        // remain at the signature. Mixed attachment cannot reorder that stream.
        if comments[first_line..]
            .iter()
            .any(|comment| comment.kind != CommentKind::Line)
        {
            return (comments, StatementBodyLeading::empty());
        }
        let mut start = first_line;
        if matches!(attachment, FunctionBodyAttachment::Arrow) {
            // Arrow own-line comments lead the block itself; only an end-of-line
            // packet after a real parameter belongs to the first body item.
            if comments.len() != first_line + 1
                || self.source_position_starts_line(comments[first_line].span.start)
            {
                return (comments, StatementBodyLeading::empty());
            }
            while start != 0 && self.source_comment_gap_is_run(comments[start - 1], comments[start])
            {
                if self.source_position_starts_line(comments[start - 1].span.start) {
                    break;
                }
                start -= 1;
            }
        }
        let ast = self.ast;
        let target = body
            .body(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .find(|statement: &Stmt| {
                !matches!(ast.stmt_data(*statement), StmtData::EmptyStatement(_))
            });
        if !body.directives(ast).is_empty() {
            let span = body.span(ast);
            let limit = target.map_or(span.end - 1, |statement| statement.span(ast).start);
            // Moving past existing directive comments would reverse source order.
            // Keep the original boundary packet when those owners precede the target.
            if !self
                .comments
                .in_range(Span::new(span.start + 1, limit))
                .is_empty()
            {
                return (comments, StatementBodyLeading::empty());
            }
        }
        let (boundary, leading) = comments.split_at(start);
        (boundary, StatementBodyLeading::new(leading, target))
    }

    pub(super) fn signature_body_modes(
        &self,
        body: FunctionBody,
        comments: &[Comment],
        body_start: u32,
        source_end: u32,
        role: SignatureBoundaryRole,
    ) -> SignatureBodyModes {
        let end = comments
            .last()
            .map_or(source_end, |comment| comment.span.end);
        let mut modes = SignatureBodyModes {
            preceding: ContentMode::Formatted,
            body: ContentMode::Formatted,
            first_statement: None,
            constraint: self.line_constraint(end, body_start),
        };
        if !self.has_suppression {
            return modes;
        }
        let mut needs_first_statement = false;
        for (run, position) in self.suppressed_comment_runs(comments, body_start) {
            for &comment in run {
                if !is_suppression_comment(self.ast, comment) {
                    continue;
                }
                match role {
                    SignatureBoundaryRole::Function { .. } if comment.kind == CommentKind::Line => {
                        needs_first_statement = true;
                    }
                    SignatureBoundaryRole::Function {
                        has_preceding: true,
                    } if matches!(position, SuppressionPosition::EndOfLine) => {
                        modes.preceding = ContentMode::Verbatim;
                    }
                    SignatureBoundaryRole::Arrow {
                        has_parameters: true,
                    } if comment.kind == CommentKind::Line
                        || matches!(position, SuppressionPosition::EndOfLine) =>
                    {
                        needs_first_statement = true;
                    }
                    _ => modes.body = ContentMode::Verbatim,
                }
            }
        }
        if needs_first_statement {
            let ast = self.ast;
            modes.first_statement = body
                .body(ast)
                .iter()
                .map(|id| ast.get_node_in_sub_range(id))
                .find(|&statement| !matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_)))
                .map(|statement| IncomingStatement {
                    span: statement.span(ast),
                    mode: ContentMode::Verbatim,
                });
        }
        if modes.preceding == ContentMode::Verbatim
            || modes.body == ContentMode::Verbatim
            || modes.first_statement.is_some()
        {
            modes.constraint = self.source_line_constraint(end, body_start);
        }
        modes
    }

    pub(super) fn format_function_body_boundary(
        &mut self,
        source: PreparedFunctionBody<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let PreparedFunctionBody {
            body,
            signature_end,
            boundary,
            leading,
            mode,
            first_statement,
            constraint,
        } = source;
        match boundary {
            FunctionBodyBoundary::Source(comments) => {
                self.comments.claim(comments)?;
                self.format_comment_run(
                    comments,
                    CommentRunLayout::SourceAfter(signature_end),
                    writer,
                )?;
                if comments
                    .last()
                    .is_some_and(|comment| comment.kind == CommentKind::Line)
                {
                    writer.hard_line();
                } else {
                    constraint.write(writer, FlatSeparator::Space, Writer::space);
                }
            }
            FunctionBodyBoundary::Formatted(comments) => {
                self.comments.claim(comments)?;
                constraint.write(writer, FlatSeparator::Space, Writer::space);
                self.format_comment_run(
                    comments,
                    CommentRunLayout::FlatLeading(body.span(self.ast).start),
                    writer,
                )?;
            }
        }
        self.format_function_body(body, mode, first_statement, leading, writer)
    }
}
