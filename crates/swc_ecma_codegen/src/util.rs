use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};

use swc_next_ecma_ast::*;
use unicode_id_start::{is_id_continue, is_id_start};

const HEX: &[u8; 16] = b"0123456789ABCDEF";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[allow(dead_code)] // Keep esbuild's complete precedence lattice.
pub(crate) enum Level {
    Lowest,
    Comma,
    Yield,
    Assignment,
    Conditional,
    Nullish,
    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Equality,
    Compare,
    Shift,
    Add,
    Multiply,
    Exponent,
    Prefix,
    Postfix,
    New,
    Call,
    Member,
    Primary,
}

pub(crate) struct AstAdapterState {
    pub(crate) lowered_template_count: usize,
    pub(crate) next_lowered_template: usize,
    class_decorators_were_printed: bool,
}

#[inline]
pub(crate) fn ast_text<'a>(ast: &'a Ast<'a>, value: String) -> &'a [u8] {
    ast.string(value)
}

impl AstAdapterState {
    pub(crate) fn new(ast: &Ast<'_>, root: Program) -> Self {
        Self {
            lowered_template_count: count_templates_with_closing_script(ast, root),
            next_lowered_template: 0,
            class_decorators_were_printed: false,
        }
    }

    pub(crate) fn take_lowered_template_index(&mut self) -> usize {
        let index = self.next_lowered_template;
        self.next_lowered_template += 1;
        index
    }

    pub(crate) fn mark_class_decorators_as_printed(&mut self) {
        self.class_decorators_were_printed = true;
    }

    pub(crate) fn take_class_decorators_were_printed(&mut self) -> bool {
        std::mem::take(&mut self.class_decorators_were_printed)
    }
}

use crate::{
    SourceMap,
    sourcemap::{Chunk, MappingsBuffer},
};

pub(crate) fn finalize_source_map<'a>(
    allocator: &'a Allocator,
    source: &'a [u8],
    source_path: &[u8],
    chunk: Chunk<'a>,
) -> SourceMap<'a> {
    let mut path = ArenaVec::with_capacity_in(source_path.len(), allocator);
    path.extend_from_slice_copy(source_path);
    let path = path.into_bump_slice() as &'a [u8];

    let mut sources = ArenaVec::with_capacity_in(1, allocator);
    sources.push(path);
    let sources = sources.into_bump_slice() as &'a [&'a [u8]];

    let mut sources_content = ArenaVec::with_capacity_in(1, allocator);
    sources_content.push(Some(source));
    let sources_content = sources_content.into_bump_slice() as &'a [Option<&'a [u8]>];

    // Chunk joining and JSON assembly are intentionally outside sourcemap.rs:
    // they adapt esbuild's chunk-oriented API to this crate's arena-backed,
    // single-source public result type.
    let Chunk {
        buffer: MappingsBuffer { data: mappings, .. },
        quoted_names: names,
        ..
    } = chunk;
    SourceMap {
        version: 3,
        file: None,
        source_root: None,
        sources,
        sources_content: Some(sources_content),
        names,
        mappings,
    }
}

