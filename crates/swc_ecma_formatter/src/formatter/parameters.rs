//! Runtime function and arrow parameters with source-owned bindings and delimiters.
use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use super::suppression::{ContentMode, DelimitedSuppression, SuppressionPosition};
use super::types::callable::{CallableParameter, SignatureParameterHug};
use crate::{ArrowParentheses, FormatError, TrailingCommas, document::Writer};
use swc_next_ecma_ast::{
    Ast, FormalParameter, FormalParameterItem, FormalParameterItemData, FormalParameterPatternData,
    FormalParameterRest, FormalParameters, FunctionBody, GetSpan, Span,
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowParameterParentheses {
    Optional,
    Required,
}
/// Shape requested only while recording a sole last Function callback.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CallbackParameterShape {
    Identifiers,
    Other,
}

impl CallbackParameterShape {
    fn include(&mut self, parameter: RuntimeParameter, ast: &Ast<'_>) {
        if *self == Self::Other {
            return;
        }
        let identifier = matches!(parameter,
            RuntimeParameter::Formal(node)
                if node.type_annotation(ast).is_none()
                    && node.initializer(ast).is_none()
                    && matches!(ast.formal_parameter_pattern_data(node.pattern(ast)),
                        FormalParameterPatternData::BindingIdentifier(_)));
        if !identifier {
            *self = Self::Other;
        }
    }
}

enum ParameterRole {
    Arrow(ArrowParameterParentheses),
    Function,
    PrivateMethod(FunctionBody),
}
enum ParameterSource {
    Parenthesized,
    Bare(FormalParameter),
    PrivateEmpty,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum RuntimeParameter {
    Formal(FormalParameter),
    Rest(FormalParameterRest),
}
impl RuntimeParameter {
    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Formal(node) => node.span(ast),
            Self::Rest(node) => node.span(ast),
        }
    }
    fn from_formal(item: FormalParameterItem, ast: &Ast<'_>) -> Result<Self, FormatError> {
        match ast.formal_parameter_item_data(item) {
            FormalParameterItemData::FormalParameter(node) => Ok(Self::Formal(node)),
            _ => Err(FormatError::unsupported(
                "parameter property",
                None,
                item.span(ast),
            )),
        }
    }
}
struct PrivateParameterBoundary {
    parameter: RuntimeParameter,
    relation: DelimitedSuppression,
    closing: LineConstraint,
}

pub(super) struct SourceParameters {
    node: FormalParameters,
    source: ParameterSource,
    first: Option<RuntimeParameter>,
    rest: Option<FormalParameterRest>,
    omit_parentheses: bool,
    hug: Option<SignatureParameterHug>,
    enclosing_mode: ContentMode,
    leading_mode: ContentMode,
    suppressed_tail: Option<RuntimeParameter>,
    private_boundary: Option<PrivateParameterBoundary>,
    key_mode: ContentMode,
    body_target: Option<(FunctionBody, ContentMode)>,
}
impl SourceParameters {
    pub(super) fn key_mode(&self) -> ContentMode {
        self.key_mode
    }
    pub(super) fn body_mode(&self, body: FunctionBody) -> ContentMode {
        self.body_target
            .filter(|(target, _)| *target == body)
            .map_or(ContentMode::Formatted, |(_, mode)| mode)
    }

    pub(super) fn span(&self, ast: &Ast<'_>) -> Span {
        self.node.span(ast)
    }

    pub(super) fn set_leading_mode(&mut self, mode: ContentMode) {
        self.leading_mode = mode;
        if mode == ContentMode::Verbatim {
            self.hug = None;
        }
    }
    pub(super) fn suppress_last(&mut self, ast: &Ast<'_>) -> Result<(), FormatError> {
        self.hug = None;
        self.suppressed_tail = if let Some(rest) = self.rest {
            Some(RuntimeParameter::Rest(rest))
        } else {
            self.node
                .items(ast)
                .last()
                .map(|id| RuntimeParameter::from_formal(ast.get_node_in_sub_range(id), ast))
                .transpose()?
        };
        Ok(())
    }

