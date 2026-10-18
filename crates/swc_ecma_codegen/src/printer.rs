use std::path::PathBuf;

use arrayvec::ArrayVec;
use memchr::memchr3;
use swc_next_allocator::{Allocator, vec::Vec as ArenaVec, wtf8::Wtf8};
use swc_next_ecma_ast::*;

use crate::operator::{BinaryExprVisitor, Binaryish, BinaryishOperator, ExprContext, OperatorCode};
use crate::util::{next_code_point, push_code_point_escape, push_u16_escape, quote_into};

use crate::{
    SourceMap,
    sourcemap::{ChunkBuilder, generate_line_offset_tables, make_chunk_builder},
};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum NumberFormat {
    #[default]
    Preserve,
    Decimal,
    Binary,
    Octal,
    Hexadecimal {
        lowercase: bool,
    },
}

#[derive(Clone)]
pub struct Options {
    pub ascii_only: bool,
    /// Escape closing script tags in strings, untagged templates, and comments,
    /// and prevent them from forming across token boundaries. Defaults to false.
    /// Tagged template raw text is preserved; this is not a guarantee that arbitrary
    /// output can be embedded in HTML without further processing.
    pub inline_script: bool,
    pub minify: bool,
    pub number_format: NumberFormat,
    pub source_map_path: Option<PathBuf>,
    pub comments: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            ascii_only: false,
            inline_script: false,
            minify: false,
            number_format: NumberFormat::Preserve,
            source_map_path: None,
            comments: true,
        }
    }
}

pub struct PrintResult<'a> {
    pub code: &'a str,
    pub legal_comments: Vec<Comment>,
    pub map: Option<SourceMap<'a>>,
}

const INFINITY_LITERAL: &str = "1e999";

fn radix_integer_parts(value: f64, bits_per_digit: u32) -> Option<(u64, usize)> {
    debug_assert!(value.is_finite());
    debug_assert!(!value.is_sign_negative());
    debug_assert!(matches!(bits_per_digit, 1 | 3 | 4));

    if value == 0.0 {
        return Some((0, 0));
    }

    const FRACTION_BITS: u32 = 52;
    const EXPONENT_BIAS: i32 = 1023;
    const FRACTION_MASK: u64 = (1_u64 << FRACTION_BITS) - 1;

    let bits = value.to_bits();
    let biased_exponent = ((bits >> FRACTION_BITS) & 0x7ff) as i32;
    if biased_exponent == 0 {
        // Every non-zero subnormal is smaller than one and therefore non-integral.
        return None;
    }

    let significand = (1_u64 << FRACTION_BITS) | (bits & FRACTION_MASK);
    let binary_shift = biased_exponent - EXPONENT_BIAS - FRACTION_BITS as i32;
    if binary_shift < 0 {
        let right_shift = (-binary_shift) as u32;
        if right_shift >= u64::BITS {
            return None;
        }
        let discarded_mask = (1_u64 << right_shift) - 1;
        if significand & discarded_mask != 0 {
            return None;
        }
        return Some((significand >> right_shift, 0));
    }

    let binary_shift = binary_shift as u32;
    let trailing_zero_digits = (binary_shift / bits_per_digit) as usize;
    let remaining_shift = binary_shift % bits_per_digit;
    Some((significand << remaining_shift, trailing_zero_digits))
}

fn parse_small_int(bytes: &[u8]) -> i32 {
    let was_negative = bytes[0] == b'-';
    let bytes = if was_negative { &bytes[1..] } else { bytes };

    // Parse without error checking. These integers are floating-point
    // exponents, so they never get large enough to overflow.
    let mut n = 0;
    for &byte in bytes {
        n = n * 10 + i32::from(byte - b'0');
    }
    if was_negative { -n } else { n }
}

struct Printer<'a> {
    allocator: &'a Allocator,
    ast: &'a Ast<'a>,
    options: Options,
    js: ArenaVec<'a, u8>,
    comments: &'a [Comment],
    expr_visitor_stack: ArenaVec<'a, BinaryExprVisitor>,
    builder: ChunkBuilder<'a>,
    next_comment: usize,
    indent: usize,
    print_next_indent_as_space: bool,
    needs_semicolon: bool,

    // Byte offsets used to detect expression contexts where otherwise-valid
    // syntax must be parenthesized.
    stmt_start: usize,
    export_default_start: usize,
    arrow_expr_start: usize,
    for_of_init_start: usize,

    // State used to keep neighboring tokens lexically separate.
    prev_op_end: usize,
    prev_op: Option<OperatorCode>,
    need_space_before_dot: usize,
    prev_reg_exp_end: usize,
    forbid_in: bool,
    int_to_bytes_buffer: [u8; 64],
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TsTypePrecedence {
    Lowest,
    Function,
    Conditional,
    Union,
    Intersection,
    TypeOperator,
    Postfix,
}

enum TsTypeList {
    Union(TsUnionType),
    Intersection(TsIntersectionType),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TsTypeContext {
    Default,
    // Conditional extends clauses and infer constraints accept function types
    // but not bare conditional types. Direct infer constraints consume `extends`.
    ExtendsClause,
    // Function returns allow conditional types again, so a constrained infer
    // needs grouping before the enclosing conditional's `?`.
    BeforeConditionalQuestion,
}

impl TsTypeContext {
    fn inside_group(self, grouped: bool) -> Self {
        if grouped { Self::Default } else { self }
    }

    fn function_return(self, grouped: bool) -> Self {
        match self.inside_group(grouped) {
            Self::Default => Self::Default,
            Self::ExtendsClause | Self::BeforeConditionalQuestion => {
                Self::BeforeConditionalQuestion
            }
        }
    }
}

pub fn print<'a>(allocator: &'a Allocator, tree: &'a Ast<'a>, options: Options) -> PrintResult<'a> {
    Printer::init(allocator, tree, options).generate()
}

impl BinaryExprVisitor {
    fn prepare(&mut self, p: &mut Printer<'_>) {
        let e = self.e;

        let operator = e.operator(p.ast);
        let precedence = operator.precedence();

        self.operator = operator;
        self.wrap = self.level > precedence
            || (self.flags && operator == BinaryishOperator::Binary(BinaryOperator::In));

        if self.wrap {
            p.print("(");
        }

        let local_forbid_in = self.flags && !self.wrap;
        p.forbid_in = local_forbid_in;

        self.left_level = if operator.is_right_associative() {
            precedence.next()
        } else {
            precedence
        };
        self.right_level = if operator.is_right_associative() {
            precedence
        } else {
            precedence.next()
        };
        self.left_flags = local_forbid_in;
        self.left_context = operator.following_context();
        self.right_context = self.context.inside_group(self.wrap)
            & (ExprContext::WRAP_TS_CAST | ExprContext::NO_INSTANTIATION);

        match operator {
            BinaryishOperator::Logical(LogicalOperator::NullishCoalescing) => {
                // "??" cannot directly contain "||" or "&&" without
                // parentheses.
                let needs_nullish_wrap = |child| {
                    matches!(
                        p.ast.expr_data(child),
                        ExprData::LogicalExpression(logical)
                            if matches!(
                                logical.operator(p.ast),
                                LogicalOperator::And | LogicalOperator::Or
                            )
                    )
                };
                if needs_nullish_wrap(e.left(p.ast)) {
                    self.left_level = Precedence::Prefix;
                }
                if needs_nullish_wrap(e.right(p.ast)) {
                    self.right_level = Precedence::Prefix;
                }
            }
            BinaryishOperator::Binary(BinaryOperator::Exponent) => {
                // The left-hand side of "**" cannot be certain unary
                // expressions without parentheses.
                if matches!(
                    p.ast.expr_data(e.left(p.ast)),
                    ExprData::UnaryExpression(_)
                        | ExprData::AwaitExpression(_)
                        | ExprData::TsTypeAssertion(_)
                ) {
                    self.left_level = Precedence::Call;
                }
            }
            BinaryishOperator::Binary(BinaryOperator::LeftShift) => {
                let (mut expression, type_annotation) = match p.ast.expr_data(e.left(p.ast)) {
                    ExprData::TsAsExpression(assertion) => (
                        assertion.expression(p.ast),
                        assertion.type_annotation(p.ast),
                    ),
                    ExprData::TsSatisfiesExpression(assertion) => (
                        assertion.expression(p.ast),
                        assertion.type_annotation(p.ast),
                    ),
                    _ => return,
                };
                if !p.ts_type_can_precede_left_shift(type_annotation) {
                    return;
                }
                // Keep grouping if the cast's operand would absorb the shift.
                loop {
                    expression = match p.ast.expr_data(expression) {
                        ExprData::TsAsExpression(assertion) => assertion.expression(p.ast),
                        ExprData::TsSatisfiesExpression(assertion) => assertion.expression(p.ast),
                        ExprData::BinaryExpression(binary)
                            if binary.operator(p.ast).precedence() < Precedence::Shift =>
                        {
                            return;
                        }
                        ExprData::LogicalExpression(logical)
                            if logical.operator(p.ast).precedence() < Precedence::Shift =>
                        {
                            return;
                        }
                        _ => break,
                    };
                }
                self.left_level = Precedence::Compare;
            }
            _ => {}
        }
    }

    fn visit_right_and_finish(self, p: &mut Printer<'_>) {
        p.print_space();
        if self.operator.is_keyword() {
            p.print_space_before_identifier();
            p.print(self.operator.as_str());
            // This separator is syntactically required even when whitespace
            // minification is enabled (`in this`, not `inthis`).
            p.print(" ");
        } else {
            match self.operator {
                BinaryishOperator::Binary(operator) => {
                    let operator_code = OperatorCode::Binary(operator);
                    p.print_operator(operator_code, operator.as_str());
                }
                BinaryishOperator::Logical(_) => p.print(self.operator.as_str()),
            }
        }
        if !self.operator.is_keyword() {
            p.print_space();
        }

        p.forbid_in = self.flags && !self.wrap;
        p.print_expr_with_context(self.e.right(p.ast), self.right_level, self.right_context);

        if self.wrap {
            p.print(")");
        }
        p.forbid_in = self.flags;
    }
}

impl<'a> Printer<'a> {
    fn init(allocator: &'a Allocator, ast: &'a Ast<'a>, options: Options) -> Self {
        let line_offset_tables = if options.source_map_path.is_some() {
            generate_line_offset_tables(allocator, ast.source().as_bytes())
        } else {
            ArenaVec::new_in(allocator)
        };
        Self {
            allocator,
            ast,
            options,
            js: ArenaVec::with_capacity_in(ast.source().len(), allocator),
            comments: ast.comments(),
            expr_visitor_stack: ArenaVec::new_in(allocator),
            builder: make_chunk_builder(allocator, line_offset_tables),
            next_comment: 0,
            indent: 0,
            print_next_indent_as_space: false,
            needs_semicolon: false,
            stmt_start: usize::MAX,
            export_default_start: usize::MAX,
            arrow_expr_start: usize::MAX,
            for_of_init_start: usize::MAX,
            prev_op_end: usize::MAX,
            prev_op: None,
            need_space_before_dot: usize::MAX,
            prev_reg_exp_end: usize::MAX,
            forbid_in: false,
            int_to_bytes_buffer: [0; 64],
        }
    }

    fn generate(mut self) -> PrintResult<'a> {
        self.print_program(self.ast.root_program());
        let Self {
            allocator,
            ast,
            options,
            js,
            comments,
            builder,
            ..
        } = self;
        let map = if let Some(path) = options.source_map_path.as_deref() {
            let path = path.as_os_str().as_encoded_bytes();
            let chunk = builder.generate_chunk(js.as_slice());
            Some(chunk.finalize(allocator, ast.source().as_bytes(), path))
        } else {
            None
        };

        let code =
            str::from_utf8(js.into_bump_slice()).expect("generated code must be valid UTF-8");
        let legal_comments = comments
            .iter()
            .filter(|comment| is_legal_comment(ast, **comment))
            .copied()
            .collect();

