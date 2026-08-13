// Ported from upstream esbuild's internal/js_parser/js_parser_test.go.
// This upstream mirror should normally only change when syncing with esbuild.
// Add swc-next-specific regressions to the parent suite (`../js.rs` or `../ts.rs`).
use crate::{expect_printed, expect_printed_jsx, expect_printed_script};

#[test]
fn test_un_op() {
    // Depends on esbuild's parser-stage unused-expression folding.
    // expect_printed("let x; void 0; x", "let x;\nx;\n");
    expect_printed("let x; void x; x", "let x;\nvoid x;\nx;\n");
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
        expect_printed(&source, &expected);
        let source = format!("(a {op} b) {op} c");
        expect_printed(&source, &expected);
        let source = format!("a {op} (b {op} c)");
        let expected = format!("a {op} (b {op} c);\n");
        expect_printed(&source, &expected);
    }
    expect_printed("a ** b ** c", "a ** b ** c;\n");
    expect_printed("(a ** b) ** c", "(a ** b) ** c;\n");
    expect_printed("a ** (b ** c)", "a ** b ** c;\n");
    const ASSIGNMENTS: &[&str] = &[
        "=", "+=", "-=", "*=", "/=", "%=", "**=", "<<=", ">>=", ">>>=", "|=", "&=", "^=", "??=",
        "||=", "&&=",
    ];
    for op in ASSIGNMENTS {
        let source = format!("a {op} b {op} c");
        let expected = format!("a {op} b {op} c;\n");
        expect_printed(&source, &expected);
        let source = format!("a {op} (b {op} c)");
        expect_printed(&source, &expected);
    }
}

#[test]
#[ignore = "depends on esbuild comment metadata"]
fn test_comments() {
    expect_printed("return //\n x", "return;\nx;\n");
    expect_printed("return /**/\n x", "return;\nx;\n");
    expect_printed_script("return <!--\n x", "return;\nx;\n");
    expect_printed_script("-->\nx", "x;\n");
    expect_printed_script("x\n-->\ny", "x;\ny;\n");
    expect_printed_script("x\n -->\ny", "x;\ny;\n");
    expect_printed_script("x\n/**/-->\ny", "x;\ny;\n");
    expect_printed_script("x/*\n*/-->\ny", "x;\ny;\n");
    expect_printed_script("x\n/**/ /**/-->\ny", "x;\ny;\n");
    expect_printed("if(x-->y)z", "if (x-- > y) z;\n");
}

#[test]
#[ignore = "depends on esbuild directive and comment metadata"]
fn test_strict_mode() {
    expect_printed("'use strict'", "\"use strict\";\n");
    expect_printed("`use strict`", "`use strict`;\n");
    expect_printed(
        "//! @legal comment\n 'use strict'",
        "\"use strict\";\n//! @legal comment\n",
    );
    expect_printed(
        "/*! @legal comment */ 'use strict'",
        "\"use strict\";\n/*! @legal comment */\n",
    );
    expect_printed(
        "function f() { //! @legal comment\n 'use strict' }",
        "function f() {\n  //! @legal comment\n  \"use strict\";\n}\n",
    );
    expect_printed(
        "function f() { /*! @legal comment */ 'use strict' }",
        "function f() {\n  /*! @legal comment */\n  \"use strict\";\n}\n",
    );
    expect_printed("let x = '\\0'", "let x = \"\\0\";\n");
    expect_printed("let x = '\\00'", "let x = \"\\0\";\n");
    expect_printed(
        "'use strict'; let x = '\\0'",
        "\"use strict\";\nlet x = \"\\0\";\n",
    );
    expect_printed("let x = '\\0'; export {}", "let x = \"\\0\";\nexport {};\n");
    expect_printed("'\\0'", "\"\\0\";\n");
    expect_printed("'\\00'", "\"\\0\";\n");
    expect_printed("'use strict'; '\\0'", "\"use strict\";\n\"\\0\";\n");
    expect_printed("with (x) y", "with (x) y;\n");
    expect_printed("delete x", "delete x;\n");
    expect_printed("for (var x = y in z) ;", "x = y;\nfor (var x in z) ;\n");
    expect_printed("function f(a, a) {}", "function f(a, a) {\n}\n");
    expect_printed("(function(a, a) {})", "(function(a, a) {\n});\n");
    expect_printed(
        "({ f: function(a, a) {} })",
        "({ f: function(a, a) {\n} });\n",
    );
    expect_printed(
        "({ f: function*(a, a) {} })",
        "({ f: function* (a, a) {\n} });\n",
    );
    expect_printed(
        "({ f: async function(a, a) {} })",
        "({ f: async function(a, a) {\n} });\n",
    );
    expect_printed("eval++", "eval++;\n");
    expect_printed("eval = 0", "eval = 0;\n");
    expect_printed("eval += 0", "eval += 0;\n");
    expect_printed("[eval] = 0", "[eval] = 0;\n");
    expect_printed("arguments++", "arguments++;\n");
    expect_printed("arguments = 0", "arguments = 0;\n");
    expect_printed("arguments += 0", "arguments += 0;\n");
    expect_printed("[arguments] = 0", "[arguments] = 0;\n");
    expect_printed("function eval() {}", "function eval() {\n}\n");
    expect_printed("function arguments() {}", "function arguments() {\n}\n");
    expect_printed("function f(eval) {}", "function f(eval) {\n}\n");
    expect_printed("function f(arguments) {}", "function f(arguments) {\n}\n");
    expect_printed("({ f(eval) {} })", "({ f(eval) {\n} });\n");
    expect_printed("({ f(arguments) {} })", "({ f(arguments) {\n} });\n");
    expect_printed("let protected", "let protected;\n");
    expect_printed("let protecte\\u0064", "let protected;\n");
    expect_printed("let x = protected", "let x = protected;\n");
    expect_printed("let x = protecte\\u0064", "let x = protected;\n");
    expect_printed("0123", "83;\n");
    expect_printed("({0123: 4})", "({ 83: 4 });\n");
    expect_printed("let {0123: x} = y", "let { 83: x } = y;\n");
    expect_printed(
        "function f() { 'use strict' } with (x) y",
        "function f() {\n  \"use strict\";\n}\nwith (x) y;\n",
    );
    expect_printed(
        "with (x) y; function f() { 'use strict' }",
        "with (x) y;\nfunction f() {\n  \"use strict\";\n}\n",
    );
    expect_printed("class f {} with (x) y", "class f {\n}\nwith (x) y;\n");
    expect_printed("with (x) y; class f {}", "with (x) y;\nclass f {\n}\n");
    expect_printed("`use strict`; with (x) y", "`use strict`;\nwith (x) y;\n");
    expect_printed(
        "{ 'use strict'; with (x) y }",
        "{\n  \"use strict\";\n  with (x) y;\n}\n",
    );
    expect_printed(
        "if (0) { 'use strict'; with (x) y }",
        "if (0) {\n  \"use strict\";\n  with (x) y;\n}\n",
    );
    expect_printed(
        "while (0) { 'use strict'; with (x) y }",
        "while (0) {\n  \"use strict\";\n  with (x) y;\n}\n",
    );
    expect_printed(
        "try { 'use strict'; with (x) y } catch {}",
        "try {\n  \"use strict\";\n  with (x) y;\n} catch {\n}\n",
    );
    expect_printed(
        "try {} catch { 'use strict'; with (x) y }",
        "try {\n} catch {\n  \"use strict\";\n  with (x) y;\n}\n",
    );
    expect_printed(
        "try {} finally { 'use strict'; with (x) y }",
        "try {\n} finally {\n  \"use strict\";\n  with (x) y;\n}\n",
    );
    expect_printed("import(x); with (y) z", "import(x);\nwith (y) z;\n");
    expect_printed("import('x'); with (y) z", "import(\"x\");\nwith (y) z;\n");
    expect_printed("with (y) z; import(x)", "with (y) z;\nimport(x);\n");
    expect_printed("with (y) z; import('x')", "with (y) z;\nimport(\"x\");\n");
    expect_printed("(import(x)); with (y) z", "import(x);\nwith (y) z;\n");
    expect_printed("(import('x')); with (y) z", "import(\"x\");\nwith (y) z;\n");
    expect_printed("with (y) z; (import(x))", "with (y) z;\nimport(x);\n");
    expect_printed("with (y) z; (import('x'))", "with (y) z;\nimport(\"x\");\n");
}

#[test]
fn test_exponentiation() {
    expect_printed("--x ** 2", "--x ** 2;\n");
    expect_printed("++x ** 2", "++x ** 2;\n");
    expect_printed("x-- ** 2", "x-- ** 2;\n");
    expect_printed("x++ ** 2", "x++ ** 2;\n");
    expect_printed("(-x) ** 2", "(-x) ** 2;\n");
    expect_printed("(+x) ** 2", "(+x) ** 2;\n");
    expect_printed("(~x) ** 2", "(~x) ** 2;\n");
    expect_printed("(!x) ** 2", "(!x) ** 2;\n");
    expect_printed("(-1) ** 2", "(-1) ** 2;\n");
    // Depends on esbuild's parser-stage constant folding.
    // expect_printed("(+1) ** 2", "1 ** 2;\n");
    expect_printed("(~1) ** 2", "(~1) ** 2;\n");
    // Depends on esbuild's parser-stage constant folding.
    // expect_printed("(!1) ** 2", "false ** 2;\n");
    expect_printed("(void x) ** 2", "(void x) ** 2;\n");
    expect_printed("(delete x) ** 2", "(delete x) ** 2;\n");
    expect_printed("(typeof x) ** 2", "(typeof x) ** 2;\n");
    // Depends on esbuild's parser-stage undefined normalization.
    // expect_printed("undefined ** 2", "(void 0) ** 2;\n");
    expect_printed("(delete x) ** 0", "(delete x) ** 0;\n");
    expect_printed("(delete x.prop) ** 0", "(delete x.prop) ** 0;\n");
    expect_printed("(delete x[0]) ** 0", "(delete x[0]) ** 0;\n");
    expect_printed("(delete x?.prop) ** 0", "(delete x?.prop) ** 0;\n");
    expect_printed("(void x) ** 0", "(void x) ** 0;\n");
    expect_printed("(typeof x) ** 0", "(typeof x) ** 0;\n");
    expect_printed("(+x) ** 0", "(+x) ** 0;\n");
    expect_printed("(-x) ** 0", "(-x) ** 0;\n");
    expect_printed("(~x) ** 0", "(~x) ** 0;\n");
    expect_printed("(!x) ** 0", "(!x) ** 0;\n");
    expect_printed("(await x) ** 0", "(await x) ** 0;\n");
    expect_printed("(await -x) ** 0", "(await -x) ** 0;\n");
}

#[test]
fn test_await() {
    expect_printed("await x", "await x;\n");
    expect_printed("await +x", "await +x;\n");
    expect_printed("await -x", "await -x;\n");
    expect_printed("await ~x", "await ~x;\n");
    expect_printed("await !x", "await !x;\n");
    expect_printed("await --x", "await --x;\n");
    expect_printed("await ++x", "await ++x;\n");
    expect_printed("await x--", "await x--;\n");
    expect_printed("await x++", "await x++;\n");
    expect_printed("await void x", "await void x;\n");
    expect_printed("await typeof x", "await typeof x;\n");
    expect_printed("await (x * y)", "await (x * y);\n");
    expect_printed("await (x ** y)", "await (x ** y);\n");
    expect_printed(
        "async function f() { await delete x }",
        "async function f() {\n  await delete x;\n}\n",
    );
}

#[test]
fn test_reg_exp() {
    expect_printed("/x/d", "/x/d;\n");
    expect_printed("/x/g", "/x/g;\n");
    expect_printed("/x/i", "/x/i;\n");
    expect_printed("/x/m", "/x/m;\n");
    expect_printed("/x/s", "/x/s;\n");
    expect_printed("/x/u", "/x/u;\n");
    expect_printed("/x/y", "/x/y;\n");
    expect_printed("/[\\])]/", "/[\\])]/;\n");
}

#[test]
fn test_unicode_identifier_names() {
    expect_printed("x = {x\u{30fb}: 0}", "x = { \"x\u{30fb}\": 0 };\n");
    expect_printed("x = {x\u{ff65}: 0}", "x = { \"x\u{ff65}\": 0 };\n");
    expect_printed("x = {x\u{3c0}: 0}", "x = { x\u{3c0}: 0 };\n");
    expect_printed("x = y.x\u{30fb}", "x = y[\"x\u{30fb}\"];\n");
    expect_printed("x = y.x\u{ff65}", "x = y[\"x\u{ff65}\"];\n");
    expect_printed("x = y.x\u{3c0}", "x = y.x\u{3c0};\n");
}

#[test]
fn test_identifier_escapes() {
    expect_printed("var _\\u0076\\u0061\\u0072", "var _var;\n");
    expect_printed("foo._\\u0076\\u0061\\u0072", "foo._var;\n");
    expect_printed("foo.\\u0076\\u0061\\u0072", "foo.var;\n");
}

#[test]
fn test_special_identifiers() {
    expect_printed("exports", "exports;\n");
    expect_printed("require", "require;\n");
    expect_printed("module", "module;\n");
}

#[test]
fn test_decls() {
    expect_printed("var {[x]: y} = {}", "var { [x]: y } = {};\n");
    expect_printed("var {...x} = {}", "var { ...x } = {};\n");
    expect_printed("var [...x] = []", "var [...x] = [];\n");
    expect_printed("var {...x} = {}", "var { ...x } = {};\n");
    expect_printed("([...x] = []) => {}", "([...x] = []) => {\n};\n");
    expect_printed("({...x} = {}) => {}", "({ ...x } = {}) => {\n};\n");
    expect_printed("[b, ...c] = d", "[b, ...c] = d;\n");
    expect_printed("([b, ...c] = d)", "[b, ...c] = d;\n");
    expect_printed("({b, ...c} = d)", "({ b, ...c } = d);\n");
    expect_printed("({a = b} = c)", "({ a = b } = c);\n");
    expect_printed("({a: b = c} = d)", "({ a: b = c } = d);\n");
    expect_printed("({a: b.c} = d)", "({ a: b.c } = d);\n");
    expect_printed("[a = {}] = b", "[a = {}] = b;\n");
    expect_printed("[[...a, b].x] = c", "[[...a, b].x] = c;\n");
    expect_printed("[{...a, b}.x] = c", "[{ ...a, b }.x] = c;\n");
    expect_printed("({x: [...a, b].x} = c)", "({ x: [...a, b].x } = c);\n");
    expect_printed("({x: {...a, b}.x} = c)", "({ x: { ...a, b }.x } = c);\n");
    expect_printed("[x = [...a, b]] = c", "[x = [...a, b]] = c;\n");
    expect_printed("[x = {...a, b}] = c", "[x = { ...a, b }] = c;\n");
    expect_printed("({x = [...a, b]} = c)", "({ x = [...a, b] } = c);\n");
    expect_printed("({x = {...a, b}} = c)", "({ x = { ...a, b } } = c);\n");
    expect_printed("(x = y)", "x = y;\n");
    expect_printed("([] = [])", "[] = [];\n");
    expect_printed("({} = {})", "({} = {});\n");
    expect_printed("([[]] = [[]])", "[[]] = [[]];\n");
    expect_printed("({x: {}} = {x: {}})", "({ x: {} } = { x: {} });\n");
    expect_printed("(x) = y", "x = y;\n");
    expect_printed("for (x in y) ;", "for (x in y) ;\n");
    expect_printed("for ([] in y) ;", "for ([] in y) ;\n");
    expect_printed("for ({} in y) ;", "for ({} in y) ;\n");
    expect_printed("for ((x) in y) ;", "for (x in y) ;\n");
    expect_printed("for (x of y) ;", "for (x of y) ;\n");
    expect_printed("for ([] of y) ;", "for ([] of y) ;\n");
    expect_printed("for ({} of y) ;", "for ({} of y) ;\n");
    expect_printed("for ((x) of y) ;", "for (x of y) ;\n");
    expect_printed("for ([{a = {}}] in b) {}", "for ([{ a = {} }] in b) {\n}\n");
    expect_printed("for ([{a = {}}] of b) {}", "for ([{ a = {} }] of b) {\n}\n");
    expect_printed("for ({a = {}} in b) {}", "for ({ a = {} } in b) {\n}\n");
    expect_printed("for ({a = {}} of b) {}", "for ({ a = {} } of b) {\n}\n");
}

