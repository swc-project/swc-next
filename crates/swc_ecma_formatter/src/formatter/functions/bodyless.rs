//! Real bodyless declarations and class methods share one prepared signature suffix.

use swc_next_ecma_ast::{GetSpan, Span, TsFunction, TsTypeAnnotation, TsTypeParameterDeclaration};

use super::SignaturePreceding;
use crate::formatter::Formatter;
use crate::formatter::comments::ExpressionTail;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::formatter::types::TypeGap;
use crate::formatter::types::callable::{
    CallableParameters, SignatureIncoming, SignatureReturn, SignatureReturnDelimiter,
    SignatureReturnEolTarget,
};
use crate::formatter::types::generics::{TypeListKind, TypeListPrefix, TypeParameterContext};
use crate::{FormatError, document::Writer};

pub(in crate::formatter) struct SourceBodylessSignature {
    pub(in crate::formatter) node: TsFunction,
    parameters: CallableParameters,
    leading_mode: ContentMode,
    pub(in crate::formatter) parameter_span: Span,
    pub(in crate::formatter) generic: Option<TsTypeParameterDeclaration>,
    pub(in crate::formatter) annotation: Option<TsTypeAnnotation>,
}

struct BodylessGenerics<'ast> {
    node: TsTypeParameterDeclaration,
    mode: ContentMode,
    prefix: Option<TypeListPrefix<'ast>>,
    following: TypeGap<'ast>,
}

pub(in crate::formatter) struct PreparedBodylessSignature<'ast> {
    preceding: Option<SignaturePreceding>,
    pub(in crate::formatter) preceding_mode: ContentMode,
    prefix: Option<TypeGap<'ast>>,
    generics: Option<BodylessGenerics<'ast>>,
    parameters: CallableParameters,
    result: Option<SignatureReturn<'ast>>,
    pub(in crate::formatter) content_end: u32,
}

impl SourceBodylessSignature {
    pub(in crate::formatter) fn set_leading_mode(&mut self, mode: ContentMode) {
        self.leading_mode = mode;
    }