        PrintResult {
            code,
            legal_comments,
            map,
        }
    }

    #[inline]
    fn add_source_mapping(&mut self, original_offset: u32) {
        if self.options.source_map_path.is_some() {
            self.builder
                .add_source_mapping(original_offset, &[], self.js.as_slice());
        }
    }

    #[inline]
    fn add_source_mapping_for_name(&mut self, original_offset: u32, original_name: &'a str) {
        if self.options.source_map_path.is_some() {
            self.builder.add_source_mapping(
                original_offset,
                original_name.as_bytes(),
                self.js.as_slice(),
            );
        }
    }

    #[inline]
    fn print(&mut self, text: &str) {
        self.js.extend_from_slice_copy(text.as_bytes());
    }

    #[inline]
    fn print_bytes(&mut self, text: impl AsRef<[u8]>) {
        self.js.extend_from_slice_copy(text.as_ref());
    }

    fn print_program(&mut self, program: Program) {
        if let Some(hashbang) = program.hashbang(self.ast) {
            self.print("#!");
            self.print_bytes(self.ast.get_utf8(hashbang.value(self.ast)));
            self.print("\n");
        }
        for id in program.directives(self.ast).iter() {
            let directive = self.ast.get_node_in_sub_range(id);
            self.print_indent();
            self.print_directive(directive);
            self.print(";");
            self.print_newline();
            self.print_semicolon_if_needed();
        }
        for (index, id) in program.body(self.ast).iter().enumerate() {
            let statement = self.ast.get_node_in_sub_range(id);
            self.print_stmt_with_context(statement, index == 0);
            self.print_semicolon_if_needed();
        }
        let last_statement_end = program
            .body(self.ast)
            .last()
            .map(|id| self.ast.get_node_in_sub_range(id).span(self.ast).end)
            .or_else(|| {
                program
                    .directives(self.ast)
                    .last()
                    .map(|id| self.ast.get_node_in_sub_range(id).span(self.ast).end)
            })
            .unwrap_or(0);
        while let Some(comment) = self.comments.get(self.next_comment).copied() {
            // Interior comments belong to their node's printer. Keep legal
            // notices even when a node has not consumed them.
            let trailing = comment.span.start >= last_statement_end;
            self.print_comments_before(comment.span.end, true, trailing);
        }
    }

    fn print_directive(&mut self, directive: Directive) {
        let expression = directive.expression(self.ast);
        let value = self.ast.get_wtf8(expression.value(self.ast));
        let raw = self.ast.get_utf8(expression.raw(self.ast));
        let raw_is_safe = !raw.is_empty()
            && (!self.options.inline_script
                || !raw
                    .as_bytes()
                    .windows(8)
                    .any(|text| text.eq_ignore_ascii_case(b"</script")));
        if value == "use strict" && raw.as_bytes().contains(&b'\\') && raw_is_safe {
            self.print_bytes(raw);
            return;
        }
        if !self.options.minify && raw_is_safe && (!self.options.ascii_only || raw.is_ascii()) {
            self.print_bytes(raw);
            return;
        }
        self.print_quoted(value, false);
    }

    #[inline]
    fn print_comments_before(&mut self, offset: u32, statement: bool, preserve_all: bool) {
        if self
            .comments
            .get(self.next_comment)
            .is_some_and(|comment| comment.span.end <= offset)
        {
            self.print_comments_before_slow(offset, statement, preserve_all);
        }
    }

    #[inline(never)]
    fn print_comments_before_slow(&mut self, offset: u32, statement: bool, preserve_all: bool) {
        while self.next_comment < self.comments.len()
            && self.comments[self.next_comment].span.end <= offset
        {
            let comment = self.comments[self.next_comment];
            self.next_comment += 1;
            if !self.options.comments || (!preserve_all && !is_legal_comment(self.ast, comment)) {
                continue;
            }
            let value = self
                .ast
                .get_utf8(comment.value(self.ast.source().as_bytes()))
                .as_bytes();
            let mut trim_start = 0;
            while value[trim_start..]
                .first()
                .is_some_and(|c| c.is_ascii_whitespace())
            {
                trim_start += 1;
            }
            let mut trim_end = value.len();
            while trim_end > trim_start && value[trim_end - 1].is_ascii_whitespace() {
                trim_end -= 1;
            }
            let trimmed = &value[trim_start..trim_end];
            if comment.kind == CommentKind::Block
                && matches!(
                    trimmed,
                    b"@__PURE__" | b"#__PURE__" | b"@__NO_SIDE_EFFECTS__" | b"#__NO_SIDE_EFFECTS__"
                )
            {
                if self.js.last().is_some_and(|&c| !c.is_ascii_whitespace()) {
                    self.print(" ");
                }
                self.print("/* ");
                self.print_bytes(trimmed);
                self.print(" */ ");
                continue;
            }
            let comment_at_stmt_start = !statement && self.js.len() == self.stmt_start;
            if statement {
                if self.js.last().is_some_and(|&c| c != b'\n') {
                    self.print_newline();
                }
                self.print_indent();
            } else if self
                .js
                .last()
                .is_some_and(|&c| !c.is_ascii_whitespace() && (!self.options.minify || c == b'/'))
            {
                self.print(" ");
            }
            match comment.kind {
                CommentKind::Line => {
                    self.print("//");
                    self.print_script_text(value);
                    self.print("\n");
                    if !statement {
                        self.print_indent();
                    }
                }
                CommentKind::Block => {
                    self.print_indented_block_comment(value);
                    if statement || value.contains(&b'\n') || comment_at_stmt_start {
                        self.print_newline();
                    } else {
                        self.print_space();
                    }
                }
            }
        }
    }

    fn print_indented_block_comment(&mut self, value: &[u8]) {
        // Re-indent multi-line comments to match the surrounding statement.
        self.print("/*");
        let mut start = 0;
        let mut search = 0;
        while let Some(relative) = memchr3(b'\n', b'\r', 0xe2, &value[search..]) {
            let end = search + relative;
            let newline_width = match value[end] {
                b'\n' => 1,
                b'\r' => usize::from(value.get(end + 1) == Some(&b'\n')) + 1,
                0xe2 if value.get(end + 1) == Some(&0x80)
                    && matches!(value.get(end + 2), Some(0xa8 | 0xa9)) =>
                {
                    3
                }
                _ => {
                    search = end + 1;
                    continue;
                }
            };
            self.print_script_text(&value[start..end]);
            self.print("\n");
            self.print_indent();
            start = end + newline_width;
            let whitespace_start = start;
            while value[start..]
                .first()
                .is_some_and(|c| c.is_ascii_whitespace())
            {
                start += 1;
            }
            if !self.options.minify
                || start != whitespace_start
                || value[start..].first() == Some(&b'*')
            {
                self.print(" ");
            }
            search = start;
        }
        self.print_script_text(&value[start..]);
        self.print("*/");
    }

    fn print_script_text(&mut self, text: &[u8]) {
        if !self.options.inline_script {
            self.print_bytes(text);
            return;
        }
        let mut start = 0;
        for index in 0..text.len().saturating_sub(7) {
            if text[index] == b'<'
                && text[index + 1] == b'/'
                && text[index + 2..index + 8].eq_ignore_ascii_case(b"script")
            {
                self.print_bytes(&text[start..index + 1]);
                self.print("\\");
                start = index + 1;
            }
        }
        self.print_bytes(&text[start..]);
    }

    fn print_attached_trailing_comments(&mut self, attached_to: u32) {
        let mut trim_trailing_space = false;
        while attached_to != 0
            && let Some(comment) = self.comments.get(self.next_comment).copied()
        {
            if comment.position != CommentPosition::Trailing || comment.attached_to != attached_to {
                break;
            }
            self.print_comments_before(comment.span.end, false, true);
            trim_trailing_space =
                comment.kind == CommentKind::Block && !self.comment_has_line_terminator(comment);
        }
        if trim_trailing_space && self.js.last() == Some(&b' ') {
            self.js.pop();
        }
    }

    fn discard_comments_before_trailing_comma(&mut self, close_offset: u32) {
        while let Some(comment) = self.comments.get(self.next_comment).copied() {
            if comment.span.end > close_offset || is_legal_comment(self.ast, comment) {
                break;
            }
            if !self.ast.source()[comment.span.end as usize..close_offset as usize].contains(',') {
                break;
            }
            self.next_comment += 1;
        }
    }

    fn has_direct_argument_comments(&self, args: SubRange<Argument>, close_offset: u32) -> bool {
        self.comments[self.next_comment..]
            .iter()
            .take_while(|comment| comment.span.end <= close_offset)
            .any(|comment| {
                !args.iter().any(|id| {
                    let span = self.ast.get_node_in_sub_range(id).span(self.ast);
                    span.start <= comment.span.start && comment.span.end <= span.end
                })
            })
    }

    #[inline]
    fn print_indent(&mut self) {
        if self.options.minify {
            return;
        }
        if self.print_next_indent_as_space {
            self.print(" ");
            self.print_next_indent_as_space = false;
            return;
        }
        for _ in 0..self.indent {
            self.print("  ");
        }
    }

    #[inline]
    fn print_space(&mut self) {
        if !self.options.minify {
            self.print(" ");
        }
    }

    #[inline]
    fn print_newline(&mut self) {
        if !self.options.minify {
            self.print("\n");
        }
    }

    #[inline]
    fn print_operator(&mut self, next: OperatorCode, text: &str) {
        self.print_space_before_operator(next, text);
        self.print_operator_token(next, text);
    }

    #[inline]
    fn print_mapped_operator(&mut self, next: OperatorCode, text: &str, source_offset: u32) {
        self.print_space_before_operator(next, text);
        self.add_source_mapping(source_offset);
        self.print_operator_token(next, text);
    }

    #[inline]
    fn print_operator_token(&mut self, next: OperatorCode, text: &str) {
        self.print(text);
        // Assignment operators end in `=` and cannot fuse with the next
        // expression token. Their leading edge still needs separation.
        if !matches!(next, OperatorCode::Assignment) {
            self.prev_op = Some(next);
            self.prev_op_end = self.js.len();
        }
    }

    #[inline]
    fn print_space_before_operator(&mut self, next: OperatorCode, text: &str) {
        if self.prev_op_end != self.js.len() {
            return;
        }
        let Some(previous) = self.prev_op else {
            return;
        };
        let needs_space = match previous {
            // A prefix or binary sign must not turn into an update token.
            OperatorCode::Binary(BinaryOperator::Add)
            | OperatorCode::Unary(UnaryOperator::Positive) => matches!(
                next,
                OperatorCode::Binary(BinaryOperator::Add)
                    | OperatorCode::Unary(UnaryOperator::Positive)
                    | OperatorCode::Update(UpdateOperator::Increment, true)
            ),
            OperatorCode::Binary(BinaryOperator::Subtract)
            | OperatorCode::Unary(UnaryOperator::Negate) => matches!(
                next,
                OperatorCode::Binary(BinaryOperator::Subtract)
                    | OperatorCode::Unary(UnaryOperator::Negate)
                    | OperatorCode::Update(UpdateOperator::Decrement, true)
            ),
            // Avoid HTML comment delimiters across operator boundaries.
            OperatorCode::Update(UpdateOperator::Decrement, false) => {
                next == OperatorCode::Binary(BinaryOperator::GreaterThan)
            }
            OperatorCode::Unary(UnaryOperator::LogicalNot) => {
                next == OperatorCode::Update(UpdateOperator::Decrement, true)
                    && self.js.len() > 1
                    && self.js[self.js.len() - 2] == b'<'
            }
            // A non-null assertion must not fuse into `!=` or `!==`.
            OperatorCode::TsNonNull => text.starts_with('='),
            // A comparison followed by a type assertion must not become `<<`.
            // The same boundary occurs before a generic function type.
            OperatorCode::Binary(BinaryOperator::LessThan) | OperatorCode::TsTypeOpen => {
                next == OperatorCode::TsTypeOpen
            }
            // Keep type closers separate from expression operators, while
            // allowing nested type argument lists to close with `>>`.
            OperatorCode::TsTypeClose => {
                matches!(next, OperatorCode::Binary(_) | OperatorCode::Assignment)
                    && matches!(text.as_bytes()[0], b'>' | b'=')
            }
            _ => false,
        };
        if needs_space {
            self.print(" ");
        }
    }

    #[inline]
    fn print_semicolon_after_statement(&mut self) {
        if self.options.minify {
            self.needs_semicolon = true;
        } else {
            self.print(";\n");
            self.needs_semicolon = false;
        }
    }

    #[inline]
    fn print_semicolon_if_needed(&mut self) {
        if self.needs_semicolon {
            self.print(";");
            self.needs_semicolon = false;
        }
    }

    #[inline]
    fn print_space_before_identifier(&mut self) {
        if self.prev_reg_exp_end == self.js.len()
            || self
                .js
                .last()
                .is_some_and(|&last| is_ident_continue_ascii(last) || last == b'\\' || last >= 0x80)
        {
            self.print(" ");
        }
    }

    #[inline]
    fn print_identifier(&mut self, name: &str) {
        self.print_space_before_identifier();
        if !self.options.ascii_only || name.is_ascii() {
            self.print_bytes(name);
        } else {
            self.print_identifier_ascii_only(name);
        }
    }

    #[cold]
    fn print_identifier_ascii_only(&mut self, name: &str) {
        let name = name.as_bytes();
        let mut i = 0;
        while i < name.len() {
            let (cp, size) = next_code_point(name, i);
            i += size;
            if cp <= 0x7e {
                self.js.push(cp as u8);
            } else if cp <= 0xffff {
                push_u16_escape(&mut self.js, cp as u16);
            } else {
                push_code_point_escape(&mut self.js, cp);
            }
        }
    }

    fn print_quoted(&mut self, value: &Wtf8, allow_backtick: bool) {
        let allow_backtick = allow_backtick && self.options.minify;
        let bytes = value.as_bytes();
        let single_cost = bytes.iter().filter(|&&byte| byte == b'\'').count();
        let double_cost = bytes.iter().filter(|&&byte| byte == b'"').count();
        let (mut quote, quote_cost) = if double_cost > single_cost {
            (b'\'', single_cost)
        } else {
            (b'"', double_cost)
        };
        if allow_backtick {
            let backtick_cost = bytes.iter().filter(|&&byte| byte == b'`').count()
                + bytes.windows(2).filter(|pair| *pair == b"${").count();
            if backtick_cost < quote_cost {
                quote = b'`';
            }
        }
        quote_into(
            &mut self.js,
            value,
            quote,
            self.options.ascii_only,
            self.options.inline_script,
        );
    }

    fn print_stmt(&mut self, stmt: Stmt) {
        self.print_stmt_with_context(stmt, false);
    }

    fn print_stmt_with_context(&mut self, stmt: Stmt, prevent_directive: bool) {
        self.print_comments_before(stmt.span(self.ast).start, true, true);
        self.add_source_mapping(stmt.span(self.ast).start);
        self.stmt_start = self.js.len();
        match self.ast.stmt_data(stmt) {
            StmtData::BlockStatement(s) => {
                self.print_indent();
                self.print_block(s);
                self.print_newline();
            }
            StmtData::BreakStatement(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("break");
                if let Some(label) = s.label(self.ast) {
                    self.print(" ");
                    let name = self.ast.get_utf8(label.name(self.ast));
                    self.add_source_mapping_for_name(label.span(self.ast).start, name);
                    self.print_identifier(name);
                }
                self.print_semicolon_after_statement();
            }
            StmtData::ContinueStatement(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("continue");
                if let Some(label) = s.label(self.ast) {
                    self.print(" ");
                    let name = self.ast.get_utf8(label.name(self.ast));
                    self.add_source_mapping_for_name(label.span(self.ast).start, name);
                    self.print_identifier(name);
                }
                self.print_semicolon_after_statement();
            }
            StmtData::DebuggerStatement(_) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("debugger");
                self.print_semicolon_after_statement();
            }
            StmtData::DoWhileStatement(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("do");
                let body = s.body(self.ast);
                if let Some(block) = body.as_block_statement(self.ast) {
                    self.print_space();
                    self.print_block(block);
                    self.print_space();
                } else {
                    if self.options.minify {
                        self.print(" ");
                    }
                    self.print_newline();
                    self.indent += 1;
                    self.print_stmt(body);
                    self.indent -= 1;
                    self.print_indent();
                }
                self.print_semicolon_if_needed();
                self.print("while");
                self.print_space();
                self.print("(");
                self.print_expr(s.test(self.ast), Precedence::Lowest);
                self.print(");");
                self.needs_semicolon = false;
                self.print_newline();
            }
            StmtData::EmptyStatement(_) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                // The semicolon is the statement itself, not an optional terminator.
                self.print(";");
                self.needs_semicolon = false;
                self.print_newline();
            }
            StmtData::ExpressionStatement(s) => {
                let expression = s.expression(self.ast);
                if self
                    .comments
                    .get(self.next_comment)
                    .is_some_and(|comment| comment.span.end <= expression.span(self.ast).end)
                {
                    let first_token = self.expr_first_token_offset(expression);
                    self.print_comments_before(first_token, true, true);
                }
                self.print_indent();
                self.stmt_start = self.js.len();
                let wrap = prevent_directive
                    && matches!(self.ast.expr_data(expression), ExprData::StringLiteral(_));
                if wrap {
                    self.print("(");
                }
                self.print_expr(expression, Precedence::Lowest);
                self.print_comments_before(s.span(self.ast).end, false, true);
                if wrap {
                    self.print(")");
                }
                self.print_semicolon_after_statement();
            }
            StmtData::ForInStatement(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("for");
                self.print_space();
                self.print("(");
                let left = s.left(self.ast);
                let leading_comment_end = self.comments[self.next_comment..]
                    .first()
                    .filter(|comment| comment.span.end <= left.span(self.ast).end)
                    .map(|comment| comment.span.end);
                if !self.options.minify
                    && let Some(comment_end) = leading_comment_end
                {
                    self.print_newline();
                    self.indent += 1;
                    self.print_comments_before(comment_end, true, true);
                    self.print_indent();
                    self.print_for_left(left);
                    self.print(" in ");
                    self.print_expr(s.right(self.ast), Precedence::Lowest);
                    self.print_newline();
                    self.indent -= 1;
                    self.print_indent();
                } else {
                    self.print_for_left(left);
                    self.print(" in ");
                    self.print_expr(s.right(self.ast), Precedence::Lowest);
                }
                self.print(")");
                self.print_body(s.body(self.ast));
            }
            StmtData::ForOfStatement(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("for");
                let is_await = s.r#await(self.ast);
                if is_await {
                    self.print(" await");
                }
                self.print_space();
                self.print("(");
                let left = s.left(self.ast);
                let leading_comment_end = self.comments[self.next_comment..]
                    .first()
                    .filter(|comment| comment.span.end <= left.span(self.ast).end)
                    .map(|comment| comment.span.end);
                if !self.options.minify
                    && let Some(comment_end) = leading_comment_end
                {
                    self.print_newline();
                    self.indent += 1;
                    self.print_comments_before(comment_end, true, true);
                    self.print_indent();
                    self.for_of_init_start = self.js.len();
                    self.print_for_of_left(left, is_await);
                    self.print(" of ");
                    self.print_expr(s.right(self.ast), Precedence::Assignment);
                    self.print_newline();
                    self.indent -= 1;
                    self.print_indent();
                } else {
                    self.for_of_init_start = self.js.len();
                    self.print_for_of_left(left, is_await);
                    self.print(" of ");
                    self.print_expr(s.right(self.ast), Precedence::Assignment);
                }
                self.print(")");
                self.print_body(s.body(self.ast));
            }
            StmtData::ForStatement(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("for");
                self.print_space();
                self.print("(");
                if let Some(init) = s.init(self.ast) {
                    let old_forbid_in = self.forbid_in;
                    self.forbid_in = true;
                    match self.ast.for_statement_init_data(init) {
                        ForStatementInitData::VariableDeclaration(v) => {
                            self.print_variable_decl(v, false)
                        }
                        ForStatementInitData::Expr(e) => self.print_expr(e, Precedence::Lowest),
                    }
                    self.forbid_in = old_forbid_in;
                }
                self.print(";");
                self.print_space();
                if let Some(test) = s.test(self.ast) {
                    self.print_expr(test, Precedence::Lowest);
                }
                self.print(";");
                self.print_space();
                if let Some(update) = s.update(self.ast) {
                    self.print_expr(update, Precedence::Lowest);
                }
                self.print(")");
                self.print_body(s.body(self.ast));
            }
            StmtData::IfStatement(s) => self.print_if(s),
            StmtData::LabeledStatement(s) => {
                self.print_indent();
                let label = s.label(self.ast);
                let name = self.ast.get_utf8(label.name(self.ast));
                self.add_source_mapping_for_name(label.span(self.ast).start, name);
                self.print_identifier(name);
                self.print(":");
                self.print_body(s.body(self.ast));
            }
            StmtData::ReturnStatement(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("return");
                if let Some(value) = s.argument(self.ast) {
                    let can_omit_space = self.options.minify
                        && matches!(
                            self.ast.expr_data(value),
                            ExprData::BinaryExpression(binary)
                                if binary.operator(self.ast) == BinaryOperator::In
                                    && matches!(
                                        self.ast.expr_data(binary.left(self.ast)),
                                        ExprData::PrivateIdentifier(_)
                                    )
                        );
                    if !can_omit_space {
                        self.print(" ");
                    }
                    self.print_expr_without_leading_line_terminator(value, Precedence::Lowest);
                }
                self.print_semicolon_after_statement();
            }
            StmtData::SwitchStatement(s) => self.print_switch(s),
            StmtData::ThrowStatement(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("throw");
                self.print_space();
                self.print_expr_without_leading_line_terminator(
                    s.argument(self.ast),
                    Precedence::Lowest,
                );
                self.print_semicolon_after_statement();
            }
            StmtData::TryStatement(s) => self.print_try(s),
            StmtData::WhileStatement(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("while");
                self.print_space();
                self.print("(");
                self.print_expr(s.test(self.ast), Precedence::Lowest);
                self.print(")");
                self.print_body(s.body(self.ast));
            }
            StmtData::WithStatement(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("with");
                self.print_space();
                self.print("(");
                self.print_expr(s.object(self.ast), Precedence::Lowest);
                self.print(")");
                self.print_body(s.body(self.ast));
            }
            StmtData::Declaration(d) => {
                if !matches!(self.ast.decl_data(d), DeclData::Class(_)) {
                    self.print_indent();
                }
                self.print_decl(d);
                self.print_decl_terminator(d);
            }
            StmtData::ImportDeclaration(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print_import(s);
                self.print_semicolon_after_statement();
            }
            StmtData::ExportNamedDeclaration(s) => self.print_export_named(s),
            StmtData::ExportDefaultDeclaration(s) => self.print_export_default(s),
            StmtData::ExportAllDeclaration(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("export");
                self.print_space();
                if s.export_kind(self.ast) == ImportOrExportKind::Type {
                    self.print("type ");
                }
                self.print("*");
                if let Some(exported) = s.exported(self.ast) {
                    self.print_space();
                    self.print_space_before_identifier();
                    self.print("as");
                    if matches!(
                        self.ast.module_export_name_data(exported),
                        ModuleExportNameData::IdentifierName(_)
                    ) {
                        self.print(" ");
                    } else {
                        self.print_space();
                    }
                    self.print_module_name(exported);
                }
                self.print_space();
                self.print_space_before_identifier();
                self.print("from");
                self.print_space();
                self.add_source_mapping(s.source(self.ast).span(self.ast).start);
                self.print_quoted(self.ast.get_wtf8(s.source(self.ast).value(self.ast)), false);
                self.print_import_attributes(s.attributes(self.ast), s.span(self.ast));
                self.print_semicolon_after_statement();
            }
            StmtData::TsExportAssignment(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("export = ");
                self.print_expr(s.expression(self.ast), Precedence::Lowest);
                self.print_semicolon_after_statement();
            }
            StmtData::TsNamespaceExportDeclaration(s) => {
                self.print_indent();
                self.add_source_mapping(stmt.span(self.ast).start);
                self.print("export as namespace ");
                self.print_identifier(self.ast.get_utf8(s.id(self.ast).name(self.ast)));
                self.print_semicolon_after_statement();
            }
        }
    }

    fn print_body(&mut self, body: Stmt) {
        if let Some(block) = body.as_block_statement(self.ast) {
            self.print_space();
            self.print_block(block);
            self.print_newline();
        } else {
            while self.next_comment < self.comments.len()
                && self.comments[self.next_comment].span.end <= body.span(self.ast).start
            {
                self.next_comment += 1;
            }
            self.print_next_indent_as_space = true;
            self.print_stmt(body);
        }
    }

    fn print_block(&mut self, block: BlockStatement) {
        self.add_source_mapping(block.span(self.ast).start);
        self.print("{");
        let body = block.body(self.ast);
        if body.is_empty() {
            let close_offset = block.span(self.ast).end.saturating_sub(1);
            if self.comments[self.next_comment..]
                .iter()
                .any(|comment| comment.span.end <= close_offset)
            {
                self.print_newline();
                self.indent += 1;
                self.print_comments_before(close_offset, true, true);
                self.indent -= 1;
                self.print_indent();
            } else if !self.options.minify {
                self.print_newline();
                self.print_indent();
            }
            self.add_source_mapping(block.span(self.ast).end.saturating_sub(1));
            self.print("}");
            return;
        }
        self.print_newline();
        self.indent += 1;
        for id in body.iter() {
            self.print_semicolon_if_needed();
            self.print_stmt(self.ast.get_node_in_sub_range(id));
        }
        self.needs_semicolon = false;
        self.print_comments_before(block.span(self.ast).end.saturating_sub(1), true, true);
        self.indent -= 1;
        self.print_indent();
        self.add_source_mapping(block.span(self.ast).end.saturating_sub(1));
        self.print("}");
    }

    fn print_function_body(&mut self, body: FunctionBody) {
        self.add_source_mapping(body.span(self.ast).start);
        self.print("{");
        let directives = body.directives(self.ast);
        let statements = body.body(self.ast);
        if directives.is_empty() && statements.is_empty() {
            let close_offset = body.span(self.ast).end.saturating_sub(1);
            if self.comments[self.next_comment..]
                .iter()
                .any(|comment| comment.span.end <= close_offset)
            {
                self.print_newline();
                self.indent += 1;
                self.print_comments_before(close_offset, true, true);
                self.indent -= 1;
                self.print_indent();
            } else if !self.options.minify {
                self.print_newline();
                self.print_indent();
            }
            self.add_source_mapping(body.span(self.ast).end.saturating_sub(1));
            self.print("}");
            return;
        }
        self.print_newline();
        self.indent += 1;
        for id in directives.iter() {
            self.print_semicolon_if_needed();
            let directive = self.ast.get_node_in_sub_range(id);
            self.print_indent();
            self.print_directive(directive);
            self.print_semicolon_after_statement();
        }
        for (index, id) in statements.iter().enumerate() {
            self.print_semicolon_if_needed();
            let statement = self.ast.get_node_in_sub_range(id);
            self.print_stmt_with_context(statement, index == 0);
        }
        self.needs_semicolon = false;
        self.print_comments_before(body.span(self.ast).end.saturating_sub(1), true, true);
        self.indent -= 1;
        self.print_indent();
        self.add_source_mapping(body.span(self.ast).end.saturating_sub(1));
        self.print("}");
    }

    fn print_if(&mut self, s: IfStatement) {
        self.print_indent();
        self.add_source_mapping(s.span(self.ast).start);
        self.print("if");
        self.print_space();
        self.print("(");
        self.print_expr(s.test(self.ast), Precedence::Lowest);
        self.print(")");
        let consequent = s.consequent(self.ast);
        if let Some(block) = consequent.as_block_statement(self.ast) {
            self.print_space();
            self.print_block(block);
        } else {
            self.print_body(consequent);
        }
        if let Some(alternate) = s.alternate(self.ast) {
            self.print_semicolon_if_needed();
            if consequent.is_block_statement(self.ast) {
                self.print_space();
            } else {
                self.print_indent();
            }
            self.print("else");
            if let Some(nested) = alternate.as_if_statement(self.ast) {
                self.print(" ");
                let saved = self.indent;
                self.indent = 0;
                self.print_if(nested);
                self.indent = saved;
                return;
            }
            if let Some(block) = alternate.as_block_statement(self.ast) {
                self.print_space();
                self.print_block(block);
                self.print_newline();
            } else {
                if self.options.minify {
                    self.print(" ");
                }
                self.print_body(alternate);
            }
        } else {
            if consequent.is_block_statement(self.ast) {
                self.print_newline();
            }
        }
    }

    fn print_switch(&mut self, s: SwitchStatement) {
        self.print_indent();
        self.add_source_mapping(s.span(self.ast).start);
        self.print("switch");
        self.print_space();
        self.print("(");
        self.print_expr(s.discriminant(self.ast), Precedence::Lowest);
        self.print(")");
        self.print_space();
        self.print("{");
        let cases = s.cases(self.ast);
        if !cases.is_empty() {
            self.print_newline();
        }
        self.indent += 1;
        for id in cases.iter() {
            self.print_semicolon_if_needed();
            let case = self.ast.get_node_in_sub_range(id);
            self.print_comments_before(case.span(self.ast).start, true, true);
            self.print_indent();
            self.add_source_mapping(case.span(self.ast).start);
            if let Some(test) = case.test(self.ast) {
                self.print("case");
                self.print_space();
                self.print_expr(test, Precedence::Lowest);
                self.print(":");
            } else {
                self.print("default:");
            }
            let consequent = case.consequent(self.ast);
            if consequent.len() == 1 {
                let statement = self.ast.get_node_in_sub_range(consequent.first().unwrap());
                if let Some(block) = statement.as_block_statement(self.ast) {
                    self.print_space();
                    self.print_block(block);
                    self.print_newline();
                    continue;
                }
            }
            self.print_newline();
            self.indent += 1;
            for id in consequent.iter() {
                self.print_semicolon_if_needed();
                self.print_stmt(self.ast.get_node_in_sub_range(id));
            }
            self.indent -= 1;
        }
        self.needs_semicolon = false;
        self.indent -= 1;
        self.print_indent();
        self.add_source_mapping(s.span(self.ast).end.saturating_sub(1));
        self.print("}");
        self.print_newline();
    }

    fn print_try(&mut self, s: TryStatement) {
        self.print_indent();
        self.add_source_mapping(s.span(self.ast).start);
        self.print("try");
        self.print_space();
        self.print_block(s.block(self.ast));
        if let Some(handler) = s.handler(self.ast) {
            self.print_space();
            self.print("catch");
            if let Some(param) = handler.param(self.ast) {
                self.print_space();
                self.print("(");
                self.print_binding(param);
                if let Some(annotation) = handler.type_annotation(self.ast) {
                    self.print_ts_annotation(annotation);
                }
                self.print(")");
            }
            self.print_space();
            self.print_block(handler.body(self.ast));
        }
        if let Some(finalizer) = s.finalizer(self.ast) {
            self.print_space();
            self.print("finally");
            self.print_space();
            self.print_block(finalizer);
        }
        self.print_newline();
    }

    fn print_for_left(&mut self, left: ForStatementLeft) {
        match self.ast.for_statement_left_data(left) {
            ForStatementLeftData::VariableDeclaration(v) => self.print_variable_decl(v, false),
            ForStatementLeftData::AssignmentTarget(t) => self.print_assignment_target(t),
        }
    }

    fn print_for_of_left(&mut self, left: ForStatementLeft, is_await: bool) {
        if is_await {
            self.print_for_left(left);
            return;
        }
        let Some(identifier) = left
            .as_assignment_target(self.ast)
            .and_then(|target| target.as_simple_assignment_target(self.ast))
            .and_then(|target| target.as_identifier_reference(self.ast))
        else {
            self.print_for_left(left);
            return;
        };
        let wrap = self.ast.get_utf8(identifier.name(self.ast)) == "async";
        if wrap {
            self.print("(");
        }
        self.print_for_left(left);
        if wrap {
            self.print(")");
        }
    }

    fn print_decl(&mut self, decl: Decl) {
        match self.ast.decl_data(decl) {
            DeclData::VariableDeclaration(v) => self.print_variable_decl(v, true),
            DeclData::Function(f) => self.print_function(f),
            DeclData::TsFunction(f) => self.print_ts_function(f, true),
            DeclData::Class(c) => self.print_class(c, true),
            DeclData::TsTypeAliasDeclaration(d) => self.print_ts_type_alias(d),
            DeclData::TsInterfaceDeclaration(d) => self.print_ts_interface(d),
            DeclData::TsEnumDeclaration(d) => self.print_ts_enum(d),
            DeclData::TsModuleDeclaration(d) => self.print_ts_module(d),
            DeclData::TsGlobalDeclaration(d) => self.print_ts_global(d),
            DeclData::TsImportEqualsDeclaration(d) => self.print_ts_import_equals(d),
        }
    }

    fn print_decl_terminator(&mut self, decl: Decl) {
        let ends_with_block = match self.ast.decl_data(decl) {
            DeclData::Function(_)
            | DeclData::Class(_)
            | DeclData::TsInterfaceDeclaration(_)
            | DeclData::TsEnumDeclaration(_)
            | DeclData::TsGlobalDeclaration(_) => true,
            DeclData::TsModuleDeclaration(module) => module.body(self.ast).is_some(),
            _ => false,
        };
        if ends_with_block {
            self.print_newline();
        } else {
            self.print_semicolon_after_statement();
        }
    }

    fn print_variable_decl(&mut self, decl: VariableDeclaration, include_declare: bool) {
        self.add_source_mapping(decl.span(self.ast).start);
        if include_declare && decl.declare(self.ast) {
            self.print("declare ");
        }
        self.print(decl.kind(self.ast).as_str());
        self.print_space();
        for (i, id) in decl.declarators(self.ast).iter().enumerate() {
            if i != 0 {
                self.print(",");
                self.print_space();
            }
            let d = self.ast.get_node_in_sub_range(id);
            self.print_comments_before(d.span(self.ast).start, false, true);
            self.print_binding(d.id(self.ast));
            if d.definite(self.ast) {
                self.print("!");
            }
            if let Some(annotation) = d.type_annotation(self.ast) {
                self.print_ts_annotation(annotation);
            }
            if let Some(value) = d.init(self.ast) {
                self.print_space();
                self.print("=");
                self.print_space();
                self.print_expr(value, Precedence::Assignment);
            }
        }
        self.print_comments_before(decl.span(self.ast).end, false, true);
    }

    fn print_function(&mut self, f: Function) {
        let is_expression = matches!(f.function_type(self.ast), FunctionType::FunctionExpression);
        let offset = self.js.len();
        let wrap =
            is_expression && (offset == self.stmt_start || offset == self.export_default_start);
        if wrap {
            self.print("(");
        }
        self.print_space_before_identifier();
        self.add_source_mapping(f.span(self.ast).start);
        if f.r#async(self.ast) {
            self.print("async ");
        }
        self.print("function");
        if f.generator(self.ast) {
            self.print("*");
            self.print_space();
        }
        if let Some(id) = f.id(self.ast) {
            self.print_space_before_identifier();
            self.print_binding_identifier(id);
        }
        if let Some(params) = f.type_parameters(self.ast) {
            self.print_ts_type_params(params);
        }
        self.print_formal_parameters(f.params(self.ast), f.this_param(self.ast));
        if let Some(ret) = f.return_type(self.ast) {
            self.print_ts_annotation(ret);
        }
        self.print_space();
        self.print_function_body(f.body(self.ast));
        self.print_attached_trailing_comments(f.body(self.ast).span(self.ast).end);
        if wrap {
            self.print(")");
        }
    }

    fn print_ts_function(&mut self, f: TsFunction, declaration: bool) {
        self.print_space_before_identifier();
        self.add_source_mapping(f.span(self.ast).start);
        if f.declare(self.ast) {
            self.print("declare ");
        }
        if f.r#async(self.ast) {
            self.print("async ");
        }
        self.print("function");
        if f.generator(self.ast) {
            self.print("*");
        }
        if let Some(id) = f.id(self.ast) {
            self.print_space();
            self.print_binding_identifier(id);
        } else if declaration || f.generator(self.ast) {
            self.print_space();
        }
        if let Some(params) = f.type_parameters(self.ast) {
            self.print_ts_type_params(params);
        }
        self.print_formal_parameters(f.params(self.ast), f.this_param(self.ast));
        if let Some(ret) = f.return_type(self.ast) {
            self.print_ts_annotation(ret);
        }
    }

    fn print_formal_parameters(
        &mut self,
        params: FormalParameters,
        this_param: Option<TsThisParameter>,
    ) {
        self.add_source_mapping(params.span(self.ast).start);
        self.print("(");
        if let Some(parameter) = this_param {
            self.print("this");
            if let Some(annotation) = parameter.type_annotation(self.ast) {
                self.print_ts_annotation(annotation);
            }
        }
        let items = params.items(self.ast);
        for (i, id) in items.iter().enumerate() {
            if i != 0 || this_param.is_some() {
                self.print(",");
                self.print_space();
            }
            match self
                .ast
                .formal_parameter_item_data(self.ast.get_node_in_sub_range(id))
            {
                FormalParameterItemData::FormalParameter(p) => self.print_formal_parameter(p),
                FormalParameterItemData::TsParameterProperty(p) => {
                    self.print_ts_parameter_property(p)
                }
            }
        }
        if let Some(rest) = params.rest(self.ast) {
            if !items.is_empty() || this_param.is_some() {
                self.print(",");
                self.print_space();
            }
            self.print_decorators(rest.decorators(self.ast));
            self.print("...");
            self.print_binding(rest.argument(self.ast).into());
            if let Some(annotation) = rest.type_annotation(self.ast) {
                self.print_ts_annotation(annotation);
            }
        }
        self.print(")");
    }

    fn print_formal_parameter(&mut self, parameter: FormalParameter) {
        self.print_decorators(parameter.decorators(self.ast));
        self.print_binding(parameter.pattern(self.ast).into());
        if parameter.optional(self.ast) {
            self.print("?");
        }
        if let Some(annotation) = parameter.type_annotation(self.ast) {
            self.print_ts_annotation(annotation);
        }
        if let Some(initializer) = parameter.initializer(self.ast) {
            let old_forbid_in = self.forbid_in;
            self.forbid_in = false;
            self.print_space();
            self.print("=");
            self.print_space();
            self.print_expr(initializer, Precedence::Assignment);
            self.forbid_in = old_forbid_in;
        }
    }

    fn print_binding_identifier(&mut self, id: BindingIdentifier) {
        let name = self.ast.get_utf8(id.name(self.ast));
        self.add_source_mapping_for_name(id.span(self.ast).start, name);
        self.print_identifier(name);
    }

    fn print_binding(&mut self, binding: BindingPattern) {
        match self.ast.binding_pattern_data(binding) {
            BindingPatternData::BindingIdentifier(id) => self.print_binding_identifier(id),
            BindingPatternData::SimpleAssignmentTarget(t) => self.print_simple_assignment_target(t),
            BindingPatternData::AssignmentPattern(p) => {
                let old_forbid_in = self.forbid_in;
                self.forbid_in = false;
                self.print_binding(p.left(self.ast));
                self.print_space();
                self.print("=");
                self.print_space();
                self.print_expr(p.right(self.ast), Precedence::Assignment);
                self.forbid_in = old_forbid_in;
            }
            BindingPatternData::BindingRestElement(r) => {
                self.add_source_mapping(r.span(self.ast).start);
                self.print("...");
                self.print_binding(r.argument(self.ast));
            }
            BindingPatternData::ArrayPattern(a) => self.print_array_pattern(a),
            BindingPatternData::ObjectPattern(o) => self.print_object_pattern(o),
        }
    }

    fn print_array_pattern(&mut self, a: ArrayPattern) {
        self.add_source_mapping(a.span(self.ast).start);
        self.print("[");
        let elements = a.elements(self.ast);
        let rest = a.rest(self.ast);
        for (i, id) in elements.iter().enumerate() {
            if i != 0 {
                self.print(",");
                self.print_space();
            }
            if let Some(item) = self.ast.get_node_in_sub_range(id) {
                self.print_binding(item);
            } else if i + 1 == elements.len() && rest.is_none() {
                self.print(",");
            }
        }
        if let Some(rest) = rest {
            if !elements.is_empty() {
                self.print(",");
                self.print_space();
            }
            self.add_source_mapping(rest.span(self.ast).start);
            self.print("...");
            self.print_binding(rest.argument(self.ast));
        }
        self.add_source_mapping(a.span(self.ast).end.saturating_sub(1));
        self.print("]");
    }

    fn print_object_pattern(&mut self, o: ObjectPattern) {
        self.add_source_mapping(o.span(self.ast).start);
        self.print("{");
        let props = o.properties(self.ast);
        if !props.is_empty() || o.rest(self.ast).is_some() {
            self.print_space();
        }
        for (i, id) in props.iter().enumerate() {
            if i != 0 {
                self.print(",");
                self.print_space();
            }
            let prop = self.ast.get_node_in_sub_range(id);
            let value = prop.value(self.ast);
            let can_shorthand = self.options.minify
                && !prop.computed(self.ast)
                && match self.ast.property_key_data(prop.key(self.ast)) {
                    PropertyKeyData::IdentifierName(key) => {
                        self.binding_pattern_identifier_name(value)
                            == Some(self.ast.get_utf8(key.name(self.ast)))
                    }
                    _ => false,
                };
            if prop.shorthand(self.ast) {
                self.print_binding(value);
            } else if can_shorthand {
                self.print_property_key(prop.key(self.ast), false);
            } else {
                self.print_property_key(prop.key(self.ast), prop.computed(self.ast));
                self.print(":");
                self.print_space();
                self.print_binding(value);
            }
        }
        if let Some(rest) = o.rest(self.ast) {
            if !props.is_empty() {
                self.print(",");
                self.print_space();
            }
            self.add_source_mapping(rest.span(self.ast).start);
            self.print("...");
            self.print_binding(rest.argument(self.ast));
        }
        if !props.is_empty() || o.rest(self.ast).is_some() {
            self.print_space();
        }
        self.add_source_mapping(o.span(self.ast).end.saturating_sub(1));
        self.print("}");
    }

    fn print_assignment_target(&mut self, target: AssignmentTarget) {
        match self.ast.assignment_target_data(target) {
            AssignmentTargetData::SimpleAssignmentTarget(t) => {
                self.print_simple_assignment_target(t)
            }
            AssignmentTargetData::ArrayAssignmentTarget(a) => self.print_array_pattern(a),
            AssignmentTargetData::ObjectAssignmentTarget(o) => self.print_object_pattern(o),
        }
    }

    fn print_simple_assignment_target(&mut self, target: SimpleAssignmentTarget) {
        match self.ast.simple_assignment_target_data(target) {
            SimpleAssignmentTargetData::IdentifierReference(i) => {
                let name = self.ast.get_utf8(i.name(self.ast));
                let wrap = self.js.len() == self.for_of_init_start && name == "let";
                if wrap {
                    self.print("(");
                }
                self.add_source_mapping_for_name(i.span(self.ast).start, name);
                self.print_identifier(name);
                if wrap {
                    self.print(")");
                }
            }
            SimpleAssignmentTargetData::MemberExpression(m) => self.print_member(m, false),
            SimpleAssignmentTargetData::TsAsExpression(e) => {
                self.print_expr(Expr::TsAsExpression(e), Precedence::Postfix);
            }
            SimpleAssignmentTargetData::TsSatisfiesExpression(e) => {
                self.print_expr(Expr::TsSatisfiesExpression(e), Precedence::Postfix);
            }
            SimpleAssignmentTargetData::TsTypeAssertion(e) => {
                self.print_expr(Expr::TsTypeAssertion(e), Precedence::Postfix);
            }
            SimpleAssignmentTargetData::TsNonNullExpression(e) => self.print_ts_non_null(e, false),
        }
    }

    fn comment_has_line_terminator(&self, comment: Comment) -> bool {
        if comment.kind == CommentKind::Line {
            return true;
        }
        self.ast
            .get_utf8(comment.value(self.ast.source().as_bytes()))
            .chars()
            .any(|c| matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
    }

    fn has_leading_line_terminator_before(&self, offset: u32) -> bool {
        self.options.comments
            && self.comments[self.next_comment..]
                .iter()
                .take_while(|comment| comment.span.end <= offset)
                .any(|comment| self.comment_has_line_terminator(*comment))
    }

    fn simple_assignment_target_expression(&self, target: SimpleAssignmentTarget) -> Expr {
        match self.ast.simple_assignment_target_data(target) {
            SimpleAssignmentTargetData::IdentifierReference(expression) => {
                Expr::IdentifierReference(expression)
            }
            SimpleAssignmentTargetData::MemberExpression(expression) => {
                Expr::MemberExpression(expression)
            }
            SimpleAssignmentTargetData::TsAsExpression(expression) => {
                Expr::TsAsExpression(expression)
            }
            SimpleAssignmentTargetData::TsSatisfiesExpression(expression) => {
                Expr::TsSatisfiesExpression(expression)
            }
            SimpleAssignmentTargetData::TsTypeAssertion(expression) => {
                Expr::TsTypeAssertion(expression)
            }
            SimpleAssignmentTargetData::TsNonNullExpression(expression) => {
                Expr::TsNonNullExpression(expression)
            }
        }
    }

    fn expr_first_token_offset(&self, mut expr: Expr) -> u32 {
        loop {
            expr = match self.ast.expr_data(expr) {
                ExprData::MemberExpression(expression) => expression.object(self.ast),
                ExprData::CallExpression(expression) => expression.callee(self.ast),
                ExprData::ChainExpression(expression) => expression.expression(self.ast),
                ExprData::TaggedTemplateExpression(expression) => expression.tag(self.ast),
                ExprData::BinaryExpression(expression) => expression.left(self.ast),
                ExprData::LogicalExpression(expression) => expression.left(self.ast),
                ExprData::ConditionalExpression(expression) => expression.test(self.ast),
                ExprData::SequenceExpression(expression) => {
                    let Some(first) = expression.expressions(self.ast).get_node(self.ast, 0) else {
                        break expr.span(self.ast).start;
                    };
                    first
                }
                ExprData::AssignmentExpression(expression) => {
                    match self.ast.assignment_target_data(expression.left(self.ast)) {
                        AssignmentTargetData::SimpleAssignmentTarget(target) => {
                            self.simple_assignment_target_expression(target)
                        }
                        AssignmentTargetData::ArrayAssignmentTarget(pattern) => {
                            break pattern.span(self.ast).start;
                        }
                        AssignmentTargetData::ObjectAssignmentTarget(pattern) => {
                            break pattern.span(self.ast).start;
                        }
                    }
                }
                ExprData::UpdateExpression(expression) if !expression.prefix(self.ast) => {
                    self.simple_assignment_target_expression(expression.argument(self.ast))
                }
                ExprData::TsAsExpression(expression) => expression.expression(self.ast),
                ExprData::TsSatisfiesExpression(expression) => expression.expression(self.ast),
                ExprData::TsNonNullExpression(expression) => expression.expression(self.ast),
                ExprData::TsInstantiationExpression(expression) => expression.expression(self.ast),
                _ => break expr.span(self.ast).start,
            };
        }
    }

    // Only rightmost binary/logical operands carry the cast grouping context.
    // Skip consequent lookahead when it cannot affect the test.
    fn expr_ends_with_ts_cast(&self, mut expr: Expr) -> bool {
        loop {
            expr = match self.ast.expr_data(expr) {
                ExprData::BinaryExpression(expression) => expression.right(self.ast),
                ExprData::LogicalExpression(expression) => expression.right(self.ast),
                data => {
                    break matches!(
                        data,
                        ExprData::TsAsExpression(_) | ExprData::TsSatisfiesExpression(_)
                    );
                }
            };
        }
    }

    // TS6 and TS7 look at the token after `?` when parsing an `as` or
    // `satisfies` type. If it can start a type, `?` starts the conditional.
    // Otherwise, `?` becomes part of a nullable type (`T?`).
    // `x as T ? true : false` is safe because `true` can start a type.
    // A regexp cannot, so we need `(x as T) ? /a/ : false`.
    //
    // TS also looks inside parentheses, so `(/a/)` still needs a grouped cast.
    // `-` can start a type only when followed by a numeric or bigint literal.
    //
    // TS6 (6.0.3): type lookahead and nullable `?` parsing:
    // https://github.com/microsoft/TypeScript/blob/050880ce59e30b356b686bd3144efe24f875ebc8/src/compiler/parser.ts#L4659-L4750
    // TS7 (7.0.2): type lookahead:
    // https://github.com/microsoft/TypeScript/blob/1e4744d68260a7cb91b62b12edc3f6a2187faaf1/tsc/internal/parser/parser.go#L6210-L6242
    // TS7: nullable `?` parsing:
    // https://github.com/microsoft/TypeScript/blob/1e4744d68260a7cb91b62b12edc3f6a2187faaf1/tsc/internal/parser/parser.go#L2719-L2755
    fn expr_can_start_ts_type(&self, mut expr: Expr) -> bool {
        loop {
            expr = match self.ast.expr_data(expr) {
                ExprData::MemberExpression(expression) => expression.object(self.ast),
                ExprData::CallExpression(expression) => expression.callee(self.ast),
                ExprData::ChainExpression(expression) => expression.expression(self.ast),
                ExprData::TaggedTemplateExpression(expression) => expression.tag(self.ast),
                ExprData::BinaryExpression(expression) => expression.left(self.ast),
                ExprData::LogicalExpression(expression) => expression.left(self.ast),
                ExprData::ConditionalExpression(expression) => expression.test(self.ast),
                ExprData::SequenceExpression(expression) => {
                    let Some(first) = expression.expressions(self.ast).get_node(self.ast, 0) else {
                        break true;
                    };
                    first
                }
                ExprData::AssignmentExpression(expression) => {
                    match self.ast.assignment_target_data(expression.left(self.ast)) {
                        AssignmentTargetData::SimpleAssignmentTarget(target) => {
                            self.simple_assignment_target_expression(target)
                        }
                        AssignmentTargetData::ArrayAssignmentTarget(_)
                        | AssignmentTargetData::ObjectAssignmentTarget(_) => break true,
                    }
                }
                ExprData::UpdateExpression(expression) if !expression.prefix(self.ast) => {
                    self.simple_assignment_target_expression(expression.argument(self.ast))
                }
                ExprData::TsAsExpression(expression) => expression.expression(self.ast),
                ExprData::TsSatisfiesExpression(expression) => expression.expression(self.ast),
                ExprData::TsNonNullExpression(expression) => expression.expression(self.ast),
                ExprData::TsInstantiationExpression(expression) => expression.expression(self.ast),
                ExprData::ParenthesizedExpression(expression) => expression.expression(self.ast),
                ExprData::RegExpLiteral(_)
                | ExprData::Class(_)
                | ExprData::Super(_)
                | ExprData::UpdateExpression(_) => break false,
                ExprData::UnaryExpression(expression) => {
                    break match expression.operator(self.ast) {
                        UnaryOperator::LogicalNot | UnaryOperator::Typeof | UnaryOperator::Void => {
                            true
                        }
                        UnaryOperator::Negate => {
                            let argument = expression.argument(self.ast);
                            matches!(
                                self.ast.expr_data(argument),
                                ExprData::NumericLiteral(_) | ExprData::BigIntLiteral(_)
                            )
                        }
                        _ => false,
                    };
                }
                _ => break true,
            };
        }
    }

    fn expr_has_leading_line_terminator(&self, expr: Expr) -> bool {
        // Locate the first token before scanning comments. Scanning at every
        // node on the left spine would make deep expressions quadratic.
        let offset = self.expr_first_token_offset(expr);
        self.has_leading_line_terminator_before(offset)
    }

    fn print_expr_with_leading_comments_in_parens(&mut self, expr: Expr, level: Precedence) {
        self.print("(");
        self.print_newline();
        self.indent += 1;
        self.print_comments_before(expr.span(self.ast).start, true, true);
        self.print_indent();
        self.print_expr(expr, level);
        self.print_newline();
        self.indent -= 1;
        self.print_indent();
        self.print(")");
    }

    fn print_expr_without_leading_line_terminator(&mut self, expr: Expr, level: Precedence) {
        if self.expr_has_leading_line_terminator(expr) {
            self.print_expr_with_leading_comments_in_parens(expr, level);
        } else {
            self.print_expr(expr, level);
        }
    }

    fn print_expr(&mut self, expr: Expr, level: Precedence) {
        self.print_expr_with_context(expr, level, ExprContext::empty());
    }

    fn print_expr_leading_comments(&mut self, expr: Expr) {
        let is_at_stmt_start = self.js.len() == self.stmt_start;
        let is_at_export_default_start = self.js.len() == self.export_default_start;
        self.print_comments_before(expr.span(self.ast).start, false, true);
        if is_at_stmt_start {
            self.stmt_start = self.js.len();
        }
        if is_at_export_default_start {
            self.export_default_start = self.js.len();
        }
    }

    fn print_expr_with_context(&mut self, expr: Expr, level: Precedence, context: ExprContext) {
        self.print_expr_leading_comments(expr);
        match self.ast.expr_data(expr) {
            ExprData::IdentifierReference(e) => {
                let name = self.ast.get_utf8(e.name(self.ast));
                let wrap = self.js.len() == self.for_of_init_start && name == "let";
                if wrap {
                    self.print("(");
                }
                self.add_source_mapping_for_name(expr.span(self.ast).start, name);
                self.print_identifier(name);
                if wrap {
                    self.print(")");
                }
            }
            ExprData::ThisExpression(_) => {
                self.print_space_before_identifier();
                self.add_source_mapping(expr.span(self.ast).start);
                self.print("this");
            }
            ExprData::Super(_) => {
                self.print_space_before_identifier();
                self.add_source_mapping(expr.span(self.ast).start);
                self.print("super");
            }
            ExprData::MetaProperty(e) => {
                self.add_source_mapping(expr.span(self.ast).start);
                self.print_identifier(self.ast.get_utf8(e.meta(self.ast).name(self.ast)));
                self.print(".");
                self.print_identifier(self.ast.get_utf8(e.property(self.ast).name(self.ast)));
            }
            ExprData::StringLiteral(e) => {
                self.add_source_mapping(expr.span(self.ast).start);
                self.print_quoted(self.ast.get_wtf8(e.value(self.ast)), true)
            }
            ExprData::NumericLiteral(e) => {
                self.add_source_mapping(expr.span(self.ast).start);
                self.print_number_expr(e, level)
            }
            ExprData::BigIntLiteral(e) => {
                self.print_space_before_identifier();
                self.add_source_mapping(expr.span(self.ast).start);
                self.print_bigint_literal(e);
            }
            ExprData::BooleanLiteral(e) => {
                self.print_space_before_identifier();
                self.add_source_mapping(expr.span(self.ast).start);
                self.print(if e.value(self.ast) { "true" } else { "false" });
            }
            ExprData::NullLiteral(_) => {
                self.print_space_before_identifier();
                self.add_source_mapping(expr.span(self.ast).start);
                self.print("null");
            }
            ExprData::RegExpLiteral(e) => {
                // Avoid forming a single-line comment or a "</script"
                // sequence across the token boundary.
                if let Some(&last) = self.js.last()
                    && (last == b'/'
                        || (self.options.inline_script
                            && last == b'<'
                            && self
                                .ast
                                .get_utf8(e.pattern(self.ast))
                                .get(..6)
                                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("script"))))
                {
                    self.print(" ");
                }
                self.add_source_mapping(expr.span(self.ast).start);
                self.print("/");
                self.print_bytes(self.ast.get_utf8(e.pattern(self.ast)));
                self.print("/");
                self.print_bytes(self.ast.get_utf8(e.flags(self.ast)));

                // The next identifier needs a separator or it would become
                // part of the regular expression flags.
                self.prev_reg_exp_end = self.js.len();
            }
            ExprData::TemplateLiteral(e) => self.print_template(e, false),
            ExprData::BinaryExpression(e) => {
                self.print_binaryish(Binaryish::Binary(e), level, context)
            }
            ExprData::LogicalExpression(e) => {
                self.print_binaryish(Binaryish::Logical(e), level, context)
            }
            ExprData::UnaryExpression(e) => {
                // Select the operand mode before printing to shorten the context live range.
                if context.contains(ExprContext::NO_INSTANTIATION) {
                    self.print_unary::<true>(e, level);
                } else {
                    self.print_unary::<false>(e, level);
                }
            }
            ExprData::UpdateExpression(e) => self.print_update(e, level),
            ExprData::AssignmentExpression(e) => {
                let at_stmt_start = self.js.len() == self.stmt_start;
                let at_arrow_start = self.js.len() == self.arrow_expr_start;
                let destructuring_at_start = (at_stmt_start || at_arrow_start)
                    && match self.ast.assignment_target_data(e.left(self.ast)) {
                        AssignmentTargetData::ObjectAssignmentTarget(_) => true,
                        AssignmentTargetData::ArrayAssignmentTarget(_) => false,
                        AssignmentTargetData::SimpleAssignmentTarget(_) => false,
                    };
                let wrap = level > Precedence::Assignment || destructuring_at_start;
                if wrap {
                    self.print("(");
                }
                self.print_assignment_target(e.left(self.ast));
                self.print_space();
                let operator = e.operator(self.ast);
                self.print_operator(OperatorCode::Assignment, operator.as_str());
                self.print_space();
                self.print_expr(e.right(self.ast), Precedence::Assignment);
                if wrap {
                    self.print(")");
                }
            }
            ExprData::ConditionalExpression(e) => {
                let old_forbid_in = self.forbid_in;
                let wrap = level > Precedence::Conditional;
                if wrap {
                    self.print("(");
                    self.forbid_in = false;
                }
                // TypeScript consumes `T ?` as a nullable type unless the next
                // expression can start a type. Group casts to end the type first.
                let test = e.test(self.ast);
                let consequent = e.consequent(self.ast);
                let mut test_context = ExprContext::empty();
                test_context.set(
                    ExprContext::WRAP_TS_CAST,
                    self.ast.is_ts()
                        && self.expr_ends_with_ts_cast(test)
                        && !self.expr_can_start_ts_type(consequent),
                );
                self.print_expr_with_context(test, Precedence::Conditional.next(), test_context);
                self.print_space();
                self.print("?");
                self.print_space();
                self.forbid_in = false;
                self.print_expr(consequent, Precedence::Yield);
                self.print_space();
                self.print(":");
                self.print_space();
                self.forbid_in = old_forbid_in && !wrap;
                self.print_expr(e.alternate(self.ast), Precedence::Yield);
                if wrap {
                    self.print(")");
                }
                self.forbid_in = old_forbid_in;
            }
            ExprData::SequenceExpression(e) => {
                let wrap = level >= Precedence::Comma;
                if wrap {
                    self.print("(");
                }
                for (i, id) in e.expressions(self.ast).iter().enumerate() {
                    if i != 0 {
                        self.print(",");
                        self.print_space();
                    }
                    let member_level = if self.options.minify {
                        Precedence::Lowest
                    } else {
                        Precedence::Assignment
                    };
                    self.print_expr(self.ast.get_node_in_sub_range(id), member_level);
                }
                if wrap {
                    self.print(")");
                }
            }
            ExprData::MemberExpression(e) => {
                self.print_member(e, context.contains(ExprContext::IN_NEW_CALLEE))
            }
            ExprData::CallExpression(e) => {
                self.print_call(e, level, context.contains(ExprContext::IN_NEW_CALLEE))
            }
            ExprData::ChainExpression(e) => {
                let wrap =
                    level >= Precedence::Call || context.contains(ExprContext::IN_NEW_CALLEE);
                if wrap {
                    self.print("(");
                }
                let chain_context = context.instantiation_tail(wrap);
                self.print_expr_with_context(
                    e.expression(self.ast),
                    Precedence::Lowest,
                    chain_context,
                );
                if wrap {
                    self.print(")");
                }
            }
            ExprData::NewExpression(e) => self.print_new(e, level),
            ExprData::TaggedTemplateExpression(e) => {
                let tag = e.tag(self.ast);
                let type_arguments = e.type_arguments(self.ast);
                let tag_level = if type_arguments.is_some() {
                    Precedence::Call
                } else {
                    Precedence::Postfix
                };
                if matches!(self.ast.expr_data(tag), ExprData::ChainExpression(_)) {
                    self.print("(");
                    self.print_expr(tag, Precedence::Lowest);
                    self.print(")");
                } else {
                    let tag_context =
                        ExprContext::NO_INSTANTIATION | (context & ExprContext::IN_NEW_CALLEE);
                    self.print_expr_with_context(tag, tag_level, tag_context);
                }
                if let Some(args) = type_arguments {
                    self.print_ts_type_args(args);
                }
                self.print_template(e.quasi(self.ast), true);
            }
            ExprData::ArrowFunctionExpression(e) => self.print_arrow(e, level),
            ExprData::ArrayExpression(e) => self.print_array(e),
            ExprData::ObjectExpression(e) => self.print_object(e, level),
            ExprData::ParenthesizedExpression(e) => {
                self.print("(");
                self.print_expr(e.expression(self.ast), Precedence::Lowest);
                self.print_comments_before(e.span(self.ast).end, false, true);
                self.print(")");
            }
            ExprData::Function(e) => self.print_function(e),
            ExprData::Class(e) => self.print_class(e, false),
            ExprData::ImportExpression(e) => {
                if context.contains(ExprContext::IN_NEW_CALLEE) {
                    self.print("(");
                }
                let old_forbid_in = self.forbid_in;
                self.forbid_in = false;
                self.print_space_before_identifier();
                self.add_source_mapping(expr.span(self.ast).start);
                if e.sync(self.ast) {
                    self.print("import.sync(");
                } else {
                    match e.phase(self.ast) {
                        Some(ImportPhase::Source) => self.print("import.source("),
                        Some(ImportPhase::Defer) => self.print("import.defer("),
                        None => self.print("import("),
                    }
                }
                let close_offset = e.span(self.ast).end.saturating_sub(1);
                let source_span = e.source(self.ast).span(self.ast);
                let options_span = e.options(self.ast).map(|options| options.span(self.ast));
                let multiline = !self.options.minify
                    && self.comments[self.next_comment..]
                        .iter()
                        .take_while(|comment| comment.span.end <= close_offset)
                        .any(|comment| {
                            !(source_span.start <= comment.span.start
                                && comment.span.end <= source_span.end
                                || options_span.is_some_and(|span| {
                                    span.start <= comment.span.start && comment.span.end <= span.end
                                }))
                        });
                if multiline {
                    self.indent += 1;
                    self.print_newline();
                    let source = e.source(self.ast);
                    self.print_comments_before(source.span(self.ast).start, true, true);
                    self.print_indent();
                    self.print_expr(source, Precedence::Assignment);
                    if let Some(options) = e.options(self.ast) {
                        self.print(",");
                        self.print_newline();
                        self.print_comments_before(options.span(self.ast).start, true, true);
                        self.print_indent();
                        self.print_expr(options, Precedence::Assignment);
                    }
                    self.print_newline();
                    self.discard_comments_before_trailing_comma(close_offset);
                    self.print_comments_before(close_offset, true, true);
                    self.indent -= 1;
                    self.print_indent();
                } else {
                    self.print_expr(e.source(self.ast), Precedence::Assignment);
                    if let Some(options) = e.options(self.ast) {
                        self.print(",");
                        self.print_space();
                        self.print_expr(options, Precedence::Assignment);
                    }
                }
                self.print(")");
                self.forbid_in = old_forbid_in;
                if context.contains(ExprContext::IN_NEW_CALLEE) {
                    self.print(")");
                }
            }
            ExprData::AwaitExpression(e) => self.print_await(e, level, context),
            ExprData::YieldExpression(e) => {
                let wrap = level > Precedence::Assignment;
                if wrap {
                    self.print("(");
                }
                self.add_source_mapping(expr.span(self.ast).start);
                self.print("yield");
                if e.delegate(self.ast) {
                    self.print("*");
                }
                if let Some(value) = e.argument(self.ast) {
                    self.print(" ");
                    self.print_expr_without_leading_line_terminator(value, Precedence::Yield);
                }
                if wrap {
                    self.print(")");
                }
            }
            ExprData::PrivateIdentifier(e) => {
                let name = self.ast.get_utf8(e.name(self.ast));
                self.add_source_mapping_for_name(expr.span(self.ast).start, name);
                self.print("#");
                self.print_identifier(name);
            }
            ExprData::TsAsExpression(e) => {
                let wrap =
                    level > Precedence::Compare || context.contains(ExprContext::WRAP_TS_CAST);
                if wrap {
                    self.print("(");
                }
                self.print_expr(e.expression(self.ast), Precedence::Compare);
                self.print(" as ");
                self.print_ts_type(e.type_annotation(self.ast), TsTypePrecedence::Lowest);
                if wrap {
                    self.print(")");
                }
            }
            ExprData::TsSatisfiesExpression(e) => {
                let wrap =
                    level > Precedence::Compare || context.contains(ExprContext::WRAP_TS_CAST);
                if wrap {
                    self.print("(");
                }
                self.print_expr(e.expression(self.ast), Precedence::Compare);
                self.print(" satisfies ");
                self.print_ts_type(e.type_annotation(self.ast), TsTypePrecedence::Lowest);
                if wrap {
                    self.print(")");
                }
            }
            ExprData::TsTypeAssertion(e) => self.print_ts_type_assertion(e, level, context),
            ExprData::TsNonNullExpression(e) => {
                self.print_ts_non_null(e, context.contains(ExprContext::IN_NEW_CALLEE))
            }
            ExprData::TsInstantiationExpression(e) => {
                let wrap =
                    level > Precedence::Call || context.contains(ExprContext::NO_INSTANTIATION);
                if wrap {
                    self.print("(");
                }
                let mut operand_context =
                    ExprContext::NO_INSTANTIATION | (context & ExprContext::IN_NEW_CALLEE);
                if wrap {
                    operand_context.remove(ExprContext::IN_NEW_CALLEE);
                }
                self.print_expr_with_context(
                    e.expression(self.ast),
                    Precedence::Call,
                    operand_context,
                );
                self.print_ts_type_args(e.type_arguments(self.ast));
                if wrap {
                    self.print(")");
                }
            }
            ExprData::JsxElement(e) => self.print_jsx_element(e),
            ExprData::JsxFragment(e) => self.print_jsx_fragment(e),
        }
    }

    fn print_ts_non_null(&mut self, expression: TsNonNullExpression, in_new_callee: bool) {
        let operand = expression.expression(self.ast);
        let level = if matches!(self.ast.expr_data(operand), ExprData::ChainExpression(_)) {
            Precedence::Call
        } else {
            Precedence::Postfix
        };
        let mut context = ExprContext::NO_INSTANTIATION;
        context.set(ExprContext::IN_NEW_CALLEE, in_new_callee);
        self.print_expr_with_context(operand, level, context);
        self.print_operator(OperatorCode::TsNonNull, "!");
    }

    fn print_binaryish(&mut self, e: Binaryish, level: Precedence, context: ExprContext) {
        // Binary-expression printing is iterative instead of recursive to
        // avoid overflowing the call stack for deeply nested ASTs.
        let operator = e.operator(self.ast);
        let mut visitor = BinaryExprVisitor {
            e,
            level,
            flags: self.forbid_in,
            context,
            left_level: Precedence::Lowest,
            left_flags: false,
            left_context: ExprContext::empty(),
            operator,
            wrap: false,
            right_level: Precedence::Lowest,
            right_context: ExprContext::empty(),
        };

        // Use a single stack to reduce allocation overhead.
        let stack_bottom = self.expr_visitor_stack.len();

        loop {
            visitor.prepare(self);

            let left = visitor.e.left(self.ast);
            let left_binaryish = match self.ast.expr_data(left) {
                ExprData::BinaryExpression(e) => Some(Binaryish::Binary(e)),
                ExprData::LogicalExpression(e) => Some(Binaryish::Logical(e)),
                _ => None,
            };
            let Some(left_binaryish) = left_binaryish else {
                self.print_expr_with_context(left, visitor.left_level, visitor.left_context);
                visitor.visit_right_and_finish(self);
                break;
            };

            self.print_expr_leading_comments(left);

            // Only allocate heap memory on the stack for nested binary-ish
            // expressions.
            self.expr_visitor_stack.push(visitor);
            visitor = BinaryExprVisitor {
                e: left_binaryish,
                level: visitor.left_level,
                flags: visitor.left_flags,
                context: visitor.left_context,
                left_level: Precedence::Lowest,
                left_flags: false,
                left_context: ExprContext::empty(),
                operator: visitor.operator,
                wrap: false,
                right_level: Precedence::Lowest,
                right_context: ExprContext::empty(),
            };
        }

        // Process all binary operations from the deepest-visited node back
        // toward our original top-level binary operation.
        while self.expr_visitor_stack.len() > stack_bottom {
            let visitor = self.expr_visitor_stack.pop().unwrap();
            visitor.visit_right_and_finish(self);
        }
    }

    // Keep prefix operand contexts outside the main expression dispatch.
    #[inline(never)]
    fn print_await(&mut self, e: AwaitExpression, level: Precedence, context: ExprContext) {
        let wrap = level > Precedence::Prefix;
        if wrap {
            self.print("(");
        }
        self.print_space_before_identifier();
        self.add_source_mapping(e.span(self.ast).start);
        self.print("await");
        self.print_space();
        let operand_context = context.instantiation_tail(wrap);
        self.print_expr_with_context(e.argument(self.ast), Precedence::Prefix, operand_context);
        if wrap {
            self.print(")");
        }
    }

    #[inline(never)]
    fn print_ts_type_assertion(
        &mut self,
        e: TsTypeAssertion,
        level: Precedence,
        context: ExprContext,
    ) {
        let wrap = level > Precedence::Prefix;
        if wrap {
            self.print("(");
        }
        self.print_ts_type_assertion_prefix(e.type_annotation(self.ast), e.span(self.ast).start);
        let operand_context = context.instantiation_tail(wrap);
        self.print_expr_with_context(e.expression(self.ast), Precedence::Prefix, operand_context);
        if wrap {
            self.print(")");
        }
    }

    fn print_unary<const NO_INSTANTIATION: bool>(&mut self, e: UnaryExpression, level: Precedence) {
        let wrap = level > Precedence::Prefix;
        if wrap {
            self.print("(");
        }
        let operator = e.operator(self.ast);
        let op = operator.as_str();
        if matches!(
            operator,
            UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::Delete
        ) {
            self.print_space_before_identifier();
            self.add_source_mapping(e.span(self.ast).start);
            self.print(op);
            self.print(" ");
        } else {
            let operator_code = OperatorCode::Unary(operator);
            self.print_mapped_operator(operator_code, op, e.span(self.ast).start);
        }
        let operand_context = if NO_INSTANTIATION && !wrap {
            ExprContext::NO_INSTANTIATION
        } else {
            ExprContext::empty()
        };
        self.print_expr_with_context(e.argument(self.ast), Precedence::Prefix, operand_context);
        if wrap {
            self.print(")");
        }
    }

    fn print_update(&mut self, e: UpdateExpression, level: Precedence) {
        let wrap = level >= Precedence::Postfix;
        if wrap {
            self.print("(");
        }
        let operator = e.operator(self.ast);
        let op = operator.as_str();
        let operator_code = OperatorCode::Update(operator, e.prefix(self.ast));
        if e.prefix(self.ast) {
            self.print_mapped_operator(operator_code, op, e.span(self.ast).start);
            self.print_simple_assignment_target(e.argument(self.ast));
        } else {
            self.print_simple_assignment_target(e.argument(self.ast));
            self.print_mapped_operator(operator_code, op, e.span(self.ast).end.saturating_sub(2));
        }
        if wrap {
            self.print(")");
        }
    }

    fn print_member(&mut self, e: MemberExpression, in_new_callee: bool) {
        let object = e.object(self.ast);
        let object_data = self.ast.expr_data(object);
        let computed = e.computed(self.ast);
        let optional = e.optional(self.ast);
        // `let[0]` is ambiguous at the start of a statement because it can
        // begin a lexical declaration. A static member such as `let.foo`
        // doesn't have this ambiguity.
        let wrap_object = computed
            && matches!(
                object_data,
                ExprData::IdentifierReference(identifier)
                    if self.ast.get_utf8(identifier.name(self.ast)) == "let"
            );
        if wrap_object {
            self.print("(");
        }
        let mut context = ExprContext::empty();
        context.set(ExprContext::IN_NEW_CALLEE, in_new_callee && !wrap_object);
        context.set(ExprContext::NO_INSTANTIATION, !optional || !computed);
        self.print_expr_with_context(
            object,
            if matches!(object_data, ExprData::NewExpression(_)) {
                Precedence::New
            } else {
                Precedence::Call
            },
            context,
        );
        if wrap_object {
            self.print(")");
        }
        let need_space_before_dot = self.need_space_before_dot == self.js.len();
        self.need_space_before_dot = usize::MAX;
        let property = e.property(self.ast);
        let property_data = self.ast.property_key_data(property);
        let (identifier_text, property_text): (Option<&str>, Option<&Wtf8>) = match property_data {
            PropertyKeyData::IdentifierName(identifier) => {
                let name = self.ast.get_utf8(identifier.name(self.ast));
                (Some(name), Some(Wtf8::from_str(name)))
            }
            PropertyKeyData::Expr(expression) => match self.ast.expr_data(expression) {
                ExprData::StringLiteral(string) => {
                    (None, Some(self.ast.get_wtf8(string.value(self.ast))))
                }
                _ => (None, None),
            },
        };
        let force_computed = !computed
            && match identifier_text {
                Some(name) => !self.can_print_identifier(name),
                None => property_text.is_some_and(|name| {
                    name.as_str()
                        .is_none_or(|name| !self.can_print_identifier(name))
                }),
            };
        if optional {
            self.print("?.");
        }
        if computed || force_computed {
            self.print("[");
            if force_computed {
                self.add_source_mapping(property.span(self.ast).start);
                self.print_quoted(property_text.unwrap(), true);
            } else {
                let old_forbid_in = self.forbid_in;
                self.forbid_in = false;
                self.print_property_key_expr(property, Precedence::Lowest);
                self.forbid_in = old_forbid_in;
                self.add_source_mapping(e.span(self.ast).end.saturating_sub(1));
            }
            self.print("]");
        } else {
            if !optional {
                if need_space_before_dot {
                    // "1.toString" is a syntax error, so print
                    // "1 .toString" instead.
                    self.print(" ");
                }
                self.print(".");
            }
            self.add_source_mapping(property.span(self.ast).start);
            if matches!(property_data, PropertyKeyData::IdentifierName(_)) {
                self.print_identifier(identifier_text.unwrap());
            } else {
                self.print_property_key_plain_data(property, property_data);
            }
        }
    }

    fn print_call(&mut self, e: CallExpression, level: Precedence, in_new_callee: bool) {
        let wrap = level > Precedence::Call || in_new_callee;
        if wrap {
            self.print("(");
        }
        let callee = e.callee(self.ast);
        let optional = e.optional(self.ast);
        let callee_level = if matches!(self.ast.expr_data(callee), ExprData::NewExpression(_)) {
            Precedence::New
        } else {
            Precedence::Call
        };
        let type_arguments = e.type_arguments(self.ast);
        let mut context = ExprContext::empty();
        context.set(ExprContext::NO_INSTANTIATION, !optional);
        self.print_expr_with_context(callee, callee_level, context);
        if optional {
            self.print("?.");
        }
        if let Some(args) = type_arguments {
            self.print_ts_type_args(args);
        }
        self.print_arguments(
            e.arguments(self.ast),
            e.span(self.ast).end.saturating_sub(1),
        );
        if wrap {
            self.print(")");
        }
    }

    fn print_new(&mut self, e: NewExpression, level: Precedence) {
        let wrap = level > Precedence::New;
        if wrap {
            self.print("(");
        }
        self.print_space_before_identifier();
        self.add_source_mapping(e.span(self.ast).start);
        self.print("new");
        self.print_space();
        let callee = e.callee(self.ast);
        let type_arguments = e.type_arguments(self.ast);
        self.print_expr_with_context(
            callee,
            Precedence::New,
            ExprContext::NO_INSTANTIATION | ExprContext::IN_NEW_CALLEE,
        );
        if let Some(args) = type_arguments {
            self.print_ts_type_args(args);
        }
        let arguments = e.arguments(self.ast);
        // Generic constructors need `()` to disambiguate following operators.
        if !self.options.minify
            || !arguments.is_empty()
            || type_arguments.is_some()
            || level >= Precedence::Postfix
        {
            self.print_arguments(arguments, e.span(self.ast).end.saturating_sub(1));
        }
        if wrap {
            self.print(")");
        }
    }

    fn print_arguments(&mut self, args: SubRange<Argument>, close_offset: u32) {
        let old_forbid_in = self.forbid_in;
        self.forbid_in = false;
        self.print("(");
        let multiline = !self.options.minify
            && !args.is_empty()
            && self.has_direct_argument_comments(args, close_offset);
        if multiline {
            self.indent += 1;
            self.print_newline();
        }
        for (i, id) in args.iter().enumerate() {
            if i != 0 {
                self.print(",");
                if multiline {
                    self.print_newline();
                } else {
                    self.print_space();
                }
            }
            let argument = self.ast.get_node_in_sub_range(id);
            if multiline {
                self.print_comments_before(argument.span(self.ast).start, true, true);
                self.print_indent();
            }
            match self.ast.argument_data(argument) {
                ArgumentData::Expr(e) => self.print_expr(e, Precedence::Assignment),
                ArgumentData::SpreadElement(s) => {
                    self.print("...");
                    self.print_expr(s.argument(self.ast), Precedence::Assignment);
                }
            }
        }
        if multiline {
            self.print_newline();
            self.discard_comments_before_trailing_comma(close_offset);
            self.print_comments_before(close_offset, true, true);
            self.indent -= 1;
            self.print_indent();
        } else if args.is_empty() || self.options.minify {
            self.print_comments_before(close_offset, false, true);
        }
        self.add_source_mapping(close_offset);
        self.print(")");
        self.forbid_in = old_forbid_in;
    }

    fn print_array(&mut self, e: ArrayExpression) {
        let old_forbid_in = self.forbid_in;
        self.forbid_in = false;
        self.add_source_mapping(e.span(self.ast).start);
        self.print("[");
        let elements = e.elements(self.ast);
        for (i, id) in elements.iter().enumerate() {
            if i != 0 {
                self.print(",");
                self.print_space();
            }
            if let Some(arg) = self.ast.get_node_in_sub_range(id) {
                match self.ast.argument_data(arg) {
                    ArgumentData::Expr(e) => self.print_expr(e, Precedence::Assignment),
                    ArgumentData::SpreadElement(s) => {
                        self.print("...");
                        self.print_expr(s.argument(self.ast), Precedence::Assignment);
                    }
                }
            }
        }
        if !elements.is_empty()
            && self
                .ast
                .get_node_in_sub_range(elements.last().unwrap())
                .is_none()
        {
            self.print(",");
        }
        let close_offset = e.span(self.ast).end.saturating_sub(1);
        self.print_comments_before(close_offset, false, true);
        self.add_source_mapping(close_offset);
        self.print("]");
        self.forbid_in = old_forbid_in;
    }

    fn print_object(&mut self, e: ObjectExpression, _level: Precedence) {
        let old_forbid_in = self.forbid_in;
        self.forbid_in = false;
        let wrap = self.js.len() == self.stmt_start || self.js.len() == self.arrow_expr_start;
        if wrap {
            self.print("(");
        }
        self.add_source_mapping(e.span(self.ast).start);
        self.print("{");
        let props = e.properties(self.ast);
        if !props.is_empty() {
            self.print_space();
        }
        for (i, id) in props.iter().enumerate() {
            if i != 0 {
                self.print(",");
                self.print_space();
            }
            match self
                .ast
                .object_property_kind_data(self.ast.get_node_in_sub_range(id))
            {
                ObjectPropertyKindData::SpreadElement(s) => {
                    self.print("...");
                    self.print_expr(s.argument(self.ast), Precedence::Assignment);
                }
                ObjectPropertyKindData::ObjectProperty(p) => self.print_object_property(p),
            }
        }
        let close_offset = e.span(self.ast).end.saturating_sub(1);
        if self
            .comments
            .get(self.next_comment)
            .is_some_and(|comment| comment.span.end <= close_offset)
            && let Some(id) = props.last()
        {
            match self
                .ast
                .object_property_kind_data(self.ast.get_node_in_sub_range(id))
            {
                ObjectPropertyKindData::SpreadElement(s) => {
                    self.print_attached_trailing_comments(s.argument(self.ast).span(self.ast).end);
                }
                ObjectPropertyKindData::ObjectProperty(p)
                    if !p.method(self.ast)
                        && p.kind(self.ast) == PropertyKind::Init
                        && !self.can_use_shorthand_property(p) =>
                {
                    self.print_attached_trailing_comments(p.value(self.ast).span(self.ast).end);
                }
                _ => {}
            }
        }
        self.print_comments_before(close_offset, false, true);
        if !props.is_empty() {
            self.print_space();
        }
        self.add_source_mapping(close_offset);
        self.print("}");
        if wrap {
            self.print(")");
        }
        self.forbid_in = old_forbid_in;
    }

    fn print_object_property(&mut self, p: ObjectProperty) {
        let kind = p.kind(self.ast);
        match kind {
            PropertyKind::Get => self.print("get "),
            PropertyKind::Set => self.print("set "),
            PropertyKind::Init => {}
        }
        if (p.method(self.ast) || kind != PropertyKind::Init)
            && let ExprData::Function(f) = self.ast.expr_data(p.value(self.ast))
        {
            if f.r#async(self.ast) {
                self.print("async ");
            }
            if f.generator(self.ast) {
                self.print("*");
            }
            self.print_property_key(p.key(self.ast), p.computed(self.ast));
            if let Some(t) = f.type_parameters(self.ast) {
                self.print_ts_type_params(t);
            }
            self.print_formal_parameters(f.params(self.ast), f.this_param(self.ast));
            if let Some(t) = f.return_type(self.ast) {
                self.print_ts_annotation(t);
            }
            self.print_space();
            let body = f.body(self.ast);
            self.print_function_body(body);
            if matches!(kind, PropertyKind::Get | PropertyKind::Set) {
                self.print_attached_trailing_comments(body.span(self.ast).end);
            }
            return;
        }
        self.print_property_key(p.key(self.ast), p.computed(self.ast));
        if !self.can_use_shorthand_property(p) {
            self.print(":");
            self.print_space();
            self.print_expr(p.value(self.ast), Precedence::Assignment);
        }
    }

    #[inline(always)]
    fn can_use_shorthand_property(&self, property: ObjectProperty) -> bool {
        property.shorthand(self.ast)
            && !property.computed(self.ast)
            && match self.ast.property_key_data(property.key(self.ast)) {
                PropertyKeyData::IdentifierName(identifier) => {
                    self.can_print_identifier(self.ast.get_utf8(identifier.name(self.ast)))
                }
                _ => false,
            }
    }

    fn print_property_key(&mut self, key: PropertyKey, computed: bool) {
        self.add_source_mapping(key.span(self.ast).start);
        if computed {
            self.print("[");
            self.print_comments_before(key.span(self.ast).start, false, true);
            self.print_property_key_expr(key, Precedence::Assignment);
            self.print("]");
        } else {
            self.print_property_key_plain(key);
        }
    }

    fn print_property_key_expr(&mut self, key: PropertyKey, level: Precedence) {
        match self.ast.property_key_data(key) {
            PropertyKeyData::IdentifierName(e) => {
                self.print_identifier(self.ast.get_utf8(e.name(self.ast)))
            }
            // Literal and private keys must not consume expression-leading comments
            // after a property's modifiers have already been printed.
            PropertyKeyData::Expr(expression) => match self.ast.expr_data(expression) {
                ExprData::PrivateIdentifier(e) => {
                    self.print("#");
                    self.print_identifier(self.ast.get_utf8(e.name(self.ast)));
                }
                ExprData::StringLiteral(e) => {
                    self.print_quoted(self.ast.get_wtf8(e.value(self.ast)), true)
                }
                ExprData::NumericLiteral(e) => {
                    self.print_number_expr(e, Precedence::Lowest);
                }
                ExprData::BigIntLiteral(e) => {
                    self.print_bigint_literal(e);
                }
                _ => self.print_expr(expression, level),
            },
        }
    }

    #[inline(always)]
    fn print_property_key_plain(&mut self, key: PropertyKey) {
        self.print_property_key_plain_data(key, self.ast.property_key_data(key));
    }

    #[inline(always)]
    fn print_property_key_plain_data(&mut self, key: PropertyKey, data: PropertyKeyData) {
        match data {
            PropertyKeyData::IdentifierName(value) => {
                let name = self.ast.get_utf8(value.name(self.ast));
                if self.can_print_identifier(name) {
                    self.print_identifier(name);
                } else {
                    self.print_quoted(Wtf8::from_str(name), false);
                }
            }
            PropertyKeyData::Expr(expression) => match self.ast.expr_data(expression) {
                ExprData::StringLiteral(value) => {
                    self.print_quoted(self.ast.get_wtf8(value.value(self.ast)), false);
                }
                ExprData::NumericLiteral(value) => self.print_number_property_key(value),
                _ => self.print_property_key_expr(key, Precedence::Assignment),
            },
        }
    }

    fn print_number_property_key(&mut self, literal: NumericLiteral) {
        let value = literal.value(self.ast);
        let has_source_spelling = literal.raw(self.ast).is_some();
        let requires_string = value.is_sign_negative() || !value.is_finite();
        if !has_source_spelling && requires_string {
            let mut buffer = zmij_ecma::Buffer::new();
            let property_name = buffer.format(value);
            self.print_quoted(Wtf8::from_str(property_name), false);
            return;
        }

        self.print_number_literal(literal, Precedence::Lowest);
    }

    fn print_template(&mut self, e: TemplateLiteral, is_tagged: bool) {
        if !is_tagged {
            self.add_source_mapping(e.span(self.ast).start);
        }
        self.print("`");
        let quasis = e.quasis(self.ast);
        let exprs = e.expressions(self.ast);
        for (i, id) in quasis.iter().enumerate() {
            let quasi = self.ast.get_node_in_sub_range(id);
            let raw = self.ast.get_utf8(quasi.raw(self.ast));
            if is_tagged {
                self.print_bytes(raw);
            } else if !self.options.minify && !self.options.ascii_only && !raw.is_empty() {
                self.print_script_text(raw.as_bytes());
            } else {
                let cooked = if quasi.is_cooked_undefined(self.ast) {
                    Wtf8::from_str(self.ast.get_utf8(quasi.raw(self.ast)))
                } else {
                    self.ast.get_wtf8(quasi.cooked(self.ast))
                };
                let start = self.js.len();
                quote_into(
                    &mut self.js,
                    cooked,
                    b'`',
                    self.options.ascii_only,
                    self.options.inline_script,
                );
                self.js.remove(start);
                self.js.pop();
            }
            if i < exprs.len() {
                self.print("${");
                self.print_expr(exprs.get_node(self.ast, i).unwrap(), Precedence::Lowest);
                let tail = quasis.get_node(self.ast, i + 1).unwrap();
                self.add_source_mapping(tail.span(self.ast).start.saturating_sub(1));
                self.print("}");
            }
        }
        self.print("`");
    }

    fn print_arrow(&mut self, e: ArrowFunctionExpression, level: Precedence) {
        let old_forbid_in = self.forbid_in;
        let wrap = level > Precedence::Assignment;
        if wrap {
            self.print("(");
        }
        if old_forbid_in {
            self.forbid_in = false;
        }
        if e.r#async(self.ast) {
            self.add_source_mapping(e.span(self.ast).start);
            self.print("async");
            self.print_space();
        }
        let type_parameters = e.type_parameters(self.ast);
        if let Some(t) = type_parameters {
            // Disambiguate singleton generic arrows from JSX.
            let trailing_comma = self.ast.lang() == Lang::Tsx && t.params(self.ast).len() == 1;
            self.print_ts_type_params_with_trailing_comma(t, trailing_comma);
        }
        let params = e.params(self.ast);
        let return_type = e.return_type(self.ast);
        let bare_parameter =
            if self.options.minify && type_parameters.is_none() && return_type.is_none() {
                self.bare_arrow_parameter(params)
            } else {
                None
            };
        if let Some(identifier) = bare_parameter {
            self.print_binding_identifier(identifier);
        } else {
            self.print_formal_parameters(params, None);
        }
        if let Some(t) = return_type {
            self.print_ts_annotation(t);
        }
        self.print_space();
        self.print("=>");
        self.print_space();
        match self.ast.arrow_function_body_data(e.body(self.ast)) {
            ArrowFunctionBodyData::FunctionBody(body) => self.print_function_body(body),
            ArrowFunctionBodyData::Expr(value) => {
                if old_forbid_in && !wrap {
                    self.forbid_in = true;
                }
                if self.options.comments {
                    let span = value.span(self.ast);
                    if self
                        .comments
                        .get(self.next_comment)
                        .is_some_and(|comment| comment.span.end <= span.start)
                    {
                        self.print_comments_before(span.start, false, true);
                    }
                    if self
                        .comments
                        .get(self.next_comment)
                        .is_some_and(|comment| comment.span.end <= span.end)
                    {
                        let first_token_offset = self.expr_first_token_offset(value);
                        self.print_comments_before(first_token_offset, false, true);
                    }
                }
                let old_arrow_expr_start = self.arrow_expr_start;
                self.arrow_expr_start = self.js.len();
                self.print_expr(value, Precedence::Assignment);
                self.arrow_expr_start = old_arrow_expr_start;
            }
        }
        if wrap {
            self.print(")");
        }
        self.forbid_in = old_forbid_in;
    }

    fn print_class(&mut self, c: Class, declaration: bool) {
        let is_expression = c.class_type(self.ast) == ClassType::ClassExpression;
        let offset = self.js.len();
        let wrap =
            is_expression && (offset == self.stmt_start || offset == self.export_default_start);
        if wrap {
            self.print("(");
        }
        if declaration {
            self.print_indent();
        }
        self.print_decorators(c.decorators(self.ast));
        self.add_source_mapping(c.span(self.ast).start);
        if c.declare(self.ast) {
            self.print("declare ");
        }
        if c.r#abstract(self.ast) {
            self.print("abstract ");
        }
        self.print_space_before_identifier();
        self.print("class");
        if let Some(id) = c.id(self.ast) {
            self.print(" ");
            self.print_binding_identifier(id);
        }
        if let Some(t) = c.type_parameters(self.ast) {
            self.print_ts_type_params(t);
        }
        if let Some(s) = c.super_class(self.ast) {
            self.print(" extends ");
            let type_arguments = c.super_type_arguments(self.ast);
            self.print_expr_with_context(s, Precedence::Postfix, ExprContext::NO_INSTANTIATION);
            if let Some(t) = type_arguments {
                self.print_ts_type_args(t);
            }
        }
        let implements = c.implements(self.ast);
        if !implements.is_empty() {
            self.print(" implements ");
            for (i, id) in implements.iter().enumerate() {
                if i != 0 {
                    self.print(",");
                    self.print_space();
                }
                let item = self.ast.get_node_in_sub_range(id);
                self.print_expr(item.expression(self.ast), Precedence::Assignment);
                if let Some(t) = item.type_arguments(self.ast) {
                    self.print_ts_type_args(t);
                }
            }
        }
        self.print_space();
        let old_forbid_in = self.forbid_in;
        self.forbid_in = false;
        self.add_source_mapping(c.body(self.ast).span(self.ast).start);
        self.print("{");
        let body = c.body(self.ast).body(self.ast);
        if !body.is_empty() || !self.options.minify {
            self.print_newline();
        }
        self.indent += 1;
        for id in body.iter() {
            self.print_semicolon_if_needed();
            self.print_class_element(self.ast.get_node_in_sub_range(id));
        }
        self.needs_semicolon = false;
        self.print_comments_before(
            c.body(self.ast).span(self.ast).end.saturating_sub(1),
            true,
            true,
        );
        self.indent -= 1;
        self.print_indent();
        self.add_source_mapping(c.body(self.ast).span(self.ast).end.saturating_sub(1));
        self.print("}");
        self.forbid_in = old_forbid_in;
        if wrap {
            self.print(")");
        }
    }

    fn print_class_element(&mut self, element: ClassElement) {
        match self.ast.class_element_data(element) {
            ClassElementData::MethodDefinition(m) => {
                self.print_indent();
                self.add_source_mapping(element.span(self.ast).start);
                self.print_decorators(m.decorators(self.ast));
                self.print_accessibility(m.accessibility(self.ast));
                if m.r#static(self.ast) {
                    self.print("static ");
                }
                if m.r#abstract(self.ast) {
                    self.print("abstract ");
                }
                if m.r#override(self.ast) {
                    self.print("override ");
                }
                let f = m.value(self.ast);
                if f.r#async(self.ast) {
                    self.print("async ");
                }
                match m.kind(self.ast) {
                    MethodDefinitionKind::Get => self.print("get "),
                    MethodDefinitionKind::Set => self.print("set "),
                    _ => {}
                }
                if f.generator(self.ast) {
                    self.print("*");
                }
                self.print_property_key(m.key(self.ast), m.computed(self.ast));
                if m.optional(self.ast) {
                    self.print("?");
                }
                if let Some(t) = f.type_parameters(self.ast) {
                    self.print_ts_type_params(t);
                }
                self.print_formal_parameters(f.params(self.ast), f.this_param(self.ast));
                if let Some(t) = f.return_type(self.ast) {
                    self.print_ts_annotation(t);
                }
                self.print_space();
                self.print_function_body(f.body(self.ast));
                self.print_newline();
            }
            ClassElementData::TsMethodDefinition(m) => {
                self.print_indent();
                self.add_source_mapping(element.span(self.ast).start);
                self.print_decorators(m.decorators(self.ast));
                self.print_accessibility(m.accessibility(self.ast));
                if m.r#static(self.ast) {
                    self.print("static ");
                }
                if m.r#abstract(self.ast) {
                    self.print("abstract ");
                }
                if m.r#override(self.ast) {
                    self.print("override ");
                }
                let f = m.value(self.ast);
                if f.r#async(self.ast) {
                    self.print("async ");
                }
                match m.kind(self.ast) {
                    MethodDefinitionKind::Get => self.print("get "),
                    MethodDefinitionKind::Set => self.print("set "),
                    _ => {}
                }
                if f.generator(self.ast) {
                    self.print("*");
                }
                self.print_property_key(m.key(self.ast), m.computed(self.ast));
                if m.optional(self.ast) {
                    self.print("?");
                }
                if let Some(t) = f.type_parameters(self.ast) {
                    self.print_ts_type_params(t);
                }
                self.print_formal_parameters(f.params(self.ast), f.this_param(self.ast));
                if let Some(t) = f.return_type(self.ast) {
                    self.print_ts_annotation(t);
                }
                self.print_semicolon_after_statement();
            }
            ClassElementData::PropertyDefinition(p) => {
                self.print_indent();
                self.add_source_mapping(element.span(self.ast).start);
                self.print_decorators(p.decorators(self.ast));
                self.print_accessibility(p.accessibility(self.ast));
                if p.r#static(self.ast) {
                    self.print("static ");
                }
                if p.r#abstract(self.ast) {
                    self.print("abstract ");
                }
                if p.declare(self.ast) {
                    self.print("declare ");
                }
                if p.r#override(self.ast) {
                    self.print("override ");
                }
                if p.readonly(self.ast) {
                    self.print("readonly ");
                }
                if p.accessor(self.ast) {
                    self.print("accessor");
                    if !self.options.minify || !p.computed(self.ast) {
                        self.print(" ");
                    }
                }
                self.print_property_key(p.key(self.ast), p.computed(self.ast));
                if p.optional(self.ast) {
                    self.print("?");
                }
                if p.definite(self.ast) {
                    self.print("!");
                }
                if let Some(t) = p.type_annotation(self.ast) {
                    self.print_ts_annotation(t);
                }
                if let Some(v) = p.value(self.ast) {
                    self.print_space();
                    self.print("=");
                    self.print_space();
                    self.print_expr(v, Precedence::Assignment);
                }
                self.print_semicolon_after_statement();
            }
            ClassElementData::StaticBlock(b) => {
                self.print_indent();
                self.add_source_mapping(element.span(self.ast).start);
                self.print("static {");
                let body = b.body(self.ast);
                if !body.is_empty() {
                    self.print_newline();
                    self.indent += 1;
                    for id in body.iter() {
                        self.print_semicolon_if_needed();
                        self.print_stmt(self.ast.get_node_in_sub_range(id));
                    }
                    self.needs_semicolon = false;
                    self.indent -= 1;
                    self.print_indent();
                } else if !self.options.minify {
                    self.print_newline();
                    self.print_indent();
                }
                self.add_source_mapping(element.span(self.ast).end.saturating_sub(1));
                self.print("}");
                self.print_newline();
            }
            ClassElementData::TsIndexSignature(s) => {
                self.print_indent();
                self.add_source_mapping(element.span(self.ast).start);
                self.print_ts_index_signature(s);
                self.print_semicolon_after_statement();
            }
        }
    }

    fn print_decorators(&mut self, decorators: SubRange<Decorator>) {
        fn need_wrap(ast: &Ast<'_>, mut expression: Expr) -> bool {
            loop {
                match ast.expr_data(expression) {
                    ExprData::IdentifierReference(_) => return false,
                    ExprData::ParenthesizedExpression(_) => return false,
                    ExprData::CallExpression(call) => expression = call.callee(ast),
                    ExprData::TsNonNullExpression(assertion) => {
                        expression = assertion.expression(ast)
                    }
                    ExprData::MemberExpression(member) if !member.computed(ast) => {
                        let name = match ast.property_key_data(member.property(ast)) {
                            PropertyKeyData::IdentifierName(identifier) => {
                                ast.get_utf8(identifier.name(ast))
                            }
                            PropertyKeyData::Expr(expression) => match ast.expr_data(expression) {
                                ExprData::PrivateIdentifier(identifier) => {
                                    ast.get_utf8(identifier.name(ast))
                                }
                                _ => return true,
                            },
                        };
                        if !is_identifier_es5_and_esnext(name.as_bytes()) {
                            return true;
                        }
                        expression = member.object(ast);
                    }
                    _ => return true,
                }
            }
        }

        for id in decorators.iter() {
            let decorator = self.ast.get_node_in_sub_range(id);
            self.add_source_mapping(decorator.span(self.ast).start);
            self.print("@");
            let expression = decorator.expression(self.ast);
            let wrap = need_wrap(self.ast, expression);
            if wrap {
                self.print("(");
            }
            self.print_expr(expression, Precedence::Lowest);
            if wrap {
                self.print(")");
            }
            self.print_space();
            self.print_space_before_identifier();
        }
    }

    fn print_accessibility(&mut self, a: Accessibility) {
        match a {
            Accessibility::None => {}
            Accessibility::Public => self.print("public "),
            Accessibility::Private => self.print("private "),
            Accessibility::Protected => self.print("protected "),
        }
    }

    fn print_bigint_literal(&mut self, literal: BigIntLiteral) {
        let raw = self.ast.get_utf8(literal.raw(self.ast));
        for byte in raw.bytes() {
            if byte != b'_' {
                self.js.push(byte);
            }
        }
        self.print("n");
    }

    fn print_number_literal(&mut self, literal: NumericLiteral, level: Precedence) {
        let value = literal.value(self.ast);
        assert!(
            !value.is_nan(),
            "NaN cannot be represented as a numeric literal"
        );
        if self.try_print_number_raw(literal) {
            return;
        }
        if value.is_infinite() {
            // Literal-only positions cannot use the `1 / 0` expression. Emit a canonical
            // overflowing numeric token derived from the authoritative value instead.
            if value.is_sign_negative() {
                self.print("-");
            } else {
                self.print_space_before_identifier();
            }
            self.print(INFINITY_LITERAL);
        } else {
            self.print_number_value(value, level);
        }
    }

    fn print_number_expr(&mut self, literal: NumericLiteral, level: Precedence) {
        if self.try_print_number_raw(literal) {
            return;
        }
        let value = literal.value(self.ast);
        self.print_number_value(value, level);
    }

    #[inline(always)]
    fn try_print_number_raw(&mut self, literal: NumericLiteral) -> bool {
        if self.options.minify || self.options.number_format != NumberFormat::Preserve {
            return false;
        }
        let Some(raw) = literal.raw(self.ast) else {
            return false;
        };
        let raw = self.ast.get_utf8(raw);

        self.print_space_before_identifier();
        self.print_bytes(raw);
        self.mark_raw_number_before_dot(raw);
        true
    }

    fn mark_raw_number_before_dot(&mut self, raw: &str) {
        let mut bytes = raw.bytes().filter(|&byte| byte != b'_');
        let Some(first) = bytes.next() else {
            return;
        };
        if first.is_ascii_digit() && bytes.all(|byte| byte.is_ascii_digit()) {
            self.need_space_before_dot = self.js.len();
        }
    }

    #[inline(always)]
    fn print_number_value(&mut self, value: f64, level: Precedence) {
        let abs_value = value.abs();
        if !value.is_finite() {
            self.print_non_finite_number_expr(value, level);
            return;
        }
        if self.try_print_number_with_format(value, level) {
            return;
        }
        self.print_signed_finite_number(value, level, |printer| {
            printer.print_non_negative_float(abs_value);
        });
    }

    fn try_print_number_with_format(&mut self, value: f64, level: Precedence) -> bool {
        let (prefix, bits_per_digit, alphabet) = match self.options.number_format {
            NumberFormat::Preserve => return false,
            NumberFormat::Decimal => {
                let abs_value = value.abs();
                self.print_signed_finite_number(value, level, |printer| {
                    printer.print_non_negative_number_as_decimal(abs_value);
                });
                return true;
            }
            NumberFormat::Binary => ("0b", 1, b"01".as_slice()),
            NumberFormat::Octal => ("0o", 3, b"01234567".as_slice()),
            NumberFormat::Hexadecimal { lowercase: true } => {
                ("0x", 4, b"0123456789abcdef".as_slice())
            }
            NumberFormat::Hexadecimal { lowercase: false } => {
                ("0x", 4, b"0123456789ABCDEF".as_slice())
            }
        };

        let abs_value = value.abs();
        let Some((mut integer, trailing_zero_digits)) =
            radix_integer_parts(abs_value, bits_per_digit)
        else {
            return false;
        };

        let digit_mask = (1_u64 << bits_per_digit) - 1;
        let mut digits = [0u8; 64];
        let mut start = digits.len();
        loop {
            start -= 1;
            digits[start] = alphabet[(integer & digit_mask) as usize];
            integer >>= bits_per_digit;
            if integer == 0 {
                break;
            }
        }
        self.print_signed_finite_number(value, level, |printer| {
            printer.print(prefix);
            printer.print_bytes(&digits[start..]);
            for _ in 0..trailing_zero_digits {
                printer.print("0");
            }
        });
        true
    }

    fn print_signed_finite_number(
        &mut self,
        value: f64,
        level: Precedence,
        print_absolute_value: impl FnOnce(&mut Self),
    ) {
        debug_assert!(value.is_finite());
        if !value.is_sign_negative() {
            self.print_space_before_identifier();
            print_absolute_value(self);
            return;
        }

        let wrap = level >= Precedence::Prefix;
        if wrap {
            self.print("(");
        }
        self.print_operator(OperatorCode::Unary(UnaryOperator::Negate), "-");
        print_absolute_value(self);
        if wrap {
            self.print(")");
        }
    }

    fn print_non_negative_number_as_decimal(&mut self, value: f64) {
        debug_assert!(!value.is_sign_negative());

        let mut buffer = zmij_ecma::Buffer::new();
        let formatted = buffer.format_finite(value);
        self.print_bytes(formatted);
        if formatted.bytes().all(|byte| byte.is_ascii_digit()) {
            self.need_space_before_dot = self.js.len();
        }
    }

    fn print_non_finite_number_expr(&mut self, value: f64, level: Precedence) {
        let requires_parentheses = level > Precedence::Multiply;
        if requires_parentheses && value == f64::INFINITY {
            self.print_space_before_identifier();
            self.print(INFINITY_LITERAL);
        } else {
            if requires_parentheses {
                self.print("(");
            }
            if value.is_nan() {
                self.print_space_before_identifier();
                self.print("0");
            } else {
                if value.is_sign_negative() {
                    self.print_operator(OperatorCode::Unary(UnaryOperator::Negate), "-");
                } else {
                    self.print_space_before_identifier();
                }
                self.print("1");
            }
            self.print_space();
            self.print("/");
            self.print_space();
            self.print("0");
            if requires_parentheses {
                self.print(")");
            }
        }
    }

    // Convert an integer to bytes without allocating.
    fn small_int_to_bytes(&mut self, mut n: i32) -> usize {
        let was_negative = n < 0;
        if was_negative {
            // Negating i32::MIN is not a concern because these integers are
            // floating-point exponents and never get that large.
            n = -n;
        }
        let mut start = self.int_to_bytes_buffer.len();

        // Write the number from the end of the buffer toward the front.
        loop {
            start -= 1;
            self.int_to_bytes_buffer[start] = b'0' + (n % 10) as u8;
            n /= 10;
            if n == 0 {
                break;
            }
        }
        if was_negative {
            // Put the negative sign immediately before the digits.
            start -= 1;
            self.int_to_bytes_buffer[start] = b'-';
        }
        start
    }

    fn print_non_negative_float(&mut self, abs_value: f64) {
        // Format according to ECMAScript Number::toString, then copy into a
        // stack-backed mutable buffer so subsequent source-level shortening can
        // happen in place without allocating for every numeric literal. The
        // longest finite f64 representation produced by `zmij_ecma` is 24 bytes,
        // and every rewrite below preserves or reduces that length.
        let mut buffer = zmij_ecma::Buffer::new();
        let formatted = buffer.format_finite(abs_value).as_bytes();
        let mut result = ArrayVec::<u8, 24>::new();
        result
            .try_extend_from_slice(formatted)
            .expect("an ECMAScript f64 representation fits in 24 bytes");

        // The plus sign is optional in a positive JavaScript exponent.
        if let Some(e) = result.iter().rposition(|&byte| byte == b'e')
            && result.get(e + 1) == Some(&b'+')
        {
            result.remove(e + 1);
        }

        let dot = result.iter().position(|&byte| byte == b'.');
        if dot == Some(1) && result[0] == b'0' {
            // Simplify numbers beginning with "0.".
            let mut after_dot = 2;
            if self.options.minify {
                // "0.5" => ".5"
                result.remove(0);
                after_dot -= 1;
            }
            if result.get(after_dot) == Some(&b'0') {
                // Try exponent notation: "0.001" => "1e-3".
                let mut i = after_dot + 1;
                while result.get(i) == Some(&b'0') {
                    i += 1;
                }
                let remaining_len = result.len() - i;
                let start =
                    self.small_int_to_bytes(after_dot as i32 - i as i32 - remaining_len as i32);
                let exponent = &self.int_to_bytes_buffer[start..];

                // Only switch representations when it is actually shorter.
                if result.len() > remaining_len + 1 + exponent.len() {
                    result.copy_within(i.., 0);
                    result.truncate(remaining_len);
                    result.push(b'e');
                    result
                        .try_extend_from_slice(exponent)
                        .expect("the shorter exponent representation fits");
                }
            }
        } else if let Some(dot) = dot {
            // Try removing the decimal point and, when possible, the exponent.
            if let Some(e) = result.iter().rposition(|&byte| byte == b'e') {
                let integer_len = dot;
                let fraction_len = e - dot - 1;
                let parsed_exponent = parse_small_int(&result[e + 1..]);
                let exponent = parsed_exponent - fraction_len as i32;
                if (0..=2).contains(&exponent) {
                    // Small exponents can be replaced by appended zeros:
                    // "1.2e1" => "12", "1.2e2" => "120".
                    let new_len = integer_len + fraction_len + exponent as usize;
                    if result.len() >= new_len {
                        result.copy_within(dot + 1..e, dot);
                        result.truncate(integer_len + fraction_len);
                        while result.len() < new_len {
                            result.push(b'0');
                        }
                    }
                } else {
                    // "1.2e4" => "12e3"
                    let start = self.small_int_to_bytes(exponent);
                    let exponent_bytes = &self.int_to_bytes_buffer[start..];
                    if result.len() >= integer_len + fraction_len + 1 + exponent_bytes.len() {
                        result.copy_within(dot + 1..e, dot);
                        result.truncate(integer_len + fraction_len);
                        result.push(b'e');
                        result
                            .try_extend_from_slice(exponent_bytes)
                            .expect("the shorter exponent representation fits");
                    }
                }
            }
        } else if result.last() == Some(&b'0') {
            // Try exponent notation for trailing zeros: "1000" => "1e3".
            let mut i = result.len() - 1;
            while i > 0 && result[i - 1] == b'0' {
                i -= 1;
            }
            let start = self.small_int_to_bytes((result.len() - i) as i32);
            let exponent = &self.int_to_bytes_buffer[start..];

            // Only switch representations when it is actually shorter.
            if result.len() > i + 1 + exponent.len() {
                result.truncate(i);
                result.push(b'e');
                result
                    .try_extend_from_slice(exponent)
                    .expect("the shorter exponent representation fits");
            }
        }

        // Integers in this range may be one byte shorter in hexadecimal. The
        // upper bound is the largest relevant exactly representable f64 below
        // the point where conversion can no longer fit in u64.
        if self.options.minify
            && (1_000_000_000_000.0..=18_446_744_073_709_549_568.0).contains(&abs_value)
        {
            let as_int = abs_value as u64;
            if abs_value == as_int as f64 {
                let mut hex_buffer = [0u8; 16];
                let mut value = as_int;
                let mut start = hex_buffer.len();
                loop {
                    start -= 1;
                    hex_buffer[start] = b"0123456789abcdef"[(value & 15) as usize];
                    value >>= 4;
                    if value == 0 {
                        break;
                    }
                }
                if 2 + hex_buffer.len() - start < result.len() {
                    result.clear();
                    result
                        .try_extend_from_slice(b"0x")
                        .expect("the shorter hexadecimal representation fits");
                    result
                        .try_extend_from_slice(&hex_buffer[start..])
                        .expect("the shorter hexadecimal representation fits");
                }
            }
        }

        self.print_bytes(&result);

        // A following "." needs a separator if it could be parsed as a decimal
        // point.
        if !result
            .iter()
            .any(|&byte| matches!(byte, b'.' | b'e' | b'x'))
        {
            self.need_space_before_dot = self.js.len();
        }
    }

    fn print_import(&mut self, d: ImportDeclaration) {
        self.print("import");
        if d.phase(self.ast) == Some(ImportPhase::Source) {
            self.print(" source");
        } else if d.phase(self.ast) == Some(ImportPhase::Defer) {
            self.print(" defer");
        }
        if d.import_kind(self.ast) == ImportOrExportKind::Type {
            self.print(" type");
        }
        let specs = d.specifiers(self.ast);
        if specs.is_empty() {
            self.print_space();
            if d.import_kind(self.ast) == ImportOrExportKind::Type {
                self.print("{}");
                self.print_space();
                self.print("from");
                self.print_space();
            }
            self.add_source_mapping(d.source(self.ast).span(self.ast).start);
            self.print_quoted(self.ast.get_wtf8(d.source(self.ast).value(self.ast)), false);
        } else {
            self.print_space();
            let mut first = true;
            let mut named = ArenaVec::new_in(self.allocator);
            for id in specs.iter() {
                match self
                    .ast
                    .import_declaration_specifier_data(self.ast.get_node_in_sub_range(id))
                {
                    ImportDeclarationSpecifierData::ImportDefaultSpecifier(s) => {
                        self.print_binding_identifier(s.local(self.ast));
                        first = false;
                    }
                    ImportDeclarationSpecifierData::ImportNamespaceSpecifier(s) => {
                        if !first {
                            self.print(",");
                            self.print_space();
                        }
                        self.print("*");
                        self.print_space();
                        self.print("as ");
                        self.print_binding_identifier(s.local(self.ast));
                        first = false;
                    }
                    ImportDeclarationSpecifierData::ImportSpecifier(s) => named.push(s),
                }
            }
            if !named.is_empty() {
                if !first {
                    self.print(",");
                    self.print_space();
                }
                self.print("{");
                self.print_space();
                for (i, s) in named.into_iter().enumerate() {
                    if i != 0 {
                        self.print(",");
                        self.print_space();
                    }
                    if s.import_kind(self.ast) == ImportOrExportKind::Type {
                        self.print("type ");
                    }
                    let imported = s.imported(self.ast);
                    self.print_module_name(imported);
                    let quoted = !self.options.minify
                        && self.ast.node_kind(imported.node_id()) == NodeKind::StringLiteral;
                    let local = self.ast.get_utf8(s.local(self.ast).name(self.ast));
                    if quoted || self.module_name(imported) != local {
                        self.print_space();
                        self.print_space_before_identifier();
                        self.print("as ");
                        self.print_binding_identifier(s.local(self.ast));
                    }
                }
                self.print_space();
                self.print("}");
            }
            self.print_space();
            self.print_space_before_identifier();
            self.print("from");
            self.print_space();
            self.add_source_mapping(d.source(self.ast).span(self.ast).start);
            self.print_quoted(self.ast.get_wtf8(d.source(self.ast).value(self.ast)), false);
        }
        self.print_import_attributes(d.attributes(self.ast), d.span(self.ast));
    }

    fn print_import_attributes(&mut self, attrs: SubRange<ImportAttribute>, span: Span) {
        if attrs.is_empty() {
            return;
        }

        self.print_space();
        self.add_source_mapping(span.start);
        self.print("with");
        self.print_space();
        self.print("{");
        self.print_space();
        for (i, id) in attrs.iter().enumerate() {
            if i != 0 {
                self.print(",");
                self.print_space();
            }
            let a = self.ast.get_node_in_sub_range(id);
            let key = a.key(self.ast);
            if let ModuleExportNameData::StringLiteral(string) =
                self.ast.module_export_name_data(key)
            {
                self.add_source_mapping(key.span(self.ast).start);
                self.print_quoted(self.ast.get_wtf8(string.value(self.ast)), false);
            } else {
                self.print_module_name(key);
            }
            self.print(":");
            self.print_space();
            self.add_source_mapping(a.value(self.ast).span(self.ast).start);
            self.print_quoted(self.ast.get_wtf8(a.value(self.ast).value(self.ast)), false);
        }
        self.print_space();
        self.add_source_mapping(span.end);
        self.print("}");
    }

    fn print_module_name(&mut self, name: ModuleExportName) {
        match self.ast.module_export_name_data(name) {
            ModuleExportNameData::IdentifierName(i) => {
                let text = self.ast.get_utf8(i.name(self.ast));
                self.add_source_mapping_for_name(name.span(self.ast).start, text);
                self.print_identifier(text)
            }
            ModuleExportNameData::StringLiteral(s) => {
                let text = self.ast.get_wtf8(s.value(self.ast));
                self.add_source_mapping(name.span(self.ast).start);
                if let Some(text) = text
                    .as_str()
                    .filter(|text| self.options.minify && is_identifier(text.as_bytes()))
                {
                    self.print_identifier(text)
                } else {
                    self.print_quoted(text, false)
                }
            }
        }
    }

    fn print_export_named(&mut self, d: ExportNamedDeclaration) {
        self.print_indent();
        self.add_source_mapping(d.span(self.ast).start);
        let declaration = d.declaration(self.ast);
        self.print("export");
        if let Some(decl) = declaration {
            let decorated_class = matches!(
                self.ast.decl_data(decl),
                DeclData::Class(class) if !class.decorators(self.ast).is_empty()
            );
            if decorated_class {
                self.print_space();
            } else {
                self.print(" ");
            }
        } else if d.export_kind(self.ast) == ImportOrExportKind::Type {
            self.print(" type ");
        } else {
            self.print_space();
        }
        if let Some(decl) = declaration {
            if let DeclData::Class(class) = self.ast.decl_data(decl) {
                self.print_class(class, false);
            } else {
                self.print_decl(decl);
            }
            self.print_decl_terminator(decl);
            return;
        }
        self.print("{");
        let specs = d.specifiers(self.ast);
        if !specs.is_empty() {
            self.print_space();
        }
        for (i, id) in specs.iter().enumerate() {
            if i != 0 {
                self.print(",");
                self.print_space();
            }
            let s = self.ast.get_node_in_sub_range(id);
            if s.export_kind(self.ast) == ImportOrExportKind::Type {
                self.print("type ");
            }
            let local = s.local(self.ast);
            let exported = s.exported(self.ast);
            self.print_module_name(local);
            let preserve_kind = !self.options.minify
                && self.ast.node_kind(local.node_id()) != self.ast.node_kind(exported.node_id());
            if preserve_kind || self.module_name(local) != self.module_name(exported) {
                self.print_space();
                self.print_space_before_identifier();
                self.print("as");
                if matches!(
                    self.ast.module_export_name_data(exported),
                    ModuleExportNameData::IdentifierName(_)
                ) {
                    self.print(" ");
                } else {
                    self.print_space();
                }
                self.print_module_name(exported);
            }
        }
        if !specs.is_empty() {
            self.print_space();
        }
        self.print("}");
        if let Some(source) = d.source(self.ast) {
            self.print_space();
            self.print_space_before_identifier();
            self.print("from");
            self.print_space();
            self.add_source_mapping(source.span(self.ast).start);
            self.print_quoted(self.ast.get_wtf8(source.value(self.ast)), false);
        }
        self.print_import_attributes(d.attributes(self.ast), d.span(self.ast));
        self.print_semicolon_after_statement();
    }

    fn print_export_default(&mut self, d: ExportDefaultDeclaration) {
        self.print_indent();
        self.add_source_mapping(d.span(self.ast).start);
        self.print("export default");
        let decorated_class = matches!(
            self.ast
                .export_default_declaration_kind_data(d.declaration(self.ast)),
            ExportDefaultDeclarationKindData::Class(class)
                if !class.decorators(self.ast).is_empty()
        );
        if decorated_class {
            self.print_space();
        } else {
            self.print(" ");
        }
        self.export_default_start = self.js.len();
        match self
            .ast
            .export_default_declaration_kind_data(d.declaration(self.ast))
        {
            ExportDefaultDeclarationKindData::Function(f) => {
                self.print_function(f);
                self.print_newline();
            }
            ExportDefaultDeclarationKindData::TsFunction(f) => {
                self.print_ts_function(f, false);
                self.print_semicolon_after_statement();
            }
            ExportDefaultDeclarationKindData::Class(c) => {
                self.print_class(c, false);
                self.print_newline();
            }
            ExportDefaultDeclarationKindData::TsInterfaceDeclaration(i) => {
                self.print_ts_interface(i);
                self.print_newline();
            }
            ExportDefaultDeclarationKindData::Expr(e) => {
                self.export_default_start = self.js.len();
                self.print_expr(e, Precedence::Comma);
                self.print_semicolon_after_statement();
            }
        }
    }

    // TypeScript printing follows the same recursive shape as the JavaScript
    // printer. Keeping it here avoids a second representation or conversion.
    fn print_ts_annotation(&mut self, a: TsTypeAnnotation) {
        self.print(":");
        self.print_space();
        self.print_ts_type(a.type_annotation(self.ast), TsTypePrecedence::Lowest);
    }

    fn print_ts_type_params(&mut self, d: TsTypeParameterDeclaration) {
        self.print_ts_type_params_with_trailing_comma(d, false);
    }

    fn print_ts_type_params_with_trailing_comma(
        &mut self,
        d: TsTypeParameterDeclaration,
        trailing_comma: bool,
    ) {
        self.print_operator(OperatorCode::TsTypeOpen, "<");
        for (i, id) in d.params(self.ast).iter().enumerate() {
            if i != 0 {
                self.print(",");
                self.print_space();
            }
            let p = self.ast.get_node_in_sub_range(id);
            if p.r#const(self.ast) {
                self.print("const ");
            }
            if p.r#in(self.ast) {
                self.print("in ");
            }
            if p.out(self.ast) {
                self.print("out ");
            }
            self.print_binding_identifier(p.name(self.ast));
            if let Some(t) = p.constraint(self.ast) {
                self.print(" extends ");
                self.print_ts_type(t, TsTypePrecedence::Lowest);
            }
            if let Some(t) = p.default(self.ast) {
                self.print_space();
                self.print("=");
                self.print_space();
                self.print_ts_type(t, TsTypePrecedence::Lowest);
            }
        }
        if trailing_comma {
            self.print(",");
        }
        self.print_operator(OperatorCode::TsTypeClose, ">");
    }

    fn print_ts_type_args(&mut self, d: TsTypeParameterInstantiation) {
        self.print_operator(OperatorCode::TsTypeOpen, "<");
        for (i, id) in d.params(self.ast).iter().enumerate() {
            if i != 0 {
                self.print(",");
                self.print_space();
            }
            self.print_ts_type(self.ast.get_node_in_sub_range(id), TsTypePrecedence::Lowest);
        }
        self.print_operator(OperatorCode::TsTypeClose, ">");
    }

    fn print_ts_type_assertion_prefix(&mut self, ty: TsType, source_offset: u32) {
        self.print_mapped_operator(OperatorCode::TsTypeOpen, "<", source_offset);
        self.print_ts_type(ty, TsTypePrecedence::Lowest);
        self.print_operator(OperatorCode::TsTypeClose, ">");
    }

    fn ts_type_can_precede_left_shift(&self, mut ty: TsType) -> bool {
        let mut level = TsTypePrecedence::Lowest;
        let mut context = TsTypeContext::Default;
        // Follow the printed right edge. Explicit or generated parentheses,
        // postfix operators, and other closing delimiters end the type instead.
        loop {
            ty = match self.ast.ts_type_data(ty) {
                TsTypeData::TsTypeQuery(query) => {
                    // TypeScript requires a standalone `<` token to start these
                    // type arguments, leaving `<<` for expression parsing.
                    // https://github.com/microsoft/TypeScript/blob/15dee00a1c6549a4953ab17ccac4c56d252ba4cc/tsc/internal/parser/parser.go#L3067-L3072
                    // `typeof import(...)` takes the separate import-type path.
                    // https://github.com/microsoft/TypeScript/blob/15dee00a1c6549a4953ab17ccac4c56d252ba4cc/tsc/internal/parser/parser.go#L2846-L2850
                    return !query.expr_name(self.ast).is_ts_import_type(self.ast);
                }
                TsTypeData::TsUnionType(t) if level <= TsTypePrecedence::Union => {
                    level = if self.options.minify {
                        TsTypePrecedence::Union
                    } else {
                        TsTypePrecedence::Intersection
                    };
                    context = TsTypeContext::Default;
                    let Some(last) = t.types(self.ast).last() else {
                        return false;
                    };
                    self.ast.get_node_in_sub_range(last)
                }
                TsTypeData::TsIntersectionType(t) if level <= TsTypePrecedence::Intersection => {
                    level = if self.options.minify {
                        TsTypePrecedence::Intersection
                    } else {
                        TsTypePrecedence::TypeOperator
                    };
                    context = TsTypeContext::Default;
                    let Some(last) = t.types(self.ast).last() else {
                        return false;
                    };
                    self.ast.get_node_in_sub_range(last)
                }
                TsTypeData::TsConditionalType(t)
                    if level <= TsTypePrecedence::Conditional
                        && context != TsTypeContext::ExtendsClause =>
                {
                    level = TsTypePrecedence::Lowest;
                    t.false_type(self.ast)
                }
                TsTypeData::TsTypeOperator(t) if level <= TsTypePrecedence::TypeOperator => {
                    level = TsTypePrecedence::TypeOperator;
                    t.type_annotation(self.ast)
                }
                TsTypeData::TsFunctionType(t) if level <= TsTypePrecedence::Function => {
                    level = TsTypePrecedence::Function;
                    context = context.function_return(false);
                    t.return_type(self.ast).type_annotation(self.ast)
                }
                TsTypeData::TsConstructorType(t) if level <= TsTypePrecedence::Function => {
                    level = TsTypePrecedence::Function;
                    context = context.function_return(false);
                    t.return_type(self.ast).type_annotation(self.ast)
                }
                TsTypeData::TsInferType(t) => {
                    let Some(constraint) = t.type_parameter(self.ast).constraint(self.ast) else {
                        return false;
                    };
                    let precedence = if context == TsTypeContext::ExtendsClause {
                        TsTypePrecedence::TypeOperator
                    } else {
                        TsTypePrecedence::Conditional
                    };
                    if level > precedence || context == TsTypeContext::BeforeConditionalQuestion {
                        return false;
                    }
                    level = TsTypePrecedence::Lowest;
                    context = TsTypeContext::ExtendsClause;
                    constraint
                }
                TsTypeData::TsTypePredicate(t) => {
                    let Some(annotation) = t.type_annotation(self.ast) else {
                        return false;
                    };
                    level = TsTypePrecedence::Lowest;
                    annotation.type_annotation(self.ast)
                }
                TsTypeData::TsRestType(t) => {
                    level = TsTypePrecedence::Lowest;
                    context = TsTypeContext::Default;
                    match self.ast.ts_tuple_element_data(t.type_annotation(self.ast)) {
                        TsTupleElementData::TsType(t) => t,
                        TsTupleElementData::TsNamedTupleMember(t) => t.element_type(self.ast),
                    }
                }
                TsTypeData::TsJsDocNullableType(t) if !t.postfix(self.ast) => {
                    level = TsTypePrecedence::TypeOperator;
                    context = TsTypeContext::Default;
                    t.type_annotation(self.ast)
                }
                TsTypeData::TsJsDocNonNullableType(t) if !t.postfix(self.ast) => {
                    level = TsTypePrecedence::TypeOperator;
                    context = TsTypeContext::Default;
                    t.type_annotation(self.ast)
                }
                _ => return false,
            };
        }
    }

    fn print_ts_type(&mut self, ty: TsType, level: TsTypePrecedence) {
        self.print_ts_type_with_context(ty, level, TsTypeContext::Default);
    }

    fn print_ts_type_with_context(
        &mut self,
        ty: TsType,
        level: TsTypePrecedence,
        context: TsTypeContext,
    ) {
        self.add_source_mapping(ty.span(self.ast).start);
        match self.ast.ts_type_data(ty) {
            TsTypeData::TsAnyKeyword(_) => self.print("any"),
            TsTypeData::TsUnknownKeyword(_) => self.print("unknown"),
            TsTypeData::TsNeverKeyword(_) => self.print("never"),
            TsTypeData::TsVoidKeyword(_) => self.print("void"),
            TsTypeData::TsNullKeyword(_) => self.print("null"),
            TsTypeData::TsUndefinedKeyword(_) => self.print("undefined"),
            TsTypeData::TsStringKeyword(_) => self.print("string"),
            TsTypeData::TsNumberKeyword(_) => self.print("number"),
            TsTypeData::TsBigIntKeyword(_) => self.print("bigint"),
            TsTypeData::TsBooleanKeyword(_) => self.print("boolean"),
            TsTypeData::TsSymbolKeyword(_) => self.print("symbol"),
            TsTypeData::TsObjectKeyword(_) => self.print("object"),
            TsTypeData::TsIntrinsicKeyword(_) => self.print("intrinsic"),
            TsTypeData::TsThisType(_) => self.print("this"),
            TsTypeData::TsTypeReference(t) => {
                self.print_ts_type_name(t.type_name(self.ast));
                if let Some(a) = t.type_arguments(self.ast) {
                    self.print_ts_type_args(a);
                }
            }
            TsTypeData::TsTypeQuery(t) => {
                self.print("typeof ");
                self.print_ts_query_name(t.expr_name(self.ast));
                if let Some(a) = t.type_arguments(self.ast) {
                    self.print_ts_type_args(a);
                }
            }
            TsTypeData::TsImportType(t) => self.print_ts_import_type(t),
            TsTypeData::TsLiteralType(t) => self.print_ts_literal(t.literal(self.ast)),
            TsTypeData::TsTemplateLiteralType(t) => {
                self.print("`");
                let qs = t.quasis(self.ast);
                let types = t.types(self.ast);
                for (i, id) in qs.iter().enumerate() {
                    self.print_bytes(
                        self.ast
                            .get_utf8(self.ast.get_node_in_sub_range(id).raw(self.ast)),
                    );
                    if i < types.len() {
                        self.print("${");
                        self.print_ts_type(
                            types.get_node(self.ast, i).unwrap(),
                            TsTypePrecedence::Lowest,
                        );
                        self.print("}");
                    }
                }
                self.print("`");
            }
            TsTypeData::TsArrayType(t) => {
                let wrap = level > TsTypePrecedence::Postfix;
                if wrap {
                    self.print("(");
                }
                self.print_ts_type(t.element_type(self.ast), TsTypePrecedence::Postfix);
                self.print("[]");
                if wrap {
                    self.print(")");
                }
            }
            TsTypeData::TsIndexedAccessType(t) => {
                let wrap = level > TsTypePrecedence::Postfix;
                if wrap {
                    self.print("(");
                }
                self.print_ts_type(t.object_type(self.ast), TsTypePrecedence::Postfix);
                self.print("[");
                self.print_ts_type(t.index_type(self.ast), TsTypePrecedence::Lowest);
                self.print("]");
                if wrap {
                    self.print(")");
                }
            }
            TsTypeData::TsTupleType(t) => {
                self.print("[");
                for (i, id) in t.element_types(self.ast).iter().enumerate() {
                    if i != 0 {
                        self.print(",");
                        self.print_space();
                    }
                    self.print_ts_tuple_element(self.ast.get_node_in_sub_range(id));
                }
                self.print("]");
            }
            TsTypeData::TsOptionalType(t) => {
                let wrap = level > TsTypePrecedence::Postfix;
                if wrap {
                    self.print("(");
                }
                self.print_ts_type(t.type_annotation(self.ast), TsTypePrecedence::Postfix);
                self.print("?");
                if wrap {
                    self.print(")");
                }
            }
            TsTypeData::TsRestType(t) => {
                self.print("...");
                self.print_ts_tuple_element(t.type_annotation(self.ast));
            }
            TsTypeData::TsJsDocNullableType(t) => {
                if !t.postfix(self.ast) {
                    self.print("?");
                }
                self.print_ts_type(t.type_annotation(self.ast), TsTypePrecedence::TypeOperator);
                if t.postfix(self.ast) {
                    self.print("?");
                }
            }
            TsTypeData::TsJsDocNonNullableType(t) => {
                if !t.postfix(self.ast) {
                    self.print("!");
                }
                self.print_ts_type(t.type_annotation(self.ast), TsTypePrecedence::TypeOperator);
                if t.postfix(self.ast) {
                    self.print("!");
                }
            }
            TsTypeData::TsJsDocUnknownType(_) => self.print("?"),
            TsTypeData::TsUnionType(t) => self.print_ts_type_list(TsTypeList::Union(t), level),
            TsTypeData::TsIntersectionType(t) => {
                self.print_ts_type_list(TsTypeList::Intersection(t), level)
            }
            TsTypeData::TsConditionalType(t) => {
                let wrap = level > TsTypePrecedence::Conditional
                    || context == TsTypeContext::ExtendsClause;
                if wrap {
                    self.print("(");
                }
                self.print_ts_type(t.check_type(self.ast), TsTypePrecedence::Union);
                self.print(" extends ");
                self.print_ts_type_with_context(
                    t.extends_type(self.ast),
                    TsTypePrecedence::Lowest,
                    TsTypeContext::ExtendsClause,
                );
                self.print(" ? ");
                self.print_ts_type(t.true_type(self.ast), TsTypePrecedence::Lowest);
                self.print(" : ");
                self.print_ts_type_with_context(
                    t.false_type(self.ast),
                    TsTypePrecedence::Lowest,
                    context.inside_group(wrap),
                );
                if wrap {
                    self.print(")");
                }
            }
            TsTypeData::TsInferType(t) => {
                let p = t.type_parameter(self.ast);
                let precedence = if p.constraint(self.ast).is_some()
                    && context != TsTypeContext::ExtendsClause
                {
                    TsTypePrecedence::Conditional
                } else {
                    TsTypePrecedence::TypeOperator
                };
                let wrap = level > precedence
                    || (context == TsTypeContext::BeforeConditionalQuestion
                        && p.constraint(self.ast).is_some());
                if wrap {
                    self.print("(");
                }
                self.print("infer ");
                self.print_binding_identifier(p.name(self.ast));
                if let Some(c) = p.constraint(self.ast) {
                    self.print(" extends ");
                    self.print_ts_type_with_context(
                        c,
                        TsTypePrecedence::Lowest,
                        TsTypeContext::ExtendsClause,
                    );
                }
                if wrap {
                    self.print(")");
                }
            }
            TsTypeData::TsTypeOperator(t) => {
                let wrap = level > TsTypePrecedence::TypeOperator;
                if wrap {
                    self.print("(");
                }
                self.print(t.operator(self.ast).as_str());
                self.print(" ");
                self.print_ts_type_with_context(
                    t.type_annotation(self.ast),
                    TsTypePrecedence::TypeOperator,
                    context.inside_group(wrap),
                );
                if wrap {
                    self.print(")");
                }
            }
            TsTypeData::TsParenthesizedType(t) => {
                self.print("(");
                self.print_ts_type(t.type_annotation(self.ast), TsTypePrecedence::Lowest);
                self.print(")");
            }
            TsTypeData::TsFunctionType(t) => {
                let wrap = level > TsTypePrecedence::Function;
                if wrap {
                    self.print("(");
                }
                if let Some(p) = t.type_parameters(self.ast) {
                    self.print_ts_type_params(p);
                }
                self.print_formal_parameters(t.params(self.ast), t.this_param(self.ast));
                self.print(" => ");
                self.print_ts_type_with_context(
                    t.return_type(self.ast).type_annotation(self.ast),
                    TsTypePrecedence::Function,
                    context.function_return(wrap),
                );
                if wrap {
                    self.print(")");
                }
            }
            TsTypeData::TsConstructorType(t) => {
                let wrap = level > TsTypePrecedence::Function;
                if wrap {
                    self.print("(");
                }
                if t.r#abstract(self.ast) {
                    self.print("abstract ");
                }
                self.print("new ");
                if let Some(p) = t.type_parameters(self.ast) {
                    self.print_ts_type_params(p);
                }
                self.print_formal_parameters(t.params(self.ast), None);
                self.print(" => ");
                self.print_ts_type_with_context(
                    t.return_type(self.ast).type_annotation(self.ast),
                    TsTypePrecedence::Function,
                    context.function_return(wrap),
                );
                if wrap {
                    self.print(")");
                }
            }
            TsTypeData::TsTypePredicate(t) => {
                if t.asserts(self.ast) {
                    self.print("asserts ");
                }
                match self
                    .ast
                    .ts_type_predicate_name_data(t.parameter_name(self.ast))
                {
                    TsTypePredicateNameData::IdentifierName(i) => {
                        self.print_identifier(self.ast.get_utf8(i.name(self.ast)))
                    }
                    TsTypePredicateNameData::TsThisType(_) => self.print("this"),
                };
                if let Some(a) = t.type_annotation(self.ast) {
                    self.print(" is ");
                    self.print_ts_type_with_context(
                        a.type_annotation(self.ast),
                        TsTypePrecedence::Lowest,
                        context,
                    );
                }
            }
            TsTypeData::TsTypeLiteral(t) => self.print_ts_type_members(t.members(self.ast)),
            TsTypeData::TsMappedType(t) => self.print_ts_mapped(t),
        }
    }

    fn print_ts_type_list(&mut self, list: TsTypeList, level: TsTypePrecedence) {
        let (types, sep, precedence, member_precedence) = match list {
            TsTypeList::Union(t) => (
                t.types(self.ast),
                " | ",
                TsTypePrecedence::Union,
                TsTypePrecedence::Intersection,
            ),
            TsTypeList::Intersection(t) => (
                t.types(self.ast),
                " & ",
                TsTypePrecedence::Intersection,
                TsTypePrecedence::TypeOperator,
            ),
        };
        let wrap = level > precedence;
        if wrap {
            self.print("(");
        }
        if types.len() == 1 && !self.options.minify {
            self.print(sep.trim());
            self.print_space();
        }
        let member_precedence = if self.options.minify {
            precedence
        } else {
            member_precedence
        };
        for (i, id) in types.iter().enumerate() {
            if i != 0 {
                self.print(sep);
            }
            self.print_ts_type(self.ast.get_node_in_sub_range(id), member_precedence);
        }
        if wrap {
            self.print(")");
        }
    }
    fn print_ts_type_name(&mut self, n: TsTypeName) {
        match self.ast.ts_type_name_data(n) {
            TsTypeNameData::IdentifierReference(i) => {
                let name = self.ast.get_utf8(i.name(self.ast));
                self.add_source_mapping_for_name(i.span(self.ast).start, name);
                self.print_identifier(name)
            }
            TsTypeNameData::TsQualifiedName(q) => self.print_ts_qualified(q),
        }
    }
    fn print_ts_qualified(&mut self, q: TsQualifiedName) {
        match self.ast.ts_qualified_name_left_data(q.left(self.ast)) {
            TsQualifiedNameLeftData::IdentifierReference(i) => {
                let name = self.ast.get_utf8(i.name(self.ast));
                self.add_source_mapping_for_name(i.span(self.ast).start, name);
                self.print_identifier(name)
            }
            TsQualifiedNameLeftData::IdentifierName(i) => {
                let name = self.ast.get_utf8(i.name(self.ast));
                self.add_source_mapping_for_name(i.span(self.ast).start, name);
                self.print_identifier(name)
            }
            TsQualifiedNameLeftData::BindingIdentifier(i) => {
                let name = self.ast.get_utf8(i.name(self.ast));
                self.add_source_mapping_for_name(i.span(self.ast).start, name);
                self.print_identifier(name)
            }
            TsQualifiedNameLeftData::ThisExpression(_) => self.print("this"),
            TsQualifiedNameLeftData::TsQualifiedName(x) => self.print_ts_qualified(x),
        }
        self.print(".");
        let right = q.right(self.ast);
        let name = self.ast.get_utf8(right.name(self.ast));
        self.add_source_mapping_for_name(right.span(self.ast).start, name);
        self.print_identifier(name);
    }
    fn print_ts_import_type(&mut self, t: TsImportType) {
        self.print("import(");
        let source = t.source(self.ast);
        self.add_source_mapping(source.span(self.ast).start);
        self.print_quoted(self.ast.get_wtf8(source.value(self.ast)), false);
        if let Some(options) = t.options(self.ast) {
            self.print(",");
            self.print_space();
            self.print_expr(options, Precedence::Assignment);
        }
        self.print(")");
        if let Some(qualifier) = t.qualifier(self.ast) {
            self.print(".");
            self.print_ts_import_qualifier(qualifier);
        }
        if let Some(type_arguments) = t.type_arguments(self.ast) {
            self.print_ts_type_args(type_arguments);
        }
    }

    fn print_ts_import_qualifier(&mut self, q: TsImportTypeQualifier) {
        match self.ast.ts_import_type_qualifier_data(q) {
            TsImportTypeQualifierData::IdentifierName(i) => {
                let name = self.ast.get_utf8(i.name(self.ast));
                self.add_source_mapping_for_name(i.span(self.ast).start, name);
                self.print_identifier(name)
            }
            TsImportTypeQualifierData::TsQualifiedName(q) => self.print_ts_qualified(q),
        }
    }
    fn print_ts_query_name(&mut self, n: TsTypeQueryExprName) {
        match self.ast.ts_type_query_expr_name_data(n) {
            TsTypeQueryExprNameData::TsTypeName(n) => self.print_ts_type_name(n),
            TsTypeQueryExprNameData::ThisExpression(_) => self.print("this"),
            TsTypeQueryExprNameData::TsImportType(t) => self.print_ts_import_type(t),
        }
    }
    fn print_ts_literal(&mut self, l: TsLiteral) {
        match self.ast.ts_literal_data(l) {
            TsLiteralData::StringLiteral(s) => {
                self.print_quoted(self.ast.get_wtf8(s.value(self.ast)), true)
            }
            TsLiteralData::NumericLiteral(n) => {
                self.print_number_literal(n, Precedence::Lowest);
            }
            TsLiteralData::BigIntLiteral(n) => self.print_bigint_literal(n),
            TsLiteralData::BooleanLiteral(b) => {
                self.print(if b.value(self.ast) { "true" } else { "false" })
            }
            TsLiteralData::TemplateLiteral(t) => self.print_template(t, false),
            TsLiteralData::UnaryExpression(u) => self.print_ts_signed_numeric_literal(u),
        }
    }

    fn print_ts_signed_numeric_literal(&mut self, expression: UnaryExpression) {
        let operator = expression.operator(self.ast);
        let argument = expression.argument(self.ast);
        let ExprData::NumericLiteral(number) = self.ast.expr_data(argument) else {
            self.print_unary::<false>(expression, Precedence::Lowest);
            return;
        };
        if !matches!(operator, UnaryOperator::Negate | UnaryOperator::Positive) {
            self.print_unary::<false>(expression, Precedence::Lowest);
            return;
        }

        self.print_comments_before(argument.span(self.ast).start, false, true);
        let operator_code = OperatorCode::Unary(operator);
        self.print_mapped_operator(
            operator_code,
            operator.as_str(),
            expression.span(self.ast).start,
        );

        self.add_source_mapping(argument.span(self.ast).start);
        self.print_number_literal(number, Precedence::Prefix);
    }
    fn print_ts_tuple_element(&mut self, e: TsTupleElement) {
        match self.ast.ts_tuple_element_data(e) {
            TsTupleElementData::TsType(t) => self.print_ts_type(t, TsTypePrecedence::Lowest),
            TsTupleElementData::TsNamedTupleMember(n) => {
                let label = n.label(self.ast);
                let name = self.ast.get_utf8(label.name(self.ast));
                self.add_source_mapping_for_name(label.span(self.ast).start, name);
                self.print_identifier(name);
                if n.optional(self.ast) {
                    self.print("?");
                }
                self.print(":");
                self.print_space();
                self.print_ts_type(n.element_type(self.ast), TsTypePrecedence::Lowest);
            }
        }
    }
    fn print_ts_type_members(&mut self, m: SubRange<TsTypeElement>) {
        self.print("{");
        if !m.is_empty() {
            self.print_newline();
        }
        self.indent += 1;
        for (index, id) in m.iter().enumerate() {
            self.print_indent();
            self.print_ts_type_element(self.ast.get_node_in_sub_range(id));
            // Type member separators must not affect deferred statement semicolons.
            if !self.options.minify || index + 1 < m.len() {
                self.print(";");
            }
            self.print_newline();
        }
        self.indent -= 1;
        self.print_indent();
        self.print("}");
    }
    fn print_ts_type_element(&mut self, e: TsTypeElement) {
        match self.ast.ts_type_element_data(e) {
            TsTypeElementData::TsPropertySignature(s) => {
                if s.readonly(self.ast) {
                    self.print("readonly ");
                }
                self.print_property_key(s.key(self.ast), s.computed(self.ast));
                if s.optional(self.ast) {
                    self.print("?");
                }
                if let Some(a) = s.type_annotation(self.ast) {
                    self.print_ts_annotation(a);
                }
            }
            TsTypeElementData::TsMethodSignature(s) => {
                match s.kind(self.ast) {
                    TsMethodSignatureKind::Get => self.print("get "),
                    TsMethodSignatureKind::Set => self.print("set "),
                    _ => {}
                }
                self.print_property_key(s.key(self.ast), s.computed(self.ast));
                if s.optional(self.ast) {
                    self.print("?");
                }
                if let Some(p) = s.type_parameters(self.ast) {
                    self.print_ts_type_params(p);
                }
                self.print_formal_parameters(s.params(self.ast), s.this_param(self.ast));
                if let Some(a) = s.return_type(self.ast) {
                    self.print_ts_annotation(a);
                }
            }
            TsTypeElementData::TsCallSignatureDeclaration(s) => {
                if let Some(p) = s.type_parameters(self.ast) {
                    self.print_ts_type_params(p);
                }
                self.print_formal_parameters(s.params(self.ast), s.this_param(self.ast));
                if let Some(a) = s.return_type(self.ast) {
                    self.print_ts_annotation(a);
                }
            }
            TsTypeElementData::TsConstructSignatureDeclaration(s) => {
                self.print("new ");
                if let Some(p) = s.type_parameters(self.ast) {
                    self.print_ts_type_params(p);
                }
                self.print_formal_parameters(s.params(self.ast), None);
                if let Some(a) = s.return_type(self.ast) {
                    self.print_ts_annotation(a);
                }
            }
            TsTypeElementData::TsIndexSignature(s) => self.print_ts_index_signature(s),
        }
    }
    fn print_ts_index_signature(&mut self, s: TsIndexSignature) {
        if s.r#static(self.ast) {
            self.print("static ");
        }
        if s.readonly(self.ast) {
            self.print("readonly ");
        }
        self.print("[");
        for (i, id) in s.parameters(self.ast).iter().enumerate() {
            if i != 0 {
                self.print(",");
                self.print_space();
            }
            let parameter = self.ast.get_node_in_sub_range(id);
            self.print_binding_identifier(parameter.name(self.ast));
            self.print_ts_annotation(parameter.type_annotation(self.ast));
        }
        self.print("]");
        self.print_ts_annotation(s.type_annotation(self.ast));
    }
    fn print_ts_mapped(&mut self, t: TsMappedType) {
        self.print("{");
        self.print_newline();
        self.indent += 1;
        self.print_indent();
        match t.readonly(self.ast) {
            TsMappedTypeModifier::True => self.print("readonly "),
            TsMappedTypeModifier::Plus => self.print("+readonly "),
            TsMappedTypeModifier::Minus => self.print("-readonly "),
            TsMappedTypeModifier::None => {}
        }
        self.print("[");
        self.print_binding_identifier(t.key(self.ast));
        self.print(" in ");
        self.print_ts_type(t.constraint(self.ast), TsTypePrecedence::Lowest);
        if let Some(n) = t.name_type(self.ast) {
            self.print(" as ");
            self.print_ts_type(n, TsTypePrecedence::Lowest);
        }
        self.print("]");
        match t.optional(self.ast) {
            TsMappedTypeModifier::True => self.print("?"),
            TsMappedTypeModifier::Plus => self.print("+?"),
            TsMappedTypeModifier::Minus => self.print("-?"),
            TsMappedTypeModifier::None => {}
        }
        if let Some(a) = t.type_annotation(self.ast) {
            self.print(":");
            self.print_space();
            self.print_ts_type(a, TsTypePrecedence::Lowest);
        }
        self.print(";");
        self.print_newline();
        self.indent -= 1;
        self.print_indent();
        self.print("}");
    }
    fn print_ts_type_alias(&mut self, d: TsTypeAliasDeclaration) {
        self.add_source_mapping(d.span(self.ast).start);
        if d.declare(self.ast) {
            self.print("declare ");
        }
        self.print("type ");
        self.print_binding_identifier(d.id(self.ast));
        if let Some(p) = d.type_parameters(self.ast) {
            self.print_ts_type_params(p);
        }
        self.print_space();
        self.print("=");
        self.print_space();
        self.print_ts_type(d.type_annotation(self.ast), TsTypePrecedence::Lowest);
    }
    fn print_ts_interface(&mut self, d: TsInterfaceDeclaration) {
        self.add_source_mapping(d.span(self.ast).start);
        if d.declare(self.ast) {
            self.print("declare ");
        }
        self.print("interface ");
        self.print_binding_identifier(d.id(self.ast));
        if let Some(p) = d.type_parameters(self.ast) {
            self.print_ts_type_params(p);
        }
        let ex = d.extends(self.ast);
        if !ex.is_empty() {
            self.print(" extends ");
            for (i, id) in ex.iter().enumerate() {
                if i != 0 {
                    self.print(",");
                    self.print_space();
                }
                let h = self.ast.get_node_in_sub_range(id);
                self.print_expr(h.expression(self.ast), Precedence::Assignment);
                if let Some(a) = h.type_arguments(self.ast) {
                    self.print_ts_type_args(a);
                }
            }
        }
        self.print_space();
        self.print_ts_type_members(d.body(self.ast).body(self.ast));
    }
    fn print_ts_enum(&mut self, d: TsEnumDeclaration) {
        self.add_source_mapping(d.span(self.ast).start);
        if d.declare(self.ast) {
            self.print("declare ");
        }
        if d.is_const(self.ast) {
            self.print("const ");
        }
        self.print("enum ");
        self.print_binding_identifier(d.id(self.ast));
        self.print_space();
        self.add_source_mapping(d.body(self.ast).span(self.ast).start);
        self.print("{");
        let m = d.body(self.ast).members(self.ast);
        if !m.is_empty() {
            self.print_newline();
        }
        self.indent += 1;
        for id in m.iter() {
            let x = self.ast.get_node_in_sub_range(id);
            self.print_indent();
            self.add_source_mapping(x.span(self.ast).start);
            self.print_ts_enum_name(x.id(self.ast), x.computed(self.ast));
            if let Some(v) = x.initializer(self.ast) {
                self.print_space();
                self.print("=");
                self.print_space();
                self.print_expr(v, Precedence::Assignment);
            }
            self.print(",");
            self.print_newline();
        }
        self.indent -= 1;
        self.print_indent();
        self.add_source_mapping(d.body(self.ast).span(self.ast).end.saturating_sub(1));
        self.print("}");
    }
    fn print_ts_enum_name(&mut self, n: TsEnumMemberName, computed: bool) {
        if computed {
            self.print("[");
            self.print_comments_before(n.span(self.ast).start, false, true);
        }
        match self.ast.ts_enum_member_name_data(n) {
            TsEnumMemberNameData::IdentifierName(i) => {
                let name = self.ast.get_utf8(i.name(self.ast));
                self.add_source_mapping_for_name(i.span(self.ast).start, name);
                self.print_identifier(name)
            }
            TsEnumMemberNameData::StringLiteral(s) => {
                self.add_source_mapping(s.span(self.ast).start);
                self.print_quoted(self.ast.get_wtf8(s.value(self.ast)), computed)
            }
            TsEnumMemberNameData::TemplateLiteral(t) => self.print_template(t, false),
        }
        if computed {
            self.print("]");
        }
    }
    fn print_ts_module(&mut self, d: TsModuleDeclaration) {
        self.add_source_mapping(d.span(self.ast).start);
        if d.declare(self.ast) {
            self.print("declare ");
        }
        self.print(match d.kind(self.ast) {
            TsModuleDeclarationKind::Namespace => "namespace ",
            TsModuleDeclarationKind::Module => "module ",
        });
        self.print_ts_module_name(d.id(self.ast));
        if let Some(b) = d.body(self.ast) {
            self.print_space();
            self.print_ts_module_body(b);
        }
    }
    fn print_ts_module_name(&mut self, n: TsModuleName) {
        match self.ast.ts_module_name_data(n) {
            TsModuleNameData::BindingIdentifier(i) => self.print_binding_identifier(i),
            TsModuleNameData::StringLiteral(s) => {
                self.add_source_mapping(s.span(self.ast).start);
                self.print_quoted(self.ast.get_wtf8(s.value(self.ast)), false)
            }
            TsModuleNameData::TsQualifiedName(q) => self.print_ts_qualified(q),
        }
    }
    fn print_ts_module_body(&mut self, b: TsModuleBody) {
        match self.ast.ts_module_body_data(b) {
            TsModuleBodyData::TsModuleBlock(b) => {
                self.add_source_mapping(b.span(self.ast).start);
                self.print("{");
                let directives = b.directives(self.ast);
                let statements = b.body(self.ast);
                if !directives.is_empty() || !statements.is_empty() {
                    self.print_newline();
                }
                self.indent += 1;
                for id in directives.iter() {
                    self.print_semicolon_if_needed();
                    let directive = self.ast.get_node_in_sub_range(id);
                    self.print_indent();
                    self.print_directive(directive);
                    self.print_semicolon_after_statement();
                }
                for (index, id) in statements.iter().enumerate() {
                    self.print_semicolon_if_needed();
                    let statement = self.ast.get_node_in_sub_range(id);
                    self.print_stmt_with_context(statement, index == 0);
                }
                self.needs_semicolon = false;
                self.indent -= 1;
                self.print_indent();
                self.add_source_mapping(b.span(self.ast).end.saturating_sub(1));
                self.print("}");
            }
            TsModuleBodyData::TsModuleDeclaration(d) => self.print_ts_module(d),
        }
    }
    fn print_ts_global(&mut self, d: TsGlobalDeclaration) {
        self.add_source_mapping(d.span(self.ast).start);
        if d.declare(self.ast) {
            self.print("declare ");
        }
        let id = d.id(self.ast);
        let name = self.ast.get_utf8(id.name(self.ast));
        self.add_source_mapping_for_name(id.span(self.ast).start, name);
        self.print_identifier(name);
        self.print_space();
        let b = d.body(self.ast);
        self.add_source_mapping(b.span(self.ast).start);
        self.print("{");
        let directives = b.directives(self.ast);
        let statements = b.body(self.ast);
        if !directives.is_empty() || !statements.is_empty() {
            self.print_newline();
        }
        self.indent += 1;
        // Keep the existing TS global behavior: directives participate in
        // statement-list layout but are not emitted.
        for _ in directives.iter() {
            self.print_semicolon_if_needed();
        }
        for (index, id) in statements.iter().enumerate() {
            self.print_semicolon_if_needed();
            let statement = self.ast.get_node_in_sub_range(id);
            self.print_stmt_with_context(statement, index == 0);
        }
        self.needs_semicolon = false;
        self.indent -= 1;
        self.print_indent();
        self.add_source_mapping(b.span(self.ast).end.saturating_sub(1));
        self.print("}");
    }
    fn print_ts_parameter_property(&mut self, p: TsParameterProperty) {
        self.print_decorators(p.decorators(self.ast));
        self.print_accessibility(p.accessibility(self.ast));
        if p.r#override(self.ast) {
            self.print("override ");
        }
        if p.readonly(self.ast) {
            self.print("readonly ");
        }
        self.print_formal_parameter(p.parameter(self.ast));
    }
    fn print_ts_import_equals(&mut self, d: TsImportEqualsDeclaration) {
        self.add_source_mapping(d.span(self.ast).start);
        self.print("import ");
        if d.import_kind(self.ast) == ImportOrExportKind::Type {
            self.print("type ");
        }
        self.print_binding_identifier(d.id(self.ast));
        self.print_space();
        self.print("=");
        self.print_space();
        match self
            .ast
            .ts_module_reference_data(d.module_reference(self.ast))
        {
            TsModuleReferenceData::TsTypeName(n) => self.print_ts_type_name(n),
            TsModuleReferenceData::TsExternalModuleReference(r) => {
                self.print("require(");
                self.add_source_mapping(r.expression(self.ast).span(self.ast).start);
                self.print_quoted(
                    self.ast.get_wtf8(r.expression(self.ast).value(self.ast)),
                    false,
                );
                self.print(")");
            }
        }
    }

    fn print_jsx_element(&mut self, e: JsxElement) {
        let o = e.opening_element(self.ast);
        self.add_source_mapping(e.span(self.ast).start);
        self.print("<");
        self.print_jsx_name(o.name(self.ast));
        if let Some(type_arguments) = o.type_arguments(self.ast) {
            self.print_ts_type_args(type_arguments);
        }
        for id in o.attributes(self.ast).iter() {
            let item = self.ast.get_node_in_sub_range(id);
            self.print_comments_before(item.span(self.ast).start, false, false);
            let data = self.ast.jsx_attribute_item_data(item);
            self.print_space();
            match data {
                JsxAttributeItemData::JsxAttribute(a) => {
                    self.print_space_before_identifier();
                    self.print_jsx_attr_name(a.name(self.ast));
                    if let Some(v) = a.value(self.ast) {
                        self.print("=");
                        self.print_jsx_attr_value(v);
                    }
                }
                JsxAttributeItemData::JsxSpreadAttribute(s) => {
                    self.add_source_mapping(item.span(self.ast).start);
                    self.print("{...");
                    self.print_expr(s.argument(self.ast), Precedence::Comma);
                    self.print_comments_before(s.span(self.ast).end.saturating_sub(1), false, true);
                    self.print("}");
                }
            }
        }
        self.print_comments_before(o.span(self.ast).end, false, false);
        if o.self_closing(self.ast) || (self.options.minify && e.children(self.ast).is_empty()) {
            self.print_space();
            self.add_source_mapping(o.span(self.ast).end.saturating_sub(2));
            self.print("/>");
            return;
        }
        self.print(">");
        for id in e.children(self.ast).iter() {
            self.print_jsx_child(self.ast.get_node_in_sub_range(id));
        }
        if let Some(c) = e.closing_element(self.ast) {
            self.add_source_mapping(c.span(self.ast).start);
            self.print("</");
            self.print_jsx_name(c.name(self.ast));
            self.print_comments_before(c.span(self.ast).end, false, false);
            self.print(">");
        }
    }
    fn print_jsx_fragment(&mut self, e: JsxFragment) {
        self.add_source_mapping(e.span(self.ast).start);
        self.print("<>");
        for id in e.children(self.ast).iter() {
            self.print_jsx_child(self.ast.get_node_in_sub_range(id));
        }
        self.add_source_mapping(e.span(self.ast).end.saturating_sub(3));
        self.print("</>");
    }
    fn print_jsx_name(&mut self, n: JsxElementName) {
        self.print_comments_before(n.span(self.ast).start, false, false);
        match self.ast.jsx_element_name_data(n) {
            JsxElementNameData::JsxIdentifier(i) => {
                let name = self.ast.get_utf8(i.name(self.ast));
                self.add_source_mapping_for_name(n.span(self.ast).start, name);
                self.print_bytes(name)
            }
            JsxElementNameData::JsxNamespacedName(x) => {
                self.add_source_mapping(n.span(self.ast).start);
                self.print_bytes(self.ast.get_utf8(x.namespace(self.ast).name(self.ast)));
                self.print(":");
                self.print_bytes(self.ast.get_utf8(x.name(self.ast).name(self.ast)));
            }
            JsxElementNameData::JsxMemberExpression(m) => self.print_jsx_member(m),
        }
    }
    fn print_jsx_member(&mut self, m: JsxMemberExpression) {
        match self
            .ast
            .jsx_member_expression_object_data(m.object(self.ast))
        {
            JsxMemberExpressionObjectData::JsxIdentifier(i) => {
                let name = self.ast.get_utf8(i.name(self.ast));
                self.add_source_mapping_for_name(i.span(self.ast).start, name);
                self.print_bytes(name)
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(m) => self.print_jsx_member(m),
        }
        self.print(".");
        let property = m.property(self.ast);
        let name = self.ast.get_utf8(property.name(self.ast));
        self.add_source_mapping_for_name(property.span(self.ast).start, name);
        self.print_bytes(name);
    }
    fn print_jsx_attr_name(&mut self, n: JsxAttributeName) {
        match self.ast.jsx_attribute_name_data(n) {
            JsxAttributeNameData::JsxIdentifier(i) => {
                let name = self.ast.get_utf8(i.name(self.ast));
                self.add_source_mapping_for_name(n.span(self.ast).start, name);
                self.print_bytes(name)
            }
            JsxAttributeNameData::JsxNamespacedName(x) => {
                self.add_source_mapping(n.span(self.ast).start);
                self.print_bytes(self.ast.get_utf8(x.namespace(self.ast).name(self.ast)));
                self.print(":");
                self.print_bytes(self.ast.get_utf8(x.name(self.ast).name(self.ast)));
            }
        }
    }
    fn print_jsx_attr_value(&mut self, v: JsxAttributeValue) {
        self.print_comments_before(v.span(self.ast).start, false, false);
        match self.ast.jsx_attribute_value_data(v) {
            JsxAttributeValueData::StringLiteral(s) => {
                let raw = self.ast.get_utf8(s.raw(self.ast));
                if raw.is_empty() {
                    let value = self.ast.get_wtf8(s.value(self.ast));
                    // JSX attribute strings do not interpret JavaScript escape sequences, so
                    // print the value bytes directly instead of using `print_quoted`.
                    let quote = if value.as_bytes().contains(&b'"') {
                        "'"
                    } else {
                        "\""
                    };
                    self.print(quote);
                    self.print_bytes(value.as_bytes());
                    self.print(quote);
                } else {
                    self.print_bytes(raw);
                }
            }
            JsxAttributeValueData::JsxExpressionContainer(c) => self.print_jsx_container(c),
            JsxAttributeValueData::JsxElement(e) => self.print_jsx_element(e),
            JsxAttributeValueData::JsxFragment(e) => self.print_jsx_fragment(e),
        }
    }
    fn print_jsx_container(&mut self, c: JsxExpressionContainer) {
        self.print("{");
        match self.ast.jsx_expression_data(c.expression(self.ast)) {
            JsxExpressionData::Expr(e) => {
                self.print_expr(e, Precedence::Assignment);
            }
            JsxExpressionData::JsxEmptyExpression(_) => {
                let close_offset = c.span(self.ast).end.saturating_sub(1);
                if !self.options.minify
                    && self.comments[self.next_comment..]
                        .iter()
                        .any(|comment| comment.span.end <= close_offset)
                {
                    self.print_newline();
                    self.indent += 1;
                    self.print_comments_before(close_offset, true, true);
                    self.indent -= 1;
                    self.print_indent();
                }
            }
        }
        self.print_comments_before(c.span(self.ast).end.saturating_sub(1), false, true);
        self.print("}");
    }
    fn print_jsx_child(&mut self, c: JsxChild) {
        match self.ast.jsx_child_data(c) {
            JsxChildData::JsxText(t) => {
                self.print_bytes(self.ast.get_utf8(t.value(self.ast)));
            }
            JsxChildData::JsxExpressionContainer(c) => self.print_jsx_container(c),
            JsxChildData::JsxSpreadChild(s) => {
                self.print("{...");
                self.print_expr(s.expression(self.ast), Precedence::Lowest);
                self.print_comments_before(s.span(self.ast).end.saturating_sub(1), false, true);
                self.print("}");
            }
            JsxChildData::JsxElement(e) => self.print_jsx_element(e),
            JsxChildData::JsxFragment(e) => self.print_jsx_fragment(e),
        }
    }
}

