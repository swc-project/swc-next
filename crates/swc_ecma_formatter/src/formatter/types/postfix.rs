//! Type postfix delimiters and prefix operands retain their actual owners.

use swc_next_ecma_ast::{
    GetSpan, Span, TsArrayType, TsIndexedAccessType, TsTypeData, TsTypeOperator,
    TsTypeOperatorKind, TsTypeQuery, TsTypeQueryExprNameData,
};

use super::compound::{PendingTypeHead, PreparedTypeContent};
use super::context::TypeContext;
use super::{Formatter, PreparedType, TypeBoundaryLayout, TypeLayout};
use crate::formatter::comments::{CommentBoundary, ExpressionTail};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

enum TypePrefixLayout<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}

pub(super) struct TypePrefix<'ast> {
    pub(super) mode: ContentMode,
    layout: TypePrefixLayout<'ast>,
}

struct TypeBrackets {
    opening: Span,
    closing: Span,
}

enum ArraySuffix<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        inside: CommentBoundary<'ast>,
    },
}

struct PreparedIndex<'ast> {
    object: PreparedType,
    index: PreparedType,
    prefix: TypeBoundaryLayout<'ast>,
    suffix: TypePrefix<'ast>,
}

struct PreparedArray<'ast> {
    element: PreparedType,
    suffix: ArraySuffix<'ast>,
}

pub(super) enum PostfixTypeOwner {
    Array(TsArrayType),
    Indexed(TsIndexedAccessType),
}

pub(super) struct PreparedPostfixType<'ast> {
    data: TsTypeData,
    operand: PostfixOperand<'ast>,
    suffix: PostfixSuffix<'ast>,
}

enum PostfixOperand<'ast> {
    Type(PreparedType),
    Prepared(Box<PreparedTypeContent<'ast>>),
}

