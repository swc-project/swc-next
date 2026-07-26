// Ported from upstream esbuild's internal/js_parser/js_parser_test.go.
use crate::{expect_printed, expect_printed_jsx, expect_printed_script};

#[test]
fn test_un_op() {
    // Depends on esbuild's parser-stage unused-expression folding.
    // expect_printed(b"let x; void 0; x", b"let x;\nx;\n");
    expect_printed(b"let x; void x; x", b"let x;\nvoid x;\nx;\n");
}

#[test]
fn test_bin_op() {
    const LEFT_ASSOCIATIVE: &[&str] = &[
        "+",
        "-",
        "*",
        "/",
        "%",
        "<",
        "<=",
        ">",
        ">=",
        "in",
        "instanceof",
        "<<",
        ">>",
        ">>>",
        "==",
        "!=",
        "===",
        "!==",
        "??",
        "||",
        "&&",
        "|",
        "&",
        "^",
    ];
    for op in LEFT_ASSOCIATIVE {
        let source = format!("a {op} b {op} c");
        let expected = format!("a {op} b {op} c;\n");
        expect_printed(source.as_bytes(), expected.as_bytes());
        let source = format!("(a {op} b) {op} c");
        expect_printed(source.as_bytes(), expected.as_bytes());
        let source = format!("a {op} (b {op} c)");
        let expected = format!("a {op} (b {op} c);\n");
        expect_printed(source.as_bytes(), expected.as_bytes());
    }
    expect_printed(b"a ** b ** c", b"a ** b ** c;\n");
    expect_printed(b"(a ** b) ** c", b"(a ** b) ** c;\n");
    expect_printed(b"a ** (b ** c)", b"a ** b ** c;\n");
    const ASSIGNMENTS: &[&str] = &[
        "=", "+=", "-=", "*=", "/=", "%=", "**=", "<<=", ">>=", ">>>=", "|=", "&=", "^=", "??=",
        "||=", "&&=",
    ];
    for op in ASSIGNMENTS {
        let source = format!("a {op} b {op} c");
        let expected = format!("a {op} b {op} c;\n");
        expect_printed(source.as_bytes(), expected.as_bytes());
        let source = format!("a {op} (b {op} c)");
        expect_printed(source.as_bytes(), expected.as_bytes());
    }
}

#[test]
#[ignore = "depends on esbuild comment metadata"]
fn test_comments() {
    expect_printed(b"return //\n x", b"return;\nx;\n");
    expect_printed(b"return /**/\n x", b"return;\nx;\n");
    expect_printed_script(b"return <!--\n x", b"return;\nx;\n");
    expect_printed_script(b"-->\nx", b"x;\n");
    expect_printed_script(b"x\n-->\ny", b"x;\ny;\n");
    expect_printed_script(b"x\n -->\ny", b"x;\ny;\n");
    expect_printed_script(b"x\n/**/-->\ny", b"x;\ny;\n");
    expect_printed_script(b"x/*\n*/-->\ny", b"x;\ny;\n");
    expect_printed_script(b"x\n/**/ /**/-->\ny", b"x;\ny;\n");
    expect_printed(b"if(x-->y)z", b"if (x-- > y) z;\n");
}

#[test]
#[ignore = "depends on esbuild directive and comment metadata"]
fn test_strict_mode() {
    expect_printed(b"'use strict'", b"\"use strict\";\n");
    expect_printed(b"`use strict`", b"`use strict`;\n");
    expect_printed(
        b"//! @legal comment\n 'use strict'",
        b"\"use strict\";\n//! @legal comment\n",
    );
    expect_printed(
        b"/*! @legal comment */ 'use strict'",
        b"\"use strict\";\n/*! @legal comment */\n",
    );
    expect_printed(
        b"function f() { //! @legal comment\n 'use strict' }",
        b"function f() {\n  //! @legal comment\n  \"use strict\";\n}\n",
    );
    expect_printed(
        b"function f() { /*! @legal comment */ 'use strict' }",
        b"function f() {\n  /*! @legal comment */\n  \"use strict\";\n}\n",
    );
    expect_printed(b"let x = '\\0'", b"let x = \"\\0\";\n");
    expect_printed(b"let x = '\\00'", b"let x = \"\\0\";\n");
    expect_printed(
        b"'use strict'; let x = '\\0'",
        b"\"use strict\";\nlet x = \"\\0\";\n",
    );
    expect_printed(
        b"let x = '\\0'; export {}",
        b"let x = \"\\0\";\nexport {};\n",
    );
    expect_printed(b"'\\0'", b"\"\\0\";\n");
    expect_printed(b"'\\00'", b"\"\\0\";\n");
    expect_printed(b"'use strict'; '\\0'", b"\"use strict\";\n\"\\0\";\n");
    expect_printed(b"with (x) y", b"with (x) y;\n");
    expect_printed(b"delete x", b"delete x;\n");
    expect_printed(b"for (var x = y in z) ;", b"x = y;\nfor (var x in z) ;\n");
    expect_printed(b"function f(a, a) {}", b"function f(a, a) {\n}\n");
    expect_printed(b"(function(a, a) {})", b"(function(a, a) {\n});\n");
    expect_printed(
        b"({ f: function(a, a) {} })",
        b"({ f: function(a, a) {\n} });\n",
    );
    expect_printed(
        b"({ f: function*(a, a) {} })",
        b"({ f: function* (a, a) {\n} });\n",
    );
    expect_printed(
        b"({ f: async function(a, a) {} })",
        b"({ f: async function(a, a) {\n} });\n",
    );
    expect_printed(b"eval++", b"eval++;\n");
    expect_printed(b"eval = 0", b"eval = 0;\n");
    expect_printed(b"eval += 0", b"eval += 0;\n");
    expect_printed(b"[eval] = 0", b"[eval] = 0;\n");
    expect_printed(b"arguments++", b"arguments++;\n");
    expect_printed(b"arguments = 0", b"arguments = 0;\n");
    expect_printed(b"arguments += 0", b"arguments += 0;\n");
    expect_printed(b"[arguments] = 0", b"[arguments] = 0;\n");
    expect_printed(b"function eval() {}", b"function eval() {\n}\n");
    expect_printed(b"function arguments() {}", b"function arguments() {\n}\n");
    expect_printed(b"function f(eval) {}", b"function f(eval) {\n}\n");
    expect_printed(b"function f(arguments) {}", b"function f(arguments) {\n}\n");
    expect_printed(b"({ f(eval) {} })", b"({ f(eval) {\n} });\n");
    expect_printed(b"({ f(arguments) {} })", b"({ f(arguments) {\n} });\n");
    expect_printed(b"let protected", b"let protected;\n");
    expect_printed(b"let protecte\\u0064", b"let protected;\n");
    expect_printed(b"let x = protected", b"let x = protected;\n");
    expect_printed(b"let x = protecte\\u0064", b"let x = protected;\n");
    expect_printed(b"0123", b"83;\n");
    expect_printed(b"({0123: 4})", b"({ 83: 4 });\n");
    expect_printed(b"let {0123: x} = y", b"let { 83: x } = y;\n");
    expect_printed(
        b"function f() { 'use strict' } with (x) y",
        b"function f() {\n  \"use strict\";\n}\nwith (x) y;\n",
    );
    expect_printed(
        b"with (x) y; function f() { 'use strict' }",
        b"with (x) y;\nfunction f() {\n  \"use strict\";\n}\n",
    );
    expect_printed(b"class f {} with (x) y", b"class f {\n}\nwith (x) y;\n");
    expect_printed(b"with (x) y; class f {}", b"with (x) y;\nclass f {\n}\n");
    expect_printed(b"`use strict`; with (x) y", b"`use strict`;\nwith (x) y;\n");
    expect_printed(
        b"{ 'use strict'; with (x) y }",
        b"{\n  \"use strict\";\n  with (x) y;\n}\n",
    );
    expect_printed(
        b"if (0) { 'use strict'; with (x) y }",
        b"if (0) {\n  \"use strict\";\n  with (x) y;\n}\n",
    );
    expect_printed(
        b"while (0) { 'use strict'; with (x) y }",
        b"while (0) {\n  \"use strict\";\n  with (x) y;\n}\n",
    );
    expect_printed(
        b"try { 'use strict'; with (x) y } catch {}",
        b"try {\n  \"use strict\";\n  with (x) y;\n} catch {\n}\n",
    );
    expect_printed(
        b"try {} catch { 'use strict'; with (x) y }",
        b"try {\n} catch {\n  \"use strict\";\n  with (x) y;\n}\n",
    );
    expect_printed(
        b"try {} finally { 'use strict'; with (x) y }",
        b"try {\n} finally {\n  \"use strict\";\n  with (x) y;\n}\n",
    );
    expect_printed(b"import(x); with (y) z", b"import(x);\nwith (y) z;\n");
    expect_printed(b"import('x'); with (y) z", b"import(\"x\");\nwith (y) z;\n");
    expect_printed(b"with (y) z; import(x)", b"with (y) z;\nimport(x);\n");
    expect_printed(b"with (y) z; import('x')", b"with (y) z;\nimport(\"x\");\n");
    expect_printed(b"(import(x)); with (y) z", b"import(x);\nwith (y) z;\n");
    expect_printed(
        b"(import('x')); with (y) z",
        b"import(\"x\");\nwith (y) z;\n",
    );
    expect_printed(b"with (y) z; (import(x))", b"with (y) z;\nimport(x);\n");
    expect_printed(
        b"with (y) z; (import('x'))",
        b"with (y) z;\nimport(\"x\");\n",
    );
}

#[test]
fn test_exponentiation() {
    expect_printed(b"--x ** 2", b"--x ** 2;\n");
    expect_printed(b"++x ** 2", b"++x ** 2;\n");
    expect_printed(b"x-- ** 2", b"x-- ** 2;\n");
    expect_printed(b"x++ ** 2", b"x++ ** 2;\n");
    expect_printed(b"(-x) ** 2", b"(-x) ** 2;\n");
    expect_printed(b"(+x) ** 2", b"(+x) ** 2;\n");
    expect_printed(b"(~x) ** 2", b"(~x) ** 2;\n");
    expect_printed(b"(!x) ** 2", b"(!x) ** 2;\n");
    expect_printed(b"(-1) ** 2", b"(-1) ** 2;\n");
    // Depends on esbuild's parser-stage constant folding.
    // expect_printed(b"(+1) ** 2", b"1 ** 2;\n");
    expect_printed(b"(~1) ** 2", b"(~1) ** 2;\n");
    // Depends on esbuild's parser-stage constant folding.
    // expect_printed(b"(!1) ** 2", b"false ** 2;\n");
    expect_printed(b"(void x) ** 2", b"(void x) ** 2;\n");
    expect_printed(b"(delete x) ** 2", b"(delete x) ** 2;\n");
    expect_printed(b"(typeof x) ** 2", b"(typeof x) ** 2;\n");
    // Depends on esbuild's parser-stage undefined normalization.
    // expect_printed(b"undefined ** 2", b"(void 0) ** 2;\n");
    expect_printed(b"(delete x) ** 0", b"(delete x) ** 0;\n");
    expect_printed(b"(delete x.prop) ** 0", b"(delete x.prop) ** 0;\n");
    expect_printed(b"(delete x[0]) ** 0", b"(delete x[0]) ** 0;\n");
    expect_printed(b"(delete x?.prop) ** 0", b"(delete x?.prop) ** 0;\n");
    expect_printed(b"(void x) ** 0", b"(void x) ** 0;\n");
    expect_printed(b"(typeof x) ** 0", b"(typeof x) ** 0;\n");
    expect_printed(b"(+x) ** 0", b"(+x) ** 0;\n");
    expect_printed(b"(-x) ** 0", b"(-x) ** 0;\n");
    expect_printed(b"(~x) ** 0", b"(~x) ** 0;\n");
    expect_printed(b"(!x) ** 0", b"(!x) ** 0;\n");
    expect_printed(b"(await x) ** 0", b"(await x) ** 0;\n");
    expect_printed(b"(await -x) ** 0", b"(await -x) ** 0;\n");
}

#[test]
fn test_await() {
    expect_printed(b"await x", b"await x;\n");
    expect_printed(b"await +x", b"await +x;\n");
    expect_printed(b"await -x", b"await -x;\n");
    expect_printed(b"await ~x", b"await ~x;\n");
    expect_printed(b"await !x", b"await !x;\n");
    expect_printed(b"await --x", b"await --x;\n");
    expect_printed(b"await ++x", b"await ++x;\n");
    expect_printed(b"await x--", b"await x--;\n");
    expect_printed(b"await x++", b"await x++;\n");
    expect_printed(b"await void x", b"await void x;\n");
    expect_printed(b"await typeof x", b"await typeof x;\n");
    expect_printed(b"await (x * y)", b"await (x * y);\n");
    expect_printed(b"await (x ** y)", b"await (x ** y);\n");
    expect_printed(
        b"async function f() { await delete x }",
        b"async function f() {\n  await delete x;\n}\n",
    );
}

#[test]
fn test_reg_exp() {
    expect_printed(b"/x/d", b"/x/d;\n");
    expect_printed(b"/x/g", b"/x/g;\n");
    expect_printed(b"/x/i", b"/x/i;\n");
    expect_printed(b"/x/m", b"/x/m;\n");
    expect_printed(b"/x/s", b"/x/s;\n");
    expect_printed(b"/x/u", b"/x/u;\n");
    expect_printed(b"/x/y", b"/x/y;\n");
    expect_printed(b"/[\\])]/", b"/[\\])]/;\n");
}

#[test]
fn test_unicode_identifier_names() {
    expect_printed(
        b"x = {x\xe3\x83\xbb: 0}",
        b"x = { \"x\xe3\x83\xbb\": 0 };\n",
    );
    expect_printed(
        b"x = {x\xef\xbd\xa5: 0}",
        b"x = { \"x\xef\xbd\xa5\": 0 };\n",
    );
    expect_printed(b"x = {x\xcf\x80: 0}", b"x = { x\xcf\x80: 0 };\n");
    expect_printed(b"x = y.x\xe3\x83\xbb", b"x = y[\"x\xe3\x83\xbb\"];\n");
    expect_printed(b"x = y.x\xef\xbd\xa5", b"x = y[\"x\xef\xbd\xa5\"];\n");
    expect_printed(b"x = y.x\xcf\x80", b"x = y.x\xcf\x80;\n");
}

#[test]
fn test_identifier_escapes() {
    expect_printed(b"var _\\u0076\\u0061\\u0072", b"var _var;\n");
    expect_printed(b"foo._\\u0076\\u0061\\u0072", b"foo._var;\n");
    expect_printed(b"foo.\\u0076\\u0061\\u0072", b"foo.var;\n");
}

#[test]
fn test_special_identifiers() {
    expect_printed(b"exports", b"exports;\n");
    expect_printed(b"require", b"require;\n");
    expect_printed(b"module", b"module;\n");
}

#[test]
fn test_decls() {
    expect_printed(b"var {[x]: y} = {}", b"var { [x]: y } = {};\n");
    expect_printed(b"var {...x} = {}", b"var { ...x } = {};\n");
    expect_printed(b"var [...x] = []", b"var [...x] = [];\n");
    expect_printed(b"var {...x} = {}", b"var { ...x } = {};\n");
    expect_printed(b"([...x] = []) => {}", b"([...x] = []) => {\n};\n");
    expect_printed(b"({...x} = {}) => {}", b"({ ...x } = {}) => {\n};\n");
    expect_printed(b"[b, ...c] = d", b"[b, ...c] = d;\n");
    expect_printed(b"([b, ...c] = d)", b"[b, ...c] = d;\n");
    expect_printed(b"({b, ...c} = d)", b"({ b, ...c } = d);\n");
    expect_printed(b"({a = b} = c)", b"({ a = b } = c);\n");
    expect_printed(b"({a: b = c} = d)", b"({ a: b = c } = d);\n");
    expect_printed(b"({a: b.c} = d)", b"({ a: b.c } = d);\n");
    expect_printed(b"[a = {}] = b", b"[a = {}] = b;\n");
    expect_printed(b"[[...a, b].x] = c", b"[[...a, b].x] = c;\n");
    expect_printed(b"[{...a, b}.x] = c", b"[{ ...a, b }.x] = c;\n");
    expect_printed(b"({x: [...a, b].x} = c)", b"({ x: [...a, b].x } = c);\n");
    expect_printed(b"({x: {...a, b}.x} = c)", b"({ x: { ...a, b }.x } = c);\n");
    expect_printed(b"[x = [...a, b]] = c", b"[x = [...a, b]] = c;\n");
    expect_printed(b"[x = {...a, b}] = c", b"[x = { ...a, b }] = c;\n");
    expect_printed(b"({x = [...a, b]} = c)", b"({ x = [...a, b] } = c);\n");
    expect_printed(b"({x = {...a, b}} = c)", b"({ x = { ...a, b } } = c);\n");
    expect_printed(b"(x = y)", b"x = y;\n");
    expect_printed(b"([] = [])", b"[] = [];\n");
    expect_printed(b"({} = {})", b"({} = {});\n");
    expect_printed(b"([[]] = [[]])", b"[[]] = [[]];\n");
    expect_printed(b"({x: {}} = {x: {}})", b"({ x: {} } = { x: {} });\n");
    expect_printed(b"(x) = y", b"x = y;\n");
    expect_printed(b"for (x in y) ;", b"for (x in y) ;\n");
    expect_printed(b"for ([] in y) ;", b"for ([] in y) ;\n");
    expect_printed(b"for ({} in y) ;", b"for ({} in y) ;\n");
    expect_printed(b"for ((x) in y) ;", b"for (x in y) ;\n");
    expect_printed(b"for (x of y) ;", b"for (x of y) ;\n");
    expect_printed(b"for ([] of y) ;", b"for ([] of y) ;\n");
    expect_printed(b"for ({} of y) ;", b"for ({} of y) ;\n");
    expect_printed(b"for ((x) of y) ;", b"for (x of y) ;\n");
    expect_printed(
        b"for ([{a = {}}] in b) {}",
        b"for ([{ a = {} }] in b) {\n}\n",
    );
    expect_printed(
        b"for ([{a = {}}] of b) {}",
        b"for ([{ a = {} }] of b) {\n}\n",
    );
    expect_printed(b"for ({a = {}} in b) {}", b"for ({ a = {} } in b) {\n}\n");
    expect_printed(b"for ({a = {}} of b) {}", b"for ({ a = {} } of b) {\n}\n");
}