#[test]
#[ignore = "depends on esbuild parser lowering"]
fn test_for() {
    expect_printed("for (; of; ) ;", "for (; of; ) ;\n");
    expect_printed("for (a in b) ;", "for (a in b) ;\n");
    expect_printed("for (var a in b) ;", "for (var a in b) ;\n");
    expect_printed("for (let a in b) ;", "for (let a in b) ;\n");
    expect_printed("for (const a in b) ;", "for (const a in b) ;\n");
    expect_printed("for (a in b, c) ;", "for (a in b, c) ;\n");
    expect_printed("for (a in b = c) ;", "for (a in b = c) ;\n");
    expect_printed("for (var a in b, c) ;", "for (var a in b, c) ;\n");
    expect_printed("for (var a in b = c) ;", "for (var a in b = c) ;\n");
    expect_printed("for (a of b) ;", "for (a of b) ;\n");
    expect_printed("for (var a of b) ;", "for (var a of b) ;\n");
    expect_printed("for (let a of b) ;", "for (let a of b) ;\n");
    expect_printed("for (const a of b) ;", "for (const a of b) ;\n");
    expect_printed("for (a of b = c) ;", "for (a of b = c) ;\n");
    expect_printed("for (var a of b = c) ;", "for (var a of b = c) ;\n");
    expect_printed("for ((let) of bar);", "for ((let) of bar) ;\n");
    expect_printed("for ((let).foo of bar);", "for ((let).foo of bar) ;\n");
    expect_printed("for ((let.foo) of bar);", "for ((let).foo of bar) ;\n");
    expect_printed("for ((let``.foo) of bar);", "for ((let)``.foo of bar) ;\n");
    expect_printed("for (var x = 0 in y) ;", "x = 0;\nfor (var x in y) ;\n");
    expect_printed(
        "for (var x = () => a in b);",
        "x = () => a;\nfor (var x in b) ;\n",
    );
    expect_printed(
        "for (var x = a + b in c);",
        "x = a + b;\nfor (var x in c) ;\n",
    );
    expect_printed(
        "for (var x = `${y in z}`;;);",
        "for (var x = `${y in z}`; ; ) ;\n",
    );
    expect_printed(
        "for (var {[x in y]: z} = {};;);",
        "for (var { [x in y]: z } = {}; ; ) ;\n",
    );
    expect_printed(
        "for (var {x = y in z} = {};;);",
        "for (var { x = y in z } = {}; ; ) ;\n",
    );
    expect_printed(
        "for (var [x = y in z] = {};;);",
        "for (var [x = y in z] = {}; ; ) ;\n",
    );
    expect_printed(
        "for (var {x: y = z in w} = {};;);",
        "for (var { x: y = z in w } = {}; ; ) ;\n",
    );
    expect_printed(
        "for (var x = (a in b);;);",
        "for (var x = (a in b); ; ) ;\n",
    );
    expect_printed(
        "for (var x = [a in b];;);",
        "for (var x = [a in b]; ; ) ;\n",
    );
    expect_printed(
        "for (var x = y(a in b);;);",
        "for (var x = y(a in b); ; ) ;\n",
    );
    expect_printed(
        "for (var x = {y: a in b};;);",
        "for (var x = { y: a in b }; ; ) ;\n",
    );
    expect_printed("for (a ? b in c : d;;);", "for (a ? b in c : d; ; ) ;\n");
    expect_printed(
        "for (var x = () => { a in b };;);",
        "for (var x = () => {\n  a in b;\n}; ; ) ;\n",
    );
    expect_printed(
        "for (var x = async () => { a in b };;);",
        "for (var x = async () => {\n  a in b;\n}; ; ) ;\n",
    );
    expect_printed(
        "for (var x = function() { a in b };;);",
        "for (var x = function() {\n  a in b;\n}; ; ) ;\n",
    );
    expect_printed(
        "for (var x = async function() { a in b };;);",
        "for (var x = async function() {\n  a in b;\n}; ; ) ;\n",
    );
    expect_printed(
        "for (var x = class { [a in b]() {} };;);",
        "for (var x = class {\n  [a in b]() {\n  }\n}; ; ) ;\n",
    );
    expect_printed("async of => {}", "async (of) => {\n};\n");
    expect_printed("for ((async) of []) ;", "for ((async) of []) ;\n");
    expect_printed("for (async.x of []) ;", "for (async.x of []) ;\n");
    expect_printed(
        "for (async of => {};;) ;",
        "for (async (of) => {\n}; ; ) ;\n",
    );
    expect_printed("for (\\u0061sync of []) ;", "for ((async) of []) ;\n");
    expect_printed("for await (async of []) ;", "for await (async of []) ;\n");
}

#[test]
#[ignore = "depends on esbuild parser ASI behavior"]
fn test_asi() {
    expect_printed("return\n0", "return;\n0;\n");
    expect_printed("0\n[1]", "0[1];\n");
    expect_printed("0\n(1)", "0(1);\n");
    expect_printed("new x\n(1)", "new x(1);\n");
    expect_printed("while (true) break\nx", "while (true) break;\nx;\n");
    expect_printed("x\n!y", "x;\n!y;\n");
    expect_printed("x\n++y", "x;\n++y;\n");
    expect_printed("x\n--y", "x;\n--y;\n");
    expect_printed(
        "function* foo(){yield\na}",
        "function* foo() {\n  yield;\n  a;\n}\n",
    );
    expect_printed(
        "function* foo(){yield*\na}",
        "function* foo() {\n  yield* a;\n}\n",
    );
    expect_printed("async\nx => {}", "async;\n(x) => {\n};\n");
    expect_printed("async\nfunction foo() {}", "async;\nfunction foo() {\n}\n");
    expect_printed(
        "export default async\nx => {}",
        "export default async;\n(x) => {\n};\n",
    );
    expect_printed(
        "export default async\nfunction foo() {}",
        "export default async;\nfunction foo() {\n}\n",
    );
    expect_printed("if (0) let\nx = 0", "if (0) let;\nx = 0;\n");
    expect_printed("if (0) let\n{x}", "if (0) let;\n{\n  x;\n}\n");
    expect_printed(
        "function *foo() { if (0) let\nyield 0 }",
        "function* foo() {\n  if (0) let;\n  yield 0;\n}\n",
    );
    expect_printed(
        "async function foo() { if (0) let\nawait 0 }",
        "async function foo() {\n  if (0) let;\n  await 0;\n}\n",
    );
    expect_printed("let\nx = 0", "let x = 0;\n");
    expect_printed("let\n{x} = 0", "let { x } = 0;\n");
    expect_printed("let\n[x] = 0", "let [x] = 0;\n");
    expect_printed("do x;while(y)z", "do\n  x;\nwhile (y);\nz;\n");
    expect_printed("do x;while(y);z", "do\n  x;\nwhile (y);\nz;\n");
    expect_printed("{do x;while(y)}", "{\n  do\n    x;\n  while (y);\n}\n");
}

#[test]
fn test_local() {
    expect_printed("var let = 0", "var let = 0;\n");
    expect_printed("var\nlet = 0", "var let = 0;\n");
    expect_printed("for (var let in x) ;", "for (var let in x) ;\n");
    expect_printed("for (var let of x) ;", "for (var let of x) ;\n");
}

#[test]
fn test_arrays() {
    expect_printed("[]", "[];\n");
    expect_printed("[,]", "[,];\n");
    expect_printed("[1]", "[1];\n");
    expect_printed("[1,]", "[1];\n");
    expect_printed("[,1]", "[, 1];\n");
    expect_printed("[1,2]", "[1, 2];\n");
    expect_printed("[,1,2]", "[, 1, 2];\n");
    expect_printed("[1,,2]", "[1, , 2];\n");
    expect_printed("[1,2,]", "[1, 2];\n");
    expect_printed("[1,2,,]", "[1, 2, ,];\n");
}

#[test]
fn test_pattern() {
    expect_printed("let {if: x} = y", "let { if: x } = y;\n");
    expect_printed("let {1_2_3n: x} = y", "let { 123n: x } = y;\n");
    expect_printed("let {0x1_2_3n: x} = y", "let { 0x123n: x } = y;\n");
}

#[test]
fn test_object() {
    expect_printed("({foo})", "({ foo });\n");
    expect_printed("({foo:0})", "({ foo: 0 });\n");
    expect_printed("({1e9:0})", "({ 1e9: 0 });\n");
    expect_printed("({1_2_3n:0})", "({ 123n: 0 });\n");
    expect_printed("({0x1_2_3n:0})", "({ 0x123n: 0 });\n");
    expect_printed("({foo() {}})", "({ foo() {\n} });\n");
    expect_printed("({*foo() {}})", "({ *foo() {\n} });\n");
    expect_printed("({get foo() {}})", "({ get foo() {\n} });\n");
    expect_printed("({set foo(x) {}})", "({ set foo(x) {\n} });\n");
    expect_printed("({if:0})", "({ if: 0 });\n");
    expect_printed("({if() {}})", "({ if() {\n} });\n");
    expect_printed("({*if() {}})", "({ *if() {\n} });\n");
    expect_printed("({get if() {}})", "({ get if() {\n} });\n");
    expect_printed("({set if(x) {}})", "({ set if(x) {\n} });\n");
    expect_printed("x = { get \n x() {} }", "x = { get x() {\n} };\n");
    expect_printed("x = { set \n x(_) {} }", "x = { set x(_) {\n} };\n");
}

#[test]
fn test_computed_property() {
    expect_printed("({[a]: foo})", "({ [a]: foo });\n");
    expect_printed("({[(a, b)]: foo})", "({ [(a, b)]: foo });\n");
    expect_printed("({[a]: foo}) => {}", "({ [a]: foo }) => {\n};\n");
    expect_printed("({[(a, b)]: foo}) => {}", "({ [(a, b)]: foo }) => {\n};\n");
    expect_printed("var {[a]: foo} = bar", "var { [a]: foo } = bar;\n");
    expect_printed(
        "var {[(a, b)]: foo} = bar",
        "var { [(a, b)]: foo } = bar;\n",
    );
    expect_printed("class Foo {[a] = foo}", "class Foo {\n  [a] = foo;\n}\n");
    expect_printed(
        "class Foo {[(a, b)] = foo}",
        "class Foo {\n  [(a, b)] = foo;\n}\n",
    );
}

#[test]
fn test_quoted_property() {
    expect_printed("x.x; y['y']", "x.x;\ny[\"y\"];\n");
    expect_printed("({y: y, 'z': z} = x)", "({ y, \"z\": z } = x);\n");
    expect_printed("var {y: y, 'z': z} = x", "var { y, \"z\": z } = x;\n");
    expect_printed("x = {y: 1, 'z': 2}", "x = { y: 1, \"z\": 2 };\n");
    expect_printed(
        "x = {y() {}, 'z'() {}}",
        "x = { y() {\n}, \"z\"() {\n} };\n",
    );
    expect_printed(
        "x = {get y() {}, set 'z'(z) {}}",
        "x = { get y() {\n}, set \"z\"(z) {\n} };\n",
    );
    expect_printed(
        "x = class {y = 1; 'z' = 2}",
        "x = class {\n  y = 1;\n  \"z\" = 2;\n};\n",
    );
    expect_printed(
        "x = class {y() {}; 'z'() {}}",
        "x = class {\n  y() {\n  }\n  \"z\"() {\n  }\n};\n",
    );
    expect_printed(
        "x = class {get y() {}; set 'z'(z) {}}",
        "x = class {\n  get y() {\n  }\n  set \"z\"(z) {\n  }\n};\n",
    );
}

#[test]
#[ignore = "depends on esbuild parser lowering"]
fn test_lexical_decl() {
    expect_printed("if (1) var x", "if (1) var x;\n");
    expect_printed(
        "if (1) function x() {}",
        "if (1) {\n  let x = function() {\n  };\n  var x = x;\n}\n",
    );
    expect_printed(
        "if (1) {} else function x() {}",
        "if (1) {\n} else {\n  let x = function() {\n  };\n  var x = x;\n}\n",
    );
    expect_printed(
        "switch (1) { case 1: const x = 1 }",
        "switch (1) {\n  case 1:\n    const x = 1;\n}\n",
    );
    expect_printed(
        "switch (1) { default: const x = 1 }",
        "switch (1) {\n  default:\n    const x = 1;\n}\n",
    );
    expect_printed("function f() {}", "function f() {\n}\n");
    expect_printed(
        "{function f() {}} let f",
        "{\n  let f = function() {\n  };\n}\nlet f;\n",
    );
    expect_printed(
        "if (1) function f() {} let f",
        "if (1) {\n  let f = function() {\n  };\n}\nlet f;\n",
    );
    expect_printed(
        "if (0) ; else function f() {} let f",
        "if (0) ;\nelse {\n  let f = function() {\n  };\n}\nlet f;\n",
    );
    expect_printed(
        "x: function f() {}",
        "x: {\n  let f = function() {\n  };\n  var f = f;\n}\n",
    );
    expect_printed(
        "{function* f() {}} let f",
        "{\n  function* f() {\n  }\n}\nlet f;\n",
    );
    expect_printed(
        "{async function f() {}} let f",
        "{\n  async function f() {\n  }\n}\nlet f;\n",
    );
    expect_printed(
        "if (foo) { function x() {} }",
        "if (foo) {\n  let x = function() {\n  };\n  var x = x;\n}\n",
    );
    expect_printed(
        "if (foo) { function x() {} eval('') }",
        "if (foo) {\n  function x() {\n  }\n  eval(\"\");\n}\n",
    );
    expect_printed(
        "if (foo) { function x() {} if (bar) { eval('') } }",
        "if (foo) {\n  function x() {\n  }\n  if (bar) {\n    eval(\"\");\n  }\n}\n",
    );
    expect_printed(
        "if (foo) { eval(''); function x() {} }",
        "if (foo) {\n  function x() {\n  }\n  eval(\"\");\n}\n",
    );
    expect_printed(
        "'use strict'; if (foo) { function x() {} }",
        "\"use strict\";\nif (foo) {\n  let x = function() {\n  };\n}\n",
    );
    expect_printed(
        "'use strict'; if (foo) { function x() {} eval('') }",
        "\"use strict\";\nif (foo) {\n  function x() {\n  }\n  eval(\"\");\n}\n",
    );
    expect_printed(
        "'use strict'; if (foo) { function x() {} if (bar) { eval('') } }",
        "\"use strict\";\nif (foo) {\n  function x() {\n  }\n  if (bar) {\n    eval(\"\");\n  }\n}\n",
    );
    expect_printed(
        "'use strict'; if (foo) { eval(''); function x() {} }",
        "\"use strict\";\nif (foo) {\n  function x() {\n  }\n  eval(\"\");\n}\n",
    );
}

#[test]
fn test_function() {
    expect_printed(
        "function f() {} function f() {}",
        "function f() {\n}\nfunction f() {\n}\n",
    );
    expect_printed(
        "function f() {} function* f() {}",
        "function f() {\n}\nfunction* f() {\n}\n",
    );
    expect_printed(
        "function* f() {} function* f() {}",
        "function* f() {\n}\nfunction* f() {\n}\n",
    );
    expect_printed(
        "function f() {} async function f() {}",
        "function f() {\n}\nasync function f() {\n}\n",
    );
    expect_printed(
        "async function f() {} async function f() {}",
        "async function f() {\n}\nasync function f() {\n}\n",
    );
    expect_printed("function arguments() {}", "function arguments() {\n}\n");
    expect_printed(
        "(function arguments() {})",
        "(function arguments() {\n});\n",
    );
    expect_printed(
        "function foo(arguments) {}",
        "function foo(arguments) {\n}\n",
    );
    expect_printed(
        "(function foo(arguments) {})",
        "(function foo(arguments) {\n});\n",
    );
    expect_printed(
        "(function foo() { var arguments })",
        "(function foo() {\n  var arguments;\n});\n",
    );
    expect_printed(
        "(function foo() { { var arguments } })",
        "(function foo() {\n  {\n    var arguments;\n  }\n});\n",
    );
    expect_printed(
        "var x = function() { y() }()",
        "var x = function() {\n  y();\n}();\n",
    );
    // Depends on esbuild's parser-stage constant folding.
    // expect_printed(
    //     "var x = (true && function() { y() })()",
    //     "var x = (function() {\n  y();\n})();\n",
    // );
}