pub(crate) fn unary_operator(op: UnaryOperator) -> &'static str {
    match op {
        UnaryOperator::Negate => "-",
        UnaryOperator::Positive => "+",
        UnaryOperator::LogicalNot => "!",
        UnaryOperator::BitwiseNot => "~",
        UnaryOperator::Typeof => "typeof",
        UnaryOperator::Void => "void",
        UnaryOperator::Delete => "delete",
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum OperatorCode {
    Add,
    Subtract,
    Positive,
    Negative,
    PreIncrement,
    PreDecrement,
    PostDecrement,
    GreaterThan,
    Not,
    #[default]
    Other,
}

pub(crate) fn binary_operator_code(operator: BinaryOperator) -> OperatorCode {
    match operator {
        BinaryOperator::Add => OperatorCode::Add,
        BinaryOperator::Subtract => OperatorCode::Subtract,
        BinaryOperator::GreaterThan => OperatorCode::GreaterThan,
        _ => OperatorCode::Other,
    }
}

pub(crate) fn unary_operator_code(operator: UnaryOperator) -> OperatorCode {
    match operator {
        UnaryOperator::Positive => OperatorCode::Positive,
        UnaryOperator::Negate => OperatorCode::Negative,
        UnaryOperator::LogicalNot => OperatorCode::Not,
        _ => OperatorCode::Other,
    }
}

pub(crate) fn update_operator_code(operator: UpdateOperator, prefix: bool) -> OperatorCode {
    match (operator, prefix) {
        (UpdateOperator::Increment, true) => OperatorCode::PreIncrement,
        (UpdateOperator::Decrement, true) => OperatorCode::PreDecrement,
        (UpdateOperator::Decrement, false) => OperatorCode::PostDecrement,
        _ => OperatorCode::Other,
    }
}
pub(crate) fn assignment_operator(op: AssignmentOperator) -> &'static str {
    use AssignmentOperator::*;
    match op {
        Assign => "=",
        AddAssign => "+=",
        SubtractAssign => "-=",
        MultiplyAssign => "*=",
        DivideAssign => "/=",
        ModuloAssign => "%=",
        ExponentAssign => "**=",
        LeftShiftAssign => "<<=",
        RightShiftAssign => ">>=",
        UnsignedRightShiftAssign => ">>>=",
        BitwiseOrAssign => "|=",
        BitwiseXorAssign => "^=",
        BitwiseAndAssign => "&=",
        LogicalOrAssign => "||=",
        LogicalAndAssign => "&&=",
        NullishAssign => "??=",
    }
}
pub(crate) fn next_level(level: Level) -> Level {
    unsafe { std::mem::transmute((level as u8 + 1).min(Level::Primary as u8)) }
}
pub(crate) fn is_identifier_continue_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$' || b >= 0x80
}

pub(crate) fn is_identifier(name: &[u8]) -> bool {
    let Ok(name) = std::str::from_utf8(name) else {
        return false;
    };
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '$' || first == '_' || is_id_start(first))
        && chars.all(|c| c == '$' || c == '\u{200c}' || c == '\u{200d}' || is_id_continue(c))
}

pub(crate) fn is_identifier_es5_and_esnext(name: &[u8]) -> bool {
    is_identifier(name)
        && std::str::from_utf8(name).is_ok_and(|name| {
            name.chars().all(|character| {
                // esbuild uses the intersection of ES5's Unicode tables and
                // the current ECMAScript tables here. Its intersection table
                // contains only BMP code points.
                character <= '\u{ffff}' && character != '\u{30fb}' && character != '\u{ff65}'
            })
        })
}

pub(crate) fn expr_was_parenthesized(ast: &Ast<'_>, span: Span) -> bool {
    let source = ast.source();
    let mut start = span.start as usize;
    let mut end = span.end as usize;
    while start > 0 && source[start - 1].is_ascii_whitespace() {
        start -= 1;
    }
    while source.get(end).is_some_and(u8::is_ascii_whitespace) {
        end += 1;
    }
    if start == 0 || source[start - 1] != b'(' || source.get(end) != Some(&b')') {
        return false;
    }
    let mut before_open = start - 1;
    while before_open > 0 && source[before_open - 1].is_ascii_whitespace() {
        before_open -= 1;
    }
    before_open == 0
        || !matches!(
            source[before_open - 1],
            b')' | b']' | b'_' | b'$' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z'
        )
}

pub(crate) fn import_declaration_has_empty_clause(
    ast: &Ast<'_>,
    declaration: ImportDeclaration,
) -> bool {
    let source = ast.source();
    let start = declaration.span(ast).start as usize;
    let end = declaration.source(ast).span(ast).start as usize;
    source[start..end].windows(2).any(|window| window == b"{}")
}