#[test]
#[ignore = "depends on esbuild parser lowering"]
fn test_for() {
    expect_printed(b"for (; of; ) ;", b"for (; of; ) ;\n");
    expect_printed(b"for (a in b) ;", b"for (a in b) ;\n");
    expect_printed(b"for (var a in b) ;", b"for (var a in b) ;\n");
    expect_printed(b"for (let a in b) ;", b"for (let a in b) ;\n");
    expect_printed(b"for (const a in b) ;", b"for (const a in b) ;\n");
    expect_printed(b"for (a in b, c) ;", b"for (a in b, c) ;\n");
    expect_printed(b"for (a in b = c) ;", b"for (a in b = c) ;\n");
    expect_printed(b"for (var a in b, c) ;", b"for (var a in b, c) ;\n");
    expect_printed(b"for (var a in b = c) ;", b"for (var a in b = c) ;\n");
    expect_printed(b"for (a of b) ;", b"for (a of b) ;\n");
    expect_printed(b"for (var a of b) ;", b"for (var a of b) ;\n");
    expect_printed(b"for (let a of b) ;", b"for (let a of b) ;\n");
    expect_printed(b"for (const a of b) ;", b"for (const a of b) ;\n");
    expect_printed(b"for (a of b = c) ;", b"for (a of b = c) ;\n");
    expect_printed(b"for (var a of b = c) ;", b"for (var a of b = c) ;\n");
    expect_printed(b"for ((let) of bar);", b"for ((let) of bar) ;\n");
    expect_printed(b"for ((let).foo of bar);", b"for ((let).foo of bar) ;\n");
    expect_printed(b"for ((let.foo) of bar);", b"for ((let).foo of bar) ;\n");
    expect_printed(
        b"for ((let``.foo) of bar);",
        b"for ((let)``.foo of bar) ;\n",
    );
    expect_printed(b"for (var x = 0 in y) ;", b"x = 0;\nfor (var x in y) ;\n");
    expect_printed(
        b"for (var x = () => a in b);",
        b"x = () => a;\nfor (var x in b) ;\n",
    );
    expect_printed(
        b"for (var x = a + b in c);",
        b"x = a + b;\nfor (var x in c) ;\n",
    );
    expect_printed(
        b"for (var x = `${y in z}`;;);",
        b"for (var x = `${y in z}`; ; ) ;\n",
    );
    expect_printed(
        b"for (var {[x in y]: z} = {};;);",
        b"for (var { [x in y]: z } = {}; ; ) ;\n",
    );
    expect_printed(
        b"for (var {x = y in z} = {};;);",
        b"for (var { x = y in z } = {}; ; ) ;\n",
    );
    expect_printed(
        b"for (var [x = y in z] = {};;);",
        b"for (var [x = y in z] = {}; ; ) ;\n",
    );
    expect_printed(
        b"for (var {x: y = z in w} = {};;);",
        b"for (var { x: y = z in w } = {}; ; ) ;\n",
    );
    expect_printed(
        b"for (var x = (a in b);;);",
        b"for (var x = (a in b); ; ) ;\n",
    );
    expect_printed(
        b"for (var x = [a in b];;);",
        b"for (var x = [a in b]; ; ) ;\n",
    );
    expect_printed(
        b"for (var x = y(a in b);;);",
        b"for (var x = y(a in b); ; ) ;\n",
    );
    expect_printed(
        b"for (var x = {y: a in b};;);",
        b"for (var x = { y: a in b }; ; ) ;\n",
    );
    expect_printed(b"for (a ? b in c : d;;);", b"for (a ? b in c : d; ; ) ;\n");
    expect_printed(
        b"for (var x = () => { a in b };;);",
        b"for (var x = () => {\n  a in b;\n}; ; ) ;\n",
    );
    expect_printed(
        b"for (var x = async () => { a in b };;);",
        b"for (var x = async () => {\n  a in b;\n}; ; ) ;\n",
    );
    expect_printed(
        b"for (var x = function() { a in b };;);",
        b"for (var x = function() {\n  a in b;\n}; ; ) ;\n",
    );
    expect_printed(
        b"for (var x = async function() { a in b };;);",
        b"for (var x = async function() {\n  a in b;\n}; ; ) ;\n",
    );
    expect_printed(
        b"for (var x = class { [a in b]() {} };;);",
        b"for (var x = class {\n  [a in b]() {\n  }\n}; ; ) ;\n",
    );
    expect_printed(b"async of => {}", b"async (of) => {\n};\n");
    expect_printed(b"for ((async) of []) ;", b"for ((async) of []) ;\n");
    expect_printed(b"for (async.x of []) ;", b"for (async.x of []) ;\n");
    expect_printed(
        b"for (async of => {};;) ;",
        b"for (async (of) => {\n}; ; ) ;\n",
    );
    expect_printed(b"for (\\u0061sync of []) ;", b"for ((async) of []) ;\n");
    expect_printed(b"for await (async of []) ;", b"for await (async of []) ;\n");
}

#[test]
#[ignore = "depends on esbuild parser ASI behavior"]
fn test_asi() {
    expect_printed(b"return\n0", b"return;\n0;\n");
    expect_printed(b"0\n[1]", b"0[1];\n");
    expect_printed(b"0\n(1)", b"0(1);\n");
    expect_printed(b"new x\n(1)", b"new x(1);\n");
    expect_printed(b"while (true) break\nx", b"while (true) break;\nx;\n");
    expect_printed(b"x\n!y", b"x;\n!y;\n");
    expect_printed(b"x\n++y", b"x;\n++y;\n");
    expect_printed(b"x\n--y", b"x;\n--y;\n");
    expect_printed(
        b"function* foo(){yield\na}",
        b"function* foo() {\n  yield;\n  a;\n}\n",
    );
    expect_printed(
        b"function* foo(){yield*\na}",
        b"function* foo() {\n  yield* a;\n}\n",
    );
    expect_printed(b"async\nx => {}", b"async;\n(x) => {\n};\n");
    expect_printed(
        b"async\nfunction foo() {}",
        b"async;\nfunction foo() {\n}\n",
    );
    expect_printed(
        b"export default async\nx => {}",
        b"export default async;\n(x) => {\n};\n",
    );
    expect_printed(
        b"export default async\nfunction foo() {}",
        b"export default async;\nfunction foo() {\n}\n",
    );
    expect_printed(b"if (0) let\nx = 0", b"if (0) let;\nx = 0;\n");
    expect_printed(b"if (0) let\n{x}", b"if (0) let;\n{\n  x;\n}\n");
    expect_printed(
        b"function *foo() { if (0) let\nyield 0 }",
        b"function* foo() {\n  if (0) let;\n  yield 0;\n}\n",
    );
    expect_printed(
        b"async function foo() { if (0) let\nawait 0 }",
        b"async function foo() {\n  if (0) let;\n  await 0;\n}\n",
    );
    expect_printed(b"let\nx = 0", b"let x = 0;\n");
    expect_printed(b"let\n{x} = 0", b"let { x } = 0;\n");
    expect_printed(b"let\n[x] = 0", b"let [x] = 0;\n");
    expect_printed(b"do x;while(y)z", b"do\n  x;\nwhile (y);\nz;\n");
    expect_printed(b"do x;while(y);z", b"do\n  x;\nwhile (y);\nz;\n");
    expect_printed(b"{do x;while(y)}", b"{\n  do\n    x;\n  while (y);\n}\n");
}

#[test]
fn test_local() {
    expect_printed(b"var let = 0", b"var let = 0;\n");
    expect_printed(b"var\nlet = 0", b"var let = 0;\n");
    expect_printed(b"for (var let in x) ;", b"for (var let in x) ;\n");
    expect_printed(b"for (var let of x) ;", b"for (var let of x) ;\n");
}

#[test]
fn test_arrays() {
    expect_printed(b"[]", b"[];\n");
    expect_printed(b"[,]", b"[,];\n");
    expect_printed(b"[1]", b"[1];\n");
    expect_printed(b"[1,]", b"[1];\n");
    expect_printed(b"[,1]", b"[, 1];\n");
    expect_printed(b"[1,2]", b"[1, 2];\n");
    expect_printed(b"[,1,2]", b"[, 1, 2];\n");
    expect_printed(b"[1,,2]", b"[1, , 2];\n");
    expect_printed(b"[1,2,]", b"[1, 2];\n");
    expect_printed(b"[1,2,,]", b"[1, 2, ,];\n");
}

#[test]
fn test_pattern() {
    expect_printed(b"let {if: x} = y", b"let { if: x } = y;\n");
    expect_printed(b"let {1_2_3n: x} = y", b"let { 123n: x } = y;\n");
    expect_printed(b"let {0x1_2_3n: x} = y", b"let { 0x123n: x } = y;\n");
}

#[test]
fn test_object() {
    expect_printed(b"({foo})", b"({ foo });\n");
    expect_printed(b"({foo:0})", b"({ foo: 0 });\n");
    expect_printed(b"({1e9:0})", b"({ 1e9: 0 });\n");
    expect_printed(b"({1_2_3n:0})", b"({ 123n: 0 });\n");
    expect_printed(b"({0x1_2_3n:0})", b"({ 0x123n: 0 });\n");
    expect_printed(b"({foo() {}})", b"({ foo() {\n} });\n");
    expect_printed(b"({*foo() {}})", b"({ *foo() {\n} });\n");
    expect_printed(b"({get foo() {}})", b"({ get foo() {\n} });\n");
    expect_printed(b"({set foo(x) {}})", b"({ set foo(x) {\n} });\n");
    expect_printed(b"({if:0})", b"({ if: 0 });\n");
    expect_printed(b"({if() {}})", b"({ if() {\n} });\n");
    expect_printed(b"({*if() {}})", b"({ *if() {\n} });\n");
    expect_printed(b"({get if() {}})", b"({ get if() {\n} });\n");
    expect_printed(b"({set if(x) {}})", b"({ set if(x) {\n} });\n");
    expect_printed(b"x = { get \n x() {} }", b"x = { get x() {\n} };\n");
    expect_printed(b"x = { set \n x(_) {} }", b"x = { set x(_) {\n} };\n");
}

#[test]
fn test_computed_property() {
    expect_printed(b"({[a]: foo})", b"({ [a]: foo });\n");
    expect_printed(b"({[(a, b)]: foo})", b"({ [(a, b)]: foo });\n");
    expect_printed(b"({[a]: foo}) => {}", b"({ [a]: foo }) => {\n};\n");
    expect_printed(
        b"({[(a, b)]: foo}) => {}",
        b"({ [(a, b)]: foo }) => {\n};\n",
    );
    expect_printed(b"var {[a]: foo} = bar", b"var { [a]: foo } = bar;\n");
    expect_printed(
        b"var {[(a, b)]: foo} = bar",
        b"var { [(a, b)]: foo } = bar;\n",
    );
    expect_printed(b"class Foo {[a] = foo}", b"class Foo {\n  [a] = foo;\n}\n");
    expect_printed(
        b"class Foo {[(a, b)] = foo}",
        b"class Foo {\n  [(a, b)] = foo;\n}\n",
    );
}

#[test]
fn test_quoted_property() {
    expect_printed(b"x.x; y['y']", b"x.x;\ny[\"y\"];\n");
    expect_printed(b"({y: y, 'z': z} = x)", b"({ y, \"z\": z } = x);\n");
    expect_printed(b"var {y: y, 'z': z} = x", b"var { y, \"z\": z } = x;\n");
    expect_printed(b"x = {y: 1, 'z': 2}", b"x = { y: 1, \"z\": 2 };\n");
    expect_printed(
        b"x = {y() {}, 'z'() {}}",
        b"x = { y() {\n}, \"z\"() {\n} };\n",
    );
    expect_printed(
        b"x = {get y() {}, set 'z'(z) {}}",
        b"x = { get y() {\n}, set \"z\"(z) {\n} };\n",
    );
    expect_printed(
        b"x = class {y = 1; 'z' = 2}",
        b"x = class {\n  y = 1;\n  \"z\" = 2;\n};\n",
    );
    expect_printed(
        b"x = class {y() {}; 'z'() {}}",
        b"x = class {\n  y() {\n  }\n  \"z\"() {\n  }\n};\n",
    );
    expect_printed(
        b"x = class {get y() {}; set 'z'(z) {}}",
        b"x = class {\n  get y() {\n  }\n  set \"z\"(z) {\n  }\n};\n",
    );
}

#[test]
#[ignore = "depends on esbuild parser lowering"]
fn test_lexical_decl() {
    expect_printed(b"if (1) var x", b"if (1) var x;\n");
    expect_printed(
        b"if (1) function x() {}",
        b"if (1) {\n  let x = function() {\n  };\n  var x = x;\n}\n",
    );
    expect_printed(
        b"if (1) {} else function x() {}",
        b"if (1) {\n} else {\n  let x = function() {\n  };\n  var x = x;\n}\n",
    );
    expect_printed(
        b"switch (1) { case 1: const x = 1 }",
        b"switch (1) {\n  case 1:\n    const x = 1;\n}\n",
    );
    expect_printed(
        b"switch (1) { default: const x = 1 }",
        b"switch (1) {\n  default:\n    const x = 1;\n}\n",
    );
    expect_printed(b"function f() {}", b"function f() {\n}\n");
    expect_printed(
        b"{function f() {}} let f",
        b"{\n  let f = function() {\n  };\n}\nlet f;\n",
    );
    expect_printed(
        b"if (1) function f() {} let f",
        b"if (1) {\n  let f = function() {\n  };\n}\nlet f;\n",
    );
    expect_printed(
        b"if (0) ; else function f() {} let f",
        b"if (0) ;\nelse {\n  let f = function() {\n  };\n}\nlet f;\n",
    );
    expect_printed(
        b"x: function f() {}",
        b"x: {\n  let f = function() {\n  };\n  var f = f;\n}\n",
    );
    expect_printed(
        b"{function* f() {}} let f",
        b"{\n  function* f() {\n  }\n}\nlet f;\n",
    );
    expect_printed(
        b"{async function f() {}} let f",
        b"{\n  async function f() {\n  }\n}\nlet f;\n",
    );
    expect_printed(
        b"if (foo) { function x() {} }",
        b"if (foo) {\n  let x = function() {\n  };\n  var x = x;\n}\n",
    );
    expect_printed(
        b"if (foo) { function x() {} eval('') }",
        b"if (foo) {\n  function x() {\n  }\n  eval(\"\");\n}\n",
    );
    expect_printed(
        b"if (foo) { function x() {} if (bar) { eval('') } }",
        b"if (foo) {\n  function x() {\n  }\n  if (bar) {\n    eval(\"\");\n  }\n}\n",
    );
    expect_printed(
        b"if (foo) { eval(''); function x() {} }",
        b"if (foo) {\n  function x() {\n  }\n  eval(\"\");\n}\n",
    );
    expect_printed(
        b"'use strict'; if (foo) { function x() {} }",
        b"\"use strict\";\nif (foo) {\n  let x = function() {\n  };\n}\n",
    );
    expect_printed(
        b"'use strict'; if (foo) { function x() {} eval('') }",
        b"\"use strict\";\nif (foo) {\n  function x() {\n  }\n  eval(\"\");\n}\n",
    );
    expect_printed(
        b"'use strict'; if (foo) { function x() {} if (bar) { eval('') } }",
        b"\"use strict\";\nif (foo) {\n  function x() {\n  }\n  if (bar) {\n    eval(\"\");\n  }\n}\n",
    );
    expect_printed(
        b"'use strict'; if (foo) { eval(''); function x() {} }",
        b"\"use strict\";\nif (foo) {\n  function x() {\n  }\n  eval(\"\");\n}\n",
    );
}

#[test]
fn test_function() {
    expect_printed(
        b"function f() {} function f() {}",
        b"function f() {\n}\nfunction f() {\n}\n",
    );
    expect_printed(
        b"function f() {} function* f() {}",
        b"function f() {\n}\nfunction* f() {\n}\n",
    );
    expect_printed(
        b"function* f() {} function* f() {}",
        b"function* f() {\n}\nfunction* f() {\n}\n",
    );
    expect_printed(
        b"function f() {} async function f() {}",
        b"function f() {\n}\nasync function f() {\n}\n",
    );
    expect_printed(
        b"async function f() {} async function f() {}",
        b"async function f() {\n}\nasync function f() {\n}\n",
    );
    expect_printed(b"function arguments() {}", b"function arguments() {\n}\n");
    expect_printed(
        b"(function arguments() {})",
        b"(function arguments() {\n});\n",
    );
    expect_printed(
        b"function foo(arguments) {}",
        b"function foo(arguments) {\n}\n",
    );
    expect_printed(
        b"(function foo(arguments) {})",
        b"(function foo(arguments) {\n});\n",
    );
    expect_printed(
        b"(function foo() { var arguments })",
        b"(function foo() {\n  var arguments;\n});\n",
    );
    expect_printed(
        b"(function foo() { { var arguments } })",
        b"(function foo() {\n  {\n    var arguments;\n  }\n});\n",
    );
    expect_printed(
        b"var x = function() { y() }()",
        b"var x = function() {\n  y();\n}();\n",
    );
    // Depends on esbuild's parser-stage constant folding.
    // expect_printed(
    //     b"var x = (true && function() { y() })()",
    //     b"var x = (function() {\n  y();\n})();\n",
    // );
}

