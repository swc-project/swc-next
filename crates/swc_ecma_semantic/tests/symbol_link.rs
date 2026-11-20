mod support {
    pub(crate) mod analysis;
    pub(crate) mod options;
}

use support::{analysis::with_semantic, options::TestOptions};
use swc_next_allocator::Idx;
use swc_next_ecma_ast::{Ast, Lang, SourceType};
use swc_next_ecma_semantic::{ScopeModel, Semantic, SyntaxChecks};

#[track_caller]
fn symbol_links(source: &str, options: TestOptions, name: &str) -> Vec<Vec<usize>> {
    let inspect = |ast: &Ast<'_>, semantic: Semantic<'_>| {
        let symbols = semantic
            .iter_symbols()
            .filter(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == name.as_bytes())
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        assert!(!symbols.is_empty());
        let mut groups = Vec::new();
        for &symbol in &symbols {
            let mut group = semantic.linked_symbols(symbol).collect::<Vec<_>>();
            group.sort_unstable();
            assert!(group.contains(&symbol));
            assert!(group.windows(2).all(|pair| pair[0] != pair[1]));
            assert!(group.iter().all(|id| symbols.contains(id)));
            groups.push(group.iter().map(|id| id.index()).collect::<Vec<_>>());
        }
        groups.sort_unstable();
        groups.dedup();
        assert_eq!(groups.iter().map(Vec::len).sum::<usize>(), symbols.len());
        let model = ScopeModel::new(semantic);
        for symbol in symbols {
            let mut linked = model
                .linked_symbols(symbol)
                .map(Idx::index)
                .collect::<Vec<_>>();
            linked.sort_unstable();
            assert!(linked.contains(&symbol.index()));
            assert!(groups.contains(&linked));
        }
        groups
    };
    let checked = with_semantic(source, options, inspect);
    let unchecked = with_semantic(source, options.syntax(SyntaxChecks::empty()), inspect);
    assert_eq!(checked, unchecked);
    checked
}

#[test]
#[rustfmt::skip]
fn annex_b_linked_symbols() {
    let js_script = TestOptions::new(Lang::Js, SourceType::Script);
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);

    let actual = symbol_links("var f; { function f() { return f; } } f;", js_script, "f");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("{ function f() { return f; } } f;", js_script, "f");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("var f; { function f() {} } { function f() {} } f;", js_script, "f");
    assert_eq!(actual, [vec![0, 1, 2]]);

    let actual = symbol_links("if (condition) function f() { return f; } f;", js_script, "f");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("try {} catch (f) { var f; } { function f() {} } f;", js_script, "f");
    assert_eq!(actual, [vec![0, 1, 2]]);

    let actual = symbol_links("'use strict'; var f; { function f() {} } f;", js_script, "f");
    assert_eq!(actual, [vec![0], vec![1]]);

    let actual = symbol_links("let f; { function f() {} } f;", js_script, "f");
    assert_eq!(actual, [vec![0], vec![1]]);

    let actual = symbol_links("function outer(f = 1) { { function f() {} } return f; }", js_script, "f");
    assert_eq!(actual, [vec![1], vec![2]]);

    let actual = symbol_links("var f; { function f() { return f; } } f;", ts_script, "f");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("{ function f() { return f; } } f;", ts_script, "f");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("var f; { function f() {} } { function f() {} } f;", ts_script, "f");
    assert_eq!(actual, [vec![0, 1, 2]]);

    let actual = symbol_links("if (condition) function f() { return f; } f;", ts_script, "f");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("try {} catch (f) { var f; } { function f() {} } f;", ts_script, "f");
    assert_eq!(actual, [vec![0, 1, 2]]);

    let actual = symbol_links("'use strict'; var f; { function f() {} } f;", ts_script, "f");
    assert_eq!(actual, [vec![0], vec![1]]);

    let actual = symbol_links("let f; { function f() {} } f;", ts_script, "f");
    assert_eq!(actual, [vec![0], vec![1]]);

    let actual = symbol_links("function outer(f = 1) { { function f() {} } return f; }", ts_script, "f");
    assert_eq!(actual, [vec![1], vec![2]]);
}