pub(crate) fn push_code_point_escape(out: &mut ArenaVec<'_, u8>, code_point: u32) {
    out.extend_from_slice(b"\\u{");
    let mut shift = 28;
    while shift > 0 && code_point >> shift == 0 {
        shift -= 4;
    }
    loop {
        out.push(HEX[((code_point >> shift) & 15) as usize]);
        if shift == 0 {
            break;
        }
        shift -= 4;
    }
    out.push(b'}');
}

pub(crate) fn module_name_bytes<'a>(ast: &'a Ast<'a>, n: ModuleExportName) -> &'a [u8] {
    match ast.module_export_name_data(n) {
        ModuleExportNameData::IdentifierName(i) => ast.string(i.name(ast)),
        ModuleExportNameData::StringLiteral(s) => ast.string(s.value(ast)),
    }
}

pub(crate) fn token_offset_after_decorators(
    ast: &Ast<'_>,
    fallback: u32,
    decorators: TypedSubRange<Decorator>,
    end: u32,
) -> u32 {
    let mut offset = decorators
        .last()
        .map(|id| ast.get_node_in_sub_range(id).span(ast).end)
        .unwrap_or(fallback) as usize;
    let end = end as usize;
    let source = ast.source();
    while offset < end {
        match source[offset] {
            byte if byte.is_ascii_whitespace() => offset += 1,
            b'/' if source.get(offset + 1) == Some(&b'/') => {
                offset += 2;
                while offset < end && !matches!(source[offset], b'\r' | b'\n') {
                    offset += 1;
                }
            }
            b'/' if source.get(offset + 1) == Some(&b'*') => {
                offset += 2;
                while offset + 1 < end && !(source[offset] == b'*' && source[offset + 1] == b'/') {
                    offset += 1;
                }
                offset = (offset + 2).min(end);
            }
            _ => break,
        }
    }
    offset as u32
}

pub(crate) fn import_attribute_offsets(
    ast: &Ast<'_>,
    attrs: TypedSubRange<ImportAttribute>,
) -> Option<(u32, u32, u32, bool)> {
    let first = ast.get_node_in_sub_range(attrs.first()?);
    let last = ast.get_node_in_sub_range(attrs.last()?);
    let source = ast.source();
    let first_start = first.span(ast).start as usize;
    let open = source[..first_start]
        .iter()
        .rposition(|&byte| byte == b'{')?;
    let with = source[..open]
        .windows(b"with".len())
        .rposition(|window| window == b"with");
    let assert = source[..open]
        .windows(b"assert".len())
        .rposition(|window| window == b"assert");
    let (keyword, is_assert) = match (with, assert) {
        (Some(with), Some(assert)) if assert > with => (assert, true),
        (Some(with), _) => (with, false),
        (_, Some(assert)) => (assert, true),
        _ => return None,
    };
    let close = source[last.span(ast).end as usize..]
        .iter()
        .position(|&byte| byte == b'}')?
        + last.span(ast).end as usize;
    Some((keyword as u32, open as u32, close as u32, is_assert))
}

pub(crate) fn empty_import_attribute_offsets(
    ast: &Ast<'_>,
    owner: Span,
    search_start: u32,
) -> Option<(u32, u32, u32, bool)> {
    let source = ast.source();
    let start = search_start as usize;
    let end = owner.end as usize;
    let tail = &source[start..end];
    let with = tail
        .windows(b"with".len())
        .position(|window| window == b"with")
        .map(|offset| start + offset);
    let assert = tail
        .windows(b"assert".len())
        .position(|window| window == b"assert")
        .map(|offset| start + offset);
    let (keyword, is_assert) = match (with, assert) {
        (Some(with), Some(assert)) if assert < with => (assert, true),
        (Some(with), _) => (with, false),
        (_, Some(assert)) => (assert, true),
        _ => return None,
    };
    if is_assert
        && source[start..keyword]
            .iter()
            .any(|&byte| matches!(byte, b'\r' | b'\n'))
    {
        return None;
    }
    let open = source[keyword..end].iter().position(|&byte| byte == b'{')? + keyword;
    let close = source[open..end].iter().position(|&byte| byte == b'}')? + open;
    Some((keyword as u32, open as u32, close as u32, is_assert))
}

