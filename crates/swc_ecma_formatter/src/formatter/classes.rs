//! Class heads and members retain real keys, signatures, and source terminators.

use swc_next_ecma_ast::{
    Accessibility, BindingIdentifier, Class, ClassBody, ClassElement, ClassElementData, Comment,
    CommentKind, Expr, GetSpan, MethodDefinitionKind, PropertyDefinition, PropertyKey, Span,
    StaticBlock, StmtData, SubRange, TsClassImplements, TsFunction, TsIndexSignature,
    TsMethodDefinition, TsTypeAnnotation, TsTypeParameterDeclaration,
};

pub(in crate::formatter) mod member_prefix;

use super::Formatter;
use super::bindings::InitializerLayout;
use super::comments::{
    CommentBinding, CommentBoundary, CommentGap, CommentRunLayout, ExpressionTail, NodeHead,
};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::objects::{PropertyKeyContext, PropertyKeyQuoting};
use super::expressions::{
    AccessKind, ExpressionContext, ExpressionPosition, ExpressionStart, ExpressionTypeArguments,
    TailOwner,
};
use super::functions::bodyless::PreparedBodylessSignature;
use super::functions::{MethodOwner, SignaturePreceding, SourceMethodHeader, SourceMethodPrefix};
use super::heritage::{HeritageItem, HeritageLayout, HeritageOwners, HeritagePrefix};
use super::statements::IncomingStatement;
use super::suppression::{
    ContentMode, DelimitedFollowing, DelimitedSuppression, SuppressionPosition,
    is_suppression_comment,
};
use super::types::generics::{TypeListKind, TypeListPrefix};
use super::types::members::PreparedIndexSignature;
use super::types::{PreparedAnnotation, TypeGap};
use crate::{FormatError, QuoteProperties, Semicolons, document::Writer};
use member_prefix::ClassMemberPrefix;

#[expect(
    clippy::large_enum_variant,
    reason = "Keep the prepared borrowed header inline instead of allocating per class."
)]
enum PreparedClassHead<'ast> {
    Verbatim,
    Source(SourceClassHead<'ast>),
}

struct PreparedClass<'ast> {
    head: PreparedClassHead<'ast>,
    span: Span,
}

struct SourceClassHead<'ast> {
    prefix: Option<SourceClassPrefix<'ast>>,
    keyword_end: u32,
    name: Option<BindingIdentifier>,
    name_mode: ContentMode,
    parameters: Option<SourceClassParameters<'ast>>,
    heritage: Option<SourceClassHeritage<'ast>>,
    implements: Option<SourceClassImplements<'ast>>,
    layout: HeritageLayout,
    body: ClassBody,
    body_mode: ContentMode,
    incoming: Option<IncomingClassMember>,
    end: u32,
    body_comments: &'ast [Comment],
    body_leading: Option<&'ast [Comment]>,
    body_gap: CommentGap,
}

enum SourceClassPrefix<'ast> {
    Formatted {
        declare: bool,
        abstract_class: bool,
        name_gap: Option<CommentBoundary<'ast>>,
    },
    Source(Span),
}

struct SourceClassParameters<'ast> {
    node: TsTypeParameterDeclaration,
    prefix: TypeListPrefix<'ast>,
    mode: ContentMode,
}

#[derive(Clone, Copy)]
struct IncomingClassMember {
    node: ClassElement,
    mode: ContentMode,
}

#[derive(Clone, Copy)]
enum ClassCaller {
    Ordinary,
    Assignment,
}

struct SourceClassHeritage<'ast> {
    caller: ClassCaller,
    expression: Expr,
    mode: ContentMode,
    prefix: HeritagePrefix<'ast>,
    arguments: Option<ExpressionTypeArguments<'ast>>,
}

struct SourceClassImplements<'ast> {
    first: TsClassImplements,
    remaining: SubRange<TsClassImplements>,
    first_mode: ContentMode,
    end: u32,
    prefix: HeritagePrefix<'ast>,
}

struct PreparedClassMember<'ast> {
    node: ClassElement,
    span: Span,
    content: ClassMemberContent<'ast>,
    sibling: Option<DelimitedSuppression>,
}

enum ClassMemberContent<'ast> {
    Verbatim,
    Method(SourceMethodHeader<'ast>),
    Bodyless(SourceBodylessClassMethod<'ast>),
    BodylessValue {
        prefix: SourceMethodPrefix,
        value: TsFunction,
    },
    Field(SourceClassField<'ast>),
    Static(SourceStaticBlock<'ast>),
    Index(SourceClassIndexSignature<'ast>),
}

struct SourceBodylessClassMethod<'ast> {
    prefix: SourceMethodPrefix,
    signature: PreparedBodylessSignature<'ast>,
    terminal: TypedClassMemberTerminal,
}

struct SourceClassField<'ast> {
    key: PropertyKey,
    computed: Option<Span>,
    key_start: u32,
    key_mode: ContentMode,
    prefix: ClassMemberPrefix,
    key_context: PropertyKeyContext,
    quoting: PropertyKeyQuoting,
    suffix: Option<FieldSuffix<'ast>>,
    initializer: Option<FieldInitializer<'ast>>,
    terminal: FieldTerminal,
}

struct FieldInitializer<'ast> {
    right: Expr,
    layout: InitializerLayout<'ast>,
}

struct SourceClassIndexSignature<'ast> {
    signature: PreparedIndexSignature<'ast>,
    terminal: TypedClassMemberTerminal,
}

enum TypedClassMemberTerminal {
    Canonical(Span),
    Source(Span),
}

struct SourceFieldSuffix {
    key_end: u32,
    marker: Option<(Span, &'static str)>,
    annotation: Option<TsTypeAnnotation>,
    end: u32,
}

struct FieldSuffix<'ast> {
    key_end: u32,
    marker: Option<(Span, &'static str)>,
    annotation: Option<PreparedAnnotation<'ast>>,
}

enum FieldTerminal {
    Canonical { span: Span, semicolon: bool },
    Source(Span),
}

struct ClassMemberGap<'ast> {
    trailing: CommentBoundary<'ast>,
    leading: Option<NodeHead<'ast>>,
    blank_before: bool,
}

