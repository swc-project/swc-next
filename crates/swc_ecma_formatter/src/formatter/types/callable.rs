//! Shared parameter items retain real bindings and annotation wrappers.

use swc_next_ecma_ast::{
    Ast, BindingPattern, Comment, Expr, FormalParameter, FormalParameterItemData,
    FormalParameterPatternData, FormalParameterRest, FormalParameters, GetSpan, Span,
    TsConstructorType, TsFunctionType, TsThisParameter, TsType, TsTypeAnnotation, TsTypeData,
    TsTypeParameterDeclaration, TsTypePredicate, TsTypePredicateNameData,
};

use super::generics::TypeParameterContext;
use super::{
    Formatter, PreparedAnnotation, PreparedType, PreparedTypeParentheses, TypeBoundaryLayout,
    TypeContext, TypeGap, TypeLayout,
};
use crate::formatter::bindings::{DefaultOwner, InitializerLayout};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::ExpressionContext;
use crate::formatter::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use crate::formatter::suppression::{ContentMode, DelimitedSuppression, SuppressionPosition};
use crate::{FormatError, TrailingCommas, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum CallableType {
    Function(TsFunctionType),
    Constructor(TsConstructorType),
}

struct CallableParts {
    parameters: FormalParameters,
    this_parameter: Option<TsThisParameter>,
    type_parameters: Option<TsTypeParameterDeclaration>,
    return_type: TsTypeAnnotation,
}

impl CallableType {
    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Function(node) => node.span(ast),
            Self::Constructor(node) => node.span(ast),
        }
    }

    fn parts(self, ast: &Ast<'_>) -> CallableParts {
        match self {
            Self::Function(node) => CallableParts {
                parameters: node.params(ast),
                this_parameter: node.this_param(ast),
                type_parameters: node.type_parameters(ast),
                return_type: node.return_type(ast),
            },
            Self::Constructor(node) => CallableParts {
                parameters: node.params(ast),
                this_parameter: None,
                type_parameters: node.type_parameters(ast),
                return_type: node.return_type(ast),
            },
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum SignatureIncoming {
    Parameter(CallableParameter),
    Return(TsTypeAnnotation),
    Empty,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CallableParameter {
    This(TsThisParameter),
    Formal(FormalParameter),
    Rest(FormalParameterRest),
}

impl CallableParameter {
    pub(super) fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::This(node) => node.span(ast),
            Self::Formal(node) => node.span(ast),
            Self::Rest(node) => node.span(ast),
        }
    }
}

pub(super) enum AnnotationContent {
    Wrapper(TsTypeAnnotation),
    Type(PreparedType),
}

struct PredicateBound<'ast> {
    content: AnnotationContent,
    prefix: TypeBoundaryLayout<'ast>,
}

pub(super) struct SignatureReturn<'ast> {
    pub(super) content: AnnotationContent,
    prefix: TypeBoundaryLayout<'ast>,
    pub(super) preceding_mode: ContentMode,
    delimiter: SignatureReturnDelimiter,
}

enum FunctionReturnContent<'ast> {
    Annotation(AnnotationContent),
    Parenthesized(PreparedTypeParentheses<'ast>),
}

pub(super) struct FunctionReturn<'ast> {
    content: FunctionReturnContent<'ast>,
    prefix: TypeBoundaryLayout<'ast>,
    delimiter: SignatureReturnDelimiter,
}

impl FunctionReturn<'_> {
    pub(super) fn has_object_type(&self) -> bool {
        matches!(&self.content,
            FunctionReturnContent::Annotation(AnnotationContent::Type(node))
                if matches!(node.data,
                    swc_next_ecma_ast::TsTypeData::TsTypeLiteral(_)
                        | swc_next_ecma_ast::TsTypeData::TsMappedType(_)))
    }
}

#[derive(Clone, Copy)]
pub(super) enum SignatureReturnDelimiter {
    Arrow,
    Colon,
}

#[derive(Clone, Copy)]
pub(super) enum SignatureParameterHug {
    Pattern,
    Object,
    Type,
}

pub(super) struct SignatureGenerics<'ast> {
    pub(super) node: TsTypeParameterDeclaration,
    pub(super) mode: ContentMode,
    pub(super) following: TypeGap<'ast>,
}

pub(super) enum SignatureReturnEolTarget<'mode> {
    Parameter,
    Header(&'mode mut ContentMode),
    Annotation,
}

impl SignatureReturnDelimiter {
    fn token(self) -> &'static str {
        match self {
            Self::Arrow => "=>",
            Self::Colon => ":",
        }
    }

    fn preceding_space(self) -> FlatSeparator {
        match self {
            Self::Arrow => FlatSeparator::Space,
            Self::Colon => FlatSeparator::Empty,
        }
    }
}

enum CallableHead<'ast> {
    Function,
    Constructor { abstract_gap: Option<TypeGap<'ast>> },
}

#[derive(Clone, Copy)]
enum CallableIncomingOwner {
    Parameter(CallableParameter),
    Return(TsTypeAnnotation),
}

struct CallableTerminal {
    parameter: CallableParameter,
    closing: LineConstraint,
}

#[derive(Clone, Copy)]
struct IncomingParameterHead {
    packet: Span,
    target_start: u32,
}

pub(super) struct CallableParameters {
    node: FormalParameters,
    this_parameter: Option<TsThisParameter>,
    count: usize,
    hug: Option<SignatureParameterHug>,
    pub(super) source_head: Option<CallableParameter>,
    suppressed_tail: Option<CallableParameter>,
    terminal: Option<CallableTerminal>,
    incoming_head: Option<IncomingParameterHead>,
}

impl CallableParameters {
    pub(super) fn has_parameters(&self) -> bool {
        self.count != 0
    }

    pub(super) fn count(&self) -> usize {
        self.count
    }
}

enum ParameterBinding<'ast> {
    Pattern {
        node: BindingPattern,
        mode: ContentMode,
        spread: Option<TypeGap<'ast>>,
    },
    This(TsThisParameter),
    SignatureObject(BindingPattern),
}