pub(crate) fn can_omit_arrow_parens(ast: &Ast<'_>, params: FormalParameters) -> bool {
    if params.items(ast).len() != 1 || params.rest(ast).is_some() {
        return false;
    }
    let item = params.items(ast).get_node(ast, 0).unwrap();
    let FormalParameterItemData::FormalParameter(param) = ast.formal_parameter_item_data(item)
    else {
        return false;
    };
    let FormalParameterPatternData::BindingPattern(pattern) =
        ast.formal_parameter_pattern_data(param.pattern(ast))
    else {
        return false;
    };
    let BindingPatternData::BindingIdentifier(identifier) = ast.binding_pattern_data(pattern)
    else {
        return false;
    };
    identifier.decorators(ast).is_empty()
        && identifier.type_annotation(ast).is_none()
        && !identifier.optional(ast)
}

pub(crate) fn is_jsx_tag_single_line(ast: &Ast<'_>, opening: JsxOpeningElement) -> bool {
    let span = opening.span(ast);
    let source = &ast.source()[span.start as usize..span.end as usize];
    let mut quote = 0;
    let mut brace_depth: u32 = 0;
    let mut index = 0;
    while index < source.len() {
        let byte = source[index];
        if quote != 0 {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == quote {
                quote = 0;
            }
        } else {
            match byte {
                b'\'' | b'"' => quote = byte,
                b'{' => brace_depth += 1,
                b'}' => brace_depth = brace_depth.saturating_sub(1),
                b'\n' | b'\r' if brace_depth == 0 => return false,
                _ => {}
            }
        }
        index += 1;
    }
    true
}

pub(crate) fn export_default_expr_was_parenthesized(
    ast: &Ast<'_>,
    declaration: ExportDefaultDeclaration,
    expression: Expr,
) -> bool {
    let source = ast.source();
    let start = declaration.span(ast).start as usize;
    let end = expression.span(ast).start as usize;
    if start > end {
        return false;
    }
    let prefix = &source[start..end];
    let Some(default_end) = prefix
        .windows(b"default".len())
        .position(|window| window == b"default")
        .map(|index| index + b"default".len())
    else {
        return false;
    };
    prefix[default_end..]
        .iter()
        .find(|byte| !byte.is_ascii_whitespace())
        == Some(&b'(')
}

