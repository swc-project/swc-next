//! Actual runtime parameter boundaries and receiver/formal/rest ownership.

use swc_next_ecma_ast::{
    Ast, Decorator, FormalParameter, FormalParameterItemData, FormalParameterPatternData,
    FormalParameterRest, FormalParameters, FunctionBody, GetSpan, MethodDefinition, Span, SubRange,
    TsParameterProperty, TsThisParameter,
};

use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail};
use super::decorators::{DecoratorLayout, DecoratorPrefix};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use super::suppression::{ContentMode, DelimitedSuppression, SuppressionPosition};
use super::types::TypeGap;
use super::types::callable::{CallableParameter, SignatureParameterHug, SignatureParameterLayout};
use crate::{
    ArrowParentheses, FormatError, TrailingCommas,
    document::{Fragment, Writer},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowParameterParentheses {
    Optional,
    Required,
}

enum ParameterRole {
    Arrow(ArrowParameterParentheses),
    Function,
    Constructor(MethodDefinition),
    PrivateMethod(FunctionBody),
}

enum ParameterSource {
    Parenthesized,
    Constructor { owner: MethodDefinition },
    Bare(FormalParameter),
    PrivateEmpty,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RuntimeParameter {
    Callable(CallableParameter),
    Property(TsParameterProperty),
}

impl RuntimeParameter {
    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Callable(parameter) => parameter.span(ast),
            Self::Property(property) => property.span(ast),
        }
    }

    fn from_formal(
        item: swc_next_ecma_ast::FormalParameterItem,
        ast: &Ast<'_>,
        constructor: Option<MethodDefinition>,
    ) -> Result<Self, FormatError> {
        match ast.formal_parameter_item_data(item) {
            FormalParameterItemData::FormalParameter(parameter) => {
                Ok(Self::Callable(CallableParameter::Formal(parameter)))
            }
            FormalParameterItemData::TsParameterProperty(property)
                if constructor.is_some() && ast.is_ts() =>
            {
                Ok(Self::Property(property))
            }
            FormalParameterItemData::TsParameterProperty(_) => Err(FormatError::unsupported(
                "arrow parameter property",
                Some(ast.node_kind(item.node_id())),
                item.span(ast),
            )),
        }
    }
}

#[derive(Clone, Copy)]
struct RuntimeParameterHead {
    node: RuntimeParameter,
    decorators: SubRange<Decorator>,
}

impl RuntimeParameterHead {
    fn new(ast: &Ast<'_>, node: RuntimeParameter) -> Self {
        let decorators = match node {
            RuntimeParameter::Callable(CallableParameter::Formal(node)) => node.decorators(ast),
            RuntimeParameter::Callable(CallableParameter::Rest(node)) => node.decorators(ast),
            RuntimeParameter::Property(node) => node.decorators(ast),
            RuntimeParameter::Callable(CallableParameter::This(_)) => SubRange::empty(),
        };
        Self { node, decorators }
    }
}

#[derive(Clone, Copy)]
struct RuntimeParameterEntry {
    node: RuntimeParameter,
    span: Span,
    decorators: Option<DecoratorPrefix>,
}

impl RuntimeParameterEntry {
    fn new(ast: &Ast<'_>, node: RuntimeParameter) -> Self {
        Self::from_head(ast, RuntimeParameterHead::new(ast, node))
    }

    fn from_head(ast: &Ast<'_>, head: RuntimeParameterHead) -> Self {
        let native = head.node.span(ast);
        let decorators = DecoratorPrefix::new(ast, head.decorators, native);
        Self {
            node: head.node,
            span: decorators.map_or(native, |prefix| prefix.envelope),
            decorators,
        }
    }
}

#[derive(Clone, Copy)]
enum ParameterModifierKind {
    Accessibility(&'static str),
    Override,
    Readonly,
}

impl ParameterModifierKind {
    fn keyword(self) -> &'static str {
        match self {
            Self::Accessibility(keyword) => keyword,
            Self::Override => "override",
            Self::Readonly => "readonly",
        }
    }

    fn canonical_index(self) -> usize {
        match self {
            Self::Accessibility(_) => 0,
            Self::Override => 1,
            Self::Readonly => 2,
        }
    }
}

struct ParameterModifier<'ast> {
    kind: ParameterModifierKind,
    after: super::comments::CommentBoundary<'ast>,
}

