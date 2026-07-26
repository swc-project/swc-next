// Ported from upstream esbuild's internal/js_printer/js_printer_test.go.
use super::{
    expect_printed, expect_printed_ascii, expect_printed_jsx, expect_printed_jsx_ascii,
    expect_printed_jsx_minify, expect_printed_minify, expect_printed_minify_ascii,
    expect_printed_ts,
};

#[test]
fn test_number() {
    expect_printed(b"x = 1e-100", b"x = 1e-100;\n");
    expect_printed(b"x = 1e-4", b"x = 1e-4;\n");
    expect_printed(b"x = 1e-3", b"x = 1e-3;\n");
    expect_printed(b"x = 1e-2", b"x = 0.01;\n");
    expect_printed(b"x = 1e-1", b"x = 0.1;\n");
    expect_printed(b"x = 1e0", b"x = 1;\n");
    expect_printed(b"x = 1e1", b"x = 10;\n");
    expect_printed(b"x = 1e2", b"x = 100;\n");
    expect_printed(b"x = 1e3", b"x = 1e3;\n");
    expect_printed(b"x = 1e4", b"x = 1e4;\n");
    expect_printed(b"x = 1e100", b"x = 1e100;\n");
    expect_printed_minify(b"x = 1e-100", b"x=1e-100;");
    expect_printed_minify(b"x = 1e-5", b"x=1e-5;");
    expect_printed_minify(b"x = 1e-4", b"x=1e-4;");
    expect_printed_minify(b"x = 1e-3", b"x=.001;");
    expect_printed_minify(b"x = 1e-2", b"x=.01;");
    expect_printed_minify(b"x = 1e-1", b"x=.1;");
    expect_printed_minify(b"x = 1e0", b"x=1;");
    expect_printed_minify(b"x = 1e1", b"x=10;");
    expect_printed_minify(b"x = 1e2", b"x=100;");
    expect_printed_minify(b"x = 1e3", b"x=1e3;");
    expect_printed_minify(b"x = 1e4", b"x=1e4;");
    expect_printed_minify(b"x = 1e100", b"x=1e100;");
    expect_printed(b"x = 12e-100", b"x = 12e-100;\n");
    expect_printed(b"x = 12e-5", b"x = 12e-5;\n");
    expect_printed(b"x = 12e-4", b"x = 12e-4;\n");
    expect_printed(b"x = 12e-3", b"x = 0.012;\n");
    expect_printed(b"x = 12e-2", b"x = 0.12;\n");
    expect_printed(b"x = 12e-1", b"x = 1.2;\n");
    expect_printed(b"x = 12e0", b"x = 12;\n");
    expect_printed(b"x = 12e1", b"x = 120;\n");
    expect_printed(b"x = 12e2", b"x = 1200;\n");
    expect_printed(b"x = 12e3", b"x = 12e3;\n");
    expect_printed(b"x = 12e4", b"x = 12e4;\n");
    expect_printed(b"x = 12e100", b"x = 12e100;\n");
    expect_printed_minify(b"x = 12e-100", b"x=12e-100;");
    expect_printed_minify(b"x = 12e-6", b"x=12e-6;");
    expect_printed_minify(b"x = 12e-5", b"x=12e-5;");
    expect_printed_minify(b"x = 12e-4", b"x=.0012;");
    expect_printed_minify(b"x = 12e-3", b"x=.012;");
    expect_printed_minify(b"x = 12e-2", b"x=.12;");
    expect_printed_minify(b"x = 12e-1", b"x=1.2;");
    expect_printed_minify(b"x = 12e0", b"x=12;");
    expect_printed_minify(b"x = 12e1", b"x=120;");
    expect_printed_minify(b"x = 12e2", b"x=1200;");
    expect_printed_minify(b"x = 12e3", b"x=12e3;");
    expect_printed_minify(b"x = 12e4", b"x=12e4;");
    expect_printed_minify(b"x = 12e100", b"x=12e100;");
    expect_printed(b"x = 123456789", b"x = 123456789;\n");
    expect_printed(b"x = 1123456789", b"x = 1123456789;\n");
    expect_printed(b"x = 10123456789", b"x = 10123456789;\n");
    expect_printed(b"x = 100123456789", b"x = 100123456789;\n");
    expect_printed(b"x = 1000123456789", b"x = 1000123456789;\n");
    expect_printed(b"x = 10000123456789", b"x = 10000123456789;\n");
    expect_printed(b"x = 100000123456789", b"x = 100000123456789;\n");
    expect_printed(b"x = 1000000123456789", b"x = 1000000123456789;\n");
    expect_printed(b"x = 10000000123456789", b"x = 10000000123456788;\n");
    expect_printed(b"x = 100000000123456789", b"x = 100000000123456780;\n");
    expect_printed(b"x = 1000000000123456789", b"x = 1000000000123456800;\n");
    expect_printed(b"x = 10000000000123456789", b"x = 10000000000123458e3;\n");
    expect_printed(b"x = 100000000000123456789", b"x = 10000000000012345e4;\n");
    expect_printed(b"x = 0x7fff_ffff", b"x = 2147483647;\n");
    expect_printed(b"x = 0x8000_0000", b"x = 2147483648;\n");
    expect_printed(b"x = 0x8000_0001", b"x = 2147483649;\n");
    expect_printed(b"x = -0x7fff_ffff", b"x = -2147483647;\n");
    expect_printed(b"x = -0x8000_0000", b"x = -2147483648;\n");
    expect_printed(b"x = -0x8000_0001", b"x = -2147483649;\n");
    expect_printed(b"x = 0xffff_ffff", b"x = 4294967295;\n");
    expect_printed(b"x = 0x1_0000_0000", b"x = 4294967296;\n");
    expect_printed(b"x = 0x1_0000_0001", b"x = 4294967297;\n");
    expect_printed(b"x = -0xffff_ffff", b"x = -4294967295;\n");
    expect_printed(b"x = -0x1_0000_0000", b"x = -4294967296;\n");
    expect_printed(b"x = -0x1_0000_0001", b"x = -4294967297;\n");
    expect_printed(b"x = 0x7fff_ffff_ffff_fdff", b"x = 9223372036854775e3;\n");
    expect_printed(b"x = 0x8000_0000_0000_0000", b"x = 9223372036854776e3;\n");
    expect_printed(b"x = 0x8000_0000_0000_3000", b"x = 9223372036854788e3;\n");
    expect_printed(b"x = -0x7fff_ffff_ffff_fdff", b"x = -9223372036854775e3;\n");
    expect_printed(b"x = -0x8000_0000_0000_0000", b"x = -9223372036854776e3;\n");
    expect_printed(b"x = -0x8000_0000_0000_3000", b"x = -9223372036854788e3;\n");
    expect_printed(b"x = 0xffff_ffff_ffff_fbff", b"x = 1844674407370955e4;\n");
    expect_printed(
        b"x = 0x1_0000_0000_0000_0000",
        b"x = 18446744073709552e3;\n",
    );
    expect_printed(
        b"x = 0x1_0000_0000_0000_1000",
        b"x = 18446744073709556e3;\n",
    );
    expect_printed(b"x = -0xffff_ffff_ffff_fbff", b"x = -1844674407370955e4;\n");
    expect_printed(
        b"x = -0x1_0000_0000_0000_0000",
        b"x = -18446744073709552e3;\n",
    );
    expect_printed(
        b"x = -0x1_0000_0000_0000_1000",
        b"x = -18446744073709556e3;\n",
    );
    expect_printed(b"x = 999999999999", b"x = 999999999999;\n");
    expect_printed(b"x = 1000000000001", b"x = 1000000000001;\n");
    expect_printed(b"x = 0x0FFF_FFFF_FFFF_FF80", b"x = 1152921504606846800;\n");
    expect_printed(b"x = 0x1000_0000_0000_0000", b"x = 1152921504606847e3;\n");
    expect_printed(b"x = 0xFFFF_FFFF_FFFF_F000", b"x = 18446744073709548e3;\n");
    expect_printed(b"x = 0xFFFF_FFFF_FFFF_F800", b"x = 1844674407370955e4;\n");
    expect_printed(b"x = 0xFFFF_FFFF_FFFF_FFFF", b"x = 18446744073709552e3;\n");
    expect_printed_minify(b"x = 999999999999", b"x=999999999999;");
    expect_printed_minify(b"x = 1000000000001", b"x=0xe8d4a51001;");
    expect_printed_minify(b"x = 0x0FFF_FFFF_FFFF_FF80", b"x=0xfffffffffffff80;");
    expect_printed_minify(b"x = 0x1000_0000_0000_0000", b"x=1152921504606847e3;");
    expect_printed_minify(b"x = 0xFFFF_FFFF_FFFF_F000", b"x=0xfffffffffffff000;");
    expect_printed_minify(b"x = 0xFFFF_FFFF_FFFF_F800", b"x=1844674407370955e4;");
    expect_printed_minify(b"x = 0xFFFF_FFFF_FFFF_FFFF", b"x=18446744073709552e3;");
    expect_printed_minify(b"x = 0.0001 .y", b"x=1e-4.y;");
    expect_printed_minify(b"x = 0.001 .y", b"x=.001.y;");
    expect_printed_minify(b"x = 0.01 .y", b"x=.01.y;");
    expect_printed_minify(b"x = 0.1 .y", b"x=.1.y;");
    expect_printed_minify(b"x = 0 .y", b"x=0 .y;");
    expect_printed_minify(b"x = 10 .y", b"x=10 .y;");
    expect_printed_minify(b"x = 100 .y", b"x=100 .y;");
    expect_printed_minify(b"x = 1000 .y", b"x=1e3.y;");
    expect_printed_minify(b"x = 12345 .y", b"x=12345 .y;");
    expect_printed_minify(b"x = 0xFFFF_0000_FFFF_0000 .y", b"x=0xffff0000ffff0000.y;");
}

