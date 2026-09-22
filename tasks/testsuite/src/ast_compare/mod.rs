//! Structural comparison for parse/print round trips, independent of codegen.
//!
//! Compare reachable nodes and decoded strings, never arena IDs or source spans.
//! Compare the parsed structure directly, including parenthesis nodes when
//! present. Literal spellings may change; node kinds, flags, tagged-template raw
//! text, strict-mode directives, and optional-chain boundaries may not.

use swc_next_ecma_ast::*;

mod generated;

pub(crate) fn compare(left: &Ast<'_>, right: &Ast<'_>) -> Result<(), String> {
    let asts = AstPair { left, right };
    if left.source_type() != right.source_type() {
        return Err(format!(
            "source type: {:?} != {:?}",
            left.source_type(),
            right.source_type()
        ));
    }
    left.root_program().compare(right.root_program(), &asts)
}

struct AstPair<'a, 'b> {
    left: &'a Ast<'a>,
    right: &'b Ast<'b>,
}

impl AstPair<'_, '_> {
    fn nodes(&self, left: NodeId, right: NodeId) -> Result<(), String> {
        let left = self.left.kind_data(left);
        let right = self.right.kind_data(right);
        generated::compare_nodes(left, right, self)
    }
}

trait Compare: Sized {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String>;
}

fn compare_bigint(left: AstUtf8, right: AstUtf8, asts: &AstPair<'_, '_>) -> Result<(), String> {
    let left = asts.left.get_utf8(left);
    let right = asts.right.get_utf8(right);
    fn digits(text: &str) -> impl Iterator<Item = u8> + '_ {
        text.bytes().filter(|byte| *byte != b'_')
    }
    if digits(left).eq(digits(right)) {
        Ok(())
    } else {
        Err(format!("BigInt spelling: {left:?} != {right:?}"))
    }
}

fn compare_directive(left: AstUtf8, right: AstUtf8, asts: &AstPair<'_, '_>) -> Result<(), String> {
    // The decoded StringLiteral is compared separately. Only the exact source
    // spelling enables strict mode; escaped lookalikes must stay non-directives.
    let left = asts.left.get_utf8(left) == "use strict";
    let right = asts.right.get_utf8(right) == "use strict";
    left.compare(right, asts)
}

macro_rules! compare_value {
    ($ty:ty) => {
        impl Compare for $ty {
            fn compare(self, other: Self, _: &AstPair<'_, '_>) -> Result<(), String> {
                if self == other {
                    Ok(())
                } else {
                    Err(format!("value: {self:?} != {other:?}"))
                }
            }
        }
    };
}
use compare_value;

impl Compare for AstUtf8 {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String> {
        let left = asts.left.get_utf8(self);
        let right = asts.right.get_utf8(other);
        if left == right {
            Ok(())
        } else {
            Err(format!("value: {left:?} != {right:?}"))
        }
    }
}

impl Compare for AstWtf8 {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String> {
        let left = asts.left.get_wtf8(self);
        let right = asts.right.get_wtf8(other);
        if left == right {
            Ok(())
        } else {
            Err(format!("value: {left:?} != {right:?}"))
        }
    }
}

impl Compare for f64 {
    fn compare(self, other: Self, _: &AstPair<'_, '_>) -> Result<(), String> {
        // Preserve negative zero and compare NaN payloads deterministically.
        if self.to_bits() == other.to_bits() {
            Ok(())
        } else {
            Err(format!("value: {self:?} != {other:?}"))
        }
    }
}

impl<T: Compare> Compare for Option<T> {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String> {
        match (self, other) {
            (Some(left), Some(right)) => left.compare(right, asts),
            (None, None) => Ok(()),
            _ => Err("presence changed".into()),
        }
    }
}

