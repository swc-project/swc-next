//! Function headers and their actual parameter/body boundaries.
pub(super) mod bodyless;
mod methods;
use super::Formatter;
use super::comments::{CommentBinding, CommentRunLayout};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::parameters::{CallbackParameterShape, FunctionParameters};
use super::statements::{IncomingStatement, StatementBodyLeading};
use super::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use super::types::TypeGap;
use super::types::callable::{
    SignatureParameterLayout, SignatureReturn, SignatureReturnDelimiter, SignatureReturnEolTarget,
};
use super::types::generics::{TypeListKind, TypeListPrefix, TypeParameterContext};
use crate::{FormatError, document::Writer};
pub(super) use methods::{MethodOwner, SourceMethodHeader};
use swc_next_ecma_ast::{
    BindingIdentifier, Comment, CommentKind, Function, FunctionBody, GetSpan, PropertyKey, Span,
    Stmt, StmtData, TsTypeAnnotation, TsTypeParameterDeclaration,
};
pub(super) struct SourceFunctionSignature<'ast> {
    span: Span,
    parameters: FunctionParameters,
    body: SourceFunctionBody<'ast>,
    preceding: Option<SignaturePreceding>,
    result: Option<FunctionReturnInput>,
    generics: Option<FunctionGenericInput>,
}
pub(super) struct PreparedFunctionSignature<'ast> {
    span: Span,
    parameters: FunctionParameters,
    body: PreparedFunctionBody<'ast>,
    preceding: Option<SignaturePreceding>,
    result: Option<SignatureReturn<'ast>>,
    generics: Option<PreparedFunctionGenerics<'ast>>,
}
struct FunctionGenericInput {
    node: TsTypeParameterDeclaration,
    owner: Option<SignaturePreceding>,
    mode: ContentMode,
}

enum FunctionGenericPrefix<'ast> {
    Header(TypeListPrefix<'ast>),
    Anonymous(TypeGap<'ast>),
}

struct PreparedFunctionGenerics<'ast> {
    node: TsTypeParameterDeclaration,
    mode: ContentMode,
    prefix: FunctionGenericPrefix<'ast>,
    following: TypeGap<'ast>,
}

struct FunctionReturnInput {
    annotation: TsTypeAnnotation,
    source_return: Option<TsTypeAnnotation>,
}

pub(super) enum FunctionSignatureOwner {
    Function,
    Method {
        key: PropertyKey,
        kind: MethodSignatureKind,
    },
    Constructor {
        method: swc_next_ecma_ast::MethodDefinition,
        key: PropertyKey,
    },
}

#[derive(Clone, Copy)]
pub(super) enum MethodSignatureKind {
    Public,
    Private,
}

#[derive(Clone, Copy)]
pub(super) enum SignaturePreceding {
    Name(BindingIdentifier),
    Key(PropertyKey),
}

pub(super) enum SignatureBoundaryRole {
    Function { has_preceding: bool },
    PrivateMethod { has_preceding: bool },
    Arrow { has_parameters: bool },
}

pub(super) struct SignatureBodyModes {
    preceding: ContentMode,
    pub(super) body: ContentMode,
    pub(super) first_statement: Option<IncomingStatement>,
    pub(super) constraint: LineConstraint,
}

