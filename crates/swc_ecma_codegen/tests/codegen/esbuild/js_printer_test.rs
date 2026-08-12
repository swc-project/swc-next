// Ported from upstream esbuild's internal/js_printer/js_printer_test.go.
use crate::{
    expect_printed, expect_printed_ascii, expect_printed_jsx, expect_printed_jsx_ascii,
    expect_printed_jsx_minify, expect_printed_minify, expect_printed_minify_ascii,
    expect_printed_ts,
};

#[test]
fn test_number() {
    expect_printed("x = 1e-100", "x = 1e-100;\n");
    expect_printed("x = 1e-4", "x = 1e-4;\n");
    expect_printed("x = 1e-3", "x = 1e-3;\n");
    expect_printed("x = 1e-2", "x = 0.01;\n");
    expect_printed("x = 1e-1", "x = 0.1;\n");
    expect_printed("x = 1e0", "x = 1;\n");
    expect_printed("x = 1e1", "x = 10;\n");
    expect_printed("x = 1e2", "x = 100;\n");
    expect_printed("x = 1e3", "x = 1e3;\n");
    expect_printed("x = 1e4", "x = 1e4;\n");
    expect_printed("x = 1e100", "x = 1e100;\n");
    expect_printed_minify("x = 1e-100", "x=1e-100;");
    expect_printed_minify("x = 1e-5", "x=1e-5;");
    expect_printed_minify("x = 1e-4", "x=1e-4;");
    expect_printed_minify("x = 1e-3", "x=.001;");
    expect_printed_minify("x = 1e-2", "x=.01;");
    expect_printed_minify("x = 1e-1", "x=.1;");
    expect_printed_minify("x = 1e0", "x=1;");
    expect_printed_minify("x = 1e1", "x=10;");
    expect_printed_minify("x = 1e2", "x=100;");
    expect_printed_minify("x = 1e3", "x=1e3;");
    expect_printed_minify("x = 1e4", "x=1e4;");
    expect_printed_minify("x = 1e100", "x=1e100;");
    expect_printed("x = 12e-100", "x = 12e-100;\n");
    expect_printed("x = 12e-5", "x = 12e-5;\n");
    expect_printed("x = 12e-4", "x = 12e-4;\n");
    expect_printed("x = 12e-3", "x = 0.012;\n");
    expect_printed("x = 12e-2", "x = 0.12;\n");
    expect_printed("x = 12e-1", "x = 1.2;\n");
    expect_printed("x = 12e0", "x = 12;\n");
    expect_printed("x = 12e1", "x = 120;\n");
    expect_printed("x = 12e2", "x = 1200;\n");
    expect_printed("x = 12e3", "x = 12e3;\n");
    expect_printed("x = 12e4", "x = 12e4;\n");
    expect_printed("x = 12e100", "x = 12e100;\n");
    expect_printed_minify("x = 12e-100", "x=12e-100;");
    expect_printed_minify("x = 12e-6", "x=12e-6;");
    expect_printed_minify("x = 12e-5", "x=12e-5;");
    expect_printed_minify("x = 12e-4", "x=.0012;");
    expect_printed_minify("x = 12e-3", "x=.012;");
    expect_printed_minify("x = 12e-2", "x=.12;");
    expect_printed_minify("x = 12e-1", "x=1.2;");
    expect_printed_minify("x = 12e0", "x=12;");
    expect_printed_minify("x = 12e1", "x=120;");
    expect_printed_minify("x = 12e2", "x=1200;");
    expect_printed_minify("x = 12e3", "x=12e3;");
    expect_printed_minify("x = 12e4", "x=12e4;");
    expect_printed_minify("x = 12e100", "x=12e100;");
    expect_printed("x = 123456789", "x = 123456789;\n");
    expect_printed("x = 1123456789", "x = 1123456789;\n");
    expect_printed("x = 10123456789", "x = 10123456789;\n");
    expect_printed("x = 100123456789", "x = 100123456789;\n");
    expect_printed("x = 1000123456789", "x = 1000123456789;\n");
    expect_printed("x = 10000123456789", "x = 10000123456789;\n");
    expect_printed("x = 100000123456789", "x = 100000123456789;\n");
    expect_printed("x = 1000000123456789", "x = 1000000123456789;\n");
    expect_printed("x = 10000000123456789", "x = 10000000123456788;\n");
    expect_printed("x = 100000000123456789", "x = 100000000123456780;\n");
    expect_printed("x = 1000000000123456789", "x = 1000000000123456800;\n");
    expect_printed("x = 10000000000123456789", "x = 10000000000123458e3;\n");
    expect_printed("x = 100000000000123456789", "x = 10000000000012345e4;\n");
    expect_printed("x = 0x7fff_ffff", "x = 2147483647;\n");
    expect_printed("x = 0x8000_0000", "x = 2147483648;\n");
    expect_printed("x = 0x8000_0001", "x = 2147483649;\n");
    expect_printed("x = -0x7fff_ffff", "x = -2147483647;\n");
    expect_printed("x = -0x8000_0000", "x = -2147483648;\n");
    expect_printed("x = -0x8000_0001", "x = -2147483649;\n");
    expect_printed("x = 0xffff_ffff", "x = 4294967295;\n");
    expect_printed("x = 0x1_0000_0000", "x = 4294967296;\n");
    expect_printed("x = 0x1_0000_0001", "x = 4294967297;\n");
    expect_printed("x = -0xffff_ffff", "x = -4294967295;\n");
    expect_printed("x = -0x1_0000_0000", "x = -4294967296;\n");
    expect_printed("x = -0x1_0000_0001", "x = -4294967297;\n");
    expect_printed("x = 0x7fff_ffff_ffff_fdff", "x = 9223372036854775e3;\n");
    expect_printed("x = 0x8000_0000_0000_0000", "x = 9223372036854776e3;\n");
    expect_printed("x = 0x8000_0000_0000_3000", "x = 9223372036854788e3;\n");
    expect_printed("x = -0x7fff_ffff_ffff_fdff", "x = -9223372036854775e3;\n");
    expect_printed("x = -0x8000_0000_0000_0000", "x = -9223372036854776e3;\n");
    expect_printed("x = -0x8000_0000_0000_3000", "x = -9223372036854788e3;\n");
    expect_printed("x = 0xffff_ffff_ffff_fbff", "x = 1844674407370955e4;\n");
    expect_printed("x = 0x1_0000_0000_0000_0000", "x = 18446744073709552e3;\n");
    expect_printed("x = 0x1_0000_0000_0000_1000", "x = 18446744073709556e3;\n");
    expect_printed("x = -0xffff_ffff_ffff_fbff", "x = -1844674407370955e4;\n");
    expect_printed(
        "x = -0x1_0000_0000_0000_0000",
        "x = -18446744073709552e3;\n",
    );
    expect_printed(
        "x = -0x1_0000_0000_0000_1000",
        "x = -18446744073709556e3;\n",
    );
    expect_printed("x = 999999999999", "x = 999999999999;\n");
    expect_printed("x = 1000000000001", "x = 1000000000001;\n");
    expect_printed("x = 0x0FFF_FFFF_FFFF_FF80", "x = 1152921504606846800;\n");
    expect_printed("x = 0x1000_0000_0000_0000", "x = 1152921504606847e3;\n");
    expect_printed("x = 0xFFFF_FFFF_FFFF_F000", "x = 18446744073709548e3;\n");
    expect_printed("x = 0xFFFF_FFFF_FFFF_F800", "x = 1844674407370955e4;\n");
    expect_printed("x = 0xFFFF_FFFF_FFFF_FFFF", "x = 18446744073709552e3;\n");
    expect_printed_minify("x = 999999999999", "x=999999999999;");
    expect_printed_minify("x = 1000000000001", "x=0xe8d4a51001;");
    expect_printed_minify("x = 0x0FFF_FFFF_FFFF_FF80", "x=0xfffffffffffff80;");
    expect_printed_minify("x = 0x1000_0000_0000_0000", "x=1152921504606847e3;");
    expect_printed_minify("x = 0xFFFF_FFFF_FFFF_F000", "x=0xfffffffffffff000;");
    expect_printed_minify("x = 0xFFFF_FFFF_FFFF_F800", "x=1844674407370955e4;");
    expect_printed_minify("x = 0xFFFF_FFFF_FFFF_FFFF", "x=18446744073709552e3;");
    expect_printed_minify("x = 0.0001 .y", "x=1e-4.y;");
    expect_printed_minify("x = 0.001 .y", "x=.001.y;");
    expect_printed_minify("x = 0.01 .y", "x=.01.y;");
    expect_printed_minify("x = 0.1 .y", "x=.1.y;");
    expect_printed_minify("x = 0 .y", "x=0 .y;");
    expect_printed_minify("x = 10 .y", "x=10 .y;");
    expect_printed_minify("x = 100 .y", "x=100 .y;");
    expect_printed_minify("x = 1000 .y", "x=1e3.y;");
    expect_printed_minify("x = 12345 .y", "x=12345 .y;");
    expect_printed_minify("x = 0xFFFF_0000_FFFF_0000 .y", "x=0xffff0000ffff0000.y;");
}

