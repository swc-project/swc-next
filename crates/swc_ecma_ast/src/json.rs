//! JSON printers for the Rust AST, enabled by the `json` feature.
//!
//! Nodes are objects with `type` (the Rust type name), `span`, and every declared
//! field under its Rust name. Node unions print their concrete node; scalar enums
//! print their Rust variant name. ESTree annotations do not affect this format.
//! Ranges print as arrays, absent options as `null`, and semantic IDs as zero-based
//! indices. Spans retain their original byte offsets. Strings are decoded from the
//! AST pool, preserving lone UTF-16 surrogates with `\uXXXX` escapes. Non-finite
//! floats print as `null`.
//!
//! ```
//! use swc_next_allocator::Allocator;
//! use swc_next_ecma_ast::{Ast, DUMMY_SP, json::JsonPrint};
//!
//! let allocator = Allocator::new();
//! let mut ast = Ast::new("", &allocator);
//! let node = ast.boolean_literal(DUMMY_SP, true);
//! assert_eq!(node.to_json(&ast),
//!     r#"{"type":"BooleanLiteral","span":{"start":0,"end":0},"value":true}"#);
//! ```

use std::fmt::Write;

use swc_next_allocator::{Idx, wtf8::Wtf8};

use crate::{
    Ast, AstUtf8, AstWtf8, ExtraDataValue, NodeId, ReferenceId, ScopeId, Span, SubRange, SymbolId,
};

/// Prints an AST value as compact JSON using its owning AST's storage.
///
/// Node handles and ranges must belong to `ast`. Printing recursively follows AST
/// edges and requires an acyclic tree.
pub trait JsonPrint: Copy {
    /// Appends one JSON value to `output`.
    fn print_json(self, ast: &Ast<'_>, output: &mut String);

    /// Returns this value as a JSON string.
    fn to_json(self, ast: &Ast<'_>) -> String {
        let mut output = String::new();
        self.print_json(ast, &mut output);
        output
    }
}

impl Ast<'_> {
    /// Prints the root node as JSON.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root.
    pub fn to_json(&self) -> String {
        self.root_node_id().to_json(self)
    }
}

impl JsonPrint for NodeId {
    fn print_json(self, ast: &Ast<'_>, output: &mut String) {
        ast.kind_data(self).print_json(ast, output);
    }
}

impl<T: JsonPrint> JsonPrint for Option<T> {
    fn print_json(self, ast: &Ast<'_>, output: &mut String) {
        match self {
            Some(value) => value.print_json(ast, output),
            None => output.push_str("null"),
        }
    }
}

impl<T: ExtraDataValue + JsonPrint> JsonPrint for SubRange<T> {
    fn print_json(self, ast: &Ast<'_>, output: &mut String) {
        output.push('[');
        for (index, id) in self.iter().enumerate() {
            if index != 0 {
                output.push(',');
            }
            ast.get_node_in_sub_range(id).print_json(ast, output);
        }
        output.push(']');
    }
}

impl JsonPrint for Span {
    fn print_json(self, ast: &Ast<'_>, output: &mut String) {
        output.push_str("{\"start\":");
        self.start.print_json(ast, output);
        output.push_str(",\"end\":");
        self.end.print_json(ast, output);
        output.push('}');
    }
}

impl JsonPrint for AstUtf8 {
    fn print_json(self, ast: &Ast<'_>, output: &mut String) {
        print_string(Wtf8::from_str(ast.get_utf8(self)), output);
    }
}

impl JsonPrint for AstWtf8 {
    fn print_json(self, ast: &Ast<'_>, output: &mut String) {
        print_string(ast.get_wtf8(self), output);
    }
}

fn print_string(value: &Wtf8, output: &mut String) {
    output.push('"');
    for point in value.code_points() {
        match point.to_u32() {
            0x22 => output.push_str("\\\""),
            0x5c => output.push_str("\\\\"),
            0x08 => output.push_str("\\b"),
            0x0c => output.push_str("\\f"),
            0x0a => output.push_str("\\n"),
            0x0d => output.push_str("\\r"),
            0x09 => output.push_str("\\t"),
            value @ (0..=0x1f | 0xd800..=0xdfff) => write!(output, "\\u{value:04x}").unwrap(),
            _ => output.push(point.to_char().unwrap()),
        }
    }
    output.push('"');
}