struct PreparedParameterDefault<'ast> {
    left: Span,
    right: Expr,
    left_mode: ContentMode,
    layout: InitializerLayout<'ast>,
    head: &'ast [Comment],
}

struct ParameterLeft<'ast> {
    binding: ParameterBinding<'ast>,
    optional: bool,
    annotation: Option<PreparedAnnotation<'ast>>,
    annotation_layout: TypeLayout,
    end: u32,
}

enum PreparedParameterLeft<'ast> {
    Source(Span),
    Formatted(ParameterLeft<'ast>),
}

impl PreparedParameterLeft<'_> {
    fn end(&self) -> u32 {
        match self {
            Self::Source(span) => span.end,
            Self::Formatted(left) => left.end,
        }
    }
}

enum ParameterItemState<'ast> {
    Source(CallableParameter),
    Formatted(ParameterLeft<'ast>),
    Default {
        left: PreparedParameterLeft<'ast>,
        right: Expr,
        layout: InitializerLayout<'ast>,
        head: &'ast [Comment],
    },
}

pub(super) struct PreparedParameterItem<'ast>(ParameterItemState<'ast>);

impl PreparedParameterItem<'_> {
    pub(super) fn share_signature_group(mut self, hug: SignatureParameterHug) -> Self {
        if let ParameterItemState::Formatted(left) = &mut self.0 {
            if matches!(hug, SignatureParameterHug::Object)
                && let ParameterBinding::Pattern {
                    node,
                    mode: ContentMode::Formatted,
                    spread: None,
                } = &left.binding
            {
                left.binding = ParameterBinding::SignatureObject(*node);
            }
            if let Some(annotation) = &left.annotation
                && matches!(&annotation.content, super::AnnotationContent::Formatted(_))
            {
                left.annotation_layout = TypeLayout::SignatureParameter;
            }
        }
        self
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn signature_incoming(
        &self,
        parameters: &CallableParameters,
        result: Option<TsTypeAnnotation>,
        selected: &mut Option<SignatureIncoming>,
    ) -> Result<SignatureIncoming, FormatError> {
        if let Some(owner) = *selected {
            return Ok(owner);
        }
        let owner = if let Some(first) = self.first_callable_parameter(parameters)? {
            SignatureIncoming::Parameter(first)
        } else if let Some(annotation) = result {
            SignatureIncoming::Return(annotation)
        } else {
            SignatureIncoming::Empty
        };
        *selected = Some(owner);
        Ok(owner)
    }

    pub(super) fn format_callable_type(
        &mut self,
        node: CallableType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let parts = node.parts(ast);
        let parameters_span = parts.parameters.span(ast);
        let mut parameters = self.callable_parameters(parts.parameters, parts.this_parameter);
        if self.has_suppression
            && !parameters.has_parameters()
            && self.content_mode(self.comments.in_range(parameters_span)) == ContentMode::Verbatim
        {
            self.format_verbatim(node.span(ast), writer)?;
            return Ok(None);
        }
        let head_end = parts
            .type_parameters
            .map_or(parameters_span.start, |node| node.span(ast).start);
        let mut generic_mode = ContentMode::Formatted;
        let mut incoming = None;
        let (head, prefix_end) = match node {
            CallableType::Function(_) => (CallableHead::Function, node.span(ast).start),
            CallableType::Constructor(constructor) => {
                let start = constructor.span(ast).start;
                let gap = Span::new(start, head_end);
                let comments = self.comments.in_range(gap);
                let Some(keyword) = self.find_token(gap, comments, "new") else {
                    return Err(self.unsupported("constructor type keyword", constructor));
                };
                let abstract_gap = if constructor.r#abstract(ast) {
                    let gap = Span::new(start + 8, keyword);
                    let source = self.has_suppression
                        && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim;
                    if source {
                        if parts.type_parameters.is_some() {
                            generic_mode = ContentMode::Verbatim;
                        } else {
                            self.select_callable_incoming(
                                &parameters,
                                parts.return_type,
                                &mut incoming,
                            )?;
                        }
                    }
                    Some(self.prepare_type_gap(gap, source))
                } else {
                    None
                };
                (CallableHead::Constructor { abstract_gap }, keyword + 3)
            }
        };
        let head_gap = Span::new(prefix_end, head_end);
        let head_source = self.has_suppression
            && self.content_mode(self.comments.in_range(head_gap)) == ContentMode::Verbatim;
        if head_source {
            if parts.type_parameters.is_some() {
                generic_mode = ContentMode::Verbatim;
            } else {
                self.select_callable_incoming(&parameters, parts.return_type, &mut incoming)?;
            }
        }
        let relocated = !head_source
            && head_end == parameters_span.start
            && matches!(head, CallableHead::Constructor { .. })
            && self.prepare_incoming_parameter_head(&mut parameters, head_gap)?;
        let prefix = self.prepare_type_gap(
            if relocated {
                Span::new(head_gap.end, head_gap.end)
            } else {
                head_gap
            },
            head_source,
        );
        let generic_tail = parts
            .type_parameters
            .map(|generic| {
                let gap = Span::new(generic.span(ast).end, parameters_span.start);
                let comments = self.comments.in_range(gap);
                let source =
                    self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
                if source {
                    for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                        if matches!(position, SuppressionPosition::EndOfLine) {
                            generic_mode = ContentMode::Verbatim;
                        } else {
                            self.select_callable_incoming(
                                &parameters,
                                parts.return_type,
                                &mut incoming,
                            )?;
                        }
                    }
                }
                Ok((generic, self.prepare_type_gap(gap, source)))
            })
            .transpose()?;
        let source_return = match incoming {
            Some(CallableIncomingOwner::Parameter(parameter)) => {
                parameters.source_head = Some(parameter);
                None
            }
            Some(CallableIncomingOwner::Return(annotation)) => Some(annotation),
            None => None,
        };
        let eol_target = if parameters.has_parameters() {
            SignatureReturnEolTarget::Parameter
        } else {
            SignatureReturnEolTarget::Annotation
        };
        let return_type = self.prepare_signature_return(
            parameters_span.end,
            eol_target,
            parts.return_type,
            source_return,
            SignatureReturnDelimiter::Arrow,
        );
        self.prepare_callable_parameter_tail(&mut parameters, return_type.preceding_mode)?;
        let can_group_parameters = self.signature_parameters_can_group(
            parameters.count() == 1,
            parts.type_parameters,
            Some(&return_type),
        );
        let generics = generic_tail.map(|(node, following)| SignatureGenerics {
            node,
            mode: generic_mode,
            following,
        });
        writer.group(false, |writer| {
            if let CallableHead::Constructor { abstract_gap } = head {
                if let Some(gap) = abstract_gap {
                    writer.text("abstract");
                    self.format_type_gap(gap, FlatSeparator::Space, writer)?;
                }
                writer.text("new");
                self.format_type_gap(prefix, FlatSeparator::Space, writer)?;
            } else {
                self.format_type_gap(prefix, FlatSeparator::Empty, writer)?;
            }
            self.format_callable_signature_contents(
                parameters,
                Some(return_type),
                generics,
                can_group_parameters,
                false,
                writer,
            )
        })
    }

    pub(super) fn callable_parameters(
        &self,
        node: FormalParameters,
        this_parameter: Option<TsThisParameter>,
    ) -> CallableParameters {
        let count = node.items(self.ast).len()
            + usize::from(this_parameter.is_some())
            + usize::from(node.rest(self.ast).is_some());
        let hug = if count == 1 {
            let parameter = if let Some(this) = this_parameter {
                Some(CallableParameter::This(this))
            } else {
                node.items(self.ast).iter().next().and_then(|id| {
                    match self
                        .ast
                        .formal_parameter_item_data(self.ast.get_node_in_sub_range(id))
                    {
                        FormalParameterItemData::FormalParameter(parameter) => {
                            Some(CallableParameter::Formal(parameter))
                        }
                        FormalParameterItemData::TsParameterProperty(_) => None,
                    }
                })
            };
            parameter
                .and_then(|parameter| self.callable_parameter_hug(parameter, node.span(self.ast)))
        } else {
            None
        };
        CallableParameters {
            node,
            this_parameter,
            count,
            hug,
            source_head: None,
            suppressed_tail: None,
            terminal: None,
            incoming_head: None,
        }
    }

    pub(super) fn callable_parameter_hug(
        &self,
        parameter: CallableParameter,
        envelope: Span,
    ) -> Option<SignatureParameterHug> {
        let ast = self.ast;
        let (hug, start) = match parameter {
            CallableParameter::Formal(node) => {
                if !node.decorators(ast).is_empty() {
                    return None;
                }
                let pattern = node.pattern(ast);
                let initializer = node.initializer(ast);
                if initializer.is_some_and(|expression| {
                    match self.expression_content(expression).data {
                        swc_next_ecma_ast::ExprData::IdentifierReference(_) => false,
                        swc_next_ecma_ast::ExprData::ArrayExpression(array) => {
                            !array.elements(ast).is_empty()
                        }
                        swc_next_ecma_ast::ExprData::ObjectExpression(object) => {
                            !object.properties(ast).is_empty()
                        }
                        _ => true,
                    }
                }) {
                    return None;
                }
                let hug = match ast.formal_parameter_pattern_data(pattern) {
                    FormalParameterPatternData::ObjectPattern(_) if initializer.is_none() => {
                        SignatureParameterHug::Object
                    }
                    FormalParameterPatternData::ObjectPattern(_)
                    | FormalParameterPatternData::ArrayPattern(_) => SignatureParameterHug::Pattern,
                    FormalParameterPatternData::BindingIdentifier(_) if initializer.is_none() => {
                        let annotation = node.type_annotation(ast)?;
                        let type_node = annotation.type_annotation(ast);
                        if !self.parameter_has_object_type(type_node)
                            || !self
                                .comments
                                .in_range(Span::new(
                                    pattern.span(ast).end,
                                    type_node.span(ast).start,
                                ))
                                .is_empty()
                        {
                            return None;
                        }
                        SignatureParameterHug::Type
                    }
                    _ => return None,
                };
                (hug, pattern.span(ast).start)
            }
            CallableParameter::This(node) => {
                let annotation = node.type_annotation(ast)?;
                let type_node = annotation.type_annotation(ast);
                if !self.parameter_has_object_type(type_node)
                    || !self
                        .comments
                        .in_range(Span::new(
                            node.span(ast).start + 4,
                            type_node.span(ast).start,
                        ))
                        .is_empty()
                {
                    return None;
                }
                (SignatureParameterHug::Type, node.span(ast).start)
            }
            CallableParameter::Rest(_) => return None,
        };
        if self.diagnostic_range_is_sensitive(envelope) {
            return None;
        }
        if !self
            .comments
            .in_range(Span::new(envelope.start, start))
            .is_empty()
            || !self
                .comments
                .in_range(Span::new(parameter.span(ast).end, envelope.end))
                .is_empty()
        {
            return None;
        }
        Some(hug)
    }

    fn parameter_has_object_type(&self, mut node: TsType) -> bool {
        loop {
            match self.ast.ts_type_data(node) {
                TsTypeData::TsTypeLiteral(_) | TsTypeData::TsMappedType(_) => return true,
                TsTypeData::TsParenthesizedType(parentheses) => {
                    node = parentheses.type_annotation(self.ast)
                }
                _ => return false,
            }
        }
    }

    pub(super) fn first_callable_parameter(
        &self,
        parameters: &CallableParameters,
    ) -> Result<Option<CallableParameter>, FormatError> {
        let ast = self.ast;
        let first = if let Some(this) = parameters.this_parameter {
            Some(CallableParameter::This(this))
        } else if let Some(id) = parameters.node.items(ast).iter().next() {
            let item = ast.get_node_in_sub_range(id);
            match ast.formal_parameter_item_data(item) {
                FormalParameterItemData::FormalParameter(parameter) => {
                    Some(CallableParameter::Formal(parameter))
                }
                FormalParameterItemData::TsParameterProperty(_) => {
                    return Err(self.unsupported("callable type parameter property", item));
                }
            }
        } else {
            parameters.node.rest(ast).map(CallableParameter::Rest)
        };
        Ok(first)
    }

    /// The real first parameter owns an ordinary packet immediately before its opener.
    /// An empty frame has no such target, and its existing inner packet stays ordered.
    pub(super) fn prepare_incoming_parameter_head(
        &self,
        parameters: &mut CallableParameters,
        packet: Span,
    ) -> Result<bool, FormatError> {
        let comments = self.comments.in_range(packet);
        if comments.is_empty()
            || !self
                .comment_binding(comments, packet, None)
                .allows_relocation(None)
        {
            return Ok(false);
        }
        let Some(first) = self.first_callable_parameter(parameters)? else {
            return Ok(false);
        };
        let target = first.span(self.ast);
        let leading = Span::new(parameters.node.span(self.ast).start + 1, target.start);
        if !self
            .comment_binding(self.comments.in_range(leading), leading, Some(target))
            .allows_relocation(Some(target))
        {
            return Ok(false);
        }
        parameters.incoming_head = Some(IncomingParameterHead {
            packet,
            target_start: target.start,
        });
        parameters.hug = None;
        Ok(true)
    }

    pub(super) fn prepare_callable_parameter_tail(
        &self,
        parameters: &mut CallableParameters,
        preceding_mode: ContentMode,
    ) -> Result<(), FormatError> {
        if !self.has_suppression {
            return Ok(());
        }
        let ast = self.ast;
        let parameters_span = parameters.node.span(ast);
        let content = Span::new(parameters_span.start + 1, parameters_span.end - 1);
        let candidate = self.content_mode(self.comments.in_range(content)) == ContentMode::Verbatim;
        if candidate || preceding_mode == ContentMode::Verbatim {
            let last = self.last_callable_parameter(parameters)?;
            if preceding_mode == ContentMode::Verbatim {
                parameters.suppressed_tail = last;
            }
            if let Some(last) = last {
                let end = last.span(ast).end;
                let comments = self.comments.in_range(Span::new(end, content.end));
                if self.content_mode(comments) == ContentMode::Verbatim {
                    parameters.suppressed_tail = Some(last);
                    let last_end = comments.last().map_or(end, |comment| comment.span.end);
                    parameters.terminal = Some(CallableTerminal {
                        parameter: last,
                        closing: self.source_line_constraint(last_end, content.end),
                    });
                }
            }
        }
        Ok(())
    }

    fn select_callable_incoming(
        &self,
        parameters: &CallableParameters,
        annotation: TsTypeAnnotation,
        owner: &mut Option<CallableIncomingOwner>,
    ) -> Result<(), FormatError> {
        if owner.is_some() {
            return Ok(());
        }
        let first = self.first_callable_parameter(parameters)?;
        *owner = Some(first.map_or(
            CallableIncomingOwner::Return(annotation),
            CallableIncomingOwner::Parameter,
        ));
        Ok(())
    }

    pub(super) fn prepare_signature_return(
        &self,
        preceding_end: u32,
        mut eol_target: SignatureReturnEolTarget<'_>,
        annotation: TsTypeAnnotation,
        source_return: Option<TsTypeAnnotation>,
        delimiter: SignatureReturnDelimiter,
    ) -> SignatureReturn<'ast> {
        let ast = self.ast;
        let delimiter_start = annotation.span(ast).start;
        let before = self
            .comments
            .in_range(Span::new(preceding_end, delimiter_start));
        let mut preceding_mode = ContentMode::Formatted;
        let mut source_before = false;
        let mut wrapper_mode =
            source_return.map_or(ContentMode::Formatted, |_| ContentMode::Verbatim);
        if self.has_suppression && self.content_mode(before) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(before, delimiter_start) {
                if matches!(position, SuppressionPosition::EndOfLine) {
                    match &mut eol_target {
                        SignatureReturnEolTarget::Parameter => {
                            preceding_mode = ContentMode::Verbatim;
                            source_before = true;
                        }
                        SignatureReturnEolTarget::Header(mode) => {
                            **mode = ContentMode::Verbatim;
                            source_before = true;
                        }
                        SignatureReturnEolTarget::Annotation => {
                            wrapper_mode = ContentMode::Verbatim;
                        }
                    }
                } else {
                    wrapper_mode = ContentMode::Verbatim;
                }
            }
        }
        if wrapper_mode == ContentMode::Verbatim {
            return SignatureReturn {
                content: AnnotationContent::Wrapper(source_return.unwrap_or(annotation)),
                prefix: TypeBoundaryLayout::Source(Span::new(preceding_end, delimiter_start)),
                preceding_mode,
                delimiter,
            };
        }
        let mut node = self.prepare_type(annotation.type_annotation(ast), ContentMode::Formatted);
        let type_start = node.node.span(ast).start;
        let after = self.comments.in_range(Span::new(
            delimiter_start + delimiter.token().len() as u32,
            type_start,
        ));
        self.select_type_head_comments(&mut node, after, type_start);
        let prefix = if source_before || node.source_owner.is_some() {
            TypeBoundaryLayout::Source(Span::new(preceding_end, type_start))
        } else {
            TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(before, preceding_end, delimiter_start),
                after: self.prepare_comment_boundary(
                    after,
                    delimiter_start + delimiter.token().len() as u32,
                    type_start,
                ),
            }
        };
        SignatureReturn {
            content: AnnotationContent::Type(node),
            prefix,
            preceding_mode,
            delimiter,
        }
    }

    pub(super) fn prepare_function_return(
        &self,
        result: SignatureReturn<'ast>,
    ) -> FunctionReturn<'ast> {
        let content = match result.content {
            AnnotationContent::Wrapper(annotation) => {
                FunctionReturnContent::Annotation(AnnotationContent::Wrapper(annotation))
            }
            AnnotationContent::Type(node) => self.prepare_function_return_type(node),
        };
        FunctionReturn {
            content,
            prefix: result.prefix,
            delimiter: result.delimiter,
        }
    }

    fn prepare_function_return_type(&self, mut node: PreparedType) -> FunctionReturnContent<'ast> {
        while node.source_owner != Some(node.node) {
            let swc_next_ecma_ast::TsTypeData::TsParenthesizedType(parentheses) = node.data else {
                break;
            };
            let parentheses =
                self.prepare_type_parentheses(parentheses, node.source_owner, TypeContext::Root);
            if parentheses.retained {
                return FunctionReturnContent::Parenthesized(parentheses);
            }
            node = parentheses.child;
        }
        FunctionReturnContent::Annotation(AnnotationContent::Type(node))
    }

    pub(super) fn format_function_return(
        &mut self,
        result: FunctionReturn<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_function_return_with_context(result, TypeContext::Root, writer)
    }

    pub(super) fn format_signature_return(
        &mut self,
        result: SignatureReturn<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_signature_return_with_context(result, TypeContext::Root, writer)
    }

    pub(super) fn format_signature_return_with_context(
        &mut self,
        result: SignatureReturn<'ast>,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_function_return_with_context(
            FunctionReturn {
                content: FunctionReturnContent::Annotation(result.content),
                prefix: result.prefix,
                delimiter: result.delimiter,
            },
            context,
            writer,
        )
    }

    pub(super) fn signature_parameters_can_group(
        &self,
        has_single_parameter: bool,
        generics: Option<TsTypeParameterDeclaration>,
        result: Option<&SignatureReturn<'ast>>,
    ) -> bool {
        if !has_single_parameter {
            return false;
        }
        let Some(result) = result else {
            return false;
        };
        if let AnnotationContent::Type(node) = &result.content
            && node.source_owner.is_none()
            && let TypeBoundaryLayout::Formatted { before, after } = &result.prefix
            && before.is_empty()
            && after.is_empty()
            && !before.gap().requires_line()
            && !after.gap().requires_line()
        {
            let simple = match node.data {
                TsTypeData::TsAnyKeyword(_)
                | TsTypeData::TsUnknownKeyword(_)
                | TsTypeData::TsNeverKeyword(_)
                | TsTypeData::TsVoidKeyword(_)
                | TsTypeData::TsNullKeyword(_)
                | TsTypeData::TsUndefinedKeyword(_)
                | TsTypeData::TsStringKeyword(_)
                | TsTypeData::TsNumberKeyword(_)
                | TsTypeData::TsBigIntKeyword(_)
                | TsTypeData::TsBooleanKeyword(_)
                | TsTypeData::TsSymbolKeyword(_)
                | TsTypeData::TsObjectKeyword(_)
                | TsTypeData::TsIntrinsicKeyword(_)
                | TsTypeData::TsThisType(_) => true,
                TsTypeData::TsTypeReference(reference) => {
                    reference.type_arguments(self.ast).is_none()
                        && self.comments.in_range(reference.span(self.ast)).is_empty()
                }
                _ => false,
            };
            if simple {
                return false;
            }
        }
        let Some(generics) = generics else {
            return true;
        };
        let parameters = generics.params(self.ast);
        if parameters.len() > 1 {
            return false;
        }
        parameters.iter().next().is_none_or(|id| {
            let parameter = self.ast.get_node_in_sub_range(id);
            parameter.constraint(self.ast).is_none() && parameter.default(self.ast).is_none()
        })
    }

    pub(super) fn format_callable_signature_contents(
        &mut self,
        parameters: CallableParameters,
        result: Option<SignatureReturn<'ast>>,
        generics: Option<SignatureGenerics<'ast>>,
        can_group_parameters: bool,
        group_return: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if can_group_parameters && let Some(result) = result {
            let (parameters, ()) = writer.record_group_with_value(|writer| {
                self.format_callable_signature_parameters(parameters, generics, writer)
            })?;
            let result = self.prepare_function_return(result);
            let object_return = result.has_object_type();
            let (result, tail) = writer.record_with_value(|writer| {
                if group_return {
                    writer.group(false, |writer| self.format_function_return(result, writer))
                } else {
                    self.format_function_return(result, writer)
                }
            })?;
            writer.write_fragment(if object_return || result.preferred_breaks() {
                parameters.grouped()
            } else {
                parameters.contents()
            });
            writer.write_fragment(result);
            return Ok(tail);
        }
        if result
            .as_ref()
            .is_some_and(|result| result.preceding_mode == ContentMode::Verbatim)
        {
            writer.group(false, |writer| {
                self.format_callable_signature_parameters(parameters, generics, writer)
            })?;
        } else {
            self.format_callable_signature_parameters(parameters, generics, writer)?;
        }
        match result {
            Some(result) if group_return => {
                writer.group(false, |writer| self.format_signature_return(result, writer))
            }
            Some(result) => self.format_signature_return(result, writer),
            None => Ok(None),
        }
    }

    fn format_callable_signature_parameters(
        &mut self,
        parameters: CallableParameters,
        generics: Option<SignatureGenerics<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if let Some(generic) = generics {
            self.format_type_parameters(
                generic.node,
                TypeParameterContext::Callable,
                generic.mode,
                writer,
            )?;
            self.format_type_gap(generic.following, FlatSeparator::Empty, writer)?;
        }
        self.format_callable_parameters(parameters, writer)
    }

    fn format_function_return_with_context(
        &mut self,
        result: FunctionReturn<'ast>,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let layout = match result.prefix {
            TypeBoundaryLayout::Source(gap) => {
                self.format_verbatim(gap, writer)?;
                TypeLayout::OpeningProvided
            }
            TypeBoundaryLayout::Formatted { before, after } => {
                self.format_comment_boundary(&before, writer)?;
                before
                    .gap()
                    .write(writer, result.delimiter.preceding_space(), |writer| {
                        if matches!(result.delimiter, SignatureReturnDelimiter::Arrow) {
                            writer.space();
                        }
                    });
                writer.text(result.delimiter.token());
                if let FunctionReturnContent::Annotation(AnnotationContent::Type(node)) =
                    &result.content
                {
                    return self.format_prepared_type_after_comments(
                        *node,
                        context,
                        &after,
                        TypeLayout::Root,
                        writer,
                    );
                }
                if let FunctionReturnContent::Parenthesized(parentheses) = result.content {
                    return self.format_prepared_parentheses_after_comments(
                        parentheses,
                        &after,
                        context,
                        TypeLayout::Root,
                        writer,
                    );
                }
                self.format_type_comment_gap(&after, FlatSeparator::Space, writer)?;
                TypeLayout::Root
            }
        };
        match result.content {
            FunctionReturnContent::Annotation(AnnotationContent::Wrapper(annotation)) => {
                self.format_verbatim(annotation.span(self.ast), writer)?;
                Ok(None)
            }
            FunctionReturnContent::Annotation(AnnotationContent::Type(node)) => {
                self.format_prepared_type_with_layout(node, context, layout, writer)
            }
            FunctionReturnContent::Parenthesized(parentheses) => {
                self.format_prepared_type_parentheses(parentheses, context, layout, writer)
            }
        }
    }

    fn format_annotation_owner(
        &mut self,
        content: AnnotationContent,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match content {
            AnnotationContent::Wrapper(annotation) => {
                self.format_verbatim(annotation.span(self.ast), writer)?;
                Ok(None)
            }
            AnnotationContent::Type(node) => self.format_prepared_type(node, context, writer),
        }
    }

    fn last_callable_parameter(
        &self,
        parameters: &CallableParameters,
    ) -> Result<Option<CallableParameter>, FormatError> {
        let ast = self.ast;
        if let Some(rest) = parameters.node.rest(ast) {
            return Ok(Some(CallableParameter::Rest(rest)));
        }
        if let Some(id) = parameters.node.items(ast).iter().next_back() {
            let item = ast.get_node_in_sub_range(id);
            return match ast.formal_parameter_item_data(item) {
                FormalParameterItemData::FormalParameter(node) => {
                    Ok(Some(CallableParameter::Formal(node)))
                }
                FormalParameterItemData::TsParameterProperty(_) => {
                    Err(self.unsupported("callable type parameter property", item))
                }
            };
        }
        Ok(parameters.this_parameter.map(CallableParameter::This))
    }

    pub(super) fn format_callable_parameters(
        &mut self,
        parameters: CallableParameters,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = parameters.node.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let has_parameters = parameters.has_parameters();
        let hug = parameters
            .hug
            .filter(|_| parameters.source_head.is_none() && parameters.suppressed_tail.is_none());
        let members = parameters
            .this_parameter
            .into_iter()
            .map(|node| Ok(CallableParameter::This(node)))
            .chain(parameters.node.items(ast).iter().map(|id| {
                let item = ast.get_node_in_sub_range(id);
                match ast.formal_parameter_item_data(item) {
                    FormalParameterItemData::FormalParameter(node) => {
                        Ok(CallableParameter::Formal(node))
                    }
                    FormalParameterItemData::TsParameterProperty(_) => {
                        Err(FormatError::unsupported(
                            "callable type parameter property",
                            None,
                            item.span(ast),
                        ))
                    }
                }
            }))
            .chain(
                parameters
                    .node
                    .rest(ast)
                    .into_iter()
                    .map(|node| Ok(CallableParameter::Rest(node))),
            );
        writer.text("(");
        let write_items = |writer: &mut Writer<'ast>| {
            if hug.is_none() && (has_parameters || !self.comments.in_range(content).is_empty()) {
                self.opening_line_constraint(content).write(
                    writer,
                    FlatSeparator::Empty,
                    Writer::soft_line,
                );
            }
            if let Some(head) = parameters.incoming_head {
                let comments = self.comments.in_range(head.packet);
                self.comments.claim(comments)?;
                self.format_comment_run(
                    comments,
                    CommentRunLayout::FlatLeading(head.target_start),
                    writer,
                )?;
            }
            let mut members = members.peekable();
            let mut end = content.start;
            let mut following_mode = None;
            let mut separator = Separator::Line;
            let mut has_previous = false;
            while let Some(member) = members.next() {
                let member = member?;
                let member_span = member.span(ast);
                if has_previous {
                    let next = self
                        .comments
                        .between(Span::new(end, member_span.start))?
                        .first()
                        .map_or(member_span.start, |comment| comment.span.start);
                    ArgumentLayout::Lines.separator(
                        separator,
                        self.line_constraint(end, next),
                        writer,
                    );
                }
                let following = match members.peek() {
                    Some(Ok(node)) => FollowingItem::Node(node.span(ast)),
                    Some(Err(_)) | None => FollowingItem::End,
                };
                let following = parameters
                    .terminal
                    .as_ref()
                    .filter(|terminal| terminal.parameter == member)
                    .map_or(following, |_| {
                        FollowingItem::ResolvedTerminal(DelimitedSuppression {
                            preceding: Some(member_span),
                            following: ContentMode::Formatted,
                        })
                    });
                let mut item = self.prepare_delimited_item(
                    member_span,
                    following,
                    Span::new(end, content.end),
                    hug.is_none()
                        && !matches!(member, CallableParameter::Rest(_))
                        && self.options.trailing_commas == TrailingCommas::All,
                    writer,
                )?;
                if let Some(mode) = following_mode {
                    item.set_leading_mode(mode);
                }
                if parameters.source_head == Some(member)
                    || parameters.suppressed_tail == Some(member)
                {
                    item.set_leading_mode(ContentMode::Verbatim);
                }
                let prepared = self.prepare_callable_parameter(member, item.mode)?;
                let prepared = if let Some(hug) = hug {
                    prepared.share_signature_group(hug)
                } else {
                    prepared
                };
                let tail = self.format_prepared_parameter_item(prepared, writer)?;
                let boundary = self.finish_delimited_item(item, tail, writer)?;
                end = boundary.end;
                following_mode = Some(boundary.following_mode);
                separator = boundary.separator;
                has_previous = true;
            }
            let dangling = self.comments.between(Span::new(end, content.end))?;
            let boundary = self.prepare_comment_boundary(dangling, end, content.end);
            self.comments.claim(dangling)?;
            self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
            Ok::<_, FormatError>((
                dangling.last().map_or(end, |comment| comment.span.end),
                boundary.gap(),
            ))
        };
        let (end, closing) = if hug.is_some() {
            write_items(writer)?
        } else {
            writer.indent(write_items)?
        };
        if hug.is_none() && (has_parameters || end != content.start) {
            if let Some(terminal) = &parameters.terminal {
                terminal
                    .closing
                    .write(writer, FlatSeparator::Empty, Writer::soft_line);
            } else {
                closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
            }
        }
        writer.text(")");
        Ok(())
    }

    fn prepare_callable_parameter(
        &self,
        node: CallableParameter,
        mode: ContentMode,
    ) -> Result<PreparedParameterItem<'ast>, FormatError> {
        if mode == ContentMode::Verbatim {
            return Ok(PreparedParameterItem(ParameterItemState::Source(node)));
        }
        let ast = self.ast;
        let (annotation, optional) = match node {
            CallableParameter::This(this) => (this.type_annotation(ast), false),
            CallableParameter::Formal(parameter) => {
                (parameter.type_annotation(ast), parameter.optional(ast))
            }
            CallableParameter::Rest(rest) => (rest.type_annotation(ast), false),
        };
        self.prepare_parameter_item(node, annotation, optional)
    }

    fn prepare_parameter_item(
        &self,
        node: CallableParameter,
        annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> Result<PreparedParameterItem<'ast>, FormatError> {
        let ast = self.ast;
        let (binding, annotation, end, initializer) = match node {
            CallableParameter::This(this) => (
                ParameterBinding::This(this),
                annotation,
                this.span(ast).start + 4,
                None,
            ),
            CallableParameter::Formal(parameter) => {
                let pattern = self.parameter_binding(parameter.pattern(ast));
                let initializer = if let Some(right) = parameter.initializer(ast) {
                    let left_end = if let Some(annotation) = annotation {
                        annotation.span(ast).end
                    } else if optional {
                        let gap = Span::new(pattern.span(ast).end, right.span(ast).start);
                        self.find_token(gap, self.comments.in_range(gap), "?")
                            .ok_or_else(|| {
                                self.unsupported("optional callable default", parameter)
                            })?
                            + 1
                    } else {
                        pattern.span(ast).end
                    };
                    let left = Span::new(pattern.span(ast).start, left_end);
                    let owner = self.prepare_default_boundary(left, right)?;
                    let (left_mode, layout, head) = match owner {
                        DefaultOwner::Whole => {
                            return Ok(PreparedParameterItem(ParameterItemState::Source(node)));
                        }
                        DefaultOwner::Bound {
                            gap,
                            left_mode,
                            right_mode,
                        } => (
                            left_mode,
                            InitializerLayout::Bound { gap, right_mode },
                            &[][..],
                        ),
                        DefaultOwner::Formatted {
                            boundary,
                            rhs_layout,
                            head,
                        } => (
                            ContentMode::Formatted,
                            InitializerLayout::Formatted {
                                boundary,
                                rhs_layout,
                            },
                            head,
                        ),
                    };
                    Some(PreparedParameterDefault {
                        left,
                        right,
                        left_mode,
                        layout,
                        head,
                    })
                } else {
                    None
                };
                if !parameter.decorators(ast).is_empty() {
                    return Err(self.unsupported("callable parameter decorator", parameter));
                }
                (
                    ParameterBinding::Pattern {
                        node: pattern,
                        mode: ContentMode::Formatted,
                        spread: None,
                    },
                    annotation,
                    pattern.span(ast).end,
                    initializer,
                )
            }
            CallableParameter::Rest(rest) => {
                if !rest.decorators(ast).is_empty() {
                    return Err(self.unsupported("callable type rest decorator", rest));
                }
                let pattern = self.parameter_binding(rest.argument(ast));
                let gap = Span::new(rest.span(ast).start + 3, pattern.span(ast).start);
                let mode = if self.has_suppression {
                    self.content_mode(self.comments.in_range(gap))
                } else {
                    ContentMode::Formatted
                };
                let spread = self.prepare_type_gap(gap, mode == ContentMode::Verbatim);
                (
                    ParameterBinding::Pattern {
                        node: pattern,
                        mode,
                        spread: Some(spread),
                    },
                    annotation,
                    pattern.span(ast).end,
                    None,
                )
            }
        };
        if initializer.is_none()
            && annotation.is_none()
            && self.has_suppression
            && self.content_mode(self.comments.in_range(Span::new(end, node.span(ast).end)))
                == ContentMode::Verbatim
        {
            return Ok(PreparedParameterItem(ParameterItemState::Source(node)));
        }
        let annotation = if initializer
            .as_ref()
            .is_some_and(|initializer| initializer.left_mode == ContentMode::Verbatim)
        {
            None
        } else {
            annotation
                .map(|annotation| self.prepare_annotation_content(end, annotation))
                .transpose()?
        };
        let left = ParameterLeft {
            binding,
            optional,
            annotation,
            annotation_layout: TypeLayout::Root,
            end: initializer
                .as_ref()
                .map_or(node.span(ast).end, |initializer| initializer.left.end),
        };
        let parameter = match initializer {
            Some(initializer) => ParameterItemState::Default {
                left: if initializer.left_mode == ContentMode::Verbatim {
                    PreparedParameterLeft::Source(initializer.left)
                } else {
                    PreparedParameterLeft::Formatted(left)
                },
                right: initializer.right,
                layout: initializer.layout,
                head: initializer.head,
            },
            None => ParameterItemState::Formatted(left),
        };
        Ok(PreparedParameterItem(parameter))
    }

    pub(super) fn format_prepared_parameter_item(
        &mut self,
        parameter: PreparedParameterItem<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match parameter.0 {
            ParameterItemState::Source(node) => {
                self.format_verbatim(node.span(self.ast), writer)?;
                Ok(None)
            }
            ParameterItemState::Formatted(left) => {
                self.format_parameter_left(PreparedParameterLeft::Formatted(left), writer)
            }
            ParameterItemState::Default {
                left,
                right,
                layout,
                head,
            } => {
                self.comments.claim(head)?;
                self.format_comment_run(
                    head,
                    CommentRunLayout::Leading(right.span(self.ast).start),
                    writer,
                )?;
                let end = left.end();
                let tail = self.format_parameter_left(left, writer)?;
                self.format_expression_tail(tail, writer)?;
                self.format_initializer_right(
                    end,
                    right,
                    layout,
                    ExpressionContext::binding_initializer(),
                    writer,
                )
            }
        }
    }

    fn format_parameter_left(
        &mut self,
        left: PreparedParameterLeft<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let left = match left {
            PreparedParameterLeft::Source(span) => {
                self.format_verbatim(span, writer)?;
                return Ok(None);
            }
            PreparedParameterLeft::Formatted(left) => left,
        };
        let end = match left.binding {
            ParameterBinding::This(this) => {
                writer.text("this");
                this.span(self.ast).start + 4
            }
            ParameterBinding::SignatureObject(pattern) => {
                if let Some(object) = pattern.as_object_pattern(self.ast) {
                    self.format_object_binding_contents(object, writer)?;
                } else {
                    let tail =
                        self.format_parameter_pattern(pattern, ContentMode::Formatted, writer)?;
                    self.format_expression_tail(tail, writer)?;
                }
                pattern.span(self.ast).end
            }
            ParameterBinding::Pattern { node, mode, spread } => {
                if let Some(gap) = spread {
                    writer.text("...");
                    self.format_type_gap(gap, FlatSeparator::Empty, writer)?;
                }
                let tail = self.format_parameter_pattern(node, mode, writer)?;
                self.format_expression_tail(tail, writer)?;
                node.span(self.ast).end
            }
        };
        let optional = left.optional;
        if let Some(annotation) = left.annotation {
            if optional && matches!(&annotation.prefix, TypeBoundaryLayout::Formatted { .. }) {
                writer.text("?");
            }
            self.format_annotation_content_with_layout(annotation, left.annotation_layout, writer)
        } else {
            if optional {
                writer.text("?");
            }
            self.format_source_token_gap(end, left.end, false, writer)?;
            Ok(None)
        }
    }

    pub(super) fn format_type_predicate(
        &mut self,
        node: TsTypePredicate,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let name = node.parameter_name(ast);
        let name_span = name.span(ast);
        let mut name_mode = ContentMode::Formatted;
        let asserts_prefix = node.asserts(ast).then(|| {
            let gap = Span::new(node.span(ast).start + 7, name_span.start);
            if self.has_suppression
                && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim
            {
                name_mode = ContentMode::Verbatim;
            }
            self.prepare_type_gap(gap, name_mode == ContentMode::Verbatim)
        });
        let bound = node
            .type_annotation(ast)
            .map(|annotation| {
                let gap = Span::new(name_span.end, annotation.span(ast).start);
                let comments = self.comments.in_range(gap);
                let Some(keyword) = self.find_token(gap, comments, "is") else {
                    return Err(self.unsupported("type predicate keyword", node));
                };
                let split = comments.partition_point(|comment| comment.span.end <= keyword);
                let (before, after) = comments.split_at(split);
                let mut mode = ContentMode::Formatted;
                if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
                    for (_, position) in self.suppressed_comment_runs(before, keyword) {
                        if matches!(position, SuppressionPosition::OwnLine) {
                            mode = ContentMode::Verbatim;
                        } else {
                            name_mode = ContentMode::Verbatim;
                        }
                    }
                    for (_, position) in
                        self.suppressed_comment_runs(after, annotation.span(ast).start)
                    {
                        if matches!(position, SuppressionPosition::EndOfLine) {
                            name_mode = ContentMode::Verbatim;
                        } else {
                            mode = ContentMode::Verbatim;
                        }
                    }
                }
                let prefix = if name_mode == ContentMode::Verbatim || mode == ContentMode::Verbatim
                {
                    TypeBoundaryLayout::Source(gap)
                } else {
                    TypeBoundaryLayout::Formatted {
                        before: self.prepare_comment_boundary(before, gap.start, keyword),
                        after: self.prepare_comment_boundary(after, keyword + 2, gap.end),
                    }
                };
                let content = if mode == ContentMode::Verbatim {
                    AnnotationContent::Wrapper(annotation)
                } else {
                    AnnotationContent::Type(
                        self.prepare_type(annotation.type_annotation(ast), ContentMode::Formatted),
                    )
                };
                Ok(PredicateBound { content, prefix })
            })
            .transpose()?;
        if let Some(prefix) = asserts_prefix {
            writer.text("asserts");
            self.format_type_gap(prefix, FlatSeparator::Space, writer)?;
        }
        if name_mode == ContentMode::Verbatim {
            self.format_verbatim(name_span, writer)?;
        } else {
            match ast.ts_type_predicate_name_data(name) {
                TsTypePredicateNameData::IdentifierName(name) => {
                    writer.text(ast.get_utf8(name.name(ast)))
                }
                TsTypePredicateNameData::TsThisType(_) => writer.text("this"),
            }
        }
        if let Some(bound) = bound {
            match bound.prefix {
                TypeBoundaryLayout::Source(gap) => self.format_verbatim(gap, writer)?,
                TypeBoundaryLayout::Formatted { before, after } => {
                    self.format_comment_boundary(&before, writer)?;
                    before
                        .gap()
                        .write(writer, FlatSeparator::Space, Writer::space);
                    writer.text("is");
                    self.format_comment_boundary(&after, writer)?;
                    after
                        .gap()
                        .write(writer, FlatSeparator::Space, Writer::space);
                }
            }
            return self.format_annotation_owner(bound.content, TypeContext::Root, writer);
        }
        Ok(None)
    }
}
