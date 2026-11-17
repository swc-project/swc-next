mod support {
    pub(crate) mod options;
}

use support::options::TestOptions;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::{NoTokenParserConfig, Parser};
use swc_next_ecma_semantic2::{AnalyzeOptions, SyntaxChecks};

impl TestOptions {
    fn syntax(mut self, checks: SyntaxChecks) -> Self {
        self.checks = checks;
        self
    }
}

#[track_caller]
fn error_spans(source: &str, options: TestOptions) -> Vec<(u32, u32)> {
    let allocator = Allocator::new();
    let parsed = Parser::init(&allocator, source, options.parser, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let result = swc_next_ecma_semantic2::analyze(
        &parsed.ast,
        AnalyzeOptions {
            check_syntax: options.checks,
        },
    );
    let diagnostics = swc_next_ecma_semantic2::check_syntax(&parsed.ast, options.checks);
    assert_eq!(diagnostics, result.diagnostics);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity == swc_next_ecma_ast::Severity::Error)
    );
    diagnostics
        .into_iter()
        .map(|diagnostic| (diagnostic.span.start, diagnostic.span.end))
        .collect()
}

#[test]
#[rustfmt::skip]
fn class_function_conflicts() {
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);

    let actual = error_spans("{ class f {} function f() {} function f() {} }", ts_script);
    assert_eq!(actual, [(22, 23), (38, 39)]);

    let actual = error_spans("{ function f() {} class f {} function f() {} }", ts_script);
    assert_eq!(actual, [(24, 25), (38, 39)]);
}

