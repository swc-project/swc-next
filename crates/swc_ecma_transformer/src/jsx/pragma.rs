//! File-header JSX pragmas shared by syntax transforms.
use swc_next_ecma_ast::{Ast, GetSpan, Program};

pub(crate) fn pragmas<'a>(
    program: Program,
    ast: &'a Ast<'_>,
) -> impl Iterator<Item = (PragmaKind, &'a str)> {
    let mut boundary = u32::MAX;
    if let Some(id) = program.directives(ast).iter().next() {
        boundary = boundary.min(ast.get_node_in_sub_range(id).span(ast).start);
    }
    if let Some(id) = program.body(ast).iter().next() {
        boundary = boundary.min(ast.get_node_in_sub_range(id).span(ast).start);
    }
    let source = ast.source().as_bytes();
    ast.comments()
        .iter()
        .take_while(move |comment| comment.span.end <= boundary)
        .flat_map(move |comment| {
            let mut text = ast.get_utf8(comment.value(source));
            std::iter::from_fn(move || {
                let (kind, value, rest) = find_pragma(text)?;
                text = rest;
                Some((kind, value))
            })
        })
}

#[derive(Clone, Copy)]
pub(crate) enum PragmaKind {
    Jsx,
    Runtime,
    ImportSource,
    Fragment,
}

pub(crate) fn find_pragma(mut source: &str) -> Option<(PragmaKind, &str, &str)> {
    loop {
        let at = source.find('@')?;
        let boundary = at == 0
            || matches!(
                source.as_bytes()[at - 1],
                b' ' | b'\t' | b'\r' | b'\n' | b'*'
            );
        source = &source[at + 1..];
        if !boundary {
            continue;
        }

        let end = source
            .as_bytes()
            .iter()
            .position(|byte| byte.is_ascii_whitespace())
            .unwrap_or(source.len());
        let (name, rest) = source.split_at(end);
        source = rest;
        let kind = match name {
            "jsx" => PragmaKind::Jsx,
            "jsxRuntime" => PragmaKind::Runtime,
            "jsxImportSource" => PragmaKind::ImportSource,
            "jsxFrag" => PragmaKind::Fragment,
            _ => continue,
        };
        // Arguments belong to the same line and require a separator after the
        // complete tag. A malformed tag must not hide later valid pragmas.
        if !source.starts_with([' ', '\t']) {
            continue;
        }
        source = source.trim_start_matches([' ', '\t']);
        let end = source
            .as_bytes()
            .iter()
            .position(|byte| byte.is_ascii_whitespace())
            .unwrap_or(source.len());
        let (value, rest) = source.split_at(end);
        source = rest;
        if !value.is_empty() {
            return Some((kind, value, rest));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PragmaKind, find_pragma};

    #[test]
    fn pragma_boundaries() {
        assert!(find_pragma("@jsxFactory h").is_none());
        assert!(find_pragma("@jsxRuntimeautomatic").is_none());
        assert!(find_pragma("`@jsx h`").is_none());
        assert!(find_pragma("@jsx").is_none());

        let (kind, value, rest) = find_pragma("@jsx\th @jsxFrag Frag").unwrap();
        assert!(matches!(kind, PragmaKind::Jsx));
        assert_eq!(value, "h");

        let (kind, value, _) = find_pragma(rest).unwrap();
        assert!(matches!(kind, PragmaKind::Fragment));
        assert_eq!(value, "Frag");
    }

    #[test]
    fn malformed_pragmas() {
        let (kind, value, _) = find_pragma("@jsxFactory h @jsxRuntime automatic").unwrap();
        assert!(matches!(kind, PragmaKind::Runtime));
        assert_eq!(value, "automatic");

        let (kind, value, _) = find_pragma("@jsx\n * @jsx h").unwrap();
        assert!(matches!(kind, PragmaKind::Jsx));
        assert_eq!(value, "h");

        let (kind, value, _) = find_pragma("@jsx \r\n * @jsxImportSource preact").unwrap();
        assert!(matches!(kind, PragmaKind::ImportSource));
        assert_eq!(value, "preact");
    }
}
