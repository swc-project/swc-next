use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};

use swc_next_ecma_ast::*;
use unicode_id_start::{is_id_continue, is_id_start};

const HEX: &[u8; 16] = b"0123456789ABCDEF";

pub(crate) struct AstAdapterState {
    class_decorators_were_printed: bool,
}

impl AstAdapterState {
    pub(crate) fn new() -> Self {
        Self {
            class_decorators_were_printed: false,
        }
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