#[test]
fn test_array() {
    expect_printed("[]", "[];\n");
    expect_printed("[,]", "[,];\n");
    expect_printed("[,,]", "[, ,];\n");
}

#[test]
fn test_splat() {
    expect_printed("[...(a, b)]", "[...(a, b)];\n");
    expect_printed("x(...(a, b))", "x(...(a, b));\n");
    expect_printed("({...(a, b)})", "({ ...(a, b) });\n");
}

#[test]
fn test_new() {
    expect_printed("new x", "new x();\n");
    expect_printed("new x()", "new x();\n");
    expect_printed("new (x)", "new x();\n");
    expect_printed("new (x())", "new (x())();\n");
    expect_printed("new (new x())", "new new x()();\n");
    expect_printed("new (x + x)", "new (x + x)();\n");
    expect_printed("(new x)()", "new x()();\n");
    expect_printed("new foo().bar", "new foo().bar;\n");
    expect_printed("new (foo().bar)", "new (foo()).bar();\n");
    expect_printed("new (foo()).bar", "new (foo()).bar();\n");
    expect_printed("new foo()[bar]", "new foo()[bar];\n");
    expect_printed("new (foo()[bar])", "new (foo())[bar]();\n");
    expect_printed("new (foo())[bar]", "new (foo())[bar]();\n");
    expect_printed("new (import('foo').bar)", "new (import(\"foo\")).bar();\n");
    expect_printed("new (import('foo')).bar", "new (import(\"foo\")).bar();\n");
    expect_printed(
        "new (import('foo')[bar])",
        "new (import(\"foo\"))[bar]();\n",
    );
    expect_printed(
        "new (import('foo'))[bar]",
        "new (import(\"foo\"))[bar]();\n",
    );
    expect_printed_minify("new x", "new x;");
    expect_printed_minify("new x.y", "new x.y;");
    expect_printed_minify("(new x).y", "new x().y;");
    expect_printed_minify("new x().y", "new x().y;");
    expect_printed_minify("new x() + y", "new x+y;");
    expect_printed_minify("new x() ** 2", "new x**2;");
    expect_printed(
        "new Worker(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
        "new Worker(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\"\n);\n",
    );
    expect_printed(
        "new Worker(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
        "new Worker(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\"\n);\n",
    );
    expect_printed(
        "new Worker(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
        "new Worker(\n  /* multi\n   * line\n   * webpackBar: */\n  \"path\"\n);\n",
    );
    expect_printed(
        "new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
        "new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    expect_printed(
        "new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);",
        "new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n);\n",
    );
    expect_printed(
        "new Worker(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
        "new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    expect_printed(
        "new Worker(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))",
        "new Worker(new URL(\n  \"path\",\n  /* webpackFoo: these can go anywhere */\n  import.meta.url\n));\n",
    );
}

#[test]
fn test_call() {
    expect_printed("x()()()", "x()()();\n");
    expect_printed("x().y()[z]()", "x().y()[z]();\n");
    expect_printed("(--x)();", "(--x)();\n");
    expect_printed("(x--)();", "(x--)();\n");
    expect_printed("eval(x)", "eval(x);\n");
    expect_printed("eval?.(x)", "eval?.(x);\n");
    expect_printed("(eval)(x)", "eval(x);\n");
    expect_printed("(eval)?.(x)", "eval?.(x);\n");
    expect_printed("eval(x, y)", "eval(x, y);\n");
    expect_printed("eval?.(x, y)", "eval?.(x, y);\n");
    expect_printed("(1, eval)(x)", "(1, eval)(x);\n");
    expect_printed("(1, eval)?.(x)", "(1, eval)?.(x);\n");
    expect_printed_minify("eval?.(x)", "eval?.(x);");
    expect_printed_minify("eval(x,y)", "eval(x,y);");
    expect_printed_minify("eval?.(x,y)", "eval?.(x,y);");
    expect_printed_minify("(1, eval)(x)", "(1,eval)(x);");
    expect_printed_minify("(1, eval)?.(x)", "(1,eval)?.(x);");
}

#[test]
fn test_member() {
    expect_printed("x.y[z]", "x.y[z];\n");
    expect_printed("((x+1).y+1)[z]", "((x + 1).y + 1)[z];\n");
}

#[test]
fn test_comma() {
    expect_printed("1, 2, 3", "1, 2, 3;\n");
    expect_printed("(1, 2), 3", "1, 2, 3;\n");
    expect_printed("1, (2, 3)", "1, 2, 3;\n");
    expect_printed("a ? (b, c) : (d, e)", "a ? (b, c) : (d, e);\n");
    expect_printed("let x = (a, b)", "let x = (a, b);\n");
    expect_printed("(x = a), b", "x = a, b;\n");
    expect_printed("x = (a, b)", "x = (a, b);\n");
    expect_printed("x((1, 2))", "x((1, 2));\n");
}

#[test]
fn test_unary() {
    expect_printed("+(x--)", "+x--;\n");
    expect_printed("-(x++)", "-x++;\n");
}

#[test]
fn test_nullish() {
    expect_printed("(a && b) ?? c", "(a && b) ?? c;\n");
    expect_printed("(a || b) ?? c", "(a || b) ?? c;\n");
    expect_printed("a ?? (b && c)", "a ?? (b && c);\n");
    expect_printed("a ?? (b || c)", "a ?? (b || c);\n");
    expect_printed("a && (b ?? c)", "a && (b ?? c);\n");
    expect_printed("a || (b ?? c)", "a || (b ?? c);\n");
    expect_printed("(a ?? b) && c", "(a ?? b) && c;\n");
    expect_printed("(a ?? b) || c", "(a ?? b) || c;\n");
}