impl<T: Compare + ExtraDataValue> Compare for SubRange<T> {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String> {
        if self.len() != other.len() {
            return Err("list length changed".into());
        }
        for (index, (left, right)) in self.iter().zip(other.iter()).enumerate() {
            let left = asts.left.get_node_in_sub_range(left);
            let right = asts.right.get_node_in_sub_range(right);
            left.compare(right, asts)
                .map_err(|error| format!("[{index}].{error}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use super::*;

    fn compare_sources(left: &str, right: &str) -> Result<(), String> {
        compare_sources_with_parens(left, right, false)
    }

    fn compare_sources_with_parens(
        left: &str,
        right: &str,
        preserve_parens: bool,
    ) -> Result<(), String> {
        let left_allocator = Allocator::new();
        let right_allocator = Allocator::new();
        let options = Options {
            lang: Lang::Tsx,
            preserve_parens,
            ..Options::default()
        };
        let left = Parser::init(&left_allocator, left, options, NoTokenParserConfig).parse();
        let right = Parser::init(&right_allocator, right, options, NoTokenParserConfig).parse();
        assert!(left.diagnostics.is_empty());
        assert!(right.diagnostics.is_empty());
        compare(&left.ast, &right.ast)
    }

    #[test]
    fn representation() {
        let result = compare_sources("const a = 0x10;", "const a = 16;");
        assert!(result.is_ok());

        let result = compare_sources("const a = '\\u0061';", "const a = \"a\";");
        assert!(result.is_ok());

        let result = compare_sources("(a + b) * c;", "((a + b)) * c;");
        assert!(result.is_ok());

        let result = compare_sources("type T = (A | B)[];", "type T = ((A | B))[];");
        assert!(result.is_ok());

        let result = compare_sources("const a = 1_000n;", "const a = 1000n;");
        assert!(result.is_ok());

        let result = compare_sources("type T = | number;", "type T = number;");
        assert!(result.is_err());

        let result = compare_sources("type T = & number;", "type T = number;");
        assert!(result.is_err());

        let result = compare_sources_with_parens("(a + b) * c;", "((a + b)) * c;", true);
        assert!(result.is_err());
    }

    #[test]
    fn properties() {
        let result = compare_sources("object.name;", "object['name'];");
        assert!(result.is_err());

        let result = compare_sources("object.name;", "object[name];");
        assert!(result.is_err());

        let result = compare_sources("const o = {name};", "const o = {name: name};");
        assert!(result.is_err());

        let result = compare_sources(
            "const o = {__proto__};",
            "const o = {__proto__: __proto__};",
        );
        assert!(result.is_err());

        let result = compare_sources("const {name: name} = object;", "const {name} = object;");
        assert!(result.is_err());

        let result = compare_sources("const {name = 1} = object;", "const {name} = object;");
        assert!(result.is_err());
    }

    #[test]
    fn templates() {
        let result = compare_sources("const a = 'a';", "const a = `a`;");
        assert!(result.is_err());

        let result = compare_sources("const a = `a`;", "const a = `\\u0061`;");
        assert!(result.is_err());

        let result = compare_sources("const a = `a`;", "const a = `${a}`;");
        assert!(result.is_err());

        let result = compare_sources("const a = tag`a`;", "const a = tag`\\u0061`;");
        assert!(result.is_err());
    }

    #[test]
    fn statement_and_jsx_syntax() {
        let result = compare_sources(";f();;", "f();");
        assert!(result.is_err());

        let result = compare_sources("; 'use strict';", "'use strict';");
        assert!(result.is_err());

        let result = compare_sources("const a = <A></A>;", "const a = <A />;");
        assert!(result.is_err());

        let result = compare_sources("const a = <A><B /></A>;", "const a = <A />;");
        assert!(result.is_err());

        let result = compare_sources("const a = <A<T> />;", "const a = <A />;");
        assert!(result.is_err());
    }

    #[test]
    fn structure() {
        let result = compare_sources("a + b;", "a - b;");
        assert!(matches!(result, Err(error) if error.contains("BinaryExpression.operator")));

        let result = compare_sources("(a + b) * c;", "a + b * c;");
        assert!(result.is_err());

        let result = compare_sources("f(a, b);", "f(b, a);");
        assert!(result.is_err());

        let result = compare_sources("[, a];", "[a, ,];");
        assert!(result.is_err());

        let result = compare_sources("a?.b.c;", "(a?.b).c;");
        assert!(result.is_err());

        let result = compare_sources("type T = (A | B)[];", "type T = A | B[];");
        assert!(result.is_err());
    }

    #[test]
    fn associative_grouping() {
        let result = compare_sources("const a = ((f(), g()), h());", "const a = (f(), g(), h());");
        assert!(result.is_err());

        let result = compare_sources("const a = ((f(), g()), h());", "const a = (f(), h(), g());");
        assert!(result.is_err());

        let result = compare_sources("type T = A | (B | C);", "type T = A | B | C;");
        assert!(result.is_err());

        let result = compare_sources("type T = (A & B) & C;", "type T = A & B & C;");
        assert!(result.is_err());

        let result = compare_sources("type T = A | (& (B | C));", "type T = A | B | C;");
        assert!(result.is_err());

        let result = compare_sources("type T = A & (| (B & C));", "type T = A & B & C;");
        assert!(result.is_err());

        let result = compare_sources("type T = (A | B) & C;", "type T = A | (B & C);");
        assert!(result.is_err());
    }

    #[test]
    fn values() {
        let result = compare_sources("const a = 1;", "const a = 2;");
        assert!(result.is_err());

        let result = compare_sources("'use strict';", "'use\\x20strict';");
        assert!(result.is_err());

        let result = compare_sources("const a = '\\uD800';", "const a = '\\uD801';");
        assert!(result.is_err());
    }
}