#[test]
#[rustfmt::skip]
fn function_declarations_annex_b() {
    let js_script = TestOptions::new(Lang::Js, SourceType::Script);
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);
    let js_module = TestOptions::new(Lang::Js, SourceType::Module);
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("let f = 1;\nif (true) function f() { \"use strict\"; }", js_script);
    assert_eq!(actual, []);

    let actual = error_spans("if (true) function f() { \"use strict\"; }", js_script);
    assert_eq!(actual, []);

    let actual = error_spans("if (true) function eval() { \"use strict\"; }", js_script);
    assert_eq!(actual, [(19, 23)]);

    let actual = error_spans("if (true) function f(value = 1) { \"use strict\"; }", js_script);
    assert_eq!(actual, [(34, 47)]);

    let actual = error_spans("label: function f() { \"use strict\"; }", js_script);
    assert_eq!(actual, []);

    let actual = error_spans("function outer() {\n  if (true) function f() { \"use strict\"; }\n}", js_script);
    assert_eq!(actual, []);

    let actual = error_spans("\"use strict\";\nif (true) function f() {}", js_script);
    assert_eq!(actual, [(24, 39)]);

    let actual = error_spans("\"use strict\";\nlabel: function f() {}", js_script);
    assert_eq!(actual, [(21, 36)]);

    let actual = error_spans("let f = 1;\nif (true) function f() { \"use strict\"; }", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("if (true) function f() { \"use strict\"; }", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("if (true) function eval() { \"use strict\"; }", ts_script);
    assert_eq!(actual, [(19, 23)]);

    let actual = error_spans("if (true) function f(value = 1) { \"use strict\"; }", ts_script);
    assert_eq!(actual, [(34, 47)]);

    let actual = error_spans("label: function f() { \"use strict\"; }", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("function outer() {\n  if (true) function f() { \"use strict\"; }\n}", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("\"use strict\";\nif (true) function f() {}", ts_script);
    assert_eq!(actual, [(24, 39)]);

    let actual = error_spans("\"use strict\";\nlabel: function f() {}", ts_script);
    assert_eq!(actual, [(21, 36)]);

    let actual = error_spans("if (true) function f() { \"use strict\"; }", js_module);
    assert_eq!(actual, [(10, 40)]);

    let actual = error_spans("label: function f() { \"use strict\"; }", js_module);
    assert_eq!(actual, [(7, 37)]);

    let actual = error_spans("if (true) function f() { \"use strict\"; }", ts_module);
    assert_eq!(actual, [(10, 40)]);

    let actual = error_spans("label: function f() { \"use strict\"; }", ts_module);
    assert_eq!(actual, [(7, 37)]);
}

#[test]
#[rustfmt::skip]
fn function_declarations_arrow_body() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("() => {\n  function f() {}\n  var f;\n};", ts_module);
    assert_eq!(actual, [(32, 33)]);

    let actual = error_spans("() => {\n  var f;\n  function f() {}\n};", ts_module);
    assert_eq!(actual, [(28, 29)]);
}

#[test]
#[rustfmt::skip]
fn function_declarations_block() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);
    let js_script = TestOptions::new(Lang::Js, SourceType::Script);

    let actual = error_spans("{\n  async function f() {}\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(37, 38)]);

    let actual = error_spans("{\n  class f {}\n  function f() {}\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(26, 27), (44, 45)]);

    let actual = error_spans("{\n  async function f() {}\n  async function f() {}\n}", ts_module);
    assert_eq!(actual, [(43, 44)]);

    let actual = error_spans("{\n  function* f() {}\n  function* f() {}\n}", ts_module);
    assert_eq!(actual, [(33, 34)]);

    let actual = error_spans("{\n  function f() {}\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(31, 32)]);

    let actual = error_spans("{\n  function f() {}\n  async function f() {}\n}", ts_module);
    assert_eq!(actual, [(37, 38)]);

    let actual = error_spans("{\n  function f() {}\n  function* f() {}\n}", ts_module);
    assert_eq!(actual, [(32, 33)]);

    let actual = error_spans("{\n  function f() {}\n  var f;\n}", ts_module);
    assert_eq!(actual, [(26, 27)]);

    let actual = error_spans("{\n  function* f() {}\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(32, 33)]);

    let actual = error_spans("{\n  function f(): void;\n  function f() {}\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(53, 54)]);

    let actual = error_spans("{\n  function f(): void;\n  function f() {}\n}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("{\n  var f;\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(22, 23)]);

    let actual = error_spans("{\n  var f;\n  function f(): void;\n}", ts_module);
    assert_eq!(actual, [(22, 23)]);

    let actual = error_spans("{\n  var f;\n  interface f {}\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(39, 40)]);

    let actual = error_spans("{\n  function f() {}\n  function f() {}\n}", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("\"use strict\";\n{ function f() {} function f() {} }", ts_script);
    assert_eq!(actual, [(41, 42)]);

    let actual = error_spans("{\n  function f() { \"use strict\"; }\n  function f() { \"use strict\"; }\n}", js_script);
    assert_eq!(actual, []);

    let actual = error_spans("{\n  function f() { \"use strict\"; }\n  function f() { \"use strict\"; }\n}", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("function f() {}\n{\n  var f;\n  function f() {}\n}", js_script);
    assert_eq!(actual, [(38, 39)]);
}

#[test]
#[rustfmt::skip]
fn function_declarations_catch() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);
    let js_module = TestOptions::new(Lang::Js, SourceType::Module);

    let actual = error_spans("try {} catch {\n  function f() {}\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(44, 45)]);

    let actual = error_spans("try {} catch {\n  function f() {}\n  var f;\n}", ts_module);
    assert_eq!(actual, [(39, 40)]);

    let actual = error_spans("try {} catch {\n  function f(): void;\n  function f() {}\n}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("try {} catch (f) {\n  function f(): void;\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(30, 31), (52, 53)]);

    let actual = error_spans("try {} catch ([error = 1]) {\n  function f() {}\n  var f;\n}", ts_module);
    assert_eq!(actual, [(53, 54)]);

    let actual = error_spans("try {} catch ([error = 1]) {\n  function f(): void;\n  function f() {}\n}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("try {} catch ([f = 1]) {\n  function f(): void;\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(36, 37), (58, 59)]);

    let actual = error_spans("try {} catch ([f = 1]) {\n  function f(): void;\n}", ts_module);
    assert_eq!(actual, [(36, 37)]);

    let actual = error_spans("try {} catch ([error = 1]) {\n  var f;\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(49, 50)]);

    let actual = error_spans("try {} catch {\n  var f;\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(35, 36)]);

    let actual = error_spans("try {} catch {\n  function f() {}\n  function f() {}\n}", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("try {} catch ([f = 1]) {\n  {\n    function f() {}\n  }\n}", js_module);
    assert_eq!(actual, []);

    let actual = error_spans("try {} catch ([f = 1]) {\n  function f() {}\n}", js_module);
    assert_eq!(actual, [(36, 37)]);

    let actual = error_spans("try {} catch ([f = 1]) {\n  {\n    function f() {}\n  }\n}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("try {} catch ([f = 1]) {\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(36, 37)]);
}

#[test]
#[rustfmt::skip]
fn function_declarations_function_body() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);
    let js_module = TestOptions::new(Lang::Js, SourceType::Module);

    let actual = error_spans("function outer() {\n  function f() {}\n  var f;\n}", ts_module);
    assert_eq!(actual, [(43, 44)]);

    let actual = error_spans("function outer(f: unknown) {\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(40, 41)]);

    let actual = error_spans("function outer(f = 1) {\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(35, 36)]);

    let actual = error_spans("function outer() {\n  var f;\n  function f() {}\n}", ts_module);
    assert_eq!(actual, [(39, 40)]);

    let actual = error_spans("function outer(f) {\n  function f() {}\n}", js_module);
    assert_eq!(actual, []);

    let actual = error_spans("function outer(f = 1) {\n  function f() {}\n}", js_module);
    assert_eq!(actual, []);
}

#[test]
#[rustfmt::skip]
fn function_declarations_namespace() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("namespace N {\n  export function f() {}\n}\nnamespace N {\n  export var f: unknown;\n}", ts_module);
    assert_eq!(actual, [(68, 69)]);

    let actual = error_spans("namespace N {\n  export var f: unknown;\n}\nnamespace N {\n  export function f() {}\n}", ts_module);
    assert_eq!(actual, [(73, 74)]);
}

#[test]
#[rustfmt::skip]
fn function_declarations_program() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);
    let dts_module = TestOptions::new(Lang::Dts, SourceType::Module);

    let actual = error_spans("declare class f {}\nfunction f(): void;\nfunction f() {}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("function f() {}\ndeclare class f {}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("function f() {}\nvar f;", ts_module);
    assert_eq!(actual, [(20, 21)]);

    let actual = error_spans("var f;\nfunction f() {}", ts_module);
    assert_eq!(actual, [(16, 17)]);

    let actual = error_spans("declare class f {}\nfunction f(): void;\nfunction f() {}", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("function f() {}\ndeclare class f {}", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("function f() {}\nvar f;", ts_script);
    assert_eq!(actual, [(20, 21)]);

    let actual = error_spans("var f;\nfunction f() {}", ts_script);
    assert_eq!(actual, [(16, 17)]);

    let actual = error_spans("declare function f(): void;\ndeclare var f: unknown;", dts_module);
    assert_eq!(actual, [(40, 41)]);

    let actual = error_spans("declare var f: unknown;\ndeclare function f(): void;", dts_module);
    assert_eq!(actual, [(41, 42)]);
}

#[test]
#[rustfmt::skip]
fn function_declarations_static_block() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);
    let js_module = TestOptions::new(Lang::Js, SourceType::Module);

    let actual = error_spans("class C {\n  static {\n    async function f() {}\n    async function f() {}\n  }\n}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("class C {\n  static {\n    function* f() {}\n    function* f() {}\n  }\n}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("class C {\n  static {\n    function f() {}\n    var f;\n  }\n}", ts_module);
    assert_eq!(actual, [(49, 50)]);

    let actual = error_spans("class C {\n  static {\n    function f(): void;\n    function f() {}\n  }\n}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("class C {\n  static {\n    var f;\n    function f() {}\n  }\n}", ts_module);
    assert_eq!(actual, [(45, 46)]);

    let actual = error_spans("class C {\n  static {\n    function f() {}\n    function f() {}\n  }\n}", js_module);
    assert_eq!(actual, []);

    let actual = error_spans("class C {\n  static {\n    function f() {}\n    function f() {}\n  }\n}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("class C {\n  static {\n    function f() {}\n    var f;\n  }\n}", js_module);
    assert_eq!(actual, []);

    let actual = error_spans("class C {\n  static {\n    var f;\n    function f() {}\n  }\n}", js_module);
    assert_eq!(actual, []);
}

#[test]
#[rustfmt::skip]
fn function_declarations_switch() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);

    let actual = error_spans("switch (0) {\n  case 0:\n    function f() {}\n    function f() {}\n}", ts_module);
    assert_eq!(actual, [(56, 57)]);

    let actual = error_spans("switch (0) {\n  case 0:\n    function f() {}\n    var f;\n}", ts_module);
    assert_eq!(actual, [(51, 52)]);

    let actual = error_spans("switch (0) {\n  case 0:\n    function f(): void;\n    function f() {}\n}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("switch (0) {\n  case 0:\n    var f;\n    function f() {}\n}", ts_module);
    assert_eq!(actual, [(47, 48)]);

    let actual = error_spans("switch (0) {\n  case 0:\n    function f() {}\n    function f() {}\n}", ts_script);
    assert_eq!(actual, []);
}

#[test]
#[rustfmt::skip]
fn hoisting_variables() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("{\n  const enum E {}\n  var E;\n}", ts_module);
    assert_eq!(actual, [(26, 27)]);

    let actual = error_spans("{\n  enum E {}\n  var E;\n}", ts_module);
    assert_eq!(actual, [(20, 21)]);

    let actual = error_spans("{\n  var E;\n  const enum E {}\n}", ts_module);
    assert_eq!(actual, [(24, 25)]);

    let actual = error_spans("{\n  var E;\n  enum E {}\n}", ts_module);
    assert_eq!(actual, [(18, 19)]);
}

#[test]
#[rustfmt::skip]
fn default_export() {
    let js_module = TestOptions::new(Lang::Js, SourceType::Module);

    let actual = error_spans("let f;\nexport default\nasync function f() {}", js_module);
    assert_eq!(actual, [(37, 38)]);

    let actual = error_spans("export default\nasync function f() {}\nexport { f };", js_module);
    assert_eq!(actual, []);
}

#[test]
#[rustfmt::skip]
fn namespace_export_aliases() {
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);
    let dts_module = TestOptions::new(Lang::Dts, SourceType::Module);

    let actual = error_spans("interface X {} declare namespace N { export { X } }", ts_script);
    assert_eq!(actual, [(46, 47)]);

    let actual = error_spans("declare namespace N { const a: number; export { a as x } } declare namespace N { const b: string; export { b as x } }", dts_module);
    assert_eq!(actual, [(107, 113)]);

    let actual = error_spans("declare namespace N { export const x: number } declare namespace N { const y: string; export { y as x } }", dts_module);
    assert_eq!(actual, [(95, 101)]);

    let actual = error_spans("declare namespace N { export const x: number } declare namespace N { interface Y {} export type { Y as x } }", dts_module);
    assert_eq!(actual, []);

    let actual = error_spans("declare namespace N { namespace A {} export import X = A; export { X } }", dts_module);
    assert_eq!(actual, [(67, 68)]);

    let actual = error_spans("declare namespace N { namespace A { const value: number } import Y = A; export const X: number; export { Y as X } }", dts_module);
    assert_eq!(actual, [(105, 111)]);

    let actual = error_spans("declare namespace N { export const X: number; interface X {} export interface Y {} export { X as Y } }", dts_module);
    assert_eq!(actual, [(92, 98)]);

    let actual = error_spans("declare namespace N { export const X: number; interface X {} export const Y: string; export { X as Y } }", dts_module);
    assert_eq!(actual, []);
}

#[test]
#[rustfmt::skip]
fn namespace_export_declaration_spaces() {
    let dts_module = TestOptions::new(Lang::Dts, SourceType::Module);
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("declare namespace A { interface T {} } declare namespace N { export import X = A.T; interface X {} }", dts_module);
    assert_eq!(actual, [(75, 76)]);

    let actual = error_spans("declare namespace A { interface T {} } declare namespace N { interface X {} export import X = A.T }", dts_module);
    assert_eq!(actual, [(90, 91)]);

    let actual = error_spans("declare namespace A { const T: number } declare namespace N { export import X = A.T; interface X {} }", dts_module);
    assert_eq!(actual, []);

    let actual = error_spans("declare enum E { A } declare namespace E { interface X {} export type { X as A } }", dts_module);
    assert_eq!(actual, [(72, 78)]);

    let actual = error_spans("declare namespace A { namespace T { interface U {} } export type { T as X } } declare namespace B { export import Y = A.X.U }", dts_module);
    assert_eq!(actual, [(118, 123)]);

    let actual = error_spans("declare namespace N { interface T {} export type { T as x } } declare namespace M { export import y = N.x }", dts_module);
    assert_eq!(actual, [(102, 105)]);

    let actual = error_spans("declare namespace N { export interface x {} } declare namespace M { export import y = N.x; interface y {} }", dts_module);
    assert_eq!(actual, [(82, 83)]);

    let actual = error_spans("namespace A {} namespace N { interface A {} import X = A; interface X {} }", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("namespace A { export interface T {} } import B = A; import X = B.T; type X = number", ts_module);
    assert_eq!(actual, [(59, 60)]);

    let actual = error_spans("namespace A { export interface T {} } namespace N { import X = A.T; export interface X {} }", ts_module);
    assert_eq!(actual, [(59, 60)]);

    let actual = error_spans("namespace A { export interface T {} } namespace N { import X = A.T } namespace N { export interface X {} }", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("declare namespace N { export const X: number; interface X {} export { X as Y } } import Alias = N.Y; declare const Alias: string", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("declare namespace N { export const X: number; interface X {} export { X as Y } } import Alias = N.Y; interface Alias {}", ts_module);
    assert_eq!(actual, [(88, 93)]);
}

#[test]
#[rustfmt::skip]
fn augmentation_contexts() {
    let ts_commonjs = TestOptions::new(Lang::Ts, SourceType::CommonJs);
    let dts_script = TestOptions::new(Lang::Dts, SourceType::Script);
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);

    let actual = error_spans("declare global {}", ts_commonjs);
    assert_eq!(actual, []);

    let actual = error_spans("declare module 'pkg' { global {} }", ts_commonjs);
    assert_eq!(actual, [(23, 29)]);

    let actual = error_spans("global {}", ts_commonjs);
    assert_eq!(actual, [(0, 6)]);

    let actual = error_spans("declare module 'outer' { module 'inner' {} }", ts_commonjs);
    assert_eq!(actual, [(32, 39)]);

    let actual = error_spans("module 'pkg' { global {} }", dts_script);
    assert_eq!(actual, []);

    let actual = error_spans("module 'pkg' { global {} }", ts_script);
    assert_eq!(actual, [(15, 21)]);
}

#[test]
#[rustfmt::skip]
fn class_definition_super_calls() {
    let js_module = TestOptions::new(Lang::Js, SourceType::Module);

    let actual = error_spans("class B {} class C extends B { constructor() { class Inner { [(super(), 'x')] = 0; } } }", js_module);
    assert_eq!(actual, []);

    let actual = error_spans("class B {} class C extends B { constructor() { class Inner { x = super(); } } }", js_module);
    assert_eq!(actual, [(65, 72)]);
}

#[test]
#[rustfmt::skip]
fn global_augmentation() {
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("declare module \"pkg\" { global {} }", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("declare namespace N { global {} }", ts_script);
    assert_eq!(actual, [(22, 28)]);

    let actual = error_spans("{ declare global {} }", ts_script);
    assert_eq!(actual, [(2, 19)]);

    let actual = error_spans("function f() { declare global {} }", ts_script);
    assert_eq!(actual, [(15, 32)]);

    let actual = error_spans("declare global {}", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("global {}", ts_module);
    assert_eq!(actual, [(0, 6)]);

    let actual = error_spans("declare module \"pkg\" { global {} }", ts_module);
    assert_eq!(actual, [(23, 29)]);

    let actual = error_spans("declare global { global {} }", ts_module);
    assert_eq!(actual, [(17, 23)]);
}

#[test]
#[rustfmt::skip]
fn global_augmentation_recovery() {
    let ts_unambiguous = TestOptions::new(Lang::Ts, SourceType::Unambiguous);

    let actual = error_spans("export const value = 0; global { global { let copy: typeof value; } }", ts_unambiguous);
    assert_eq!(actual, [(33, 39), (33, 39), (24, 30)]);

    let actual = error_spans("export {}; declare global { global {} }", ts_unambiguous);
    assert_eq!(actual, [(28, 34)]);
}

#[test]
#[rustfmt::skip]
fn namespace_context() {
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("declare module \"outer\" { module \"inner\" {} }", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("declare module \"outer\" { module \"inner\" {} }", ts_module);
    assert_eq!(actual, [(32, 39)]);
}

#[test]
#[rustfmt::skip]
fn namespace_lexical_visibility() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("namespace N { export class X {} } namespace N { export interface X {}; const X = 1 }", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("namespace N { export class X {} } namespace N { const X = 1; export interface X {} }", ts_module);
    assert_eq!(actual, []);
}

#[test]
#[rustfmt::skip]
fn using_declarations() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);

    let actual = error_spans("declare namespace N { using x: null; }", ts_module);
    assert_eq!(actual, [(22, 36)]);

    let actual = error_spans("namespace N { using x = null; }", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("declare namespace N { using x: null; }", ts_script);
    assert_eq!(actual, [(22, 36)]);

    let actual = error_spans("namespace N { using x = null; }", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("namespace N.M { using x = null; }", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("namespace N { { using x = null; } }", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("namespace N { export namespace M { using x = null; } }", ts_script);
    assert_eq!(actual, []);

    let actual = error_spans("using x = null;", ts_script);
    assert_eq!(actual, [(0, 15)]);
}

#[test]
#[rustfmt::skip]
fn abstract_method_syntax_check() {
    let ts_module_unchecked = TestOptions::new(Lang::Ts, SourceType::Module).syntax(SyntaxChecks::empty());
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("abstract class A { abstract m() {} }", ts_module_unchecked);
    assert_eq!(actual, []);

    let actual = error_spans("abstract class A { abstract m() {} }", ts_module);
    assert_eq!(actual, [(19, 34)]);
}

#[test]
#[rustfmt::skip]
fn class_member_name_syntax_check() {
    let js_module_unchecked = TestOptions::new(Lang::Js, SourceType::Module).syntax(SyntaxChecks::empty());
    let js_module_without_member_names = TestOptions::new(Lang::Js, SourceType::Module).syntax(SyntaxChecks::all() - SyntaxChecks::CLASS_MEMBER_NAMES);
    let js_module_member_names = TestOptions::new(Lang::Js, SourceType::Module).syntax(SyntaxChecks::CLASS_MEMBER_NAMES);
    let ts_module_unchecked = TestOptions::new(Lang::Ts, SourceType::Module).syntax(SyntaxChecks::empty());
    let ts_module_without_member_names = TestOptions::new(Lang::Ts, SourceType::Module).syntax(SyntaxChecks::all() - SyntaxChecks::CLASS_MEMBER_NAMES);
    let ts_module_member_names = TestOptions::new(Lang::Ts, SourceType::Module).syntax(SyntaxChecks::CLASS_MEMBER_NAMES);

    let actual = error_spans("class C { static prototype() {} }", js_module_unchecked);
    assert_eq!(actual, []);

    let actual = error_spans("class C { static prototype() {} }", js_module_without_member_names);
    assert_eq!(actual, []);

    let actual = error_spans("class C { static prototype() {} }", js_module_member_names);
    assert_eq!(actual, [(17, 26)]);

    let actual = error_spans("class C { static prototype; }", js_module_unchecked);
    assert_eq!(actual, []);

    let actual = error_spans("class C { static prototype; }", js_module_without_member_names);
    assert_eq!(actual, []);

    let actual = error_spans("class C { static prototype; }", js_module_member_names);
    assert_eq!(actual, [(17, 26)]);

    let actual = error_spans("class C { static prototype(): void; }", ts_module_unchecked);
    assert_eq!(actual, []);

    let actual = error_spans("class C { static prototype(): void; }", ts_module_without_member_names);
    assert_eq!(actual, []);

    let actual = error_spans("class C { static prototype(): void; }", ts_module_member_names);
    assert_eq!(actual, [(17, 26)]);
}

#[test]
#[rustfmt::skip]
fn computed_parameter_keys() {
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("type F = ({ [(() => { const yield = \"prop\"; return yield; })()]: value }: { prop: number }) => typeof value;", ts_module);
    assert_eq!(actual, [(28, 33), (51, 56)]);

    let actual = error_spans("type F = ({ [\"\\1\"]: value }: object) => typeof value;", ts_module);
    assert_eq!(actual, [(13, 17)]);

    let actual = error_spans("type F = ({ [010]: value }: { 8: number }) => typeof value;", ts_module);
    assert_eq!(actual, [(13, 16)]);

    let actual = error_spans("type F = ({ [yield]: value }: object) => typeof value;", ts_module);
    assert_eq!(actual, [(13, 18)]);
}

#[test]
#[rustfmt::skip]
fn contextual_expression_recovery() {
    let js_script = TestOptions::new(Lang::Js, SourceType::Script);
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = error_spans("function read(await, yield) { await + yield; await(value); yield[value]; }", js_script);
    assert_eq!(actual, []);

    let actual = error_spans("function read(await, yield) { await\nvalue; yield\nvalue; }", js_script);
    assert_eq!(actual, []);

    let actual = error_spans("async function read() { await value; }", ts_module);
    assert_eq!(actual, []);

    let actual = error_spans("function* read() { yield value; }", ts_module);
    assert_eq!(actual, []);
}

#[test]
#[rustfmt::skip]
fn regular_expression_diagnostics() {
    let js_module = TestOptions::new(Lang::Js, SourceType::Module);
    let js_module_unchecked = TestOptions::new(Lang::Js, SourceType::Module).syntax(SyntaxChecks::empty());

    let actual = error_spans(" /a/gg;", js_module);
    assert_eq!(actual, [(1, 6)]);

    let actual = error_spans(" /a/gg;", js_module_unchecked);
    assert_eq!(actual, []);

    let actual = error_spans(" /🥹[z-a]/u;", js_module);
    assert_eq!(actual, [(1, 13)]);

    let actual = error_spans(" /(?<x>a)(?<x>b)/;", js_module);
    assert_eq!(actual, [(1, 17)]);

    let without_regexp = js_module.syntax(SyntaxChecks::all() - SyntaxChecks::REGEXP_LITERALS);

    let actual = error_spans(" /a/gg;", without_regexp);
    assert_eq!(actual, []);
}