#[test]
fn test_class() {
    expect_printed(
        b"class Foo { foo() {} }",
        b"class Foo {\n  foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { *foo() {} }",
        b"class Foo {\n  *foo() {\n  }\n}\n",
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
        b"class Foo { async foo() {} }",
        b"class Foo {\n  async foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { async *foo() {} }",
        b"class Foo {\n  async *foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static foo() {} }",
        b"class Foo {\n  static foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static *foo() {} }",
        b"class Foo {\n  static *foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static get foo() {} }",
        b"class Foo {\n  static get foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static set foo(x) {} }",
        b"class Foo {\n  static set foo(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static async foo() {} }",
        b"class Foo {\n  static async foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static async *foo() {} }",
        b"class Foo {\n  static async *foo() {\n  }\n}\n",
    );
    expect_printed(b"class Foo { if() {} }", b"class Foo {\n  if() {\n  }\n}\n");
    expect_printed(
        b"class Foo { *if() {} }",
        b"class Foo {\n  *if() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { get if() {} }",
        b"class Foo {\n  get if() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { set if(x) {} }",
        b"class Foo {\n  set if(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { async if() {} }",
        b"class Foo {\n  async if() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { async *if() {} }",
        b"class Foo {\n  async *if() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static if() {} }",
        b"class Foo {\n  static if() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static *if() {} }",
        b"class Foo {\n  static *if() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static get if() {} }",
        b"class Foo {\n  static get if() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static set if(x) {} }",
        b"class Foo {\n  static set if(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static async if() {} }",
        b"class Foo {\n  static async if() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static async *if() {} }",
        b"class Foo {\n  static async *if() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} b() {} }",
        b"class Foo {\n  a() {\n  }\n  b() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} get b() {} }",
        b"class Foo {\n  a() {\n  }\n  get b() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} set b(x) {} }",
        b"class Foo {\n  a() {\n  }\n  set b(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} async b() {} }",
        b"class Foo {\n  a() {\n  }\n  async b() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} async *b() {} }",
        b"class Foo {\n  a() {\n  }\n  async *b() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} static b() {} }",
        b"class Foo {\n  a() {\n  }\n  static b() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} static *b() {} }",
        b"class Foo {\n  a() {\n  }\n  static *b() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} static get b() {} }",
        b"class Foo {\n  a() {\n  }\n  static get b() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} static set b(x) {} }",
        b"class Foo {\n  a() {\n  }\n  static set b(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} static async b() {} }",
        b"class Foo {\n  a() {\n  }\n  static async b() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { a() {} static async *b() {} }",
        b"class Foo {\n  a() {\n  }\n  static async *b() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { [arguments] }",
        b"class Foo {\n  [arguments];\n}\n",
    );
    expect_printed(
        b"class Foo { [arguments] = 1 }",
        b"class Foo {\n  [arguments] = 1;\n}\n",
    );
    expect_printed(
        b"class Foo { arguments = 1 }",
        b"class Foo {\n  arguments = 1;\n}\n",
    );
    expect_printed(
        b"class Foo { x = class { arguments = 1 } }",
        b"class Foo {\n  x = class {\n    arguments = 1;\n  };\n}\n",
    );
    expect_printed(
        b"class Foo { x = function() { arguments } }",
        b"class Foo {\n  x = function() {\n    arguments;\n  };\n}\n",
    );
    expect_printed(
        b"class Foo { get ['constructor']() {} }",
        b"class Foo {\n  get [\"constructor\"]() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { set ['constructor'](x) {} }",
        b"class Foo {\n  set [\"constructor\"](x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { *['constructor']() {} }",
        b"class Foo {\n  *[\"constructor\"]() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { async ['constructor']() {} }",
        b"class Foo {\n  async [\"constructor\"]() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { async *['constructor']() {} }",
        b"class Foo {\n  async *[\"constructor\"]() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static get constructor() {} }",
        b"class Foo {\n  static get constructor() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static get 'constructor'() {} }",
        b"class Foo {\n  static get \"constructor\"() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static set constructor(x) {} }",
        b"class Foo {\n  static set constructor(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static set 'constructor'(x) {} }",
        b"class Foo {\n  static set \"constructor\"(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static *constructor() {} }",
        b"class Foo {\n  static *constructor() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static *'constructor'() {} }",
        b"class Foo {\n  static *\"constructor\"() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static async constructor() {} }",
        b"class Foo {\n  static async constructor() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static async 'constructor'() {} }",
        b"class Foo {\n  static async \"constructor\"() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static async *constructor() {} }",
        b"class Foo {\n  static async *constructor() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static async *'constructor'() {} }",
        b"class Foo {\n  static async *\"constructor\"() {\n  }\n}\n",
    );
    expect_printed(b"({ constructor: 1 })", b"({ constructor: 1 });\n");
    expect_printed(
        b"({ get constructor() {} })",
        b"({ get constructor() {\n} });\n",
    );
    expect_printed(
        b"({ set constructor(x) {} })",
        b"({ set constructor(x) {\n} });\n",
    );
    expect_printed(b"({ *constructor() {} })", b"({ *constructor() {\n} });\n");
    expect_printed(
        b"({ async constructor() {} })",
        b"({ async constructor() {\n} });\n",
    );
    expect_printed(
        b"({ async* constructor() {} })",
        b"({ async *constructor() {\n} });\n",
    );
    expect_printed(
        b"class Foo { get prototype() {} }",
        b"class Foo {\n  get prototype() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { get 'prototype'() {} }",
        b"class Foo {\n  get \"prototype\"() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { set prototype(x) {} }",
        b"class Foo {\n  set prototype(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { set 'prototype'(x) {} }",
        b"class Foo {\n  set \"prototype\"(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { *prototype() {} }",
        b"class Foo {\n  *prototype() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { *'prototype'() {} }",
        b"class Foo {\n  *\"prototype\"() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { async prototype() {} }",
        b"class Foo {\n  async prototype() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { async 'prototype'() {} }",
        b"class Foo {\n  async \"prototype\"() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { async *prototype() {} }",
        b"class Foo {\n  async *prototype() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { async *'prototype'() {} }",
        b"class Foo {\n  async *\"prototype\"() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static get ['prototype']() {} }",
        b"class Foo {\n  static get [\"prototype\"]() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static set ['prototype'](x) {} }",
        b"class Foo {\n  static set [\"prototype\"](x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static *['prototype']() {} }",
        b"class Foo {\n  static *[\"prototype\"]() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static async ['prototype']() {} }",
        b"class Foo {\n  static async [\"prototype\"]() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static async *['prototype']() {} }",
        b"class Foo {\n  static async *[\"prototype\"]() {\n  }\n}\n",
    );
    expect_printed(b"({ prototype: 1 })", b"({ prototype: 1 });\n");
    expect_printed(
        b"({ get prototype() {} })",
        b"({ get prototype() {\n} });\n",
    );
    expect_printed(
        b"({ set prototype(x) {} })",
        b"({ set prototype(x) {\n} });\n",
    );
    expect_printed(b"({ *prototype() {} })", b"({ *prototype() {\n} });\n");
    expect_printed(
        b"({ async prototype() {} })",
        b"({ async prototype() {\n} });\n",
    );
    expect_printed(
        b"({ async* prototype() {} })",
        b"({ async *prototype() {\n} });\n",
    );
    expect_printed(
        b"class Foo { foo = [Foo, eval(bar)] }",
        b"class Foo {\n  foo = [Foo, eval(bar)];\n}\n",
    );
    expect_printed(
        b"class A { get \n x() {} }",
        b"class A {\n  get x() {\n  }\n}\n",
    );
    expect_printed(
        b"class A { set \n x(_) {} }",
        b"class A {\n  set x(_) {\n  }\n}\n",
    );
    expect_printed(
        b"class A { get \n *x() {} }",
        b"class A {\n  get;\n  *x() {\n  }\n}\n",
    );
    expect_printed(
        b"class A { set \n *x(_) {} }",
        b"class A {\n  set;\n  *x(_) {\n  }\n}\n",
    );
}

#[test]
fn test_super_call() {
    expect_printed(
        b"class Foo extends Bar { constructor() { super() } }",
        b"class Foo extends Bar {\n  constructor() {\n    super();\n  }\n}\n",
    );
    expect_printed(
        b"class Foo extends Bar { constructor() { () => super() } }",
        b"class Foo extends Bar {\n  constructor() {\n    () => super();\n  }\n}\n",
    );
    expect_printed(b"class Foo extends Bar { constructor() { () => { super() } } }", b"class Foo extends Bar {\n  constructor() {\n    () => {\n      super();\n    };\n  }\n}\n");
    expect_printed(
        b"class Foo extends Bar { constructor(x = super()) {} }",
        b"class Foo extends Bar {\n  constructor(x = super()) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo extends Bar { constructor(x = () => super()) {} }",
        b"class Foo extends Bar {\n  constructor(x = () => super()) {\n  }\n}\n",
    );
}

#[test]
fn test_super_prop() {
    expect_printed(
        b"class Foo { foo() { super.x } }",
        b"class Foo {\n  foo() {\n    super.x;\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { foo() { super[x] } }",
        b"class Foo {\n  foo() {\n    super[x];\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { foo(x = super.x) {} }",
        b"class Foo {\n  foo(x = super.x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { foo(x = super[x]) {} }",
        b"class Foo {\n  foo(x = super[x]) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static foo() { super.x } }",
        b"class Foo {\n  static foo() {\n    super.x;\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static foo() { super[x] } }",
        b"class Foo {\n  static foo() {\n    super[x];\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static foo(x = super.x) {} }",
        b"class Foo {\n  static foo(x = super.x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static foo(x = super[x]) {} }",
        b"class Foo {\n  static foo(x = super[x]) {\n  }\n}\n",
    );
    expect_printed(
        b"(class { foo() { super.x } })",
        b"(class {\n  foo() {\n    super.x;\n  }\n});\n",
    );
    expect_printed(
        b"(class { foo() { super[x] } })",
        b"(class {\n  foo() {\n    super[x];\n  }\n});\n",
    );
    expect_printed(
        b"(class { foo(x = super.x) {} })",
        b"(class {\n  foo(x = super.x) {\n  }\n});\n",
    );
    expect_printed(
        b"(class { foo(x = super[x]) {} })",
        b"(class {\n  foo(x = super[x]) {\n  }\n});\n",
    );
    expect_printed(
        b"(class { static foo() { super.x } })",
        b"(class {\n  static foo() {\n    super.x;\n  }\n});\n",
    );
    expect_printed(
        b"(class { static foo() { super[x] } })",
        b"(class {\n  static foo() {\n    super[x];\n  }\n});\n",
    );
    expect_printed(
        b"(class { static foo(x = super.x) {} })",
        b"(class {\n  static foo(x = super.x) {\n  }\n});\n",
    );
    expect_printed(
        b"(class { static foo(x = super[x]) {} })",
        b"(class {\n  static foo(x = super[x]) {\n  }\n});\n",
    );
    expect_printed(
        b"class Foo { foo = super.x }",
        b"class Foo {\n  foo = super.x;\n}\n",
    );
    expect_printed(
        b"class Foo { foo = super[x] }",
        b"class Foo {\n  foo = super[x];\n}\n",
    );
    expect_printed(
        b"class Foo { foo = () => super.x }",
        b"class Foo {\n  foo = () => super.x;\n}\n",
    );
    expect_printed(
        b"class Foo { foo = () => super[x] }",
        b"class Foo {\n  foo = () => super[x];\n}\n",
    );
    expect_printed(
        b"class Foo { static foo = super.x }",
        b"class Foo {\n  static foo = super.x;\n}\n",
    );
    expect_printed(
        b"class Foo { static foo = super[x] }",
        b"class Foo {\n  static foo = super[x];\n}\n",
    );
    expect_printed(
        b"class Foo { static foo = () => super.x }",
        b"class Foo {\n  static foo = () => super.x;\n}\n",
    );
    expect_printed(
        b"class Foo { static foo = () => super[x] }",
        b"class Foo {\n  static foo = () => super[x];\n}\n",
    );
    expect_printed(
        b"(class { foo = super.x })",
        b"(class {\n  foo = super.x;\n});\n",
    );
    expect_printed(
        b"(class { foo = super[x] })",
        b"(class {\n  foo = super[x];\n});\n",
    );
    expect_printed(
        b"(class { foo = () => super.x })",
        b"(class {\n  foo = () => super.x;\n});\n",
    );
    expect_printed(
        b"(class { foo = () => super[x] })",
        b"(class {\n  foo = () => super[x];\n});\n",
    );
    expect_printed(
        b"(class { static foo = super.x })",
        b"(class {\n  static foo = super.x;\n});\n",
    );
    expect_printed(
        b"(class { static foo = super[x] })",
        b"(class {\n  static foo = super[x];\n});\n",
    );
    expect_printed(
        b"(class { static foo = () => super.x })",
        b"(class {\n  static foo = () => super.x;\n});\n",
    );
    expect_printed(
        b"(class { static foo = () => super[x] })",
        b"(class {\n  static foo = () => super[x];\n});\n",
    );
    expect_printed(
        b"({ foo() { super.x } })",
        b"({ foo() {\n  super.x;\n} });\n",
    );
    expect_printed(
        b"({ foo() { super[x] } })",
        b"({ foo() {\n  super[x];\n} });\n",
    );
    expect_printed(
        b"({ foo(x = super.x) {} })",
        b"({ foo(x = super.x) {\n} });\n",
    );
}

#[test]
fn test_class_fields() {
    expect_printed(b"class Foo { a }", b"class Foo {\n  a;\n}\n");
    expect_printed(b"class Foo { a = 1 }", b"class Foo {\n  a = 1;\n}\n");
    expect_printed(
        b"class Foo { a = 1; b }",
        b"class Foo {\n  a = 1;\n  b;\n}\n",
    );
    expect_printed(b"class Foo { [a] }", b"class Foo {\n  [a];\n}\n");
    expect_printed(b"class Foo { [a] = 1 }", b"class Foo {\n  [a] = 1;\n}\n");
    expect_printed(
        b"class Foo { [a] = 1; [b] }",
        b"class Foo {\n  [a] = 1;\n  [b];\n}\n",
    );
    expect_printed(b"class Foo { static a }", b"class Foo {\n  static a;\n}\n");
    expect_printed(
        b"class Foo { static a = 1 }",
        b"class Foo {\n  static a = 1;\n}\n",
    );
    expect_printed(
        b"class Foo { static a = 1; b }",
        b"class Foo {\n  static a = 1;\n  b;\n}\n",
    );
    expect_printed(
        b"class Foo { static [a] }",
        b"class Foo {\n  static [a];\n}\n",
    );
    expect_printed(
        b"class Foo { static [a] = 1 }",
        b"class Foo {\n  static [a] = 1;\n}\n",
    );
    expect_printed(
        b"class Foo { static [a] = 1; [b] }",
        b"class Foo {\n  static [a] = 1;\n  [b];\n}\n",
    );
    expect_printed(
        b"class Foo { ['constructor'] }",
        b"class Foo {\n  [\"constructor\"];\n}\n",
    );
    expect_printed(
        b"class Foo { ['constructor'] = 1 }",
        b"class Foo {\n  [\"constructor\"] = 1;\n}\n",
    );
    expect_printed(
        b"class Foo { static ['constructor'] }",
        b"class Foo {\n  static [\"constructor\"];\n}\n",
    );
    expect_printed(
        b"class Foo { static ['constructor'] = 1 }",
        b"class Foo {\n  static [\"constructor\"] = 1;\n}\n",
    );
    expect_printed(
        b"class Foo { prototype }",
        b"class Foo {\n  prototype;\n}\n",
    );
    expect_printed(
        b"class Foo { 'prototype' }",
        b"class Foo {\n  \"prototype\";\n}\n",
    );
    expect_printed(
        b"class Foo { prototype = 1 }",
        b"class Foo {\n  prototype = 1;\n}\n",
    );
    expect_printed(
        b"class Foo { 'prototype' = 1 }",
        b"class Foo {\n  \"prototype\" = 1;\n}\n",
    );
    expect_printed(
        b"class Foo { static ['prototype'] }",
        b"class Foo {\n  static [\"prototype\"];\n}\n",
    );
    expect_printed(
        b"class Foo { static ['prototype'] = 1 }",
        b"class Foo {\n  static [\"prototype\"] = 1;\n}\n",
    );
}

#[test]
fn test_class_static_blocks() {
    expect_printed(
        b"class Foo { static {} }",
        b"class Foo {\n  static {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static {} x = 1 }",
        b"class Foo {\n  static {\n  }\n  x = 1;\n}\n",
    );
    expect_printed(
        b"class Foo { static { this.foo() } }",
        b"class Foo {\n  static {\n    this.foo();\n  }\n}\n",
    );
}

#[test]
fn test_auto_accessors() {
    expect_printed(b"class Foo { accessor }", b"class Foo {\n  accessor;\n}\n");
    expect_printed(
        b"class Foo { accessor \n x }",
        b"class Foo {\n  accessor;\n  x;\n}\n",
    );
    expect_printed(
        b"class Foo { static accessor }",
        b"class Foo {\n  static accessor;\n}\n",
    );
    expect_printed(
        b"class Foo { static accessor \n x }",
        b"class Foo {\n  static accessor;\n  x;\n}\n",
    );
    expect_printed(
        b"class Foo { accessor x }",
        b"class Foo {\n  accessor x;\n}\n",
    );
    expect_printed(
        b"class Foo { accessor x = y }",
        b"class Foo {\n  accessor x = y;\n}\n",
    );
    expect_printed(
        b"class Foo { accessor [x] }",
        b"class Foo {\n  accessor [x];\n}\n",
    );
    expect_printed(
        b"class Foo { accessor [x] = y }",
        b"class Foo {\n  accessor [x] = y;\n}\n",
    );
    expect_printed(
        b"class Foo { static accessor x }",
        b"class Foo {\n  static accessor x;\n}\n",
    );
    expect_printed(
        b"class Foo { static accessor [x] }",
        b"class Foo {\n  static accessor [x];\n}\n",
    );
    expect_printed(
        b"class Foo { static accessor x = y }",
        b"class Foo {\n  static accessor x = y;\n}\n",
    );
    expect_printed(
        b"class Foo { static accessor [x] = y }",
        b"class Foo {\n  static accessor [x] = y;\n}\n",
    );
    expect_printed(
        b"Foo = class { accessor x }",
        b"Foo = class {\n  accessor x;\n};\n",
    );
    expect_printed(
        b"Foo = class { accessor [x] }",
        b"Foo = class {\n  accessor [x];\n};\n",
    );
    expect_printed(
        b"Foo = class { accessor x = y }",
        b"Foo = class {\n  accessor x = y;\n};\n",
    );
    expect_printed(
        b"Foo = class { accessor [x] = y }",
        b"Foo = class {\n  accessor [x] = y;\n};\n",
    );
    expect_printed(
        b"Foo = class { static accessor x }",
        b"Foo = class {\n  static accessor x;\n};\n",
    );
    expect_printed(
        b"Foo = class { static accessor [x] }",
        b"Foo = class {\n  static accessor [x];\n};\n",
    );
    expect_printed(
        b"Foo = class { static accessor x = y }",
        b"Foo = class {\n  static accessor x = y;\n};\n",
    );
    expect_printed(
        b"class Foo { accessor get }",
        b"class Foo {\n  accessor get;\n}\n",
    );
    expect_printed(
        b"class Foo { get accessor() {} }",
        b"class Foo {\n  get accessor() {\n  }\n}\n",
    );
    expect_printed(
        b"Foo = { get accessor() {} }",
        b"Foo = { get accessor() {\n} };\n",
    );
}

#[test]
#[ignore = "depends on esbuild decorator AST placement"]
fn test_decorators() {
    expect_printed(b"@x @y class Foo {}", b"@x @y class Foo {\n}\n");
    expect_printed(
        b"@x @y export class Foo {}",
        b"@x @y export class Foo {\n}\n",
    );
    expect_printed(
        b"@x @y export default class Foo {}",
        b"@x @y export default class Foo {\n}\n",
    );
    expect_printed(b"_ = @x @y class {}", b"_ = @x @y class {\n};\n");
    expect_printed(b"class Foo { @x y }", b"class Foo {\n  @x y;\n}\n");
    expect_printed(
        b"class Foo { @x y() {} }",
        b"class Foo {\n  @x y() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { @x static y }",
        b"class Foo {\n  @x static y;\n}\n",
    );
    expect_printed(
        b"class Foo { @x static y() {} }",
        b"class Foo {\n  @x static y() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { @x accessor y }",
        b"class Foo {\n  @x accessor y;\n}\n",
    );
    expect_printed(b"class Foo { @x #y }", b"class Foo {\n  @x #y;\n}\n");
    expect_printed(
        b"class Foo { @x #y() {} }",
        b"class Foo {\n  @x #y() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { @x static #y }",
        b"class Foo {\n  @x static #y;\n}\n",
    );
    expect_printed(
        b"class Foo { @x static #y() {} }",
        b"class Foo {\n  @x static #y() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { @x accessor #y }",
        b"class Foo {\n  @x accessor #y;\n}\n",
    );
    expect_printed(
        b"@\na\n(\n)\n@\n(\nb\n)\nclass\nFoo\n{\n}\n",
        b"@a()\n@b\nclass Foo {\n}\n",
    );
    expect_printed(b"@(a, b) class Foo {}", b"@(a, b) class Foo {\n}\n");
    expect_printed(b"@x() class Foo {}", b"@x() class Foo {\n}\n");
    expect_printed(b"@x.y() class Foo {}", b"@x.y() class Foo {\n}\n");
    expect_printed(
        b"@(() => {}) class Foo {}",
        b"@(() => {\n}) class Foo {\n}\n",
    );
    expect_printed(
        b"class Foo { #x = @y.#x.y.#x class {} }",
        b"class Foo {\n  #x = @y.#x.y.#x class {\n  };\n}\n",
    );
    expect_printed(b"@x export class Foo {}", b"@x export class Foo {\n}\n");
    expect_printed(b"export @x class Foo {}", b"@x export class Foo {\n}\n");
    expect_printed(
        b"@x export default class {}",
        b"@x export default class {\n}\n",
    );
    expect_printed(
        b"export default @x class {}",
        b"@x export default class {\n}\n",
    );
    expect_printed(
        b"@x export default class Foo {}",
        b"@x export default class Foo {\n}\n",
    );
    expect_printed(
        b"export default @x class Foo {}",
        b"@x export default class Foo {\n}\n",
    );
    expect_printed(
        b"export default (@x class {})",
        b"export default (@x class {\n});\n",
    );
    expect_printed(
        b"export default (@x class Foo {})",
        b"export default (@x class Foo {\n});\n",
    );
}