    pub(in crate::formatter) fn has_parameters(&self) -> bool {
        self.parameters.has_parameters()
    }
    pub(in crate::formatter) fn header_start(&self, ast: &swc_next_ecma_ast::Ast<'_>) -> u32 {
        self.generic
            .map_or(self.parameter_span.start, |node| node.span(ast).start)
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn source_bodyless_signature(
        &self,
        node: TsFunction,
    ) -> SourceBodylessSignature {
        let ast = self.ast;
        let parameters = node.params(ast);
        SourceBodylessSignature {
            node,
            parameters: self.callable_parameters(parameters, node.this_param(ast)),
            leading_mode: ContentMode::Formatted,
            parameter_span: parameters.span(ast),
            generic: node.type_parameters(ast),
            annotation: node.return_type(ast),
        }
    }

    pub(in crate::formatter) fn bodyless_empty_parameters_are_verbatim(
        &self,
        source: &SourceBodylessSignature,
    ) -> bool {
        self.has_suppression
            && !source.parameters.has_parameters()
            && self.content_mode(self.comments.in_range(source.parameter_span))
                == ContentMode::Verbatim
    }

    pub(in crate::formatter) fn prepare_bodyless_signature_suffix(
        &self,
        source: SourceBodylessSignature,
        prefix_start: u32,
        preceding: Option<SignaturePreceding>,
        mut preceding_mode: ContentMode,
    ) -> Result<PreparedBodylessSignature<'ast>, FormatError> {
        let ast = self.ast;
        let signature_start = source.header_start(ast);
        let SourceBodylessSignature {
            mut parameters,
            leading_mode,
            parameter_span,
            generic,
            annotation,
            ..
        } = source;
        let mut incoming = None;
        let mut source_return = None;
        let mut generic_mode = leading_mode;
        if generic.is_none() && leading_mode == ContentMode::Verbatim {
            match self.signature_incoming(&parameters, annotation, &mut incoming)? {
                SignatureIncoming::Parameter(first) => parameters.source_head = Some(first),
                SignatureIncoming::Return(annotation) => source_return = Some(annotation),
                SignatureIncoming::Empty => preceding_mode = ContentMode::Verbatim,
            }
        }
        let prefix;
        let generic_prefix;
        if generic.is_some() && preceding.is_some() {
            let prepared = self.prepare_type_list_prefix(
                Span::new(prefix_start, signature_start),
                TypeListKind::Parameters {
                    has_left_owner: true,
                },
            );
            if prepared.left_mode == ContentMode::Verbatim {
                preceding_mode = ContentMode::Verbatim;
            }
            if prepared.right_mode == ContentMode::Verbatim {
                generic_mode = ContentMode::Verbatim;
            }
            generic_prefix = Some(prepared);
            prefix = None;
        } else {
            let gap = Span::new(prefix_start, signature_start);
            let comments = self.comments.in_range(gap);
            let source =
                self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
            if source {
                if generic.is_some() {
                    generic_mode = ContentMode::Verbatim;
                } else {
                    for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                        let owner =
                            self.signature_incoming(&parameters, annotation, &mut incoming)?;
                        if preceding.is_some()
                            && (matches!(position, SuppressionPosition::EndOfLine)
                                || matches!(
                                    owner,
                                    SignatureIncoming::Return(_) | SignatureIncoming::Empty
                                ) && matches!(position, SuppressionPosition::Inline))
                        {
                            preceding_mode = ContentMode::Verbatim;
                        } else {
                            match owner {
                                SignatureIncoming::Parameter(first) => {
                                    parameters.source_head = Some(first)
                                }
                                SignatureIncoming::Return(annotation) => {
                                    source_return = Some(annotation)
                                }
                                SignatureIncoming::Empty => {
                                    preceding_mode = ContentMode::Verbatim;
                                }
                            }
                        }
                    }
                }
            }
            prefix = Some(if source {
                TypeGap::Source(gap)
            } else {
                TypeGap::Formatted(self.prepare_comment_boundary(comments, gap.start, gap.end))
            });
            generic_prefix = None;
        }
        let mut generics = generic
            .map(|generic| {
                let gap = Span::new(generic.span(ast).end, parameter_span.start);
                let comments = self.comments.in_range(gap);
                let source =
                    self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
                if source {
                    for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                        let owner =
                            self.signature_incoming(&parameters, annotation, &mut incoming)?;
                        if matches!(position, SuppressionPosition::EndOfLine)
                            || matches!(
                                owner,
                                SignatureIncoming::Return(_) | SignatureIncoming::Empty
                            ) && matches!(position, SuppressionPosition::Inline)
                        {
                            generic_mode = ContentMode::Verbatim;
                        } else {
                            match owner {
                                SignatureIncoming::Parameter(first) => {
                                    parameters.source_head = Some(first)
                                }
                                SignatureIncoming::Return(annotation) => {
                                    source_return = Some(annotation)
                                }
                                SignatureIncoming::Empty => generic_mode = ContentMode::Verbatim,
                            }
                        }
                    }
                }
                Ok::<_, FormatError>(BodylessGenerics {
                    node: generic,
                    mode: generic_mode,
                    prefix: generic_prefix,
                    following: if source {
                        TypeGap::Source(gap)
                    } else {
                        TypeGap::Formatted(
                            self.prepare_comment_boundary(comments, gap.start, gap.end),
                        )
                    },
                })
            })
            .transpose()?;
        let result = annotation.map(|annotation| {
            let eol_target = if parameters.has_parameters() {
                SignatureReturnEolTarget::Parameter
            } else if let Some(generics) = &mut generics {
                SignatureReturnEolTarget::Header(&mut generics.mode)
            } else if preceding.is_some() {
                SignatureReturnEolTarget::Header(&mut preceding_mode)
            } else {
                SignatureReturnEolTarget::Annotation
            };
            self.prepare_signature_return(
                parameter_span.end,
                eol_target,
                annotation,
                source_return,
                SignatureReturnDelimiter::Colon,
            )
        });
        let tail_mode = result
            .as_ref()
            .map_or(ContentMode::Formatted, |result| result.preceding_mode);
        self.prepare_callable_parameter_tail(&mut parameters, tail_mode)?;
        let content_end =
            annotation.map_or(parameter_span.end, |annotation| annotation.span(ast).end);
        Ok(PreparedBodylessSignature {
            preceding,
            preceding_mode,
            prefix,
            generics,
            parameters,
            result,
            content_end,
        })
    }

    pub(in crate::formatter) fn format_bodyless_signature_suffix(
        &mut self,
        signature: PreparedBodylessSignature<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let generics = signature.generics.as_ref().map(|generic| generic.node);
        let can_group_parameters = self.signature_parameters_can_group(
            signature.parameters.count() == 1,
            generics,
            signature.result.as_ref(),
        );
        if let Some(prefix) = signature.prefix {
            self.format_type_gap(
                prefix,
                if signature.preceding.is_some() {
                    FlatSeparator::Empty
                } else {
                    FlatSeparator::Space
                },
                writer,
            )?;
        }
        if let Some(generic) = signature.generics {
            if let Some(prefix) = generic.prefix {
                self.format_type_list_prefix(prefix, writer)?;
            }
            self.format_type_parameters(
                generic.node,
                TypeParameterContext::Callable,
                generic.mode,
                writer,
            )?;
            self.format_type_gap(generic.following, FlatSeparator::Empty, writer)?;
        }
        writer.group(false, |writer| {
            self.format_callable_signature_contents(
                signature.parameters,
                signature.result,
                None,
                can_group_parameters,
                false,
                writer,
            )
        })
    }
}
