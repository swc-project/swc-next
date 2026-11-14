//! Formatting context and the program entry point.

mod bindings;
mod classes;
mod comments;
mod decorators;
mod diagnostics;
mod expressions;
mod functions;
mod heritage;
mod literals;
mod parameters;
mod restricted;
mod source;
mod statements;
mod suppression;
mod templates;
mod types;
mod variables;

use std::cell::Cell;

use swc_next_ecma_ast::{Ast, AwaitExpression, GetSpan, NodeId, Program, Span, StmtData};

use self::comments::CommentCursor;
use self::statements::{StatementItem, StatementSequenceKind};
use crate::{FormatError, FormatOptions, document::Writer};

const MAX_RECURSION_DEPTH: usize = if cfg!(debug_assertions) { 32 } else { 128 };

pub(crate) struct Formatter<'ast, 'source> {
    ast: &'ast Ast<'source>,
    allocator: &'ast swc_next_allocator::Allocator,
    options: FormatOptions,
    comments: CommentCursor<'ast>,
    diagnostic_lines: diagnostics::DiagnosticLines,
    external_opening_owners: Vec<Option<expressions::parentheses::PreparedSourceOpening>>,
    has_suppression: bool,
    has_target_annotations: bool,
    has_type_casts: bool,
    has_jsdoc_enclosures: std::cell::OnceCell<bool>,
    has_file_directives: bool,
    enclosing_await: Option<AwaitExpression>,
    for_initializer_span: Option<Span>,
    label_chain_head: Option<Span>,
    terminal_close_owner: diagnostics::TerminalCloseOwner,
    source_object_layout: SourceObjectLayout,
    recursion: &'ast RecursionState,
}

#[derive(Clone, Copy)]
struct RecursionFrame {
    depth: usize,
    node: Option<NodeId>,
}

/// Document-arena ownership lets scopes restore state without borrowing the formatter.
struct RecursionState {
    current: Cell<RecursionFrame>,
}

struct RecursionScope<'state> {
    state: &'state RecursionState,
    previous: RecursionFrame,
}

struct RecursionLimit;

impl RecursionState {
    fn enter(&self, node: Option<NodeId>) -> Result<Option<RecursionScope<'_>>, RecursionLimit> {
        let previous = self.current.get();
        // Preparing and emitting the same node share one active scope.
        if node.is_some() && node == previous.node {
            return Ok(None);
        }
        if previous.depth == MAX_RECURSION_DEPTH {
            return Err(RecursionLimit);
        }
        self.current.set(RecursionFrame {
            depth: previous.depth + 1,
            node,
        });
        Ok(Some(RecursionScope {
            state: self,
            previous,
        }))
    }

    /// A physical frame recurses even when its body keeps the active node.
    fn enter_frame(&self) -> Result<RecursionScope<'_>, RecursionLimit> {
        let previous = self.current.get();
        if previous.depth == MAX_RECURSION_DEPTH {
            return Err(RecursionLimit);
        }
        self.current.set(RecursionFrame {
            depth: previous.depth + 1,
            ..previous
        });
        Ok(RecursionScope {
            state: self,
            previous,
        })
    }
}

impl Drop for RecursionScope<'_> {
    fn drop(&mut self) {
        self.state.current.set(self.previous);
    }
}

/// Language construction facts for stable chain selection, never Doc fitting.
#[derive(Default)]
struct SourceObjectLayout {
    observing: bool,
    generation: usize,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(crate) fn new(
        ast: &'ast Ast<'source>,
        options: FormatOptions,
        allocator: &'ast swc_next_allocator::Allocator,
    ) -> Self {
        let mut formatter = Self {
            ast,
            allocator,
            options,
            comments: CommentCursor::new(ast.comments()),
            diagnostic_lines: diagnostics::DiagnosticLines::new(ast.source(), ast.comments()),
            external_opening_owners: Vec::new(),
            has_suppression: ast
                .comments()
                .iter()
                .any(|&comment| suppression::is_suppression_comment(ast, comment)),
            has_target_annotations: false,
            has_type_casts: false,
            has_jsdoc_enclosures: std::cell::OnceCell::new(),
            has_file_directives: false,
            enclosing_await: None,
            for_initializer_span: None,
            label_chain_head: None,
            terminal_close_owner: diagnostics::TerminalCloseOwner::Source,
            source_object_layout: SourceObjectLayout::default(),
            recursion: allocator.alloc(RecursionState {
                current: Cell::new(RecursionFrame {
                    depth: 0,
                    node: None,
                }),
            }),
        };
        for &comment in ast.comments() {
            if !formatter.has_type_casts {
                formatter.has_type_casts = formatter.comment_is_type_cast(comment);
            }
            if !formatter.has_target_annotations {
                formatter.has_target_annotations =
                    formatter.has_type_casts || formatter.comment_is_optimizer_annotation(comment);
            }
            if !formatter.has_file_directives {
                formatter.has_file_directives = formatter.comment_is_file_directive(comment);
            }
            if formatter.has_type_casts && formatter.has_file_directives {
                break;
            }
        }
        formatter.external_opening_owners = formatter.prepare_external_opening_owners();
        formatter
    }