#[test]
fn test_generator() {
    expect_printed(
        b"function* foo() { -(yield 100) }",
        b"function* foo() {\n  -(yield 100);\n}\n",
    );
}

#[test]
fn test_yield() {
    expect_printed(b"yield\n100", b"yield;\n100;\n");
    expect_printed(
        b"function *foo() { x = yield }",
        b"function* foo() {\n  x = yield;\n}\n",
    );
    expect_printed(
        b"function *foo() { x = yield; }",
        b"function* foo() {\n  x = yield;\n}\n",
    );
    expect_printed(
        b"function *foo() { (x = yield) }",
        b"function* foo() {\n  x = yield;\n}\n",
    );
    expect_printed(
        b"function *foo() { [x = yield] }",
        b"function* foo() {\n  [x = yield];\n}\n",
    );
    expect_printed(
        b"function *foo() { x = (yield, yield) }",
        b"function* foo() {\n  x = (yield, yield);\n}\n",
    );
    expect_printed(
        b"function *foo() { x = y ? yield : yield }",
        b"function* foo() {\n  x = y ? yield : yield;\n}\n",
    );
    expect_printed(
        b"function *foo() { x = yield y }",
        b"function* foo() {\n  x = yield y;\n}\n",
    );
    expect_printed(
        b"function *foo() { (x = yield y) }",
        b"function* foo() {\n  x = yield y;\n}\n",
    );
    expect_printed(
        b"function *foo() { x = yield \n y }",
        b"function* foo() {\n  x = yield;\n  y;\n}\n",
    );
    expect_printed(
        b"function *foo() { x = yield * y }",
        b"function* foo() {\n  x = yield* y;\n}\n",
    );
    expect_printed(
        b"function *foo() { (x = yield * y) }",
        b"function* foo() {\n  x = yield* y;\n}\n",
    );
    expect_printed(
        b"function *foo() { x = yield * \n y }",
        b"function* foo() {\n  x = yield* y;\n}\n",
    );
    expect_printed(
        b"function foo() { x = yield * y }",
        b"function foo() {\n  x = yield * y;\n}\n",
    );
    expect_printed(
        b"function foo() { (x = yield * y) }",
        b"function foo() {\n  x = yield * y;\n}\n",
    );
    expect_printed(b"({yield} = x)", b"({ yield } = x);\n");
    expect_printed(b"let x = {yield}", b"let x = { yield };\n");
    expect_printed(b"function* yield() {}", b"function* yield() {\n}\n");
    expect_printed(
        b"function foo() { ({yield} = x) }",
        b"function foo() {\n  ({ yield } = x);\n}\n",
    );
    expect_printed(
        b"function foo() { let x = {yield} }",
        b"function foo() {\n  let x = { yield };\n}\n",
    );
    expect_printed(b"({ *yield() {} })", b"({ *yield() {\n} });\n");
    expect_printed(
        b"(class { *yield() {} })",
        b"(class {\n  *yield() {\n  }\n});\n",
    );
    expect_printed(
        b"class Foo { *yield() {} }",
        b"class Foo {\n  *yield() {\n  }\n}\n",
    );
    expect_printed(b"function* yield() {}", b"function* yield() {\n}\n");
    expect_printed(b"({ async *yield() {} })", b"({ async *yield() {\n} });\n");
    expect_printed(
        b"(class { async *yield() {} })",
        b"(class {\n  async *yield() {\n  }\n});\n",
    );
    expect_printed(
        b"class Foo { async *yield() {} }",
        b"class Foo {\n  async *yield() {\n  }\n}\n",
    );
    expect_printed(
        b"async function* yield() {}",
        b"async function* yield() {\n}\n",
    );
}

#[test]
#[ignore = "depends on esbuild parser ASI behavior"]
fn test_async() {
    expect_printed_script(
        b"function foo() { await }",
        b"function foo() {\n  await;\n}\n",
    );
    expect_printed(
        b"async function foo() { await 0 }",
        b"async function foo() {\n  await 0;\n}\n",
    );
    expect_printed(
        b"-async function foo() { await 0 }",
        b"-async function foo() {\n  await 0;\n};\n",
    );
    expect_printed(
        b"-async function() { await 0 }",
        b"-async function() {\n  await 0;\n};\n",
    );
    expect_printed(
        b"1 - async function foo() { await 0 }",
        b"1 - async function foo() {\n  await 0;\n};\n",
    );
    expect_printed(
        b"1 - async function() { await 0 }",
        b"1 - async function() {\n  await 0;\n};\n",
    );
    expect_printed(
        b"(async function foo() { await 0 })",
        b"(async function foo() {\n  await 0;\n});\n",
    );
    expect_printed(
        b"(async function() { await 0 })",
        b"(async function() {\n  await 0;\n});\n",
    );
    expect_printed(
        b"(x, async function foo() { await 0 })",
        b"x, async function foo() {\n  await 0;\n};\n",
    );
    expect_printed(
        b"(x, async function() { await 0 })",
        b"x, async function() {\n  await 0;\n};\n",
    );
    expect_printed(
        b"new async function() { await 0 }",
        b"new async function() {\n  await 0;\n}();\n",
    );
    expect_printed(
        b"new async function() { await 0 }.x",
        b"new async function() {\n  await 0;\n}.x();\n",
    );
    expect_printed(b"async", b"async;\n");
    expect_printed(b"async + 1", b"async + 1;\n");
    expect_printed(b"async => {}", b"(async) => {\n};\n");
    expect_printed(b"(async, 1)", b"async, 1;\n");
    expect_printed(b"(async, x) => {}", b"(async, x) => {\n};\n");
    expect_printed(b"async ()", b"async();\n");
    expect_printed(b"async (x)", b"async(x);\n");
    expect_printed(b"async (...x)", b"async(...x);\n");
    expect_printed(b"async (...x, ...y)", b"async(...x, ...y);\n");
    expect_printed(b"async () => {}", b"async () => {\n};\n");
    expect_printed(b"async x => {}", b"async (x) => {\n};\n");
    expect_printed(b"async (x) => {}", b"async (x) => {\n};\n");
    expect_printed(b"async (...x) => {}", b"async (...x) => {\n};\n");
    expect_printed(b"async x => await 0", b"async (x) => await 0;\n");
    expect_printed(b"async () => await 0", b"async () => await 0;\n");
    expect_printed(b"new async()", b"new async();\n");
    expect_printed(b"new async().x", b"new async().x;\n");
    expect_printed(b"new (async())", b"new (async())();\n");
    expect_printed(b"new (async().x)", b"new (async()).x();\n");
    expect_printed(b"(async x => y), z", b"(async (x) => y), z;\n");
    expect_printed(b"(async x => y, z)", b"async (x) => y, z;\n");
    expect_printed(b"(async x => (y, z))", b"(async (x) => (y, z));\n");
    expect_printed(b"(async (x) => y), z", b"(async (x) => y), z;\n");
    expect_printed(b"(async (x) => y, z)", b"async (x) => y, z;\n");
    expect_printed(b"(async (x) => (y, z))", b"(async (x) => (y, z));\n");
    expect_printed(b"async x => y, z", b"async (x) => y, z;\n");
    expect_printed(b"async x => (y, z)", b"async (x) => (y, z);\n");
    expect_printed(b"async (x) => y, z", b"async (x) => y, z;\n");
    expect_printed(b"async (x) => (y, z)", b"async (x) => (y, z);\n");
    expect_printed(
        b"export default async x => (y, z)",
        b"export default async (x) => (y, z);\n",
    );
    expect_printed(
        b"export default async (x) => (y, z)",
        b"export default async (x) => (y, z);\n",
    );
    expect_printed(
        b"class Foo { async async() {} }",
        b"class Foo {\n  async async() {\n  }\n}\n",
    );
    expect_printed(
        b"(class { async async() {} })",
        b"(class {\n  async async() {\n  }\n});\n",
    );
    expect_printed(b"({ async async() {} })", b"({ async async() {\n} });\n");
    expect_printed(
        b"async function foo() { (x = await y) }",
        b"async function foo() {\n  x = await y;\n}\n",
    );
    expect_printed(
        b"(class { async \n foo() {} })",
        b"(class {\n  async;\n  foo() {\n  }\n});\n",
    );
    expect_printed(
        b"(class { async \n *foo() {} })",
        b"(class {\n  async;\n  *foo() {\n  }\n});\n",
    );
    expect_printed(b"await foo;", b"await foo;\n");
    expect_printed(b"for await(foo of bar);", b"for await (foo of bar) ;\n");
    expect_printed_script(
        b"function foo(x = await) {}",
        b"function foo(x = await) {\n}\n",
    );
    expect_printed_script(
        b"(function(x = await) {})",
        b"(function(x = await) {\n});\n",
    );
    expect_printed_script(b"({ foo(x = await) {} })", b"({ foo(x = await) {\n} });\n");
    expect_printed_script(
        b"class Foo { foo(x = await) {} }",
        b"class Foo {\n  foo(x = await) {\n  }\n}\n",
    );
    expect_printed_script(
        b"(class { foo(x = await) {} })",
        b"(class {\n  foo(x = await) {\n  }\n});\n",
    );
    expect_printed(b"(x = await y)", b"x = await y;\n");
    expect_printed(b"async(x = await y)", b"async(x = await y);\n");
    expect_printed(b"\\u0061sync", b"async;\n");
    expect_printed(b"(\\u0061sync)", b"async;\n");
    expect_printed_script(
        b"function foo() { \\u0061wait }",
        b"function foo() {\n  await;\n}\n",
    );
    expect_printed_script(
        b"function foo() { var \\u0061wait }",
        b"function foo() {\n  var await;\n}\n",
    );
    expect_printed(
        b"async function foo(){for await(x of y);}",
        b"async function foo() {\n  for await (x of y) ;\n}\n",
    );
    expect_printed(
        b"async function foo(){for await(let x of y);}",
        b"async function foo() {\n  for await (let x of y) ;\n}\n",
    );
    expect_printed_script(b"(function await() {})", b"(function await() {\n});\n");
    expect_printed_script(
        b"function foo() { ({await} = x) }",
        b"function foo() {\n  ({ await } = x);\n}\n",
    );
    expect_printed_script(
        b"function foo() { let x = {await} }",
        b"function foo() {\n  let x = { await };\n}\n",
    );
    expect_printed(b"({ async await() {} })", b"({ async await() {\n} });\n");
    expect_printed(
        b"(class { async await() {} })",
        b"(class {\n  async await() {\n  }\n});\n",
    );
    expect_printed(
        b"class Foo { async await() {} }",
        b"class Foo {\n  async await() {\n  }\n}\n",
    );
    expect_printed(b"({ async *await() {} })", b"({ async *await() {\n} });\n");
    expect_printed(
        b"(class { async *await() {} })",
        b"(class {\n  async *await() {\n  }\n});\n",
    );
    expect_printed(
        b"class Foo { async *await() {} }",
        b"class Foo {\n  async *await() {\n  }\n}\n",
    );
}

#[test]
fn test_labels() {
    expect_printed(b"{a:b}", b"{\n  a: b;\n}\n");
    expect_printed(b"({a:b})", b"({ a: b });\n");
    expect_printed(b"x: y: z: 1", b"x: y: z: 1;\n");
    expect_printed(b"x: 1; y: 2; x: 3", b"x: 1;\ny: 2;\nx: 3;\n");
    expect_printed(b"x: (() => { x: 1; })()", b"x: (() => {\n  x: 1;\n})();\n");
    expect_printed(
        b"x: ({ f() { x: 1; } }).f()",
        b"x: ({ f() {\n  x: 1;\n} }).f();\n",
    );
    expect_printed(
        b"x: (function() { x: 1; })()",
        b"x: (function() {\n  x: 1;\n})();\n",
    );
    expect_printed(b"x: break x", b"x: break x;\n");
    expect_printed(b"x: { break x; foo() }", b"x: {\n  break x;\n  foo();\n}\n");
    expect_printed(
        b"x: { y: { z: { foo(); break x; } } }",
        b"x: {\n  y: {\n    z: {\n      foo();\n      break x;\n    }\n  }\n}\n",
    );
    expect_printed(
        b"x: { class X { static { new X } } }",
        b"x: {\n  class X {\n    static {\n      new X();\n    }\n  }\n}\n",
    );
}

#[test]
#[ignore = "depends on esbuild parser purity metadata"]
fn test_arrow() {
    expect_printed(b"x = ([ y = [ (z) ] ]) => 0", b"x = ([y = [z]]) => 0;\n");
    expect_printed(
        b"x = ([ y = [ ...(z) ] ]) => 0",
        b"x = ([y = [...z]]) => 0;\n",
    );
    expect_printed(
        b"x = ({ y = { y: (z) } }) => 0",
        b"x = ({ y = { y: z } }) => 0;\n",
    );
    expect_printed(
        b"x = ({ y = { ...(y) } }) => 0",
        b"x = ({ y = { ...y } }) => 0;\n",
    );
    expect_printed(b"x => function() {}", b"(x) => function() {\n};\n");
    expect_printed(b"(x) => function() {}", b"(x) => function() {\n};\n");
    expect_printed(b"(x => function() {})", b"((x) => function() {\n});\n");
    expect_printed(b"(x = () => {}) => {}", b"(x = () => {\n}) => {\n};\n");
    expect_printed(
        b"async (x = () => {}) => {}",
        b"async (x = () => {\n}) => {\n};\n",
    );
    expect_printed(b"(() => {}) ? a : b", b"(() => {\n}) ? a : b;\n");
    expect_printed(b"1 < (() => {})", b"1 < (() => {\n});\n");
    expect_printed(b"y = x => {}", b"y = (x) => {\n};\n");
    expect_printed(b"y = () => {}", b"y = () => {\n};\n");
    expect_printed(b"y = (x) => {}", b"y = (x) => {\n};\n");
    expect_printed(b"y = async x => {}", b"y = async (x) => {\n};\n");
    expect_printed(b"y = async () => {}", b"y = async () => {\n};\n");
    expect_printed(b"y = async (x) => {}", b"y = async (x) => {\n};\n");
    expect_printed(b"1 + function () {}", b"1 + function() {\n};\n");
    expect_printed(b"1 + async function () {}", b"1 + async function() {\n};\n");
    expect_printed(
        b"class Foo extends function () {} {}",
        b"class Foo extends function() {\n} {\n}\n",
    );
    expect_printed(
        b"class Foo extends async function () {} {}",
        b"class Foo extends async function() {\n} {\n}\n",
    );
    expect_printed(b"() => {}\n(0)", b"() => {\n};\n0;\n");
    expect_printed(b"x => {}\n(0)", b"(x) => {\n};\n0;\n");
    expect_printed(b"async () => {}\n(0)", b"async () => {\n};\n0;\n");
    expect_printed(b"async x => {}\n(0)", b"async (x) => {\n};\n0;\n");
    expect_printed(b"async (x) => {}\n(0)", b"async (x) => {\n};\n0;\n");
    expect_printed(b"() => {}\n,0", b"() => {\n}, 0;\n");
    expect_printed(b"x => {}\n,0", b"(x) => {\n}, 0;\n");
    expect_printed(b"async () => {}\n,0", b"async () => {\n}, 0;\n");
    expect_printed(b"async x => {}\n,0", b"async (x) => {\n}, 0;\n");
    expect_printed(b"async (x) => {}\n,0", b"async (x) => {\n}, 0;\n");
    expect_printed(b"(() => {})\n(0)", b"/* @__PURE__ */ (() => {\n})(0);\n");
    expect_printed(b"(x => {})\n(0)", b"/* @__PURE__ */ ((x) => {\n})(0);\n");
    expect_printed(b"(async () => {})\n(0)", b"(async () => {\n})(0);\n");
    expect_printed(b"(async x => {})\n(0)", b"(async (x) => {\n})(0);\n");
    expect_printed(b"(async (x) => {})\n(0)", b"(async (x) => {\n})(0);\n");
    expect_printed(b"y = () => {}\n(0)", b"y = () => {\n};\n0;\n");
    expect_printed(b"y = x => {}\n(0)", b"y = (x) => {\n};\n0;\n");
    expect_printed(b"y = async () => {}\n(0)", b"y = async () => {\n};\n0;\n");
    expect_printed(b"y = async x => {}\n(0)", b"y = async (x) => {\n};\n0;\n");
    expect_printed(b"y = async (x) => {}\n(0)", b"y = async (x) => {\n};\n0;\n");
    expect_printed(b"y = () => {}\n,0", b"y = () => {\n}, 0;\n");
    expect_printed(b"y = x => {}\n,0", b"y = (x) => {\n}, 0;\n");
    expect_printed(b"y = async () => {}\n,0", b"y = async () => {\n}, 0;\n");
    expect_printed(b"y = async x => {}\n,0", b"y = async (x) => {\n}, 0;\n");
    expect_printed(b"y = async (x) => {}\n,0", b"y = async (x) => {\n}, 0;\n");
    expect_printed(
        b"y = (() => {})\n(0)",
        b"y = /* @__PURE__ */ (() => {\n})(0);\n",
    );
    expect_printed(
        b"y = (x => {})\n(0)",
        b"y = /* @__PURE__ */ ((x) => {\n})(0);\n",
    );
    expect_printed(
        b"y = (async () => {})\n(0)",
        b"y = (async () => {\n})(0);\n",
    );
    expect_printed(
        b"y = (async x => {})\n(0)",
        b"y = (async (x) => {\n})(0);\n",
    );
    expect_printed(
        b"y = (async (x) => {})\n(0)",
        b"y = (async (x) => {\n})(0);\n",
    );
    expect_printed(b"(() => {}\n,0)", b"() => {\n}, 0;\n");
    expect_printed(b"(x => {}\n,0)", b"(x) => {\n}, 0;\n");
    expect_printed(b"(async () => {}\n,0)", b"async () => {\n}, 0;\n");
    expect_printed(b"(async x => {}\n,0)", b"async (x) => {\n}, 0;\n");
    expect_printed(b"(async (x) => {}\n,0)", b"async (x) => {\n}, 0;\n");
    expect_printed(b"((() => {})\n(0))", b"/* @__PURE__ */ (() => {\n})(0);\n");
    expect_printed(b"((x => {})\n(0))", b"/* @__PURE__ */ ((x) => {\n})(0);\n");
    expect_printed(b"((async () => {})\n(0))", b"(async () => {\n})(0);\n");
    expect_printed(b"((async x => {})\n(0))", b"(async (x) => {\n})(0);\n");
    expect_printed(b"((async (x) => {})\n(0))", b"(async (x) => {\n})(0);\n");
    expect_printed(b"y = (() => {}\n,0)", b"y = (() => {\n}, 0);\n");
    expect_printed(b"y = (x => {}\n,0)", b"y = ((x) => {\n}, 0);\n");
    expect_printed(b"y = (async () => {}\n,0)", b"y = (async () => {\n}, 0);\n");
    expect_printed(b"y = (async x => {}\n,0)", b"y = (async (x) => {\n}, 0);\n");
    expect_printed(
        b"y = (async (x) => {}\n,0)",
        b"y = (async (x) => {\n}, 0);\n",
    );
    expect_printed(
        b"y = ((() => {})\n(0))",
        b"y = /* @__PURE__ */ (() => {\n})(0);\n",
    );
    expect_printed(
        b"y = ((x => {})\n(0))",
        b"y = /* @__PURE__ */ ((x) => {\n})(0);\n",
    );
    expect_printed(
        b"y = ((async () => {})\n(0))",
        b"y = (async () => {\n})(0);\n",
    );
    expect_printed(
        b"y = ((async x => {})\n(0))",
        b"y = (async (x) => {\n})(0);\n",
    );
    expect_printed(
        b"y = ((async (x) => {})\n(0))",
        b"y = (async (x) => {\n})(0);\n",
    );
}

