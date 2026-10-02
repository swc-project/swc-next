//! Object method signatures and their lexical prefix boundaries.

use swc_next_ecma_ast::{
    ExprData, Function, GetSpan, ObjectProperty, PropertyKey, PropertyKeyData, PropertyKind, Span,
};

use super::{FunctionNameModes, FunctionSignatureOwner, PreparedFunctionSignature};
use crate::formatter::comments::{CommentBinding, CommentBoundary};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::objects::{PropertyKeyContext, PropertyKeyQuoting};
use crate::formatter::{
    Formatter,
    suppression::{ContentMode, SuppressionPosition},
};
use crate::{FormatError, document::Writer};

pub(in crate::formatter) struct SourceMethodHeader<'ast> {
    property: ObjectProperty,
    prefix: SourceMethodPrefix,
    enclosing_mode: ContentMode,
    signature: MethodSignature<'ast>,
}

struct SourceMethodPrefix {
    start: u32,
    keyword: Option<(&'static str, Span)>,
    star: Option<u32>,
    key: PropertyKey,
    computed: Option<Span>,
    key_mode: ContentMode,
    key_context: PropertyKeyContext,
    quoting: PropertyKeyQuoting,
}

struct MethodKeyInput {
    start: u32,
    key: PropertyKey,
    computed: bool,
    keyword: Option<&'static str>,
    generator: bool,
    header_start: u32,
    quoting: PropertyKeyQuoting,
}

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

impl SourceMethodPrefix {
    fn end(&self, ast: &swc_next_ecma_ast::Ast<'_>) -> u32 {
        self.computed
            .map_or(self.key.span(ast).end, |span| span.end)
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn source_method_header(
        &self,
        property: ObjectProperty,
        quoting: PropertyKeyQuoting,
    ) -> Result<SourceMethodHeader<'ast>, FormatError> {
        let ast = self.ast;
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
        let key = property.key(ast);
        let start = property.span(ast).start;
        let mut signature =
            self.source_function_signature(function, FunctionSignatureOwner::Method { key })?;
        let header_start = signature.span.start;
        let mut prefix = self.source_method_key_prefix(MethodKeyInput {
            start,
            key,
            computed: property.computed(ast),
            keyword,
            generator: function.generator(ast),
            header_start,
            quoting,
        })?;
        let computed = prefix.computed;
        let key_span = key.span(ast);
        let modes = if self.has_suppression
            && computed.is_none()
            && matches!(
                ast.property_key_data(key),
                PropertyKeyData::IdentifierName(_)
            ) {
            // Babel attaches this packet to the Identifier key. An own-line
            // directive still selects the following parameter or body owner.
            let comments = self
                .comments
                .in_range(Span::new(key_span.end, header_start));
            FunctionNameModes {
                name: ContentMode::Formatted,
                following: self.content_mode(comments),
            }
        } else {
            self.function_name_modes(Span::new(key_span.end, header_start))
        };
        let enclosing_mode = signature.enclosing_mode();
        let function_prefix = if self.has_suppression
            && enclosing_mode == ContentMode::Formatted
            && computed.is_none()
            && matches!(ast.property_key_data(key), PropertyKeyData::Expr(expression)
                if matches!(ast.expr_data(expression), ExprData::StringLiteral(_)))
        {
            let function_span = function.span(ast);
            let key_end = prefix.end(ast);
            let gap = Span::new(key_end, function_span.start);
            let comments = self.comments.in_range(gap);
            if self
                .suppression_positions(comments, function_span.start)
                .any(|position| matches!(position, SuppressionPosition::OwnLine))
            {
                // The Function value owns its parameters and body together.
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
        let (signature, signature_key_mode) = match function_prefix {
            Some(prefix) => {
                let key_mode = signature.key_mode(key);
                (
                    MethodSignature::FunctionValue { function, prefix },
                    key_mode,
                )
            }
            None => {
                let signature = self.prepare_function_signature(signature);
                let key_mode = signature.key_mode(key);
                (MethodSignature::Formatted(signature), key_mode)
            }
        };
        prefix.key_mode =
            if signature_key_mode == ContentMode::Verbatim || modes.name == ContentMode::Verbatim {
                ContentMode::Verbatim
            } else if self.has_suppression {
                self.content_mode(self.comments.in_range(Span::new(start, key_span.start)))
            } else {
                ContentMode::Formatted
            };
        if matches!(signature, MethodSignature::FunctionValue { .. }) {
            // Keep the StringLiteral identity targeted by this Function packet.
            prefix.key_context = PropertyKeyContext::FunctionValueBinding;
        }
        Ok(SourceMethodHeader {
            property,
            prefix,
            enclosing_mode,
            signature,
        })
    }

    fn source_method_key_prefix(
        &self,
        input: MethodKeyInput,
    ) -> Result<SourceMethodPrefix, FormatError> {
        let MethodKeyInput {
            start,
            key,
            computed,
            keyword,
            generator,
            header_start,
            quoting,
        } = input;
        let ast = self.ast;
        let key_span = key.span(ast);
        let keyword =
            keyword.map(|keyword| (keyword, Span::new(start, start + keyword.len() as u32)));
        let keyword_end = keyword.map_or(start, |(_, span)| span.end);
        let star = if generator {
            let gap = Span::new(keyword_end, key_span.start);
            Some(
                self.find_token(gap, self.comments.in_range(gap), "*")
                    .ok_or_else(|| self.unsupported("method generator boundary", key))?,
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
            start,
            keyword,
            star,
            key,
            computed,
            key_mode: ContentMode::Formatted,
            key_context: PropertyKeyContext::Ordinary,
            quoting,
        })
    }

    pub(in crate::formatter) fn format_method_header(
        &mut self,
        header: SourceMethodHeader<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let SourceMethodHeader {
            property,
            prefix,
            enclosing_mode,
            signature,
        } = header;
        if enclosing_mode == ContentMode::Verbatim {
            return self.format_verbatim(property.span(self.ast), writer);
        }
        let signature_span = match &signature {
            MethodSignature::Formatted(signature) => signature.span,
            MethodSignature::FunctionValue { function, .. } => function.span(self.ast),
        };
        let key_end = prefix.end(self.ast);
        self.format_method_key_prefix(prefix, writer)?;
        match signature {
            MethodSignature::Formatted(signature) => {
                self.format_source_token_gap(key_end, signature_span.start, false, writer)?;
                let body = self.format_function_signature(signature, writer)?;
                self.format_function_body_boundary(body, writer)
            }
            MethodSignature::FunctionValue { prefix, .. } => {
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
            start,
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
        let mut head_end = start;
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