#[test]
fn test_class() {
    expect_printed("class Foo { foo() {} }", "class Foo {\n  foo() {\n  }\n}\n");
    expect_printed(
        "class Foo { *foo() {} }",
        "class Foo {\n  *foo() {\n  }\n}\n",
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
        "class Foo { async foo() {} }",
        "class Foo {\n  async foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { async *foo() {} }",
        "class Foo {\n  async *foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static foo() {} }",
        "class Foo {\n  static foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static *foo() {} }",
        "class Foo {\n  static *foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static get foo() {} }",
        "class Foo {\n  static get foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static set foo(x) {} }",
        "class Foo {\n  static set foo(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static async foo() {} }",
        "class Foo {\n  static async foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static async *foo() {} }",
        "class Foo {\n  static async *foo() {\n  }\n}\n",
    );
    expect_printed("class Foo { if() {} }", "class Foo {\n  if() {\n  }\n}\n");
    expect_printed("class Foo { *if() {} }", "class Foo {\n  *if() {\n  }\n}\n");
    expect_printed(
        "class Foo { get if() {} }",
        "class Foo {\n  get if() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { set if(x) {} }",
        "class Foo {\n  set if(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { async if() {} }",
        "class Foo {\n  async if() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { async *if() {} }",
        "class Foo {\n  async *if() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static if() {} }",
        "class Foo {\n  static if() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static *if() {} }",
        "class Foo {\n  static *if() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static get if() {} }",
        "class Foo {\n  static get if() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static set if(x) {} }",
        "class Foo {\n  static set if(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static async if() {} }",
        "class Foo {\n  static async if() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static async *if() {} }",
        "class Foo {\n  static async *if() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} b() {} }",
        "class Foo {\n  a() {\n  }\n  b() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} get b() {} }",
        "class Foo {\n  a() {\n  }\n  get b() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} set b(x) {} }",
        "class Foo {\n  a() {\n  }\n  set b(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} async b() {} }",
        "class Foo {\n  a() {\n  }\n  async b() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} async *b() {} }",
        "class Foo {\n  a() {\n  }\n  async *b() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} static b() {} }",
        "class Foo {\n  a() {\n  }\n  static b() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} static *b() {} }",
        "class Foo {\n  a() {\n  }\n  static *b() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} static get b() {} }",
        "class Foo {\n  a() {\n  }\n  static get b() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} static set b(x) {} }",
        "class Foo {\n  a() {\n  }\n  static set b(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} static async b() {} }",
        "class Foo {\n  a() {\n  }\n  static async b() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { a() {} static async *b() {} }",
        "class Foo {\n  a() {\n  }\n  static async *b() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { [arguments] }",
        "class Foo {\n  [arguments];\n}\n",
    );
    expect_printed(
        "class Foo { [arguments] = 1 }",
        "class Foo {\n  [arguments] = 1;\n}\n",
    );
    expect_printed(
        "class Foo { arguments = 1 }",
        "class Foo {\n  arguments = 1;\n}\n",
    );
    expect_printed(
        "class Foo { x = class { arguments = 1 } }",
        "class Foo {\n  x = class {\n    arguments = 1;\n  };\n}\n",
    );
    expect_printed(
        "class Foo { x = function() { arguments } }",
        "class Foo {\n  x = function() {\n    arguments;\n  };\n}\n",
    );
    expect_printed(
        "class Foo { get ['constructor']() {} }",
        "class Foo {\n  get [\"constructor\"]() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { set ['constructor'](x) {} }",
        "class Foo {\n  set [\"constructor\"](x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { *['constructor']() {} }",
        "class Foo {\n  *[\"constructor\"]() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { async ['constructor']() {} }",
        "class Foo {\n  async [\"constructor\"]() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { async *['constructor']() {} }",
        "class Foo {\n  async *[\"constructor\"]() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static get constructor() {} }",
        "class Foo {\n  static get constructor() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static get 'constructor'() {} }",
        "class Foo {\n  static get \"constructor\"() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static set constructor(x) {} }",
        "class Foo {\n  static set constructor(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static set 'constructor'(x) {} }",
        "class Foo {\n  static set \"constructor\"(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static *constructor() {} }",
        "class Foo {\n  static *constructor() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static *'constructor'() {} }",
        "class Foo {\n  static *\"constructor\"() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static async constructor() {} }",
        "class Foo {\n  static async constructor() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static async 'constructor'() {} }",
        "class Foo {\n  static async \"constructor\"() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static async *constructor() {} }",
        "class Foo {\n  static async *constructor() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static async *'constructor'() {} }",
        "class Foo {\n  static async *\"constructor\"() {\n  }\n}\n",
    );
    expect_printed("({ constructor: 1 })", "({ constructor: 1 });\n");
    expect_printed(
        "({ get constructor() {} })",
        "({ get constructor() {\n} });\n",
    );
    expect_printed(
        "({ set constructor(x) {} })",
        "({ set constructor(x) {\n} });\n",
    );
    expect_printed("({ *constructor() {} })", "({ *constructor() {\n} });\n");
    expect_printed(
        "({ async constructor() {} })",
        "({ async constructor() {\n} });\n",
    );
    expect_printed(
        "({ async* constructor() {} })",
        "({ async *constructor() {\n} });\n",
    );
    expect_printed(
        "class Foo { get prototype() {} }",
        "class Foo {\n  get prototype() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { get 'prototype'() {} }",
        "class Foo {\n  get \"prototype\"() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { set prototype(x) {} }",
        "class Foo {\n  set prototype(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { set 'prototype'(x) {} }",
        "class Foo {\n  set \"prototype\"(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { *prototype() {} }",
        "class Foo {\n  *prototype() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { *'prototype'() {} }",
        "class Foo {\n  *\"prototype\"() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { async prototype() {} }",
        "class Foo {\n  async prototype() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { async 'prototype'() {} }",
        "class Foo {\n  async \"prototype\"() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { async *prototype() {} }",
        "class Foo {\n  async *prototype() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { async *'prototype'() {} }",
        "class Foo {\n  async *\"prototype\"() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static get ['prototype']() {} }",
        "class Foo {\n  static get [\"prototype\"]() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static set ['prototype'](x) {} }",
        "class Foo {\n  static set [\"prototype\"](x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static *['prototype']() {} }",
        "class Foo {\n  static *[\"prototype\"]() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static async ['prototype']() {} }",
        "class Foo {\n  static async [\"prototype\"]() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static async *['prototype']() {} }",
        "class Foo {\n  static async *[\"prototype\"]() {\n  }\n}\n",
    );
    expect_printed("({ prototype: 1 })", "({ prototype: 1 });\n");
    expect_printed("({ get prototype() {} })", "({ get prototype() {\n} });\n");
    expect_printed(
        "({ set prototype(x) {} })",
        "({ set prototype(x) {\n} });\n",
    );
    expect_printed("({ *prototype() {} })", "({ *prototype() {\n} });\n");
    expect_printed(
        "({ async prototype() {} })",
        "({ async prototype() {\n} });\n",
    );
    expect_printed(
        "({ async* prototype() {} })",
        "({ async *prototype() {\n} });\n",
    );
    expect_printed(
        "class Foo { foo = [Foo, eval(bar)] }",
        "class Foo {\n  foo = [Foo, eval(bar)];\n}\n",
    );
    expect_printed(
        "class A { get \n x() {} }",
        "class A {\n  get x() {\n  }\n}\n",
    );
    expect_printed(
        "class A { set \n x(_) {} }",
        "class A {\n  set x(_) {\n  }\n}\n",
    );
    expect_printed(
        "class A { get \n *x() {} }",
        "class A {\n  get;\n  *x() {\n  }\n}\n",
    );
    expect_printed(
        "class A { set \n *x(_) {} }",
        "class A {\n  set;\n  *x(_) {\n  }\n}\n",
    );
}

#[test]
fn test_super_call() {
    expect_printed(
        "class Foo extends Bar { constructor() { super() } }",
        "class Foo extends Bar {\n  constructor() {\n    super();\n  }\n}\n",
    );
    expect_printed(
        "class Foo extends Bar { constructor() { () => super() } }",
        "class Foo extends Bar {\n  constructor() {\n    () => super();\n  }\n}\n",
    );
    expect_printed(
        "class Foo extends Bar { constructor() { () => { super() } } }",
        "class Foo extends Bar {\n  constructor() {\n    () => {\n      super();\n    };\n  }\n}\n",
    );
    expect_printed(
        "class Foo extends Bar { constructor(x = super()) {} }",
        "class Foo extends Bar {\n  constructor(x = super()) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo extends Bar { constructor(x = () => super()) {} }",
        "class Foo extends Bar {\n  constructor(x = () => super()) {\n  }\n}\n",
    );
}

#[test]
fn test_super_prop() {
    expect_printed(
        "class Foo { foo() { super.x } }",
        "class Foo {\n  foo() {\n    super.x;\n  }\n}\n",
    );
    expect_printed(
        "class Foo { foo() { super[x] } }",
        "class Foo {\n  foo() {\n    super[x];\n  }\n}\n",
    );
    expect_printed(
        "class Foo { foo(x = super.x) {} }",
        "class Foo {\n  foo(x = super.x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { foo(x = super[x]) {} }",
        "class Foo {\n  foo(x = super[x]) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static foo() { super.x } }",
        "class Foo {\n  static foo() {\n    super.x;\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static foo() { super[x] } }",
        "class Foo {\n  static foo() {\n    super[x];\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static foo(x = super.x) {} }",
        "class Foo {\n  static foo(x = super.x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static foo(x = super[x]) {} }",
        "class Foo {\n  static foo(x = super[x]) {\n  }\n}\n",
    );
    expect_printed(
        "(class { foo() { super.x } })",
        "(class {\n  foo() {\n    super.x;\n  }\n});\n",
    );
    expect_printed(
        "(class { foo() { super[x] } })",
        "(class {\n  foo() {\n    super[x];\n  }\n});\n",
    );
    expect_printed(
        "(class { foo(x = super.x) {} })",
        "(class {\n  foo(x = super.x) {\n  }\n});\n",
    );
    expect_printed(
        "(class { foo(x = super[x]) {} })",
        "(class {\n  foo(x = super[x]) {\n  }\n});\n",
    );
    expect_printed(
        "(class { static foo() { super.x } })",
        "(class {\n  static foo() {\n    super.x;\n  }\n});\n",
    );
    expect_printed(
        "(class { static foo() { super[x] } })",
        "(class {\n  static foo() {\n    super[x];\n  }\n});\n",
    );
    expect_printed(
        "(class { static foo(x = super.x) {} })",
        "(class {\n  static foo(x = super.x) {\n  }\n});\n",
    );
    expect_printed(
        "(class { static foo(x = super[x]) {} })",
        "(class {\n  static foo(x = super[x]) {\n  }\n});\n",
    );
    expect_printed(
        "class Foo { foo = super.x }",
        "class Foo {\n  foo = super.x;\n}\n",
    );
    expect_printed(
        "class Foo { foo = super[x] }",
        "class Foo {\n  foo = super[x];\n}\n",
    );
    expect_printed(
        "class Foo { foo = () => super.x }",
        "class Foo {\n  foo = () => super.x;\n}\n",
    );
    expect_printed(
        "class Foo { foo = () => super[x] }",
        "class Foo {\n  foo = () => super[x];\n}\n",
    );
    expect_printed(
        "class Foo { static foo = super.x }",
        "class Foo {\n  static foo = super.x;\n}\n",
    );
    expect_printed(
        "class Foo { static foo = super[x] }",
        "class Foo {\n  static foo = super[x];\n}\n",
    );
    expect_printed(
        "class Foo { static foo = () => super.x }",
        "class Foo {\n  static foo = () => super.x;\n}\n",
    );
    expect_printed(
        "class Foo { static foo = () => super[x] }",
        "class Foo {\n  static foo = () => super[x];\n}\n",
    );
    expect_printed(
        "(class { foo = super.x })",
        "(class {\n  foo = super.x;\n});\n",
    );
    expect_printed(
        "(class { foo = super[x] })",
        "(class {\n  foo = super[x];\n});\n",
    );
    expect_printed(
        "(class { foo = () => super.x })",
        "(class {\n  foo = () => super.x;\n});\n",
    );
    expect_printed(
        "(class { foo = () => super[x] })",
        "(class {\n  foo = () => super[x];\n});\n",
    );
    expect_printed(
        "(class { static foo = super.x })",
        "(class {\n  static foo = super.x;\n});\n",
    );
    expect_printed(
        "(class { static foo = super[x] })",
        "(class {\n  static foo = super[x];\n});\n",
    );
    expect_printed(
        "(class { static foo = () => super.x })",
        "(class {\n  static foo = () => super.x;\n});\n",
    );
    expect_printed(
        "(class { static foo = () => super[x] })",
        "(class {\n  static foo = () => super[x];\n});\n",
    );
    expect_printed("({ foo() { super.x } })", "({ foo() {\n  super.x;\n} });\n");
    expect_printed(
        "({ foo() { super[x] } })",
        "({ foo() {\n  super[x];\n} });\n",
    );
    expect_printed(
        "({ foo(x = super.x) {} })",
        "({ foo(x = super.x) {\n} });\n",
    );
}

#[test]
fn test_class_fields() {
    expect_printed("class Foo { a }", "class Foo {\n  a;\n}\n");
    expect_printed("class Foo { a = 1 }", "class Foo {\n  a = 1;\n}\n");
    expect_printed("class Foo { a = 1; b }", "class Foo {\n  a = 1;\n  b;\n}\n");
    expect_printed("class Foo { [a] }", "class Foo {\n  [a];\n}\n");
    expect_printed("class Foo { [a] = 1 }", "class Foo {\n  [a] = 1;\n}\n");
    expect_printed(
        "class Foo { [a] = 1; [b] }",
        "class Foo {\n  [a] = 1;\n  [b];\n}\n",
    );
    expect_printed("class Foo { static a }", "class Foo {\n  static a;\n}\n");
    expect_printed(
        "class Foo { static a = 1 }",
        "class Foo {\n  static a = 1;\n}\n",
    );
    expect_printed(
        "class Foo { static a = 1; b }",
        "class Foo {\n  static a = 1;\n  b;\n}\n",
    );
    expect_printed(
        "class Foo { static [a] }",
        "class Foo {\n  static [a];\n}\n",
    );
    expect_printed(
        "class Foo { static [a] = 1 }",
        "class Foo {\n  static [a] = 1;\n}\n",
    );
    expect_printed(
        "class Foo { static [a] = 1; [b] }",
        "class Foo {\n  static [a] = 1;\n  [b];\n}\n",
    );
    expect_printed(
        "class Foo { ['constructor'] }",
        "class Foo {\n  [\"constructor\"];\n}\n",
    );
    expect_printed(
        "class Foo { ['constructor'] = 1 }",
        "class Foo {\n  [\"constructor\"] = 1;\n}\n",
    );
    expect_printed(
        "class Foo { static ['constructor'] }",
        "class Foo {\n  static [\"constructor\"];\n}\n",
    );
    expect_printed(
        "class Foo { static ['constructor'] = 1 }",
        "class Foo {\n  static [\"constructor\"] = 1;\n}\n",
    );
    expect_printed("class Foo { prototype }", "class Foo {\n  prototype;\n}\n");
    expect_printed(
        "class Foo { 'prototype' }",
        "class Foo {\n  \"prototype\";\n}\n",
    );
    expect_printed(
        "class Foo { prototype = 1 }",
        "class Foo {\n  prototype = 1;\n}\n",
    );
    expect_printed(
        "class Foo { 'prototype' = 1 }",
        "class Foo {\n  \"prototype\" = 1;\n}\n",
    );
    expect_printed(
        "class Foo { static ['prototype'] }",
        "class Foo {\n  static [\"prototype\"];\n}\n",
    );
    expect_printed(
        "class Foo { static ['prototype'] = 1 }",
        "class Foo {\n  static [\"prototype\"] = 1;\n}\n",
    );
}

#[test]
fn test_class_static_blocks() {
    expect_printed(
        "class Foo { static {} }",
        "class Foo {\n  static {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static {} x = 1 }",
        "class Foo {\n  static {\n  }\n  x = 1;\n}\n",
    );
    expect_printed(
        "class Foo { static { this.foo() } }",
        "class Foo {\n  static {\n    this.foo();\n  }\n}\n",
    );
}

#[test]
fn test_auto_accessors() {
    expect_printed("class Foo { accessor }", "class Foo {\n  accessor;\n}\n");
    expect_printed(
        "class Foo { accessor \n x }",
        "class Foo {\n  accessor;\n  x;\n}\n",
    );
    expect_printed(
        "class Foo { static accessor }",
        "class Foo {\n  static accessor;\n}\n",
    );
    expect_printed(
        "class Foo { static accessor \n x }",
        "class Foo {\n  static accessor;\n  x;\n}\n",
    );
    expect_printed(
        "class Foo { accessor x }",
        "class Foo {\n  accessor x;\n}\n",
    );
    expect_printed(
        "class Foo { accessor x = y }",
        "class Foo {\n  accessor x = y;\n}\n",
    );
    expect_printed(
        "class Foo { accessor [x] }",
        "class Foo {\n  accessor [x];\n}\n",
    );
    expect_printed(
        "class Foo { accessor [x] = y }",
        "class Foo {\n  accessor [x] = y;\n}\n",
    );
    expect_printed(
        "class Foo { static accessor x }",
        "class Foo {\n  static accessor x;\n}\n",
    );
    expect_printed(
        "class Foo { static accessor [x] }",
        "class Foo {\n  static accessor [x];\n}\n",
    );
    expect_printed(
        "class Foo { static accessor x = y }",
        "class Foo {\n  static accessor x = y;\n}\n",
    );
    expect_printed(
        "class Foo { static accessor [x] = y }",
        "class Foo {\n  static accessor [x] = y;\n}\n",
    );
    expect_printed(
        "Foo = class { accessor x }",
        "Foo = class {\n  accessor x;\n};\n",
    );
    expect_printed(
        "Foo = class { accessor [x] }",
        "Foo = class {\n  accessor [x];\n};\n",
    );
    expect_printed(
        "Foo = class { accessor x = y }",
        "Foo = class {\n  accessor x = y;\n};\n",
    );
    expect_printed(
        "Foo = class { accessor [x] = y }",
        "Foo = class {\n  accessor [x] = y;\n};\n",
    );
    expect_printed(
        "Foo = class { static accessor x }",
        "Foo = class {\n  static accessor x;\n};\n",
    );
    expect_printed(
        "Foo = class { static accessor [x] }",
        "Foo = class {\n  static accessor [x];\n};\n",
    );
    expect_printed(
        "Foo = class { static accessor x = y }",
        "Foo = class {\n  static accessor x = y;\n};\n",
    );
    expect_printed(
        "class Foo { accessor get }",
        "class Foo {\n  accessor get;\n}\n",
    );
    expect_printed(
        "class Foo { get accessor() {} }",
        "class Foo {\n  get accessor() {\n  }\n}\n",
    );
    expect_printed(
        "Foo = { get accessor() {} }",
        "Foo = { get accessor() {\n} };\n",
    );
}

#[test]
fn test_decorators() {
    expect_printed("@x @y class Foo {}", "@x @y class Foo {\n}\n");
    expect_printed("@x @y export class Foo {}", "export @x @y class Foo {\n}\n");
    expect_printed(
        "@x @y export default class Foo {}",
        "export default @x @y class Foo {\n}\n",
    );
    expect_printed("_ = @x @y class {}", "_ = @x @y class {\n};\n");
    expect_printed("class Foo { @x y }", "class Foo {\n  @x y;\n}\n");
    expect_printed(
        "class Foo { @x y() {} }",
        "class Foo {\n  @x y() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { @x static y }",
        "class Foo {\n  @x static y;\n}\n",
    );
    expect_printed(
        "class Foo { @x static y() {} }",
        "class Foo {\n  @x static y() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { @x accessor y }",
        "class Foo {\n  @x accessor y;\n}\n",
    );
    expect_printed("class Foo { @x #y }", "class Foo {\n  @x #y;\n}\n");
    expect_printed(
        "class Foo { @x #y() {} }",
        "class Foo {\n  @x #y() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { @x static #y }",
        "class Foo {\n  @x static #y;\n}\n",
    );
    expect_printed(
        "class Foo { @x static #y() {} }",
        "class Foo {\n  @x static #y() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { @x accessor #y }",
        "class Foo {\n  @x accessor #y;\n}\n",
    );
    expect_printed(
        "@\na\n(\n)\n@\n(\nb\n)\nclass\nFoo\n{\n}\n",
        "@a() @b class Foo {\n}\n",
    );
    expect_printed("@(a, b) class Foo {}", "@(a, b) class Foo {\n}\n");
    expect_printed("@x() class Foo {}", "@x() class Foo {\n}\n");
    expect_printed("@x.y() class Foo {}", "@x.y() class Foo {\n}\n");
    expect_printed("@(() => {}) class Foo {}", "@(() => {\n}) class Foo {\n}\n");
    expect_printed(
        "class Foo { #x = @y.#x.y.#x class {} }",
        "class Foo {\n  #x = @y.#x.y.#x class {\n  };\n}\n",
    );
    expect_printed("@x export class Foo {}", "export @x class Foo {\n}\n");
    expect_printed("export @x class Foo {}", "export @x class Foo {\n}\n");
    expect_printed(
        "@x export default class {}",
        "export default @x class {\n}\n",
    );
    expect_printed(
        "export default @x class {}",
        "export default @x class {\n}\n",
    );
    expect_printed(
        "@x export default class Foo {}",
        "export default @x class Foo {\n}\n",
    );
    expect_printed(
        "export default @x class Foo {}",
        "export default @x class Foo {\n}\n",
    );
    expect_printed(
        "export default (@x class {})",
        "export default (@x class {\n});\n",
    );
    expect_printed(
        "export default (@x class Foo {})",
        "export default (@x class Foo {\n});\n",
    );
}