impl Printer<'_> {
    fn binding_pattern_identifier_name(&self, binding: BindingPattern) -> Option<&str> {
        match self.ast.binding_pattern_data(binding) {
            BindingPatternData::BindingIdentifier(identifier) => {
                Some(self.ast.get_utf8(identifier.name(self.ast)))
            }
            BindingPatternData::SimpleAssignmentTarget(target) => {
                match self.ast.simple_assignment_target_data(target) {
                    SimpleAssignmentTargetData::IdentifierReference(identifier) => {
                        Some(self.ast.get_utf8(identifier.name(self.ast)))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn bare_arrow_parameter(&self, params: FormalParameters) -> Option<BindingIdentifier> {
        let ast = self.ast;
        let items = params.items(ast);
        if items.len() != 1 || params.rest(ast).is_some() {
            return None;
        }
        let item = items.get_node(ast, 0)?;
        let param = item.as_formal_parameter(ast)?;
        let identifier = param.pattern(ast).as_binding_identifier(ast)?;
        if !param.decorators(ast).is_empty()
            || param.type_annotation(ast).is_some()
            || param.initializer(ast).is_some()
            || param.optional(ast)
        {
            return None;
        }
        Some(identifier)
    }

    #[inline]
    fn can_print_identifier(&self, name: &str) -> bool {
        let bytes = name.as_bytes();
        let Some((&first, rest)) = bytes.split_first() else {
            return false;
        };
        if is_ident_start_ascii(first) {
            if rest.iter().all(|&byte| is_ident_continue_ascii(byte)) {
                return true;
            }
            if bytes.is_ascii() {
                return false;
            }
        } else if first.is_ascii() {
            return false;
        }
        if self.options.ascii_only || self.options.minify {
            is_identifier_es5_and_esnext(bytes)
        } else {
            is_identifier(bytes)
        }
    }

    fn module_name(&self, n: ModuleExportName) -> &Wtf8 {
        let ast = self.ast;
        match ast.module_export_name_data(n) {
            ModuleExportNameData::IdentifierName(i) => Wtf8::from_str(ast.get_utf8(i.name(ast))),
            ModuleExportNameData::StringLiteral(s) => ast.get_wtf8(s.value(ast)),
        }
    }
}

fn is_legal_comment(ast: &Ast, comment: Comment) -> bool {
    let value = ast
        .get_utf8(comment.value(ast.source().as_bytes()))
        .as_bytes();
    if ast
        .source()
        .as_bytes()
        .get(comment.span.start as usize..comment.span.start as usize + 3)
        .is_some_and(|prefix| prefix == b"/*!" || prefix == b"//!")
    {
        return true;
    }

    value.iter().enumerate().any(|(index, &byte)| {
        byte == b'@'
            && value
                .get(index..)
                .is_some_and(|rest| rest.starts_with(b"@license") || rest.starts_with(b"@preserve"))
    })
}
