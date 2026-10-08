//! Method owners share one signature while retaining their lexical prefixes.

use swc_next_ecma_ast::{
    ExprData, Function, GetSpan, MethodDefinition, MethodDefinitionKind, ObjectProperty,
    PropertyKey, PropertyKeyData, PropertyKind, Span,
};

use super::{
    FunctionNameModes, FunctionSignatureOwner, MethodSignatureKind, PreparedFunctionSignature,
};
use crate::formatter::classes::member_prefix::ClassMemberPrefix;
use crate::formatter::comments::{CommentBinding, CommentBoundary};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::objects::{
    ConstructorKeyGrammar, PropertyKeyContext, PropertyKeyQuoting,
};
use crate::formatter::{
    Formatter,
    suppression::{ContentMode, DelimitedFollowing, SuppressionPosition},
};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
pub(in crate::formatter) enum MethodOwner {
    Object(ObjectProperty),
    Class(MethodDefinition),
}

pub(in crate::formatter) struct SourceMethodHeader<'ast> {
    owner: MethodOwner,
    prefix: SourceMethodPrefix,
    parameter_gap: Option<Span>,
    enclosing_mode: ContentMode,
    signature: MethodSignature<'ast>,
}

pub(in crate::formatter) struct SourceMethodPrefix {
    member: ClassMemberPrefix,
    keyword: Option<(&'static str, Span)>,
    star: Option<u32>,
    pub(in crate::formatter) key: PropertyKey,
    computed: Option<Span>,
    pub(in crate::formatter) key_mode: ContentMode,
    key_context: PropertyKeyContext,
    quoting: PropertyKeyQuoting,
}

struct MethodKeyInput {
    start: u32,
    key: PropertyKey,
    computed: bool,
    is_static: bool,
    keyword: Option<&'static str>,
    generator: bool,
    header_start: u32,
    key_context: PropertyKeyContext,
    quoting: PropertyKeyQuoting,
}

#[expect(
    clippy::large_enum_variant,
    reason = "Keep the prepared borrowed header inline instead of allocating per method."
)]
enum MethodSignature<'ast> {
    Formatted(PreparedFunctionSignature<'ast>),
    FunctionValue {
        function: Function,
        prefix: MethodFunctionPrefix<'ast>,
    },
}

struct MethodFunctionPrefix<'ast> {
    boundary: CommentBoundary<'ast>,
    binding: CommentBinding,
    following: LineConstraint,
}

impl MethodOwner {
    fn span(self, ast: &swc_next_ecma_ast::Ast<'_>) -> Span {
        match self {
            Self::Object(property) => property.span(ast),
            Self::Class(method) => method.span(ast),
        }
    }
}

impl SourceMethodHeader<'_> {
    pub(in crate::formatter) fn needs_field_separator(
        &self,
        formatter: &Formatter<'_, '_>,
    ) -> bool {
        if self.enclosing_mode == ContentMode::Verbatim {
            formatter.source_class_member_needs_field_separator(
                self.owner.span(formatter.ast),
                self.prefix.key,
            )
        } else {
            self.prefix.needs_field_separator(formatter)
        }
    }
}