#[test]
fn test_generator() {
    expect_printed(
        "function* foo() { -(yield 100) }",
        "function* foo() {\n  -(yield 100);\n}\n",
    );
}

#[test]
fn test_yield() {
    expect_printed("yield\n100", "yield;\n100;\n");
    expect_printed(
        "function *foo() { x = yield }",
        "function* foo() {\n  x = yield;\n}\n",
    );
    expect_printed(
        "function *foo() { x = yield; }",
        "function* foo() {\n  x = yield;\n}\n",
    );
    expect_printed(
        "function *foo() { (x = yield) }",
        "function* foo() {\n  x = yield;\n}\n",
    );
    expect_printed(
        "function *foo() { [x = yield] }",
        "function* foo() {\n  [x = yield];\n}\n",
    );
    expect_printed(
        "function *foo() { x = (yield, yield) }",
        "function* foo() {\n  x = (yield, yield);\n}\n",
    );
    expect_printed(
        "function *foo() { x = y ? yield : yield }",
        "function* foo() {\n  x = y ? yield : yield;\n}\n",
    );
    expect_printed(
        "function *foo() { x = yield y }",
        "function* foo() {\n  x = yield y;\n}\n",
    );
    expect_printed(
        "function *foo() { (x = yield y) }",
        "function* foo() {\n  x = yield y;\n}\n",
    );
    expect_printed(
        "function *foo() { x = yield \n y }",
        "function* foo() {\n  x = yield;\n  y;\n}\n",
    );
    expect_printed(
        "function *foo() { x = yield * y }",
        "function* foo() {\n  x = yield* y;\n}\n",
    );
    expect_printed(
        "function *foo() { (x = yield * y) }",
        "function* foo() {\n  x = yield* y;\n}\n",
    );
    expect_printed(
        "function *foo() { x = yield * \n y }",
        "function* foo() {\n  x = yield* y;\n}\n",
    );
    expect_printed(
        "function foo() { x = yield * y }",
        "function foo() {\n  x = yield * y;\n}\n",
    );
    expect_printed(
        "function foo() { (x = yield * y) }",
        "function foo() {\n  x = yield * y;\n}\n",
    );
    expect_printed("({yield} = x)", "({ yield } = x);\n");
    expect_printed("let x = {yield}", "let x = { yield };\n");
    expect_printed("function* yield() {}", "function* yield() {\n}\n");
    expect_printed(
        "function foo() { ({yield} = x) }",
        "function foo() {\n  ({ yield } = x);\n}\n",
    );
    expect_printed(
        "function foo() { let x = {yield} }",
        "function foo() {\n  let x = { yield };\n}\n",
    );
    expect_printed("({ *yield() {} })", "({ *yield() {\n} });\n");
    expect_printed(
        "(class { *yield() {} })",
        "(class {\n  *yield() {\n  }\n});\n",
    );
    expect_printed(
        "class Foo { *yield() {} }",
        "class Foo {\n  *yield() {\n  }\n}\n",
    );
    expect_printed("function* yield() {}", "function* yield() {\n}\n");
    expect_printed("({ async *yield() {} })", "({ async *yield() {\n} });\n");
    expect_printed(
        "(class { async *yield() {} })",
        "(class {\n  async *yield() {\n  }\n});\n",
    );
    expect_printed(
        "class Foo { async *yield() {} }",
        "class Foo {\n  async *yield() {\n  }\n}\n",
    );
    expect_printed(
        "async function* yield() {}",
        "async function* yield() {\n}\n",
    );
}

#[test]
#[ignore = "depends on esbuild parser ASI behavior"]
fn test_async() {
    expect_printed_script(
        "function foo() { await }",
        "function foo() {\n  await;\n}\n",
    );
    expect_printed(
        "async function foo() { await 0 }",
        "async function foo() {\n  await 0;\n}\n",
    );
    expect_printed(
        "-async function foo() { await 0 }",
        "-async function foo() {\n  await 0;\n};\n",
    );
    expect_printed(
        "-async function() { await 0 }",
        "-async function() {\n  await 0;\n};\n",
    );
    expect_printed(
        "1 - async function foo() { await 0 }",
        "1 - async function foo() {\n  await 0;\n};\n",
    );
    expect_printed(
        "1 - async function() { await 0 }",
        "1 - async function() {\n  await 0;\n};\n",
    );
    expect_printed(
        "(async function foo() { await 0 })",
        "(async function foo() {\n  await 0;\n});\n",
    );
    expect_printed(
        "(async function() { await 0 })",
        "(async function() {\n  await 0;\n});\n",
    );
    expect_printed(
        "(x, async function foo() { await 0 })",
        "x, async function foo() {\n  await 0;\n};\n",
    );
    expect_printed(
        "(x, async function() { await 0 })",
        "x, async function() {\n  await 0;\n};\n",
    );
    expect_printed(
        "new async function() { await 0 }",
        "new async function() {\n  await 0;\n}();\n",
    );
    expect_printed(
        "new async function() { await 0 }.x",
        "new async function() {\n  await 0;\n}.x();\n",
    );
    expect_printed("async", "async;\n");
    expect_printed("async + 1", "async + 1;\n");
    expect_printed("async => {}", "(async) => {\n};\n");
    expect_printed("(async, 1)", "async, 1;\n");
    expect_printed("(async, x) => {}", "(async, x) => {\n};\n");
    expect_printed("async ()", "async();\n");
    expect_printed("async (x)", "async(x);\n");
    expect_printed("async (...x)", "async(...x);\n");
    expect_printed("async (...x, ...y)", "async(...x, ...y);\n");
    expect_printed("async () => {}", "async () => {\n};\n");
    expect_printed("async x => {}", "async (x) => {\n};\n");
    expect_printed("async (x) => {}", "async (x) => {\n};\n");
    expect_printed("async (...x) => {}", "async (...x) => {\n};\n");
    expect_printed("async x => await 0", "async (x) => await 0;\n");
    expect_printed("async () => await 0", "async () => await 0;\n");
    expect_printed("new async()", "new async();\n");
    expect_printed("new async().x", "new async().x;\n");
    expect_printed("new (async())", "new (async())();\n");
    expect_printed("new (async().x)", "new (async()).x();\n");
    expect_printed("(async x => y), z", "(async (x) => y), z;\n");
    expect_printed("(async x => y, z)", "async (x) => y, z;\n");
    expect_printed("(async x => (y, z))", "(async (x) => (y, z));\n");
    expect_printed("(async (x) => y), z", "(async (x) => y), z;\n");
    expect_printed("(async (x) => y, z)", "async (x) => y, z;\n");
    expect_printed("(async (x) => (y, z))", "(async (x) => (y, z));\n");
    expect_printed("async x => y, z", "async (x) => y, z;\n");
    expect_printed("async x => (y, z)", "async (x) => (y, z);\n");
    expect_printed("async (x) => y, z", "async (x) => y, z;\n");
    expect_printed("async (x) => (y, z)", "async (x) => (y, z);\n");
    expect_printed(
        "export default async x => (y, z)",
        "export default async (x) => (y, z);\n",
    );
    expect_printed(
        "export default async (x) => (y, z)",
        "export default async (x) => (y, z);\n",
    );
    expect_printed(
        "class Foo { async async() {} }",
        "class Foo {\n  async async() {\n  }\n}\n",
    );
    expect_printed(
        "(class { async async() {} })",
        "(class {\n  async async() {\n  }\n});\n",
    );
    expect_printed("({ async async() {} })", "({ async async() {\n} });\n");
    expect_printed(
        "async function foo() { (x = await y) }",
        "async function foo() {\n  x = await y;\n}\n",
    );
    expect_printed(
        "(class { async \n foo() {} })",
        "(class {\n  async;\n  foo() {\n  }\n});\n",
    );
    expect_printed(
        "(class { async \n *foo() {} })",
        "(class {\n  async;\n  *foo() {\n  }\n});\n",
    );
    expect_printed("await foo;", "await foo;\n");
    expect_printed("for await(foo of bar);", "for await (foo of bar) ;\n");
    expect_printed_script(
        "function foo(x = await) {}",
        "function foo(x = await) {\n}\n",
    );
    expect_printed_script("(function(x = await) {})", "(function(x = await) {\n});\n");
    expect_printed_script("({ foo(x = await) {} })", "({ foo(x = await) {\n} });\n");
    expect_printed_script(
        "class Foo { foo(x = await) {} }",
        "class Foo {\n  foo(x = await) {\n  }\n}\n",
    );
    expect_printed_script(
        "(class { foo(x = await) {} })",
        "(class {\n  foo(x = await) {\n  }\n});\n",
    );
    expect_printed("(x = await y)", "x = await y;\n");
    expect_printed("async(x = await y)", "async(x = await y);\n");
    expect_printed("\\u0061sync", "async;\n");
    expect_printed("(\\u0061sync)", "async;\n");
    expect_printed_script(
        "function foo() { \\u0061wait }",
        "function foo() {\n  await;\n}\n",
    );
    expect_printed_script(
        "function foo() { var \\u0061wait }",
        "function foo() {\n  var await;\n}\n",
    );
    expect_printed(
        "async function foo(){for await(x of y);}",
        "async function foo() {\n  for await (x of y) ;\n}\n",
    );
    expect_printed(
        "async function foo(){for await(let x of y);}",
        "async function foo() {\n  for await (let x of y) ;\n}\n",
    );
    expect_printed_script("(function await() {})", "(function await() {\n});\n");
    expect_printed_script(
        "function foo() { ({await} = x) }",
        "function foo() {\n  ({ await } = x);\n}\n",
    );
    expect_printed_script(
        "function foo() { let x = {await} }",
        "function foo() {\n  let x = { await };\n}\n",
    );
    expect_printed("({ async await() {} })", "({ async await() {\n} });\n");
    expect_printed(
        "(class { async await() {} })",
        "(class {\n  async await() {\n  }\n});\n",
    );
    expect_printed(
        "class Foo { async await() {} }",
        "class Foo {\n  async await() {\n  }\n}\n",
    );
    expect_printed("({ async *await() {} })", "({ async *await() {\n} });\n");
    expect_printed(
        "(class { async *await() {} })",
        "(class {\n  async *await() {\n  }\n});\n",
    );
    expect_printed(
        "class Foo { async *await() {} }",
        "class Foo {\n  async *await() {\n  }\n}\n",
    );
}

#[test]
fn test_labels() {
    expect_printed("{a:b}", "{\n  a: b;\n}\n");
    expect_printed("({a:b})", "({ a: b });\n");
    expect_printed("x: y: z: 1", "x: y: z: 1;\n");
    expect_printed("x: 1; y: 2; x: 3", "x: 1;\ny: 2;\nx: 3;\n");
    expect_printed("x: (() => { x: 1; })()", "x: (() => {\n  x: 1;\n})();\n");
    expect_printed(
        "x: ({ f() { x: 1; } }).f()",
        "x: ({ f() {\n  x: 1;\n} }).f();\n",
    );
    expect_printed(
        "x: (function() { x: 1; })()",
        "x: (function() {\n  x: 1;\n})();\n",
    );
    expect_printed("x: break x", "x: break x;\n");
    expect_printed("x: { break x; foo() }", "x: {\n  break x;\n  foo();\n}\n");
    expect_printed(
        "x: { y: { z: { foo(); break x; } } }",
        "x: {\n  y: {\n    z: {\n      foo();\n      break x;\n    }\n  }\n}\n",
    );
    expect_printed(
        "x: { class X { static { new X } } }",
        "x: {\n  class X {\n    static {\n      new X();\n    }\n  }\n}\n",
    );
}

#[test]
#[ignore = "depends on esbuild parser purity metadata"]
fn test_arrow() {
    expect_printed("x = ([ y = [ (z) ] ]) => 0", "x = ([y = [z]]) => 0;\n");
    expect_printed(
        "x = ([ y = [ ...(z) ] ]) => 0",
        "x = ([y = [...z]]) => 0;\n",
    );
    expect_printed(
        "x = ({ y = { y: (z) } }) => 0",
        "x = ({ y = { y: z } }) => 0;\n",
    );
    expect_printed(
        "x = ({ y = { ...(y) } }) => 0",
        "x = ({ y = { ...y } }) => 0;\n",
    );
    expect_printed("x => function() {}", "(x) => function() {\n};\n");
    expect_printed("(x) => function() {}", "(x) => function() {\n};\n");
    expect_printed("(x => function() {})", "((x) => function() {\n});\n");
    expect_printed("(x = () => {}) => {}", "(x = () => {\n}) => {\n};\n");
    expect_printed(
        "async (x = () => {}) => {}",
        "async (x = () => {\n}) => {\n};\n",
    );
    expect_printed("(() => {}) ? a : b", "(() => {\n}) ? a : b;\n");
    expect_printed("1 < (() => {})", "1 < (() => {\n});\n");
    expect_printed("y = x => {}", "y = (x) => {\n};\n");
    expect_printed("y = () => {}", "y = () => {\n};\n");
    expect_printed("y = (x) => {}", "y = (x) => {\n};\n");
    expect_printed("y = async x => {}", "y = async (x) => {\n};\n");
    expect_printed("y = async () => {}", "y = async () => {\n};\n");
    expect_printed("y = async (x) => {}", "y = async (x) => {\n};\n");
    expect_printed("1 + function () {}", "1 + function() {\n};\n");
    expect_printed("1 + async function () {}", "1 + async function() {\n};\n");
    expect_printed(
        "class Foo extends function () {} {}",
        "class Foo extends function() {\n} {\n}\n",
    );
    expect_printed(
        "class Foo extends async function () {} {}",
        "class Foo extends async function() {\n} {\n}\n",
    );
    expect_printed("() => {}\n(0)", "() => {\n};\n0;\n");
    expect_printed("x => {}\n(0)", "(x) => {\n};\n0;\n");
    expect_printed("async () => {}\n(0)", "async () => {\n};\n0;\n");
    expect_printed("async x => {}\n(0)", "async (x) => {\n};\n0;\n");
    expect_printed("async (x) => {}\n(0)", "async (x) => {\n};\n0;\n");
    expect_printed("() => {}\n,0", "() => {\n}, 0;\n");
    expect_printed("x => {}\n,0", "(x) => {\n}, 0;\n");
    expect_printed("async () => {}\n,0", "async () => {\n}, 0;\n");
    expect_printed("async x => {}\n,0", "async (x) => {\n}, 0;\n");
    expect_printed("async (x) => {}\n,0", "async (x) => {\n}, 0;\n");
    expect_printed("(() => {})\n(0)", "/* @__PURE__ */ (() => {\n})(0);\n");
    expect_printed("(x => {})\n(0)", "/* @__PURE__ */ ((x) => {\n})(0);\n");
    expect_printed("(async () => {})\n(0)", "(async () => {\n})(0);\n");
    expect_printed("(async x => {})\n(0)", "(async (x) => {\n})(0);\n");
    expect_printed("(async (x) => {})\n(0)", "(async (x) => {\n})(0);\n");
    expect_printed("y = () => {}\n(0)", "y = () => {\n};\n0;\n");
    expect_printed("y = x => {}\n(0)", "y = (x) => {\n};\n0;\n");
    expect_printed("y = async () => {}\n(0)", "y = async () => {\n};\n0;\n");
    expect_printed("y = async x => {}\n(0)", "y = async (x) => {\n};\n0;\n");
    expect_printed("y = async (x) => {}\n(0)", "y = async (x) => {\n};\n0;\n");
    expect_printed("y = () => {}\n,0", "y = () => {\n}, 0;\n");
    expect_printed("y = x => {}\n,0", "y = (x) => {\n}, 0;\n");
    expect_printed("y = async () => {}\n,0", "y = async () => {\n}, 0;\n");
    expect_printed("y = async x => {}\n,0", "y = async (x) => {\n}, 0;\n");
    expect_printed("y = async (x) => {}\n,0", "y = async (x) => {\n}, 0;\n");
    expect_printed(
        "y = (() => {})\n(0)",
        "y = /* @__PURE__ */ (() => {\n})(0);\n",
    );
    expect_printed(
        "y = (x => {})\n(0)",
        "y = /* @__PURE__ */ ((x) => {\n})(0);\n",
    );
    expect_printed("y = (async () => {})\n(0)", "y = (async () => {\n})(0);\n");
    expect_printed("y = (async x => {})\n(0)", "y = (async (x) => {\n})(0);\n");
    expect_printed(
        "y = (async (x) => {})\n(0)",
        "y = (async (x) => {\n})(0);\n",
    );
    expect_printed("(() => {}\n,0)", "() => {\n}, 0;\n");
    expect_printed("(x => {}\n,0)", "(x) => {\n}, 0;\n");
    expect_printed("(async () => {}\n,0)", "async () => {\n}, 0;\n");
    expect_printed("(async x => {}\n,0)", "async (x) => {\n}, 0;\n");
    expect_printed("(async (x) => {}\n,0)", "async (x) => {\n}, 0;\n");
    expect_printed("((() => {})\n(0))", "/* @__PURE__ */ (() => {\n})(0);\n");
    expect_printed("((x => {})\n(0))", "/* @__PURE__ */ ((x) => {\n})(0);\n");
    expect_printed("((async () => {})\n(0))", "(async () => {\n})(0);\n");
    expect_printed("((async x => {})\n(0))", "(async (x) => {\n})(0);\n");
    expect_printed("((async (x) => {})\n(0))", "(async (x) => {\n})(0);\n");
    expect_printed("y = (() => {}\n,0)", "y = (() => {\n}, 0);\n");
    expect_printed("y = (x => {}\n,0)", "y = ((x) => {\n}, 0);\n");
    expect_printed("y = (async () => {}\n,0)", "y = (async () => {\n}, 0);\n");
    expect_printed("y = (async x => {}\n,0)", "y = (async (x) => {\n}, 0);\n");
    expect_printed("y = (async (x) => {}\n,0)", "y = (async (x) => {\n}, 0);\n");
    expect_printed(
        "y = ((() => {})\n(0))",
        "y = /* @__PURE__ */ (() => {\n})(0);\n",
    );
    expect_printed(
        "y = ((x => {})\n(0))",
        "y = /* @__PURE__ */ ((x) => {\n})(0);\n",
    );
    expect_printed(
        "y = ((async () => {})\n(0))",
        "y = (async () => {\n})(0);\n",
    );
    expect_printed(
        "y = ((async x => {})\n(0))",
        "y = (async (x) => {\n})(0);\n",
    );
    expect_printed(
        "y = ((async (x) => {})\n(0))",
        "y = (async (x) => {\n})(0);\n",
    );
}