#[test]
fn test_string() {
    expect_printed("let x = ''", "let x = \"\";\n");
    expect_printed("let x = '\x08'", "let x = \"\\b\";\n");
    expect_printed("let x = '\x0c'", "let x = \"\\f\";\n");
    expect_printed("let x = '\t'", "let x = \"\t\";\n");
    expect_printed("let x = '\x0b'", "let x = \"\\v\";\n");
    expect_printed("let x = '\\n'", "let x = \"\\n\";\n");
    expect_printed("let x = '\\''", "let x = \"'\";\n");
    expect_printed("let x = '\\\"'", "let x = '\"';\n");
    expect_printed("let x = '\\'\"'", "let x = `'\"`;\n");
    expect_printed("let x = '\\\\'", "let x = \"\\\\\";\n");
    expect_printed("let x = '\x00'", "let x = \"\\0\";\n");
    expect_printed("let x = '\x00!'", "let x = \"\\0!\";\n");
    expect_printed("let x = '\x001'", "let x = \"\\x001\";\n");
    expect_printed("let x = '\\0'", "let x = \"\\0\";\n");
    expect_printed("let x = '\\0!'", "let x = \"\\0!\";\n");
    expect_printed("let x = '\x07'", "let x = \"\\x07\";\n");
    expect_printed("let x = '\x07!'", "let x = \"\\x07!\";\n");
    expect_printed("let x = '\x071'", "let x = \"\\x071\";\n");
    expect_printed("let x = '\\7'", "let x = \"\\x07\";\n");
    expect_printed("let x = '\\7!'", "let x = \"\\x07!\";\n");
    expect_printed("let x = '\\01'", "let x = \"\x01\";\n");
    expect_printed("let x = '\x10'", "let x = \"\x10\";\n");
    expect_printed("let x = '\\x10'", "let x = \"\x10\";\n");
    expect_printed("let x = '\x1b'", "let x = \"\\x1B\";\n");
    expect_printed("let x = '\\x1B'", "let x = \"\\x1B\";\n");
    expect_printed("let x = '\u{abcd}'", "let x = \"\u{abcd}\";\n");
    expect_printed("let x = '\\uABCD'", "let x = \"\u{abcd}\";\n");
    expect_printed("let x = '\u{123ab}'", "let x = \"\u{123ab}\";\n");
    expect_printed("let x = '\\u{123AB}'", "let x = \"\u{123ab}\";\n");
    expect_printed("let x = '\\uD808\\uDFAB'", "let x = \"\u{123ab}\";\n");
    expect_printed("let x = '\\uD808'", "let x = \"\\uD808\";\n");
    expect_printed("let x = '\\uD808X'", "let x = \"\\uD808X\";\n");
    expect_printed("let x = '\\uDFAB'", "let x = \"\\uDFAB\";\n");
    expect_printed("let x = '\\uDFABX'", "let x = \"\\uDFABX\";\n");
    expect_printed("let x = '\\x80'", "let x = \"\u{80}\";\n");
    expect_printed("let x = '\\xFF'", "let x = \"\u{ff}\";\n");
    expect_printed(
        "let x = '\\xF0\\x9F\\x8D\\x95'",
        "let x = \"\u{f0}\u{9f}\u{8d}\u{95}\";\n",
    );
    expect_printed(
        "let x = '\\uD801\\uDC02\\uDC03\\uD804'",
        "let x = \"\u{10402}\\uDC03\\uD804\";\n",
    );
}

#[test]
fn test_template() {
    expect_printed("let x = `\\0`", "let x = `\\0`;\n");
    expect_printed("let x = `\\x01`", "let x = `\x01`;\n");
    expect_printed("let x = `\\0${0}`", "let x = `\\0${0}`;\n");
    expect_printed("let x = `\\x01${0}`", "let x = `\x01${0}`;\n");
    expect_printed("let x = `${0}\\0`", "let x = `${0}\\0`;\n");
    expect_printed("let x = `${0}\\x01`", "let x = `${0}\x01`;\n");
    expect_printed("let x = `${0}\\0${1}`", "let x = `${0}\\0${1}`;\n");
    expect_printed("let x = `${0}\\x01${1}`", "let x = `${0}\x01${1}`;\n");
    expect_printed("let x = String.raw`\\1`", "let x = String.raw`\\1`;\n");
    expect_printed("let x = String.raw`\\x01`", "let x = String.raw`\\x01`;\n");
    expect_printed(
        "let x = String.raw`\\1${0}`",
        "let x = String.raw`\\1${0}`;\n",
    );
    expect_printed(
        "let x = String.raw`\\x01${0}`",
        "let x = String.raw`\\x01${0}`;\n",
    );
    expect_printed(
        "let x = String.raw`${0}\\1`",
        "let x = String.raw`${0}\\1`;\n",
    );
    expect_printed(
        "let x = String.raw`${0}\\x01`",
        "let x = String.raw`${0}\\x01`;\n",
    );
    expect_printed(
        "let x = String.raw`${0}\\1${1}`",
        "let x = String.raw`${0}\\1${1}`;\n",
    );
    expect_printed(
        "let x = String.raw`${0}\\x01${1}`",
        "let x = String.raw`${0}\\x01${1}`;\n",
    );
    expect_printed("let x = `${y}`", "let x = `${y}`;\n");
    expect_printed("let x = `$(y)`", "let x = `$(y)`;\n");
    expect_printed("let x = `{y}$`", "let x = `{y}$`;\n");
    expect_printed("let x = `$}y{`", "let x = `$}y{`;\n");
    expect_printed("let x = `\\${y}`", "let x = `\\${y}`;\n");
    expect_printed("let x = `$\\{y}`", "let x = `\\${y}`;\n");
    expect_printed("await tag`x`", "await tag`x`;\n");
    expect_printed("await (tag`x`)", "await tag`x`;\n");
    expect_printed("(await tag)`x`", "(await tag)`x`;\n");
    expect_printed("await tag`${x}`", "await tag`${x}`;\n");
    expect_printed("await (tag`${x}`)", "await tag`${x}`;\n");
    expect_printed("(await tag)`${x}`", "(await tag)`${x}`;\n");
    expect_printed("new tag`x`", "new tag`x`();\n");
    expect_printed("new (tag`x`)", "new tag`x`();\n");
    expect_printed("new tag()`x`", "new tag()`x`;\n");
    expect_printed("(new tag)`x`", "new tag()`x`;\n");
    expect_printed_minify("new tag`x`", "new tag`x`;");
    expect_printed_minify("new (tag`x`)", "new tag`x`;");
    expect_printed_minify("new tag()`x`", "new tag()`x`;");
    expect_printed_minify("(new tag)`x`", "new tag()`x`;");
    expect_printed("new tag`${x}`", "new tag`${x}`();\n");
    expect_printed("new (tag`${x}`)", "new tag`${x}`();\n");
    expect_printed("new tag()`${x}`", "new tag()`${x}`;\n");
    expect_printed("(new tag)`${x}`", "new tag()`${x}`;\n");
    expect_printed_minify("new tag`${x}`", "new tag`${x}`;");
    expect_printed_minify("new (tag`${x}`)", "new tag`${x}`;");
    expect_printed_minify("new tag()`${x}`", "new tag()`${x}`;");
    expect_printed_minify("(new tag)`${x}`", "new tag()`${x}`;");
}

#[test]
fn test_object() {
    expect_printed("let x = {'(':')'}", "let x = { \"(\": \")\" };\n");
    expect_printed("({})", "({});\n");
    expect_printed("({}.x)", "({}).x;\n");
    expect_printed("({} = {})", "({} = {});\n");
    expect_printed("(x, {} = {})", "x, {} = {};\n");
    expect_printed("let x = () => ({})", "let x = () => ({});\n");
    expect_printed("let x = () => ({}.x)", "let x = () => ({}).x;\n");
    expect_printed("let x = () => ({} = {})", "let x = () => ({} = {});\n");
    expect_printed(
        "let x = () => (x, {} = {})",
        "let x = () => (x, {} = {});\n",
    );
    expect_printed(
        "function foo(__proto__) { return { __proto__: __proto__ } }",
        "function foo(__proto__) {\n  return { __proto__: __proto__ };\n}\n",
    );
    expect_printed(
        "function foo(__proto__) { return { '__proto__': __proto__ } }",
        "function foo(__proto__) {\n  return { \"__proto__\": __proto__ };\n}\n",
    );
    expect_printed(
        "function foo(__proto__) { return { ['__proto__']: __proto__ } }",
        "function foo(__proto__) {\n  return { [\"__proto__\"]: __proto__ };\n}\n",
    );
    expect_printed(
        "import { __proto__ } from 'foo'; let foo = () => ({ __proto__: __proto__ })",
        "import { __proto__ } from \"foo\";\nlet foo = () => ({ __proto__: __proto__ });\n",
    );
    expect_printed(
        "import { __proto__ } from 'foo'; let foo = () => ({ '__proto__': __proto__ })",
        "import { __proto__ } from \"foo\";\nlet foo = () => ({ \"__proto__\": __proto__ });\n",
    );
    expect_printed(
        "import { __proto__ } from 'foo'; let foo = () => ({ ['__proto__']: __proto__ })",
        "import { __proto__ } from \"foo\";\nlet foo = () => ({ [\"__proto__\"]: __proto__ });\n",
    );
}

