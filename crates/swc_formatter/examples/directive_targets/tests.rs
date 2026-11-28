use super::*;

fn check(input: &str, candidate: &str, lang: Lang) -> ResultCounts {
    let options = Options {
        lang,
        comments: CommentMode::Flat,
        preserve_parens: true,
        ..Default::default()
    };
    let mut counts = ResultCounts::default();
    compare(
        "test",
        &inspect(input, options).unwrap(),
        &inspect(candidate, options).unwrap(),
        false,
        &mut counts,
    );
    counts
}

#[test]
fn optimizer_targets() {
    let comma = check(
        "sink(1, /*#__PURE__*/ receiver.method());",
        "sink(1 /*#__PURE__*/, receiver.method());",
        Lang::Js,
    );
    assert_eq!(comma.changed, 1);

    let missing = check("sink(/*#__PURE__*/ first());", "other();", Lang::Js);
    assert_eq!(missing.status(), 1);

    let repeated = check(
        "f(); /*@__PURE__*/ f();",
        "/*@__PURE__*/ f(); f();",
        Lang::Js,
    );
    assert_eq!(repeated.changed, 1);

    let constructor = check(
        "/*#__PURE__*/ new A(f());",
        "new A(/*#__PURE__*/ f());",
        Lang::Js,
    );
    assert_eq!(constructor.changed, 1);

    let chain = check("/*@__PURE__*/ f()();", "/*@__PURE__*/ f()();", Lang::Js);
    assert_eq!(chain.unverified, 1);

    let line = check("// @__PURE__\nf();", "// @__PURE__\nf();", Lang::Js);
    assert_eq!(line.unverified, 1);

    let activation = check("/*@__PURE__*/; f();", "; /*@__PURE__*/ f();", Lang::Js);
    assert_eq!(activation.activated, 1);
    assert_eq!(activation.status(), 0);
}

#[test]
fn cast_targets() {
    let moved = check(
        "use(/** @type {string} */ (x), (x));",
        "use((x), /** @type {string} */ (x));",
        Lang::Js,
    );
    assert_eq!(moved.changed, 1);

    let removed = check(
        "use(/** @type {string} */ (value));",
        "use(/** @type {string} */ value);",
        Lang::Js,
    );
    assert_eq!(removed.changed, 1);

    let retained = check(
        "use(/** @type {string} */ ((value)));",
        "use(/** @type {string} */ (value));",
        Lang::Js,
    );
    assert_eq!(retained.preserved, 1);

    let nested = check(
        "/*@__PURE__*/ f(g(x), y);",
        "/*@__PURE__*/ f(g(x, y));",
        Lang::Js,
    );
    assert_eq!(nested.unverified, 1);
}

#[test]
fn diagnostic_lines() {
    let reindented = check(
        "/* note\n * @ts-ignore */\nf(a);",
        "/* note\n   * @ts-ignore */\nf(a);",
        Lang::Ts,
    );
    assert_eq!(reindented.preserved, 1);

    let code_to_opener = check(
        "// @ts-ignore\n(f(a));",
        "// @ts-ignore\n(\nf(a));",
        Lang::Ts,
    );
    assert_eq!(code_to_opener.status(), 1);

    let opener_to_code = check(
        "// @ts-ignore\n(\nf(a));",
        "// @ts-ignore\n(f(a));",
        Lang::Ts,
    );
    assert_eq!(opener_to_code.status(), 2);

    let array_closer = check(
        "[\n// @ts-ignore\na + b];",
        "[\n// @ts-ignore\na + b,\n];",
        Lang::Ts,
    );
    assert_eq!(array_closer.status(), 2);

    let same_opener = check(
        "// @ts-ignore\n(\nf(a));",
        "// @ts-ignore\n(\n  f(a));",
        Lang::Ts,
    );
    assert_eq!(same_opener.status(), 0);

    let narrowed = check(
        "// @ts-ignore\nf(a, b);",
        "// @ts-ignore\nf(a,\n b);",
        Lang::Ts,
    );
    assert_eq!(narrowed.changed, 1);

    let widened = check(
        "// @ts-expect-error\nf(a);\ng(b);",
        "// @ts-expect-error\nf(a); g(b);",
        Lang::Ts,
    );
    assert_eq!(widened.changed, 1);

    let blank = check(
        "// @ts-ignore\r\n// detail\r\n\r\nf(a);",
        "// @ts-ignore\nf(a);",
        Lang::Ts,
    );
    assert_eq!(blank.preserved, 1);

    let block = check(
        "// @ts-ignore\n/* note */\nf(a);",
        "/* note */\n// @ts-ignore\nf(a);",
        Lang::Ts,
    );
    assert_eq!(block.activated, 1);

    let changed_delimiters = check("// @ts-ignore\nf(a)", "// @ts-ignore\nf(a);", Lang::Ts);
    assert_eq!(changed_delimiters.unverified, 1);

    let shared_line = check(
        "/* @ts-ignore */ // @ts-expect-error\nf(a);",
        "/* @ts-ignore */ // @ts-expect-error\nf(a);",
        Lang::Ts,
    );
    assert_eq!(shared_line.unverified, 2);
}

#[test]
fn directive_prologues() {
    let quotes = check("'use strict';", "\"use strict\";", Lang::Js);
    assert_eq!(quotes.preserved, 1);

    let activated = check("('use strict');", "'use strict';", Lang::Js);
    assert_eq!(activated.changed, 1);

    let escaped = check("'\\x75se strict';", "'use strict';", Lang::Js);
    assert_eq!(escaped.changed, 1);

    let scopes = check(
        "function a(){'custom'} function b(){('custom')}",
        "function a(){('custom')} function b(){'custom'}",
        Lang::Js,
    );
    assert_eq!(scopes.changed, 2);
}