impl SourceMethodPrefix {
    pub(in crate::formatter) fn end(&self, ast: &swc_next_ecma_ast::Ast<'_>) -> u32 {
        self.computed
            .map_or(self.key.span(ast).end, |span| span.end)
    }
    pub(in crate::formatter) fn needs_field_separator(
        &self,
        formatter: &Formatter<'_, '_>,
    ) -> bool {
        if self.member.has_modifiers() || self.keyword.is_some() {
            return false;
        }
        self.star.is_some()
            || self.computed.is_some()
            || matches!(
                formatter.property_key_identifier(
                    self.key,
                    self.key_context,
                    self.key_mode,
                    self.quoting
                ),
                Some("in" | "instanceof")
            )
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn source_method_header(
        &self,
        owner: MethodOwner,
        quoting: PropertyKeyQuoting,
    ) -> Result<SourceMethodHeader<'ast>, FormatError> {
        let ast = self.ast;
        let start = owner.span(ast).start;
        let (function, key, computed, keyword, is_static) = match owner {
            MethodOwner::Object(property) => {
                let value = property.value(ast);
                let ExprData::Function(function) = ast.expr_data(value) else {
                    return Err(self.unsupported("object method value", value));
                };
                let keyword = match property.kind(ast) {
                    PropertyKind::Get => Some("get"),
                    PropertyKind::Set => Some("set"),
                    PropertyKind::Init if function.r#async(ast) => Some("async"),
                    PropertyKind::Init => None,
                };
                (
                    function,
                    property.key(ast),
                    property.computed(ast),
                    keyword,
                    false,
                )
            }
            MethodOwner::Class(method) => {
                if method.optional(ast)
                    || method.accessibility(ast) != swc_next_ecma_ast::Accessibility::None
                    || method.r#abstract(ast)
                    || method.r#override(ast)
                    || !method.decorators(ast).is_empty()
                {
                    return Err(self.unsupported("typed or decorated class method", method));
                }
                let function = method.value(ast);
                let keyword = match method.kind(ast) {
                    MethodDefinitionKind::Get => Some("get"),
                    MethodDefinitionKind::Set => Some("set"),
                    _ if function.r#async(ast) => Some("async"),
                    _ => None,
                };
                (
                    function,
                    method.key(ast),
                    method.computed(ast),
                    keyword,
                    method.r#static(ast),
                )
            }
        };
        let (ordinary_header, getter_header) = match owner {
            MethodOwner::Object(property) => match property.kind(ast) {
                PropertyKind::Init => (true, false),
                PropertyKind::Get => (false, true),
                PropertyKind::Set => (false, false),
            },
            MethodOwner::Class(method) => match method.kind(ast) {
                MethodDefinitionKind::Method => (true, false),
                MethodDefinitionKind::Get => (false, true),
                MethodDefinitionKind::Set | MethodDefinitionKind::Constructor => (false, false),
            },
        };
        if !ordinary_header
            && (function.type_parameters(ast).is_some()
                || (!getter_header && function.return_type(ast).is_some()))
        {
            return Err(self.unsupported("typed accessor or constructor header", function));
        }
        if !ordinary_header && function.this_param(ast).is_some() {
            return Err(self.unsupported("typed function", function));
        }
        // Babel's JS private methods use generic attachment. TS MethodDefinition
        // remains function-like, including private keys.
        let kind = if !ast.lang().is_typescript()
            && matches!(ast.property_key_data(key), PropertyKeyData::Expr(expression)
                if matches!(ast.expr_data(expression), ExprData::PrivateIdentifier(_)))
        {
            MethodSignatureKind::Private
        } else {
            MethodSignatureKind::Public
        };
        let signature_owner = FunctionSignatureOwner::Method { key, kind };
        let mut signature = self.source_function_signature(function, signature_owner)?;
        let header_start = signature.header_start(ast);
        let has_generics = signature.has_generics();
        let mut prefix = self.source_method_key_prefix(MethodKeyInput {
            start,
            key,
            computed,
            is_static,
            keyword,
            generator: function.generator(ast),
            header_start,
            quoting,
            key_context: match owner {
                MethodOwner::Object(_) => PropertyKeyContext::Ordinary,
                MethodOwner::Class(method) => PropertyKeyContext::ClassMethod(
                    if function.generator(ast)
                        || matches!(
                            method.kind(ast),
                            MethodDefinitionKind::Get | MethodDefinitionKind::Set
                        )
                    {
                        ConstructorKeyGrammar::DistinctDiagnostics
                    } else if has_generics {
                        ConstructorKeyGrammar::DistinctClassification
                    } else {
                        ConstructorKeyGrammar::Equivalent
                    },
                ),
            },
        })?;
        let computed = prefix.computed;
        let key_span = key.span(ast);
        let modes = match kind {
            _ if has_generics => FunctionNameModes {
                name: ContentMode::Formatted,
                following: ContentMode::Formatted,
            },
            MethodSignatureKind::Public
                if self.has_suppression
                    && matches!(owner, MethodOwner::Object(_))
                    && computed.is_none()
                    && matches!(
                        ast.property_key_data(key),
                        PropertyKeyData::IdentifierName(_)
                    ) =>
            {
                // Babel attaches this method-name prefix to the actual Identifier
                // key. It does not suppress a Function value or its first formal.
                FunctionNameModes {
                    name: self.content_mode(
                        self.comments
                            .in_range(Span::new(key_span.end, header_start)),
                    ),
                    following: ContentMode::Formatted,
                }
            }
            MethodSignatureKind::Public => {
                self.function_name_modes(Span::new(key_span.end, header_start))
            }
            MethodSignatureKind::Private => {
                let mut modes = FunctionNameModes {
                    name: ContentMode::Formatted,
                    following: ContentMode::Formatted,
                };
                if self.has_suppression {
                    let comments = self
                        .comments
                        .in_range(Span::new(key_span.end, header_start));
                    for position in self.suppression_positions(comments, header_start) {
                        match position {
                            SuppressionPosition::OwnLine => modes.following = ContentMode::Verbatim,
                            SuppressionPosition::Inline if signature.has_parameters() => {
                                modes.following = ContentMode::Verbatim
                            }
                            _ => modes.name = ContentMode::Verbatim,
                        }
                    }
                }
                modes
            }
        };
        let enclosing_mode = signature.enclosing_mode();
        let function_prefix = if self.has_suppression
            && enclosing_mode == ContentMode::Formatted
            && match owner {
                MethodOwner::Object(_) => {
                    computed.is_none()
                        && match ast.property_key_data(key) {
                            PropertyKeyData::IdentifierName(_) => has_generics,
                            PropertyKeyData::Expr(expression) => {
                                matches!(ast.expr_data(expression), ExprData::StringLiteral(_))
                            }
                        }
                }
                MethodOwner::Class(_) => {
                    ast.lang().is_typescript()
                        && (signature.has_parameters() || function.return_type(ast).is_some())
                }
            } {
            let function_span = function.span(ast);
            let key_end = prefix.end(ast);
            let gap = Span::new(key_end, function_span.start);
            let comments = self.comments.in_range(gap);
            if self
                .suppression_positions(comments, function_span.start)
                .any(|position| {
                    if has_generics {
                        !matches!(position, SuppressionPosition::EndOfLine)
                    } else {
                        matches!(position, SuppressionPosition::OwnLine)
                    }
                })
            {
                // The real method value owns the complete Function, including
                // its parameters and body, rather than a first formal.
                let binding = self.comment_binding(comments, gap, Some(function_span));
                let boundary =
                    self.prepare_comment_boundary(comments, key_end, function_span.start);
                let following = self.source_line_constraint(boundary.end(), function_span.start);
                Some(MethodFunctionPrefix {
                    boundary,
                    binding,
                    following,
                })
            } else {
                None
            }
        } else {
            None
        };
        if function_prefix.is_none() {
            signature.set_leading_mode(modes.following);
        }
        let parameter_gap = (self.has_suppression
            && matches!(kind, MethodSignatureKind::Private)
            && self.content_mode(
                self.comments
                    .in_range(Span::new(key_span.end, header_start)),
            ) == ContentMode::Verbatim)
            .then_some(Span::new(key_span.end, header_start));
        let (signature, signature_key_mode) = match function_prefix {
            Some(prefix) => {
                let key_mode = signature.key_mode(key);
                (
                    MethodSignature::FunctionValue { function, prefix },
                    key_mode,
                )
            }
            None => {
                let signature = self.prepare_function_signature(signature, prefix.end(ast))?;
                let key_mode = signature.key_mode(key);
                (MethodSignature::Formatted(signature), key_mode)
            }
        };
        let key_mode = if prefix.key_mode == ContentMode::Verbatim
            || signature_key_mode == ContentMode::Verbatim
            || modes.name == ContentMode::Verbatim
        {
            ContentMode::Verbatim
        } else if self.has_suppression {
            self.content_mode(self.comments.in_range(Span::new(start, key_span.start)))
        } else {
            ContentMode::Formatted
        };
        let key_context = if matches!(owner, MethodOwner::Object(_))
            && matches!(signature, MethodSignature::FunctionValue { .. })
        {
            // Unquoting would turn the actual StringLiteral into an Identifier
            // and move this effective Function-value binding to the key.
            PropertyKeyContext::FunctionValueBinding
        } else {
            prefix.key_context
        };
        prefix.key_context = key_context;
        prefix.key_mode = key_mode;
        Ok(SourceMethodHeader {
            owner,
            prefix,
            parameter_gap,
            enclosing_mode,
            signature,
        })
    }