#[test]
#[ignore = "depends on esbuild tagged-template newline normalization"]
fn test_template() {
    expect_printed(b"`\\0`", b"`\\0`;\n");
    expect_printed(b"`${'\\00'}`", b"`${\"\\0\"}`;\n");
    expect_printed(b"tag`\\7`", b"tag`\\7`;\n");
    expect_printed(b"tag`\\8`", b"tag`\\8`;\n");
    expect_printed(b"tag`\\9`", b"tag`\\9`;\n");
    expect_printed(b"tag`\\00`", b"tag`\\00`;\n");
    expect_printed(b"tag`\\00${x}`", b"tag`\\00${x}`;\n");
    expect_printed(b"tag`${x}\\00`", b"tag`${x}\\00`;\n");
    expect_printed(b"tag`${x}\\00${y}`", b"tag`${x}\\00${y}`;\n");
    expect_printed(b"tag`\\unicode`", b"tag`\\unicode`;\n");
    expect_printed(b"tag`\\unicode${x}`", b"tag`\\unicode${x}`;\n");
    expect_printed(b"tag`${x}\\unicode`", b"tag`${x}\\unicode`;\n");
    expect_printed(b"tag`\\u{10FFFFF}`", b"tag`\\u{10FFFFF}`;\n");
    expect_printed(b"new foo`bar`()", b"new foo`bar`();\n");
    expect_printed(b"new (foo`bar`)()", b"new foo`bar`();\n");
    expect_printed(b"(new foo)`bar`()", b"new foo()`bar`();\n");
    expect_printed(b"new foo()`bar`()", b"new foo()`bar`();\n");
    expect_printed(b"(new foo())`bar`()", b"new foo()`bar`();\n");
    expect_printed(b"new (foo()`bar`)()", b"new (foo())`bar`();\n");
    expect_printed(b"tag``", b"tag``;\n");
    expect_printed(b"(a?.b)``", b"(a?.b)``;\n");
    expect_printed(b"(a?.(b))``", b"(a?.(b))``;\n");
    expect_printed(b"(a?.[b])``", b"(a?.[b])``;\n");
    expect_printed(b"(a?.b.c)``", b"(a?.b.c)``;\n");
    expect_printed(b"(a?.(b).c)``", b"(a?.(b).c)``;\n");
    expect_printed(b"(a?.[b].c)``", b"(a?.[b].c)``;\n");
    // Depends on esbuild's parser-stage template constant folding.
    // expect_printed(b"`a${1 + `b${2}c` + 3}d`", b"`a${`1b${2}c3`}d`;\n");
    expect_printed(b"`a\nb`", b"`a\nb`;\n");
    expect_printed(b"`a\rb`", b"`a\nb`;\n");
    expect_printed(b"`a\r\nb`", b"`a\nb`;\n");
    expect_printed(b"`a\\nb`", b"`a\nb`;\n");
    expect_printed(b"`a\\rb`", b"`a\\rb`;\n");
    expect_printed(b"`a\\r\\nb`", b"`a\\r\nb`;\n");
    expect_printed(b"`a\xe2\x80\xa8b`", b"`a\\u2028b`;\n");
    expect_printed(b"`a\xe2\x80\xa9b`", b"`a\\u2029b`;\n");
    expect_printed(b"`a\n${b}`", b"`a\n${b}`;\n");
    expect_printed(b"`a\r${b}`", b"`a\n${b}`;\n");
    expect_printed(b"`a\r\n${b}`", b"`a\n${b}`;\n");
    expect_printed(b"`a\\n${b}`", b"`a\n${b}`;\n");
    expect_printed(b"`a\\r${b}`", b"`a\\r${b}`;\n");
    expect_printed(b"`a\\r\\n${b}`", b"`a\\r\n${b}`;\n");
    expect_printed(b"`a\xe2\x80\xa8${b}`", b"`a\\u2028${b}`;\n");
    expect_printed(b"`a\xe2\x80\xa9${b}`", b"`a\\u2029${b}`;\n");
    expect_printed(b"`${a}\nb`", b"`${a}\nb`;\n");
    expect_printed(b"`${a}\rb`", b"`${a}\nb`;\n");
    expect_printed(b"`${a}\r\nb`", b"`${a}\nb`;\n");
    expect_printed(b"`${a}\\nb`", b"`${a}\nb`;\n");
    expect_printed(b"`${a}\\rb`", b"`${a}\\rb`;\n");
    expect_printed(b"`${a}\\r\\nb`", b"`${a}\\r\nb`;\n");
    expect_printed(b"`${a}\xe2\x80\xa8b`", b"`${a}\\u2028b`;\n");
    expect_printed(b"`${a}\xe2\x80\xa9b`", b"`${a}\\u2029b`;\n");
    expect_printed(b"tag`a\nb`", b"tag`a\nb`;\n");
    expect_printed(b"tag`a\rb`", b"tag`a\nb`;\n");
    expect_printed(b"tag`a\r\nb`", b"tag`a\nb`;\n");
    expect_printed(b"tag`a\\nb`", b"tag`a\\nb`;\n");
    expect_printed(b"tag`a\\rb`", b"tag`a\\rb`;\n");
    expect_printed(b"tag`a\\r\\nb`", b"tag`a\\r\\nb`;\n");
    expect_printed(b"tag`a\xe2\x80\xa8b`", b"tag`a\xe2\x80\xa8b`;\n");
    expect_printed(b"tag`a\xe2\x80\xa9b`", b"tag`a\xe2\x80\xa9b`;\n");
    expect_printed(b"tag`a\n${b}`", b"tag`a\n${b}`;\n");
    expect_printed(b"tag`a\r${b}`", b"tag`a\n${b}`;\n");
    expect_printed(b"tag`a\r\n${b}`", b"tag`a\n${b}`;\n");
    expect_printed(b"tag`a\\n${b}`", b"tag`a\\n${b}`;\n");
    expect_printed(b"tag`a\\r${b}`", b"tag`a\\r${b}`;\n");
    expect_printed(b"tag`a\\r\\n${b}`", b"tag`a\\r\\n${b}`;\n");
    expect_printed(b"tag`a\xe2\x80\xa8${b}`", b"tag`a\xe2\x80\xa8${b}`;\n");
    expect_printed(b"tag`a\xe2\x80\xa9${b}`", b"tag`a\xe2\x80\xa9${b}`;\n");
    expect_printed(b"tag`${a}\nb`", b"tag`${a}\nb`;\n");
    expect_printed(b"tag`${a}\rb`", b"tag`${a}\nb`;\n");
    expect_printed(b"tag`${a}\r\nb`", b"tag`${a}\nb`;\n");
    expect_printed(b"tag`${a}\\nb`", b"tag`${a}\\nb`;\n");
    expect_printed(b"tag`${a}\\rb`", b"tag`${a}\\rb`;\n");
    expect_printed(b"tag`${a}\\r\\nb`", b"tag`${a}\\r\\nb`;\n");
    expect_printed(b"tag`${a}\xe2\x80\xa8b`", b"tag`${a}\xe2\x80\xa8b`;\n");
    expect_printed(b"tag`${a}\xe2\x80\xa9b`", b"tag`${a}\xe2\x80\xa9b`;\n");
}

#[test]
fn test_switch() {
    expect_printed(b"switch (x) { default: }", b"switch (x) {\n  default:\n}\n");
    expect_printed(
        b"switch ((x => x + 1)(0)) { case 1: var y } y = 2",
        b"switch (((x) => x + 1)(0)) {\n  case 1:\n    var y;\n}\ny = 2;\n",
    );
    expect_printed(
        b"switch (0) { case x(() => 1): y = () => 2; case x(() => 3): y = () => 4 }",
        b"switch (0) {\n  case x(() => 1):\n    y = () => 2;\n  case x(() => 3):\n    y = () => 4;\n}\n",
    );
    expect_printed(
        b"switch (x) { case 0: { using y = z } }\n",
        b"switch (x) {\n  case 0: {\n    using y = z;\n  }\n}\n",
    );
    expect_printed(
        b"switch (x) { case 0: { await using y = z } }\n",
        b"switch (x) {\n  case 0: {\n    await using y = z;\n  }\n}\n",
    );
}

#[test]
#[ignore = "depends on esbuild import phase metadata"]
fn test_import() {
    expect_printed(b"import \"foo\"", b"import \"foo\";\n");
    expect_printed(b"import {} from \"foo\"", b"import {} from \"foo\";\n");
    expect_printed(
        b"import {x} from \"foo\";x",
        b"import { x } from \"foo\";\nx;\n",
    );
    expect_printed(
        b"import {x as y} from \"foo\";y",
        b"import { x as y } from \"foo\";\ny;\n",
    );
    expect_printed(
        b"import {x as y, z} from \"foo\";y;z",
        b"import { x as y, z } from \"foo\";\ny;\nz;\n",
    );
    expect_printed(
        b"import {x as y, z,} from \"foo\";y;z",
        b"import { x as y, z } from \"foo\";\ny;\nz;\n",
    );
    expect_printed(
        b"import z, {x as y} from \"foo\";y;z",
        b"import z, { x as y } from \"foo\";\ny;\nz;\n",
    );
    expect_printed(b"import z from \"foo\";z", b"import z from \"foo\";\nz;\n");
    expect_printed(
        b"import * as ns from \"foo\";ns;ns.x",
        b"import * as ns from \"foo\";\nns;\nns.x;\n",
    );
    expect_printed(
        b"import z, * as ns from \"foo\";z;ns;ns.x",
        b"import z, * as ns from \"foo\";\nz;\nns;\nns.x;\n",
    );
    expect_printed(b"import('foo')", b"import(\"foo\");\n");
    expect_printed(b"(import('foo'))", b"import(\"foo\");\n");
    expect_printed(b"{import('foo')}", b"{\n  import(\"foo\");\n}\n");
    expect_printed(
        b"import('foo').then(() => {})",
        b"import(\"foo\").then(() => {\n});\n",
    );
    expect_printed(b"new import.meta", b"new import.meta();\n");
    expect_printed(b"new (import('foo'))", b"new (import(\"foo\"))();\n");
    expect_printed(b"import.meta", b"import.meta;\n");
    expect_printed(b"(import.meta)", b"import.meta;\n");
    expect_printed(b"{import.meta}", b"{\n  import.meta;\n}\n");
    expect_printed(
        b"import x from \"foo\"; x = 1",
        b"import x from \"foo\";\nx = 1;\n",
    );
    expect_printed(
        b"import x from \"foo\"; x++",
        b"import x from \"foo\";\nx++;\n",
    );
    expect_printed(
        b"import x from \"foo\"; ([x] = 1)",
        b"import x from \"foo\";\n[x] = 1;\n",
    );
    expect_printed(
        b"import x from \"foo\"; ({x} = 1)",
        b"import x from \"foo\";\n({ x } = 1);\n",
    );
    expect_printed(
        b"import x from \"foo\"; ({y: x} = 1)",
        b"import x from \"foo\";\n({ y: x } = 1);\n",
    );
    expect_printed(
        b"import {x} from \"foo\"; x++",
        b"import { x } from \"foo\";\nx++;\n",
    );
    expect_printed(
        b"import * as x from \"foo\"; x++",
        b"import * as x from \"foo\";\nx++;\n",
    );
    expect_printed(
        b"import * as x from \"foo\"; x.y = 1",
        b"import * as x from \"foo\";\nx.y = 1;\n",
    );
    expect_printed(
        b"import * as x from \"foo\"; x[y] = 1",
        b"import * as x from \"foo\";\nx[y] = 1;\n",
    );
    expect_printed(
        b"import * as x from \"foo\"; x['y'] = 1",
        b"import * as x from \"foo\";\nx[\"y\"] = 1;\n",
    );
    expect_printed(
        b"import * as x from \"foo\"; x['y z'] = 1",
        b"import * as x from \"foo\";\nx[\"y z\"] = 1;\n",
    );
    expect_printed(
        b"import x from \"foo\"; ({y = x} = 1)",
        b"import x from \"foo\";\n({ y = x } = 1);\n",
    );
    expect_printed(
        b"import x from \"foo\"; ({[x]: y} = 1)",
        b"import x from \"foo\";\n({ [x]: y } = 1);\n",
    );
    expect_printed(
        b"import x from \"foo\"; x.y = 1",
        b"import x from \"foo\";\nx.y = 1;\n",
    );
    expect_printed(
        b"import x from \"foo\"; x[y] = 1",
        b"import x from \"foo\";\nx[y] = 1;\n",
    );
    expect_printed(
        b"import x from \"foo\"; x['y'] = 1",
        b"import x from \"foo\";\nx[\"y\"] = 1;\n",
    );
    expect_printed(
        b"import {eval as x} from 'foo'",
        b"import { eval as x } from \"foo\";\n",
    );
    expect_printed(
        b"import {ev\\u0061l as x} from 'foo'",
        b"import { eval as x } from \"foo\";\n",
    );
    expect_printed(
        b"import {arguments as x} from 'foo'",
        b"import { arguments as x } from \"foo\";\n",
    );
    expect_printed(
        b"import {\\u0061rguments as x} from 'foo'",
        b"import { arguments as x } from \"foo\";\n",
    );
    expect_printed(
        b"import {'' as x} from 'foo'",
        b"import { \"\" as x } from \"foo\";\n",
    );
    expect_printed(
        b"import {'\xf0\x9f\x8d\x95' as x} from 'foo'",
        b"import { \"\xf0\x9f\x8d\x95\" as x } from \"foo\";\n",
    );
    expect_printed(
        b"import {'a b' as x} from 'foo'",
        b"import { \"a b\" as x } from \"foo\";\n",
    );
    expect_printed(
        b"import {'\\uD800\\uDC00' as x} from 'foo'",
        b"import { \xf0\x90\x80\x80 as x } from \"foo\";\n",
    );
    expect_printed(b"import defer from 'bar'", b"import defer from \"bar\";\n");
    expect_printed(
        b"import defer, { foo } from 'bar'",
        b"import defer, { foo } from \"bar\";\n",
    );
    expect_printed(
        b"import defer * as foo from 'bar'",
        b"import defer * as foo from \"bar\";\n",
    );
    expect_printed(b"import.defer('foo')", b"import.defer(\"foo\");\n");
    expect_printed(
        b"import source from 'bar'",
        b"import source from \"bar\";\n",
    );
    expect_printed(
        b"import source, { foo } from 'bar'",
        b"import source, { foo } from \"bar\";\n",
    );
    expect_printed(
        b"import source foo from 'bar'",
        b"import source foo from \"bar\";\n",
    );
    expect_printed(
        b"import source from from 'bar'",
        b"import source from from \"bar\";\n",
    );
    expect_printed(
        b"import source source from 'bar'",
        b"import source source from \"bar\";\n",
    );
    expect_printed(b"import.source('foo')", b"import.source(\"foo\");\n");
}

#[test]
fn test_export() {
    expect_printed(b"export default x", b"export default x;\n");
    expect_printed(b"export class x {}", b"export class x {\n}\n");
    expect_printed(b"export function x() {}", b"export function x() {\n}\n");
    expect_printed(
        b"export async function x() {}",
        b"export async function x() {\n}\n",
    );
    expect_printed(b"export var x, y", b"export var x, y;\n");
    expect_printed(b"export let x, y", b"export let x, y;\n");
    expect_printed(
        b"export const x = 0, y = 1",
        b"export const x = 0, y = 1;\n",
    );
    expect_printed(b"export * from \"foo\"", b"export * from \"foo\";\n");
    expect_printed(
        b"export * as ns from \"foo\"",
        b"export * as ns from \"foo\";\n",
    );
    expect_printed(
        b"export * as if from \"foo\"",
        b"export * as if from \"foo\";\n",
    );
    expect_printed(b"let x; export {x}", b"let x;\nexport { x };\n");
    expect_printed(b"let x; export {x as y}", b"let x;\nexport { x as y };\n");
    expect_printed(
        b"let x, z; export {x as y, z}",
        b"let x, z;\nexport { x as y, z };\n",
    );
    expect_printed(
        b"let x, z; export {x as y, z,}",
        b"let x, z;\nexport { x as y, z };\n",
    );
    expect_printed(
        b"let x; export {x} from \"foo\"",
        b"let x;\nexport { x } from \"foo\";\n",
    );
    expect_printed(
        b"let x; export {x as y} from \"foo\"",
        b"let x;\nexport { x as y } from \"foo\";\n",
    );
    expect_printed(
        b"let x, z; export {x as y, z} from \"foo\"",
        b"let x, z;\nexport { x as y, z } from \"foo\";\n",
    );
    expect_printed(
        b"let x, z; export {x as y, z,} from \"foo\"",
        b"let x, z;\nexport { x as y, z } from \"foo\";\n",
    );
    expect_printed(
        b"let x; export {x as ''}",
        b"let x;\nexport { x as \"\" };\n",
    );
    expect_printed(
        b"let x; export {x as '\xf0\x9f\x8d\x95'}",
        b"let x;\nexport { x as \"\xf0\x9f\x8d\x95\" };\n",
    );
    expect_printed(
        b"let x; export {x as 'a b'}",
        b"let x;\nexport { x as \"a b\" };\n",
    );
    expect_printed(
        b"let x; export {x as '\\uD800\\uDC00'}",
        b"let x;\nexport { x as \xf0\x90\x80\x80 };\n",
    );
    expect_printed(
        b"export {'' as x} from 'foo'",
        b"export { \"\" as x } from \"foo\";\n",
    );
    expect_printed(
        b"export {'\xf0\x9f\x8d\x95' as x} from 'foo'",
        b"export { \"\xf0\x9f\x8d\x95\" as x } from \"foo\";\n",
    );
    expect_printed(
        b"export {'a b' as x} from 'foo'",
        b"export { \"a b\" as x } from \"foo\";\n",
    );
    expect_printed(
        b"export {'\\uD800\\uDC00' as x} from 'foo'",
        b"export { \xf0\x90\x80\x80 as x } from \"foo\";\n",
    );
    expect_printed(
        b"export {x as ''} from 'foo'",
        b"export { x as \"\" } from \"foo\";\n",
    );
    expect_printed(
        b"export {x as '\xf0\x9f\x8d\x95'} from 'foo'",
        b"export { x as \"\xf0\x9f\x8d\x95\" } from \"foo\";\n",
    );
    expect_printed(
        b"export {x as 'a b'} from 'foo'",
        b"export { x as \"a b\" } from \"foo\";\n",
    );
    expect_printed(
        b"export {x as '\\uD800\\uDC00'} from 'foo'",
        b"export { x as \xf0\x90\x80\x80 } from \"foo\";\n",
    );
    expect_printed(b"export {'x'} from 'foo'", b"export { x } from \"foo\";\n");
    expect_printed(
        b"export {'a b'} from 'foo'",
        b"export { \"a b\" } from \"foo\";\n",
    );
    expect_printed(
        b"export {'x' as 'y'} from 'foo'",
        b"export { x as y } from \"foo\";\n",
    );
    expect_printed(
        b"export {'a b' as 'c d'} from 'foo'",
        b"export { \"a b\" as \"c d\" } from \"foo\";\n",
    );
    expect_printed(
        b"export * as '' from 'foo'",
        b"export * as \"\" from \"foo\";\n",
    );
    expect_printed(
        b"export * as '\xf0\x9f\x8d\x95' from 'foo'",
        b"export * as \"\xf0\x9f\x8d\x95\" from \"foo\";\n",
    );
    expect_printed(
        b"export * as 'a b' from 'foo'",
        b"export * as \"a b\" from \"foo\";\n",
    );
    expect_printed(
        b"export * as '\\uD800\\uDC00' from 'foo'",
        b"export * as \xf0\x90\x80\x80 from \"foo\";\n",
    );
}