#[test]
#[ignore = "depends on esbuild tagged-template newline normalization"]
fn test_template() {
    expect_printed("`\\0`", "`\\0`;\n");
    expect_printed("`${'\\00'}`", "`${\"\\0\"}`;\n");
    expect_printed("tag`\\7`", "tag`\\7`;\n");
    expect_printed("tag`\\8`", "tag`\\8`;\n");
    expect_printed("tag`\\9`", "tag`\\9`;\n");
    expect_printed("tag`\\00`", "tag`\\00`;\n");
    expect_printed("tag`\\00${x}`", "tag`\\00${x}`;\n");
    expect_printed("tag`${x}\\00`", "tag`${x}\\00`;\n");
    expect_printed("tag`${x}\\00${y}`", "tag`${x}\\00${y}`;\n");
    expect_printed("tag`\\unicode`", "tag`\\unicode`;\n");
    expect_printed("tag`\\unicode${x}`", "tag`\\unicode${x}`;\n");
    expect_printed("tag`${x}\\unicode`", "tag`${x}\\unicode`;\n");
    expect_printed("tag`\\u{10FFFFF}`", "tag`\\u{10FFFFF}`;\n");
    expect_printed("new foo`bar`()", "new foo`bar`();\n");
    expect_printed("new (foo`bar`)()", "new foo`bar`();\n");
    expect_printed("(new foo)`bar`()", "new foo()`bar`();\n");
    expect_printed("new foo()`bar`()", "new foo()`bar`();\n");
    expect_printed("(new foo())`bar`()", "new foo()`bar`();\n");
    expect_printed("new (foo()`bar`)()", "new (foo())`bar`();\n");
    expect_printed("tag``", "tag``;\n");
    expect_printed("(a?.b)``", "(a?.b)``;\n");
    expect_printed("(a?.(b))``", "(a?.(b))``;\n");
    expect_printed("(a?.[b])``", "(a?.[b])``;\n");
    expect_printed("(a?.b.c)``", "(a?.b.c)``;\n");
    expect_printed("(a?.(b).c)``", "(a?.(b).c)``;\n");
    expect_printed("(a?.[b].c)``", "(a?.[b].c)``;\n");
    // Depends on esbuild's parser-stage template constant folding.
    // expect_printed("`a${1 + `b${2}c` + 3}d`", "`a${`1b${2}c3`}d`;\n");
    expect_printed("`a\nb`", "`a\nb`;\n");
    expect_printed("`a\rb`", "`a\nb`;\n");
    expect_printed("`a\r\nb`", "`a\nb`;\n");
    expect_printed("`a\\nb`", "`a\nb`;\n");
    expect_printed("`a\\rb`", "`a\\rb`;\n");
    expect_printed("`a\\r\\nb`", "`a\\r\nb`;\n");
    expect_printed("`a\u{2028}b`", "`a\\u2028b`;\n");
    expect_printed("`a\u{2029}b`", "`a\\u2029b`;\n");
    expect_printed("`a\n${b}`", "`a\n${b}`;\n");
    expect_printed("`a\r${b}`", "`a\n${b}`;\n");
    expect_printed("`a\r\n${b}`", "`a\n${b}`;\n");
    expect_printed("`a\\n${b}`", "`a\n${b}`;\n");
    expect_printed("`a\\r${b}`", "`a\\r${b}`;\n");
    expect_printed("`a\\r\\n${b}`", "`a\\r\n${b}`;\n");
    expect_printed("`a\u{2028}${b}`", "`a\\u2028${b}`;\n");
    expect_printed("`a\u{2029}${b}`", "`a\\u2029${b}`;\n");
    expect_printed("`${a}\nb`", "`${a}\nb`;\n");
    expect_printed("`${a}\rb`", "`${a}\nb`;\n");
    expect_printed("`${a}\r\nb`", "`${a}\nb`;\n");
    expect_printed("`${a}\\nb`", "`${a}\nb`;\n");
    expect_printed("`${a}\\rb`", "`${a}\\rb`;\n");
    expect_printed("`${a}\\r\\nb`", "`${a}\\r\nb`;\n");
    expect_printed("`${a}\u{2028}b`", "`${a}\\u2028b`;\n");
    expect_printed("`${a}\u{2029}b`", "`${a}\\u2029b`;\n");
    expect_printed("tag`a\nb`", "tag`a\nb`;\n");
    expect_printed("tag`a\rb`", "tag`a\nb`;\n");
    expect_printed("tag`a\r\nb`", "tag`a\nb`;\n");
    expect_printed("tag`a\\nb`", "tag`a\\nb`;\n");
    expect_printed("tag`a\\rb`", "tag`a\\rb`;\n");
    expect_printed("tag`a\\r\\nb`", "tag`a\\r\\nb`;\n");
    expect_printed("tag`a\u{2028}b`", "tag`a\u{2028}b`;\n");
    expect_printed("tag`a\u{2029}b`", "tag`a\u{2029}b`;\n");
    expect_printed("tag`a\n${b}`", "tag`a\n${b}`;\n");
    expect_printed("tag`a\r${b}`", "tag`a\n${b}`;\n");
    expect_printed("tag`a\r\n${b}`", "tag`a\n${b}`;\n");
    expect_printed("tag`a\\n${b}`", "tag`a\\n${b}`;\n");
    expect_printed("tag`a\\r${b}`", "tag`a\\r${b}`;\n");
    expect_printed("tag`a\\r\\n${b}`", "tag`a\\r\\n${b}`;\n");
    expect_printed("tag`a\u{2028}${b}`", "tag`a\u{2028}${b}`;\n");
    expect_printed("tag`a\u{2029}${b}`", "tag`a\u{2029}${b}`;\n");
    expect_printed("tag`${a}\nb`", "tag`${a}\nb`;\n");
    expect_printed("tag`${a}\rb`", "tag`${a}\nb`;\n");
    expect_printed("tag`${a}\r\nb`", "tag`${a}\nb`;\n");
    expect_printed("tag`${a}\\nb`", "tag`${a}\\nb`;\n");
    expect_printed("tag`${a}\\rb`", "tag`${a}\\rb`;\n");
    expect_printed("tag`${a}\\r\\nb`", "tag`${a}\\r\\nb`;\n");
    expect_printed("tag`${a}\u{2028}b`", "tag`${a}\u{2028}b`;\n");
    expect_printed("tag`${a}\u{2029}b`", "tag`${a}\u{2029}b`;\n");
}

#[test]
fn test_switch() {
    expect_printed("switch (x) { default: }", "switch (x) {\n  default:\n}\n");
    expect_printed(
        "switch ((x => x + 1)(0)) { case 1: var y } y = 2",
        "switch (((x) => x + 1)(0)) {\n  case 1:\n    var y;\n}\ny = 2;\n",
    );
    expect_printed(
        "switch (0) { case x(() => 1): y = () => 2; case x(() => 3): y = () => 4 }",
        "switch (0) {\n  case x(() => 1):\n    y = () => 2;\n  case x(() => 3):\n    y = () => 4;\n}\n",
    );
    expect_printed(
        "switch (x) { case 0: { using y = z } }\n",
        "switch (x) {\n  case 0: {\n    using y = z;\n  }\n}\n",
    );
    expect_printed(
        "switch (x) { case 0: { await using y = z } }\n",
        "switch (x) {\n  case 0: {\n    await using y = z;\n  }\n}\n",
    );
}

#[test]
#[ignore = "depends on esbuild import phase metadata"]
fn test_import() {
    expect_printed("import \"foo\"", "import \"foo\";\n");
    expect_printed("import {} from \"foo\"", "import {} from \"foo\";\n");
    expect_printed(
        "import {x} from \"foo\";x",
        "import { x } from \"foo\";\nx;\n",
    );
    expect_printed(
        "import {x as y} from \"foo\";y",
        "import { x as y } from \"foo\";\ny;\n",
    );
    expect_printed(
        "import {x as y, z} from \"foo\";y;z",
        "import { x as y, z } from \"foo\";\ny;\nz;\n",
    );
    expect_printed(
        "import {x as y, z,} from \"foo\";y;z",
        "import { x as y, z } from \"foo\";\ny;\nz;\n",
    );
    expect_printed(
        "import z, {x as y} from \"foo\";y;z",
        "import z, { x as y } from \"foo\";\ny;\nz;\n",
    );
    expect_printed("import z from \"foo\";z", "import z from \"foo\";\nz;\n");
    expect_printed(
        "import * as ns from \"foo\";ns;ns.x",
        "import * as ns from \"foo\";\nns;\nns.x;\n",
    );
    expect_printed(
        "import z, * as ns from \"foo\";z;ns;ns.x",
        "import z, * as ns from \"foo\";\nz;\nns;\nns.x;\n",
    );
    expect_printed("import('foo')", "import(\"foo\");\n");
    expect_printed("(import('foo'))", "import(\"foo\");\n");
    expect_printed("{import('foo')}", "{\n  import(\"foo\");\n}\n");
    expect_printed(
        "import('foo').then(() => {})",
        "import(\"foo\").then(() => {\n});\n",
    );
    expect_printed("new import.meta", "new import.meta();\n");
    expect_printed("new (import('foo'))", "new (import(\"foo\"))();\n");
    expect_printed("import.meta", "import.meta;\n");
    expect_printed("(import.meta)", "import.meta;\n");
    expect_printed("{import.meta}", "{\n  import.meta;\n}\n");
    expect_printed(
        "import x from \"foo\"; x = 1",
        "import x from \"foo\";\nx = 1;\n",
    );
    expect_printed(
        "import x from \"foo\"; x++",
        "import x from \"foo\";\nx++;\n",
    );
    expect_printed(
        "import x from \"foo\"; ([x] = 1)",
        "import x from \"foo\";\n[x] = 1;\n",
    );
    expect_printed(
        "import x from \"foo\"; ({x} = 1)",
        "import x from \"foo\";\n({ x } = 1);\n",
    );
    expect_printed(
        "import x from \"foo\"; ({y: x} = 1)",
        "import x from \"foo\";\n({ y: x } = 1);\n",
    );
    expect_printed(
        "import {x} from \"foo\"; x++",
        "import { x } from \"foo\";\nx++;\n",
    );
    expect_printed(
        "import * as x from \"foo\"; x++",
        "import * as x from \"foo\";\nx++;\n",
    );
    expect_printed(
        "import * as x from \"foo\"; x.y = 1",
        "import * as x from \"foo\";\nx.y = 1;\n",
    );
    expect_printed(
        "import * as x from \"foo\"; x[y] = 1",
        "import * as x from \"foo\";\nx[y] = 1;\n",
    );
    expect_printed(
        "import * as x from \"foo\"; x['y'] = 1",
        "import * as x from \"foo\";\nx[\"y\"] = 1;\n",
    );
    expect_printed(
        "import * as x from \"foo\"; x['y z'] = 1",
        "import * as x from \"foo\";\nx[\"y z\"] = 1;\n",
    );
    expect_printed(
        "import x from \"foo\"; ({y = x} = 1)",
        "import x from \"foo\";\n({ y = x } = 1);\n",
    );
    expect_printed(
        "import x from \"foo\"; ({[x]: y} = 1)",
        "import x from \"foo\";\n({ [x]: y } = 1);\n",
    );
    expect_printed(
        "import x from \"foo\"; x.y = 1",
        "import x from \"foo\";\nx.y = 1;\n",
    );
    expect_printed(
        "import x from \"foo\"; x[y] = 1",
        "import x from \"foo\";\nx[y] = 1;\n",
    );
    expect_printed(
        "import x from \"foo\"; x['y'] = 1",
        "import x from \"foo\";\nx[\"y\"] = 1;\n",
    );
    expect_printed(
        "import {eval as x} from 'foo'",
        "import { eval as x } from \"foo\";\n",
    );
    expect_printed(
        "import {ev\\u0061l as x} from 'foo'",
        "import { eval as x } from \"foo\";\n",
    );
    expect_printed(
        "import {arguments as x} from 'foo'",
        "import { arguments as x } from \"foo\";\n",
    );
    expect_printed(
        "import {\\u0061rguments as x} from 'foo'",
        "import { arguments as x } from \"foo\";\n",
    );
    expect_printed(
        "import {'' as x} from 'foo'",
        "import { \"\" as x } from \"foo\";\n",
    );
    expect_printed(
        "import {'\u{1f355}' as x} from 'foo'",
        "import { \"\u{1f355}\" as x } from \"foo\";\n",
    );
    expect_printed(
        "import {'a b' as x} from 'foo'",
        "import { \"a b\" as x } from \"foo\";\n",
    );
    expect_printed(
        "import {'\\uD800\\uDC00' as x} from 'foo'",
        "import { \u{10000} as x } from \"foo\";\n",
    );
    expect_printed("import defer from 'bar'", "import defer from \"bar\";\n");
    expect_printed(
        "import defer, { foo } from 'bar'",
        "import defer, { foo } from \"bar\";\n",
    );
    expect_printed(
        "import defer * as foo from 'bar'",
        "import defer * as foo from \"bar\";\n",
    );
    expect_printed("import.defer('foo')", "import.defer(\"foo\");\n");
    expect_printed("import source from 'bar'", "import source from \"bar\";\n");
    expect_printed(
        "import source, { foo } from 'bar'",
        "import source, { foo } from \"bar\";\n",
    );
    expect_printed(
        "import source foo from 'bar'",
        "import source foo from \"bar\";\n",
    );
    expect_printed(
        "import source from from 'bar'",
        "import source from from \"bar\";\n",
    );
    expect_printed(
        "import source source from 'bar'",
        "import source source from \"bar\";\n",
    );
    expect_printed("import.source('foo')", "import.source(\"foo\");\n");
}