    fn source_method_key_prefix(
        &self,
        input: MethodKeyInput,
    ) -> Result<SourceMethodPrefix, FormatError> {
        let ast = self.ast;
        let MethodKeyInput {
            start,
            key,
            computed,
            is_static,
            keyword,
            generator,
            header_start,
            key_context,
            quoting,
        } = input;
        let quoting = self.property_key_quoting_context(key, computed, key_context, quoting);
        let member = ClassMemberPrefix::ordinary(start, is_static);
        let prefix_end = member.end();
        let key_span = key.span(ast);
        let keyword = keyword
            .map(|word| {
                let position = if is_static {
                    let gap = Span::new(prefix_end, key_span.start);
                    self.find_token(gap, self.comments.in_range(gap), word)
                        .ok_or_else(|| self.unsupported("method keyword boundary", key))?
                } else {
                    start
                };
                Ok::<_, FormatError>((word, Span::new(position, position + word.len() as u32)))
            })
            .transpose()?;
        let keyword_end = keyword.map_or(prefix_end, |(_, span)| span.end);
        let star = if generator {
            let gap = Span::new(keyword_end, key_span.start);
            Some(
                self.find_token(gap, self.comments.in_range(gap), "*")
                    .ok_or_else(|| self.unsupported("generator method star boundary", key))?,
            )
        } else {
            None
        };
        let head_end = star.map_or(keyword_end, |star| star + 1);
        let computed = if computed {
            let before = Span::new(head_end, key_span.start);
            let opening = self
                .find_token(before, self.comments.in_range(before), "[")
                .ok_or_else(|| self.unsupported("computed method opening boundary", key))?;
            let after = Span::new(key_span.end, header_start);
            let closing = self
                .find_token(after, self.comments.in_range(after), "]")
                .ok_or_else(|| self.unsupported("computed method closing boundary", key))?;
            Some(Span::new(opening, closing + 1))
        } else {
            None
        };
        Ok(SourceMethodPrefix {
            member,
            keyword,
            star,
            key,
            computed,
            key_mode: ContentMode::Formatted,
            key_context,
            quoting,
        })
    }