impl SourceFunctionSignature<'_> {
    pub(super) fn header_start(&self, ast: &swc_next_ecma_ast::Ast<'_>) -> u32 {
        self.generics
            .as_ref()
            .map_or(self.span.start, |generic| generic.node.span(ast).start)
    }

    pub(super) fn has_generics(&self) -> bool {
        self.generics.is_some()
    }

    pub(super) fn key_mode(&self, key: PropertyKey) -> ContentMode {
        if matches!(self.preceding, Some(SignaturePreceding::Key(target)) if target == key) {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        }
    }

    pub(super) fn enclosing_mode(&self) -> ContentMode {
        self.parameters.source.enclosing_mode()
    }
    pub(super) fn has_parameters(&self) -> bool {
        self.parameters.source.has_parameters()
    }

    pub(super) fn set_leading_mode(&mut self, mode: ContentMode) {
        if let Some(generic) = &mut self.generics {
            if mode == ContentMode::Verbatim {
                generic.mode = mode;
            }
        } else if self.parameters.source.has_parameters() {
            self.parameters.source.set_leading_mode(mode);
        } else if mode == ContentMode::Verbatim {
            if let Some(result) = &mut self.result {
                result.source_return = Some(result.annotation);
            } else {
                self.body.mode = mode;
            }
        }
    }
}
impl PreparedFunctionSignature<'_> {
    pub(super) fn has_generics(&self) -> bool {
        self.generics.is_some()
    }

    pub(super) fn key_mode(&self, key: PropertyKey) -> ContentMode {
        if matches!(self.preceding, Some(SignaturePreceding::Key(target)) if target == key) {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        }
    }

    fn name_mode(&self, name: BindingIdentifier) -> ContentMode {
        if matches!(self.preceding, Some(SignaturePreceding::Name(target)) if target == name) {
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
        let signature =
            self.source_function_signature(function, FunctionSignatureOwner::Function)?;
        if signature.parameters.source.enclosing_mode() == ContentMode::Verbatim {
            return self.format_verbatim(function.span(self.ast), writer);
        }
        let body = self.format_function_head(function, signature, writer)?;
        self.format_function_body_boundary(body, writer)
    }
    fn format_function_head(
        &mut self,
        function: Function,
        signature: SourceFunctionSignature<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedFunctionBody<'ast>, FormatError> {
        self.format_function_head_with_shape(function, signature, None, writer)
    }
    pub(super) fn format_function_head_with_shape(
        &mut self,
        function: Function,
        mut signature: SourceFunctionSignature<'ast>,
        callback_shape: Option<&mut CallbackParameterShape>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedFunctionBody<'ast>, FormatError> {
        let ast = self.ast;
        let header_start = signature.header_start(ast);
        let has_generics = signature.has_generics();
        let start = function.span(ast).start;
        let name = match &signature.generics {
            Some(generic) => match generic.owner {
                Some(SignaturePreceding::Name(name)) => Some(name),
                _ => None,
            },
            None => function.id(ast),
        };
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
            _ if has_generics => FunctionNameModes {
                name: ContentMode::Formatted,
                following: ContentMode::Formatted,
            },
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
        let prefix_end = name.map_or(header_end, |name| name.span(ast).end);
        let signature = self.prepare_function_signature(signature, prefix_end)?;
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
        if name.is_some() || !has_generics {
            self.format_source_token_gap(header_end, head_end, true, writer)?;
        }
        if let Some(name) = name {
            let name_span = name.span(ast);
            if name_modes.name == ContentMode::Verbatim {
                self.format_verbatim(name_span, writer)?;
            } else {
                writer.text(ast.get_utf8(name.name(ast)));
            }
            if !has_generics {
                self.format_source_token_gap(name_span.end, header_start, false, writer)?;
            }
        }
        self.format_function_signature_with_shape(signature, callback_shape, writer)
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

    pub(super) fn source_function_signature(
        &self,
        function: Function,
        owner: FunctionSignatureOwner,
    ) -> Result<SourceFunctionSignature<'ast>, FormatError> {
        let ast = self.ast;
        let receiver = function.this_param(ast);
        let params = function.params(ast);
        let body = function.body(ast);
        let private = matches!(
            owner,
            FunctionSignatureOwner::Method {
                kind: MethodSignatureKind::Private,
                ..
            }
        );
        let mut parameters = match owner {
            FunctionSignatureOwner::Constructor { method, .. } => {
                self.source_constructor_parameters(params, method)?
            }
            _ if private => self.source_private_method_parameters(params, body)?,
            _ => self.source_function_parameters(params, receiver)?,
        };
        let parameter_key_mode = parameters.source.key_mode();
        let parameter_body_mode = parameters.source.body_mode(body);
        let span = params.span(ast);
        let body_span = body.span(ast);
        // Whole parameter-list suppression exits before annotation preparation.
        let result = if parameters.source.enclosing_mode() == ContentMode::Formatted {
            function.return_type(ast)
        } else {
            None
        };
        let mut generics = if parameters.source.enclosing_mode() == ContentMode::Formatted {
            function.type_parameters(ast).map(|node| {
                let owner = match owner {
                    FunctionSignatureOwner::Function => {
                        function.id(ast).map(SignaturePreceding::Name)
                    }
                    FunctionSignatureOwner::Method { key, .. }
                    | FunctionSignatureOwner::Constructor { key, .. } => {
                        Some(SignaturePreceding::Key(key))
                    }
                };
                FunctionGenericInput {
                    node,
                    owner,
                    mode: ContentMode::Formatted,
                }
            })
        } else {
            None
        };
        let signature_end = result.map_or(span.end, |annotation| annotation.span(ast).end);
        let comments = self
            .comments
            .in_range(Span::new(signature_end, body_span.start));
        let previous = if parameter_key_mode == ContentMode::Verbatim
            || (result.is_none() && self.content_mode(comments) == ContentMode::Verbatim)
        {
            if let Some(generic) = &generics {
                generic.owner
            } else {
                match owner {
                    FunctionSignatureOwner::Function => {
                        function.id(ast).map(SignaturePreceding::Name)
                    }
                    FunctionSignatureOwner::Method { key, .. }
                    | FunctionSignatureOwner::Constructor { key, .. } => {
                        Some(SignaturePreceding::Key(key))
                    }
                }
            }
        } else {
            None
        };
        let mut modes = self.signature_body_modes(
            body,
            comments,
            body_span.start,
            signature_end,
            if private {
                SignatureBoundaryRole::PrivateMethod {
                    has_preceding: result.is_some()
                        || parameters.source.has_parameters()
                        || previous.is_some(),
                }
            } else {
                SignatureBoundaryRole::Function {
                    has_preceding: result.is_some()
                        || parameters.source.has_parameters()
                        || previous.is_some()
                        || generics.is_some(),
                }
            },
        );
        if parameter_body_mode == ContentMode::Verbatim {
            modes.body = parameter_body_mode;
        }
        let preceding = if parameter_key_mode == ContentMode::Verbatim {
            previous
        } else if result.is_none() && modes.preceding == ContentMode::Verbatim {
            if parameters.source.has_parameters() {
                parameters.source.suppress_last(ast)?;
                None
            } else if let Some(generic) = &mut generics {
                generic.mode = ContentMode::Verbatim;
                None
            } else {
                previous
            }
        } else {
            None
        };
        let result = result.map(|annotation| FunctionReturnInput {
            annotation,
            source_return: (modes.preceding == ContentMode::Verbatim).then_some(annotation),
        });
        Ok(SourceFunctionSignature {
            span,
            parameters,
            body: SourceFunctionBody {
                body,
                signature_end,
                comments,
                mode: modes.body,
                first_statement: modes.first_statement,
                constraint: modes.constraint,
            },
            preceding,
            result,
            generics,
        })
    }
    pub(super) fn prepare_function_signature(
        &self,
        signature: SourceFunctionSignature<'ast>,
        prefix_end: u32,
    ) -> Result<PreparedFunctionSignature<'ast>, FormatError> {
        let SourceFunctionSignature {
            span,
            mut parameters,
            mut body,
            mut preceding,
            mut result,
            generics,
        } = signature;
        let mut generics = generics.map(|input| {
            let mut mode = input.mode;
            let prefix_gap = Span::new(prefix_end, input.node.span(self.ast).start);
            let prefix = if let Some(owner) = input.owner {
                let prefix = self.prepare_type_list_prefix(prefix_gap, TypeListKind::Parameters);
                if prefix.left_mode == ContentMode::Verbatim {
                    preceding = Some(owner);
                }
                if prefix.right_mode == ContentMode::Verbatim {
                    mode = ContentMode::Verbatim;
                }
                FunctionGenericPrefix::Header(prefix)
            } else {
                let source = self.has_suppression
                    && self.content_mode(self.comments.in_range(prefix_gap))
                        == ContentMode::Verbatim;
                if source {
                    mode = ContentMode::Verbatim;
                }
                FunctionGenericPrefix::Anonymous(self.prepare_type_gap(prefix_gap, source))
            };
            let gap = Span::new(input.node.span(self.ast).end, span.start);
            let comments = self.comments.in_range(gap);
            let source =
                self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
            if source {
                for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                    if matches!(position, SuppressionPosition::EndOfLine)
                        || (!parameters.source.has_parameters()
                            && matches!(position, SuppressionPosition::Inline))
                    {
                        mode = ContentMode::Verbatim;
                    } else if parameters.source.has_parameters() {
                        parameters.source.set_leading_mode(ContentMode::Verbatim);
                    } else if let Some(result) = &mut result {
                        result.source_return = Some(result.annotation);
                    } else {
                        body.mode = ContentMode::Verbatim;
                    }
                }
            }
            PreparedFunctionGenerics {
                node: input.node,
                mode,
                prefix,
                following: self.prepare_type_gap(gap, source),
            }
        });
        let result = if let Some(input) = result {
            let eol_target = if parameters.source.has_parameters() {
                SignatureReturnEolTarget::Parameter
            } else if let Some(generic) = &mut generics {
                SignatureReturnEolTarget::Header(&mut generic.mode)
            } else {
                SignatureReturnEolTarget::Annotation
            };
            let result = self.prepare_signature_return(
                span.end,
                eol_target,
                input.annotation,
                input.source_return,
                SignatureReturnDelimiter::Colon,
            );
            if result.preceding_mode == ContentMode::Verbatim {
                parameters.source.suppress_last(self.ast)?;
            }
            Some(result)
        } else {
            None
        };
        let body = self.prepare_function_body(body);
        Ok(PreparedFunctionSignature {
            span,
            parameters,
            body,
            preceding,
            result,
            generics,
        })
    }
    pub(super) fn format_function_signature(
        &mut self,
        signature: PreparedFunctionSignature<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedFunctionBody<'ast>, FormatError> {
        self.format_function_signature_with_shape(signature, None, writer)
    }

    fn format_function_signature_with_shape(
        &mut self,
        signature: PreparedFunctionSignature<'ast>,
        callback_shape: Option<&mut CallbackParameterShape>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedFunctionBody<'ast>, FormatError> {
        let independent_parameters = self.signature_parameters_can_group(
            signature.parameters.has_single_parameter,
            signature.generics.as_ref().map(|generic| generic.node),
            signature.result.as_ref(),
        );
        if let Some(generic) = signature.generics {
            match generic.prefix {
                FunctionGenericPrefix::Header(prefix) => {
                    self.format_type_list_prefix(prefix, writer)?
                }
                FunctionGenericPrefix::Anonymous(gap) => {
                    self.format_type_gap(gap, FlatSeparator::Space, writer)?
                }
            }
            self.format_type_parameters(
                generic.node,
                TypeParameterContext::Callable,
                generic.mode,
                writer,
            )?;
            self.format_type_gap(generic.following, FlatSeparator::Empty, writer)?;
        }
        match signature.result {
            Some(result) if independent_parameters => {
                let (parameters, ()) = writer.record_group_with_value(|writer| {
                    self.format_source_parameters_with_layout(
                        signature.parameters.source,
                        callback_shape,
                        SignatureParameterLayout::Shared,
                        writer,
                    )
                })?;
                let result = self.prepare_function_return(result);
                let object_return = result.has_object_type();
                let (result, ()) = writer.record_with_value(|writer| {
                    let tail = self.format_function_return(result, writer)?;
                    self.format_expression_tail(tail, writer)
                })?;
                writer.group(false, |writer| {
                    writer.write_fragment(if object_return || result.preferred_breaks() {
                        parameters.grouped()
                    } else {
                        parameters.contents()
                    });
                    writer.write_fragment(result);
                });
            }
            result => {
                let layout = if signature.parameters.force_break {
                    SignatureParameterLayout::Forced
                } else if result
                    .as_ref()
                    .is_some_and(|result| result.preceding_mode == ContentMode::Verbatim)
                {
                    SignatureParameterLayout::Independent
                } else {
                    SignatureParameterLayout::Shared
                };
                writer.group(false, |writer| {
                    self.format_source_parameters_with_layout(
                        signature.parameters.source,
                        callback_shape,
                        layout,
                        writer,
                    )?;
                    if let Some(result) = result {
                        let tail = self.format_signature_return(result, writer)?;
                        self.format_expression_tail(tail, writer)?;
                    }
                    Ok::<_, FormatError>(())
                })?;
            }
        }
        Ok(signature.body)
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
                    SignatureBoundaryRole::PrivateMethod {
                        has_preceding: true,
                    } if matches!(position, SuppressionPosition::EndOfLine) => {
                        modes.preceding = ContentMode::Verbatim;
                    }
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