#[test]
fn test_export() {
    expect_printed("export default x", "export default x;\n");
    expect_printed("export class x {}", "export class x {\n}\n");
    expect_printed("export function x() {}", "export function x() {\n}\n");
    expect_printed(
        "export async function x() {}",
        "export async function x() {\n}\n",
    );
    expect_printed("export var x, y", "export var x, y;\n");
    expect_printed("export let x, y", "export let x, y;\n");
    expect_printed("export const x = 0, y = 1", "export const x = 0, y = 1;\n");
    expect_printed("export * from \"foo\"", "export * from \"foo\";\n");
    expect_printed(
        "export * as ns from \"foo\"",
        "export * as ns from \"foo\";\n",
    );
    expect_printed(
        "export * as if from \"foo\"",
        "export * as if from \"foo\";\n",
    );
    expect_printed("let x; export {x}", "let x;\nexport { x };\n");
    expect_printed("let x; export {x as y}", "let x;\nexport { x as y };\n");
    expect_printed(
        "let x, z; export {x as y, z}",
        "let x, z;\nexport { x as y, z };\n",
    );
    expect_printed(
        "let x, z; export {x as y, z,}",
        "let x, z;\nexport { x as y, z };\n",
    );
    expect_printed(
        "let x; export {x} from \"foo\"",
        "let x;\nexport { x } from \"foo\";\n",
    );
    expect_printed(
        "let x; export {x as y} from \"foo\"",
        "let x;\nexport { x as y } from \"foo\";\n",
    );
    expect_printed(
        "let x, z; export {x as y, z} from \"foo\"",
        "let x, z;\nexport { x as y, z } from \"foo\";\n",
    );
    expect_printed(
        "let x, z; export {x as y, z,} from \"foo\"",
        "let x, z;\nexport { x as y, z } from \"foo\";\n",
    );
    expect_printed("let x; export {x as ''}", "let x;\nexport { x as \"\" };\n");
    expect_printed(
        "let x; export {x as '\u{1f355}'}",
        "let x;\nexport { x as \"\u{1f355}\" };\n",
    );
    expect_printed(
        "let x; export {x as 'a b'}",
        "let x;\nexport { x as \"a b\" };\n",
    );
    expect_printed(
        "let x; export {x as '\\uD800\\uDC00'}",
        "let x;\nexport { x as \u{10000} };\n",
    );
    expect_printed(
        "export {'' as x} from 'foo'",
        "export { \"\" as x } from \"foo\";\n",
    );
    expect_printed(
        "export {'\u{1f355}' as x} from 'foo'",
        "export { \"\u{1f355}\" as x } from \"foo\";\n",
    );
    expect_printed(
        "export {'a b' as x} from 'foo'",
        "export { \"a b\" as x } from \"foo\";\n",
    );
    expect_printed(
        "export {'\\uD800\\uDC00' as x} from 'foo'",
        "export { \u{10000} as x } from \"foo\";\n",
    );
    expect_printed(
        "export {x as ''} from 'foo'",
        "export { x as \"\" } from \"foo\";\n",
    );
    expect_printed(
        "export {x as '\u{1f355}'} from 'foo'",
        "export { x as \"\u{1f355}\" } from \"foo\";\n",
    );
    expect_printed(
        "export {x as 'a b'} from 'foo'",
        "export { x as \"a b\" } from \"foo\";\n",
    );
    expect_printed(
        "export {x as '\\uD800\\uDC00'} from 'foo'",
        "export { x as \u{10000} } from \"foo\";\n",
    );
    expect_printed("export {'x'} from 'foo'", "export { x } from \"foo\";\n");
    expect_printed(
        "export {'a b'} from 'foo'",
        "export { \"a b\" } from \"foo\";\n",
    );
    expect_printed(
        "export {'x' as 'y'} from 'foo'",
        "export { x as y } from \"foo\";\n",
    );
    expect_printed(
        "export {'a b' as 'c d'} from 'foo'",
        "export { \"a b\" as \"c d\" } from \"foo\";\n",
    );
    expect_printed(
        "export * as '' from 'foo'",
        "export * as \"\" from \"foo\";\n",
    );
    expect_printed(
        "export * as '\u{1f355}' from 'foo'",
        "export * as \"\u{1f355}\" from \"foo\";\n",
    );
    expect_printed(
        "export * as 'a b' from 'foo'",
        "export * as \"a b\" from \"foo\";\n",
    );
    expect_printed(
        "export * as '\\uD800\\uDC00' from 'foo'",
        "export * as \u{10000} from \"foo\";\n",
    );
}

#[test]
fn test_export_duplicates() {
    expect_printed("export {x};let x", "export { x };\nlet x;\n");
    expect_printed(
        "export {x, x as y};let x",
        "export { x, x as y };\nlet x;\n",
    );
    expect_printed(
        "export {x};export {x as y} from 'foo';let x",
        "export { x };\nexport { x as y } from \"foo\";\nlet x;\n",
    );
    expect_printed(
        "export {x};export default function x() {}",
        "export { x };\nexport default function x() {\n}\n",
    );
    expect_printed(
        "export {x};export default class x {}",
        "export { x };\nexport default class x {\n}\n",
    );
}

#[test]
fn test_export_default() {
    expect_printed("export default (1, 2)", "export default (1, 2);\n");
    expect_printed("export default async", "export default async;\n");
    expect_printed("export default async()", "export default async();\n");
    expect_printed("export default async + 1", "export default async + 1;\n");
    expect_printed(
        "export default async => {}",
        "export default (async) => {\n};\n",
    );
    expect_printed(
        "export default async x => {}",
        "export default async (x) => {\n};\n",
    );
    expect_printed(
        "export default async () => {}",
        "export default async () => {\n};\n",
    );
    expect_printed(
        "export default function() {} - after",
        "export default function() {\n}\n-after;\n",
    );
    expect_printed(
        "export default function*() {} - after",
        "export default function* () {\n}\n-after;\n",
    );
    expect_printed(
        "export default function foo() {} - after",
        "export default function foo() {\n}\n-after;\n",
    );
    expect_printed(
        "export default function* foo() {} - after",
        "export default function* foo() {\n}\n-after;\n",
    );
    expect_printed(
        "export default async function() {} - after",
        "export default async function() {\n}\n-after;\n",
    );
    expect_printed(
        "export default async function*() {} - after",
        "export default async function* () {\n}\n-after;\n",
    );
    expect_printed(
        "export default async function foo() {} - after",
        "export default async function foo() {\n}\n-after;\n",
    );
    expect_printed(
        "export default async function* foo() {} - after",
        "export default async function* foo() {\n}\n-after;\n",
    );
    expect_printed(
        "export default class {} - after",
        "export default class {\n}\n-after;\n",
    );
    expect_printed(
        "export default class Foo {} - after",
        "export default class Foo {\n}\n-after;\n",
    );
    expect_printed(
        "export default abstract\nclass Foo {}",
        "export default abstract;\nclass Foo {\n}\n",
    );
}

#[test]
fn test_export_clause() {
    expect_printed("export {x, y};let x, y", "export { x, y };\nlet x, y;\n");
    expect_printed(
        "export {x, y as z,};let x, y",
        "export { x, y as z };\nlet x, y;\n",
    );
    expect_printed(
        "export {x, y} from 'path'",
        "export { x, y } from \"path\";\n",
    );
    expect_printed(
        "export {default, if} from 'path'",
        "export { default, if } from \"path\";\n",
    );
    expect_printed(
        "export {default as foo, if as bar} from 'path'",
        "export { default as foo, if as bar } from \"path\";\n",
    );
}

#[test]
#[ignore = "depends on esbuild parser lowering"]
fn test_catch() {
    expect_printed("try {} catch (e) {}", "try {\n} catch (e) {\n}\n");
    expect_printed(
        "try {} catch (e) { var e }",
        "try {\n} catch (e) {\n  var e;\n}\n",
    );
    expect_printed(
        "var e; try {} catch (e) {}",
        "var e;\ntry {\n} catch (e) {\n}\n",
    );
    expect_printed(
        "let e; try {} catch (e) {}",
        "let e;\ntry {\n} catch (e) {\n}\n",
    );
    expect_printed(
        "try { var e } catch (e) {}",
        "try {\n  var e;\n} catch (e) {\n}\n",
    );
    expect_printed(
        "try { function e() {} } catch (e) {}",
        "try {\n  let e = function() {\n  };\n  var e = e;\n} catch (e) {\n}\n",
    );
    expect_printed(
        "try {} catch (e) { { function e() {} } }",
        "try {\n} catch (e) {\n  {\n    let e = function() {\n    };\n    var e = e;\n  }\n}\n",
    );
    expect_printed(
        "try {} catch (e) { if (1) function e() {} }",
        "try {\n} catch (e) {\n  if (1) {\n    let e = function() {\n    };\n    var e = e;\n  }\n}\n",
    );
    expect_printed(
        "try {} catch (e) { if (0) ; else function e() {} }",
        "try {\n} catch (e) {\n  if (0) ;\n  else {\n    let e = function() {\n    };\n    var e = e;\n  }\n}\n",
    );
    expect_printed(
        "try {} catch ({ e }) { { function e() {} } }",
        "try {\n} catch ({ e }) {\n  {\n    let e = function() {\n    };\n    var e = e;\n  }\n}\n",
    );
}

#[test]
#[ignore = "depends on esbuild comment metadata"]
fn test_preserved_comments() {
    expect_printed("//", "");
    expect_printed("//preserve", "");
    expect_printed("//@__PURE__", "");
    expect_printed("//!", "//!\n");
    expect_printed("//@license", "//@license\n");
    expect_printed("//@preserve", "//@preserve\n");
    expect_printed("// @license", "// @license\n");
    expect_printed("// @preserve", "// @preserve\n");
    expect_printed("/**/", "");
    expect_printed("/*preserve*/", "");
    expect_printed("/*@__PURE__*/", "");
    expect_printed("/*!*/", "/*!*/\n");
    expect_printed("/*@license*/", "/*@license*/\n");
    expect_printed("/*@preserve*/", "/*@preserve*/\n");
    expect_printed("/*\n * @license\n */", "/*\n * @license\n */\n");
    expect_printed("/*\n * @preserve\n */", "/*\n * @preserve\n */\n");
    expect_printed("foo() //! test", "foo();\n//! test\n");
    expect_printed("//! test\nfoo()", "//! test\nfoo();\n");
    expect_printed("if (1) //! test\nfoo()", "if (1)\n  foo();\n");
    expect_printed(
        "if (1) {//! test\nfoo()}",
        "if (1) {\n  //! test\n  foo();\n}\n",
    );
    expect_printed(
        "if (1) {foo() //! test\n}",
        "if (1) {\n  foo();\n  //! test\n}\n",
    );
    expect_printed(
        "    /*!\r     * Re-indent test\r     */",
        "/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "    /*!\n     * Re-indent test\n     */",
        "/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "    /*!\r\n     * Re-indent test\r\n     */",
        "/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "    /*!\u{2028}     * Re-indent test\u{2028}     */",
        "/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "    /*!\u{2029}     * Re-indent test\u{2029}     */",
        "/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "\t\t/*!\r\t\t * Re-indent test\r\t\t */",
        "/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "\t\t/*!\n\t\t * Re-indent test\n\t\t */",
        "/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "\t\t/*!\r\n\t\t * Re-indent test\r\n\t\t */",
        "/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "\t\t/*!\u{2028}\t\t * Re-indent test\u{2028}\t\t */",
        "/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "\t\t/*!\u{2029}\t\t * Re-indent test\u{2029}\t\t */",
        "/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "x\r    /*!\r     * Re-indent test\r     */",
        "x;\n/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "x\n    /*!\n     * Re-indent test\n     */",
        "x;\n/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "x\r\n    /*!\r\n     * Re-indent test\r\n     */",
        "x;\n/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "x\u{2028}    /*!\u{2028}     * Re-indent test\u{2028}     */",
        "x;\n/*!\n * Re-indent test\n */\n",
    );
    expect_printed(
        "x\u{2029}    /*!\u{2029}     * Re-indent test\u{2029}     */",
        "x;\n/*!\n * Re-indent test\n */\n",
    );
}

#[test]
fn test_unicode_whitespace() {
    expect_printed_jsx("<x>\n\x08\n</x>", "<x>\n\x08\n</x>;\n");
    expect_printed_jsx("<x>\n\x08&quot;\n</x>", "<x>\n\x08&quot;\n</x>;\n");
    const WHITESPACE: &[&str] = &[
        "\u{0009}", "\u{000B}", "\u{000C}", "\u{0020}", "\u{00A0}", "\u{1680}", "\u{2000}",
        "\u{2001}", "\u{2002}", "\u{2003}", "\u{2004}", "\u{2005}", "\u{2006}", "\u{2007}",
        "\u{2008}", "\u{2009}", "\u{200A}", "\u{202F}", "\u{205F}", "\u{3000}", "\u{FEFF}",
    ];
    for whitespace in WHITESPACE {
        let source = format!("var{whitespace}x");
        expect_printed(&source, "var x;\n");
        let source = format!("<x{whitespace}y/>");
        expect_printed_jsx(&source, "<x y />;\n");
        let source = format!("<x>\n{whitespace}\n</x>");
        let expected = format!("<x>\n{whitespace}\n</x>;\n");
        expect_printed_jsx(&source, &expected);
        let source = format!("<x>\n{whitespace}&quot;\n</x>");
        let expected = format!("<x>\n{whitespace}&quot;\n</x>;\n");
        expect_printed_jsx(&source, &expected);
    }
    let whitespace = "\u{0085}";
    let source = format!("<x>\n{whitespace}\n</x>");
    let expected = format!("<x>\n{whitespace}\n</x>;\n");
    expect_printed_jsx(&source, &expected);
    let source = format!("<x>\n{whitespace}&quot;\n</x>");
    let expected = format!("<x>\n{whitespace}&quot;\n</x>;\n");
    expect_printed_jsx(&source, &expected);
}

#[test]
#[ignore = "depends on esbuild comment handling"]
fn test_replacement_character() {
    expect_printed("//\u{fffd}\n123", "123;\n");
    expect_printed("/*\u{fffd}*/123", "123;\n");
    expect_printed("'\u{fffd}'", "\"\u{fffd}\";\n");
    expect_printed("\"\u{fffd}\"", "\"\u{fffd}\";\n");
    expect_printed("`\u{fffd}`", "`\u{fffd}`;\n");
    expect_printed("/\u{fffd}/", "/\u{fffd}/;\n");
    expect_printed_jsx("<a>\u{fffd}</a>", "<a>\u{fffd}</a>;\n");
}

#[test]
fn test_new_target() {
    expect_printed(
        "function f() { new.target }",
        "function f() {\n  new.target;\n}\n",
    );
    expect_printed(
        "function f() { (new.target) }",
        "function f() {\n  new.target;\n}\n",
    );
    expect_printed(
        "function f() { () => new.target }",
        "function f() {\n  () => new.target;\n}\n",
    );
    expect_printed(
        "class Foo { x = new.target }",
        "class Foo {\n  x = new.target;\n}\n",
    );
}