#[test]
fn test_export_duplicates() {
    expect_printed(b"export {x};let x", b"export { x };\nlet x;\n");
    expect_printed(
        b"export {x, x as y};let x",
        b"export { x, x as y };\nlet x;\n",
    );
    expect_printed(
        b"export {x};export {x as y} from 'foo';let x",
        b"export { x };\nexport { x as y } from \"foo\";\nlet x;\n",
    );
    expect_printed(
        b"export {x};export default function x() {}",
        b"export { x };\nexport default function x() {\n}\n",
    );
    expect_printed(
        b"export {x};export default class x {}",
        b"export { x };\nexport default class x {\n}\n",
    );
}

#[test]
fn test_export_default() {
    expect_printed(b"export default (1, 2)", b"export default (1, 2);\n");
    expect_printed(b"export default async", b"export default async;\n");
    expect_printed(b"export default async()", b"export default async();\n");
    expect_printed(b"export default async + 1", b"export default async + 1;\n");
    expect_printed(
        b"export default async => {}",
        b"export default (async) => {\n};\n",
    );
    expect_printed(
        b"export default async x => {}",
        b"export default async (x) => {\n};\n",
    );
    expect_printed(
        b"export default async () => {}",
        b"export default async () => {\n};\n",
    );
    expect_printed(
        b"export default function() {} - after",
        b"export default function() {\n}\n-after;\n",
    );
    expect_printed(
        b"export default function*() {} - after",
        b"export default function* () {\n}\n-after;\n",
    );
    expect_printed(
        b"export default function foo() {} - after",
        b"export default function foo() {\n}\n-after;\n",
    );
    expect_printed(
        b"export default function* foo() {} - after",
        b"export default function* foo() {\n}\n-after;\n",
    );
    expect_printed(
        b"export default async function() {} - after",
        b"export default async function() {\n}\n-after;\n",
    );
    expect_printed(
        b"export default async function*() {} - after",
        b"export default async function* () {\n}\n-after;\n",
    );
    expect_printed(
        b"export default async function foo() {} - after",
        b"export default async function foo() {\n}\n-after;\n",
    );
    expect_printed(
        b"export default async function* foo() {} - after",
        b"export default async function* foo() {\n}\n-after;\n",
    );
    expect_printed(
        b"export default class {} - after",
        b"export default class {\n}\n-after;\n",
    );
    expect_printed(
        b"export default class Foo {} - after",
        b"export default class Foo {\n}\n-after;\n",
    );
    expect_printed(
        b"export default abstract\nclass Foo {}",
        b"export default abstract;\nclass Foo {\n}\n",
    );
}

#[test]
fn test_export_clause() {
    expect_printed(b"export {x, y};let x, y", b"export { x, y };\nlet x, y;\n");
    expect_printed(
        b"export {x, y as z,};let x, y",
        b"export { x, y as z };\nlet x, y;\n",
    );
    expect_printed(
        b"export {x, y} from 'path'",
        b"export { x, y } from \"path\";\n",
    );
    expect_printed(
        b"export {default, if} from 'path'",
        b"export { default, if } from \"path\";\n",
    );
    expect_printed(
        b"export {default as foo, if as bar} from 'path'",
        b"export { default as foo, if as bar } from \"path\";\n",
    );
}

#[test]
#[ignore = "depends on esbuild parser lowering"]
fn test_catch() {
    expect_printed(b"try {} catch (e) {}", b"try {\n} catch (e) {\n}\n");
    expect_printed(
        b"try {} catch (e) { var e }",
        b"try {\n} catch (e) {\n  var e;\n}\n",
    );
    expect_printed(
        b"var e; try {} catch (e) {}",
        b"var e;\ntry {\n} catch (e) {\n}\n",
    );
    expect_printed(
        b"let e; try {} catch (e) {}",
        b"let e;\ntry {\n} catch (e) {\n}\n",
    );
    expect_printed(
        b"try { var e } catch (e) {}",
        b"try {\n  var e;\n} catch (e) {\n}\n",
    );
    expect_printed(
        b"try { function e() {} } catch (e) {}",
        b"try {\n  let e = function() {\n  };\n  var e = e;\n} catch (e) {\n}\n",
    );
    expect_printed(
        b"try {} catch (e) { { function e() {} } }",
        b"try {\n} catch (e) {\n  {\n    let e = function() {\n    };\n    var e = e;\n  }\n}\n",
    );
    expect_printed(
        b"try {} catch (e) { if (1) function e() {} }",
        b"try {\n} catch (e) {\n  if (1) {\n    let e = function() {\n    };\n    var e = e;\n  }\n}\n",
    );
    expect_printed(
        b"try {} catch (e) { if (0) ; else function e() {} }",
        b"try {\n} catch (e) {\n  if (0) ;\n  else {\n    let e = function() {\n    };\n    var e = e;\n  }\n}\n",
    );
    expect_printed(
        b"try {} catch ({ e }) { { function e() {} } }",
        b"try {\n} catch ({ e }) {\n  {\n    let e = function() {\n    };\n    var e = e;\n  }\n}\n",
    );
}

#[test]
#[ignore = "depends on esbuild comment metadata"]
fn test_preserved_comments() {
    expect_printed(b"//", b"");
    expect_printed(b"//preserve", b"");
    expect_printed(b"//@__PURE__", b"");
    expect_printed(b"//!", b"//!\n");
    expect_printed(b"//@license", b"//@license\n");
    expect_printed(b"//@preserve", b"//@preserve\n");
    expect_printed(b"// @license", b"// @license\n");
    expect_printed(b"// @preserve", b"// @preserve\n");
    expect_printed(b"/**/", b"");
    expect_printed(b"/*preserve*/", b"");
    expect_printed(b"/*@__PURE__*/", b"");
    expect_printed(b"/*!*/", b"/*!*/\n");
    expect_printed(b"/*@license*/", b"/*@license*/\n");
    expect_printed(b"/*@preserve*/", b"/*@preserve*/\n");
    expect_printed(b"/*\n * @license\n */", b"/*\n * @license\n */\n");
    expect_printed(b"/*\n * @preserve\n */", b"/*\n * @preserve\n */\n");
    expect_printed(b"foo() //! test", b"foo();\n//! test\n");
    expect_printed(b"//! test\nfoo()", b"//! test\nfoo();\n");
    expect_printed(b"if (1) //! test\nfoo()", b"if (1)\n  foo();\n");
    expect_printed(
        b"if (1) {//! test\nfoo()}",
        b"if (1) {\n  //! test\n  foo();\n}\n",
    );
    expect_printed(
        b"if (1) {foo() //! test\n}",
        b"if (1) {\n  foo();\n  //! test\n}\n",
    );
    expect_printed(
        b"    /*!\r     * Re-indent test\r     */",
        b"/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"    /*!\n     * Re-indent test\n     */",
        b"/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"    /*!\r\n     * Re-indent test\r\n     */",
        b"/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"    /*!\xe2\x80\xa8     * Re-indent test\xe2\x80\xa8     */",
        b"/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"    /*!\xe2\x80\xa9     * Re-indent test\xe2\x80\xa9     */",
        b"/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"\t\t/*!\r\t\t * Re-indent test\r\t\t */",
        b"/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"\t\t/*!\n\t\t * Re-indent test\n\t\t */",
        b"/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"\t\t/*!\r\n\t\t * Re-indent test\r\n\t\t */",
        b"/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"\t\t/*!\xe2\x80\xa8\t\t * Re-indent test\xe2\x80\xa8\t\t */",
        b"/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"\t\t/*!\xe2\x80\xa9\t\t * Re-indent test\xe2\x80\xa9\t\t */",
        b"/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"x\r    /*!\r     * Re-indent test\r     */",
        b"x;\n/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"x\n    /*!\n     * Re-indent test\n     */",
        b"x;\n/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"x\r\n    /*!\r\n     * Re-indent test\r\n     */",
        b"x;\n/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"x\xe2\x80\xa8    /*!\xe2\x80\xa8     * Re-indent test\xe2\x80\xa8     */",
        b"x;\n/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        b"x\xe2\x80\xa9    /*!\xe2\x80\xa9     * Re-indent test\xe2\x80\xa9     */",
        b"x;\n/*!\n * Re-indent test\n */\n",
    );
}

#[test]
fn test_unicode_whitespace() {
    expect_printed_jsx(b"<x>\n\x08\n</x>", b"<x>\n\x08\n</x>;\n");
    expect_printed_jsx(b"<x>\n\x08&quot;\n</x>", b"<x>\n\x08&quot;\n</x>;\n");
    const WHITESPACE: &[&str] = &[
        "\u{0009}", "\u{000B}", "\u{000C}", "\u{0020}", "\u{00A0}", "\u{1680}", "\u{2000}",
        "\u{2001}", "\u{2002}", "\u{2003}", "\u{2004}", "\u{2005}", "\u{2006}", "\u{2007}",
        "\u{2008}", "\u{2009}", "\u{200A}", "\u{202F}", "\u{205F}", "\u{3000}", "\u{FEFF}",
    ];
    for whitespace in WHITESPACE {
        let source = format!("var{whitespace}x");
        expect_printed(source.as_bytes(), b"var x;\n");
        let source = format!("<x{whitespace}y/>");
        expect_printed_jsx(source.as_bytes(), b"<x y />;\n");
        let source = format!("<x>\n{whitespace}\n</x>");
        let expected = format!("<x>\n{whitespace}\n</x>;\n");
        expect_printed_jsx(source.as_bytes(), expected.as_bytes());
        let source = format!("<x>\n{whitespace}&quot;\n</x>");
        let expected = format!("<x>\n{whitespace}&quot;\n</x>;\n");
        expect_printed_jsx(source.as_bytes(), expected.as_bytes());
    }
    let whitespace = "\u{0085}";
    let source = format!("<x>\n{whitespace}\n</x>");
    let expected = format!("<x>\n{whitespace}\n</x>;\n");
    expect_printed_jsx(source.as_bytes(), expected.as_bytes());
    let source = format!("<x>\n{whitespace}&quot;\n</x>");
    let expected = format!("<x>\n{whitespace}&quot;\n</x>;\n");
    expect_printed_jsx(source.as_bytes(), expected.as_bytes());
}

#[test]
#[ignore = "depends on esbuild comment handling"]
fn test_replacement_character() {
    expect_printed(b"//\xef\xbf\xbd\n123", b"123;\n");
    expect_printed(b"/*\xef\xbf\xbd*/123", b"123;\n");
    expect_printed(b"'\xef\xbf\xbd'", b"\"\xef\xbf\xbd\";\n");
    expect_printed(b"\"\xef\xbf\xbd\"", b"\"\xef\xbf\xbd\";\n");
    expect_printed(b"`\xef\xbf\xbd`", b"`\xef\xbf\xbd`;\n");
    expect_printed(b"/\xef\xbf\xbd/", b"/\xef\xbf\xbd/;\n");
    expect_printed_jsx(b"<a>\xef\xbf\xbd</a>", b"<a>\xef\xbf\xbd</a>;\n");
}

#[test]
fn test_new_target() {
    expect_printed(
        b"function f() { new.target }",
        b"function f() {\n  new.target;\n}\n",
    );
    expect_printed(
        b"function f() { (new.target) }",
        b"function f() {\n  new.target;\n}\n",
    );
    expect_printed(
        b"function f() { () => new.target }",
        b"function f() {\n  () => new.target;\n}\n",
    );
    expect_printed(
        b"class Foo { x = new.target }",
        b"class Foo {\n  x = new.target;\n}\n",
    );
}