#[test]
fn test_switch() {
    expect_printed(
        "switch (x) { /* 1 */ case 1: /* 2 */ case 2: /* default */ default: break }",
        "switch (x) {\n  /* 1 */\n  case 1:\n  /* 2 */\n  case 2:\n  /* default */\n  default:\n    break;\n}\n",
    );
}

#[test]
fn test_for() {
    expect_printed("for ((a in b);;);", "for ((a in b); ; ) ;\n");
    expect_printed(
        "for (a ? b : (c in d);;);",
        "for (a ? b : (c in d); ; ) ;\n",
    );
    expect_printed(
        "for ((a ? b : c in d).foo;;);",
        "for ((a ? b : c in d).foo; ; ) ;\n",
    );
    expect_printed(
        "for (var x = (a in b);;);",
        "for (var x = (a in b); ; ) ;\n",
    );
    expect_printed("for (x = (a in b);;);", "for (x = (a in b); ; ) ;\n");
    expect_printed("for (x == (a in b);;);", "for (x == (a in b); ; ) ;\n");
    expect_printed(
        "for (1 * (x == a in b);;);",
        "for (1 * (x == a in b); ; ) ;\n",
    );
    expect_printed(
        "for (a ? b : x = (c in d);;);",
        "for (a ? b : x = (c in d); ; ) ;\n",
    );
    expect_printed(
        "for (var x = y = (a in b);;);",
        "for (var x = y = (a in b); ; ) ;\n",
    );
    expect_printed("for ([a in b];;);", "for ([a in b]; ; ) ;\n");
    expect_printed("for (x(a in b);;);", "for (x(a in b); ; ) ;\n");
    expect_printed("for (x[a in b];;);", "for (x[a in b]; ; ) ;\n");
    expect_printed("for (x?.[a in b];;);", "for (x?.[a in b]; ; ) ;\n");
    expect_printed("for ((x => a in b);;);", "for (((x) => a in b); ; ) ;\n");
    expect_printed("for (let a in b, c);", "for (let a in b, c) ;\n");
    expect_printed("for (let a of (b, c));", "for (let a of (b, c)) ;\n");
}

#[test]
fn test_function() {
    expect_printed(
        "function foo(a = (b, c), ...d) {}",
        "function foo(a = (b, c), ...d) {\n}\n",
    );
    expect_printed(
        "function foo({[1 + 2]: a = 3} = {[1 + 2]: 3}) {}",
        "function foo({ [1 + 2]: a = 3 } = { [1 + 2]: 3 }) {\n}\n",
    );
    expect_printed(
        "function foo([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) {}",
        "function foo([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) {\n}\n",
    );
    expect_printed("function foo([] = []) {}", "function foo([] = []) {\n}\n");
    expect_printed(
        "function foo([,] = [,]) {}",
        "function foo([,] = [,]) {\n}\n",
    );
    expect_printed(
        "function foo([,,] = [,,]) {}",
        "function foo([, ,] = [, ,]) {\n}\n",
    );
}

#[test]
fn test_array_pattern_rest_after_elision() {
    expect_printed(
        "function foo([,,...rest]) {}",
        "function foo([, , ...rest]) {\n}\n",
    );
    expect_printed("([,,...rest] = value)", "[, , ...rest] = value;\n");
}

#[test]
fn test_comments_and_parentheses() {
    expect_printed(
        "(/* foo */ { x() { foo() } }.x());",
        "/* foo */\n({ x() {\n  foo();\n} }).x();\n",
    );
    expect_printed(
        "(/* foo */ function f() { foo(f) }());",
        "/* foo */\n(function f() {\n  foo(f);\n})();\n",
    );
    expect_printed(
        "(/* foo */ class x { static y() { foo(x) } }.y());",
        "/* foo */\n(class x {\n  static y() {\n    foo(x);\n  }\n}).y();\n",
    );
    expect_printed(
        "(/* @__PURE__ */ (() => foo())());",
        "/* @__PURE__ */ (() => foo())();\n",
    );
    expect_printed(
        "export default (/* foo */ function f() {});",
        // should fix after supporting pife
        // "export default (\n  /* foo */\n  (function f() {\n  })\n);\n",
        "export default /* foo */ (function f() {\n});\n",
    );
    expect_printed(
        "export default (/* foo */ class x {});",
        // should fix after supporting pife
        // "export default (\n  /* foo */\n  class x {\n  }\n);\n",
        "export default /* foo */ (class x {\n});\n",
    );
    expect_printed(
        "x = () => (/* foo */ {});",
        "x = () => (\n  /* foo */\n  {}\n);\n",
    );
    expect_printed(
        "for ((/* foo */ let).x of y) ;",
        "for (\n  /* foo */\n  (let).x of y\n) ;\n",
    );
    expect_printed(
        "for (/* foo */ (let).x of y) ;",
        "for (\n  /* foo */\n  (let).x of y\n) ;\n",
    );
    expect_printed(
        "function *x() { yield (/* foo */ y) }",
        "function* x() {\n  yield (\n    /* foo */\n    y\n  );\n}\n",
    );
}

#[test]
fn test_pure_comment() {
    expect_printed("(function() { foo() })", "(function() {\n  foo();\n});\n");
    expect_printed(
        "(function() { foo() })()",
        "(function() {\n  foo();\n})();\n",
    );
    expect_printed(
        "/*@__PURE__*/(function() { foo() })()",
        "/* @__PURE__ */ (function() {\n  foo();\n})();\n",
    );
    expect_printed("new (function() {})", "new (function() {\n})();\n");
    expect_printed("new (function() {})()", "new (function() {\n})();\n");
    expect_printed(
        "/*@__PURE__*/new (function() {})()",
        "/* @__PURE__ */ new (function() {\n})();\n",
    );
    expect_printed(
        "export default (function() { foo() })",
        "export default (function() {\n  foo();\n});\n",
    );
    expect_printed(
        "export default (function() { foo() })()",
        "export default (function() {\n  foo();\n})();\n",
    );
    expect_printed(
        "export default /*@__PURE__*/(function() { foo() })()",
        "export default /* @__PURE__ */ (function() {\n  foo();\n})();\n",
    );
}

#[test]
fn test_generator() {
    expect_printed("function* foo() {}", "function* foo() {\n}\n");
    expect_printed("(function* () {})", "(function* () {\n});\n");
    expect_printed("(function* foo() {})", "(function* foo() {\n});\n");
    expect_printed(
        "class Foo { *foo() {} }",
        "class Foo {\n  *foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static *foo() {} }",
        "class Foo {\n  static *foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { *[foo]() {} }",
        "class Foo {\n  *[foo]() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static *[foo]() {} }",
        "class Foo {\n  static *[foo]() {\n  }\n}\n",
    );
    expect_printed("(class { *foo() {} })", "(class {\n  *foo() {\n  }\n});\n");
    expect_printed(
        "(class { static *foo() {} })",
        "(class {\n  static *foo() {\n  }\n});\n",
    );
    expect_printed(
        "(class { *[foo]() {} })",
        "(class {\n  *[foo]() {\n  }\n});\n",
    );
    expect_printed(
        "(class { static *[foo]() {} })",
        "(class {\n  static *[foo]() {\n  }\n});\n",
    );
}