#[test]
fn test_array() {
    expect_printed(b"[]", b"[];\n");
    expect_printed(b"[,]", b"[,];\n");
    expect_printed(b"[,,]", b"[, ,];\n");
}

#[test]
fn test_splat() {
    expect_printed(b"[...(a, b)]", b"[...(a, b)];\n");
    expect_printed(b"x(...(a, b))", b"x(...(a, b));\n");
    expect_printed(b"({...(a, b)})", b"({ ...(a, b) });\n");
}

#[test]
fn test_new() {
    expect_printed(b"new x", b"new x();\n");
    expect_printed(b"new x()", b"new x();\n");
    expect_printed(b"new (x)", b"new x();\n");
    expect_printed(b"new (x())", b"new (x())();\n");
    expect_printed(b"new (new x())", b"new new x()();\n");
    expect_printed(b"new (x + x)", b"new (x + x)();\n");
    expect_printed(b"(new x)()", b"new x()();\n");
    expect_printed(b"new foo().bar", b"new foo().bar;\n");
    expect_printed(b"new (foo().bar)", b"new (foo()).bar();\n");
    expect_printed(b"new (foo()).bar", b"new (foo()).bar();\n");
    expect_printed(b"new foo()[bar]", b"new foo()[bar];\n");
    expect_printed(b"new (foo()[bar])", b"new (foo())[bar]();\n");
    expect_printed(b"new (foo())[bar]", b"new (foo())[bar]();\n");
    expect_printed(
        b"new (import('foo').bar)",
        b"new (import(\"foo\")).bar();\n",
    );
    expect_printed(
        b"new (import('foo')).bar",
        b"new (import(\"foo\")).bar();\n",
    );
    expect_printed(
        b"new (import('foo')[bar])",
        b"new (import(\"foo\"))[bar]();\n",
    );
    expect_printed(
        b"new (import('foo'))[bar]",
        b"new (import(\"foo\"))[bar]();\n",
    );
    expect_printed_minify(b"new x", b"new x;");
    expect_printed_minify(b"new x.y", b"new x.y;");
    expect_printed_minify(b"(new x).y", b"new x().y;");
    expect_printed_minify(b"new x().y", b"new x().y;");
    expect_printed_minify(b"new x() + y", b"new x+y;");
    expect_printed_minify(b"new x() ** 2", b"new x**2;");
    expect_printed(
        b"new Worker(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
        b"new Worker(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\"\n);\n",
    );
    expect_printed(
        b"new Worker(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
        b"new Worker(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\"\n);\n",
    );
    expect_printed(
        b"new Worker(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
        b"new Worker(\n  /* multi\n   * line\n   * webpackBar: */\n  \"path\"\n);\n",
    );
    expect_printed(
        b"new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
        b"new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    expect_printed(
        b"new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);",
        b"new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n);\n",
    );
    expect_printed(
        b"new Worker(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
        b"new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    expect_printed(b"new Worker(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))", b"new Worker(new URL(\n  \"path\",\n  /* webpackFoo: these can go anywhere */\n  import.meta.url\n));\n");
}

#[test]
fn test_call() {
    expect_printed(b"x()()()", b"x()()();\n");
    expect_printed(b"x().y()[z]()", b"x().y()[z]();\n");
    expect_printed(b"(--x)();", b"(--x)();\n");
    expect_printed(b"(x--)();", b"(x--)();\n");
    expect_printed(b"eval(x)", b"eval(x);\n");
    expect_printed(b"eval?.(x)", b"eval?.(x);\n");
    expect_printed(b"(eval)(x)", b"eval(x);\n");
    expect_printed(b"(eval)?.(x)", b"eval?.(x);\n");
    expect_printed(b"eval(x, y)", b"eval(x, y);\n");
    expect_printed(b"eval?.(x, y)", b"eval?.(x, y);\n");
    expect_printed(b"(1, eval)(x)", b"(1, eval)(x);\n");
    expect_printed(b"(1, eval)?.(x)", b"(1, eval)?.(x);\n");
    expect_printed_minify(b"eval?.(x)", b"eval?.(x);");
    expect_printed_minify(b"eval(x,y)", b"eval(x,y);");
    expect_printed_minify(b"eval?.(x,y)", b"eval?.(x,y);");
    expect_printed_minify(b"(1, eval)(x)", b"(1,eval)(x);");
    expect_printed_minify(b"(1, eval)?.(x)", b"(1,eval)?.(x);");
}

#[test]
fn test_member() {
    expect_printed(b"x.y[z]", b"x.y[z];\n");
    expect_printed(b"((x+1).y+1)[z]", b"((x + 1).y + 1)[z];\n");
}

#[test]
fn test_comma() {
    expect_printed(b"1, 2, 3", b"1, 2, 3;\n");
    expect_printed(b"(1, 2), 3", b"1, 2, 3;\n");
    expect_printed(b"1, (2, 3)", b"1, 2, 3;\n");
    expect_printed(b"a ? (b, c) : (d, e)", b"a ? (b, c) : (d, e);\n");
    expect_printed(b"let x = (a, b)", b"let x = (a, b);\n");
    expect_printed(b"(x = a), b", b"x = a, b;\n");
    expect_printed(b"x = (a, b)", b"x = (a, b);\n");
    expect_printed(b"x((1, 2))", b"x((1, 2));\n");
}

#[test]
fn test_unary() {
    expect_printed(b"+(x--)", b"+x--;\n");
    expect_printed(b"-(x++)", b"-x++;\n");
}

#[test]
fn test_nullish() {
    expect_printed(b"(a && b) ?? c", b"(a && b) ?? c;\n");
    expect_printed(b"(a || b) ?? c", b"(a || b) ?? c;\n");
    expect_printed(b"a ?? (b && c)", b"a ?? (b && c);\n");
    expect_printed(b"a ?? (b || c)", b"a ?? (b || c);\n");
    expect_printed(b"a && (b ?? c)", b"a && (b ?? c);\n");
    expect_printed(b"a || (b ?? c)", b"a || (b ?? c);\n");
    expect_printed(b"(a ?? b) && c", b"(a ?? b) && c;\n");
    expect_printed(b"(a ?? b) || c", b"(a ?? b) || c;\n");
}

#[test]
fn test_string() {
    expect_printed(b"let x = ''", b"let x = \"\";\n");
    expect_printed(b"let x = '\x08'", b"let x = \"\\b\";\n");
    expect_printed(b"let x = '\x0c'", b"let x = \"\\f\";\n");
    expect_printed(b"let x = '\t'", b"let x = \"\t\";\n");
    expect_printed(b"let x = '\x0b'", b"let x = \"\\v\";\n");
    expect_printed(b"let x = '\\n'", b"let x = \"\\n\";\n");
    expect_printed(b"let x = '\\''", b"let x = \"'\";\n");
    expect_printed(b"let x = '\\\"'", b"let x = '\"';\n");
    expect_printed(b"let x = '\\'\"'", b"let x = `'\"`;\n");
    expect_printed(b"let x = '\\\\'", b"let x = \"\\\\\";\n");
    expect_printed(b"let x = '\x00'", b"let x = \"\\0\";\n");
    expect_printed(b"let x = '\x00!'", b"let x = \"\\0!\";\n");
    expect_printed(b"let x = '\x001'", b"let x = \"\\x001\";\n");
    expect_printed(b"let x = '\\0'", b"let x = \"\\0\";\n");
    expect_printed(b"let x = '\\0!'", b"let x = \"\\0!\";\n");
    expect_printed(b"let x = '\x07'", b"let x = \"\\x07\";\n");
    expect_printed(b"let x = '\x07!'", b"let x = \"\\x07!\";\n");
    expect_printed(b"let x = '\x071'", b"let x = \"\\x071\";\n");
    expect_printed(b"let x = '\\7'", b"let x = \"\\x07\";\n");
    expect_printed(b"let x = '\\7!'", b"let x = \"\\x07!\";\n");
    expect_printed(b"let x = '\\01'", b"let x = \"\x01\";\n");
    expect_printed(b"let x = '\x10'", b"let x = \"\x10\";\n");
    expect_printed(b"let x = '\\x10'", b"let x = \"\x10\";\n");
    expect_printed(b"let x = '\x1b'", b"let x = \"\\x1B\";\n");
    expect_printed(b"let x = '\\x1B'", b"let x = \"\\x1B\";\n");
    expect_printed(b"let x = '\xea\xaf\x8d'", b"let x = \"\xea\xaf\x8d\";\n");
    expect_printed(b"let x = '\\uABCD'", b"let x = \"\xea\xaf\x8d\";\n");
    expect_printed(
        b"let x = '\xf0\x92\x8e\xab'",
        b"let x = \"\xf0\x92\x8e\xab\";\n",
    );
    expect_printed(b"let x = '\\u{123AB}'", b"let x = \"\xf0\x92\x8e\xab\";\n");
    expect_printed(
        b"let x = '\\uD808\\uDFAB'",
        b"let x = \"\xf0\x92\x8e\xab\";\n",
    );
    expect_printed(b"let x = '\\uD808'", b"let x = \"\\uD808\";\n");
    expect_printed(b"let x = '\\uD808X'", b"let x = \"\\uD808X\";\n");
    expect_printed(b"let x = '\\uDFAB'", b"let x = \"\\uDFAB\";\n");
    expect_printed(b"let x = '\\uDFABX'", b"let x = \"\\uDFABX\";\n");
    expect_printed(b"let x = '\\x80'", b"let x = \"\xc2\x80\";\n");
    expect_printed(b"let x = '\\xFF'", b"let x = \"\xc3\xbf\";\n");
    expect_printed(
        b"let x = '\\xF0\\x9F\\x8D\\x95'",
        b"let x = \"\xc3\xb0\xc2\x9f\xc2\x8d\xc2\x95\";\n",
    );
    expect_printed(
        b"let x = '\\uD801\\uDC02\\uDC03\\uD804'",
        b"let x = \"\xf0\x90\x90\x82\\uDC03\\uD804\";\n",
    );
}

#[test]
fn test_template() {
    expect_printed(b"let x = `\\0`", b"let x = `\\0`;\n");
    expect_printed(b"let x = `\\x01`", b"let x = `\x01`;\n");
    expect_printed(b"let x = `\\0${0}`", b"let x = `\\0${0}`;\n");
    expect_printed(b"let x = `\\x01${0}`", b"let x = `\x01${0}`;\n");
    expect_printed(b"let x = `${0}\\0`", b"let x = `${0}\\0`;\n");
    expect_printed(b"let x = `${0}\\x01`", b"let x = `${0}\x01`;\n");
    expect_printed(b"let x = `${0}\\0${1}`", b"let x = `${0}\\0${1}`;\n");
    expect_printed(b"let x = `${0}\\x01${1}`", b"let x = `${0}\x01${1}`;\n");
    expect_printed(b"let x = String.raw`\\1`", b"let x = String.raw`\\1`;\n");
    expect_printed(
        b"let x = String.raw`\\x01`",
        b"let x = String.raw`\\x01`;\n",
    );
    expect_printed(
        b"let x = String.raw`\\1${0}`",
        b"let x = String.raw`\\1${0}`;\n",
    );
    expect_printed(
        b"let x = String.raw`\\x01${0}`",
        b"let x = String.raw`\\x01${0}`;\n",
    );
    expect_printed(
        b"let x = String.raw`${0}\\1`",
        b"let x = String.raw`${0}\\1`;\n",
    );
    expect_printed(
        b"let x = String.raw`${0}\\x01`",
        b"let x = String.raw`${0}\\x01`;\n",
    );
    expect_printed(
        b"let x = String.raw`${0}\\1${1}`",
        b"let x = String.raw`${0}\\1${1}`;\n",
    );
    expect_printed(
        b"let x = String.raw`${0}\\x01${1}`",
        b"let x = String.raw`${0}\\x01${1}`;\n",
    );
    expect_printed(b"let x = `${y}`", b"let x = `${y}`;\n");
    expect_printed(b"let x = `$(y)`", b"let x = `$(y)`;\n");
    expect_printed(b"let x = `{y}$`", b"let x = `{y}$`;\n");
    expect_printed(b"let x = `$}y{`", b"let x = `$}y{`;\n");
    expect_printed(b"let x = `\\${y}`", b"let x = `\\${y}`;\n");
    expect_printed(b"let x = `$\\{y}`", b"let x = `\\${y}`;\n");
    expect_printed(b"await tag`x`", b"await tag`x`;\n");
    expect_printed(b"await (tag`x`)", b"await tag`x`;\n");
    expect_printed(b"(await tag)`x`", b"(await tag)`x`;\n");
    expect_printed(b"await tag`${x}`", b"await tag`${x}`;\n");
    expect_printed(b"await (tag`${x}`)", b"await tag`${x}`;\n");
    expect_printed(b"(await tag)`${x}`", b"(await tag)`${x}`;\n");
    expect_printed(b"new tag`x`", b"new tag`x`();\n");
    expect_printed(b"new (tag`x`)", b"new tag`x`();\n");
    expect_printed(b"new tag()`x`", b"new tag()`x`;\n");
    expect_printed(b"(new tag)`x`", b"new tag()`x`;\n");
    expect_printed_minify(b"new tag`x`", b"new tag`x`;");
    expect_printed_minify(b"new (tag`x`)", b"new tag`x`;");
    expect_printed_minify(b"new tag()`x`", b"new tag()`x`;");
    expect_printed_minify(b"(new tag)`x`", b"new tag()`x`;");
    expect_printed(b"new tag`${x}`", b"new tag`${x}`();\n");
    expect_printed(b"new (tag`${x}`)", b"new tag`${x}`();\n");
    expect_printed(b"new tag()`${x}`", b"new tag()`${x}`;\n");
    expect_printed(b"(new tag)`${x}`", b"new tag()`${x}`;\n");
    expect_printed_minify(b"new tag`${x}`", b"new tag`${x}`;");
    expect_printed_minify(b"new (tag`${x}`)", b"new tag`${x}`;");
    expect_printed_minify(b"new tag()`${x}`", b"new tag()`${x}`;");
    expect_printed_minify(b"(new tag)`${x}`", b"new tag()`${x}`;");
}

#[test]
fn test_object() {
    expect_printed(b"let x = {'(':')'}", b"let x = { \"(\": \")\" };\n");
    expect_printed(b"({})", b"({});\n");
    expect_printed(b"({}.x)", b"({}).x;\n");
    expect_printed(b"({} = {})", b"({} = {});\n");
    expect_printed(b"(x, {} = {})", b"x, {} = {};\n");
    expect_printed(b"let x = () => ({})", b"let x = () => ({});\n");
    expect_printed(b"let x = () => ({}.x)", b"let x = () => ({}).x;\n");
    expect_printed(b"let x = () => ({} = {})", b"let x = () => ({} = {});\n");
    expect_printed(
        b"let x = () => (x, {} = {})",
        b"let x = () => (x, {} = {});\n",
    );
    expect_printed(
        b"function foo(__proto__) { return { __proto__: __proto__ } }",
        b"function foo(__proto__) {\n  return { __proto__: __proto__ };\n}\n",
    );
    expect_printed(
        b"function foo(__proto__) { return { '__proto__': __proto__ } }",
        b"function foo(__proto__) {\n  return { \"__proto__\": __proto__ };\n}\n",
    );
    expect_printed(
        b"function foo(__proto__) { return { ['__proto__']: __proto__ } }",
        b"function foo(__proto__) {\n  return { [\"__proto__\"]: __proto__ };\n}\n",
    );
    expect_printed(
        b"import { __proto__ } from 'foo'; let foo = () => ({ __proto__: __proto__ })",
        b"import { __proto__ } from \"foo\";\nlet foo = () => ({ __proto__: __proto__ });\n",
    );
    expect_printed(
        b"import { __proto__ } from 'foo'; let foo = () => ({ '__proto__': __proto__ })",
        b"import { __proto__ } from \"foo\";\nlet foo = () => ({ \"__proto__\": __proto__ });\n",
    );
    expect_printed(
        b"import { __proto__ } from 'foo'; let foo = () => ({ ['__proto__']: __proto__ })",
        b"import { __proto__ } from \"foo\";\nlet foo = () => ({ [\"__proto__\"]: __proto__ });\n",
    );
}

#[test]
fn test_switch() {
    expect_printed(b"switch (x) { /* 1 */ case 1: /* 2 */ case 2: /* default */ default: break }", b"switch (x) {\n  /* 1 */\n  case 1:\n  /* 2 */\n  case 2:\n  /* default */\n  default:\n    break;\n}\n");
}

#[test]
fn test_for() {
    expect_printed(b"for ((a in b);;);", b"for ((a in b); ; ) ;\n");
    expect_printed(
        b"for (a ? b : (c in d);;);",
        b"for (a ? b : (c in d); ; ) ;\n",
    );
    expect_printed(
        b"for ((a ? b : c in d).foo;;);",
        b"for ((a ? b : c in d).foo; ; ) ;\n",
    );
    expect_printed(
        b"for (var x = (a in b);;);",
        b"for (var x = (a in b); ; ) ;\n",
    );
    expect_printed(b"for (x = (a in b);;);", b"for (x = (a in b); ; ) ;\n");
    expect_printed(b"for (x == (a in b);;);", b"for (x == (a in b); ; ) ;\n");
    expect_printed(
        b"for (1 * (x == a in b);;);",
        b"for (1 * (x == a in b); ; ) ;\n",
    );
    expect_printed(
        b"for (a ? b : x = (c in d);;);",
        b"for (a ? b : x = (c in d); ; ) ;\n",
    );
    expect_printed(
        b"for (var x = y = (a in b);;);",
        b"for (var x = y = (a in b); ; ) ;\n",
    );
    expect_printed(b"for ([a in b];;);", b"for ([a in b]; ; ) ;\n");
    expect_printed(b"for (x(a in b);;);", b"for (x(a in b); ; ) ;\n");
    expect_printed(b"for (x[a in b];;);", b"for (x[a in b]; ; ) ;\n");
    expect_printed(b"for (x?.[a in b];;);", b"for (x?.[a in b]; ; ) ;\n");
    expect_printed(b"for ((x => a in b);;);", b"for (((x) => a in b); ; ) ;\n");
    expect_printed(b"for (let a in b, c);", b"for (let a in b, c) ;\n");
    expect_printed(b"for (let a of (b, c));", b"for (let a of (b, c)) ;\n");
}

#[test]
fn test_function() {
    expect_printed(
        b"function foo(a = (b, c), ...d) {}",
        b"function foo(a = (b, c), ...d) {\n}\n",
    );
    expect_printed(
        b"function foo({[1 + 2]: a = 3} = {[1 + 2]: 3}) {}",
        b"function foo({ [1 + 2]: a = 3 } = { [1 + 2]: 3 }) {\n}\n",
    );
    expect_printed(
        b"function foo([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) {}",
        b"function foo([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) {\n}\n",
    );
    expect_printed(b"function foo([] = []) {}", b"function foo([] = []) {\n}\n");
    expect_printed(
        b"function foo([,] = [,]) {}",
        b"function foo([,] = [,]) {\n}\n",
    );
    expect_printed(
        b"function foo([,,] = [,,]) {}",
        b"function foo([, ,] = [, ,]) {\n}\n",
    );
}

#[test]
fn test_comments_and_parentheses() {
    expect_printed(
        b"(/* foo */ { x() { foo() } }.x());",
        b"/* foo */\n({ x() {\n  foo();\n} }).x();\n",
    );
    expect_printed(
        b"(/* foo */ function f() { foo(f) }());",
        b"/* foo */\n(function f() {\n  foo(f);\n})();\n",
    );
    expect_printed(
        b"(/* foo */ class x { static y() { foo(x) } }.y());",
        b"/* foo */\n(class x {\n  static y() {\n    foo(x);\n  }\n}).y();\n",
    );
    expect_printed(
        b"(/* @__PURE__ */ (() => foo())());",
        b"/* @__PURE__ */ (() => foo())();\n",
    );
    expect_printed(
        b"export default (/* foo */ function f() {});",
        b"export default (\n  /* foo */\n  (function f() {\n  })\n);\n",
    );
    expect_printed(
        b"export default (/* foo */ class x {});",
        b"export default (\n  /* foo */\n  class x {\n  }\n);\n",
    );
    expect_printed(
        b"x = () => (/* foo */ {});",
        b"x = () => (\n  /* foo */\n  {}\n);\n",
    );
    expect_printed(
        b"for ((/* foo */ let).x of y) ;",
        b"for (\n  /* foo */\n  (let).x of y\n) ;\n",
    );
    expect_printed(
        b"for (/* foo */ (let).x of y) ;",
        b"for (\n  /* foo */\n  (let).x of y\n) ;\n",
    );
    expect_printed(
        b"function *x() { yield (/* foo */ y) }",
        b"function* x() {\n  yield (\n    /* foo */\n    y\n  );\n}\n",
    );
}

#[test]
fn test_pure_comment() {
    expect_printed(b"(function() { foo() })", b"(function() {\n  foo();\n});\n");
    expect_printed(
        b"(function() { foo() })()",
        b"(function() {\n  foo();\n})();\n",
    );
    expect_printed(
        b"/*@__PURE__*/(function() { foo() })()",
        b"/* @__PURE__ */ (function() {\n  foo();\n})();\n",
    );
    expect_printed(b"new (function() {})", b"new (function() {\n})();\n");
    expect_printed(b"new (function() {})()", b"new (function() {\n})();\n");
    expect_printed(
        b"/*@__PURE__*/new (function() {})()",
        b"/* @__PURE__ */ new (function() {\n})();\n",
    );
    expect_printed(
        b"export default (function() { foo() })",
        b"export default (function() {\n  foo();\n});\n",
    );
    expect_printed(
        b"export default (function() { foo() })()",
        b"export default (function() {\n  foo();\n})();\n",
    );
    expect_printed(
        b"export default /*@__PURE__*/(function() { foo() })()",
        b"export default /* @__PURE__ */ (function() {\n  foo();\n})();\n",
    );
}

#[test]
fn test_generator() {
    expect_printed(b"function* foo() {}", b"function* foo() {\n}\n");
    expect_printed(b"(function* () {})", b"(function* () {\n});\n");
    expect_printed(b"(function* foo() {})", b"(function* foo() {\n});\n");
    expect_printed(
        b"class Foo { *foo() {} }",
        b"class Foo {\n  *foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static *foo() {} }",
        b"class Foo {\n  static *foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { *[foo]() {} }",
        b"class Foo {\n  *[foo]() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static *[foo]() {} }",
        b"class Foo {\n  static *[foo]() {\n  }\n}\n",
    );
    expect_printed(
        b"(class { *foo() {} })",
        b"(class {\n  *foo() {\n  }\n});\n",
    );
    expect_printed(
        b"(class { static *foo() {} })",
        b"(class {\n  static *foo() {\n  }\n});\n",
    );
    expect_printed(
        b"(class { *[foo]() {} })",
        b"(class {\n  *[foo]() {\n  }\n});\n",
    );
    expect_printed(
        b"(class { static *[foo]() {} })",
        b"(class {\n  static *[foo]() {\n  }\n});\n",
    );
}

#[test]
fn test_arrow() {
    expect_printed(b"() => {}", b"() => {\n};\n");
    expect_printed(b"x => (x, 0)", b"(x) => (x, 0);\n");
    expect_printed(b"x => {y}", b"(x) => {\n  y;\n};\n");
    expect_printed(
        b"(a = (b, c), ...d) => {}",
        b"(a = (b, c), ...d) => {\n};\n",
    );
    expect_printed(
        b"({[1 + 2]: a = 3} = {[1 + 2]: 3}) => {}",
        b"({ [1 + 2]: a = 3 } = { [1 + 2]: 3 }) => {\n};\n",
    );
    expect_printed(
        b"([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) => {}",
        b"([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) => {\n};\n",
    );
    expect_printed(b"([] = []) => {}", b"([] = []) => {\n};\n");
    expect_printed(b"([,] = [,]) => {}", b"([,] = [,]) => {\n};\n");
    expect_printed(b"([,,] = [,,]) => {}", b"([, ,] = [, ,]) => {\n};\n");
    expect_printed(b"a = () => {}", b"a = () => {\n};\n");
    expect_printed(b"a || (() => {})", b"a || (() => {\n});\n");
    expect_printed(b"({a = b, c = d}) => {}", b"({ a = b, c = d }) => {\n};\n");
    expect_printed(
        b"([{a = b, c = d} = {}] = []) => {}",
        b"([{ a = b, c = d } = {}] = []) => {\n};\n",
    );
    expect_printed(
        b"({a: [b = c] = []} = {}) => {}",
        b"({ a: [b = c] = [] } = {}) => {\n};\n",
    );
    expect_printed(b"(a = b, c)", b"a = b, c;\n");
    expect_printed(b"([...a = b])", b"[...a = b];\n");
    expect_printed(b"([...a, ...b])", b"[...a, ...b];\n");
    expect_printed(b"({a: b, c() {}})", b"({ a: b, c() {\n} });\n");
    expect_printed(b"({a: b, get c() {}})", b"({ a: b, get c() {\n} });\n");
    expect_printed(b"({a: b, set c(x) {}})", b"({ a: b, set c(x) {\n} });\n");
}

#[test]
fn test_class() {
    expect_printed(
        b"class Foo extends (a, b) {}",
        b"class Foo extends (a, b) {\n}\n",
    );
    expect_printed(
        b"class Foo { get foo() {} }",
        b"class Foo {\n  get foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { set foo(x) {} }",
        b"class Foo {\n  set foo(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static foo() {} }",
        b"class Foo {\n  static foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static get foo() {} }",
        b"class Foo {\n  static get foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static set foo(x) {} }",
        b"class Foo {\n  static set foo(x) {\n  }\n}\n",
    );
}

#[test]
fn test_auto_accessors() {
    expect_printed(
        b"class Foo { accessor x; static accessor y }",
        b"class Foo {\n  accessor x;\n  static accessor y;\n}\n",
    );
    expect_printed(
        b"class Foo { accessor [x]; static accessor [y] }",
        b"class Foo {\n  accessor [x];\n  static accessor [y];\n}\n",
    );
    expect_printed_minify(
        b"class Foo { accessor x; static accessor y }",
        b"class Foo{accessor x;static accessor y}",
    );
    expect_printed_minify(
        b"class Foo { accessor [x]; static accessor [y] }",
        b"class Foo{accessor[x];static accessor[y]}",
    );
}

#[test]
fn test_private_identifiers() {
    expect_printed(
        b"class Foo { #foo; foo() { return #foo in this } }",
        b"class Foo {\n  #foo;\n  foo() {\n    return #foo in this;\n  }\n}\n",
    );
    expect_printed_minify(
        b"class Foo { #foo; foo() { return #foo in this } }",
        b"class Foo{#foo;foo(){return#foo in this}}",
    );
}

#[test]
fn test_decorators() {
    let example = b"class Foo {\n@w\nw; @x x; @a1\n@b1@b2\n@c1@c2@c3\ny = @y1 @y2 class {}; @a1\n@b1@b2\n@c1@c2@c3 z =\n@z1\n@z2\nclass {}}";
    expect_printed(example, b"class Foo {\n  @w\n  w;\n  @x x;\n  @a1\n  @b1 @b2\n  @c1 @c2 @c3\n  y = @y1 @y2 class {\n  };\n  @a1\n  @b1 @b2\n  @c1 @c2 @c3 z = @z1 @z2 class {\n  };\n}\n");
    expect_printed_minify(example, b"class Foo{@w w;@x x;@a1@b1@b2@c1@c2@c3 y=@y1@y2 class{};@a1@b1@b2@c1@c2@c3 z=@z1@z2 class{}}");
}

#[test]
fn test_import() {
    expect_printed(b"import('path');", b"import(\"path\");\n");
    expect_printed(
        b"import(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
        b"import(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\"\n);\n",
    );
    expect_printed(b"import(// webpackFoo: 1\n // webpackBar: 2\n 'path', {type: 'module'});", b"import(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\",\n  { type: \"module\" }\n);\n");
    expect_printed(
        b"import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
        b"import(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\"\n);\n",
    );
    expect_printed(b"import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path', {type: 'module'});", b"import(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\",\n  { type: \"module\" }\n);\n");
    expect_printed(
        b"import(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
        b"import(\n  /* multi\n   * line\n   * webpackBar: */\n  \"path\"\n);\n",
    );
    expect_printed(
        b"import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
        b"import(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    expect_printed(
        b"import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);",
        b"import(\n  /* webpackFoo: 1 */\n  \"path\"\n);\n",
    );
    expect_printed(
        b"import(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
        b"import(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    expect_printed(b"import(/* webpackFoo: 1 */ 'path', { type: 'module' } /* webpackBar:2 */ );", b"import(\n  /* webpackFoo: 1 */\n  \"path\",\n  { type: \"module\" }\n  /* webpackBar:2 */\n);\n");
    expect_printed(b"import(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))", b"import(new URL(\n  \"path\",\n  /* webpackFoo: these can go anywhere */\n  import.meta.url\n));\n");
    expect_printed_minify(
        b"import defer * as foo from 'bar'",
        b"import defer*as foo from\"bar\";",
    );
    expect_printed_minify(
        b"import source foo from 'bar'",
        b"import source foo from\"bar\";",
    );
}

#[test]
fn test_export_default() {
    expect_printed_ts(
        b"export default function(): void;",
        b"export default function(): void;\n",
    );
    expect_printed_ts(
        b"export default async function<T>(): Promise<T>;",
        b"export default async function<T>(): Promise<T>;\n",
    );
    expect_printed(
        b"export default function() {}",
        b"export default function() {\n}\n",
    );
    expect_printed(
        b"export default function foo() {}",
        b"export default function foo() {\n}\n",
    );
    expect_printed(
        b"export default async function() {}",
        b"export default async function() {\n}\n",
    );
    expect_printed(
        b"export default async function foo() {}",
        b"export default async function foo() {\n}\n",
    );
    expect_printed(b"export default class {}", b"export default class {\n}\n");
    expect_printed(
        b"export default class foo {}",
        b"export default class foo {\n}\n",
    );
    expect_printed(
        b"export default (function() {})",
        b"export default (function() {\n});\n",
    );
    expect_printed(
        b"export default (function foo() {})",
        b"export default (function foo() {\n});\n",
    );
    expect_printed(
        b"export default (async function() {})",
        b"export default (async function() {\n});\n",
    );
    expect_printed(
        b"export default (async function foo() {})",
        b"export default (async function foo() {\n});\n",
    );
    expect_printed(
        b"export default (class {})",
        b"export default (class {\n});\n",
    );
    expect_printed(
        b"export default (class foo {})",
        b"export default (class foo {\n});\n",
    );
    expect_printed(
        b"export default (function() {}.toString())",
        b"export default (function() {\n}).toString();\n",
    );
    expect_printed(
        b"export default (function foo() {}.toString())",
        b"export default (function foo() {\n}).toString();\n",
    );
    expect_printed(
        b"export default (async function() {}.toString())",
        b"export default (async function() {\n}).toString();\n",
    );
    expect_printed(
        b"export default (async function foo() {}.toString())",
        b"export default (async function foo() {\n}).toString();\n",
    );
    expect_printed(
        b"export default (class {}.toString())",
        b"export default (class {\n}).toString();\n",
    );
    expect_printed(
        b"export default (class foo {}.toString())",
        b"export default (class foo {\n}).toString();\n",
    );
    expect_printed_minify(
        b"export default function() {}",
        b"export default function(){}",
    );
    expect_printed_minify(
        b"export default function foo() {}",
        b"export default function foo(){}",
    );
    expect_printed_minify(
        b"export default async function() {}",
        b"export default async function(){}",
    );
    expect_printed_minify(
        b"export default async function foo() {}",
        b"export default async function foo(){}",
    );
    expect_printed_minify(b"export default class {}", b"export default class{}");
    expect_printed_minify(
        b"export default class foo {}",
        b"export default class foo{}",
    );
}

#[test]
fn test_whitespace() {
    expect_printed(b"- -x", b"- -x;\n");
    expect_printed(b"+ -x", b"+-x;\n");
    expect_printed(b"- +x", b"-+x;\n");
    expect_printed(b"+ +x", b"+ +x;\n");
    expect_printed(b"- --x", b"- --x;\n");
    expect_printed(b"+ --x", b"+--x;\n");
    expect_printed(b"- ++x", b"-++x;\n");
    expect_printed(b"+ ++x", b"+ ++x;\n");
    expect_printed_minify(b"- -x", b"- -x;");
    expect_printed_minify(b"+ -x", b"+-x;");
    expect_printed_minify(b"- +x", b"-+x;");
    expect_printed_minify(b"+ +x", b"+ +x;");
    expect_printed_minify(b"- --x", b"- --x;");
    expect_printed_minify(b"+ --x", b"+--x;");
    expect_printed_minify(b"- ++x", b"-++x;");
    expect_printed_minify(b"+ ++x", b"+ ++x;");
    expect_printed_minify(b"x - --y", b"x- --y;");
    expect_printed_minify(b"x + --y", b"x+--y;");
    expect_printed_minify(b"x - ++y", b"x-++y;");
    expect_printed_minify(b"x + ++y", b"x+ ++y;");
    expect_printed_minify(b"x-- > y", b"x-- >y;");
    expect_printed_minify(b"x < !--y", b"x<! --y;");
    expect_printed_minify(b"x > !--y", b"x>!--y;");
    expect_printed_minify(b"!--y", b"!--y;");
    expect_printed_minify(b"1 + -0", b"1+-0;");
    expect_printed_minify(b"1 - -0", b"1- -0;");
    expect_printed_minify(b"1 + -Infinity", b"1+-Infinity;");
    expect_printed_minify(b"1 - -Infinity", b"1- -Infinity;");
    expect_printed_minify(b"/x/ / /y/", b"/x// /y/;");
    expect_printed_minify(b"/x/ + Foo", b"/x/+Foo;");
    expect_printed_minify(b"/x/ instanceof Foo", b"/x/ instanceof Foo;");
    expect_printed_minify(b"[x] instanceof Foo", b"[x]instanceof Foo;");
    expect_printed_minify(b"throw x", b"throw x;");
    expect_printed_minify(b"throw typeof x", b"throw typeof x;");
    expect_printed_minify(b"throw delete x", b"throw delete x;");
    expect_printed_minify(b"throw function(){}", b"throw function(){};");
    expect_printed_minify(b"x in function(){}", b"x in function(){};");
    expect_printed_minify(b"x instanceof function(){}", b"x instanceof function(){};");
    expect_printed_minify(b"\xcf\x80 in function(){}", b"\xcf\x80 in function(){};");
    expect_printed_minify(
        b"\xcf\x80 instanceof function(){}",
        b"\xcf\x80 instanceof function(){};",
    );
    expect_printed_minify(b"()=>({})", b"()=>({});");
    expect_printed_minify(b"()=>({}[1])", b"()=>({})[1];");
    // This output depends on esbuild's parser-stage string addition folding.
    // expect_printed_minify(b"()=>({}+0)", b"()=>\"[object Object]0\";");
    expect_printed_minify(b"()=>function(){}", b"()=>function(){};");
    expect_printed_minify(b"(function(){})", b"(function(){});");
    expect_printed_minify(b"(class{})", b"(class{});");
    expect_printed_minify(b"({})", b"({});");
}

#[test]
fn test_minify() {
    expect_printed_minify(b"0.1", b".1;");
    expect_printed_minify(b"1.2", b"1.2;");
    expect_printed_minify(b"() => {}", b"()=>{};");
    expect_printed_minify(b"(a) => {}", b"a=>{};");
    expect_printed_minify(b"(...a) => {}", b"(...a)=>{};");
    expect_printed_minify(b"(a = 0) => {}", b"(a=0)=>{};");
    expect_printed_minify(b"(a, b) => {}", b"(a,b)=>{};");
    expect_printed(b"true ** 2", b"true ** 2;\n");
    expect_printed(b"false ** 2", b"false ** 2;\n");
    expect_printed_minify(b"true ** 2", b"true**2;");
    expect_printed_minify(b"false ** 2", b"false**2;");
    expect_printed_minify(b"import a from 'path'", b"import a from\"path\";");
    expect_printed_minify(b"import * as ns from 'path'", b"import*as ns from\"path\";");
    expect_printed_minify(
        b"import {a, b as c} from 'path'",
        b"import{a,b as c}from\"path\";",
    );
    expect_printed_minify(
        b"import {a, ' ' as c} from 'path'",
        b"import{a,\" \"as c}from\"path\";",
    );
    expect_printed_minify(b"export * as ns from 'path'", b"export*as ns from\"path\";");
    expect_printed_minify(
        b"export * as ' ' from 'path'",
        b"export*as\" \"from\"path\";",
    );
    expect_printed_minify(
        b"export {a, b as c} from 'path'",
        b"export{a,b as c}from\"path\";",
    );
    expect_printed_minify(
        b"export {' ', '-' as ';'} from 'path'",
        b"export{\" \",\"-\"as\";\"}from\"path\";",
    );
    expect_printed_minify(
        b"let a, b; export {a, b as c}",
        b"let a,b;export{a,b as c};",
    );
    expect_printed_minify(
        b"let a, b; export {a, b as ' '}",
        b"let a,b;export{a,b as\" \"};",
    );
    expect_printed(b"x = '\\n'", b"x = \"\\n\";\n");
    expect_printed_minify(b"exports", b"exports;");
    expect_printed_minify(b"require", b"require;");
    expect_printed_minify(b"module", b"module;");
    expect_printed_minify(b"//!single\nthrow 1 + 2", b"//!single\nthrow 1+2;");
    expect_printed_minify(
        b"/*!multi-\nline*/\nthrow 1 + 2",
        b"/*!multi-\nline*/throw 1+2;",
    );
}

#[test]
fn test_ascii_only() {
    expect_printed(
        b"let \xcf\x80 = '\xcf\x80'",
        b"let \xcf\x80 = \"\xcf\x80\";\n",
    );
    expect_printed(
        b"let \xcf\x80_ = '\xcf\x80'",
        b"let \xcf\x80_ = \"\xcf\x80\";\n",
    );
    expect_printed(
        b"let _\xcf\x80 = '\xcf\x80'",
        b"let _\xcf\x80 = \"\xcf\x80\";\n",
    );
    expect_printed_ascii(
        b"let \xcf\x80 = '\xcf\x80'",
        b"let \\u03C0 = \"\\u03C0\";\n",
    );
    expect_printed_ascii(
        b"let \xcf\x80_ = '\xcf\x80'",
        b"let \\u03C0_ = \"\\u03C0\";\n",
    );
    expect_printed_ascii(
        b"let _\xcf\x80 = '\xcf\x80'",
        b"let _\\u03C0 = \"\\u03C0\";\n",
    );
    expect_printed(
        b"let \xe8\xb2\x93 = '\xf0\x9f\x90\x88'",
        b"let \xe8\xb2\x93 = \"\xf0\x9f\x90\x88\";\n",
    );
    expect_printed(
        b"let \xe8\xb2\x93abc = '\xf0\x9f\x90\x88'",
        b"let \xe8\xb2\x93abc = \"\xf0\x9f\x90\x88\";\n",
    );
    expect_printed(
        b"let abc\xe8\xb2\x93 = '\xf0\x9f\x90\x88'",
        b"let abc\xe8\xb2\x93 = \"\xf0\x9f\x90\x88\";\n",
    );
    expect_printed_ascii(
        b"let \xe8\xb2\x93 = '\xf0\x9f\x90\x88'",
        b"let \\u8C93 = \"\\u{1F408}\";\n",
    );
    expect_printed_ascii(
        b"let \xe8\xb2\x93abc = '\xf0\x9f\x90\x88'",
        b"let \\u8C93abc = \"\\u{1F408}\";\n",
    );
    expect_printed_ascii(
        b"let abc\xe8\xb2\x93 = '\xf0\x9f\x90\x88'",
        b"let abc\\u8C93 = \"\\u{1F408}\";\n",
    );
    expect_printed(b"var \xf0\x90\x80\x80", b"var \xf0\x90\x80\x80;\n");
    expect_printed(b"var \\u{10000}", b"var \xf0\x90\x80\x80;\n");
    expect_printed_ascii(b"var \xf0\x90\x80\x80", b"var \\u{10000};\n");
    expect_printed_ascii(b"var \\u{10000}", b"var \\u{10000};\n");
    expect_printed_ascii(
        b"var \\u{100a} = {\\u100A: '\\u100A'}",
        b"var \\u100A = { \\u100A: \"\\u100A\" };\n",
    );
    expect_printed_ascii(
        b"var \\u{1000a} = {\\u{1000A}: '\\u{1000A}'}",
        b"var \\u{1000A} = { \"\\u{1000A}\": \"\\u{1000A}\" };\n",
    );
    expect_printed(b"let x = '\xe2\x80\xa8'", b"let x = \"\\u2028\";\n");
    expect_printed(b"let x = '\xe2\x80\xa9'", b"let x = \"\\u2029\";\n");
    expect_printed(b"let x = '\xef\xbb\xbf'", b"let x = \"\\uFEFF\";\n");
    expect_printed_ascii(
        b"class \xf0\x90\x80\x80 extends \xcf\x80 {}",
        b"class \\u{10000} extends \\u03C0 {\n}\n",
    );
    expect_printed_ascii(
        b"(class \xf0\x90\x80\x80 extends \xcf\x80 {})",
        b"(class \\u{10000} extends \\u03C0 {\n});\n",
    );
    expect_printed_minify_ascii(
        b"class \xf0\x90\x80\x80 extends \xcf\x80 {}",
        b"class \\u{10000} extends \\u03C0{}",
    );
    expect_printed_minify_ascii(
        b"(class \xf0\x90\x80\x80 extends \xcf\x80 {})",
        b"(class \\u{10000} extends \\u03C0{});",
    );
}

#[test]
fn test_jsx() {
    expect_printed_jsx(b"<a/>", b"<a />;\n");
    expect_printed_jsx(b"<A/>", b"<A />;\n");
    expect_printed_jsx(b"<a.b/>", b"<a.b />;\n");
    expect_printed_jsx(b"<A.B/>", b"<A.B />;\n");
    expect_printed_jsx(b"<a-b/>", b"<a-b />;\n");
    expect_printed_jsx(b"<a:b/>", b"<a:b />;\n");
    expect_printed_jsx(b"<a></a>", b"<a />;\n");
    expect_printed_jsx(b"<a b></a>", b"<a b />;\n");
    expect_printed_jsx(b"<a b={true}></a>", b"<a b={true} />;\n");
    expect_printed_jsx(b"<a b='x'></a>", b"<a b='x' />;\n");
    expect_printed_jsx(b"<a b=\"x\"></a>", b"<a b=\"x\" />;\n");
    expect_printed_jsx(b"<a b={'x'}></a>", b"<a b={\"x\"} />;\n");
    expect_printed_jsx(b"<a b={`'`}></a>", b"<a b={`'`} />;\n");
    expect_printed_jsx(b"<a b={`\"`}></a>", b"<a b={`\"`} />;\n");
    expect_printed_jsx(b"<a b={`'\"`}></a>", b"<a b={`'\"`} />;\n");
    expect_printed_jsx(b"<a b=\"&quot;\"></a>", b"<a b=\"&quot;\" />;\n");
    expect_printed_jsx(b"<a b=\"&amp;\"></a>", b"<a b=\"&amp;\" />;\n");
    expect_printed_jsx(b"<a>x</a>", b"<a>x</a>;\n");
    expect_printed_jsx(b"<a>x\ny</a>", b"<a>x\ny</a>;\n");
    expect_printed_jsx(b"<a>{'x'}{'y'}</a>", b"<a>{\"x\"}{\"y\"}</a>;\n");
    expect_printed_jsx(b"<a> x</a>", b"<a> x</a>;\n");
    expect_printed_jsx(b"<a>x </a>", b"<a>x </a>;\n");
    expect_printed_jsx(b"<a>&#10;</a>", b"<a>&#10;</a>;\n");
    expect_printed_jsx(b"<a>&amp;</a>", b"<a>&amp;</a>;\n");
    expect_printed_jsx(b"<a>&lt;</a>", b"<a>&lt;</a>;\n");
    expect_printed_jsx(b"<a>&gt;</a>", b"<a>&gt;</a>;\n");
    expect_printed_jsx(b"<a>&#123;</a>", b"<a>&#123;</a>;\n");
    expect_printed_jsx(b"<a>&#125;</a>", b"<a>&#125;</a>;\n");
    expect_printed_jsx(b"<a><x/></a>", b"<a><x /></a>;\n");
    expect_printed_jsx(b"<a><x/><y/></a>", b"<a><x /><y /></a>;\n");
    expect_printed_jsx(b"<a>b<c/>d</a>", b"<a>b<c />d</a>;\n");
    expect_printed_jsx(b"<></>", b"<></>;\n");
    expect_printed_jsx(b"<>x<y/>z</>", b"<>x<y />z</>;\n");
    expect_printed_jsx(b"<a b=<c/>/>", b"<a b=<c /> />;\n");
    expect_printed_jsx(b"<a b=<>c</>/>", b"<a b=<>c</> />;\n");
    expect_printed_jsx(b"<a b=<>{c}</>/>", b"<a b=<>{c}</> />;\n");
    expect_printed_jsx(b"<a b={<c/>}/>", b"<a b={<c />} />;\n");
    expect_printed_jsx(b"<a b={<>c</>}/>", b"<a b={<>c</>} />;\n");
    expect_printed_jsx(b"<a b={<>{c}</>}/>", b"<a b={<>{c}</>} />;\n");
    expect_printed_jsx_ascii(b"<\xcf\x80/>", b"<\xcf\x80 />;\n");
    expect_printed_jsx_ascii(
        b"<\xcf\x80.\xf0\x90\x80\x80/>",
        b"<\xcf\x80.\xf0\x90\x80\x80 />;\n",
    );
    expect_printed_jsx_ascii(
        b"<\xf0\x90\x80\x80.\xcf\x80/>",
        b"<\xf0\x90\x80\x80.\xcf\x80 />;\n",
    );
    expect_printed_jsx_ascii(b"<\xcf\x80>x</\xcf\x80>", b"<\xcf\x80>x</\xcf\x80>;\n");
    expect_printed_jsx_ascii(
        b"<\xf0\x90\x80\x80>x</\xf0\x90\x80\x80>",
        b"<\xf0\x90\x80\x80>x</\xf0\x90\x80\x80>;\n",
    );
    expect_printed_jsx_ascii(b"<a \xcf\x80/>", b"<a \xcf\x80 />;\n");
    expect_printed_jsx_ascii(b"<a \xf0\x90\x80\x80/>", b"<a \xf0\x90\x80\x80 />;\n");
    expect_printed_jsx_ascii(b"<a b='\xcf\x80'/>", b"<a b='\xcf\x80' />;\n");
    expect_printed_jsx_ascii(
        b"<a b='\xf0\x90\x80\x80'/>",
        b"<a b='\xf0\x90\x80\x80' />;\n",
    );
    expect_printed_jsx_ascii(b"<a>\xcf\x80</a>", b"<a>\xcf\x80</a>;\n");
    expect_printed_jsx_ascii(b"<a>\xf0\x90\x80\x80</a>", b"<a>\xf0\x90\x80\x80</a>;\n");
    expect_printed_jsx_minify(b"<a b c={x,y} d='true'/>", b"<a b c={(x,y)}d='true'/>;");
    expect_printed_jsx_minify(b"<a><b/><c/></a>", b"<a><b/><c/></a>;");
    expect_printed_jsx_minify(b"<a> x <b/> y </a>", b"<a> x <b/> y </a>;");
    expect_printed_jsx_minify(
        b"<a>{' x '}{'<b/>'}{' y '}</a>",
        b"<a>{\" x \"}{\"<b/>\"}{\" y \"}</a>;",
    );
}

#[test]
fn test_jsx_single_line() {
    expect_printed_jsx(b"<x/>", b"<x />;\n");
    expect_printed_jsx(b"<x y/>", b"<x y />;\n");
    expect_printed_jsx(b"<x\n/>", b"<x />;\n");
    expect_printed_jsx(b"<x\ny/>", b"<x\n  y\n/>;\n");
    expect_printed_jsx(b"<x y\n/>", b"<x\n  y\n/>;\n");
    expect_printed_jsx(b"<x\n{...y}/>", b"<x\n  {...y}\n/>;\n");
    expect_printed_jsx_minify(b"<x/>", b"<x/>;");
    expect_printed_jsx_minify(b"<x y/>", b"<x y/>;");
    expect_printed_jsx_minify(b"<x\n/>", b"<x/>;");
    expect_printed_jsx_minify(b"<x\ny/>", b"<x y/>;");
    expect_printed_jsx_minify(b"<x y\n/>", b"<x y/>;");
    expect_printed_jsx_minify(b"<x\n{...y}/>", b"<x{...y}/>;");
}

#[test]
fn test_avoid_slash_script() {
    expect_printed(b"x = '</script'", b"x = \"<\\/script\";\n");
    expect_printed(b"x = `</script`", b"x = `<\\/script`;\n");
    expect_printed(b"x = `</SCRIPT`", b"x = `<\\/SCRIPT`;\n");
    expect_printed(b"x = `</ScRiPt`", b"x = `<\\/ScRiPt`;\n");
    expect_printed(b"x = `</script${y}`", b"x = `<\\/script${y}`;\n");
    expect_printed(b"x = `${y}</script`", b"x = `${y}<\\/script`;\n");
    expect_printed_minify(
        b"x = 1 < /script/.exec(y).length",
        b"x=1< /script/.exec(y).length;",
    );
    expect_printed_minify(
        b"x = 1 < /SCRIPT/.exec(y).length",
        b"x=1< /SCRIPT/.exec(y).length;",
    );
    expect_printed_minify(
        b"x = 1 < /ScRiPt/.exec(y).length",
        b"x=1< /ScRiPt/.exec(y).length;",
    );
    expect_printed_minify(
        b"x = 1 << /script/.exec(y).length",
        b"x=1<< /script/.exec(y).length;",
    );
    expect_printed(
        b"//! </script\n//! >/script\n//! /script",
        b"//! <\\/script\n//! >/script\n//! /script\n",
    );
    expect_printed(
        b"//! </SCRIPT\n//! >/SCRIPT\n//! /SCRIPT",
        b"//! <\\/SCRIPT\n//! >/SCRIPT\n//! /SCRIPT\n",
    );
    expect_printed(
        b"//! </ScRiPt\n//! >/ScRiPt\n//! /ScRiPt",
        b"//! <\\/ScRiPt\n//! >/ScRiPt\n//! /ScRiPt\n",
    );
    expect_printed(
        b"/*! </script \n </script */",
        b"/*! <\\/script \n <\\/script */\n",
    );
    expect_printed(
        b"/*! </SCRIPT \n </SCRIPT */",
        b"/*! <\\/SCRIPT \n <\\/SCRIPT */\n",
    );
    expect_printed(
        b"/*! </ScRiPt \n </ScRiPt */",
        b"/*! <\\/ScRiPt \n <\\/ScRiPt */\n",
    );
    expect_printed(b"String.raw`</script`", b"import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/script\"])));\n");
    expect_printed(b"String.raw`</script${a}`", b"import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/script\", \"\"])), a);\n");
    expect_printed(b"String.raw`${a}</script`", b"import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"\", \"<\\/script\"])), a);\n");
    expect_printed(b"String.raw`</SCRIPT`", b"import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/SCRIPT\"])));\n");
    expect_printed(b"String.raw`</ScRiPt`", b"import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/ScRiPt\"])));\n");
    expect_printed(b"x = '</'", b"x = \"</\";\n");
    expect_printed(b"x = '</ script'", b"x = \"</ script\";\n");
    expect_printed(b"x = '< /script'", b"x = \"< /script\";\n");
    expect_printed(b"x = '/script>'", b"x = \"/script>\";\n");
    expect_printed(b"x = '<script>'", b"x = \"<script>\";\n");
    expect_printed_minify(
        b"x = 1 < / script/.exec(y).length",
        b"x=1</ script/.exec(y).length;",
    );
    expect_printed_minify(
        b"x = 1 << / script/.exec(y).length",
        b"x=1<</ script/.exec(y).length;",
    );
}

#[test]
fn test_infinity() {
    expect_printed(b"x = Infinity", b"x = Infinity;\n");
    expect_printed(b"x = -Infinity", b"x = -Infinity;\n");
    expect_printed(b"x = (Infinity).toString", b"x = Infinity.toString;\n");
    expect_printed(b"x = (-Infinity).toString", b"x = (-Infinity).toString;\n");
    expect_printed(b"x = (Infinity) ** 2", b"x = Infinity ** 2;\n");
    expect_printed(b"x = (-Infinity) ** 2", b"x = (-Infinity) ** 2;\n");
    expect_printed(b"x = ~Infinity", b"x = ~Infinity;\n");
    expect_printed(b"x = ~-Infinity", b"x = ~-Infinity;\n");
    expect_printed(b"x = Infinity * y", b"x = Infinity * y;\n");
    expect_printed(b"x = Infinity / y", b"x = Infinity / y;\n");
    expect_printed(b"x = y * Infinity", b"x = y * Infinity;\n");
    expect_printed(b"x = y / Infinity", b"x = y / Infinity;\n");
    expect_printed(b"throw Infinity", b"throw Infinity;\n");
    expect_printed_minify(b"x = Infinity", b"x=Infinity;");
    expect_printed_minify(b"x = -Infinity", b"x=-Infinity;");
    expect_printed_minify(b"x = (Infinity).toString", b"x=Infinity.toString;");
    expect_printed_minify(b"x = (-Infinity).toString", b"x=(-Infinity).toString;");
    expect_printed_minify(b"x = (Infinity) ** 2", b"x=Infinity**2;");
    expect_printed_minify(b"x = (-Infinity) ** 2", b"x=(-Infinity)**2;");
    expect_printed_minify(b"x = ~Infinity", b"x=~Infinity;");
    expect_printed_minify(b"x = ~-Infinity", b"x=~-Infinity;");
    expect_printed_minify(b"x = Infinity * y", b"x=Infinity*y;");
    expect_printed_minify(b"x = Infinity / y", b"x=Infinity/y;");
    expect_printed_minify(b"x = y * Infinity", b"x=y*Infinity;");
    expect_printed_minify(b"x = y / Infinity", b"x=y/Infinity;");
    expect_printed_minify(b"throw Infinity", b"throw Infinity;");
}

#[test]
fn test_binary_operator_visitor() {
    let source = format!("x = f(){};\n", " || f()".repeat(10_000));
    expect_printed(source.as_bytes(), source.as_bytes());
}

#[test]
fn test_using() {
    expect_printed(b"using x = y", b"using x = y;\n");
    expect_printed(b"using x = y, z = _", b"using x = y, z = _;\n");
    expect_printed_minify(b"using x = y", b"using x=y;");
    expect_printed_minify(b"using x = y, z = _", b"using x=y,z=_;");
    expect_printed(b"await using x = y", b"await using x = y;\n");
    expect_printed(b"await using x = y, z = _", b"await using x = y, z = _;\n");
    expect_printed_minify(b"await using x = y", b"await using x=y;");
    expect_printed_minify(b"await using x = y, z = _", b"await using x=y,z=_;");
}

#[test]
fn test_control_flow_minify_and_with() {
    expect_printed_minify(
        b"if (x) y(); else z(); for (;;) x(); while(x)y(); do x(); while(y); switch(x){case 1:y();default:z()} try{x()}catch(e){y()}finally{z()} with(x)1e400",
        b"if(x)y();else z();for(;;)x();while(x)y();do x();while(y);switch(x){case 1:y();default:z()}try{x()}catch(e){y()}finally{z()}with(x)1/0;",
    );
    expect_printed(b"with (x) 1e400", b"with (x) 1 / 0;\n");
    expect_printed_minify(
        b"switch(x){case\"a\":break;case{}:break} throw{}",
        b"switch(x){case\"a\":break;case{}:break}throw{};",
    );
    expect_printed_minify(
        b"let {a}=x; function* foo(){}; throw function(){}",
        b"let{a}=x;function*foo(){};throw function(){};",
    );
}