#[test]
#[rustfmt::skip]
fn catch_linked_symbols() {
    let js_script = TestOptions::new(Lang::Js, SourceType::Script);
    let ts_script = TestOptions::new(Lang::Ts, SourceType::Script);

    let actual = symbol_links("var e; try {} catch (e) { var e = 1; e; } e;", js_script, "e");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("try {} catch (e) { var e; } var e; e;", js_script, "e");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("var e; try {} catch (e) { var e; var e; }", js_script, "e");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("var e; try {} catch (e) { try {} catch (e) { var e; } }", js_script, "e");
    assert_eq!(actual, [vec![0, 1, 2]]);

    let actual = symbol_links("try {} catch (e) { var e; } try {} catch (e) { var e; }", js_script, "e");
    assert_eq!(actual, [vec![0, 1, 2]]);

    let actual = symbol_links("try {} catch (e) { for (var e of values) e; } e;", js_script, "e");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("try {} catch (e) { var [e] = values; } e;", js_script, "e");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("var e; try {} catch (e) { e; }", js_script, "e");
    assert_eq!(actual, [vec![0], vec![1]]);

    let actual = symbol_links("var e; try {} catch (e) { function f() { var e; } }", js_script, "e");
    assert_eq!(actual, [vec![0], vec![1], vec![3]]);

    let actual = symbol_links("var e; try {} catch (e) { var e = 1; e; } e;", ts_script, "e");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("try {} catch (e) { var e; } var e; e;", ts_script, "e");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("var e; try {} catch (e) { var e; var e; }", ts_script, "e");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("var e; try {} catch (e) { try {} catch (e) { var e; } }", ts_script, "e");
    assert_eq!(actual, [vec![0, 1, 2]]);

    let actual = symbol_links("try {} catch (e) { var e; } try {} catch (e) { var e; }", ts_script, "e");
    assert_eq!(actual, [vec![0, 1, 2]]);

    let actual = symbol_links("try {} catch (e) { for (var e of values) e; } e;", ts_script, "e");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("try {} catch (e) { var [e] = values; } e;", ts_script, "e");
    assert_eq!(actual, [vec![0, 1]]);

    let actual = symbol_links("var e; try {} catch (e) { e; }", ts_script, "e");
    assert_eq!(actual, [vec![0], vec![1]]);

    let actual = symbol_links("var e; try {} catch (e) { function f() { var e; } }", ts_script, "e");
    assert_eq!(actual, [vec![0], vec![1], vec![3]]);
}

#[test]
#[rustfmt::skip]
fn parameter_linked_symbols() {
    let js_module = TestOptions::new(Lang::Js, SourceType::Module);
    let ts_module = TestOptions::new(Lang::Ts, SourceType::Module);

    let actual = symbol_links("function f(a = 1) { var a; return a; }", js_module, "a");
    assert_eq!(actual, [vec![1, 2]]);

    let actual = symbol_links("function f(a = 1, read = () => a) { var a; a = 2; return read(); }", js_module, "a");
    assert_eq!(actual, [vec![1, 3]]);

    let actual = symbol_links("function f(a = 1) { try {} catch (a) { var a; } return a; }", js_module, "a");
    assert_eq!(actual, [vec![1, 2, 3]]);

    let actual = symbol_links("function f(a) { var a; return a; }", js_module, "a");
    assert_eq!(actual, [vec![1]]);

    let actual = symbol_links("function f(a = 1) { function g() { var a; } }", js_module, "a");
    assert_eq!(actual, [vec![1], vec![3]]);

    let actual = symbol_links("function f(a = 1) { var a; return a; }", ts_module, "a");
    assert_eq!(actual, [vec![2, 3]]);

    let actual = symbol_links("function f(a = 1, read = () => a) { var a; a = 2; return read(); }", ts_module, "a");
    assert_eq!(actual, [vec![2, 4]]);

    let actual = symbol_links("function f(a = 1) { try {} catch (a) { var a; } return a; }", ts_module, "a");
    assert_eq!(actual, [vec![2, 3, 4]]);

    let actual = symbol_links("function f(a) { var a; return a; }", ts_module, "a");
    assert_eq!(actual, [vec![2]]);

    let actual = symbol_links("function f(a = 1) { function g() { var a; } }", ts_module, "a");
    assert_eq!(actual, [vec![2], vec![4]]);

    let actual = symbol_links("function f(a = 1) { type a = number; var a; }", ts_module, "a");
    assert_eq!(actual, [vec![2, 3]]);

    let actual = symbol_links("function f(a = 1) { var a; function a() {} return a; }", js_module, "a");
    assert_eq!(actual, [vec![1], vec![2]]);

    let actual = symbol_links("function f(a = 1) { function a() {} var a; return a; }", js_module, "a");
    assert_eq!(actual, [vec![1], vec![2]]);
}