#[test]
fn test_jsx() {
    expect_printed_jsx(b"<div>></div>", b"<div>></div>;\n");
    expect_printed_jsx(b"<div>{1}}</div>", b"<div>{1}}</div>;\n");
    expect_printed_jsx(b"<a/>", b"<a />;\n");
    expect_printed_jsx(b"<a></a>", b"<a />;\n");
    expect_printed_jsx(b"<A/>", b"<A />;\n");
    expect_printed_jsx(b"<a.b/>", b"<a.b />;\n");
    expect_printed_jsx(b"<_a/>", b"<_a />;\n");
    expect_printed_jsx(b"<a-b/>", b"<a-b />;\n");
    expect_printed_jsx(b"<a0/>", b"<a0 />;\n");
    expect_printed_jsx(b"<a b/>", b"<a b />;\n");
    expect_printed_jsx(b"<a b=\"\\\"/>", b"<a b=\"\\\" />;\n");
    expect_printed_jsx(b"<a b=\"<>\"/>", b"<a b=\"<>\" />;\n");
    expect_printed_jsx(b"<a b=\"&lt;&gt;\"/>", b"<a b=\"&lt;&gt;\" />;\n");
    expect_printed_jsx(b"<a b=\"&wrong;\"/>", b"<a b=\"&wrong;\" />;\n");
    expect_printed_jsx(b"<a b={1, 2}/>", b"<a b={(1, 2)} />;\n");
    expect_printed_jsx(b"<a b={<c/>}/>", b"<a b={<c />} />;\n");
    expect_printed_jsx(b"<a {...props}/>", b"<a {...props} />;\n");
    expect_printed_jsx(
        b"<a b=\"\xf0\x9f\x99\x82\"/>",
        b"<a b=\"\xf0\x9f\x99\x82\" />;\n",
    );
    expect_printed_jsx(b"<a>\n</a>", b"<a>\n</a>;\n");
    expect_printed_jsx(b"<a>123</a>", b"<a>123</a>;\n");
    expect_printed_jsx(b"<a>}</a>", b"<a>}</a>;\n");
    expect_printed_jsx(b"<a>=</a>", b"<a>=</a>;\n");
    expect_printed_jsx(b"<a>></a>", b"<a>></a>;\n");
    expect_printed_jsx(b"<a>>=</a>", b"<a>>=</a>;\n");
    expect_printed_jsx(b"<a>>></a>", b"<a>>></a>;\n");
    expect_printed_jsx(b"<a>{}</a>", b"<a>{}</a>;\n");
    expect_printed_jsx(
        b"<a>{/* comment */}</a>",
        b"<a>{\n  /* comment */\n}</a>;\n",
    );
    expect_printed_jsx(b"<a>b{}</a>", b"<a>b{}</a>;\n");
    expect_printed_jsx(
        b"<a>b{/* comment */}</a>",
        b"<a>b{\n  /* comment */\n}</a>;\n",
    );
    expect_printed_jsx(b"<a>{}c</a>", b"<a>{}c</a>;\n");
    expect_printed_jsx(
        b"<a>{/* comment */}c</a>",
        b"<a>{\n  /* comment */\n}c</a>;\n",
    );
    expect_printed_jsx(b"<a>b{}c</a>", b"<a>b{}c</a>;\n");
    expect_printed_jsx(
        b"<a>b{/* comment */}c</a>",
        b"<a>b{\n  /* comment */\n}c</a>;\n",
    );
    expect_printed_jsx(b"<a>{1, 2}</a>", b"<a>{(1, 2)}</a>;\n");
    expect_printed_jsx(b"<a>&lt;&gt;</a>", b"<a>&lt;&gt;</a>;\n");
    expect_printed_jsx(b"<a>&wrong;</a>", b"<a>&wrong;</a>;\n");
    expect_printed_jsx(b"<a>\xf0\x9f\x99\x82</a>", b"<a>\xf0\x9f\x99\x82</a>;\n");
    expect_printed_jsx(b"<a>{...children}</a>", b"<a>{...children}</a>;\n");
    expect_printed_jsx(b"<a b=\"   c\"/>", b"<a b=\"   c\" />;\n");
    expect_printed_jsx(b"<a b=\"   \nc\"/>", b"<a b=\"   \nc\" />;\n");
    expect_printed_jsx(b"<a b=\"\n   c\"/>", b"<a b=\"\n   c\" />;\n");
    expect_printed_jsx(b"<a b=\"c   \"/>", b"<a b=\"c   \" />;\n");
    expect_printed_jsx(b"<a b=\"c   \n\"/>", b"<a b=\"c   \n\" />;\n");
    expect_printed_jsx(b"<a b=\"c\n   \"/>", b"<a b=\"c\n   \" />;\n");
    expect_printed_jsx(b"<a b=\"c   d\"/>", b"<a b=\"c   d\" />;\n");
    expect_printed_jsx(b"<a b=\"c   \nd\"/>", b"<a b=\"c   \nd\" />;\n");
    expect_printed_jsx(b"<a b=\"c\n   d\"/>", b"<a b=\"c\n   d\" />;\n");
    expect_printed_jsx(b"<a b=\"   c\"/>", b"<a b=\"   c\" />;\n");
    expect_printed_jsx(b"<a b=\"   \nc\"/>", b"<a b=\"   \nc\" />;\n");
    expect_printed_jsx(b"<a b=\"\n   c\"/>", b"<a b=\"\n   c\" />;\n");
    expect_printed_jsx(
        b"<a b=\"   \xf0\x9f\x99\x82\"/>",
        b"<a b=\"   \xf0\x9f\x99\x82\" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"   \n\xf0\x9f\x99\x82\"/>",
        b"<a b=\"   \n\xf0\x9f\x99\x82\" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"\n   \xf0\x9f\x99\x82\"/>",
        b"<a b=\"\n   \xf0\x9f\x99\x82\" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"\xf0\x9f\x99\x82   \"/>",
        b"<a b=\"\xf0\x9f\x99\x82   \" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"\xf0\x9f\x99\x82   \n\"/>",
        b"<a b=\"\xf0\x9f\x99\x82   \n\" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"\xf0\x9f\x99\x82\n   \"/>",
        b"<a b=\"\xf0\x9f\x99\x82\n   \" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"\xf0\x9f\x99\x82   \xf0\x9f\x8d\x95\"/>",
        b"<a b=\"\xf0\x9f\x99\x82   \xf0\x9f\x8d\x95\" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"\xf0\x9f\x99\x82   \n\xf0\x9f\x8d\x95\"/>",
        b"<a b=\"\xf0\x9f\x99\x82   \n\xf0\x9f\x8d\x95\" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"\xf0\x9f\x99\x82\n   \xf0\x9f\x8d\x95\"/>",
        b"<a b=\"\xf0\x9f\x99\x82\n   \xf0\x9f\x8d\x95\" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"   \xf0\x9f\x99\x82\"/>",
        b"<a b=\"   \xf0\x9f\x99\x82\" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"   \n\xf0\x9f\x99\x82\"/>",
        b"<a b=\"   \n\xf0\x9f\x99\x82\" />;\n",
    );
    expect_printed_jsx(
        b"<a b=\"\n   \xf0\x9f\x99\x82\"/>",
        b"<a b=\"\n   \xf0\x9f\x99\x82\" />;\n",
    );
    expect_printed_jsx(b"<a>   b</a>", b"<a>   b</a>;\n");
    expect_printed_jsx(b"<a>   \nb</a>", b"<a>   \nb</a>;\n");
    expect_printed_jsx(b"<a>\n   b</a>", b"<a>\n   b</a>;\n");
    expect_printed_jsx(b"<a>b   </a>", b"<a>b   </a>;\n");
    expect_printed_jsx(b"<a>b   \n</a>", b"<a>b   \n</a>;\n");
    expect_printed_jsx(b"<a>b\n   </a>", b"<a>b\n   </a>;\n");
    expect_printed_jsx(b"<a>b   c</a>", b"<a>b   c</a>;\n");
    expect_printed_jsx(b"<a>b   \nc</a>", b"<a>b   \nc</a>;\n");
    expect_printed_jsx(b"<a>b\n   c</a>", b"<a>b\n   c</a>;\n");
    expect_printed_jsx(b"<a>   b</a>", b"<a>   b</a>;\n");
    expect_printed_jsx(b"<a>   \nb</a>", b"<a>   \nb</a>;\n");
    expect_printed_jsx(b"<a>\n   b</a>", b"<a>\n   b</a>;\n");
    expect_printed_jsx(
        b"<a>   \xf0\x9f\x99\x82</a>",
        b"<a>   \xf0\x9f\x99\x82</a>;\n",
    );
    expect_printed_jsx(
        b"<a>   \n\xf0\x9f\x99\x82</a>",
        b"<a>   \n\xf0\x9f\x99\x82</a>;\n",
    );
    expect_printed_jsx(
        b"<a>\n   \xf0\x9f\x99\x82</a>",
        b"<a>\n   \xf0\x9f\x99\x82</a>;\n",
    );
    expect_printed_jsx(
        b"<a>\xf0\x9f\x99\x82   </a>",
        b"<a>\xf0\x9f\x99\x82   </a>;\n",
    );
    expect_printed_jsx(
        b"<a>\xf0\x9f\x99\x82   \n</a>",
        b"<a>\xf0\x9f\x99\x82   \n</a>;\n",
    );
    expect_printed_jsx(
        b"<a>\xf0\x9f\x99\x82\n   </a>",
        b"<a>\xf0\x9f\x99\x82\n   </a>;\n",
    );
    expect_printed_jsx(
        b"<a>\xf0\x9f\x99\x82   \xf0\x9f\x8d\x95</a>",
        b"<a>\xf0\x9f\x99\x82   \xf0\x9f\x8d\x95</a>;\n",
    );
    expect_printed_jsx(
        b"<a>\xf0\x9f\x99\x82   \n\xf0\x9f\x8d\x95</a>",
        b"<a>\xf0\x9f\x99\x82   \n\xf0\x9f\x8d\x95</a>;\n",
    );
    expect_printed_jsx(
        b"<a>\xf0\x9f\x99\x82\n   \xf0\x9f\x8d\x95</a>",
        b"<a>\xf0\x9f\x99\x82\n   \xf0\x9f\x8d\x95</a>;\n",
    );
    expect_printed_jsx(
        b"<a>   \xf0\x9f\x99\x82</a>",
        b"<a>   \xf0\x9f\x99\x82</a>;\n",
    );
    expect_printed_jsx(
        b"<a>   \n\xf0\x9f\x99\x82</a>",
        b"<a>   \n\xf0\x9f\x99\x82</a>;\n",
    );
    expect_printed_jsx(
        b"<a>\n   \xf0\x9f\x99\x82</a>",
        b"<a>\n   \xf0\x9f\x99\x82</a>;\n",
    );
    expect_printed_jsx(b"<a>{x}<b/></a>;", b"<a>{x}<b /></a>;\n");
    expect_printed_jsx(b"<a>\n{x}<b/></a>;", b"<a>\n{x}<b /></a>;\n");
    expect_printed_jsx(b"<a>{x}\n<b/></a>;", b"<a>{x}\n<b /></a>;\n");
    expect_printed_jsx(b"<a>\n{x}\n<b/></a>;", b"<a>\n{x}\n<b /></a>;\n");
    expect_printed_jsx(b"<a>{x}<b/>\n</a>;", b"<a>{x}<b />\n</a>;\n");
    expect_printed_jsx(b"<a>\n{x}<b/>\n</a>;", b"<a>\n{x}<b />\n</a>;\n");
    expect_printed_jsx(b"<a>{x}\n<b/>\n</a>;", b"<a>{x}\n<b />\n</a>;\n");
    expect_printed_jsx(b"<a>\n{x}\n<b/>\n</a>;", b"<a>\n{x}\n<b />\n</a>;\n");
    expect_printed_jsx(b"<a> {x}<b/></a>;", b"<a> {x}<b /></a>;\n");
    expect_printed_jsx(b"<a> {x}\n<b/></a>;", b"<a> {x}\n<b /></a>;\n");
    expect_printed_jsx(b"<a> {x}<b/>\n</a>;", b"<a> {x}<b />\n</a>;\n");
    expect_printed_jsx(b"<a> {x}\n<b/>\n</a>;", b"<a> {x}\n<b />\n</a>;\n");
    expect_printed_jsx(b"<a>{x} <b/></a>;", b"<a>{x} <b /></a>;\n");
    expect_printed_jsx(b"<a>\n{x} <b/></a>;", b"<a>\n{x} <b /></a>;\n");
    expect_printed_jsx(b"<a>{x} <b/>\n</a>;", b"<a>{x} <b />\n</a>;\n");
    expect_printed_jsx(b"<a>\n{x} <b/>\n</a>;", b"<a>\n{x} <b />\n</a>;\n");
    expect_printed_jsx(b"<a> {x} <b/></a>;", b"<a> {x} <b /></a>;\n");
    expect_printed_jsx(b"<a> {x} <b/>\n</a>;", b"<a> {x} <b />\n</a>;\n");
    expect_printed_jsx(b"<a>{x}<b/> </a>;", b"<a>{x}<b /> </a>;\n");
    expect_printed_jsx(b"<a>\n{x}<b/> </a>;", b"<a>\n{x}<b /> </a>;\n");
    expect_printed_jsx(b"<a>{x}\n<b/> </a>;", b"<a>{x}\n<b /> </a>;\n");
    expect_printed_jsx(b"<a>\n{x}\n<b/> </a>;", b"<a>\n{x}\n<b /> </a>;\n");
    expect_printed_jsx(b"<a> {x}<b/> </a>;", b"<a> {x}<b /> </a>;\n");
    expect_printed_jsx(b"<a> {x}\n<b/> </a>;", b"<a> {x}\n<b /> </a>;\n");
    expect_printed_jsx(b"<a>{x} <b/> </a>;", b"<a>{x} <b /> </a>;\n");
    expect_printed_jsx(b"<a>\n{x} <b/> </a>;", b"<a>\n{x} <b /> </a>;\n");
    expect_printed_jsx(b"<a> {x} <b/> </a>;", b"<a> {x} <b /> </a>;\n");
    expect_printed_jsx(b"< /**/ a/>", b"<a />;\n");
    expect_printed_jsx(b"< //\n a/>", b"<a />;\n");
    expect_printed_jsx(b"<a /**/ />", b"<a />;\n");
    expect_printed_jsx(b"<a //\n />", b"<a />;\n");
    expect_printed_jsx(b"<a/ /**/ >", b"<a />;\n");
    expect_printed_jsx(b"<a/ //\n >", b"<a />;\n");
    expect_printed_jsx(b"<a>b< /**/ /a>", b"<a>b</a>;\n");
    expect_printed_jsx(b"<a>b< //\n /a>", b"<a>b</a>;\n");
    expect_printed_jsx(b"<a>b</ /**/ a>", b"<a>b</a>;\n");
    expect_printed_jsx(b"<a>b</ //\n a>", b"<a>b</a>;\n");
    expect_printed_jsx(b"<a>b</a /**/ >", b"<a>b</a>;\n");
    expect_printed_jsx(b"<a>b</a //\n >", b"<a>b</a>;\n");
    expect_printed_jsx(b"<a> /**/ </a>", b"<a> /**/ </a>;\n");
    expect_printed_jsx(b"<a> //\n </a>", b"<a> //\n </a>;\n");
    expect_printed_jsx(b"<\xf0\xa0\x80\x80/>", b"<\xf0\xa0\x80\x80 />;\n");
    expect_printed_jsx(b"<a>\xf0\xa0\x80\x80</a>", b"<a>\xf0\xa0\x80\x80</a>;\n");
    expect_printed_jsx(
        b"<a \xf0\xa0\x80\x80={0}/>",
        b"<a \xf0\xa0\x80\x80={0} />;\n",
    );
    expect_printed_jsx(b"<a b=<c/>/>", b"<a b=<c /> />;\n");
    expect_printed_jsx(b"<a b=<></>/>", b"<a b=<></> />;\n");
    for colon in [":", " :", ": ", " : "] {
        expect_printed_jsx(format!("<a{colon}b/>").as_bytes(), b"<a:b />;\n");
        expect_printed_jsx(format!("<a-b{colon}c-d/>").as_bytes(), b"<a-b:c-d />;\n");
        expect_printed_jsx(format!("<a-{colon}b-/>").as_bytes(), b"<a-:b- />;\n");
        expect_printed_jsx(format!("<Te{colon}st/>").as_bytes(), b"<Te:st />;\n");
        expect_printed_jsx(format!("<x a{colon}b/>").as_bytes(), b"<x a:b />;\n");
        expect_printed_jsx(
            format!("<x a-b{colon}c-d/>").as_bytes(),
            b"<x a-b:c-d />;\n",
        );
        expect_printed_jsx(format!("<x a-{colon}b-/>").as_bytes(), b"<x a-:b- />;\n");
        expect_printed_jsx(format!("<x Te{colon}st/>").as_bytes(), b"<x Te:st />;\n");
        expect_printed_jsx(
            format!("<x a{colon}b={{0}}/>").as_bytes(),
            b"<x a:b={0} />;\n",
        );
        expect_printed_jsx(
            format!("<x a-b{colon}c-d={{0}}/>").as_bytes(),
            b"<x a-b:c-d={0} />;\n",
        );
        expect_printed_jsx(
            format!("<x a-{colon}b-={{0}}/>").as_bytes(),
            b"<x a-:b-={0} />;\n",
        );
        expect_printed_jsx(
            format!("<x Te{colon}st={{0}}/>").as_bytes(),
            b"<x Te:st={0} />;\n",
        );
        expect_printed_jsx(b"<a-b a-b={a-b}/>", b"<a-b a-b={a - b} />;\n");
    }
}

#[test]
fn test_jsx_single_line() {
    expect_printed_jsx(b"<x/>", b"<x />;\n");
    expect_printed_jsx(b"<x y/>", b"<x y />;\n");
    expect_printed_jsx(b"<x\n/>", b"<x />;\n");
    expect_printed_jsx(b"<x\ny/>", b"<x y />;\n");
    expect_printed_jsx(b"<x y\n/>", b"<x y />;\n");
    expect_printed_jsx(b"<x\n{...y}/>", b"<x {...y} />;\n");
}

