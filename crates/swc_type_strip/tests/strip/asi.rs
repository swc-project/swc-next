use super::*;

#[test]
fn statement_boundaries() {
    assert_strip("foo\ntype T = string;\n(1);", "foo\n;               \n(1);");

    assert_strip(
        "foo;\ntype T = string;\n(1);",
        "foo;\n                \n(1);",
    );

    assert_strip(
        "switch (value) { case 0: foo\ntype T = string;\n(next)(); }",
        "switch (value) { case 0: foo\n;               \n(next)(); }",
    );

    assert_strip(
        "switch (value) { case 0: foo\ntype T = string;\n[next](); }",
        "switch (value) { case 0: foo\n;               \n[next](); }",
    );

    assert_strip(
        "switch (value) { default: foo\ntype T = string;\n`text`; }",
        "switch (value) { default: foo\n;               \n`text`; }",
    );

    assert_strip(
        "switch (value) { default: foo\ntype T = string;\n`text ${next}`; }",
        "switch (value) { default: foo\n;               \n`text ${next}`; }",
    );

    assert_strip(
        "switch (value) { case 0: if (ok) run()\ntype T = string;\n(next)(); }",
        "switch (value) { case 0: if (ok) run()\n;               \n(next)(); }",
    );

    assert_strip("value as string\n(next)();", "value;         \n(next)();");

    assert_strip(
        "value as Type/* comment */\n(next)();",
        "value;       /* comment */\n(next)();",
    );

    assert_strip(
        "value satisfies Type\n[next]();",
        "value;              \n[next]();",
    );

    assert_strip("0 as const\n(() => {})();", "0;        \n(() => {})();");

    assert_strip("0 as const\n[1];", "0;        \n[1];");

    assert_strip("0 as const\n`text`;", "0;        \n`text`;");

    assert_strip(
        "0 as const\n`text ${value}`;",
        "0;        \n`text ${value}`;",
    );
}

#[test]
fn statement_bodies() {
    assert_strip(
        "if (ok)\n    type T = string;\nrun();",
        "if (ok)\n    ;               \nrun();",
    );

    assert_strip(
        "if (ok) type T = string;\nelse run();",
        "if (ok) ;               \nelse run();",
    );

    assert_strip(
        "if (ok) run();\nelse type T = string;\nafter();",
        "if (ok) run();\nelse ;               \nafter();",
    );

    assert_strip(
        "while (ok)\n    interface I {}\nafter();",
        "while (ok)\n    ;             \nafter();",
    );

    assert_strip(
        "for (;;)\n    type T = string;\nafter();",
        "for (;;)\n    ;               \nafter();",
    );

    assert_strip(
        "label: interface I {}\nafter();",
        "label: ;             \nafter();",
    );

    assert_strip(
        "do\n    interface I {}\nwhile (ok);",
        "do\n    ;             \nwhile (ok);",
    );

    assert_strip(
        "if (outer) if (inner) type T = string;\nelse run();\nafter();",
        "if (outer) if (inner) ;               \nelse run();\nafter();",
    );

    assert_strip(
        "if (outer) if (inner) run(); else type T = string;\nelse other();\nafter();",
        "if (outer) if (inner) run(); else ;               \nelse other();\nafter();",
    );

    assert_strip(
        "for (const key in object) type T = string;\nafter();",
        "for (const key in object) ;               \nafter();",
    );

    assert_strip(
        "for (const value of values) type T = string;\nafter();",
        "for (const value of values) ;               \nafter();",
    );

    assert_strip(
        "async function f() { for await (const value of values) type T = string;\nafter(); }",
        "async function f() { for await (const value of values) ;               \nafter(); }",
    );

    assert_strip(
        "with (object) type T = string;\nafter();",
        "with (object) ;               \nafter();",
    );
}

#[test]
fn directive_prologues() {
    assert_strip(
        "interface A {}\ntype B = string;\n'use strict';",
        ";             \n                \n'use strict';",
    );

    assert_strip(
        "type T = string;\n'use strict';",
        ";               \n'use strict';",
    );

    assert_strip(
        "\"custom\"; type T = number; \"use strict\";",
        "\"custom\"; ;                \"use strict\";",
    );

    assert_strip(
        "'custom';\n'another';\ntype T = number;\ninterface I {}\n'use strict';",
        "'custom';\n'another';\n;               \n              \n'use strict';",
    );

    assert_strip(
        "type T = string;\n'use strict';\nexport {};",
        ";               \n'use strict';\nexport {};",
    );

    assert_strip(
        "'use client';\ntype T = number;\n'use server';\nexport {};",
        "'use client';\n;               \n'use server';\nexport {};",
    );

    assert_strip(
        "function f() {\ntype T = string;\n'use strict';\n}",
        "function f() {\n;               \n'use strict';\n}",
    );

    assert_strip(
        "function f() { \"custom\"; type T = number; \"use strict\"; }",
        "function f() { \"custom\"; ;                \"use strict\"; }",
    );

    assert_strip(
        "const f = function () { 'custom'; interface I {} 'use strict'; };",
        "const f = function () { 'custom'; ;              'use strict'; };",
    );

    assert_strip(
        "const f = () => { 'custom'; 'another'; type T = number; 'use strict'; };",
        "const f = () => { 'custom'; 'another'; ;                'use strict'; };",
    );

    assert_strip(
        "class C { method() { 'custom'; type T = number; 'use strict'; } }",
        "class C { method() { 'custom'; ;                'use strict'; } }",
    );

    assert_strip(
        "function f() {\n'custom';\nrun();\ntype T = number;\n'use strict';\n}",
        "function f() {\n'custom';\nrun();\n                \n'use strict';\n}",
    );

    assert_strip(
        "function f() {\n'custom';\n;\ntype T = number;\n'use strict';\n}",
        "function f() {\n'custom';\n;\n                \n'use strict';\n}",
    );

    assert_strip(
        "function f() {\n('custom');\ntype T = number;\n'use strict';\n}",
        "function f() {\n('custom');\n                \n'use strict';\n}",
    );

    assert_strip(
        "function f() {\n{\n'custom';\ntype T = number;\n'use strict';\n}\n}",
        "function f() {\n{\n'custom';\n                \n'use strict';\n}\n}",
    );
}

#[test]
fn class_member_boundaries() {
    assert_strip(
        "class C { first = 1\npublic ['second'] = 2; }",
        "class C { first = 1\n;      ['second'] = 2; }",
    );

    assert_strip(
        "class C { value = input as Type\n['next']() {} }",
        "class C { value = input;       \n['next']() {} }",
    );
}

#[test]
fn expression_continuations() {
    assert_strip("value as Type\n+ next;", "value        \n+ next;");

    assert_strip("const value = input!\n[0];", "const value = input \n[0];");

    assert_strip(
        "const value = (input as Fn)\n(0);",
        "const value = (input      )\n(0);",
    );
}