    pub(super) fn enclosing_mode(&self) -> ContentMode {
        self.enclosing_mode
    }
    pub(super) fn has_parameters(&self) -> bool {
        self.first.is_some() || self.rest.is_some()
    }
    pub(super) fn starts_with_parenthesis(&self) -> bool {
        !self.omit_parentheses
    }
}
impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn source_arrow_parameters(
        &self,
        node: FormalParameters,
        parentheses: ArrowParameterParentheses,
    ) -> Result<SourceParameters, FormatError> {
        self.source_parameters(node, ParameterRole::Arrow(parentheses))
    }
    pub(super) fn source_function_parameters(
        &self,
        node: FormalParameters,
    ) -> Result<SourceParameters, FormatError> {
        self.source_parameters(node, ParameterRole::Function)
    }
    pub(super) fn source_private_method_parameters(
        &self,
        node: FormalParameters,
        body: FunctionBody,
    ) -> Result<SourceParameters, FormatError> {
        self.source_parameters(node, ParameterRole::PrivateMethod(body))
    }
    fn source_parameters(
        &self,
        node: FormalParameters,
        role: ParameterRole,
    ) -> Result<SourceParameters, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let items = node.items(ast);
        let rest = node.rest(ast);
        let count = items.len() + usize::from(rest.is_some());
        let first = items
            .first()
            .map(|id| RuntimeParameter::from_formal(ast.get_node_in_sub_range(id), ast))
            .transpose()?;
        let mut source = if self.source(span).starts_with('(') && self.source(span).ends_with(')') {
            ParameterSource::Parenthesized
        } else if matches!(role, ParameterRole::Arrow(_))
            && let Some(RuntimeParameter::Formal(parameter)) = first
            && items.len() == 1
            && rest.is_none()
            && parameter.span(ast) == span
            && matches!(
                ast.formal_parameter_pattern_data(parameter.pattern(ast)),
                FormalParameterPatternData::BindingIdentifier(_)
            )
            && parameter.initializer(ast).is_none()
        {
            ParameterSource::Bare(parameter)
        } else {
            return Err(self.unsupported("arrow parameter source boundary", node));
        };
        let enclosing_mode = if self.has_suppression
            && count == 0
            && !matches!(role, ParameterRole::PrivateMethod(_))
        {
            self.content_mode(self.comments.in_range(span))
        } else {
            ContentMode::Formatted
        };
        let single = match first {
            Some(RuntimeParameter::Formal(p)) if count == 1 => Some(p),
            _ => None,
        };
        let omit_parentheses = matches!(
            role,
            ParameterRole::Arrow(ArrowParameterParentheses::Optional)
        ) && self.options.arrow_parentheses == ArrowParentheses::AsNeeded
            && single.is_some_and(|p| {
                matches!(
                    ast.formal_parameter_pattern_data(p.pattern(ast)),
                    FormalParameterPatternData::BindingIdentifier(_)
                ) && p.initializer(ast).is_none()
                    && p.type_annotation(ast).is_none()
                    && !p.optional(ast)
                    && p.decorators(ast).is_empty()
            })
            && self.comments.in_range(span).is_empty()
            && !self.diagnostic_range_is_sensitive(span);
        let hug = single.and_then(|parameter| {
            self.callable_parameter_hug(CallableParameter::Formal(parameter), span)
        });
        let mut key_mode = ContentMode::Formatted;
        let mut body_mode = ContentMode::Formatted;
        let mut private_boundary = None;
        if self.has_suppression
            && let ParameterRole::PrivateMethod(_) = role
        {
            let content = Span::new(span.start + 1, span.end - 1);
            let tail = if let Some(rest) = rest {
                Some(RuntimeParameter::Rest(rest))
            } else {
                items.iter().next_back().and_then(|id| {
                    match ast.formal_parameter_item_data(ast.get_node_in_sub_range(id)) {
                        FormalParameterItemData::FormalParameter(parameter) => {
                            Some(RuntimeParameter::Formal(parameter))
                        }
                        FormalParameterItemData::TsParameterProperty(_) => None,
                    }
                })
            };
            let end = tail.map_or(content.start, |tail| tail.span(ast).end);
            let gap = Span::new(end, content.end);
            let comments = self.comments.in_range(gap);
            if self.content_mode(comments) == ContentMode::Verbatim {
                let mut preceding = None;
                for position in self.suppression_positions(comments, content.end) {
                    match position {
                        SuppressionPosition::OwnLine => body_mode = ContentMode::Verbatim,
                        SuppressionPosition::Inline | SuppressionPosition::EndOfLine => {
                            if let Some(tail) = tail {
                                preceding = Some(tail.span(ast));
                            } else {
                                key_mode = ContentMode::Verbatim;
                            }
                        }
                    }
                }
                if let Some(parameter) = tail {
                    let last = comments.last().map_or(end, |comment| comment.span.end);
                    private_boundary = Some(PrivateParameterBoundary {
                        parameter,
                        relation: DelimitedSuppression {
                            preceding,
                            following: body_mode,
                        },
                        closing: self.source_line_constraint(last, content.end),
                    });
                } else {
                    source = ParameterSource::PrivateEmpty;
                }
            }
            // The function signature consumes this actual body target below.
        }
        Ok(SourceParameters {
            node,
            source,
            first,
            rest,
            omit_parentheses,
            hug,
            enclosing_mode,
            leading_mode: ContentMode::Formatted,
            suppressed_tail: None,
            private_boundary,
            key_mode,
            body_target: match role {
                ParameterRole::PrivateMethod(body) if body_mode == ContentMode::Verbatim => {
                    Some((body, body_mode))
                }
                _ => None,
            },
        })
    }
    pub(super) fn format_source_parameters(
        &mut self,
        parameters: SourceParameters,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_source_parameters_with_shape(parameters, None, writer)
    }
    pub(super) fn format_source_parameters_with_shape(
        &mut self,
        parameters: SourceParameters,
        mut callback_shape: Option<&mut CallbackParameterShape>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let SourceParameters {
            node,
            source,
            first,
            rest,
            omit_parentheses,
            hug,
            enclosing_mode: _,
            leading_mode,
            suppressed_tail,
            private_boundary,
            key_mode: _,
            body_target: _,
        } = parameters;
        if let ParameterSource::PrivateEmpty = source {
            return self.format_verbatim(node.span(ast), writer);
        }
        let hugged_parameter = hug;
        let hug = hug.is_some();
        let has_parameters = first.is_some() || rest.is_some();
        let content = match source {
            ParameterSource::Bare(parameter) => parameter.span(ast),
            ParameterSource::Parenthesized | ParameterSource::PrivateEmpty => {
                let span = node.span(ast);
                Span::new(span.start + 1, span.end - 1)
            }
        };
        if omit_parentheses
            && let Some(entry) = first
            && matches!(entry, RuntimeParameter::Formal(_))
        {
            let tail =
                self.format_runtime_parameter(entry, ContentMode::Formatted, None, writer)?;
            self.format_expression_tail(tail, writer)?;
            return Ok(());
        }
        let mut write = |writer: &mut Writer<'ast>| {
            writer.text("(");
            let mut write_items = |writer: &mut Writer<'ast>| {
                if !hug && (has_parameters || !self.comments.in_range(content).is_empty()) {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                }
                let mut formal_ids = node.items(ast).iter();
                if first.is_some() {
                    formal_ids.next();
                }
                let formals = formal_ids
                    .map(|id| RuntimeParameter::from_formal(ast.get_node_in_sub_range(id), ast));
                let mut parameters = first
                    .into_iter()
                    .map(Ok)
                    .chain(formals)
                    .chain(rest.map(RuntimeParameter::Rest).into_iter().map(Ok));
                let mut next = parameters.next().transpose()?;
                let mut end = content.start;
                let mut separator = Separator::Line;
                let mut has_previous = false;
                let mut following_mode =
                    (leading_mode == ContentMode::Verbatim).then_some(leading_mode);
                while let Some(parameter) = next {
                    if let Some(shape) = callback_shape.as_deref_mut() {
                        shape.include(parameter, ast);
                    }
                    next = parameters.next().transpose()?;
                    let span = parameter.span(ast);
                    if has_previous {
                        if matches!(separator, Separator::EmptyLine) {
                            writer.break_parent();
                        }
                        let next = self
                            .comments
                            .between(Span::new(end, span.start))?
                            .first()
                            .map_or(span.start, |comment| comment.span.start);
                        let constraint = self.line_constraint(end, next);
                        ArgumentLayout::Lines.separator(separator, constraint, writer);
                    }
                    let following = next
                        .map(|parameter| parameter.span(ast))
                        .map_or(FollowingItem::End, FollowingItem::Node);
                    let following = private_boundary
                        .as_ref()
                        .filter(|source| source.parameter == parameter)
                        .map_or(following, |source| {
                            FollowingItem::ResolvedTerminal(source.relation)
                        });
                    let mut item = self.prepare_delimited_item(
                        span,
                        following,
                        Span::new(end, content.end),
                        !matches!(parameter, RuntimeParameter::Rest(_))
                            && !hug
                            && self.options.trailing_commas == TrailingCommas::All,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    if suppressed_tail == Some(parameter) {
                        item.set_leading_mode(ContentMode::Verbatim);
                    }
                    let hug = if has_previous { None } else { hugged_parameter };
                    let tail = self.format_runtime_parameter(parameter, item.mode, hug, writer)?;
                    let boundary = self.finish_delimited_item(item, tail, writer)?;
                    end = boundary.end;
                    separator = boundary.separator;
                    following_mode = Some(boundary.following_mode);
                    has_previous = true;
                }
                let dangling = self.comments.between(Span::new(end, content.end))?;
                self.comments.claim(dangling)?;
                self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
                if dangling
                    .iter()
                    .any(|comment| comment.kind == swc_next_ecma_ast::CommentKind::Line)
                {
                    writer.break_parent();
                }
                Ok::<_, FormatError>(dangling.last().map_or(end, |comment| comment.span.end))
            };
            let end = if hug {
                write_items(writer)?
            } else {
                writer.indent(write_items)?
            };
            if !hug && (has_parameters || end != content.start) {
                private_boundary
                    .as_ref()
                    .map_or_else(
                        || self.line_constraint(end, content.end),
                        |source| source.closing,
                    )
                    .write(writer, FlatSeparator::Empty, Writer::soft_line);
            }
            writer.text(")");
            Ok(())
        };
        write(writer)
    }

    fn format_runtime_parameter(
        &mut self,
        node: RuntimeParameter,
        mode: ContentMode,
        hug: Option<SignatureParameterHug>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match node {
            RuntimeParameter::Formal(node) => self.format_formal_parameter(node, mode, hug, writer),
            RuntimeParameter::Rest(node) => self.format_rest_parameter(node, mode, writer),
        }
    }
    fn format_formal_parameter(
        &mut self,
        parameter: FormalParameter,
        mode: ContentMode,
        signature_hug: Option<SignatureParameterHug>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(parameter.span(self.ast), writer)?;
            return Ok(None);
        }
        if !parameter.decorators(self.ast).is_empty() {
            return Err(self.unsupported("decorated parameter", parameter));
        }
        if self.ast.is_ts() {
            let annotation = parameter.type_annotation(self.ast);
            let optional = parameter.optional(self.ast);
            if annotation.is_some() || optional {
                let prepared =
                    self.prepare_typed_formal_parameter(parameter, mode, annotation, optional)?;
                if let Some(hug) = signature_hug
                    && !prepared.is_source()
                {
                    return self.format_prepared_parameter_item(
                        prepared.share_signature_group(hug),
                        writer,
                    );
                }
                return writer.group(false, |writer| {
                    self.format_prepared_parameter_item(prepared, writer)
                });
            }
        }
        let pattern = parameter.pattern(self.ast);
        if matches!(signature_hug, Some(SignatureParameterHug::Object))
            && let FormalParameterPatternData::ObjectPattern(object) =
                self.ast.formal_parameter_pattern_data(pattern)
        {
            return self
                .format_object_binding_contents(object, writer)
                .map(|()| None);
        }
        let default = parameter
            .initializer(self.ast)
            .map(|initializer| {
                self.prepare_binding_default(
                    self.parameter_binding(pattern),
                    initializer,
                    parameter.span(self.ast),
                )
            })
            .transpose()?;
        let default = match default {
            Some(default) if default.is_verbatim() => {
                return self.format_binding_default(default, writer);
            }
            default => default,
        };
        writer.group(false, |writer| match default {
            Some(default) => self.format_binding_default(default, writer),
            None => self.format_parameter_binding(pattern, mode, writer),
        })
    }

    fn format_rest_parameter(
        &mut self,
        rest: FormalParameterRest,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(rest.span(self.ast), writer)?;
            return Ok(None);
        }
        if !rest.decorators(self.ast).is_empty() {
            return Err(self.unsupported("decorated rest parameter", rest));
        }
        if self.ast.is_ts()
            && let Some(annotation) = rest.type_annotation(self.ast)
        {
            let prepared = self.prepare_typed_rest_parameter(rest, mode, annotation)?;
            return self.format_prepared_parameter_item(prepared, writer);
        }
        let argument = rest.argument(self.ast);
        writer.text("...");
        let leading = self.comments.between(Span::new(
            rest.span(self.ast).start + 3,
            argument.span(self.ast).start,
        ))?;
        let mode = self.content_mode(leading);
        self.comments.claim(leading)?;
        self.format_comment_run(
            leading,
            CommentRunLayout::FlatLeading(argument.span(self.ast).start),
            writer,
        )?;
        self.format_parameter_binding(argument, mode, writer)
    }
}