#[test]
fn test_arrow() {
    expect_printed("() => {}", "() => {\n};\n");
    expect_printed("x => (x, 0)", "(x) => (x, 0);\n");
    expect_printed("x => {y}", "(x) => {\n  y;\n};\n");
    expect_printed("(a = (b, c), ...d) => {}", "(a = (b, c), ...d) => {\n};\n");
    expect_printed(
        "({[1 + 2]: a = 3} = {[1 + 2]: 3}) => {}",
        "({ [1 + 2]: a = 3 } = { [1 + 2]: 3 }) => {\n};\n",
    );
    expect_printed(
        "([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) => {}",
        "([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) => {\n};\n",
    );
    expect_printed("([] = []) => {}", "([] = []) => {\n};\n");
    expect_printed("([,] = [,]) => {}", "([,] = [,]) => {\n};\n");
    expect_printed("([,,] = [,,]) => {}", "([, ,] = [, ,]) => {\n};\n");
    expect_printed("a = () => {}", "a = () => {\n};\n");
    expect_printed("a || (() => {})", "a || (() => {\n});\n");
    expect_printed("({a = b, c = d}) => {}", "({ a = b, c = d }) => {\n};\n");
    expect_printed(
        "([{a = b, c = d} = {}] = []) => {}",
        "([{ a = b, c = d } = {}] = []) => {\n};\n",
    );
    expect_printed(
        "({a: [b = c] = []} = {}) => {}",
        "({ a: [b = c] = [] } = {}) => {\n};\n",
    );
    expect_printed("(a = b, c)", "a = b, c;\n");
    expect_printed("([...a = b])", "[...a = b];\n");
    expect_printed("([...a, ...b])", "[...a, ...b];\n");
    expect_printed("({a: b, c() {}})", "({ a: b, c() {\n} });\n");
    expect_printed("({a: b, get c() {}})", "({ a: b, get c() {\n} });\n");
    expect_printed("({a: b, set c(x) {}})", "({ a: b, set c(x) {\n} });\n");
}

#[test]
fn test_class() {
    expect_printed(
        "class Foo extends (a, b) {}",
        "class Foo extends (a, b) {\n}\n",
    );
    expect_printed(
        "class Foo { get foo() {} }",
        "class Foo {\n  get foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { set foo(x) {} }",
        "class Foo {\n  set foo(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static foo() {} }",
        "class Foo {\n  static foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static get foo() {} }",
        "class Foo {\n  static get foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static set foo(x) {} }",
        "class Foo {\n  static set foo(x) {\n  }\n}\n",
    );
}

#[test]
fn test_auto_accessors() {
    expect_printed(
        "class Foo { accessor x; static accessor y }",
        "class Foo {\n  accessor x;\n  static accessor y;\n}\n",
    );
    expect_printed(
        "class Foo { accessor [x]; static accessor [y] }",
        "class Foo {\n  accessor [x];\n  static accessor [y];\n}\n",
    );
    expect_printed_minify(
        "class Foo { accessor x; static accessor y }",
        "class Foo{accessor x;static accessor y}",
    );
    expect_printed_minify(
        "class Foo { accessor [x]; static accessor [y] }",
        "class Foo{accessor[x];static accessor[y]}",
    );
}

#[test]
fn test_private_identifiers() {
    expect_printed(
        "class Foo { #foo; foo() { return #foo in this } }",
        "class Foo {\n  #foo;\n  foo() {\n    return #foo in this;\n  }\n}\n",
    );
    expect_printed_minify(
        "class Foo { #foo; foo() { return #foo in this } }",
        "class Foo{#foo;foo(){return#foo in this}}",
    );
}

#[test]
fn test_decorators() {
    let example = "class Foo {\n@w\nw; @x x; @a1\n@b1@b2\n@c1@c2@c3\ny = @y1 @y2 class {}; @a1\n@b1@b2\n@c1@c2@c3 z =\n@z1\n@z2\nclass {}}";
    expect_printed(
        example,
        "class Foo {\n  @w w;\n  @x x;\n  @a1 @b1 @b2 @c1 @c2 @c3 y = @y1 @y2 class {\n  };\n  @a1 @b1 @b2 @c1 @c2 @c3 z = @z1 @z2 class {\n  };\n}\n",
    );
    expect_printed_minify(
        example,
        "class Foo{@w w;@x x;@a1 @b1 @b2 @c1 @c2 @c3 y=@y1 @y2 class{};@a1 @b1 @b2 @c1 @c2 @c3 z=@z1 @z2 class{}}",
    );
    expect_printed_minify("class C { @dec static x }", "class C{@dec static x}");
}

#[test]
fn test_import() {
    expect_printed("import('path');", "import(\"path\");\n");
    expect_printed(
        "import(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
        "import(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\"\n);\n",
    );
    expect_printed(
        "import(// webpackFoo: 1\n // webpackBar: 2\n 'path', {type: 'module'});",
        "import(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\",\n  { type: \"module\" }\n);\n",
    );
    expect_printed(
        "import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
        "import(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\"\n);\n",
    );
    expect_printed(
        "import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path', {type: 'module'});",
        "import(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\",\n  { type: \"module\" }\n);\n",
    );
    expect_printed(
        "import(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
        "import(\n  /* multi\n   * line\n   * webpackBar: */\n  \"path\"\n);\n",
    );
    expect_printed(
        "import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
        "import(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    expect_printed(
        "import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);",
        "import(\n  /* webpackFoo: 1 */\n  \"path\"\n);\n",
    );
    expect_printed(
        "import(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
        "import(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    expect_printed(
        "import(/* webpackFoo: 1 */ 'path', { type: 'module' } /* webpackBar:2 */ );",
        "import(\n  /* webpackFoo: 1 */\n  \"path\",\n  { type: \"module\" }\n  /* webpackBar:2 */\n);\n",
    );
    expect_printed(
        "import(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))",
        "import(new URL(\n  \"path\",\n  /* webpackFoo: these can go anywhere */\n  import.meta.url\n));\n",
    );
    expect_printed_minify(
        "import defer * as foo from 'bar'",
        "import defer*as foo from\"bar\";",
    );
    expect_printed_minify(
        "import source foo from 'bar'",
        "import source foo from\"bar\";",
    );
}

#[test]
fn test_export_default() {
    expect_printed_ts(
        "export default function(): void;",
        "export default function(): void;\n",
    );
    expect_printed_ts(
        "export default async function<T>(): Promise<T>;",
        "export default async function<T>(): Promise<T>;\n",
    );
    expect_printed(
        "export default function() {}",
        "export default function() {\n}\n",
    );
    expect_printed(
        "export default function foo() {}",
        "export default function foo() {\n}\n",
    );
    expect_printed(
        "export default async function() {}",
        "export default async function() {\n}\n",
    );
    expect_printed(
        "export default async function foo() {}",
        "export default async function foo() {\n}\n",
    );
    expect_printed("export default class {}", "export default class {\n}\n");
    expect_printed(
        "export default class foo {}",
        "export default class foo {\n}\n",
    );
    expect_printed(
        "export default (function() {})",
        "export default (function() {\n});\n",
    );
    expect_printed(
        "export default (function foo() {})",
        "export default (function foo() {\n});\n",
    );
    expect_printed(
        "export default (async function() {})",
        "export default (async function() {\n});\n",
    );
    expect_printed(
        "export default (async function foo() {})",
        "export default (async function foo() {\n});\n",
    );
    expect_printed(
        "export default (class {})",
        "export default (class {\n});\n",
    );
    expect_printed(
        "export default (class foo {})",
        "export default (class foo {\n});\n",
    );
    expect_printed(
        "export default (function() {}.toString())",
        "export default (function() {\n}).toString();\n",
    );
    expect_printed(
        "export default (function foo() {}.toString())",
        "export default (function foo() {\n}).toString();\n",
    );
    expect_printed(
        "export default (async function() {}.toString())",
        "export default (async function() {\n}).toString();\n",
    );
    expect_printed(
        "export default (async function foo() {}.toString())",
        "export default (async function foo() {\n}).toString();\n",
    );
    expect_printed(
        "export default (class {}.toString())",
        "export default (class {\n}).toString();\n",
    );
    expect_printed(
        "export default (class foo {}.toString())",
        "export default (class foo {\n}).toString();\n",
    );
    expect_printed_minify(
        "export default function() {}",
        "export default function(){}",
    );
    expect_printed_minify(
        "export default function foo() {}",
        "export default function foo(){}",
    );
    expect_printed_minify(
        "export default async function() {}",
        "export default async function(){}",
    );
    expect_printed_minify(
        "export default async function foo() {}",
        "export default async function foo(){}",
    );
    expect_printed_minify("export default class {}", "export default class{}");
    expect_printed_minify("export default class foo {}", "export default class foo{}");
}