macro_rules! print_value {
    ($($ty:ty),+ $(,)?) => {
        $(impl JsonPrint for $ty {
            fn print_json(self, _ast: &Ast<'_>, output: &mut String) {
                write!(output, "{self}").unwrap();
            }
        })+
    };
}

print_value!(
    bool, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

macro_rules! print_float {
    ($($ty:ty),+ $(,)?) => {
        $(impl JsonPrint for $ty {
            fn print_json(self, _ast: &Ast<'_>, output: &mut String) {
                if self.is_finite() {
                    // Debug uses the shortest round-trip representation, including
                    // exponent notation for very large and very small values.
                    write!(output, "{self:?}").unwrap();
                } else {
                    output.push_str("null");
                }
            }
        })+
    };
}

print_float!(f32, f64);

macro_rules! print_index {
    ($($ty:ty),+ $(,)?) => {
        $(impl JsonPrint for $ty {
            fn print_json(self, ast: &Ast<'_>, output: &mut String) {
                self.index().print_json(ast, output);
            }
        })+
    };
}

print_index!(ScopeId, SymbolId, ReferenceId);

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use swc_next_allocator::{Allocator, wtf8::Wtf8Buf};

    use super::*;
    use crate::*;

    fn value(node: impl JsonPrint, ast: &Ast<'_>) -> Value {
        serde_json::from_str(&node.to_json(ast)).unwrap()
    }

    #[test]
    fn nodes() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let node = ast.boolean_literal(Span::new(2, 6), true);
        let expected =
            json!({"type": "BooleanLiteral", "span": {"start": 2, "end": 6}, "value": true});
        assert_eq!(value(node, &ast), expected);
        assert_eq!(value(Expr::BooleanLiteral(node), &ast), expected);
        assert_eq!(
            value(ast.expr_data(Expr::BooleanLiteral(node)), &ast),
            expected
        );
        assert_eq!(value(node.node_id(), &ast), expected);
        assert_eq!(value(ast.kind_data(node.node_id()), &ast), expected);

        let node = ast.return_statement(DUMMY_SP, Some(Expr::BooleanLiteral(node)));
        let result = value(node, &ast);
        assert_eq!(result["argument"], expected);

        let node = ast.return_statement(DUMMY_SP, None);
        let result = value(node, &ast);
        assert_eq!(result["argument"], Value::Null);

        let body = ast.add_sub_range([Stmt::ReturnStatement(node)]);
        let program = ast.program(DUMMY_SP, SubRange::empty(), body, None);
        let result = value(program, &ast);
        assert_eq!(result["directives"], json!([]));
        assert_eq!(result["body"], json!([value(node, &ast)]));
        assert_eq!(result["hashbang"], Value::Null);
        ast.set_root_program(program);
        assert_eq!(ast.to_json(), program.to_json(&ast));
        ast.set_root_statement(Stmt::ReturnStatement(node));
        assert_eq!(ast.to_json(), node.to_json(&ast));
    }

    #[test]
    fn ranges() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let element = ast.boolean_literal(DUMMY_SP, false);
        let argument = Some(Argument::Expr(Expr::BooleanLiteral(element)));
        let elements = ast.add_sub_range([None, argument, None]);
        let node = ast.array_expression(DUMMY_SP, elements);
        let result = value(node, &ast);
        assert_eq!(
            result["elements"],
            json!([null, value(element, &ast), null])
        );

        let node = ast.array_expression(DUMMY_SP, SubRange::empty());
        let result = value(node, &ast);
        assert_eq!(result["elements"], json!([]));
    }

    #[test]
    fn native_fields() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let node = ast.variable_declaration(DUMMY_SP, VariableKind::Const, SubRange::empty(), true);
        let result = value(Stmt::Declaration(Decl::VariableDeclaration(node)), &ast);
        assert_eq!(
            result,
            json!({"type": "VariableDeclaration", "span": {"start": 0, "end": 0}, "kind": "Const", "declarators": [], "declare": true})
        );

        let right = ast.expr_boolean_literal(DUMMY_SP, true);
        let body = ast.stmt_empty_statement(DUMMY_SP);
        let left = ForStatementLeft::VariableDeclaration(node);
        let node = ast.for_of_statement(DUMMY_SP, left, right, body, true);
        let result = value(node, &ast);
        assert_eq!(result["await"], true);
        assert!(!result.as_object().unwrap().contains_key("r#await"));

        let pattern = ast.add_utf8("a+");
        let flags = ast.add_utf8("g");
        let node = ast.reg_exp_literal(DUMMY_SP, pattern, flags);
        let result = value(node, &ast);
        assert_eq!(result["type"], "RegExpLiteral");
        assert_eq!(result["pattern"], "a+");
        assert_eq!(result["flags"], "g");

        let element = ast.ts_type_ts_number_keyword(DUMMY_SP);
        let node = ast.ts_type_ts_array_type(DUMMY_SP, element);
        let result = value(node, &ast);
        assert_eq!(result["type"], "TsArrayType");
        assert_eq!(result["element_type"]["type"], "TsNumberKeyword");

        let text = ast.add_utf8("hello");
        let node = ast.jsx_text(DUMMY_SP, text);
        let result = value(node, &ast);
        assert_eq!(result["type"], "JsxText");
        assert_eq!(result["value"], "hello");
    }

    #[test]
    fn strings() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("源🌍", &allocator);
        let text = ast.source_slice(0, ast.source().len() as u32);
        assert_eq!(value(text, &ast), "源🌍");

        let text = ast.add_utf8("\"\\\n\r\t\u{08}\u{0c}\0\u{1f}中文😀");
        assert_eq!(value(text, &ast), "\"\\\n\r\t\u{08}\u{0c}\0\u{1f}中文😀");

        let text = ast.add_wtf8(&Wtf8Buf::from_ill_formed_utf16(&[0xd800, 0x61, 0xdc00]));
        assert_eq!(text.to_json(&ast), r#""\ud800a\udc00""#);

        let text = ast.add_wtf8(&Wtf8Buf::from_ill_formed_utf16(&[0xd83d, 0xde00]));
        assert_eq!(value(text, &ast), "😀");

        let node = ast.string_literal(DUMMY_SP, text, ast.empty_utf8());
        let result = value(node, &ast);
        assert_eq!(result["value"], "😀");
        assert_eq!(result["raw"], "");
    }

    #[test]
    fn scalars() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let node = ast.numeric_literal(DUMMY_SP, 1.25, None);
        let result = value(node, &ast);
        assert_eq!(result["value"], 1.25);
        assert_eq!(result["raw"], Value::Null);

        let node = ast.numeric_literal(DUMMY_SP, f64::INFINITY, None);
        assert_eq!(value(node, &ast)["value"], Value::Null);
        assert_eq!(value(f64::NEG_INFINITY, &ast), Value::Null);
        assert_eq!(value(f64::NAN, &ast), Value::Null);
        assert_eq!(value(f64::MAX, &ast), json!(f64::MAX));
        assert_eq!(value(f64::MIN_POSITIVE, &ast), json!(f64::MIN_POSITIVE));
        assert_eq!((-0.0_f64).to_json(&ast), "-0.0");

        assert_eq!(value(BinaryOperator::Add, &ast), "Add");
        assert_eq!(value(Accessibility::None, &ast), "None");
        assert_eq!(value(SourceType::Module, &ast), "Module");
        assert_eq!(value(NodeKind::NumericLiteral, &ast), "NumericLiteral");
        assert_eq!(value(ScopeId::ROOT, &ast), 0);
        assert_eq!(value(SymbolId::from_usize(7), &ast), 7);

        let mut output = String::from("[");
        42_u32.print_json(&ast, &mut output);
        output.push(']');
        assert_eq!(serde_json::from_str::<Value>(&output).unwrap(), json!([42]));
    }
}