#[test]
#[ignore = "depends on esbuild JSX pragma handling"]
fn test_jsx_pragmas() {
    expect_printed_jsx(b"// @jsx h\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"/*@jsx h*/\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"/* @jsx h */\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"<a/>\n// @jsx h", b"<a />;\n");
    expect_printed_jsx(b"<a/>\n/*@jsx h*/", b"<a />;\n");
    expect_printed_jsx(b"<a/>\n/* @jsx h */", b"<a />;\n");
    expect_printed_jsx(b"// @jsx a.b.c\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"/*@jsx a.b.c*/\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"/* @jsx a.b.c */\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"// @jsxFrag f\n<></>", b"<></>;\n");
    expect_printed_jsx(b"/*@jsxFrag f*/\n<></>", b"<></>;\n");
    expect_printed_jsx(b"/* @jsxFrag f */\n<></>", b"<></>;\n");
    expect_printed_jsx(b"<></>\n// @jsxFrag f", b"<></>;\n");
    expect_printed_jsx(b"<></>\n/*@jsxFrag f*/", b"<></>;\n");
    expect_printed_jsx(b"<></>\n/* @jsxFrag f */", b"<></>;\n");
    expect_printed_jsx(b"// @jsxFrag a.b.c\n<></>", b"<></>;\n");
    expect_printed_jsx(b"/*@jsxFrag a.b.c*/\n<></>", b"<></>;\n");
    expect_printed_jsx(b"/* @jsxFrag a.b.c */\n<></>", b"<></>;\n");
}

#[test]
fn test_jsx_automatic() {
    expect_printed_jsx(b"with (x) y(<z/>)", b"with (x) y(<z />);\n");
}

#[test]
#[ignore = "depends on esbuild JSX pragma handling"]
fn test_jsx_automatic_pragmas() {
    expect_printed_jsx(b"// @jsxRuntime automatic\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"/*@jsxRuntime automatic*/\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"/* @jsxRuntime automatic */\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"<a/>\n/*@jsxRuntime automatic*/", b"<a />;\n");
    expect_printed_jsx(b"<a/>\n/* @jsxRuntime automatic */", b"<a />;\n");
    expect_printed_jsx(b"// @jsxRuntime classic\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"/*@jsxRuntime classic*/\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"/* @jsxRuntime classic */\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"<a/>\n/*@jsxRuntime classic*/\n", b"<a />;\n");
    expect_printed_jsx(b"<a/>\n/* @jsxRuntime classic */\n", b"<a />;\n");
    expect_printed_jsx(
        b"// @jsxRuntime automatic @jsxImportSource src\n<a/>",
        b"<a />;\n",
    );
    expect_printed_jsx(
        b"/*@jsxRuntime automatic @jsxImportSource src*/\n<a/>",
        b"<a />;\n",
    );
    expect_printed_jsx(
        b"/*@jsxRuntime automatic*//*@jsxImportSource src*/\n<a/>",
        b"<a />;\n",
    );
    expect_printed_jsx(
        b"/* @jsxRuntime automatic */\n/* @jsxImportSource src */\n<a/>",
        b"<a />;\n",
    );
    expect_printed_jsx(
        b"<a/>\n/*@jsxRuntime automatic @jsxImportSource src*/",
        b"<a />;\n",
    );
    expect_printed_jsx(
        b"<a/>\n/*@jsxRuntime automatic*/\n/*@jsxImportSource src*/",
        b"<a />;\n",
    );
    expect_printed_jsx(
        b"<a/>\n/* @jsxRuntime automatic */\n/* @jsxImportSource src */",
        b"<a />;\n",
    );
    expect_printed_jsx(
        b"// @jsxRuntime classic @jsxImportSource src\n<a/>",
        b"<a />;\n",
    );
    expect_printed_jsx(b"// @jsxRuntime automatic @jsx h\n<a/>", b"<a />;\n");
    expect_printed_jsx(b"// @jsxRuntime automatic @jsxFrag f\n<></>", b"<></>;\n");
}

#[test]
fn test_jsx_side_effects() {
    expect_printed_jsx(b"<a/>", b"<a />;\n");
    expect_printed_jsx(b"<></>", b"<></>;\n");
}

#[test]
fn test_preserve_optional_chain_parentheses() {
    expect_printed(b"a?.b.c", b"a?.b.c;\n");
    expect_printed(b"(a?.b).c", b"(a?.b).c;\n");
    expect_printed(b"a?.b.c.d", b"a?.b.c.d;\n");
    expect_printed(b"(a?.b.c).d", b"(a?.b.c).d;\n");
    expect_printed(b"a?.b[c]", b"a?.b[c];\n");
    expect_printed(b"(a?.b)[c]", b"(a?.b)[c];\n");
    expect_printed(b"a?.b(c)", b"a?.b(c);\n");
    expect_printed(b"(a?.b)(c)", b"(a?.b)(c);\n");
    expect_printed(b"new (a?.b)", b"new (a?.b)();\n");
    expect_printed(b"a?.[b][c]", b"a?.[b][c];\n");
    expect_printed(b"(a?.[b])[c]", b"(a?.[b])[c];\n");
    expect_printed(b"a?.[b][c][d]", b"a?.[b][c][d];\n");
    expect_printed(b"(a?.[b][c])[d]", b"(a?.[b][c])[d];\n");
    expect_printed(b"a?.[b].c", b"a?.[b].c;\n");
    expect_printed(b"(a?.[b]).c", b"(a?.[b]).c;\n");
    expect_printed(b"a?.[b](c)", b"a?.[b](c);\n");
    expect_printed(b"(a?.[b])(c)", b"(a?.[b])(c);\n");
    expect_printed(b"new (a?.[b])", b"new (a?.[b])();\n");
    expect_printed(b"a?.(b)(c)", b"a?.(b)(c);\n");
    expect_printed(b"(a?.(b))(c)", b"(a?.(b))(c);\n");
    expect_printed(b"a?.(b)(c)(d)", b"a?.(b)(c)(d);\n");
    expect_printed(b"(a?.(b)(c))(d)", b"(a?.(b)(c))(d);\n");
    expect_printed(b"a?.(b).c", b"a?.(b).c;\n");
    expect_printed(b"(a?.(b)).c", b"(a?.(b)).c;\n");
    expect_printed(b"a?.(b)[c]", b"a?.(b)[c];\n");
    expect_printed(b"(a?.(b))[c]", b"(a?.(b))[c];\n");
    expect_printed(b"new (a?.(b))", b"new (a?.(b))();\n");
    expect_printed(b"new a()?.b", b"new a()?.b;\n");
    expect_printed(b"new a()?.[b]", b"new a()?.[b];\n");
    expect_printed(b"new a()?.(b)", b"new a()?.(b);\n");
    expect_printed(b"new a.b()?.c", b"new a.b()?.c;\n");
    expect_printed(b"new a.b()?.[c]", b"new a.b()?.[c];\n");
    expect_printed(b"new a.b()?.(c)", b"new a.b()?.(c);\n");
}

#[test]
fn test_private_identifiers() {
    expect_printed(b"class Foo { #foo }", b"class Foo {\n  #foo;\n}\n");
    expect_printed(b"class Foo { #foo = 1 }", b"class Foo {\n  #foo = 1;\n}\n");
    expect_printed(
        b"class Foo { #foo = #foo in this }",
        b"class Foo {\n  #foo = #foo in this;\n}\n",
    );
    expect_printed(
        b"class Foo { #foo = #foo in (#bar in this); #bar }",
        b"class Foo {\n  #foo = #foo in (#bar in this);\n  #bar;\n}\n",
    );
    expect_printed(
        b"class Foo { #foo() {} }",
        b"class Foo {\n  #foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { get #foo() {} }",
        b"class Foo {\n  get #foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { set #foo(x) {} }",
        b"class Foo {\n  set #foo(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static #foo }",
        b"class Foo {\n  static #foo;\n}\n",
    );
    expect_printed(
        b"class Foo { static #foo = 1 }",
        b"class Foo {\n  static #foo = 1;\n}\n",
    );
    expect_printed(
        b"class Foo { static #foo() {} }",
        b"class Foo {\n  static #foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static get #foo() {} }",
        b"class Foo {\n  static get #foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { static set #foo(x) {} }",
        b"class Foo {\n  static set #foo(x) {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { #\\u0066oo; foo = this.#foo }",
        b"class Foo {\n  #foo;\n  foo = this.#foo;\n}\n",
    );
    expect_printed(
        b"class Foo { #fo\\u006f; foo = this.#foo }",
        b"class Foo {\n  #foo;\n  foo = this.#foo;\n}\n",
    );
    expect_printed(
        b"class Foo { get #foo() {} set #foo(x) { this.#foo } }",
        b"class Foo {\n  get #foo() {\n  }\n  set #foo(x) {\n    this.#foo;\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { set #foo(x) { this.#foo } get #foo() {} }",
        b"class Foo {\n  set #foo(x) {\n    this.#foo;\n  }\n  get #foo() {\n  }\n}\n",
    );
    expect_printed(
        b"class Foo { #foo } class Bar { #foo }",
        b"class Foo {\n  #foo;\n}\nclass Bar {\n  #foo;\n}\n",
    );
    expect_printed(
        b"class Foo { foo = this.#foo; #foo }",
        b"class Foo {\n  foo = this.#foo;\n  #foo;\n}\n",
    );
    expect_printed(
        b"class Foo { foo = this?.#foo; #foo }",
        b"class Foo {\n  foo = this?.#foo;\n  #foo;\n}\n",
    );
    expect_printed(b"class Foo {\n\t#if\n\t#im() { return this.#im(this.#if) }\n\tstatic #sf\n\tstatic #sm() { return this.#sm(this.#sf) }\n\tfoo() {\n\t\treturn class {\n\t\t\t#inner() {\n\t\t\t\treturn [this.#im, this?.#inner, this?.x.#if]\n\t\t\t}\n\t\t}\n\t}\n}\n", b"class Foo {\n  #if;\n  #im() {\n    return this.#im(this.#if);\n  }\n  static #sf;\n  static #sm() {\n    return this.#sm(this.#sf);\n  }\n  foo() {\n    return class {\n      #inner() {\n        return [this.#im, this?.#inner, this?.x.#if];\n      }\n    };\n  }\n}\n");
}

#[test]
#[ignore = "depends on esbuild import-assertion parsing"]
fn test_import_assertions() {
    expect_printed(b"import 'x' assert {}", b"import \"x\" assert {};\n");
    expect_printed(b"import 'x' assert {\n}", b"import \"x\" assert {};\n");
    expect_printed(b"import 'x' assert\n{}", b"import \"x\" assert {};\n");
    expect_printed(b"import 'x'\nassert\n{}", b"import \"x\";\nassert;\n{\n}\n");
    expect_printed(
        b"import 'x' assert {type: 'json'}",
        b"import \"x\" assert { type: \"json\" };\n",
    );
    expect_printed(
        b"import 'x' assert {type: 'json',}",
        b"import \"x\" assert { type: \"json\" };\n",
    );
    expect_printed(
        b"import 'x' assert {'type': 'json'}",
        b"import \"x\" assert { \"type\": \"json\" };\n",
    );
    expect_printed(
        b"import 'x' assert {a: 'b', c: 'd'}",
        b"import \"x\" assert { a: \"b\", c: \"d\" };\n",
    );
    expect_printed(
        b"import 'x' assert {a: 'b', c: 'd',}",
        b"import \"x\" assert { a: \"b\", c: \"d\" };\n",
    );
    expect_printed(
        b"import 'x' assert {if: 'keyword'}",
        b"import \"x\" assert { if: \"keyword\" };\n",
    );
    expect_printed(
        b"import x from 'x' assert {x: 'y'}",
        b"import x from \"x\" assert { x: \"y\" };\n",
    );
    expect_printed(
        b"import * as x from 'x' assert {x: 'y'}",
        b"import * as x from \"x\" assert { x: \"y\" };\n",
    );
    expect_printed(
        b"import {} from 'x' assert {x: 'y'}",
        b"import {} from \"x\" assert { x: \"y\" };\n",
    );
    expect_printed(
        b"export {} from 'x' assert {x: 'y'}",
        b"export {} from \"x\" assert { x: \"y\" };\n",
    );
    expect_printed(
        b"export * from 'x' assert {x: 'y'}",
        b"export * from \"x\" assert { x: \"y\" };\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z')",
        b"x ? import(\"y\") : import(\"z\");\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {assert: {}})",
        b"x ? import(\"y\", { assert: {} }) : import(\"z\", { assert: {} });\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {assert: {a: 'b'}})",
        b"x ? import(\"y\", { assert: { a: \"b\" } }) : import(\"z\", { assert: { a: \"b\" } });\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {assert: {'a': 'b'}})",
        b"x ? import(\"y\", { assert: { \"a\": \"b\" } }) : import(\"z\", { assert: { \"a\": \"b\" } });\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {})",
        b"import(x ? \"y\" : \"z\", {});\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {assert: []})",
        b"import(x ? \"y\" : \"z\", { assert: [] });\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {asserts: {}})",
        b"import(x ? \"y\" : \"z\", { asserts: {} });\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {assert: {x: 1}})",
        b"import(x ? \"y\" : \"z\", { assert: { x: 1 } });\n",
    );
}

#[test]
#[ignore = "depends on esbuild parser normalization"]
fn test_import_attributes() {
    expect_printed(b"import 'x' with {}", b"import \"x\" with {};\n");
    expect_printed(b"import 'x' with {\n}", b"import \"x\" with {};\n");
    expect_printed(b"import 'x' with\n{}", b"import \"x\" with {};\n");
    expect_printed(b"import 'x'\nwith\n{}", b"import \"x\" with {};\n");
    expect_printed(
        b"import 'x' with {type: 'json'}",
        b"import \"x\" with { type: \"json\" };\n",
    );
    expect_printed(
        b"import 'x' with {type: 'json',}",
        b"import \"x\" with { type: \"json\" };\n",
    );
    expect_printed(
        b"import 'x' with {'type': 'json'}",
        b"import \"x\" with { \"type\": \"json\" };\n",
    );
    expect_printed(
        b"import 'x' with {a: 'b', c: 'd'}",
        b"import \"x\" with { a: \"b\", c: \"d\" };\n",
    );
    expect_printed(
        b"import 'x' with {a: 'b', c: 'd',}",
        b"import \"x\" with { a: \"b\", c: \"d\" };\n",
    );
    expect_printed(
        b"import 'x' with {if: 'keyword'}",
        b"import \"x\" with { if: \"keyword\" };\n",
    );
    expect_printed(
        b"import x from 'x' with {x: 'y'}",
        b"import x from \"x\" with { x: \"y\" };\n",
    );
    expect_printed(
        b"import * as x from 'x' with {x: 'y'}",
        b"import * as x from \"x\" with { x: \"y\" };\n",
    );
    expect_printed(
        b"import {} from 'x' with {x: 'y'}",
        b"import {} from \"x\" with { x: \"y\" };\n",
    );
    expect_printed(
        b"export {} from 'x' with {x: 'y'}",
        b"export {} from \"x\" with { x: \"y\" };\n",
    );
    expect_printed(
        b"export * from 'x' with {x: 'y'}",
        b"export * from \"x\" with { x: \"y\" };\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z')",
        b"x ? import(\"y\") : import(\"z\");\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {with: {}})",
        b"x ? import(\"y\", { with: {} }) : import(\"z\", { with: {} });\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {with: {a: 'b'}})",
        b"x ? import(\"y\", { with: { a: \"b\" } }) : import(\"z\", { with: { a: \"b\" } });\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {with: {'a': 'b'}})",
        b"x ? import(\"y\", { with: { \"a\": \"b\" } }) : import(\"z\", { with: { \"a\": \"b\" } });\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {})",
        b"import(x ? \"y\" : \"z\", {});\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {with: []})",
        b"import(x ? \"y\" : \"z\", { with: [] });\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {whithe: {}})",
        b"import(x ? \"y\" : \"z\", { whithe: {} });\n",
    );
    expect_printed(
        b"import(x ? 'y' : 'z', {with: {x: 1}})",
        b"import(x ? \"y\" : \"z\", { with: { x: 1 } });\n",
    );
}

#[test]
fn test_ascii_only() {
    expect_printed(b"\xcf\x80", b"\xcf\x80;\n");
    expect_printed(b"\xf0\x90\x80\x80", b"\xf0\x90\x80\x80;\n");
    expect_printed(b"var \xcf\x80", b"var \xcf\x80;\n");
    expect_printed(b"var \xf0\x90\x80\x80", b"var \xf0\x90\x80\x80;\n");
    expect_printed(b"'\xcf\x80'", b"\"\xcf\x80\";\n");
    expect_printed(b"'\xf0\x90\x80\x80'", b"\"\xf0\x90\x80\x80\";\n");
    expect_printed(b"x.\xcf\x80", b"x.\xcf\x80;\n");
    expect_printed(b"x.\xf0\x90\x80\x80", b"x[\"\xf0\x90\x80\x80\"];\n");
    expect_printed(b"x?.\xcf\x80", b"x?.\xcf\x80;\n");
    expect_printed(b"x?.\xf0\x90\x80\x80", b"x?.[\"\xf0\x90\x80\x80\"];\n");
    expect_printed(b"0 .\xcf\x80", b"0 .\xcf\x80;\n");
    expect_printed(b"0 .\xf0\x90\x80\x80", b"0[\"\xf0\x90\x80\x80\"];\n");
    expect_printed(b"0?.\xcf\x80", b"0?.\xcf\x80;\n");
    expect_printed(b"0?.\xf0\x90\x80\x80", b"0?.[\"\xf0\x90\x80\x80\"];\n");
    expect_printed(b"import '\xcf\x80'", b"import \"\xcf\x80\";\n");
    expect_printed(
        b"import '\xf0\x90\x80\x80'",
        b"import \"\xf0\x90\x80\x80\";\n",
    );
    expect_printed(b"({\xcf\x80: 0})", b"({ \xcf\x80: 0 });\n");
    expect_printed(
        b"({\xf0\x90\x80\x80: 0})",
        b"({ \"\xf0\x90\x80\x80\": 0 });\n",
    );
    expect_printed(b"({\xcf\x80})", b"({ \xcf\x80 });\n");
    expect_printed(
        b"({\xf0\x90\x80\x80})",
        b"({ \"\xf0\x90\x80\x80\": \xf0\x90\x80\x80 });\n",
    );
    expect_printed(
        b"import * as \xcf\x80 from 'path'; \xcf\x80",
        b"import * as \xcf\x80 from \"path\";\n\xcf\x80;\n",
    );
    expect_printed(
        b"import * as \xf0\x90\x80\x80 from 'path'; \xf0\x90\x80\x80",
        b"import * as \xf0\x90\x80\x80 from \"path\";\n\xf0\x90\x80\x80;\n",
    );
    expect_printed(
        b"import {\xcf\x80} from 'path'; \xcf\x80",
        b"import { \xcf\x80 } from \"path\";\n\xcf\x80;\n",
    );
    expect_printed(
        b"import {\xf0\x90\x80\x80} from 'path'; \xf0\x90\x80\x80",
        b"import { \xf0\x90\x80\x80 } from \"path\";\n\xf0\x90\x80\x80;\n",
    );
    expect_printed(
        b"import {\xcf\x80 as x} from 'path'",
        b"import { \xcf\x80 as x } from \"path\";\n",
    );
    expect_printed(
        b"import {\xf0\x90\x80\x80 as x} from 'path'",
        b"import { \xf0\x90\x80\x80 as x } from \"path\";\n",
    );
    expect_printed(
        b"import {x as \xcf\x80} from 'path'",
        b"import { x as \xcf\x80 } from \"path\";\n",
    );
    expect_printed(
        b"import {x as \xf0\x90\x80\x80} from 'path'",
        b"import { x as \xf0\x90\x80\x80 } from \"path\";\n",
    );
    expect_printed(
        b"export * as \xcf\x80 from 'path'; \xcf\x80",
        b"export * as \xcf\x80 from \"path\";\n\xcf\x80;\n",
    );
    expect_printed(
        b"export * as \xf0\x90\x80\x80 from 'path'; \xf0\x90\x80\x80",
        b"export * as \xf0\x90\x80\x80 from \"path\";\n\xf0\x90\x80\x80;\n",
    );
    expect_printed(
        b"export {\xcf\x80} from 'path'; \xcf\x80",
        b"export { \xcf\x80 } from \"path\";\n\xcf\x80;\n",
    );
    expect_printed(
        b"export {\xf0\x90\x80\x80} from 'path'; \xf0\x90\x80\x80",
        b"export { \xf0\x90\x80\x80 } from \"path\";\n\xf0\x90\x80\x80;\n",
    );
    expect_printed(
        b"export {\xcf\x80 as x} from 'path'",
        b"export { \xcf\x80 as x } from \"path\";\n",
    );
    expect_printed(
        b"export {\xf0\x90\x80\x80 as x} from 'path'",
        b"export { \xf0\x90\x80\x80 as x } from \"path\";\n",
    );
    expect_printed(
        b"export {x as \xcf\x80} from 'path'",
        b"export { x as \xcf\x80 } from \"path\";\n",
    );
    expect_printed(
        b"export {x as \xf0\x90\x80\x80} from 'path'",
        b"export { x as \xf0\x90\x80\x80 } from \"path\";\n",
    );
    expect_printed(
        b"export {\xcf\x80}; var \xcf\x80",
        b"export { \xcf\x80 };\nvar \xcf\x80;\n",
    );
    expect_printed(
        b"export {\xf0\x90\x80\x80}; var \xf0\x90\x80\x80",
        b"export { \xf0\x90\x80\x80 };\nvar \xf0\x90\x80\x80;\n",
    );
    expect_printed(b"export var \xcf\x80", b"export var \xcf\x80;\n");
    expect_printed(
        b"export var \xf0\x90\x80\x80",
        b"export var \xf0\x90\x80\x80;\n",
    );
}

#[test]
fn test_using() {
    expect_printed(b"using x = y", b"using x = y;\n");
    expect_printed(b"using x = y; z", b"using x = y;\nz;\n");
    expect_printed(b"using x = y, z = _", b"using x = y, z = _;\n");
    expect_printed(b"using x = y, \n z = _", b"using x = y, z = _;\n");
    expect_printed(b"using \n x = y", b"using;\nx = y;\n");
    expect_printed(b"using [x]", b"using[x];\n");
    expect_printed(b"using [x] = y", b"using[x] = y;\n");
    expect_printed(b"using \n [x] = y", b"using[x] = y;\n");
    expect_printed(b"for (using x = y;;) ;", b"for (using x = y; ; ) ;\n");
    expect_printed(b"for (using x of y) ;", b"for (using x of y) ;\n");
    expect_printed(b"for (using of x) ;", b"for (using of x) ;\n");
    expect_printed(b"for (using of of) ;", b"for (using of of) ;\n");
    expect_printed(
        b"for (await using of of x) ;",
        b"for (await using of of x) ;\n",
    );
    expect_printed(
        b"for (await using of of of) ;",
        b"for (await using of of of) ;\n",
    );
    expect_printed(
        b"for await (using x of y) ;",
        b"for await (using x of y) ;\n",
    );
    expect_printed(b"for await (using of x) ;", b"for await (using of x) ;\n");
    expect_printed(b"await using \n x = y", b"await using;\nx = y;\n");
    expect_printed(b"await \n using \n x \n = \n y", b"await using;\nx = y;\n");
    expect_printed(b"await using [x]", b"await using[x];\n");
    expect_printed(b"await using ([x] = y)", b"await using([x] = y);\n");
    expect_printed(b"await (using [x] = y)", b"await (using[x] = y);\n");
    expect_printed(b"await using x = y", b"await using x = y;\n");
    expect_printed(b"await using x = y, z = _", b"await using x = y, z = _;\n");
    expect_printed(
        b"for (await using x of y) ;",
        b"for (await using x of y) ;\n",
    );
    expect_printed(
        b"for await (await using x of y) ;",
        b"for await (await using x of y) ;\n",
    );
    expect_printed(
        b"function foo() { using x = y }",
        b"function foo() {\n  using x = y;\n}\n",
    );
    expect_printed(
        b"foo = function() { using x = y }",
        b"foo = function() {\n  using x = y;\n};\n",
    );
    expect_printed(
        b"foo = () => { using x = y }",
        b"foo = () => {\n  using x = y;\n};\n",
    );
    expect_printed(
        b"async function foo() { using x = y }",
        b"async function foo() {\n  using x = y;\n}\n",
    );
    expect_printed(
        b"foo = async function() { using x = y }",
        b"foo = async function() {\n  using x = y;\n};\n",
    );
    expect_printed(
        b"foo = async () => { using x = y }",
        b"foo = async () => {\n  using x = y;\n};\n",
    );
    expect_printed(
        b"async function foo() { await using x = y }",
        b"async function foo() {\n  await using x = y;\n}\n",
    );
    expect_printed(
        b"foo = async function() { await using x = y }",
        b"foo = async function() {\n  await using x = y;\n};\n",
    );
    expect_printed(
        b"foo = async () => { await using x = y }",
        b"foo = async () => {\n  await using x = y;\n};\n",
    );
    expect_printed(b"using x = {}", b"using x = {};\n");
    expect_printed(b"using x = null", b"using x = null;\n");
    // Depends on esbuild's parser-stage undefined normalization.
    // expect_printed(b"using x = undefined", b"using x = void 0;\n");
    expect_printed(b"using x = (foo, y)", b"using x = (foo, y);\n");
    expect_printed(b"using x = (foo, null)", b"using x = (foo, null);\n");
    // Depends on esbuild's parser-stage undefined normalization.
    // expect_printed(b"using x = (foo, undefined)", b"using x = (foo, void 0);\n");
}