#[test]
fn test_whitespace() {
    expect_printed("- -x", "- -x;\n");
    expect_printed("+ -x", "+-x;\n");
    expect_printed("- +x", "-+x;\n");
    expect_printed("+ +x", "+ +x;\n");
    expect_printed("- --x", "- --x;\n");
    expect_printed("+ --x", "+--x;\n");
    expect_printed("- ++x", "-++x;\n");
    expect_printed("+ ++x", "+ ++x;\n");
    expect_printed_minify("- -x", "- -x;");
    expect_printed_minify("+ -x", "+-x;");
    expect_printed_minify("- +x", "-+x;");
    expect_printed_minify("+ +x", "+ +x;");
    expect_printed_minify("- --x", "- --x;");
    expect_printed_minify("+ --x", "+--x;");
    expect_printed_minify("- ++x", "-++x;");
    expect_printed_minify("+ ++x", "+ ++x;");
    expect_printed_minify("x - --y", "x- --y;");
    expect_printed_minify("x + --y", "x+--y;");
    expect_printed_minify("x - ++y", "x-++y;");
    expect_printed_minify("x + ++y", "x+ ++y;");
    expect_printed_minify("x-- > y", "x-- >y;");
    expect_printed_minify("x < !--y", "x<! --y;");
    expect_printed_minify("x > !--y", "x>!--y;");
    expect_printed_minify("!--y", "!--y;");
    expect_printed_minify("1 + -0", "1+-0;");
    expect_printed_minify("1 - -0", "1- -0;");
    expect_printed_minify("1 + -Infinity", "1+-Infinity;");
    expect_printed_minify("1 - -Infinity", "1- -Infinity;");
    expect_printed_minify("/x/ / /y/", "/x// /y/;");
    expect_printed_minify("/x/ + Foo", "/x/+Foo;");
    expect_printed_minify("/x/ instanceof Foo", "/x/ instanceof Foo;");
    expect_printed_minify("[x] instanceof Foo", "[x]instanceof Foo;");
    expect_printed_minify("throw x", "throw x;");
    expect_printed_minify("throw typeof x", "throw typeof x;");
    expect_printed_minify("throw delete x", "throw delete x;");
    expect_printed_minify("throw function(){}", "throw function(){};");
    expect_printed_minify("x in function(){}", "x in function(){};");
    expect_printed_minify("x instanceof function(){}", "x instanceof function(){};");
    expect_printed_minify("\u{3c0} in function(){}", "\u{3c0} in function(){};");
    expect_printed_minify(
        "\u{3c0} instanceof function(){}",
        "\u{3c0} instanceof function(){};",
    );
    expect_printed_minify("()=>({})", "()=>({});");
    expect_printed_minify("()=>({}[1])", "()=>({})[1];");
    // This output depends on esbuild's parser-stage string addition folding.
    // expect_printed_minify("()=>({}+0)", "()=>\"[object Object]0\";");
    expect_printed_minify("()=>function(){}", "()=>function(){};");
    expect_printed_minify("(function(){})", "(function(){});");
    expect_printed_minify("(class{})", "(class{});");
    expect_printed_minify("({})", "({});");
}

#[test]
fn test_minify() {
    expect_printed_minify("0.1", ".1;");
    expect_printed_minify("1.2", "1.2;");
    expect_printed_minify("() => {}", "()=>{};");
    expect_printed_minify("(a) => {}", "a=>{};");
    expect_printed_minify("(...a) => {}", "(...a)=>{};");
    expect_printed_minify("(a = 0) => {}", "(a=0)=>{};");
    expect_printed_minify("(a, b) => {}", "(a,b)=>{};");
    expect_printed("true ** 2", "true ** 2;\n");
    expect_printed("false ** 2", "false ** 2;\n");
    expect_printed_minify("true ** 2", "true**2;");
    expect_printed_minify("false ** 2", "false**2;");
    expect_printed_minify("import a from 'path'", "import a from\"path\";");
    expect_printed_minify("import * as ns from 'path'", "import*as ns from\"path\";");
    expect_printed_minify(
        "import {a, b as c} from 'path'",
        "import{a,b as c}from\"path\";",
    );
    expect_printed_minify(
        "import {a, ' ' as c} from 'path'",
        "import{a,\" \"as c}from\"path\";",
    );
    expect_printed_minify("export * as ns from 'path'", "export*as ns from\"path\";");
    expect_printed_minify("export * as ' ' from 'path'", "export*as\" \"from\"path\";");
    expect_printed_minify(
        "export {a, b as c} from 'path'",
        "export{a,b as c}from\"path\";",
    );
    expect_printed_minify(
        "export {' ', '-' as ';'} from 'path'",
        "export{\" \",\"-\"as\";\"}from\"path\";",
    );
    expect_printed_minify("let a, b; export {a, b as c}", "let a,b;export{a,b as c};");
    expect_printed_minify(
        "let a, b; export {a, b as ' '}",
        "let a,b;export{a,b as\" \"};",
    );
    expect_printed("x = '\\n'", "x = \"\\n\";\n");
    expect_printed_minify("exports", "exports;");
    expect_printed_minify("require", "require;");
    expect_printed_minify("module", "module;");
    expect_printed_minify("//!single\nthrow 1 + 2", "//!single\nthrow 1+2;");
    expect_printed_minify(
        "/*!multi-\nline*/\nthrow 1 + 2",
        "/*!multi-\nline*/throw 1+2;",
    );
}