struct SourceStaticBlock<'ast> {
    node: StaticBlock,
    braces: Span,
    incoming: Option<IncomingStatement>,
    whole: bool,
    comments: &'ast [Comment],
    gap: CommentGap,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_class(
        &mut self,
        class: Class,
        position: ExpressionPosition,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        if !class.decorators(ast).is_empty() {
            return Err(self.unsupported("typed or decorated class", class));
        }
        let span = class.span(ast);
        let caller = if matches!(position, ExpressionPosition::AssignmentRight(_)) {
            ClassCaller::Assignment
        } else {
            ClassCaller::Ordinary
        };
        let prepared = PreparedClass {
            head: self.source_class_head(class, span.start, caller)?,
            span,
        };
        self.format_prepared_class(prepared, writer)
    }
    fn format_prepared_class(
        &mut self,
        prepared: PreparedClass<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let head = match prepared.head {
            PreparedClassHead::Verbatim => {
                return self.format_verbatim(prepared.span, writer);
            }
            PreparedClassHead::Source(head) => head,
        };
        let SourceClassHead {
            prefix,
            keyword_end,
            name,
            name_mode,
            parameters,
            heritage,
            implements,
            layout,
            body,
            body_mode,
            incoming,
            end,
            body_comments,
            body_leading,
            body_gap,
        } = head;
        let write_header = |writer: &mut Writer<'ast>| {
            let name_gap_formatted = match prefix {
                Some(SourceClassPrefix::Formatted {
                    declare,
                    abstract_class,
                    name_gap,
                }) => {
                    if declare {
                        writer.text("declare");
                        writer.space();
                    }
                    if abstract_class {
                        writer.text("abstract");
                        writer.space();
                    }
                    writer.text("class");
                    let has_name = name_gap.is_some();
                    if let Some(boundary) = name_gap {
                        self.format_type_gap(
                            TypeGap::Formatted(boundary),
                            FlatSeparator::Space,
                            writer,
                        )?;
                    }
                    has_name
                }
                Some(SourceClassPrefix::Source(span)) => {
                    self.format_verbatim(span, writer)?;
                    false
                }
                None => {
                    writer.text("class");
                    false
                }
            };
            if let Some(name) = name {
                if !name_gap_formatted {
                    self.format_source_token_gap(
                        keyword_end,
                        name.span(self.ast).start,
                        true,
                        writer,
                    )?;
                }
                if name_mode == ContentMode::Verbatim {
                    self.format_verbatim(name.span(self.ast), writer)?;
                } else {
                    writer.text(self.ast.get_utf8(name.name(self.ast)));
                }
            }
            if let Some(parameters) = parameters {
                self.format_declaration_type_parameters(
                    parameters.node,
                    parameters.prefix,
                    parameters.mode,
                    writer,
                )?;
            }
            let write_heritage = |writer: &mut Writer<'ast>| {
                if let Some(heritage) = heritage {
                    self.format_class_heritage(
                        heritage,
                        if layout.grouped() {
                            HeritageLayout::Single
                        } else {
                            HeritageLayout::Inline
                        },
                        writer,
                    )?;
                }
                if let Some(implements) = implements {
                    self.format_class_implements(implements, layout, writer)?;
                }
                Ok(())
            };
            if layout.grouped() {
                writer.indent(write_heritage)?;
            } else {
                write_heritage(writer)?;
            }
            self.comments.claim(body_comments)?;
            self.format_comment_run(body_comments, CommentRunLayout::SourceAfter(end), writer)?;
            body_gap.write(writer, FlatSeparator::Space, |writer| {
                if layout.grouped() && body_comments.is_empty() && !body.body(self.ast).is_empty() {
                    writer.soft_line();
                    writer.if_flat(" ");
                } else {
                    writer.space();
                }
            });
            Ok(())
        };
        if layout.grouped() {
            writer.group(false, write_header)?;
        } else {
            write_header(writer)?;
        }
        self.format_class_body(body, body_mode, incoming, body_leading, writer)
    }

    fn source_class_head(
        &self,
        class: Class,
        core_start: u32,
        caller: ClassCaller,
    ) -> Result<PreparedClassHead<'ast>, FormatError> {
        let ast = self.ast;
        let body = class.body(ast);
        let span = class.span(ast);
        let name = class.id(ast);
        let (start, prefix) = if class.r#abstract(ast) || class.declare(ast) {
            let (end, prefix) = self.prepare_class_prefix(class, core_start)?;
            (end, Some(prefix))
        } else {
            (core_start + 5, None)
        };
        let mut name_mode = ContentMode::Formatted;
        if self.has_suppression {
            let prefix_start = if prefix.is_some() { core_start } else { start };
            let prefix_end = name.map_or(start, |name| name.span(ast).start);
            let comments = self.comments.in_range(Span::new(prefix_start, prefix_end));
            name_mode = self.content_mode(comments);
            if name.is_none() && name_mode == ContentMode::Verbatim {
                return Ok(PreparedClassHead::Verbatim);
            }
        }
        let superclass = class.super_class(ast);
        let mut implements_nodes = class.implements(ast);
        let first_implements: Option<TsClassImplements> = implements_nodes
            .first()
            .map(|id| ast.get_node_in_sub_range(id));
        let continuation_start = superclass
            .map(|node| node.span(ast).start)
            .or_else(|| first_implements.map(|node| node.span(ast).start));
        let mut end = name.map_or(start, |name| name.span(ast).end);
        let mut parameters = class.type_parameters(ast).map(|node| {
            let gap = Span::new(end, node.span(ast).start);
            let prefix =
                if let Some(following_start) = continuation_start.filter(|_| name.is_some()) {
                    self.prepare_declaration_type_list_prefix(gap, node, following_start)
                } else {
                    self.prepare_type_list_prefix(
                        gap,
                        TypeListKind::Parameters {
                            has_left_owner: name.is_some(),
                        },
                    )
                };
            if prefix.left_mode == ContentMode::Verbatim {
                name_mode = ContentMode::Verbatim;
            }
            end = node.span(ast).end;
            SourceClassParameters {
                node,
                mode: prefix.right_mode,
                prefix,
            }
        });
        let mut grouped = parameters
            .as_ref()
            .is_some_and(|parameters| parameters.prefix.has_trailing_name_comment());
        let mut heritage = if let Some(expression) = superclass {
            let gap = Span::new(end, expression.span(ast).start);
            let comments = self.comments.in_range(gap);
            let extends = self
                .find_token(gap, comments, "extends")
                .ok_or_else(|| self.unsupported("class extends boundary", class))?;
            let prepared = self.prepare_heritage_prefix(
                gap,
                Span::new(extends, extends + 7),
                comments,
                || {
                    let preceding = parameters
                        .as_ref()
                        .map(|parameters| parameters.node.span(ast))
                        .or_else(|| name.map(|name| name.span(ast)));
                    HeritageOwners {
                        inline_before: preceding.unwrap_or_else(|| expression.span(ast)),
                        line_preceding: preceding.unwrap_or_else(|| expression.span(ast)),
                    }
                },
                expression.span(ast),
            );
            match prepared.preceding {
                CommentBinding::Unbound => {}
                CommentBinding::SourceBound => return Ok(PreparedClassHead::Verbatim),
                CommentBinding::BoundTo(_) => {
                    if let Some(parameters) = &mut parameters {
                        parameters.mode = ContentMode::Verbatim;
                    } else if name.is_some() {
                        name_mode = ContentMode::Verbatim;
                    } else {
                        return Ok(PreparedClassHead::Verbatim);
                    }
                }
            }
            let mut mode = prepared.first_mode;
            end = expression.span(ast).end;
            let arguments = if let Some(node) = class.super_type_arguments(ast) {
                let arguments =
                    self.prepare_expression_type_arguments(expression, node, AccessKind::Ordinary)?;
                if arguments.callee_mode == ContentMode::Verbatim {
                    mode = ContentMode::Verbatim;
                }
                grouped |= arguments.has_comments();
                end = node.span(ast).end;
                Some(arguments)
            } else {
                None
            };
            grouped |= prepared.prefix.has_comments
                || !self.comments.in_range(expression.span(ast)).is_empty()
                || matches!(caller, ClassCaller::Ordinary)
                    && arguments.is_none()
                    && self.heritage_expression_is_member(expression);
            Some(SourceClassHeritage {
                caller,
                expression,
                mode,
                prefix: prepared.prefix,
                arguments,
            })
        } else {
            None
        };
        let multiple = usize::from(heritage.is_some()) + implements_nodes.len() > 1;
        let implements = if let Some(first) = first_implements {
            let remaining = implements_nodes.split_off(1);
            let gap = Span::new(end, first.span(ast).start);
            let comments = self.comments.in_range(gap);
            let keyword = self
                .find_token(gap, comments, "implements")
                .ok_or_else(|| self.unsupported("class implements boundary", class))?;
            let prepared = self.prepare_heritage_prefix(
                gap,
                Span::new(keyword, keyword + 10),
                comments,
                || {
                    let preceding_before = heritage
                        .as_ref()
                        .map(|heritage| {
                            heritage
                                .arguments
                                .as_ref()
                                .map_or(heritage.expression.span(ast), |arguments| {
                                    arguments.node().span(ast)
                                })
                        })
                        .or_else(|| {
                            parameters
                                .as_ref()
                                .map(|parameters| parameters.node.span(ast))
                        })
                        .or_else(|| name.map(|name| name.span(ast)));
                    let preceding_after = heritage.as_ref().map_or_else(
                        || {
                            parameters
                                .as_ref()
                                .map(|parameters| parameters.node.span(ast))
                                .or_else(|| name.map(|name| name.span(ast)))
                        },
                        |heritage| {
                            heritage
                                .arguments
                                .is_none()
                                .then(|| heritage.expression.span(ast))
                        },
                    );
                    HeritageOwners {
                        inline_before: preceding_before.unwrap_or(span),
                        line_preceding: preceding_after.unwrap_or(span),
                    }
                },
                first.span(ast),
            );
            match prepared.preceding {
                CommentBinding::Unbound => {}
                CommentBinding::SourceBound => return Ok(PreparedClassHead::Verbatim),
                CommentBinding::BoundTo(target) if target == span => {
                    return Ok(PreparedClassHead::Verbatim);
                }
                CommentBinding::BoundTo(target) => {
                    if let Some(heritage) = &mut heritage {
                        if let Some(arguments) = &mut heritage.arguments {
                            debug_assert_eq!(
                                target,
                                arguments.node().span(ast),
                                "a before-keyword ignore selects the actual superclass angle owner"
                            );
                            arguments.suppress();
                        } else {
                            heritage.mode = ContentMode::Verbatim;
                        }
                    } else if let Some(parameters) = &mut parameters {
                        parameters.mode = ContentMode::Verbatim;
                    } else {
                        name_mode = ContentMode::Verbatim;
                    }
                }
            }
            grouped |= prepared.prefix.has_comments
                || first.type_arguments(ast).is_none()
                    && self.heritage_expression_is_member(first.expression(ast));
            end = remaining.last().map_or(first.span(ast).end, |last| {
                ast.get_node_in_sub_range(last).span(ast).end
            });
            let mut prefix = prepared.prefix;
            // Superclass angle arguments have their own continuation owner;
            // other heads retain leading Line packets before the next keyword.
            if prefix.has_comments
                && !heritage
                    .as_ref()
                    .is_some_and(|heritage| heritage.arguments.is_some())
            {
                prefix = prefix.with_preceding_owner();
            }
            Some(SourceClassImplements {
                first,
                remaining,
                first_mode: prepared.first_mode,
                end,
                prefix,
            })
        } else {
            None
        };
        let layout = if multiple {
            HeritageLayout::Multiple
        } else if grouped {
            HeritageLayout::Single
        } else {
            HeritageLayout::Inline
        };
        let mut body_mode = ContentMode::Formatted;
        let mut incoming = None;
        let body_span = body.span(ast);
        let body_gap = Span::new(end, body_span.start);
        let mut body_comments = self.comments.in_range(body_gap);
        let last = body_comments.last().map_or(end, |comment| comment.span.end);
        let mut body_constraint = self.line_constraint(last, body.span(ast).start);
        if self.content_mode(body_comments) == ContentMode::Verbatim {
            let comments = body_comments;
            let first = body
                .body(ast)
                .iter()
                .next()
                .map(|id| ast.get_node_in_sub_range(id));
            for position in self.suppression_positions(comments, body.span(ast).start) {
                match position {
                    SuppressionPosition::Inline => body_mode = ContentMode::Verbatim,
                    _ if first.is_some() => {
                        incoming = first.map(|node| IncomingClassMember {
                            node,
                            mode: ContentMode::Verbatim,
                        })
                    }
                    _ => body_mode = ContentMode::Verbatim,
                }
            }
            if body_mode == ContentMode::Verbatim || incoming.is_some() {
                let last = comments.last().map_or(end, |comment| comment.span.end);
                body_constraint = self.source_line_constraint(last, body.span(ast).start);
            }
        }
        let mut body_leading = None;
        if self.declaration_body_comments_can_move(body_comments, body_gap) {
            let first = self.class_body_first_member(body);
            let leading_end = first.map_or(body_span.end - 1, |node| node.span(ast).start);
            let leading = self
                .comments
                .in_range(Span::new(body_comments[0].span.start, leading_end));
            if first.is_some() || self.content_mode(leading) == ContentMode::Formatted {
                body_leading = Some(leading);
                body_comments = &[];
                body_constraint = self.line_constraint(end, body_span.start);
            }
        }
        Ok(PreparedClassHead::Source(SourceClassHead {
            prefix,
            keyword_end: start,
            name,
            name_mode,
            parameters,
            heritage,
            implements,
            layout,
            body,
            body_mode,
            incoming,
            end,
            body_comments,
            body_leading,
            body_gap: if body_comments
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
            {
                CommentGap::LineComment
            } else {
                CommentGap::Source(body_constraint)
            },
        }))
    }

    pub(super) fn declaration_body_comments_can_move(
        &self,
        comments: &[Comment],
        gap: Span,
    ) -> bool {
        let Some(last) = comments.last() else {
            return false;
        };
        if self.content_mode(comments) == ContentMode::Verbatim
            || self.diagnostic_range_is_sensitive(gap)
        {
            return false;
        }
        // This boundary has no expression consumer, and file pragmas keep
        // their file owner. Diagnostic line coverage remains source-bound.
        let binding = self.comment_run_binding(comments.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.diagnostic {
                CommentBinding::SourceBound
            } else {
                CommentBinding::Unbound
            }
        });
        if !binding.allows_relocation(None) {
            return false;
        }
        comments.iter().any(|comment| {
            comment.kind == CommentKind::Line
                || self.source_position_starts_line(comment.span.start)
        }) || self.line_breaks(last.span.end, gap.end) != 0
    }

    fn format_class_heritage(
        &mut self,
        heritage: SourceClassHeritage<'ast>,
        layout: HeritageLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_heritage_clause(
            heritage.prefix,
            "extends",
            layout,
            writer,
            |formatter, writer| {
                let context = ExpressionContext::at(ExpressionPosition::Superclass)
                    .with_content_mode(heritage.mode);
                let context = if heritage.arguments.is_some() {
                    context.with_tail_owner(TailOwner::ParentOperation)
                } else {
                    context
                };
                let write_superclass = |formatter: &mut Self, writer: &mut Writer<'ast>| {
                    formatter.format_expression(heritage.expression, context, writer)
                };
                let tail = if matches!(heritage.caller, ClassCaller::Assignment)
                    && heritage.mode == ContentMode::Formatted
                {
                    writer.group(false, |writer| {
                        writer.if_break("(");
                        let tail = writer.indent(|writer| {
                            writer.soft_line();
                            write_superclass(formatter, writer)
                        })?;
                        writer.soft_line();
                        writer.if_break(")");
                        Ok::<_, FormatError>(tail)
                    })?
                } else {
                    write_superclass(formatter, writer)?
                };
                if let Some(arguments) = heritage.arguments {
                    formatter.format_expression_type_arguments(arguments, tail, writer)
                } else {
                    formatter.format_expression_tail(tail, writer)
                }
            },
        )
    }

    pub(in crate::formatter) fn class_body_first_member(
        &self,
        body: ClassBody,
    ) -> Option<ClassElement> {
        body.body(self.ast)
            .first()
            .map(|id| self.ast.get_node_in_sub_range(id))
    }

    fn format_class_body(
        &mut self,
        body: ClassBody,
        mode: ContentMode,
        incoming: Option<IncomingClassMember>,
        head_comments: Option<&'ast [Comment]>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = body.span(ast);
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(span, writer);
        }
        let content = Span::new(span.start + 1, span.end - 1);
        let quoting = match self.options.quote_properties {
            QuoteProperties::AsNeeded => PropertyKeyQuoting::AsNeeded,
            QuoteProperties::Preserve => PropertyKeyQuoting::Preserve,
        };
        let mut members = body
            .body(ast)
            .iter()
            .skip(1)
            .map(|id| ast.get_node_in_sub_range(id))
            .peekable();
        let Some(first) = self.class_body_first_member(body) else {
            let comments = head_comments.unwrap_or_else(|| self.comments.in_range(content));
            if self.content_mode(comments) == ContentMode::Verbatim {
                return self.format_verbatim(span, writer);
            }
            writer.text("{");
            if !comments.is_empty() {
                writer.indent(|writer| {
                    writer.hard_line();
                    self.comments.claim(comments)?;
                    self.format_comment_run(comments, CommentRunLayout::Lines, writer)
                })?;
                writer.hard_line();
            }
            writer.text("}");
            return Ok(());
        };
        let leading = head_comments.unwrap_or_else(|| {
            self.comments
                .in_range(Span::new(content.start, first.span(ast).start))
        });
        let first_mode = if incoming
            .is_some_and(|target| target.node == first && target.mode == ContentMode::Verbatim)
            || self.content_mode(leading) == ContentMode::Verbatim
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        let mut current = self.prepare_class_member(
            first,
            members.peek().copied(),
            content.end,
            first_mode,
            quoting,
        )?;
        writer.text("{");
        let end = writer.indent(|writer| {
            self.opening_line_constraint(content).write(
                writer,
                FlatSeparator::Space,
                Writer::hard_line,
            );
            self.comments.claim(leading)?;
            self.format_comment_run(
                leading,
                CommentRunLayout::Leading(first.span(ast).start),
                writer,
            )?;
            loop {
                let next = members
                    .next()
                    .map(|node| {
                        let mode = current
                            .sibling
                            .as_ref()
                            .map_or(ContentMode::Formatted, |source| source.following);
                        self.prepare_class_member(
                            node,
                            members.peek().copied(),
                            content.end,
                            mode,
                            quoting,
                        )
                    })
                    .transpose()?;
                let node_span = current.span;
                let next_span = next.as_ref().map(|next| next.span);
                let gap = Span::new(
                    node_span.end,
                    next_span.map_or(content.end, |span| span.start),
                );
                let member_gap = current
                    .sibling
                    .is_none()
                    .then(|| self.prepare_class_member_gap(gap, next_span));
                self.format_class_member(current.content, node_span, next.as_ref(), writer)?;
                let end = if let Some(member_gap) = member_gap {
                    self.format_class_member_gap(member_gap, writer)?
                } else {
                    let source = self.format_delimited_source_gap(
                        gap,
                        if next.is_some() {
                            DelimitedFollowing::Node
                        } else {
                            DelimitedFollowing::End
                        },
                        writer,
                    )?;
                    if next.is_some() {
                        source
                            .following
                            .write(writer, FlatSeparator::Space, Writer::hard_line);
                    }
                    source.end
                };
                let Some(next) = next else {
                    return Ok::<_, FormatError>(end);
                };
                current = next;
            }
        })?;
        self.line_constraint(end, content.end).write(
            writer,
            FlatSeparator::Space,
            Writer::hard_line,
        );
        writer.text("}");
        Ok(())
    }

    fn prepare_class_member_gap(&self, gap: Span, next: Option<Span>) -> ClassMemberGap<'ast> {
        let comments = self.ast.comments();
        let first = comments.partition_point(|comment| comment.span.start < gap.start);
        let comments = &comments[first..];
        let comments = &comments[..comments.partition_point(|comment| comment.span.end <= gap.end)];
        let (trailing, leading) = if next.is_some() {
            self.split_separator_comments(comments, 0, gap.start, Some(gap.end))
        } else {
            (comments, &[][..])
        };
        let leading_start = leading
            .first()
            .map_or(gap.end, |comment| comment.span.start);
        let trailing = self.prepare_comment_boundary(trailing, gap.start, leading_start);
        let blank_before = next.is_some()
            && self.line_constraint(trailing.end(), leading_start) == LineConstraint::Unconstrained
            && self.next_line_is_empty(trailing.end(), leading_start);
        let leading = next.map(|target| {
            let source_bound = !matches!(
                self.comment_binding(leading, Span::new(trailing.end(), gap.end), Some(target)),
                CommentBinding::Unbound
            );
            NodeHead {
                comments: leading,
                content_start: target.start,
                layout: if source_bound {
                    CommentRunLayout::Source
                } else {
                    CommentRunLayout::Leading(target.start)
                },
                gap_end: source_bound.then_some(target.start),
            }
        });
        ClassMemberGap {
            trailing,
            leading,
            blank_before,
        }
    }

    fn format_class_member_gap(
        &mut self,
        gap: ClassMemberGap<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        self.format_comment_boundary(&gap.trailing, writer)?;
        let Some(head) = gap.leading else {
            return Ok(gap.trailing.end());
        };
        gap.trailing
            .gap()
            .write(writer, FlatSeparator::Space, Writer::hard_line);
        if gap.blank_before {
            writer.hard_line();
        }
        let end = head
            .comments
            .last()
            .map_or(gap.trailing.end(), |comment| comment.span.end);
        self.comments.claim(head.comments)?;
        self.format_node_head(head, ExpressionStart::None, None, writer)?;
        Ok(end)
    }

    fn prepare_class_member(
        &self,
        node: ClassElement,
        following: Option<ClassElement>,
        end: u32,
        mode: ContentMode,
        quoting: PropertyKeyQuoting,
    ) -> Result<PreparedClassMember<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let sibling = self.delimited_suppression(
            span,
            following.map_or(end, |node| node.span(ast).start),
            if following.is_some() {
                DelimitedFollowing::Node
            } else {
                DelimitedFollowing::End
            },
        );
        let mode = if mode == ContentMode::Verbatim
            || sibling
                .as_ref()
                .and_then(|source| source.preceding)
                .is_some()
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        if mode == ContentMode::Verbatim {
            return Ok(PreparedClassMember {
                node,
                span,
                content: ClassMemberContent::Verbatim,
                sibling,
            });
        }
        let content = match ast.class_element_data(node) {
            ClassElementData::MethodDefinition(method) => ClassMemberContent::Method(
                self.source_method_header(MethodOwner::Class(method), quoting)?,
            ),
            ClassElementData::TsMethodDefinition(method) => {
                if !method.decorators(ast).is_empty() {
                    return Err(self.unsupported("decorated bodyless class method", method));
                }
                self.source_bodyless_class_method(method, sibling, span.start, quoting)?
            }
            ClassElementData::PropertyDefinition(field) => {
                match self.source_class_field(field, sibling, quoting)? {
                    Some(field) => ClassMemberContent::Field(field),
                    None => ClassMemberContent::Verbatim,
                }
            }
            ClassElementData::StaticBlock(block) => {
                ClassMemberContent::Static(self.source_static_block(block)?)
            }
            ClassElementData::TsIndexSignature(index) => {
                ClassMemberContent::Index(self.source_class_index_signature(index, sibling)?)
            }
        };
        Ok(PreparedClassMember {
            node,
            span,
            content,
            sibling,
        })
    }

    fn format_class_member(
        &mut self,
        content: ClassMemberContent<'ast>,
        span: Span,
        following: Option<&PreparedClassMember<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match content {
            ClassMemberContent::Verbatim => self.format_verbatim(span, writer),
            ClassMemberContent::Method(header) => self.format_method_header(header, writer),
            ClassMemberContent::Bodyless(method) => {
                self.format_bodyless_class_method(method, writer)
            }
            ClassMemberContent::BodylessValue { prefix, value } => {
                let end = prefix.end(self.ast);
                self.format_method_key_prefix(prefix, writer)?;
                self.format_source_token_gap(end, value.span(self.ast).start, false, writer)?;
                self.format_verbatim(value.span(self.ast), writer)
            }
            ClassMemberContent::Field(field) => self.format_class_field(field, following, writer),
            ClassMemberContent::Static(block) => self.format_static_block(block, writer),
            ClassMemberContent::Index(index) => {
                let tail = self.format_index_signature(index.signature, writer)?;
                self.format_typed_class_member_terminal(index.terminal, tail, writer)
            }
        }
    }

    fn format_source_class_member_terminal(
        &mut self,
        span: Span,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        // A child can leave pending trivia before this actual terminal.
        // Comments inside the terminal stay with its source punctuation.
        let before = tail.and_then(|tail| {
            let count = tail
                .comments
                .partition_point(|comment| comment.span.end <= span.start);
            (!tail.comments[..count].is_empty()).then_some(ExpressionTail {
                source_end: tail.source_end,
                comments: &tail.comments[..count],
            })
        });
        self.format_expression_tail(before, writer)?;
        self.format_verbatim(span, writer)
    }

    fn source_class_field(
        &self,
        node: PropertyDefinition,
        source_sibling: Option<DelimitedSuppression>,
        quoting: PropertyKeyQuoting,
    ) -> Result<Option<SourceClassField<'ast>>, FormatError> {
        let ast = self.ast;
        if node.declare(ast)
            || node.r#abstract(ast)
            || node.r#override(ast)
            || node.readonly(ast)
            || node.accessibility(ast) != Accessibility::None
            || node.accessor(ast)
            || !node.decorators(ast).is_empty()
        {
            return Err(self.unsupported("typed or decorated class field", node));
        }
        let span = node.span(ast);
        let key = node.key(ast);
        let key_span = key.span(ast);
        let prefix = ClassMemberPrefix::ordinary(span.start, node.r#static(ast));
        let prefix_end = prefix.end();
        let key_context = PropertyKeyContext::ClassField;
        let value = node.value(ast);
        let annotation = node.type_annotation(ast);
        let end = value.map_or_else(
            || annotation.map_or(key_span.end, |annotation| annotation.span(ast).end),
            |value| value.span(ast).end,
        );
        let internal = Span::new(end, span.end);
        let comments = self.comments.in_range(internal);
        let semicolon = self.find_token(internal, comments, ";");
        let tail_end = semicolon.unwrap_or(span.end);
        let mut key_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        let mut annotation_mode = ContentMode::Formatted;
        let mut tail_source = false;
        let computed = if node.computed(ast) {
            let before = Span::new(prefix_end, key_span.start);
            let opening = self
                .find_token(before, self.comments.in_range(before), "[")
                .ok_or_else(|| self.unsupported("computed field opening boundary", node))?;
            let after = Span::new(
                key_span.end,
                annotation.map_or_else(
                    || value.map_or(tail_end, |value| value.span(ast).start),
                    |annotation| annotation.span(ast).start,
                ),
            );
            let closing = self
                .find_token(after, self.comments.in_range(after), "]")
                .ok_or_else(|| self.unsupported("computed field closing boundary", node))?;
            Some(Span::new(opening, closing + 1))
        } else {
            None
        };
        let key_start = computed.map_or(key_span.start, |span| span.start);
        let key_end = computed.map_or(key_span.end, |span| span.end);
        let suffix = self.source_field_suffix(node, key_end, tail_end)?;
        if self.has_suppression
            && let Some((marker, _)) = suffix.as_ref().and_then(|suffix| suffix.marker)
        {
            let before = self.comments.in_range(Span::new(key_end, marker.start));
            for position in self.suppression_positions(before, marker.start) {
                match position {
                    SuppressionPosition::OwnLine if annotation.is_some() => {
                        annotation_mode = ContentMode::Verbatim;
                    }
                    SuppressionPosition::OwnLine if value.is_some() => {
                        right_mode = ContentMode::Verbatim;
                    }
                    _ => key_mode = ContentMode::Verbatim,
                }
            }
        }
        if self.has_suppression
            && suffix
                .as_ref()
                .is_some_and(|suffix| suffix.marker.is_none())
            && let Some(annotation) = annotation
        {
            let colon = annotation.span(ast).start;
            let before = self.comments.in_range(Span::new(key_end, colon));
            if self
                .suppression_positions(before, colon)
                .any(|position| matches!(position, SuppressionPosition::EndOfLine))
            {
                key_mode = ContentMode::Verbatim;
            }
        }
        let left_end = suffix.as_ref().map_or(key_end, |suffix| suffix.end);
        let end = value.map_or(left_end, |value| value.span(ast).end);
        let internal = Span::new(end, span.end);
        if self.has_suppression {
            let tail = self.comments.in_range(Span::new(end, tail_end));
            // There is no following child after the value. Blocks and Lines
            // therefore do not share the forward EOL relation used by siblings.
            for (run, placement) in self.suppressed_comment_runs(tail, tail_end) {
                for &comment in run {
                    if !is_suppression_comment(ast, comment) {
                        continue;
                    }
                    let position = if matches!(placement, SuppressionPosition::OwnLine) {
                        SuppressionPosition::OwnLine
                    } else {
                        self.suppression_position(std::slice::from_ref(&comment), tail_end)
                            .unwrap_or(placement)
                    };
                    if matches!(position, SuppressionPosition::Inline) {
                        return Ok(None);
                    }
                    if value.is_some() {
                        right_mode = ContentMode::Verbatim;
                    } else {
                        key_mode = ContentMode::Verbatim;
                    }
                    tail_source = true;
                }
            }
        }
        if self.has_suppression {
            if self.content_mode(
                self.comments
                    .in_range(Span::new(span.start, key_span.start)),
            ) == ContentMode::Verbatim
            {
                key_mode = ContentMode::Verbatim;
            }
            if let Some(computed) = computed {
                let trailing = self
                    .comments
                    .in_range(Span::new(key_span.end, computed.end - 1));
                for position in self.suppression_positions(trailing, computed.end - 1) {
                    match position {
                        SuppressionPosition::OwnLine if annotation.is_some() => {
                            annotation_mode = ContentMode::Verbatim
                        }
                        SuppressionPosition::OwnLine if value.is_some() => {
                            right_mode = ContentMode::Verbatim
                        }
                        _ => key_mode = ContentMode::Verbatim,
                    }
                }
            }
        }
        let has_suffix = suffix.is_some();
        let initializer = if let Some(right) = value {
            let right_start = right.span(ast).start;
            let gap = Span::new(left_end, right_start);
            let comments = self.comments.in_range(gap);
            let operator = self
                .find_token(gap, comments, "=")
                .ok_or_else(|| self.unsupported("class field initializer boundary", node))?;
            let mut bound_gap = None;
            if self.has_suppression {
                let split = comments.partition_point(|comment| comment.span.end <= operator);
                for position in self.suppression_positions(&comments[..split], operator) {
                    match position {
                        SuppressionPosition::OwnLine => right_mode = ContentMode::Verbatim,
                        _ if annotation.is_some() => annotation_mode = ContentMode::Verbatim,
                        _ => key_mode = ContentMode::Verbatim,
                    }
                    bound_gap = Some(gap);
                }
                for position in self.suppression_positions(&comments[split..], right_start) {
                    match position {
                        SuppressionPosition::EndOfLine => {
                            key_mode = ContentMode::Verbatim;
                            bound_gap = Some(gap);
                        }
                        _ => right_mode = ContentMode::Verbatim,
                    }
                }
            }
            let layout = if let Some(gap) = bound_gap {
                InitializerLayout::Bound { gap, right_mode }
            } else {
                let content = self.expression_content(right);
                let mut boundary = self.assignment_boundary(
                    left_end,
                    &content,
                    Span::new(operator, operator + 1),
                    right.span(ast).end,
                )?;
                if right_mode == ContentMode::Verbatim {
                    boundary.mode = right_mode;
                }
                let rhs_layout = self.ordinary_rhs_layout(
                    content.data,
                    ExpressionContext::variable_initializer()
                        .with_tail_owner(TailOwner::ParentOperation),
                    &boundary,
                );
                InitializerLayout::Formatted {
                    boundary,
                    rhs_layout,
                }
            };
            Some(FieldInitializer { right, layout })
        } else {
            None
        };
        let suffix = suffix
            .map(|suffix| self.prepare_field_suffix(suffix, annotation_mode))
            .transpose()?;
        let terminal = if tail_source || source_sibling.is_some() && semicolon.is_some() {
            FieldTerminal::Source(internal)
        } else {
            FieldTerminal::Canonical {
                span: Span::new(end, tail_end),
                semicolon: self.options.semicolons == Semicolons::Always
                    || initializer.is_none()
                        && !has_suffix
                        && computed.is_none()
                        && matches!(
                            self.property_key_identifier(key, key_context, key_mode, quoting),
                            Some("static" | "get" | "set")
                        ),
            }
        };
        Ok(Some(SourceClassField {
            key,
            computed,
            key_start,
            key_mode,
            prefix,
            key_context,
            quoting,
            suffix,
            initializer,
            terminal,
        }))
    }

    fn format_class_field(
        &mut self,
        field: SourceClassField<'ast>,
        following: Option<&PreparedClassMember<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let SourceClassField {
            key,
            computed,
            key_start,
            key_mode,
            prefix,
            key_context,
            quoting,
            suffix,
            initializer,
            terminal,
        } = field;
        self.format_class_member_prefix(&prefix, writer)?;
        self.format_source_token_gap(prefix.end(), key_start, prefix.has_modifiers(), writer)?;
        self.format_property_key_context(key, computed, key_mode, key_context, quoting, writer)?;
        let tail = if let Some(suffix) = suffix {
            self.format_field_suffix(suffix, writer)?
        } else {
            None
        };
        let tail = if let Some(FieldInitializer { right, layout }) = initializer {
            self.format_expression_tail(tail, writer)?;
            let left_end = match &layout {
                InitializerLayout::Bound { gap, .. } => gap.start,
                InitializerLayout::Formatted { boundary, .. } => boundary.left_end,
            };
            self.format_initializer_right(
                left_end,
                right,
                layout,
                ExpressionContext::variable_initializer()
                    .with_tail_owner(TailOwner::ParentOperation),
                writer,
            )?
        } else {
            tail
        };
        match terminal {
            FieldTerminal::Source(span) => {
                self.format_source_class_member_terminal(span, tail, writer)
            }
            FieldTerminal::Canonical { span, semicolon } => {
                let tail = self.capture_expression_tail(span, tail)?;
                if semicolon
                    || following
                        .is_some_and(|member| self.class_member_needs_field_separator(member))
                {
                    writer.text(";");
                }
                self.format_expression_tail(tail, writer)
            }
        }
    }

    fn class_member_needs_field_separator(&self, member: &PreparedClassMember<'ast>) -> bool {
        match &member.content {
            ClassMemberContent::Method(header) => header.needs_field_separator(self),
            ClassMemberContent::Bodyless(method) => method.prefix.needs_field_separator(self),
            ClassMemberContent::BodylessValue { prefix, .. } => prefix.needs_field_separator(self),
            ClassMemberContent::Field(field) => {
                !field.prefix.has_modifiers()
                    && (field.computed.is_some()
                        || matches!(
                            self.property_key_identifier(
                                field.key,
                                field.key_context,
                                field.key_mode,
                                field.quoting
                            ),
                            Some("in" | "instanceof")
                        ))
            }
            ClassMemberContent::Static(_) => false,
            ClassMemberContent::Index(index) => index.signature.starts_with_bracket(),
            ClassMemberContent::Verbatim => {
                let ast = self.ast;
                let key = match ast.class_element_data(member.node) {
                    ClassElementData::PropertyDefinition(field) => field.key(ast),
                    ClassElementData::MethodDefinition(method) => method.key(ast),
                    ClassElementData::TsMethodDefinition(method) => method.key(ast),
                    ClassElementData::TsIndexSignature(index) => {
                        return self.source(index.span(ast)).starts_with('[');
                    }
                    ClassElementData::StaticBlock(_) => return false,
                };
                self.source_class_member_needs_field_separator(member.node.span(ast), key)
            }
        }
    }

    fn source_static_block(
        &self,
        node: StaticBlock,
    ) -> Result<SourceStaticBlock<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let first = node
            .body(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .find(|&statement| !matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_)));
        let header = Span::new(
            span.start + 6,
            first.map_or(span.end - 1, |statement| statement.span(ast).start),
        );
        let opening = self
            .find_token(header, self.comments.in_range(header), "{")
            .ok_or_else(|| self.unsupported("static block opening boundary", node))?;
        let braces = Span::new(opening, span.end);
        let mut whole = false;
        let mut incoming = None;
        let comments = self.comments.in_range(Span::new(span.start + 6, opening));
        let last = comments
            .last()
            .map_or(span.start + 6, |comment| comment.span.end);
        let mut constraint = self.line_constraint(last, opening);
        if self.has_suppression {
            if self.content_mode(comments) == ContentMode::Verbatim {
                if let Some(first) = first {
                    incoming = Some(IncomingStatement {
                        span: first.span(ast),
                        mode: ContentMode::Verbatim,
                    });
                } else {
                    whole = true;
                }
                let last = comments
                    .last()
                    .map_or(span.start + 6, |comment| comment.span.end);
                constraint = self.source_line_constraint(last, opening);
            }
            if first.is_none()
                && self.content_mode(self.comments.in_range(Span::new(opening + 1, span.end - 1)))
                    == ContentMode::Verbatim
            {
                whole = true;
            }
        }
        Ok(SourceStaticBlock {
            node,
            braces,
            incoming,
            whole,
            comments,
            gap: if comments
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
            {
                CommentGap::LineComment
            } else {
                CommentGap::Source(constraint)
            },
        })
    }

    fn format_static_block(
        &mut self,
        block: SourceStaticBlock<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let SourceStaticBlock {
            node,
            braces,
            incoming,
            whole,
            comments,
            gap,
        } = block;
        if whole {
            return self.format_verbatim(node.span(self.ast), writer);
        }
        writer.text("static");
        let end = node.span(self.ast).start + 6;
        self.comments.claim(comments)?;
        self.format_comment_run(comments, CommentRunLayout::SourceAfter(end), writer)?;
        gap.write(writer, FlatSeparator::Space, Writer::space);
        self.format_static_block_body(node, braces, incoming, writer)
    }

    fn source_bodyless_class_method(
        &self,
        node: TsMethodDefinition,
        sibling: Option<DelimitedSuppression>,
        core_start: u32,
        quoting: PropertyKeyQuoting,
    ) -> Result<ClassMemberContent<'ast>, FormatError> {
        let ast = self.ast;
        let function = node.value(ast);
        if node.optional(ast) && node.kind(ast) != MethodDefinitionKind::Method
            || function.r#async(ast)
            || function.generator(ast)
        {
            return Err(self.unsupported("typed or decorated bodyless class method", node));
        }
        let kind = node.kind(ast);
        if kind != MethodDefinitionKind::Method {
            if function.type_parameters(ast).is_some()
                || (kind != MethodDefinitionKind::Get && function.return_type(ast).is_some())
            {
                return Err(self.unsupported("typed accessor or constructor header", function));
            }
            if function.this_param(ast).is_some() {
                return Err(self.unsupported("typed function", function));
            }
        }
        let mut source = self.source_bodyless_signature(function);
        let mut prefix = self.source_bodyless_method_prefix(
            node,
            source.header_start(ast),
            source.generic.is_some(),
            core_start,
            quoting,
        )?;
        source.set_leading_mode(prefix.following_mode);
        if self.bodyless_empty_parameters_are_verbatim(&source) {
            return Ok(ClassMemberContent::BodylessValue {
                prefix,
                value: source.node,
            });
        }
        let signature = self.prepare_bodyless_signature_suffix(
            source,
            prefix.end(ast),
            Some(SignaturePreceding::Key(prefix.key)),
            prefix.key_mode,
        )?;
        prefix.key_mode = self.method_key_mode(&prefix, signature.preceding_mode);
        let internal = Span::new(signature.content_end, node.span(ast).end);
        let (terminal, _) = self.prepare_typed_class_member_terminal(internal, sibling);
        Ok(ClassMemberContent::Bodyless(SourceBodylessClassMethod {
            prefix,
            signature,
            terminal,
        }))
    }

    fn format_bodyless_class_method(
        &mut self,
        method: SourceBodylessClassMethod<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let tail = writer.group(false, |writer| {
            self.format_method_key_prefix(method.prefix, writer)?;
            self.format_bodyless_signature_suffix(method.signature, writer)
        })?;
        self.format_typed_class_member_terminal(method.terminal, tail, writer)
    }

    fn prepare_typed_class_member_terminal(
        &self,
        internal: Span,
        sibling: Option<DelimitedSuppression>,
    ) -> (TypedClassMemberTerminal, ContentMode) {
        let comments = self.comments.in_range(internal);
        let mode = if self.has_suppression {
            self.content_mode(comments)
        } else {
            ContentMode::Formatted
        };
        let source_terminal = mode == ContentMode::Verbatim
            || sibling.is_some() && self.find_token(internal, comments, ";").is_some();
        let terminal = if source_terminal {
            TypedClassMemberTerminal::Source(internal)
        } else {
            TypedClassMemberTerminal::Canonical(internal)
        };
        (terminal, mode)
    }

    fn format_typed_class_member_terminal(
        &mut self,
        terminal: TypedClassMemberTerminal,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match terminal {
            TypedClassMemberTerminal::Canonical(span) => {
                let tail = self.capture_expression_tail(span, tail)?;
                if self.options.semicolons == Semicolons::Always {
                    writer.text(";");
                }
                self.format_expression_tail(tail, writer)
            }
            TypedClassMemberTerminal::Source(span) => {
                self.format_source_class_member_terminal(span, tail, writer)
            }
        }
    }

    fn source_class_index_signature(
        &self,
        node: TsIndexSignature,
        sibling: Option<DelimitedSuppression>,
    ) -> Result<SourceClassIndexSignature<'ast>, FormatError> {
        let mut signature = self.prepare_index_signature(node)?;
        let internal = Span::new(signature.content_end(), node.span(self.ast).end);
        let (terminal, mode) = self.prepare_typed_class_member_terminal(internal, sibling);
        if mode == ContentMode::Verbatim {
            signature.protect_result_type();
        }
        Ok(SourceClassIndexSignature {
            signature,
            terminal,
        })
    }

    fn format_class_implements(
        &mut self,
        list: SourceClassImplements<'ast>,
        layout: HeritageLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let items = std::iter::once(list.first)
            .chain(
                list.remaining
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id)),
            )
            .map(|node| HeritageItem {
                span: node.span(ast),
                expression: node.expression(ast),
                arguments: node.type_arguments(ast),
            });
        self.format_heritage_clause(
            list.prefix,
            "implements",
            layout,
            writer,
            |formatter, writer| {
                formatter.format_heritage_items(items, list.first_mode, list.end, writer)
            },
        )
    }

    fn prepare_class_prefix(
        &self,
        class: Class,
        core_start: u32,
    ) -> Result<(u32, SourceClassPrefix<'ast>), FormatError> {
        let ast = self.ast;
        let declare = class.declare(ast);
        let abstract_class = class.r#abstract(ast);
        let name = class.id(ast);
        let mut limit = class.body(ast).span(ast).start;
        if let Some(first) = class.implements(ast).first() {
            limit = ast.get_node_in_sub_range(first).span(ast).start;
        }
        if let Some(superclass) = class.super_class(ast) {
            limit = superclass.span(ast).start;
        }
        if let Some(parameters) = class.type_parameters(ast) {
            limit = parameters.span(ast).start;
        }
        if let Some(name) = name {
            limit = name.span(ast).start;
        }
        let span = Span::new(core_start, limit);
        let comments = self.comments.in_range(span);
        let keyword = self
            .find_token(span, comments, "class")
            .ok_or_else(|| self.unsupported("class modifier boundary", class))?;
        let keyword_end = keyword + "class".len() as u32;
        let target = name.map(|node| node.span(ast));
        let binding = self.comment_binding(comments, span, target);
        let prefix = if binding.allows_relocation(target) && (name.is_some() || comments.is_empty())
        {
            SourceClassPrefix::Formatted {
                declare,
                abstract_class,
                name_gap: name.map(|node| {
                    self.prepare_comment_boundary(comments, span.start, node.span(ast).start)
                }),
            }
        } else {
            SourceClassPrefix::Source(Span::new(span.start, keyword_end))
        };
        Ok((keyword_end, prefix))
    }

    fn source_field_suffix(
        &self,
        node: PropertyDefinition,
        key_end: u32,
        terminal_start: u32,
    ) -> Result<Option<SourceFieldSuffix>, FormatError> {
        let ast = self.ast;
        let annotation = node.type_annotation(ast);
        if !node.optional(ast) && !node.definite(ast) && annotation.is_none() {
            return Ok(None);
        }
        let marker = if node.optional(ast) || node.definite(ast) {
            let gap_end = annotation.map_or_else(
                || {
                    node.value(ast)
                        .map_or(terminal_start, |value| value.span(ast).start)
                },
                |annotation| annotation.span(ast).start,
            );
            let gap = Span::new(key_end, gap_end);
            let token = if node.optional(ast) { "?" } else { "!" };
            let start = self
                .find_token(gap, self.comments.in_range(gap), token)
                .ok_or_else(|| self.unsupported("class field suffix marker", node))?;
            Some((Span::new(start, start + 1), token))
        } else {
            None
        };
        let preceding_end = marker.map_or(key_end, |(span, _)| span.end);
        let end = annotation.map_or(preceding_end, |annotation| annotation.span(ast).end);
        Ok(Some(SourceFieldSuffix {
            key_end,
            marker,
            annotation,
            end,
        }))
    }

    fn prepare_field_suffix(
        &self,
        suffix: SourceFieldSuffix,
        annotation_mode: ContentMode,
    ) -> Result<FieldSuffix<'ast>, FormatError> {
        let preceding_end = suffix.marker.map_or(suffix.key_end, |(span, _)| span.end);
        let annotation = suffix
            .annotation
            .map(|annotation| {
                self.prepare_annotation_content_mode(preceding_end, annotation, annotation_mode)
            })
            .transpose()?;
        Ok(FieldSuffix {
            key_end: suffix.key_end,
            marker: suffix.marker,
            annotation,
        })
    }

    fn format_field_suffix(
        &mut self,
        suffix: FieldSuffix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if let Some((marker, token)) = suffix.marker {
            self.format_source_token_gap(suffix.key_end, marker.start, false, writer)?;
            writer.text(token);
        }
        if let Some(annotation) = suffix.annotation {
            self.format_annotation_content(annotation, writer)
        } else {
            Ok(None)
        }
    }
}