#[test]
fn test_jsx() {
    expect_printed_jsx("<div>></div>", "<div>></div>;\n");
    expect_printed_jsx("<div>{1}}</div>", "<div>{1}}</div>;\n");
    expect_printed_jsx("<a/>", "<a />;\n");
    expect_printed_jsx("<a></a>", "<a />;\n");
    expect_printed_jsx("<A/>", "<A />;\n");
    expect_printed_jsx("<a.b/>", "<a.b />;\n");
    expect_printed_jsx("<_a/>", "<_a />;\n");
    expect_printed_jsx("<a-b/>", "<a-b />;\n");
    expect_printed_jsx("<a0/>", "<a0 />;\n");
    expect_printed_jsx("<a b/>", "<a b />;\n");
    expect_printed_jsx("<a b=\"\\\"/>", "<a b=\"\\\" />;\n");
    expect_printed_jsx("<a b=\"<>\"/>", "<a b=\"<>\" />;\n");
    expect_printed_jsx("<a b=\"&lt;&gt;\"/>", "<a b=\"&lt;&gt;\" />;\n");
    expect_printed_jsx("<a b=\"&wrong;\"/>", "<a b=\"&wrong;\" />;\n");
    expect_printed_jsx("<a b={1, 2}/>", "<a b={(1, 2)} />;\n");
    expect_printed_jsx("<a b={<c/>}/>", "<a b={<c />} />;\n");
    expect_printed_jsx("<a {...props}/>", "<a {...props} />;\n");
    expect_printed_jsx("<a b=\"\u{1f642}\"/>", "<a b=\"\u{1f642}\" />;\n");
    expect_printed_jsx("<a>\n</a>", "<a>\n</a>;\n");
    expect_printed_jsx("<a>123</a>", "<a>123</a>;\n");
    expect_printed_jsx("<a>}</a>", "<a>}</a>;\n");
    expect_printed_jsx("<a>=</a>", "<a>=</a>;\n");
    expect_printed_jsx("<a>></a>", "<a>></a>;\n");
    expect_printed_jsx("<a>>=</a>", "<a>>=</a>;\n");
    expect_printed_jsx("<a>>></a>", "<a>>></a>;\n");
    expect_printed_jsx("<a>{}</a>", "<a>{}</a>;\n");
    expect_printed_jsx("<a>{/* comment */}</a>", "<a>{\n  /* comment */\n}</a>;\n");
    expect_printed_jsx("<a>b{}</a>", "<a>b{}</a>;\n");
    expect_printed_jsx(
        "<a>b{/* comment */}</a>",
        "<a>b{\n  /* comment */\n}</a>;\n",
    );
    expect_printed_jsx("<a>{}c</a>", "<a>{}c</a>;\n");
    expect_printed_jsx(
        "<a>{/* comment */}c</a>",
        "<a>{\n  /* comment */\n}c</a>;\n",
    );
    expect_printed_jsx("<a>b{}c</a>", "<a>b{}c</a>;\n");
    expect_printed_jsx(
        "<a>b{/* comment */}c</a>",
        "<a>b{\n  /* comment */\n}c</a>;\n",
    );
    expect_printed_jsx("<a>{1, 2}</a>", "<a>{(1, 2)}</a>;\n");
    expect_printed_jsx("<a>&lt;&gt;</a>", "<a>&lt;&gt;</a>;\n");
    expect_printed_jsx("<a>&wrong;</a>", "<a>&wrong;</a>;\n");
    expect_printed_jsx("<a>\u{1f642}</a>", "<a>\u{1f642}</a>;\n");
    expect_printed_jsx("<a>{...children}</a>", "<a>{...children}</a>;\n");
    expect_printed_jsx("<a b=\"   c\"/>", "<a b=\"   c\" />;\n");
    expect_printed_jsx("<a b=\"   \nc\"/>", "<a b=\"   \nc\" />;\n");
    expect_printed_jsx("<a b=\"\n   c\"/>", "<a b=\"\n   c\" />;\n");
    expect_printed_jsx("<a b=\"c   \"/>", "<a b=\"c   \" />;\n");
    expect_printed_jsx("<a b=\"c   \n\"/>", "<a b=\"c   \n\" />;\n");
    expect_printed_jsx("<a b=\"c\n   \"/>", "<a b=\"c\n   \" />;\n");
    expect_printed_jsx("<a b=\"c   d\"/>", "<a b=\"c   d\" />;\n");
    expect_printed_jsx("<a b=\"c   \nd\"/>", "<a b=\"c   \nd\" />;\n");
    expect_printed_jsx("<a b=\"c\n   d\"/>", "<a b=\"c\n   d\" />;\n");
    expect_printed_jsx("<a b=\"   c\"/>", "<a b=\"   c\" />;\n");
    expect_printed_jsx("<a b=\"   \nc\"/>", "<a b=\"   \nc\" />;\n");
    expect_printed_jsx("<a b=\"\n   c\"/>", "<a b=\"\n   c\" />;\n");
    expect_printed_jsx("<a b=\"   \u{1f642}\"/>", "<a b=\"   \u{1f642}\" />;\n");
    expect_printed_jsx("<a b=\"   \n\u{1f642}\"/>", "<a b=\"   \n\u{1f642}\" />;\n");
    expect_printed_jsx("<a b=\"\n   \u{1f642}\"/>", "<a b=\"\n   \u{1f642}\" />;\n");
    expect_printed_jsx("<a b=\"\u{1f642}   \"/>", "<a b=\"\u{1f642}   \" />;\n");
    expect_printed_jsx("<a b=\"\u{1f642}   \n\"/>", "<a b=\"\u{1f642}   \n\" />;\n");
    expect_printed_jsx("<a b=\"\u{1f642}\n   \"/>", "<a b=\"\u{1f642}\n   \" />;\n");
    expect_printed_jsx(
        "<a b=\"\u{1f642}   \u{1f355}\"/>",
        "<a b=\"\u{1f642}   \u{1f355}\" />;\n",
    );
    expect_printed_jsx(
        "<a b=\"\u{1f642}   \n\u{1f355}\"/>",
        "<a b=\"\u{1f642}   \n\u{1f355}\" />;\n",
    );
    expect_printed_jsx(
        "<a b=\"\u{1f642}\n   \u{1f355}\"/>",
        "<a b=\"\u{1f642}\n   \u{1f355}\" />;\n",
    );
    expect_printed_jsx("<a b=\"   \u{1f642}\"/>", "<a b=\"   \u{1f642}\" />;\n");
    expect_printed_jsx("<a b=\"   \n\u{1f642}\"/>", "<a b=\"   \n\u{1f642}\" />;\n");
    expect_printed_jsx("<a b=\"\n   \u{1f642}\"/>", "<a b=\"\n   \u{1f642}\" />;\n");
    expect_printed_jsx("<a>   b</a>", "<a>   b</a>;\n");
    expect_printed_jsx("<a>   \nb</a>", "<a>   \nb</a>;\n");
    expect_printed_jsx("<a>\n   b</a>", "<a>\n   b</a>;\n");
    expect_printed_jsx("<a>b   </a>", "<a>b   </a>;\n");
    expect_printed_jsx("<a>b   \n</a>", "<a>b   \n</a>;\n");
    expect_printed_jsx("<a>b\n   </a>", "<a>b\n   </a>;\n");
    expect_printed_jsx("<a>b   c</a>", "<a>b   c</a>;\n");
    expect_printed_jsx("<a>b   \nc</a>", "<a>b   \nc</a>;\n");
    expect_printed_jsx("<a>b\n   c</a>", "<a>b\n   c</a>;\n");
    expect_printed_jsx("<a>   b</a>", "<a>   b</a>;\n");
    expect_printed_jsx("<a>   \nb</a>", "<a>   \nb</a>;\n");
    expect_printed_jsx("<a>\n   b</a>", "<a>\n   b</a>;\n");
    expect_printed_jsx("<a>   \u{1f642}</a>", "<a>   \u{1f642}</a>;\n");
    expect_printed_jsx("<a>   \n\u{1f642}</a>", "<a>   \n\u{1f642}</a>;\n");
    expect_printed_jsx("<a>\n   \u{1f642}</a>", "<a>\n   \u{1f642}</a>;\n");
    expect_printed_jsx("<a>\u{1f642}   </a>", "<a>\u{1f642}   </a>;\n");
    expect_printed_jsx("<a>\u{1f642}   \n</a>", "<a>\u{1f642}   \n</a>;\n");
    expect_printed_jsx("<a>\u{1f642}\n   </a>", "<a>\u{1f642}\n   </a>;\n");
    expect_printed_jsx(
        "<a>\u{1f642}   \u{1f355}</a>",
        "<a>\u{1f642}   \u{1f355}</a>;\n",
    );
    expect_printed_jsx(
        "<a>\u{1f642}   \n\u{1f355}</a>",
        "<a>\u{1f642}   \n\u{1f355}</a>;\n",
    );
    expect_printed_jsx(
        "<a>\u{1f642}\n   \u{1f355}</a>",
        "<a>\u{1f642}\n   \u{1f355}</a>;\n",
    );
    expect_printed_jsx("<a>   \u{1f642}</a>", "<a>   \u{1f642}</a>;\n");
    expect_printed_jsx("<a>   \n\u{1f642}</a>", "<a>   \n\u{1f642}</a>;\n");
    expect_printed_jsx("<a>\n   \u{1f642}</a>", "<a>\n   \u{1f642}</a>;\n");
    expect_printed_jsx("<a>{x}<b/></a>;", "<a>{x}<b /></a>;\n");
    expect_printed_jsx("<a>\n{x}<b/></a>;", "<a>\n{x}<b /></a>;\n");
    expect_printed_jsx("<a>{x}\n<b/></a>;", "<a>{x}\n<b /></a>;\n");
    expect_printed_jsx("<a>\n{x}\n<b/></a>;", "<a>\n{x}\n<b /></a>;\n");
    expect_printed_jsx("<a>{x}<b/>\n</a>;", "<a>{x}<b />\n</a>;\n");
    expect_printed_jsx("<a>\n{x}<b/>\n</a>;", "<a>\n{x}<b />\n</a>;\n");
    expect_printed_jsx("<a>{x}\n<b/>\n</a>;", "<a>{x}\n<b />\n</a>;\n");
    expect_printed_jsx("<a>\n{x}\n<b/>\n</a>;", "<a>\n{x}\n<b />\n</a>;\n");
    expect_printed_jsx("<a> {x}<b/></a>;", "<a> {x}<b /></a>;\n");
    expect_printed_jsx("<a> {x}\n<b/></a>;", "<a> {x}\n<b /></a>;\n");
    expect_printed_jsx("<a> {x}<b/>\n</a>;", "<a> {x}<b />\n</a>;\n");
    expect_printed_jsx("<a> {x}\n<b/>\n</a>;", "<a> {x}\n<b />\n</a>;\n");
    expect_printed_jsx("<a>{x} <b/></a>;", "<a>{x} <b /></a>;\n");
    expect_printed_jsx("<a>\n{x} <b/></a>;", "<a>\n{x} <b /></a>;\n");
    expect_printed_jsx("<a>{x} <b/>\n</a>;", "<a>{x} <b />\n</a>;\n");
    expect_printed_jsx("<a>\n{x} <b/>\n</a>;", "<a>\n{x} <b />\n</a>;\n");
    expect_printed_jsx("<a> {x} <b/></a>;", "<a> {x} <b /></a>;\n");
    expect_printed_jsx("<a> {x} <b/>\n</a>;", "<a> {x} <b />\n</a>;\n");
    expect_printed_jsx("<a>{x}<b/> </a>;", "<a>{x}<b /> </a>;\n");
    expect_printed_jsx("<a>\n{x}<b/> </a>;", "<a>\n{x}<b /> </a>;\n");
    expect_printed_jsx("<a>{x}\n<b/> </a>;", "<a>{x}\n<b /> </a>;\n");
    expect_printed_jsx("<a>\n{x}\n<b/> </a>;", "<a>\n{x}\n<b /> </a>;\n");
    expect_printed_jsx("<a> {x}<b/> </a>;", "<a> {x}<b /> </a>;\n");
    expect_printed_jsx("<a> {x}\n<b/> </a>;", "<a> {x}\n<b /> </a>;\n");
    expect_printed_jsx("<a>{x} <b/> </a>;", "<a>{x} <b /> </a>;\n");
    expect_printed_jsx("<a>\n{x} <b/> </a>;", "<a>\n{x} <b /> </a>;\n");
    expect_printed_jsx("<a> {x} <b/> </a>;", "<a> {x} <b /> </a>;\n");
    expect_printed_jsx("< /**/ a/>", "<a />;\n");
    expect_printed_jsx("< //\n a/>", "<a />;\n");
    expect_printed_jsx("<a /**/ />", "<a />;\n");
    expect_printed_jsx("<a //\n />", "<a />;\n");
    expect_printed_jsx("<a/ /**/ >", "<a />;\n");
    expect_printed_jsx("<a/ //\n >", "<a />;\n");
    expect_printed_jsx("<a>b< /**/ /a>", "<a>b</a>;\n");
    expect_printed_jsx("<a>b< //\n /a>", "<a>b</a>;\n");
    expect_printed_jsx("<a>b</ /**/ a>", "<a>b</a>;\n");
    expect_printed_jsx("<a>b</ //\n a>", "<a>b</a>;\n");
    expect_printed_jsx("<a>b</a /**/ >", "<a>b</a>;\n");
    expect_printed_jsx("<a>b</a //\n >", "<a>b</a>;\n");
    expect_printed_jsx("<a> /**/ </a>", "<a> /**/ </a>;\n");
    expect_printed_jsx("<a> //\n </a>", "<a> //\n </a>;\n");
    expect_printed_jsx("<\u{20000}/>", "<\u{20000} />;\n");
    expect_printed_jsx("<a>\u{20000}</a>", "<a>\u{20000}</a>;\n");
    expect_printed_jsx("<a \u{20000}={0}/>", "<a \u{20000}={0} />;\n");
    expect_printed_jsx("<a b=<c/>/>", "<a b=<c /> />;\n");
    expect_printed_jsx("<a b=<></>/>", "<a b=<></> />;\n");
    for colon in [":", " :", ": ", " : "] {
        expect_printed_jsx(&format!("<a{colon}b/>"), "<a:b />;\n");
        expect_printed_jsx(&format!("<a-b{colon}c-d/>"), "<a-b:c-d />;\n");
        expect_printed_jsx(&format!("<a-{colon}b-/>"), "<a-:b- />;\n");
        expect_printed_jsx(&format!("<Te{colon}st/>"), "<Te:st />;\n");
        expect_printed_jsx(&format!("<x a{colon}b/>"), "<x a:b />;\n");
        expect_printed_jsx(&format!("<x a-b{colon}c-d/>"), "<x a-b:c-d />;\n");
        expect_printed_jsx(&format!("<x a-{colon}b-/>"), "<x a-:b- />;\n");
        expect_printed_jsx(&format!("<x Te{colon}st/>"), "<x Te:st />;\n");
        expect_printed_jsx(&format!("<x a{colon}b={{0}}/>"), "<x a:b={0} />;\n");
        expect_printed_jsx(&format!("<x a-b{colon}c-d={{0}}/>"), "<x a-b:c-d={0} />;\n");
        expect_printed_jsx(&format!("<x a-{colon}b-={{0}}/>"), "<x a-:b-={0} />;\n");
        expect_printed_jsx(&format!("<x Te{colon}st={{0}}/>"), "<x Te:st={0} />;\n");
        expect_printed_jsx("<a-b a-b={a-b}/>", "<a-b a-b={a - b} />;\n");
    }
}

#[test]
fn test_jsx_single_line() {
    expect_printed_jsx("<x/>", "<x />;\n");
    expect_printed_jsx("<x y/>", "<x y />;\n");
    expect_printed_jsx("<x\n/>", "<x />;\n");
    expect_printed_jsx("<x\ny/>", "<x y />;\n");
    expect_printed_jsx("<x y\n/>", "<x y />;\n");
    expect_printed_jsx("<x\n{...y}/>", "<x {...y} />;\n");
}

#[test]
#[ignore = "depends on esbuild JSX pragma handling"]
fn test_jsx_pragmas() {
    expect_printed_jsx("// @jsx h\n<a/>", "<a />;\n");
    expect_printed_jsx("/*@jsx h*/\n<a/>", "<a />;\n");
    expect_printed_jsx("/* @jsx h */\n<a/>", "<a />;\n");
    expect_printed_jsx("<a/>\n// @jsx h", "<a />;\n");
    expect_printed_jsx("<a/>\n/*@jsx h*/", "<a />;\n");
    expect_printed_jsx("<a/>\n/* @jsx h */", "<a />;\n");
    expect_printed_jsx("// @jsx a.b.c\n<a/>", "<a />;\n");
    expect_printed_jsx("/*@jsx a.b.c*/\n<a/>", "<a />;\n");
    expect_printed_jsx("/* @jsx a.b.c */\n<a/>", "<a />;\n");
    expect_printed_jsx("// @jsxFrag f\n<></>", "<></>;\n");
    expect_printed_jsx("/*@jsxFrag f*/\n<></>", "<></>;\n");
    expect_printed_jsx("/* @jsxFrag f */\n<></>", "<></>;\n");
    expect_printed_jsx("<></>\n// @jsxFrag f", "<></>;\n");
    expect_printed_jsx("<></>\n/*@jsxFrag f*/", "<></>;\n");
    expect_printed_jsx("<></>\n/* @jsxFrag f */", "<></>;\n");
    expect_printed_jsx("// @jsxFrag a.b.c\n<></>", "<></>;\n");
    expect_printed_jsx("/*@jsxFrag a.b.c*/\n<></>", "<></>;\n");
    expect_printed_jsx("/* @jsxFrag a.b.c */\n<></>", "<></>;\n");
}

#[test]
fn test_jsx_automatic() {
    expect_printed_jsx("with (x) y(<z/>)", "with (x) y(<z />);\n");
}

#[test]
#[ignore = "depends on esbuild JSX pragma handling"]
fn test_jsx_automatic_pragmas() {
    expect_printed_jsx("// @jsxRuntime automatic\n<a/>", "<a />;\n");
    expect_printed_jsx("/*@jsxRuntime automatic*/\n<a/>", "<a />;\n");
    expect_printed_jsx("/* @jsxRuntime automatic */\n<a/>", "<a />;\n");
    expect_printed_jsx("<a/>\n/*@jsxRuntime automatic*/", "<a />;\n");
    expect_printed_jsx("<a/>\n/* @jsxRuntime automatic */", "<a />;\n");
    expect_printed_jsx("// @jsxRuntime classic\n<a/>", "<a />;\n");
    expect_printed_jsx("/*@jsxRuntime classic*/\n<a/>", "<a />;\n");
    expect_printed_jsx("/* @jsxRuntime classic */\n<a/>", "<a />;\n");
    expect_printed_jsx("<a/>\n/*@jsxRuntime classic*/\n", "<a />;\n");
    expect_printed_jsx("<a/>\n/* @jsxRuntime classic */\n", "<a />;\n");
    expect_printed_jsx(
        "// @jsxRuntime automatic @jsxImportSource src\n<a/>",
        "<a />;\n",
    );
    expect_printed_jsx(
        "/*@jsxRuntime automatic @jsxImportSource src*/\n<a/>",
        "<a />;\n",
    );
    expect_printed_jsx(
        "/*@jsxRuntime automatic*//*@jsxImportSource src*/\n<a/>",
        "<a />;\n",
    );
    expect_printed_jsx(
        "/* @jsxRuntime automatic */\n/* @jsxImportSource src */\n<a/>",
        "<a />;\n",
    );
    expect_printed_jsx(
        "<a/>\n/*@jsxRuntime automatic @jsxImportSource src*/",
        "<a />;\n",
    );
    expect_printed_jsx(
        "<a/>\n/*@jsxRuntime automatic*/\n/*@jsxImportSource src*/",
        "<a />;\n",
    );
    expect_printed_jsx(
        "<a/>\n/* @jsxRuntime automatic */\n/* @jsxImportSource src */",
        "<a />;\n",
    );
    expect_printed_jsx(
        "// @jsxRuntime classic @jsxImportSource src\n<a/>",
        "<a />;\n",
    );
    expect_printed_jsx("// @jsxRuntime automatic @jsx h\n<a/>", "<a />;\n");
    expect_printed_jsx("// @jsxRuntime automatic @jsxFrag f\n<></>", "<></>;\n");
}

#[test]
fn test_jsx_side_effects() {
    expect_printed_jsx("<a/>", "<a />;\n");
    expect_printed_jsx("<></>", "<></>;\n");
}

#[test]
fn test_preserve_optional_chain_parentheses() {
    expect_printed("a?.b.c", "a?.b.c;\n");
    expect_printed("(a?.b).c", "(a?.b).c;\n");
    expect_printed("a?.b.c.d", "a?.b.c.d;\n");
    expect_printed("(a?.b.c).d", "(a?.b.c).d;\n");
    expect_printed("a?.b[c]", "a?.b[c];\n");
    expect_printed("(a?.b)[c]", "(a?.b)[c];\n");
    expect_printed("a?.b(c)", "a?.b(c);\n");
    expect_printed("(a?.b)(c)", "(a?.b)(c);\n");
    expect_printed("new (a?.b)", "new (a?.b)();\n");
    expect_printed("a?.[b][c]", "a?.[b][c];\n");
    expect_printed("(a?.[b])[c]", "(a?.[b])[c];\n");
    expect_printed("a?.[b][c][d]", "a?.[b][c][d];\n");
    expect_printed("(a?.[b][c])[d]", "(a?.[b][c])[d];\n");
    expect_printed("a?.[b].c", "a?.[b].c;\n");
    expect_printed("(a?.[b]).c", "(a?.[b]).c;\n");
    expect_printed("a?.[b](c)", "a?.[b](c);\n");
    expect_printed("(a?.[b])(c)", "(a?.[b])(c);\n");
    expect_printed("new (a?.[b])", "new (a?.[b])();\n");
    expect_printed("a?.(b)(c)", "a?.(b)(c);\n");
    expect_printed("(a?.(b))(c)", "(a?.(b))(c);\n");
    expect_printed("a?.(b)(c)(d)", "a?.(b)(c)(d);\n");
    expect_printed("(a?.(b)(c))(d)", "(a?.(b)(c))(d);\n");
    expect_printed("a?.(b).c", "a?.(b).c;\n");
    expect_printed("(a?.(b)).c", "(a?.(b)).c;\n");
    expect_printed("a?.(b)[c]", "a?.(b)[c];\n");
    expect_printed("(a?.(b))[c]", "(a?.(b))[c];\n");
    expect_printed("new (a?.(b))", "new (a?.(b))();\n");
    expect_printed("new a()?.b", "new a()?.b;\n");
    expect_printed("new a()?.[b]", "new a()?.[b];\n");
    expect_printed("new a()?.(b)", "new a()?.(b);\n");
    expect_printed("new a.b()?.c", "new a.b()?.c;\n");
    expect_printed("new a.b()?.[c]", "new a.b()?.[c];\n");
    expect_printed("new a.b()?.(c)", "new a.b()?.(c);\n");
}