enum ParameterModifierGap<'ast> {
    Space,
    Layout(Fragment<'ast>),
}

struct ParameterPropertyPrefix<'ast> {
    leading: super::comments::CommentBoundary<'ast>,
    modifiers: [Option<ParameterModifier<'ast>>; 3],
}

struct PrivateParameterBoundary {
    parameter: CallableParameter,
    relation: DelimitedSuppression,
    closing: LineConstraint,
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
            RuntimeParameter::Callable(CallableParameter::Formal(node))
                if node.type_annotation(ast).is_none()
                    && node.initializer(ast).is_none()
                    && matches!(ast.formal_parameter_pattern_data(node.pattern(ast)),
                        FormalParameterPatternData::BindingIdentifier(_)));
        if !identifier {
            *self = Self::Other;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ParameterFrame {
    Ordinary,
    Fixed,
}

pub(super) struct SourceParameters {
    frame: ParameterFrame,
    node: FormalParameters,
    source: ParameterSource,
    first: Option<RuntimeParameterHead>,
    rest: Option<FormalParameterRest>,
    omit_parentheses: bool,
    hug: Option<SignatureParameterHug>,
    leading_mode: ContentMode,
    enclosing_mode: ContentMode,
    suppressed_tail: Option<RuntimeParameter>,
    private_boundary: Option<PrivateParameterBoundary>,
    key_mode: ContentMode,
    body_target: Option<(FunctionBody, ContentMode)>,
}

pub(super) struct FunctionParameters {
    pub(super) source: SourceParameters,
    pub(super) has_single_parameter: bool,
    pub(super) force_break: bool,
}

impl SourceParameters {
    pub(super) fn frame(&self) -> ParameterFrame {
        self.frame
    }

    pub(super) fn set_frame(&mut self, frame: ParameterFrame) {
        self.frame = frame;
    }

    pub(super) fn span(&self, ast: &Ast<'_>) -> Span {
        self.node.span(ast)
    }

    pub(super) fn key_mode(&self) -> ContentMode {
        self.key_mode
    }
    pub(super) fn body_mode(&self, body: FunctionBody) -> ContentMode {
        self.body_target
            .filter(|(target, _)| *target == body)
            .map_or(ContentMode::Formatted, |(_, mode)| mode)
    }

    pub(super) fn enclosing_mode(&self) -> ContentMode {
        self.enclosing_mode
    }

    pub(super) fn has_parameters(&self) -> bool {
        self.first.is_some() || self.rest.is_some()
    }

    pub(super) fn suppress_last(&mut self, ast: &Ast<'_>) -> Result<(), FormatError> {
        self.hug = None;
        let constructor = match self.source {
            ParameterSource::Constructor { owner, .. } => Some(owner),
            _ => None,
        };
        self.suppressed_tail = if let Some(rest) = self.rest {
            Some(RuntimeParameter::Callable(CallableParameter::Rest(rest)))
        } else if let Some(id) = self.node.items(ast).iter().next_back() {
            let item = ast.get_node_in_sub_range(id);
            Some(
                RuntimeParameter::from_formal(item, ast, constructor).map_err(|_| {
                    FormatError::unsupported("typed parameter property", None, item.span(ast))
                })?,
            )
        } else {
            self.first.map(|entry| entry.node).filter(|parameter| {
                matches!(
                    parameter,
                    RuntimeParameter::Callable(CallableParameter::This(_))
                )
            })
        };
        Ok(())
    }

    pub(super) fn set_leading_mode(&mut self, mode: ContentMode) {
        self.leading_mode = mode;
        if mode == ContentMode::Verbatim {
            self.hug = None;
        }
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
        self.source_parameters(node, None, ParameterRole::Arrow(parentheses))
            .map(|parameters| parameters.source)
    }

    pub(super) fn source_function_parameters(
        &self,
        node: FormalParameters,
        receiver: Option<TsThisParameter>,
    ) -> Result<FunctionParameters, FormatError> {
        self.source_parameters(node, receiver, ParameterRole::Function)
    }

    pub(super) fn source_constructor_parameters(
        &self,
        node: FormalParameters,
        owner: MethodDefinition,
    ) -> Result<FunctionParameters, FormatError> {
        self.source_parameters(node, None, ParameterRole::Constructor(owner))
    }

    pub(super) fn source_private_method_parameters(
        &self,
        node: FormalParameters,
        body: FunctionBody,
    ) -> Result<FunctionParameters, FormatError> {
        self.source_parameters(node, None, ParameterRole::PrivateMethod(body))
    }

    fn source_parameters(
        &self,
        node: FormalParameters,
        receiver: Option<TsThisParameter>,
        role: ParameterRole,
    ) -> Result<FunctionParameters, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let items = node.items(ast);
        let rest = node.rest(ast);
        let count = items.len() + usize::from(receiver.is_some()) + usize::from(rest.is_some());
        let has_single_parameter = count == 1;
        let constructor = match role {
            ParameterRole::Constructor(owner) => Some(owner),
            _ => None,
        };
        let force_break = constructor.is_some()
            && count > 1
            && items.iter().any(|id| {
                matches!(
                    ast.formal_parameter_item_data(ast.get_node_in_sub_range(id)),
                    FormalParameterItemData::TsParameterProperty(_)
                )
            });
        let first = if let Some(receiver) = receiver {
            Some(RuntimeParameter::Callable(CallableParameter::This(
                receiver,
            )))
        } else {
            items
                .iter()
                .next()
                .map(|id| {
                    RuntimeParameter::from_formal(ast.get_node_in_sub_range(id), ast, constructor)
                })
                .transpose()
                .map_err(|_| self.unsupported("arrow parameter property", node))?
        };
        let first_head = first.map(|node| RuntimeParameterHead::new(ast, node));
        let mut source = if self.source(span).starts_with('(') && self.source(span).ends_with(')') {
            match constructor {
                Some(owner) => ParameterSource::Constructor { owner },
                None => ParameterSource::Parenthesized,
            }
        } else if matches!(role, ParameterRole::Arrow(_))
            && let Some(RuntimeParameter::Callable(CallableParameter::Formal(parameter))) = first
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
            && !matches!(role, ParameterRole::PrivateMethod(_))
            && first.is_none()
            && rest.is_none()
        {
            self.content_mode(self.comments.in_range(span))
        } else {
            ContentMode::Formatted
        };
        let single = match first {
            Some(RuntimeParameter::Callable(CallableParameter::Formal(parameter)))
                if items.len() == 1 && rest.is_none() =>
            {
                Some(parameter)
            }
            _ => None,
        };
        let omit_parentheses = matches!(
            role,
            ParameterRole::Arrow(ArrowParameterParentheses::Optional)
        ) && self.options.arrow_parentheses == ArrowParentheses::AsNeeded
            && single.is_some_and(|parameter| {
                matches!(
                    ast.formal_parameter_pattern_data(parameter.pattern(ast)),
                    FormalParameterPatternData::BindingIdentifier(_)
                ) && parameter.initializer(ast).is_none()
                    && parameter.type_annotation(ast).is_none()
                    && !parameter.optional(ast)
                    && first_head.is_none_or(|head| head.decorators.is_empty())
            })
            && self.comments.in_range(span).is_empty()
            && !self.diagnostic_range_is_sensitive(span);
        let hug = if has_single_parameter {
            match first {
                Some(RuntimeParameter::Callable(parameter)) => {
                    self.callable_parameter_hug(parameter, span)
                }
                Some(RuntimeParameter::Property(_)) | None => None,
            }
        } else {
            None
        };
        let mut key_mode = ContentMode::Formatted;
        let mut body_mode = ContentMode::Formatted;
        let mut private_boundary = None;
        if self.has_suppression
            && let ParameterRole::PrivateMethod(_) = role
        {
            let content = Span::new(span.start + 1, span.end - 1);
            let tail = if let Some(rest) = rest {
                Some(CallableParameter::Rest(rest))
            } else {
                items.iter().next_back().and_then(|id| {
                    match ast.formal_parameter_item_data(ast.get_node_in_sub_range(id)) {
                        FormalParameterItemData::FormalParameter(parameter) => {
                            Some(CallableParameter::Formal(parameter))
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
                                preceding = Some(
                                    RuntimeParameterEntry::new(
                                        ast,
                                        RuntimeParameter::Callable(tail),
                                    )
                                    .span,
                                );
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
        Ok(FunctionParameters {
            has_single_parameter,
            force_break,
            source: SourceParameters {
                frame: ParameterFrame::Ordinary,
                node,
                source,
                first: first_head,
                rest,
                omit_parentheses,
                hug,
                leading_mode: ContentMode::Formatted,
                enclosing_mode,
                suppressed_tail: None,
                private_boundary,
                key_mode,
                body_target: match role {
                    ParameterRole::PrivateMethod(body) if body_mode == ContentMode::Verbatim => {
                        Some((body, body_mode))
                    }
                    _ => None,
                },
            },
        })
    }

    pub(super) fn format_source_parameters_with_layout(
        &mut self,
        parameters: SourceParameters,
        mut callback_shape: Option<&mut CallbackParameterShape>,
        layout: SignatureParameterLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let SourceParameters {
            frame,
            node,
            source,
            first,
            rest,
            omit_parentheses,
            hug,
            leading_mode,
            enclosing_mode: _,
            suppressed_tail,
            private_boundary,
            key_mode: _,
            body_target: _,
        } = parameters;
        let hugged_parameter = hug;
        let hug = hug.is_some();
        if let ParameterSource::PrivateEmpty = source {
            return self.format_verbatim(node.span(ast), writer);
        }
        let first = first.map(|head| RuntimeParameterEntry::from_head(ast, head));
        let has_parameters = first.is_some() || rest.is_some();
        let constructor = match source {
            ParameterSource::Constructor { owner, .. } => Some(owner),
            _ => None,
        };
        let content = match source {
            ParameterSource::Bare(parameter) => parameter.span(ast),
            ParameterSource::Parenthesized
            | ParameterSource::Constructor { .. }
            | ParameterSource::PrivateEmpty => {
                let span = node.span(ast);
                Span::new(span.start + 1, span.end - 1)
            }
        };
        if omit_parentheses
            && let Some(entry) = first
            && matches!(
                entry.node,
                RuntimeParameter::Callable(CallableParameter::Formal(_))
            )
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
                        |writer| {
                            if frame == ParameterFrame::Ordinary {
                                writer.soft_line();
                            }
                        },
                    );
                }
                let mut formal_ids = node.items(ast).iter();
                if matches!(
                    first.map(|entry| entry.node),
                    Some(
                        RuntimeParameter::Callable(CallableParameter::Formal(_))
                            | RuntimeParameter::Property(_)
                    )
                ) {
                    // The prepared head already carries this actual formal.
                    formal_ids.next();
                }
                let formals = formal_ids.map(|id| {
                    RuntimeParameter::from_formal(ast.get_node_in_sub_range(id), ast, constructor)
                        .map(|node| RuntimeParameterEntry::new(ast, node))
                });
                let mut parameters = first.into_iter().map(Ok).chain(formals).chain(
                    rest.map(|rest| {
                        RuntimeParameterEntry::new(
                            ast,
                            RuntimeParameter::Callable(CallableParameter::Rest(rest)),
                        )
                    })
                    .into_iter()
                    .map(Ok),
                );
                let mut next = parameters.next().transpose()?;
                let mut end = content.start;
                let mut separator = Separator::Line;
                let mut has_previous = false;
                let mut following_mode =
                    (leading_mode == ContentMode::Verbatim).then_some(leading_mode);
                while let Some(parameter) = next {
                    if let Some(shape) = callback_shape.as_deref_mut() {
                        shape.include(parameter.node, ast);
                    }
                    next = parameters.next().transpose()?;
                    let span = parameter.span;
                    if has_previous {
                        if frame == ParameterFrame::Ordinary
                            && matches!(separator, Separator::EmptyLine)
                        {
                            writer.break_parent();
                        }
                        let next = self
                            .comments
                            .between(Span::new(end, span.start))?
                            .first()
                            .map_or(span.start, |comment| comment.span.start);
                        let constraint = self.line_constraint(end, next);
                        match (frame, separator) {
                            (ParameterFrame::Fixed, Separator::Line | Separator::EmptyLine) => {
                                constraint.write(writer, FlatSeparator::Space, Writer::space);
                            }
                            _ => ArgumentLayout::Lines.separator(separator, constraint, writer),
                        }
                    }
                    let following = next
                        .map(|parameter| parameter.span)
                        .map_or(FollowingItem::End, FollowingItem::Node);
                    let following = private_boundary
                        .as_ref()
                        .filter(|source| {
                            RuntimeParameter::Callable(source.parameter) == parameter.node
                        })
                        .map_or(following, |source| {
                            FollowingItem::ResolvedTerminal(source.relation)
                        });
                    let mut item = self.prepare_delimited_item(
                        span,
                        following,
                        Span::new(end, content.end),
                        !matches!(
                            parameter.node,
                            RuntimeParameter::Callable(CallableParameter::Rest(_))
                        ) && !hug
                            && frame == ParameterFrame::Ordinary
                            && self.options.trailing_commas == TrailingCommas::All,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    if suppressed_tail == Some(parameter.node) {
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
            let end = if hug || frame == ParameterFrame::Fixed {
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
                    .write(writer, FlatSeparator::Empty, |writer| {
                        if frame == ParameterFrame::Ordinary {
                            writer.soft_line();
                        }
                    });
            }
            writer.text(")");
            Ok(())
        };
        match layout {
            SignatureParameterLayout::Shared => write(writer),
            SignatureParameterLayout::Independent => writer.group(false, write),
            SignatureParameterLayout::Forced => writer.group(true, write),
        }
    }

    fn format_runtime_parameter(
        &mut self,
        entry: RuntimeParameterEntry,
        mode: ContentMode,
        signature_hug: Option<SignatureParameterHug>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(entry.span, writer)?;
            return Ok(None);
        }
        let ast = self.ast;
        let native = entry.node.span(ast);
        let displaced = entry.decorators.is_none() && self.source_starts_decorator(native);
        if entry.decorators.is_none() && !displaced {
            return match entry.node {
                RuntimeParameter::Callable(CallableParameter::Formal(node)) => {
                    self.format_formal_parameter(node, mode, signature_hug, writer)
                }
                RuntimeParameter::Callable(CallableParameter::Rest(node)) => {
                    self.format_rest_parameter(node, mode, writer)
                }
                RuntimeParameter::Property(node) => {
                    self.format_constructor_parameter_property(node, mode, writer)
                }
                RuntimeParameter::Callable(CallableParameter::This(node)) => {
                    let prepared = self.prepare_runtime_this_parameter(node, mode)?;
                    let prepared = if let Some(hug) = signature_hug {
                        prepared.share_signature_group(hug)
                    } else {
                        prepared
                    };
                    self.format_prepared_parameter_item(prepared, writer)
                }
            };
        }
        let (prepared, limit) = match entry.node {
            RuntimeParameter::Callable(CallableParameter::Formal(node)) => (
                self.prepare_typed_formal_parameter(
                    node,
                    mode,
                    node.type_annotation(ast),
                    node.optional(ast),
                )?,
                node.pattern(ast).span(ast).start,
            ),
            RuntimeParameter::Callable(CallableParameter::Rest(node)) => {
                (self.prepare_decorated_rest_parameter(node)?, native.start)
            }
            RuntimeParameter::Property(node) => (
                self.prepare_constructor_parameter_item(node)?,
                node.parameter(ast).span(ast).start,
            ),
            RuntimeParameter::Callable(CallableParameter::This(_)) => {
                return Err(FormatError::unsupported(
                    "receiver decorator boundary",
                    None,
                    native,
                ));
            }
        };
        let prepared = if let Some(hug) = signature_hug {
            prepared.share_signature_group(hug)
        } else {
            prepared
        };
        // Whole-item Source wins before optional decorator/core geometry.
        if prepared.is_source() {
            self.format_verbatim(entry.span, writer)?;
            return Ok(None);
        }
        let plain_property = matches!(entry.node, RuntimeParameter::Property(node)
            if node.accessibility(ast) == swc_next_ecma_ast::Accessibility::None && !node.r#override(ast) && !node.readonly(ast));
        let core_start = if plain_property {
            limit
        } else if let Some(prefix) = entry.decorators {
            self.decorator_core_start(prefix, native, limit)?
        } else {
            match entry.node {
                RuntimeParameter::Property(_) => self.removed_member_core_start(native, limit),
                _ => limit,
            }
        };
        let removed = if displaced {
            self.removed_decorator_head(native, core_start)?
        } else {
            None
        };
        let property_prefix = match entry.node {
            RuntimeParameter::Property(node) => {
                Some(self.prepare_parameter_property_prefix(node, core_start)?)
            }
            _ => None,
        };
        writer.group(false, |writer| {
            if let Some(head) = removed {
                self.format_removed_decorator_head(head, writer)?;
            }
            if let Some(prefix) = entry.decorators {
                self.format_decorators(prefix, DecoratorLayout::Parameter, core_start, writer)?;
            }
            if let Some(prefix) = property_prefix {
                self.format_parameter_property_prefix(prefix, writer)?;
            }
            self.format_prepared_parameter_item(prepared, writer)
        })
    }

    fn format_constructor_parameter_property(
        &mut self,
        property: TsParameterProperty,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(property.span(self.ast), writer)?;
            return Ok(None);
        }
        let prepared = self.prepare_constructor_parameter_item(property)?;
        if prepared.is_source() {
            return self.format_prepared_parameter_item(prepared, writer);
        }
        let prefix =
            self.prepare_parameter_property_prefix(property, property.span(self.ast).start)?;
        writer.group(false, |writer| {
            self.format_parameter_property_prefix(prefix, writer)?;
            self.format_prepared_parameter_item(prepared, writer)
        })
    }

    fn format_parameter_property_prefix(
        &mut self,
        prefix: ParameterPropertyPrefix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_type_gap(
            TypeGap::Formatted(prefix.leading),
            FlatSeparator::Empty,
            writer,
        )?;
        let mut modifiers = std::array::from_fn::<_, 3, _>(|_| None);
        // Claim each source boundary once before replaying the AST's modifier order.
        for modifier in prefix.modifiers.into_iter().flatten() {
            let after = if modifier.after.is_empty() && !modifier.after.gap().requires_line() {
                ParameterModifierGap::Space
            } else {
                let (fragment, ()) = writer.record_with_value(|writer| {
                    self.format_type_gap(
                        TypeGap::Formatted(modifier.after),
                        FlatSeparator::Space,
                        writer,
                    )
                })?;
                ParameterModifierGap::Layout(fragment)
            };
            modifiers[modifier.kind.canonical_index()] = Some((modifier.kind, after));
        }
        for (kind, after) in modifiers.into_iter().flatten() {
            writer.text(kind.keyword());
            match after {
                ParameterModifierGap::Space => writer.space(),
                ParameterModifierGap::Layout(fragment) => writer.write_fragment(fragment),
            }
        }
        Ok(())
    }

    fn prepare_parameter_property_prefix(
        &self,
        property: TsParameterProperty,
        core_start: u32,
    ) -> Result<ParameterPropertyPrefix<'ast>, FormatError> {
        let ast = self.ast;
        let span = Span::new(core_start, property.parameter(ast).span(ast).start);
        let mut comments = self.comments.in_range(span);
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            return Err(self.unsupported("parameter property modifier suppression", property));
        }
        let accessibility = match property.accessibility(ast) {
            swc_next_ecma_ast::Accessibility::None => None,
            swc_next_ecma_ast::Accessibility::Public => Some("public"),
            swc_next_ecma_ast::Accessibility::Protected => Some("protected"),
            swc_next_ecma_ast::Accessibility::Private => Some("private"),
        };
        let kinds = [
            accessibility.map(ParameterModifierKind::Accessibility),
            property
                .r#override(ast)
                .then_some(ParameterModifierKind::Override),
            property
                .readonly(ast)
                .then_some(ParameterModifierKind::Readonly),
        ];
        let mut tokens = [None; 3];
        for (slot, kind) in tokens.iter_mut().zip(kinds) {
            let Some(kind) = kind else {
                continue;
            };
            let keyword = kind.keyword();
            let position = self.find_token(span, comments, keyword).ok_or_else(|| {
                self.unsupported("constructor parameter modifier boundary", property)
            })?;
            *slot = Some((position, kind));
        }
        tokens.sort_unstable_by_key(|token| token.map(|(position, _)| position));
        let mut tokens = tokens.into_iter().flatten().peekable();
        let leading_end = tokens.peek().map_or(span.end, |(position, _)| *position);
        let split = comments.partition_point(|comment| comment.span.end <= leading_end);
        let (leading, following) = comments.split_at(split);
        let leading = self.prepare_comment_boundary(leading, span.start, leading_end);
        comments = following;
        let mut modifiers = std::array::from_fn(|_| None);
        for modifier in &mut modifiers {
            let Some((position, kind)) = tokens.next() else {
                break;
            };
            let end = position + kind.keyword().len() as u32;
            let next = tokens.peek().map_or(span.end, |(position, _)| *position);
            let split = comments.partition_point(|comment| comment.span.end <= next);
            let (after, following) = comments.split_at(split);
            *modifier = Some(ParameterModifier {
                kind,
                after: self.prepare_comment_boundary(after, end, next),
            });
            comments = following;
        }
        Ok(ParameterPropertyPrefix { leading, modifiers })
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