    pub(in crate::formatter) fn format_method_header(
        &mut self,
        header: SourceMethodHeader<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let SourceMethodHeader {
            owner,
            prefix,
            parameter_gap,
            enclosing_mode,
            signature,
        } = header;
        if enclosing_mode == ContentMode::Verbatim {
            return self.format_verbatim(owner.span(self.ast), writer);
        }
        let signature_span = match &signature {
            MethodSignature::Formatted(signature) => signature.span,
            MethodSignature::FunctionValue { function, .. } => function.span(self.ast),
        };
        let key_end = prefix.end(self.ast);
        self.format_method_key_prefix(prefix, writer)?;
        match signature {
            MethodSignature::Formatted(signature) => {
                if let Some(gap) = parameter_gap {
                    let source =
                        self.format_delimited_source_gap(gap, DelimitedFollowing::Node, writer)?;
                    source.following.write(writer, FlatSeparator::Empty, |_| {});
                } else if !signature.has_generics() {
                    self.format_source_token_gap(key_end, signature_span.start, false, writer)?;
                }
                let body = self.format_function_signature(signature, writer)?;
                self.format_function_body_boundary(body, writer)
            }
            MethodSignature::FunctionValue { prefix, .. } => {
                // A token-only gap does not carry the Function target. Keep the
                // prepared source relationship when its annotation cannot move there.
                self.format_comment_boundary(&prefix.boundary, writer)?;
                if prefix.binding.allows_relocation(None) {
                    prefix
                        .boundary
                        .gap()
                        .write(writer, FlatSeparator::Empty, |_| {});
                } else {
                    prefix.following.write(writer, FlatSeparator::Empty, |_| {});
                }
                self.format_verbatim(signature_span, writer)
            }
        }
    }

    fn format_method_key_prefix(
        &mut self,
        prefix: SourceMethodPrefix,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let SourceMethodPrefix {
            member,
            keyword,
            star,
            key,
            computed,
            key_mode,
            key_context,
            quoting,
        } = prefix;
        let key_span = key.span(self.ast);
        let key_start = computed.map_or(key_span.start, |span| span.start);
        let mut head_end = member.end();
        self.format_class_member_prefix(&member, writer)?;
        if member.has_modifiers() {
            let next = keyword.map_or_else(|| star.unwrap_or(key_start), |(_, span)| span.start);
            self.format_source_token_gap(head_end, next, true, writer)?;
            head_end = next;
        }
        if let Some((keyword, span)) = keyword {
            writer.text(keyword);
            head_end = span.end;
        }
        if let Some(star) = star {
            self.format_source_token_gap(head_end, star, keyword.is_some(), writer)?;
            writer.text("*");
            head_end = star + 1;
        }
        self.format_source_token_gap(
            head_end,
            key_start,
            keyword.is_some() && star.is_none(),
            writer,
        )?;
        self.format_property_key_context(key, computed, key_mode, key_context, quoting, writer)
    }
}