#[test]
fn test_private_identifiers() {
    expect_printed("class Foo { #foo }", "class Foo {\n  #foo;\n}\n");
    expect_printed("class Foo { #foo = 1 }", "class Foo {\n  #foo = 1;\n}\n");
    expect_printed(
        "class Foo { #foo = #foo in this }",
        "class Foo {\n  #foo = #foo in this;\n}\n",
    );
    expect_printed(
        "class Foo { #foo = #foo in (#bar in this); #bar }",
        "class Foo {\n  #foo = #foo in (#bar in this);\n  #bar;\n}\n",
    );
    expect_printed(
        "class Foo { #foo() {} }",
        "class Foo {\n  #foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { get #foo() {} }",
        "class Foo {\n  get #foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { set #foo(x) {} }",
        "class Foo {\n  set #foo(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static #foo }",
        "class Foo {\n  static #foo;\n}\n",
    );
    expect_printed(
        "class Foo { static #foo = 1 }",
        "class Foo {\n  static #foo = 1;\n}\n",
    );
    expect_printed(
        "class Foo { static #foo() {} }",
        "class Foo {\n  static #foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static get #foo() {} }",
        "class Foo {\n  static get #foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { static set #foo(x) {} }",
        "class Foo {\n  static set #foo(x) {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { #\\u0066oo; foo = this.#foo }",
        "class Foo {\n  #foo;\n  foo = this.#foo;\n}\n",
    );
    expect_printed(
        "class Foo { #fo\\u006f; foo = this.#foo }",
        "class Foo {\n  #foo;\n  foo = this.#foo;\n}\n",
    );
    expect_printed(
        "class Foo { get #foo() {} set #foo(x) { this.#foo } }",
        "class Foo {\n  get #foo() {\n  }\n  set #foo(x) {\n    this.#foo;\n  }\n}\n",
    );
    expect_printed(
        "class Foo { set #foo(x) { this.#foo } get #foo() {} }",
        "class Foo {\n  set #foo(x) {\n    this.#foo;\n  }\n  get #foo() {\n  }\n}\n",
    );
    expect_printed(
        "class Foo { #foo } class Bar { #foo }",
        "class Foo {\n  #foo;\n}\nclass Bar {\n  #foo;\n}\n",
    );
    expect_printed(
        "class Foo { foo = this.#foo; #foo }",
        "class Foo {\n  foo = this.#foo;\n  #foo;\n}\n",
    );
    expect_printed(
        "class Foo { foo = this?.#foo; #foo }",
        "class Foo {\n  foo = this?.#foo;\n  #foo;\n}\n",
    );
    expect_printed(
        "class Foo {\n\t#if\n\t#im() { return this.#im(this.#if) }\n\tstatic #sf\n\tstatic #sm() { return this.#sm(this.#sf) }\n\tfoo() {\n\t\treturn class {\n\t\t\t#inner() {\n\t\t\t\treturn [this.#im, this?.#inner, this?.x.#if]\n\t\t\t}\n\t\t}\n\t}\n}\n",
        "class Foo {\n  #if;\n  #im() {\n    return this.#im(this.#if);\n  }\n  static #sf;\n  static #sm() {\n    return this.#sm(this.#sf);\n  }\n  foo() {\n    return class {\n      #inner() {\n        return [this.#im, this?.#inner, this?.x.#if];\n      }\n    };\n  }\n}\n",
    );
}

#[test]
#[ignore = "depends on esbuild import-assertion parsing"]
fn test_import_assertions() {
    expect_printed("import 'x' assert {}", "import \"x\" assert {};\n");
    expect_printed("import 'x' assert {\n}", "import \"x\" assert {};\n");
    expect_printed("import 'x' assert\n{}", "import \"x\" assert {};\n");
    expect_printed("import 'x'\nassert\n{}", "import \"x\";\nassert;\n{\n}\n");
    expect_printed(
        "import 'x' assert {type: 'json'}",
        "import \"x\" assert { type: \"json\" };\n",
    );
    expect_printed(
        "import 'x' assert {type: 'json',}",
        "import \"x\" assert { type: \"json\" };\n",
    );
    expect_printed(
        "import 'x' assert {'type': 'json'}",
        "import \"x\" assert { \"type\": \"json\" };\n",
    );
    expect_printed(
        "import 'x' assert {a: 'b', c: 'd'}",
        "import \"x\" assert { a: \"b\", c: \"d\" };\n",
    );
    expect_printed(
        "import 'x' assert {a: 'b', c: 'd',}",
        "import \"x\" assert { a: \"b\", c: \"d\" };\n",
    );
    expect_printed(
        "import 'x' assert {if: 'keyword'}",
        "import \"x\" assert { if: \"keyword\" };\n",
    );
    expect_printed(
        "import x from 'x' assert {x: 'y'}",
        "import x from \"x\" assert { x: \"y\" };\n",
    );
    expect_printed(
        "import * as x from 'x' assert {x: 'y'}",
        "import * as x from \"x\" assert { x: \"y\" };\n",
    );
    expect_printed(
        "import {} from 'x' assert {x: 'y'}",
        "import {} from \"x\" assert { x: \"y\" };\n",
    );
    expect_printed(
        "export {} from 'x' assert {x: 'y'}",
        "export {} from \"x\" assert { x: \"y\" };\n",
    );
    expect_printed(
        "export * from 'x' assert {x: 'y'}",
        "export * from \"x\" assert { x: \"y\" };\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z')",
        "x ? import(\"y\") : import(\"z\");\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {assert: {}})",
        "x ? import(\"y\", { assert: {} }) : import(\"z\", { assert: {} });\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {assert: {a: 'b'}})",
        "x ? import(\"y\", { assert: { a: \"b\" } }) : import(\"z\", { assert: { a: \"b\" } });\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {assert: {'a': 'b'}})",
        "x ? import(\"y\", { assert: { \"a\": \"b\" } }) : import(\"z\", { assert: { \"a\": \"b\" } });\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {})",
        "import(x ? \"y\" : \"z\", {});\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {assert: []})",
        "import(x ? \"y\" : \"z\", { assert: [] });\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {asserts: {}})",
        "import(x ? \"y\" : \"z\", { asserts: {} });\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {assert: {x: 1}})",
        "import(x ? \"y\" : \"z\", { assert: { x: 1 } });\n",
    );
}

#[test]
#[ignore = "depends on esbuild parser normalization"]
fn test_import_attributes() {
    expect_printed("import 'x' with {}", "import \"x\" with {};\n");
    expect_printed("import 'x' with {\n}", "import \"x\" with {};\n");
    expect_printed("import 'x' with\n{}", "import \"x\" with {};\n");
    expect_printed("import 'x'\nwith\n{}", "import \"x\" with {};\n");
    expect_printed(
        "import 'x' with {type: 'json'}",
        "import \"x\" with { type: \"json\" };\n",
    );
    expect_printed(
        "import 'x' with {type: 'json',}",
        "import \"x\" with { type: \"json\" };\n",
    );
    expect_printed(
        "import 'x' with {'type': 'json'}",
        "import \"x\" with { \"type\": \"json\" };\n",
    );
    expect_printed(
        "import 'x' with {a: 'b', c: 'd'}",
        "import \"x\" with { a: \"b\", c: \"d\" };\n",
    );
    expect_printed(
        "import 'x' with {a: 'b', c: 'd',}",
        "import \"x\" with { a: \"b\", c: \"d\" };\n",
    );
    expect_printed(
        "import 'x' with {if: 'keyword'}",
        "import \"x\" with { if: \"keyword\" };\n",
    );
    expect_printed(
        "import x from 'x' with {x: 'y'}",
        "import x from \"x\" with { x: \"y\" };\n",
    );
    expect_printed(
        "import * as x from 'x' with {x: 'y'}",
        "import * as x from \"x\" with { x: \"y\" };\n",
    );
    expect_printed(
        "import {} from 'x' with {x: 'y'}",
        "import {} from \"x\" with { x: \"y\" };\n",
    );
    expect_printed(
        "export {} from 'x' with {x: 'y'}",
        "export {} from \"x\" with { x: \"y\" };\n",
    );
    expect_printed(
        "export * from 'x' with {x: 'y'}",
        "export * from \"x\" with { x: \"y\" };\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z')",
        "x ? import(\"y\") : import(\"z\");\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {with: {}})",
        "x ? import(\"y\", { with: {} }) : import(\"z\", { with: {} });\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {with: {a: 'b'}})",
        "x ? import(\"y\", { with: { a: \"b\" } }) : import(\"z\", { with: { a: \"b\" } });\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {with: {'a': 'b'}})",
        "x ? import(\"y\", { with: { \"a\": \"b\" } }) : import(\"z\", { with: { \"a\": \"b\" } });\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {})",
        "import(x ? \"y\" : \"z\", {});\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {with: []})",
        "import(x ? \"y\" : \"z\", { with: [] });\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {whithe: {}})",
        "import(x ? \"y\" : \"z\", { whithe: {} });\n",
    );
    expect_printed(
        "import(x ? 'y' : 'z', {with: {x: 1}})",
        "import(x ? \"y\" : \"z\", { with: { x: 1 } });\n",
    );
}

#[test]
fn test_ascii_only() {
    expect_printed("\u{3c0}", "\u{3c0};\n");
    expect_printed("\u{10000}", "\u{10000};\n");
    expect_printed("var \u{3c0}", "var \u{3c0};\n");
    expect_printed("var \u{10000}", "var \u{10000};\n");
    expect_printed("'\u{3c0}'", "'\u{3c0}';\n");
    expect_printed("'\u{10000}'", "'\u{10000}';\n");
    expect_printed("x.\u{3c0}", "x.\u{3c0};\n");
    expect_printed("x.\u{10000}", "x[\"\u{10000}\"];\n");
    expect_printed("x?.\u{3c0}", "x?.\u{3c0};\n");
    expect_printed("x?.\u{10000}", "x?.[\"\u{10000}\"];\n");
    expect_printed("0 .\u{3c0}", "0 .\u{3c0};\n");
    expect_printed("0 .\u{10000}", "0[\"\u{10000}\"];\n");
    expect_printed("0?.\u{3c0}", "0?.\u{3c0};\n");
    expect_printed("0?.\u{10000}", "0?.[\"\u{10000}\"];\n");
    expect_printed("import '\u{3c0}'", "import \"\u{3c0}\";\n");
    expect_printed("import '\u{10000}'", "import \"\u{10000}\";\n");
    expect_printed("({\u{3c0}: 0})", "({ \u{3c0}: 0 });\n");
    expect_printed("({\u{10000}: 0})", "({ \"\u{10000}\": 0 });\n");
    expect_printed("({\u{3c0}})", "({ \u{3c0} });\n");
    expect_printed("({\u{10000}})", "({ \"\u{10000}\": \u{10000} });\n");
    expect_printed(
        "import * as \u{3c0} from 'path'; \u{3c0}",
        "import * as \u{3c0} from \"path\";\n\u{3c0};\n",
    );
    expect_printed(
        "import * as \u{10000} from 'path'; \u{10000}",
        "import * as \u{10000} from \"path\";\n\u{10000};\n",
    );
    expect_printed(
        "import {\u{3c0}} from 'path'; \u{3c0}",
        "import { \u{3c0} } from \"path\";\n\u{3c0};\n",
    );
    expect_printed(
        "import {\u{10000}} from 'path'; \u{10000}",
        "import { \u{10000} } from \"path\";\n\u{10000};\n",
    );
    expect_printed(
        "import {\u{3c0} as x} from 'path'",
        "import { \u{3c0} as x } from \"path\";\n",
    );
    expect_printed(
        "import {\u{10000} as x} from 'path'",
        "import { \u{10000} as x } from \"path\";\n",
    );
    expect_printed(
        "import {x as \u{3c0}} from 'path'",
        "import { x as \u{3c0} } from \"path\";\n",
    );
    expect_printed(
        "import {x as \u{10000}} from 'path'",
        "import { x as \u{10000} } from \"path\";\n",
    );
    expect_printed(
        "export * as \u{3c0} from 'path'; \u{3c0}",
        "export * as \u{3c0} from \"path\";\n\u{3c0};\n",
    );
    expect_printed(
        "export * as \u{10000} from 'path'; \u{10000}",
        "export * as \u{10000} from \"path\";\n\u{10000};\n",
    );
    expect_printed(
        "export {\u{3c0}} from 'path'; \u{3c0}",
        "export { \u{3c0} } from \"path\";\n\u{3c0};\n",
    );
    expect_printed(
        "export {\u{10000}} from 'path'; \u{10000}",
        "export { \u{10000} } from \"path\";\n\u{10000};\n",
    );
    expect_printed(
        "export {\u{3c0} as x} from 'path'",
        "export { \u{3c0} as x } from \"path\";\n",
    );
    expect_printed(
        "export {\u{10000} as x} from 'path'",
        "export { \u{10000} as x } from \"path\";\n",
    );
    expect_printed(
        "export {x as \u{3c0}} from 'path'",
        "export { x as \u{3c0} } from \"path\";\n",
    );
    expect_printed(
        "export {x as \u{10000}} from 'path'",
        "export { x as \u{10000} } from \"path\";\n",
    );
    expect_printed(
        "export {\u{3c0}}; var \u{3c0}",
        "export { \u{3c0} };\nvar \u{3c0};\n",
    );
    expect_printed(
        "export {\u{10000}}; var \u{10000}",
        "export { \u{10000} };\nvar \u{10000};\n",
    );
    expect_printed("export var \u{3c0}", "export var \u{3c0};\n");
    expect_printed("export var \u{10000}", "export var \u{10000};\n");
}

#[test]
fn test_using() {
    expect_printed("using x = y", "using x = y;\n");
    expect_printed("using x = y; z", "using x = y;\nz;\n");
    expect_printed("using x = y, z = _", "using x = y, z = _;\n");
    expect_printed("using x = y, \n z = _", "using x = y, z = _;\n");
    expect_printed("using \n x = y", "using;\nx = y;\n");
    expect_printed("using [x]", "using[x];\n");
    expect_printed("using [x] = y", "using[x] = y;\n");
    expect_printed("using \n [x] = y", "using[x] = y;\n");
    expect_printed("for (using x = y;;) ;", "for (using x = y; ; ) ;\n");
    expect_printed("for (using x of y) ;", "for (using x of y) ;\n");
    expect_printed("for (using of x) ;", "for (using of x) ;\n");
    expect_printed("for (using of of) ;", "for (using of of) ;\n");
    expect_printed(
        "for (await using of of x) ;",
        "for (await using of of x) ;\n",
    );
    expect_printed(
        "for (await using of of of) ;",
        "for (await using of of of) ;\n",
    );
    expect_printed("for await (using x of y) ;", "for await (using x of y) ;\n");
    expect_printed("for await (using of x) ;", "for await (using of x) ;\n");
    expect_printed("await using \n x = y", "await using;\nx = y;\n");
    expect_printed("await \n using \n x \n = \n y", "await using;\nx = y;\n");
    expect_printed("await using [x]", "await using[x];\n");
    expect_printed("await using ([x] = y)", "await using([x] = y);\n");
    expect_printed("await (using [x] = y)", "await (using[x] = y);\n");
    expect_printed("await using x = y", "await using x = y;\n");
    expect_printed("await using x = y, z = _", "await using x = y, z = _;\n");
    expect_printed("for (await using x of y) ;", "for (await using x of y) ;\n");
    expect_printed(
        "for await (await using x of y) ;",
        "for await (await using x of y) ;\n",
    );
    expect_printed(
        "function foo() { using x = y }",
        "function foo() {\n  using x = y;\n}\n",
    );
    expect_printed(
        "foo = function() { using x = y }",
        "foo = function() {\n  using x = y;\n};\n",
    );
    expect_printed(
        "foo = () => { using x = y }",
        "foo = () => {\n  using x = y;\n};\n",
    );
    expect_printed(
        "async function foo() { using x = y }",
        "async function foo() {\n  using x = y;\n}\n",
    );
    expect_printed(
        "foo = async function() { using x = y }",
        "foo = async function() {\n  using x = y;\n};\n",
    );
    expect_printed(
        "foo = async () => { using x = y }",
        "foo = async () => {\n  using x = y;\n};\n",
    );
    expect_printed(
        "async function foo() { await using x = y }",
        "async function foo() {\n  await using x = y;\n}\n",
    );
    expect_printed(
        "foo = async function() { await using x = y }",
        "foo = async function() {\n  await using x = y;\n};\n",
    );
    expect_printed(
        "foo = async () => { await using x = y }",
        "foo = async () => {\n  await using x = y;\n};\n",
    );
    expect_printed("using x = {}", "using x = {};\n");
    expect_printed("using x = null", "using x = null;\n");
    // Depends on esbuild's parser-stage undefined normalization.
    // expect_printed("using x = undefined", "using x = void 0;\n");
    expect_printed("using x = (foo, y)", "using x = (foo, y);\n");
    expect_printed("using x = (foo, null)", "using x = (foo, null);\n");
    // Depends on esbuild's parser-stage undefined normalization.
    // expect_printed("using x = (foo, undefined)", "using x = (foo, void 0);\n");
}
