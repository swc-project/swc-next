use swc_next_ecma_ast::{Ast, Program};

use super::{JsxOptions, JsxRuntime};
use crate::jsx_pragma::{PragmaKind, pragmas};

pub(super) fn resolve_options(
    program: Program,
    ast: &Ast<'_>,
    mut options: JsxOptions,
) -> JsxOptions {
    for (kind, value) in pragmas(program, ast) {
        match kind {
            PragmaKind::Jsx => options.pragma = Some(value.to_owned()),
            PragmaKind::Runtime => match value {
                "classic" => options.runtime = JsxRuntime::Classic,
                "automatic" => options.runtime = JsxRuntime::Automatic,
                _ => {}
            },
            PragmaKind::ImportSource => options.import_source = Some(value.to_owned()),
            PragmaKind::Fragment => options.pragma_frag = Some(value.to_owned()),
        }
    }
    options
}

#[cfg(test)]
mod tests {
    use crate::jsx_pragma::{PragmaKind, find_pragma};

    #[test]
    fn pragmas() {
        let (kind, value, remainder) = find_pragma("* @jsx h\n * @jsxRuntime classic").unwrap();
        assert!(matches!(kind, PragmaKind::Jsx));
        assert_eq!(value, "h");

        let (kind, value, _) = find_pragma(remainder).unwrap();
        assert!(matches!(kind, PragmaKind::Runtime));
        assert_eq!(value, "classic");
    }
}