#[test]
fn test_ascii_only() {
    expect_printed("let \u{3c0} = '\u{3c0}'", "let \u{3c0} = \"\u{3c0}\";\n");
    expect_printed("let \u{3c0}_ = '\u{3c0}'", "let \u{3c0}_ = \"\u{3c0}\";\n");
    expect_printed("let _\u{3c0} = '\u{3c0}'", "let _\u{3c0} = \"\u{3c0}\";\n");
    expect_printed_ascii("let \u{3c0} = '\u{3c0}'", "let \\u03C0 = \"\\u03C0\";\n");
    expect_printed_ascii("let \u{3c0}_ = '\u{3c0}'", "let \\u03C0_ = \"\\u03C0\";\n");
    expect_printed_ascii("let _\u{3c0} = '\u{3c0}'", "let _\\u03C0 = \"\\u03C0\";\n");
    expect_printed(
        "let \u{8c93} = '\u{1f408}'",
        "let \u{8c93} = \"\u{1f408}\";\n",
    );
    expect_printed(
        "let \u{8c93}abc = '\u{1f408}'",
        "let \u{8c93}abc = \"\u{1f408}\";\n",
    );
    expect_printed(
        "let abc\u{8c93} = '\u{1f408}'",
        "let abc\u{8c93} = \"\u{1f408}\";\n",
    );
    expect_printed_ascii(
        "let \u{8c93} = '\u{1f408}'",
        "let \\u8C93 = \"\\u{1F408}\";\n",
    );
    expect_printed_ascii(
        "let \u{8c93}abc = '\u{1f408}'",
        "let \\u8C93abc = \"\\u{1F408}\";\n",
    );
    expect_printed_ascii(
        "let abc\u{8c93} = '\u{1f408}'",
        "let abc\\u8C93 = \"\\u{1F408}\";\n",
    );
    expect_printed("var \u{10000}", "var \u{10000};\n");
    expect_printed("var \\u{10000}", "var \u{10000};\n");
    expect_printed_ascii("var \u{10000}", "var \\u{10000};\n");
    expect_printed_ascii("var \\u{10000}", "var \\u{10000};\n");
    expect_printed_ascii(
        "var \\u{100a} = {\\u100A: '\\u100A'}",
        "var \\u100A = { \\u100A: \"\\u100A\" };\n",
    );
    expect_printed_ascii(
        "var \\u{1000a} = {\\u{1000A}: '\\u{1000A}'}",
        "var \\u{1000A} = { \"\\u{1000A}\": \"\\u{1000A}\" };\n",
    );
    expect_printed("let x = '\u{2028}'", "let x = \"\\u2028\";\n");
    expect_printed("let x = '\u{2029}'", "let x = \"\\u2029\";\n");
    expect_printed("let x = '\u{feff}'", "let x = \"\\uFEFF\";\n");
    expect_printed_ascii(
        "class \u{10000} extends \u{3c0} {}",
        "class \\u{10000} extends \\u03C0 {\n}\n",
    );
    expect_printed_ascii(
        "(class \u{10000} extends \u{3c0} {})",
        "(class \\u{10000} extends \\u03C0 {\n});\n",
    );
    expect_printed_minify_ascii(
        "class \u{10000} extends \u{3c0} {}",
        "class \\u{10000} extends \\u03C0{}",
    );
    expect_printed_minify_ascii(
        "(class \u{10000} extends \u{3c0} {})",
        "(class \\u{10000} extends \\u03C0{});",
    );
}

#[test]
fn test_jsx() {
    expect_printed_jsx("<a/>", "<a />;\n");
    expect_printed_jsx("<A/>", "<A />;\n");
    expect_printed_jsx("<a.b/>", "<a.b />;\n");
    expect_printed_jsx("<A.B/>", "<A.B />;\n");
    expect_printed_jsx("<a-b/>", "<a-b />;\n");
    expect_printed_jsx("<a:b/>", "<a:b />;\n");
    expect_printed_jsx("<a></a>", "<a />;\n");
    expect_printed_jsx("<a b></a>", "<a b />;\n");
    expect_printed_jsx("<a b={true}></a>", "<a b={true} />;\n");
    expect_printed_jsx("<a b='x'></a>", "<a b='x' />;\n");
    expect_printed_jsx("<a b=\"x\"></a>", "<a b=\"x\" />;\n");
    expect_printed_jsx("<a b={'x'}></a>", "<a b={\"x\"} />;\n");
    expect_printed_jsx("<a b={`'`}></a>", "<a b={`'`} />;\n");
    expect_printed_jsx("<a b={`\"`}></a>", "<a b={`\"`} />;\n");
    expect_printed_jsx("<a b={`'\"`}></a>", "<a b={`'\"`} />;\n");
    expect_printed_jsx("<a b=\"&quot;\"></a>", "<a b=\"&quot;\" />;\n");
    expect_printed_jsx("<a b=\"&amp;\"></a>", "<a b=\"&amp;\" />;\n");
    expect_printed_jsx("<a>x</a>", "<a>x</a>;\n");
    expect_printed_jsx("<a>x\ny</a>", "<a>x\ny</a>;\n");
    expect_printed_jsx("<a>{'x'}{'y'}</a>", "<a>{\"x\"}{\"y\"}</a>;\n");
    expect_printed_jsx("<a> x</a>", "<a> x</a>;\n");
    expect_printed_jsx("<a>x </a>", "<a>x </a>;\n");
    expect_printed_jsx("<a>&#10;</a>", "<a>&#10;</a>;\n");
    expect_printed_jsx("<a>&amp;</a>", "<a>&amp;</a>;\n");
    expect_printed_jsx("<a>&lt;</a>", "<a>&lt;</a>;\n");
    expect_printed_jsx("<a>&gt;</a>", "<a>&gt;</a>;\n");
    expect_printed_jsx("<a>&#123;</a>", "<a>&#123;</a>;\n");
    expect_printed_jsx("<a>&#125;</a>", "<a>&#125;</a>;\n");
    expect_printed_jsx("<a><x/></a>", "<a><x /></a>;\n");
    expect_printed_jsx("<a><x/><y/></a>", "<a><x /><y /></a>;\n");
    expect_printed_jsx("<a>b<c/>d</a>", "<a>b<c />d</a>;\n");
    expect_printed_jsx("<></>", "<></>;\n");
    expect_printed_jsx("<>x<y/>z</>", "<>x<y />z</>;\n");
    expect_printed_jsx("<a b=<c/>/>", "<a b=<c /> />;\n");
    expect_printed_jsx("<a b=<>c</>/>", "<a b=<>c</> />;\n");
    expect_printed_jsx("<a b=<>{c}</>/>", "<a b=<>{c}</> />;\n");
    expect_printed_jsx("<a b={<c/>}/>", "<a b={<c />} />;\n");
    expect_printed_jsx("<a b={<>c</>}/>", "<a b={<>c</>} />;\n");
    expect_printed_jsx("<a b={<>{c}</>}/>", "<a b={<>{c}</>} />;\n");
    expect_printed_jsx_ascii("<\u{3c0}/>", "<\u{3c0} />;\n");
    expect_printed_jsx_ascii("<\u{3c0}.\u{10000}/>", "<\u{3c0}.\u{10000} />;\n");
    expect_printed_jsx_ascii("<\u{10000}.\u{3c0}/>", "<\u{10000}.\u{3c0} />;\n");
    expect_printed_jsx_ascii("<\u{3c0}>x</\u{3c0}>", "<\u{3c0}>x</\u{3c0}>;\n");
    expect_printed_jsx_ascii("<\u{10000}>x</\u{10000}>", "<\u{10000}>x</\u{10000}>;\n");
    expect_printed_jsx_ascii("<a \u{3c0}/>", "<a \u{3c0} />;\n");
    expect_printed_jsx_ascii("<a \u{10000}/>", "<a \u{10000} />;\n");
    expect_printed_jsx_ascii("<a b='\u{3c0}'/>", "<a b='\u{3c0}' />;\n");
    expect_printed_jsx_ascii("<a b='\u{10000}'/>", "<a b='\u{10000}' />;\n");
    expect_printed_jsx_ascii("<a>\u{3c0}</a>", "<a>\u{3c0}</a>;\n");
    expect_printed_jsx_ascii("<a>\u{10000}</a>", "<a>\u{10000}</a>;\n");
    expect_printed_jsx_minify("<a b c={x,y} d='true'/>", "<a b c={(x,y)}d='true'/>;");
    expect_printed_jsx_minify("<a><b/><c/></a>", "<a><b/><c/></a>;");
    expect_printed_jsx_minify("<a> x <b/> y </a>", "<a> x <b/> y </a>;");
    expect_printed_jsx_minify(
        "<a>{' x '}{'<b/>'}{' y '}</a>",
        "<a>{\" x \"}{\"<b/>\"}{\" y \"}</a>;",
    );
}