    fn program_header_barrier(&self, program: Program) -> Option<Span> {
        let ast = self.ast;
        if !self.has_file_directives || !program.directives(ast).is_empty() {
            return None;
        }
        let mut statements = program.body(ast).iter();
        let first = ast.get_node_in_sub_range(statements.next()?);
        let StmtData::EmptyStatement(empty) = ast.stmt_data(first) else {
            return None;
        };
        let barrier = empty.span(ast);
        let end = statements
            .map(|id| ast.get_node_in_sub_range(id))
            .find(|&statement| !matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_)))
            .map_or(ast.source().len() as u32, |statement| {
                statement.span(ast).start
            });
        let comments = self
            .comments
            .in_range(Span::new(program.span(ast).start, end));
        let before = comments.partition_point(|comment| comment.span.end <= barrier.start);
        let original = comments[..before]
            .iter()
            .rev()
            .find_map(|&comment| self.comment_checking_mode(comment))?;
        let formatted = comments[before..]
            .iter()
            .rev()
            .find_map(|&comment| self.comment_checking_mode(comment))
            .unwrap_or(original);
        // Newly active annotations may not override an existing header binding.
        (formatted != original).then_some(barrier)
    }

    pub(crate) fn format_program(mut self, writer: &mut Writer<'ast>) -> Result<(), FormatError> {
        let program = self.ast.root_program();
        // With no comment-owning child, an ignore directive belongs to the program.
        if self.has_suppression
            && program.directives(self.ast).is_empty()
            && program.body(self.ast).iter().all(|id| {
                matches!(
                    self.ast.stmt_data(self.ast.get_node_in_sub_range(id)),
                    StmtData::EmptyStatement(_)
                )
            })
        {
            let start = if self.ast.source().starts_with('\u{feff}') {
                3
            } else {
                0
            };
            return self.format_verbatim(Span::new(start, self.ast.source().len() as u32), writer);
        }
        let hashbang = program.hashbang(self.ast);
        if let Some(hashbang) = hashbang {
            writer.text("#!");
            writer.text(self.ast.get_utf8(hashbang.value(self.ast)));
            // The complete payload, including trailing whitespace, is significant.
            writer.literal_line();
        }
        let ast = self.ast;
        let statements = program
            .body(ast)
            .iter()
            .map(|id| StatementItem::Statement(ast.get_node_in_sub_range(id)));
        let directives = program
            .directives(ast)
            .iter()
            .map(|id| StatementItem::Directive(ast.get_node_in_sub_range(id)));
        let has_content = self.format_statement_sequence(
            directives.chain(statements).peekable(),
            program.span(ast),
            StatementSequenceKind::Program {
                hashbang_end: hashbang.map(|node| node.span(ast).end),
                header_barrier: self.program_header_barrier(program),
                has_source_items: !program.body(ast).is_empty()
                    || !program.directives(ast).is_empty(),
            },
            writer,
        )?;
        if let Some(comment) = self.comments.first() {
            return Err(FormatError::unsupported(
                "comment placement",
                None,
                comment.span,
            ));
        }
        if has_content.has_content {
            writer.hard_line();
        }
        Ok(())
    }

    fn unsupported<N: swc_next_ecma_ast::AstNode + GetSpan>(
        &self,
        construct: &'static str,
        node: N,
    ) -> FormatError {
        FormatError::unsupported(
            construct,
            Some(self.ast.node_kind(node.node_id())),
            node.span(self.ast),
        )
    }

    fn with_recursion<N: swc_next_ecma_ast::AstNode + GetSpan, R>(
        &mut self,
        node: N,
        format: impl FnOnce(&mut Self) -> Result<R, FormatError>,
    ) -> Result<R, FormatError> {
        let _scope = self.recursion_scope(node)?;
        format(self)
    }

    #[inline(always)]
    fn recursion_scope<N: swc_next_ecma_ast::AstNode + GetSpan>(
        &self,
        node: N,
    ) -> Result<Option<RecursionScope<'ast>>, FormatError> {
        let recursion = self.recursion;
        recursion
            .enter(Some(node.node_id()))
            .map_err(|_| self.recursion_limit_error(node))
    }

    #[cold]
    #[inline(never)]
    fn recursion_limit_error<N: swc_next_ecma_ast::AstNode + GetSpan>(
        &self,
        node: N,
    ) -> FormatError {
        self.unsupported("nesting depth", node)
    }

    fn generated_frame_scope<N: swc_next_ecma_ast::AstNode + GetSpan>(
        &self,
        owner: N,
    ) -> Result<RecursionScope<'ast>, FormatError> {
        self.recursion
            .enter_frame()
            .map_err(|_| self.unsupported("nesting depth", owner))
    }

    fn with_recursion_scope<R>(
        &mut self,
        format: impl FnOnce(&mut Self) -> Result<R, FormatError>,
    ) -> Result<R, FormatError> {
        let _scope = self.anonymous_recursion_scope()?;
        format(self)
    }

    fn anonymous_recursion_scope(&self) -> Result<Option<RecursionScope<'ast>>, FormatError> {
        let recursion = self.recursion;
        recursion.enter(None).map_err(|_| {
            FormatError::unsupported(
                "nesting depth",
                None,
                self.ast.root_program().span(self.ast),
            )
        })
    }
}