pub(crate) fn binding_pattern_identifier_name<'a>(
    ast: &'a Ast<'a>,
    binding: BindingPattern,
) -> Option<&'a [u8]> {
    match ast.binding_pattern_data(binding) {
        BindingPatternData::BindingIdentifier(identifier) => Some(ast.string(identifier.name(ast))),
        BindingPatternData::SimpleAssignmentTarget(target) => {
            match ast.simple_assignment_target_data(target) {
                SimpleAssignmentTargetData::IdentifierReference(identifier) => {
                    Some(ast.string(identifier.name(ast)))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub(crate) fn decorator_omit_newline_after(ast: &Ast<'_>, decorator: Decorator) -> bool {
    let source = ast.source();
    let mut index = decorator.span(ast).end as usize;
    while let Some(&byte) = source.get(index) {
        if byte == b'\n' || byte == b'\r' {
            return false;
        }
        if !byte.is_ascii_whitespace() {
            return true;
        }
        index += 1;
    }
    false
}

pub(crate) fn decorator_expression_needs_parentheses(ast: &Ast<'_>, mut expression: Expr) -> bool {
    loop {
        match ast.expr_data(expression) {
            ExprData::IdentifierReference(_) => return false,
            ExprData::CallExpression(call) => expression = call.callee(ast),
            ExprData::MemberExpression(member) if !member.computed(ast) => {
                let name = match ast.property_key_data(member.property(ast)) {
                    PropertyKeyData::IdentifierName(identifier) => ast.string(identifier.name(ast)),
                    PropertyKeyData::PrivateIdentifier(identifier) => {
                        ast.string(identifier.name(ast))
                    }
                    _ => return true,
                };
                if !is_identifier_es5_and_esnext(name) {
                    return true;
                }
                expression = member.object(ast);
            }
            _ => return true,
        }
    }
}

pub(crate) fn member_object_was_parenthesized(ast: &Ast<'_>, member: MemberExpression) -> bool {
    let source = ast.source();
    let mut index = member.object(ast).span(ast).start as usize;
    loop {
        while index > 0 && source[index - 1].is_ascii_whitespace() {
            index -= 1;
        }
        if index >= 2 && &source[index - 2..index] == b"*/" {
            let Some(start) = source[..index - 2]
                .windows(2)
                .rposition(|window| window == b"/*")
            else {
                return false;
            };
            index = start;
            continue;
        }
        return index > 0 && source[index - 1] == b'(';
    }
}

pub(crate) fn conditional_in_was_reassociated(
    ast: &Ast<'_>,
    binary: BinaryExpression,
) -> Option<ConditionalExpression> {
    if binary.operator(ast) != BinaryOperator::In {
        return None;
    }
    let ExprData::ConditionalExpression(conditional) = ast.expr_data(binary.left(ast)) else {
        return None;
    };
    let between = &ast.source()
        [conditional.span(ast).end as usize..binary.right(ast).span(ast).start as usize];
    (!between.contains(&b')')).then_some(conditional)
}

pub(crate) fn template_contains_closing_script(ast: &Ast<'_>, template: TemplateLiteral) -> bool {
    template.quasis(ast).iter().any(|id| {
        let quasi = ast.get_node_in_sub_range(id);
        ast.string(quasi.raw(ast))
            .windows(8)
            .any(|window| window.eq_ignore_ascii_case(b"</script"))
    })
}

pub(crate) fn count_templates_with_closing_script(ast: &Ast<'_>, root: Program) -> usize {
    // Avoid walking the AST at all in the overwhelmingly common case. This is
    // also important for the printer's stack-safety contract: a source file
    // containing a deeply nested binary expression must not be recursively
    // traversed just to discover that there are no templates to lower.
    if !ast
        .source()
        .windows(8)
        .any(|window| window.eq_ignore_ascii_case(b"</script"))
    {
        return 0;
    }

    struct Counter<'ast, 'allocator> {
        ast: &'ast Ast<'allocator>,
        count: usize,
    }

    impl<'ast, 'allocator> Visit<'allocator> for Counter<'ast, 'allocator> {
        fn ast(&self) -> &Ast<'allocator> {
            self.ast
        }

        fn visit_tagged_template_expression(&mut self, node: TaggedTemplateExpression) {
            if template_contains_closing_script(self.ast, node.quasi(self.ast)) {
                self.count += 1;
            }
            <TaggedTemplateExpression as VisitWith<'allocator, Self>>::visit_children_with(
                node, self,
            );
        }
    }

    let mut counter = Counter { ast, count: 0 };
    root.visit_with(&mut counter);
    counter.count
}

pub(crate) fn append_lowered_template_name(output: &mut ArenaVec<'_, u8>, index: usize) {
    const IDENTIFIER_CHARS: &[u8; 52] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    output.extend_from_slice(&[b'_', IDENTIFIER_CHARS[index % IDENTIFIER_CHARS.len()]]);
    if index >= IDENTIFIER_CHARS.len() {
        let mut digits = [0u8; 20];
        let mut value = index / IDENTIFIER_CHARS.len() + 1;
        let mut start = digits.len();
        while value != 0 {
            start -= 1;
            digits[start] = b'0' + (value % 10) as u8;
            value /= 10;
        }
        output.extend_from_slice(&digits[start..]);
    }
}

pub(crate) fn append_lowered_template_prefix(output: &mut ArenaVec<'_, u8>, template_count: usize) {
    if template_count == 0 {
        return;
    }
    output.extend_from_slice(b"import { __template } from \"<runtime>\";\nvar ");
    for index in 0..template_count {
        if index != 0 {
            output.extend_from_slice(b", ");
        }
        append_lowered_template_name(output, index);
    }
    output.extend_from_slice(b";\n");
}

pub(crate) fn remove_underscores(raw: &[u8]) -> Vec<u8> {
    raw.iter().copied().filter(|&b| b != b'_').collect()
}

pub(crate) fn parse_numeric_literal(raw: &[u8]) -> Option<f64> {
    let clean = remove_underscores(raw);
    let text = std::str::from_utf8(&clean).ok()?;
    let lower = text.to_ascii_lowercase();
    if let Some(digits) = lower.strip_prefix("0x") {
        return u128::from_str_radix(digits, 16)
            .ok()
            .map(|value| value as f64);
    }
    if let Some(digits) = lower.strip_prefix("0o") {
        return u128::from_str_radix(digits, 8)
            .ok()
            .map(|value| value as f64);
    }
    if let Some(digits) = lower.strip_prefix("0b") {
        return u128::from_str_radix(digits, 2)
            .ok()
            .map(|value| value as f64);
    }
    text.parse().ok()
}

pub(crate) fn format_float(value: f64) -> Vec<u8> {
    value.to_string().into_bytes()
}

pub(crate) fn quote_into(out: &mut ArenaVec<'_, u8>, value: &[u8], quote: u8, ascii_only: bool) {
    out.push(quote);
    let mut i = 0;
    while i < value.len() {
        let (cp, size) = next_wtf8_code_point(value, i);
        i += size;
        match cp {
            0 => {
                // Use "\x00" before a digit so the escape cannot consume the
                // digit as part of a legacy octal escape.
                if i < value.len() && value[i].is_ascii_digit() {
                    out.extend_from_slice(b"\\x00")
                } else {
                    out.extend_from_slice(b"\\0")
                }
            }
            7 => out.extend_from_slice(b"\\x07"),
            8 => out.extend_from_slice(b"\\b"),
            9 => out.push(b'\t'),
            12 => out.extend_from_slice(b"\\f"),
            10 => {
                if quote == b'`' {
                    out.push(b'\n')
                } else {
                    out.extend_from_slice(b"\\n")
                }
            }
            13 => out.extend_from_slice(b"\\r"),
            11 => out.extend_from_slice(b"\\v"),
            27 => out.extend_from_slice(b"\\x1B"),
            92 => out.extend_from_slice(b"\\\\"),
            47 if out.last() == Some(&b'<')
                && value
                    .get(i..i.saturating_add(6))
                    .is_some_and(|tail| tail.eq_ignore_ascii_case(b"script")) =>
            {
                // Avoid generating the sequence "</script" inside an inline
                // script.
                out.extend_from_slice(b"\\/")
            }
            // U+2028 and U+2029 are JavaScript line terminators and must be
            // escaped inside string literals. Escape the BOM as well.
            0x2028 => out.extend_from_slice(b"\\u2028"),
            0x2029 => out.extend_from_slice(b"\\u2029"),
            0xfeff => out.extend_from_slice(b"\\uFEFF"),
            36 if quote == b'`' && i < value.len() && value[i] == b'{' => {
                // Avoid starting a template substitution.
                out.extend_from_slice(b"\\$")
            }
            x if x == quote as u32 => {
                out.push(b'\\');
                out.push(quote)
            }
            x if x <= 0x7e => out.push(x as u8),
            x if (0xd800..=0xdfff).contains(&x) => push_u16_escape(out, x as u16),
            x if ascii_only && x <= 0xff => {
                out.extend_from_slice(b"\\x");
                out.push(HEX[(x >> 4) as usize]);
                out.push(HEX[(x & 15) as usize]);
            }
            x if ascii_only && x <= 0xffff => push_u16_escape(out, x as u16),
            x if ascii_only => push_code_point_escape(out, x),
            x => {
                let mut buf = [0; 4];
                out.extend_from_slice(
                    char::from_u32(x)
                        .unwrap_or('\u{fffd}')
                        .encode_utf8(&mut buf)
                        .as_bytes(),
                );
            }
        }
    }
    out.push(quote);
}

fn next_wtf8_code_point(bytes: &[u8], i: usize) -> (u32, usize) {
    // AST strings use WTF-8 so lone UTF-16 surrogates survive parsing. Go's
    // printer receives UTF-16 directly and escapes these code units verbatim.
    if bytes[i] == 0xed
        && let (Some(&b1 @ 0xa0..=0xbf), Some(&b2 @ 0x80..=0xbf)) =
            (bytes.get(i + 1), bytes.get(i + 2))
    {
        return (
            (((bytes[i] & 0x0f) as u32) << 12) | (((b1 & 0x3f) as u32) << 6) | (b2 & 0x3f) as u32,
            3,
        );
    }
    next_code_point(bytes, i)
}
pub(crate) fn push_u16_escape(out: &mut ArenaVec<'_, u8>, c: u16) {
    out.extend_from_slice(&[
        b'\\',
        b'u',
        HEX[(c >> 12) as usize],
        HEX[((c >> 8) & 15) as usize],
        HEX[((c >> 4) & 15) as usize],
        HEX[(c & 15) as usize],
    ]);
}
pub(crate) fn next_code_point(bytes: &[u8], i: usize) -> (u32, usize) {
    let b = bytes[i];
    if b < 0x80 {
        return (b as u32, 1);
    }

    // Match Go's UTF-8 range decoding: malformed, overlong, surrogate, and
    // out-of-range encodings each consume one byte and produce U+FFFD.
    let continuation = |offset: usize| {
        bytes
            .get(i + offset)
            .copied()
            .filter(|byte| byte & 0xc0 == 0x80)
    };
    match b {
        0xc2..=0xdf => {
            let Some(b1) = continuation(1) else {
                return (0xfffd, 1);
            };
            ((((b & 0x1f) as u32) << 6) | (b1 & 0x3f) as u32, 2)
        }
        0xe0..=0xef => {
            let Some(b1) = continuation(1) else {
                return (0xfffd, 1);
            };
            let Some(b2) = continuation(2) else {
                return (0xfffd, 1);
            };
            if (b == 0xe0 && b1 < 0xa0) || (b == 0xed && b1 >= 0xa0) {
                return (0xfffd, 1);
            }
            (
                (((b & 0x0f) as u32) << 12) | (((b1 & 0x3f) as u32) << 6) | (b2 & 0x3f) as u32,
                3,
            )
        }
        0xf0..=0xf4 => {
            let Some(b1) = continuation(1) else {
                return (0xfffd, 1);
            };
            let Some(b2) = continuation(2) else {
                return (0xfffd, 1);
            };
            let Some(b3) = continuation(3) else {
                return (0xfffd, 1);
            };
            if (b == 0xf0 && b1 < 0x90) || (b == 0xf4 && b1 >= 0x90) {
                return (0xfffd, 1);
            }
            (
                (((b & 0x07) as u32) << 18)
                    | (((b1 & 0x3f) as u32) << 12)
                    | (((b2 & 0x3f) as u32) << 6)
                    | (b3 & 0x3f) as u32,
                4,
            )
        }
        _ => (0xfffd, 1),
    }
}

pub(crate) fn is_legal_comment(ast: &Ast<'_>, comment: Comment) -> bool {
    let value = ast.string(comment.value(ast.source()));
    ast.source()
        .get(comment.span.start as usize..comment.span.start as usize + 3)
        .is_some_and(|prefix| prefix == b"/*!" || prefix == b"//!")
        || value.windows(8).any(|window| window == b"@license")
        || value.windows(9).any(|window| window == b"@preserve")
}