enum PostfixSuffix<'ast> {
    Array(ArraySuffix<'ast>),
    Indexed {
        index: PreparedType,
        prefix: TypeBoundaryLayout<'ast>,
        suffix: TypePrefix<'ast>,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    fn type_brackets(&self, span: Span, left_end: u32) -> Result<TypeBrackets, FormatError> {
        let gap = Span::new(left_end, span.end - 1);
        let comments = self.comments.in_range(gap);
        let Some(opening) = self.find_token(gap, comments, "[") else {
            return Err(FormatError::unsupported("type opening bracket", None, gap));
        };
        Ok(TypeBrackets {
            opening: Span::new(opening, opening + 1),
            closing: Span::new(span.end - 1, span.end),
        })
    }

    fn prepare_array_type(&self, node: TsArrayType) -> Result<PreparedArray<'ast>, FormatError> {
        let ast = self.ast;
        let element = node.element_type(ast);
        let element_span = element.span(ast);
        let span = node.span(ast);
        let gap = Span::new(element_span.end, span.end);
        let comments = self.comments.in_range(gap);
        let (mode, suffix) = if self.content_mode(comments) == ContentMode::Verbatim {
            (ContentMode::Verbatim, ArraySuffix::Source(gap))
        } else {
            let brackets = self.type_brackets(span, element_span.end)?;
            let split =
                comments.partition_point(|comment| comment.span.end <= brackets.opening.start);
            let (before, inside) = comments.split_at(split);
            let suffix = ArraySuffix::Formatted {
                before: self.prepare_comment_boundary(
                    before,
                    element_span.end,
                    brackets.opening.start,
                ),
                inside: self.prepare_comment_boundary(
                    inside,
                    brackets.opening.end,
                    brackets.closing.start,
                ),
            };
            (ContentMode::Formatted, suffix)
        };
        Ok(PreparedArray {
            element: self.prepare_type(element, mode),
            suffix,
        })
    }

    pub(super) fn format_array_type(
        &mut self,
        node: TsArrayType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let array = self.prepare_array_type(node)?;
        let tail = self.format_prepared_type(array.element, TypeContext::PostfixOperand, writer)?;
        self.format_expression_tail(tail, writer)?;
        self.format_array_suffix(array.suffix, writer)?;
        Ok(None)
    }

    fn format_array_suffix(
        &mut self,
        suffix: ArraySuffix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match suffix {
            ArraySuffix::Source(gap) => self.format_verbatim(gap, writer)?,
            ArraySuffix::Formatted { before, inside } => {
                self.format_type_comment_gap(&before, FlatSeparator::Empty, writer)?;
                writer.text("[");
                self.format_type_comment_gap(&inside, FlatSeparator::Empty, writer)?;
                writer.text("]");
            }
        }
        Ok(())
    }

    fn prepare_indexed_type(
        &self,
        node: TsIndexedAccessType,
    ) -> Result<PreparedIndex<'ast>, FormatError> {
        let ast = self.ast;
        let object = node.object_type(ast);
        let index = node.index_type(ast);
        let mut index_type = self.prepare_type(index, ContentMode::Formatted);
        let object_span = object.span(ast);
        let index_span = index.span(ast);
        let brackets = self.type_brackets(node.span(ast), object_span.end)?;
        let prefix_span = Span::new(object_span.end, index_span.start);
        let comments = self.comments.in_range(prefix_span);
        let split = comments.partition_point(|comment| comment.span.end <= brackets.opening.start);
        let (before, after) = comments.split_at(split);
        let mut object_mode = self.content_mode(before);
        if self.has_suppression && self.content_mode(after) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(after, index_span.start) {
                if matches!(position, SuppressionPosition::EndOfLine) {
                    object_mode = ContentMode::Verbatim;
                } else {
                    self.select_type_head(&mut index_type, position);
                }
            }
        }
        let prefix = if object_mode == ContentMode::Verbatim || index_type.source_owner.is_some() {
            TypeBoundaryLayout::Source(prefix_span)
        } else {
            TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(
                    before,
                    object_span.end,
                    brackets.opening.start,
                ),
                after: self.prepare_comment_boundary(after, brackets.opening.end, index_span.start),
            }
        };
        let suffix = self.prepare_type_prefix(Span::new(index_span.end, brackets.closing.start));
        if suffix.mode == ContentMode::Verbatim {
            index_type.source_owner = Some(index);
        }
        Ok(PreparedIndex {
            object: self.prepare_type(object, object_mode),
            index: index_type,
            prefix,
            suffix,
        })
    }

    pub(super) fn format_indexed_type(
        &mut self,
        node: TsIndexedAccessType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let index = self.prepare_indexed_type(node)?;
        let tail = self.format_prepared_type(index.object, TypeContext::PostfixOperand, writer)?;
        self.format_expression_tail(tail, writer)?;
        self.format_indexed_suffix(index.index, index.prefix, index.suffix, writer)?;
        Ok(None)
    }

    fn format_indexed_suffix(
        &mut self,
        index: PreparedType,
        prefix: TypeBoundaryLayout<'ast>,
        suffix: TypePrefix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match prefix {
            TypeBoundaryLayout::Source(gap) => self.format_verbatim(gap, writer)?,
            TypeBoundaryLayout::Formatted { before, after } => {
                self.format_type_comment_gap(&before, FlatSeparator::Empty, writer)?;
                writer.text("[");
                self.format_type_comment_gap(&after, FlatSeparator::Empty, writer)?;
            }
        }
        let tail = self.format_prepared_type(index, TypeContext::Root, writer)?;
        self.format_expression_tail(tail, writer)?;
        self.format_type_prefix(suffix, FlatSeparator::Empty, writer)?;
        writer.text("]");
        Ok(())
    }

    pub(super) fn format_postfix_head(
        &mut self,
        owner: PostfixTypeOwner,
        pending: &mut PendingTypeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedPostfixType<'ast>, FormatError> {
        let (data, operand, suffix) = match owner {
            PostfixTypeOwner::Array(node) => {
                let array = self.prepare_array_type(node)?;
                (
                    TsTypeData::TsArrayType(node),
                    array.element,
                    PostfixSuffix::Array(array.suffix),
                )
            }
            PostfixTypeOwner::Indexed(node) => {
                let index = self.prepare_indexed_type(node)?;
                (
                    TsTypeData::TsIndexedAccessType(node),
                    index.object,
                    PostfixSuffix::Indexed {
                        index: index.index,
                        prefix: index.prefix,
                        suffix: index.suffix,
                    },
                )
            }
        };
        let operand =
            self.format_movable_type_head(operand, pending, TypeContext::PostfixOperand, writer)?;
        let operand = match operand {
            PreparedTypeContent::Type(node) => PostfixOperand::Type(node),
            content => PostfixOperand::Prepared(Box::new(content)),
        };
        Ok(PreparedPostfixType {
            data,
            operand,
            suffix,
        })
    }

    pub(super) fn format_prepared_postfix_type(
        &mut self,
        postfix: PreparedPostfixType<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_type_in_context(
            postfix.data,
            context,
            layout,
            writer,
            |formatter, _, writer| {
                let tail = match postfix.operand {
                    PostfixOperand::Type(node) => {
                        formatter.format_prepared_type(node, TypeContext::PostfixOperand, writer)?
                    }
                    PostfixOperand::Prepared(content) => formatter.format_prepared_type_content(
                        *content,
                        TypeContext::PostfixOperand,
                        TypeLayout::Root,
                        writer,
                    )?,
                };
                formatter.format_expression_tail(tail, writer)?;
                match postfix.suffix {
                    PostfixSuffix::Array(suffix) => {
                        formatter.format_array_suffix(suffix, writer)?
                    }
                    PostfixSuffix::Indexed {
                        index,
                        prefix,
                        suffix,
                    } => {
                        formatter.format_indexed_suffix(index, prefix, suffix, writer)?;
                    }
                }
                Ok(None)
            },
        )
    }

    pub(super) fn format_type_operator(
        &mut self,
        node: TsTypeOperator,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let keyword = match node.operator(ast) {
            TsTypeOperatorKind::Keyof => "keyof",
            TsTypeOperatorKind::Unique => "unique",
            TsTypeOperatorKind::Readonly => "readonly",
        };
        let operand = node.type_annotation(ast);
        let mut operand_type = self.prepare_type(operand, ContentMode::Formatted);
        let gap = Span::new(
            node.span(ast).start + keyword.len() as u32,
            operand.span(ast).start,
        );
        let prefix = self.prepare_type_prefix(gap);
        if prefix.mode == ContentMode::Verbatim {
            self.select_type_head_comments(&mut operand_type, self.comments.in_range(gap), gap.end);
        }
        writer.text(keyword);
        self.format_type_prefix(prefix, FlatSeparator::Space, writer)?;
        self.format_prepared_type(operand_type, TypeContext::ModifierOperand, writer)
    }

    pub(super) fn format_type_query(
        &mut self,
        node: TsTypeQuery,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let name = node.expr_name(ast);
        let name_data = ast.ts_type_query_expr_name_data(name);
        let gap = Span::new(
            node.span(ast).start + "typeof".len() as u32,
            name.span(ast).start,
        );
        let prefix = self.prepare_type_prefix(gap);
        if node.type_arguments(ast).is_some() {
            return Err(self.unsupported("generic type query", node));
        }
        let mode = prefix.mode;
        writer.text("typeof");
        self.format_type_prefix(prefix, FlatSeparator::Space, writer)?;
        if mode == ContentMode::Verbatim {
            self.format_verbatim(name.span(ast), writer)?;
        } else {
            match name_data {
                TsTypeQueryExprNameData::TsTypeName(name) => {
                    self.format_type_name(name, mode, writer)?
                }
                TsTypeQueryExprNameData::ThisExpression(_) => writer.text("this"),
                TsTypeQueryExprNameData::TsImportType(import) => {
                    return Err(self.unsupported("import type query", import));
                }
            }
        }
        Ok(None)
    }

    pub(super) fn prepare_type_prefix(&self, gap: Span) -> TypePrefix<'ast> {
        let comments = self.comments.in_range(gap);
        let mode = self.content_mode(comments);
        let layout = if mode == ContentMode::Verbatim {
            TypePrefixLayout::Source(gap)
        } else {
            TypePrefixLayout::Formatted(self.prepare_comment_boundary(comments, gap.start, gap.end))
        };
        TypePrefix { mode, layout }
    }

    pub(super) fn format_type_prefix(
        &mut self,
        prefix: TypePrefix<'ast>,
        separator: FlatSeparator,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match prefix.layout {
            TypePrefixLayout::Source(gap) => self.format_verbatim(gap, writer),
            TypePrefixLayout::Formatted(boundary) => {
                self.format_type_comment_gap(&boundary, separator, writer)
            }
        }
    }

    pub(super) fn format_type_comment_gap(
        &mut self,
        boundary: &CommentBoundary<'ast>,
        separator: FlatSeparator,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_comment_boundary(boundary, writer)?;
        boundary.gap().write(writer, separator, |writer| {
            if matches!(separator, FlatSeparator::Space) {
                writer.space();
            }
        });
        Ok(())
    }
}
