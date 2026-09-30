//! Runtime function and arrow parameters with source-owned bindings and delimiters.
use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail};
use super::diagnostics::FlatSeparator;
use super::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use super::suppression::ContentMode;
use crate::{ArrowParentheses, FormatError, TrailingCommas, document::Writer};
use swc_next_ecma_ast::{
    Ast, ExprData, FormalParameter, FormalParameterItem, FormalParameterItemData,
    FormalParameterPatternData, FormalParameterRest, FormalParameters, GetSpan, Span,
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowParameterParentheses {
    Optional,
    Required,
}
enum ParameterRole {
    Arrow(ArrowParameterParentheses),
    Function,
}
enum ParameterSource {
    Parenthesized,
    Bare(FormalParameter),
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
#[derive(Clone, Copy)]
enum ParameterHug {
    Pattern,
    Object,
}
pub(super) struct SourceParameters {
    node: FormalParameters,
    source: ParameterSource,
    first: Option<RuntimeParameter>,
    rest: Option<FormalParameterRest>,
    omit_parentheses: bool,
    hug: Option<ParameterHug>,
    enclosing_mode: ContentMode,
    leading_mode: ContentMode,
    suppressed_tail: Option<RuntimeParameter>,
}
impl SourceParameters {
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
        let source = if self.source(span).starts_with('(') && self.source(span).ends_with(')') {
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
        let enclosing_mode = if self.has_suppression && count == 0 {
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
        let hug = single.and_then(|parameter| self.parameter_hug(parameter, span));
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
        })
    }
    fn parameter_hug(&self, node: FormalParameter, envelope: Span) -> Option<ParameterHug> {
        let ast = self.ast;
        if !node.decorators(ast).is_empty() {
            return None;
        }
        let pattern = node.pattern(ast);
        let initializer = node.initializer(ast);
        if initializer.is_some_and(|expr| match self.expression_content(expr).data {
            ExprData::IdentifierReference(_) => false,
            ExprData::ArrayExpression(a) => !a.elements(ast).is_empty(),
            ExprData::ObjectExpression(o) => !o.properties(ast).is_empty(),
            _ => true,
        }) {
            return None;
        }
        let hug = match ast.formal_parameter_pattern_data(pattern) {
            FormalParameterPatternData::ObjectPattern(_) if initializer.is_none() => {
                ParameterHug::Object
            }
            FormalParameterPatternData::ObjectPattern(_)
            | FormalParameterPatternData::ArrayPattern(_) => ParameterHug::Pattern,
            _ => return None,
        };
        if self.diagnostic_range_is_sensitive(envelope)
            || !self
                .comments
                .in_range(Span::new(envelope.start, pattern.span(ast).start))
                .is_empty()
            || !self
                .comments
                .in_range(Span::new(node.span(ast).end, envelope.end))
                .is_empty()
        {
            return None;
        }
        Some(hug)
    }
    pub(super) fn format_source_parameters(
        &mut self,
        parameters: SourceParameters,
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
        } = parameters;
        let hugged_parameter = hug;
        let hug = hug.is_some();
        let has_parameters = first.is_some() || rest.is_some();
        let content = match source {
            ParameterSource::Bare(parameter) => parameter.span(ast),
            ParameterSource::Parenthesized => {
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
                self.line_constraint(end, content.end).write(
                    writer,
                    FlatSeparator::Empty,
                    Writer::soft_line,
                );
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
        hug: Option<ParameterHug>,
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
        signature_hug: Option<ParameterHug>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(parameter.span(self.ast), writer)?;
            return Ok(None);
        }
        if parameter.type_annotation(self.ast).is_some()
            || parameter.optional(self.ast)
            || !parameter.decorators(self.ast).is_empty()
        {
            return Err(self.unsupported("typed or decorated parameter", parameter));
        }
        let pattern = parameter.pattern(self.ast);
        if matches!(signature_hug, Some(ParameterHug::Object))
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
        if rest.type_annotation(self.ast).is_some() || !rest.decorators(self.ast).is_empty() {
            return Err(self.unsupported("typed or decorated rest parameter", rest));
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