#[test]
fn test_jsx_single_line() {
    expect_printed_jsx("<x/>", "<x />;\n");
    expect_printed_jsx("<x y/>", "<x y />;\n");
    expect_printed_jsx("<x\n/>", "<x />;\n");
    expect_printed_jsx("<x\ny/>", "<x y />;\n");
    expect_printed_jsx("<x y\n/>", "<x y />;\n");
    expect_printed_jsx("<x\n{...y}/>", "<x {...y} />;\n");
    expect_printed_jsx_minify("<x/>", "<x/>;");
    expect_printed_jsx_minify("<x y/>", "<x y/>;");
    expect_printed_jsx_minify("<x\n/>", "<x/>;");
    expect_printed_jsx_minify("<x\ny/>", "<x y/>;");
    expect_printed_jsx_minify("<x y\n/>", "<x y/>;");
    expect_printed_jsx_minify("<x\n{...y}/>", "<x{...y}/>;");
}

#[test]
fn test_avoid_slash_script() {
    expect_printed("x = '</script'", "x = \"<\\/script\";\n");
    expect_printed("x = `</script`", "x = `<\\/script`;\n");
    expect_printed("x = `</SCRIPT`", "x = `<\\/SCRIPT`;\n");
    expect_printed("x = `</ScRiPt`", "x = `<\\/ScRiPt`;\n");
    expect_printed("x = `</script${y}`", "x = `<\\/script${y}`;\n");
    expect_printed("x = `${y}</script`", "x = `${y}<\\/script`;\n");
    expect_printed_minify(
        "x = 1 < /script/.exec(y).length",
        "x=1< /script/.exec(y).length;",
    );
    expect_printed_minify(
        "x = 1 < /SCRIPT/.exec(y).length",
        "x=1< /SCRIPT/.exec(y).length;",
    );
    expect_printed_minify(
        "x = 1 < /ScRiPt/.exec(y).length",
        "x=1< /ScRiPt/.exec(y).length;",
    );
    expect_printed_minify(
        "x = 1 << /script/.exec(y).length",
        "x=1<< /script/.exec(y).length;",
    );
    expect_printed(
        "//! </script\n//! >/script\n//! /script",
        "//! <\\/script\n//! >/script\n//! /script\n",
    );
    expect_printed(
        "//! </SCRIPT\n//! >/SCRIPT\n//! /SCRIPT",
        "//! <\\/SCRIPT\n//! >/SCRIPT\n//! /SCRIPT\n",
    );
    expect_printed(
        "//! </ScRiPt\n//! >/ScRiPt\n//! /ScRiPt",
        "//! <\\/ScRiPt\n//! >/ScRiPt\n//! /ScRiPt\n",
    );
    expect_printed(
        "/*! </script \n </script */",
        "/*! <\\/script \n <\\/script */\n",
    );
    expect_printed(
        "/*! </SCRIPT \n </SCRIPT */",
        "/*! <\\/SCRIPT \n <\\/SCRIPT */\n",
    );
    expect_printed(
        "/*! </ScRiPt \n </ScRiPt */",
        "/*! <\\/ScRiPt \n <\\/ScRiPt */\n",
    );
    // expect_printed("String.raw`</script`", "import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/script\"])));\n");
    // expect_printed("String.raw`</script${a}`", "import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/script\", \"\"])), a);\n");
    // expect_printed("String.raw`${a}</script`", "import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"\", \"<\\/script\"])), a);\n");
    // expect_printed("String.raw`</SCRIPT`", "import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/SCRIPT\"])));\n");
    // expect_printed("String.raw`</ScRiPt`", "import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/ScRiPt\"])));\n");
    expect_printed("x = '</'", "x = \"</\";\n");
    expect_printed("x = '</ script'", "x = \"</ script\";\n");
    expect_printed("x = '< /script'", "x = \"< /script\";\n");
    expect_printed("x = '/script>'", "x = \"/script>\";\n");
    expect_printed("x = '<script>'", "x = \"<script>\";\n");
    expect_printed_minify(
        "x = 1 < / script/.exec(y).length",
        "x=1</ script/.exec(y).length;",
    );
    expect_printed_minify(
        "x = 1 << / script/.exec(y).length",
        "x=1<</ script/.exec(y).length;",
    );
}

#[test]
fn test_infinity() {
    expect_printed("x = Infinity", "x = Infinity;\n");
    expect_printed("x = -Infinity", "x = -Infinity;\n");
    expect_printed("x = (Infinity).toString", "x = Infinity.toString;\n");
    expect_printed("x = (-Infinity).toString", "x = (-Infinity).toString;\n");
    expect_printed("x = (Infinity) ** 2", "x = Infinity ** 2;\n");
    expect_printed("x = (-Infinity) ** 2", "x = (-Infinity) ** 2;\n");
    expect_printed("x = ~Infinity", "x = ~Infinity;\n");
    expect_printed("x = ~-Infinity", "x = ~-Infinity;\n");
    expect_printed("x = Infinity * y", "x = Infinity * y;\n");
    expect_printed("x = Infinity / y", "x = Infinity / y;\n");
    expect_printed("x = y * Infinity", "x = y * Infinity;\n");
    expect_printed("x = y / Infinity", "x = y / Infinity;\n");
    expect_printed("throw Infinity", "throw Infinity;\n");
    expect_printed_minify("x = Infinity", "x=Infinity;");
    expect_printed_minify("x = -Infinity", "x=-Infinity;");
    expect_printed_minify("x = (Infinity).toString", "x=Infinity.toString;");
    expect_printed_minify("x = (-Infinity).toString", "x=(-Infinity).toString;");
    expect_printed_minify("x = (Infinity) ** 2", "x=Infinity**2;");
    expect_printed_minify("x = (-Infinity) ** 2", "x=(-Infinity)**2;");
    expect_printed_minify("x = ~Infinity", "x=~Infinity;");
    expect_printed_minify("x = ~-Infinity", "x=~-Infinity;");
    expect_printed_minify("x = Infinity * y", "x=Infinity*y;");
    expect_printed_minify("x = Infinity / y", "x=Infinity/y;");
    expect_printed_minify("x = y * Infinity", "x=y*Infinity;");
    expect_printed_minify("x = y / Infinity", "x=y/Infinity;");
    expect_printed_minify("throw Infinity", "throw Infinity;");
}

#[test]
fn test_binary_operator_visitor() {
    let source = format!("x = f(){};\n", " || f()".repeat(10_000));
    expect_printed(&source, &source);
}

#[test]
fn test_using() {
    expect_printed("using x = y", "using x = y;\n");
    expect_printed("using x = y, z = _", "using x = y, z = _;\n");
    expect_printed_minify("using x = y", "using x=y;");
    expect_printed_minify("using x = y, z = _", "using x=y,z=_;");
    expect_printed("await using x = y", "await using x = y;\n");
    expect_printed("await using x = y, z = _", "await using x = y, z = _;\n");
    expect_printed_minify("await using x = y", "await using x=y;");
    expect_printed_minify("await using x = y, z = _", "await using x=y,z=_;");
}

#[test]
fn test_control_flow_minify_and_with() {
    expect_printed_minify(
        "if (x) y(); else z(); for (;;) x(); while(x)y(); do x(); while(y); switch(x){case 1:y();default:z()} try{x()}catch(e){y()}finally{z()} with(x)1e400",
        "if(x)y();else z();for(;;)x();while(x)y();do x();while(y);switch(x){case 1:y();default:z()}try{x()}catch(e){y()}finally{z()}with(x)1/0;",
    );
    expect_printed("with (x) 1e400", "with (x) 1 / 0;\n");
    expect_printed_minify(
        "switch(x){case\"a\":break;case{}:break} throw{}",
        "switch(x){case\"a\":break;case{}:break}throw{};",
    );
    expect_printed_minify(
        "let {a}=x; function* foo(){}; throw function(){}",
        "let{a}=x;function*foo(){};throw function(){};",
    );
}
