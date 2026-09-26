use super::*;

#[test]
fn enum_container_bindings() {
    let code =
        transformed_code("enum E { A = 1, B = (() => { const E = { A: 42 }; return A; })() }");
    assert!(code.contains("function(E2)"));
    assert!(code.contains("return E2.A;"));
    assert!(code.contains("E2[E2[\"B\"]"));
    assert!(code.contains("E || (E = {})"));
}

#[test]
fn enum_builtin_constants() {
    let code = transformed_code("enum E { A=Infinity, B=NaN }");
    assert!(code.contains("E[\"A\"] = 1 / 0"));
    assert!(code.contains("E[\"B\"] = 0 / 0"));

    let code = transformed_code(
        "import * as remote from 'pkg'; import Infinity=remote.value; enum E { A=Infinity }",
    );
    assert!(code.contains("E[\"A\"] = Infinity"));

    let code = transformed_code(
        "import * as remote from 'pkg'; import NaN=remote.value; enum E { A=NaN }",
    );
    assert!(code.contains("E[\"A\"] = NaN"));

    let code = transformed_code("function f(Infinity:number){ enum E { A=Infinity } return E.A; }");
    assert!(code.contains("E[\"A\"] = Infinity"));
}

#[test]
fn enum_method_name_constants() {
    let code =
        transformed_code("class C { [(()=>{ enum E { A=x,B } return E.B; })()](){} } const x=42;");
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code(
        "class C { get [(()=>{ enum E { A=x,B } return E.B; })()](){return 0;} } const x=42;",
    );
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code(
        "const obj={ [(()=>{ enum E { A=x,B } return E.B; })()](){} }; const x=42;",
    );
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code(
        "const obj={ get [(()=>{ enum E { A=x,B } return E.B; })()](){return 0;} }; const x=42;",
    );
    assert!(code.contains("E[\"B\"] = 43"));

    let code =
        transformed_code("const obj={ [(()=>{ enum E { A=x } return E.A; })()]:0 }; const x=42;");
    assert!(code.contains("E[\"A\"] = x"));

    let code = transformed_code(
        "class C { static value=(()=>{ enum E { A=x } return E.A; })(); } const x=42;",
    );
    assert!(code.contains("E[\"A\"] = x"));

    let code = transformed_code("class C { static { enum E { A=x } } } const x=42;");
    assert!(code.contains("E[\"A\"] = x"));
}

#[test]
fn enum_numeric_constants() {
    let code = transformed_code("enum E { A = 1 / -0 }");
    assert!(code.contains("E[\"A\"] = -1 / 0"));

    let code = transformed_code("enum E { A = 1 / (-1 % 1) }");
    assert!(code.contains("E[\"A\"] = -1 / 0"));

    let code = transformed_code("enum E { A = -0, B = 1 / A }");
    assert!(code.contains("E[\"A\"] = 0"));
    assert!(code.contains("E[\"B\"] = -1 / 0"));

    let code = transformed_code("const enum E { A = -0 } export const value = E.A;");
    assert!(code.contains("export const value = 0"));
}

#[test]
fn enum_optional_constants() {
    let code = transformed_code("enum E { A=42 } enum F { A=E?.A, B }");
    assert!(code.contains("F[\"A\"] = 42"));
    assert!(code.contains("F[\"B\"] = 43"));

    let code = transformed_code("enum E { A=42 } enum F { A=E?.[\"A\"], B }");
    assert!(code.contains("F[\"B\"] = 43"));

    let code = transformed_code("const enum E { A=42 } export const value=E?.A;");
    assert!(code.contains("function(E)"));
    assert!(code.contains("export const value = E?.A"));
}

#[test]
fn enum_deferred_constants() {
    let code = transformed_code("function f(){ enum E { A=x, B } return E.B; } const x=42;");
    assert!(code.contains("E[\"A\"] = 42"));
    assert!(code.contains("E[\"B\"] = 43"));

    let code =
        transformed_code("function f(){ enum F { A=E.B, B } return F.B; } enum E { A=41, B }");
    assert!(code.contains("F[\"A\"] = 42"));
    assert!(code.contains("F[\"B\"] = 43"));

    let code =
        transformed_code("function f(){ const n=x; enum E { A=n, B } return E.B; } const x=42;");
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code(
        "class C { value=(()=>{ enum E { A=x, B } return E.B; })(); } const x=42;",
    );
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code("(()=>{ enum E { A=x, B } return E.B; })(); const x=42;");
    assert!(code.contains("E[\"A\"] = x"));
    assert!(code.contains("E[\"B\"] = void 0"));

    let code = transformed_code("function f(){ enum E { A=x, B } const x=42; return E.B; }");
    assert!(code.contains("E[\"A\"] = x"));
    assert!(code.contains("E[\"B\"] = void 0"));
}

#[test]
fn ambient_enum_constants() {
    let code = transformed_code("declare enum E { A } enum F { X = E.A }");
    assert!(code.contains("F[\"X\"] = E.A"));

    let code = transformed_code("declare namespace N { enum E { A } } enum F { X = N.E.A }");
    assert!(code.contains("F[\"X\"] = N.E.A"));

    let code = transformed_code("declare enum E { A = 3, B } enum F { X = E.A, Y = E.B }");
    assert!(code.contains("F[\"X\"] = 3"));
    assert!(code.contains("F[\"Y\"] = E.B"));

    let code = transformed_code("declare const enum E { A, B } enum F { X = E.B }");
    assert!(code.contains("F[\"X\"] = 1"));

    let code = transformed_code("declare enum E { A } enum E { B } enum F { X = E.A, Y = E.B }");
    assert!(code.contains("F[\"X\"] = E.A"));
    assert!(code.contains("F[\"Y\"] = 0"));
}

#[test]
fn const_enum_emission() {
    let preserve = TypeScriptOptions {
        preserve_const_enums: true,
        ..TypeScriptOptions::default()
    };

    let verbatim = TypeScriptOptions {
        verbatim_module_syntax: true,
        ..TypeScriptOptions::default()
    };

    let source = "const enum E { X = 1 } export const x = E.X;";
    let code = transformed_code(source);
    assert!(code.contains("export const x = 1"));
    assert!(!code.contains("function(E)"));

    let code = transformed_code_with_options(source, preserve.clone());
    assert!(code.contains("export const x = 1"));
    assert!(code.contains("function(E)"));

    let code = transformed_code_with_options(source, verbatim.clone());
    assert!(code.contains("export const x = E.X"));
    assert!(code.contains("function(E)"));

    let code = transformed_code("export const enum E { X = 1 } export const x = E.X;");
    assert!(code.contains("export const x = 1"));
    assert!(code.contains("function(E)"));
}

#[test]
fn merged_const_enums() {
    let source = "const enum E { A = 1 }\nconst enum E { B = 2 }\nconsole.log(E.A, E.B);";
    let code = transformed_code(source);
    assert!(code.contains("console.log(1, 2)"));
    assert!(!code.contains("var E"));

    let source =
        "const enum E { A = 1 }\nconst enum E { B = E.A + 1, C }\nconsole.log(E.A, E.B, E.C);";
    let code = transformed_code(source);
    assert!(code.contains("console.log(1, 2, 3)"));
    assert!(!code.contains("var E"));

    let source = "const enum E { A = 'a' }\nconst enum E { B = 'b' }\nconst enum E { C = E.A + E.B }\nconsole.log(E.C);";
    let code = transformed_code(source);
    assert!(code.contains("console.log(\"ab\")"));
    assert!(!code.contains("var E"));
}

#[test]
fn merged_const_enum_preservation() {
    let source = "const enum E { A = 1 }\nconst enum E { B = bump() }\nconsole.log(E.A);";
    let code = transformed_code(source);
    assert!(code.contains("E[\"B\"] = bump()"));
    assert!(code.contains("var E"));

    let source = "const enum E { A = bump() }\nconst enum E { B = 2 }\nconsole.log(E.B);";
    let code = transformed_code(source);
    assert!(code.contains("E[\"A\"] = bump()"));
    assert!(code.contains("var E"));

    let source = "const enum E { A = bump() }\nconst enum E { A = 1 }\nconsole.log(E.A);";
    let code = transformed_code(source);
    assert!(code.contains("var E"));

    let source = "const enum E { A = 1 }\nconst enum E { B = bump() }\nconst enum E { C = 3 }\nconsole.log(E.A, E.C);";
    let code = transformed_code(source);
    assert!(code.contains("E[\"B\"] = bump()"));
    assert!(code.contains("var E"));

    let source = "const enum E { A = 1 }\nconst enum E { B = 2 }\nconsole.log(E);";
    let code = transformed_code(source);
    assert!(code.contains("console.log(E)"));
    assert!(code.contains("var E"));

    let source = "const enum E { A = 1 }\nconst enum E { B = 2 }\nE.B = 3;";
    let code = transformed_code(source);
    assert!(code.contains("E.B = 3"));
    assert!(code.contains("var E"));

    let source = "const enum E { A = 1 }\nenum E { B = 2 }\nconsole.log(E.A, E.B);";
    let code = transformed_code(source);
    assert!(code.contains("var E"));

    let source = "const enum E { A = 1 }\ndeclare const enum E { B = 2 }\nconsole.log(E.A);";
    let code = transformed_code(source);
    assert!(code.contains("var E"));

    let source =
        "export const enum E { A = 1 }\nexport const enum E { B = 2 }\nconsole.log(E.A, E.B);";
    let code = transformed_code(source);
    assert!(code.contains("console.log(1, 2)"));
    assert!(code.contains("function(E)"));

    let source = "const enum E { A = 1 }\nconst enum E { B = 2 }\nconsole.log(E.A, E.B);";
    let preserve = TypeScriptOptions {
        preserve_const_enums: true,
        ..TypeScriptOptions::default()
    };
    let code = transformed_code_with_options(source, preserve);
    assert!(code.contains("console.log(1, 2)"));
    assert!(code.contains("var E"));

    let verbatim = TypeScriptOptions {
        verbatim_module_syntax: true,
        ..TypeScriptOptions::default()
    };
    let code = transformed_code_with_options(source, verbatim);
    assert!(code.contains("console.log(E.A, E.B)"));
    assert!(code.contains("var E"));
}

#[test]
fn enum_string_expressions() {
    let code = transformed_code(
        "import { suffix } from 'dep'; export enum E { A='fixed', B=`${suffix}` }",
    );
    assert!(code.contains("E[\"B\"] = `${suffix}`;"));
    assert!(!code.contains("E[E[\"B\"]"));

    let code = transformed_code(
        "import { suffix } from 'dep'; export enum E { A='fixed', B=('' + suffix) }",
    );
    assert!(code.contains("E[\"B\"] = \"\" + suffix;"));
    assert!(!code.contains("E[E[\"B\"]"));

    let code = transformed_code("import { suffix } from 'dep'; enum E { A=suffix + '' }");
    assert!(!code.contains("E[E[\"A\"]"));

    let code = transformed_code(
        "import { suffix } from 'dep'; const value = `${suffix}`; enum E { A='fixed', B=value, C=B }",
    );
    assert!(!code.contains("E[E["));
    assert!(code.contains("E[\"B\"] = value;"));

    let code = transformed_code("import { suffix } from 'dep'; enum E { A=(`${suffix}` as any) }");
    assert!(code.contains("E[E[\"A\"]"));

    let code = transformed_code("import { suffix } from 'dep'; enum E { A=(suffix as any) + 1 }");
    assert!(code.contains("E[E[\"A\"]"));

    let code = transformed_code(
        "import { suffix } from 'dep'; const enum E { A=`${suffix}` } console.log(E.A);",
    );
    assert!(code.contains("console.log(E.A)"));
    assert!(code.contains("E[\"A\"] = `${suffix}`;"));
}

#[test]
fn assignment_target_reads() {
    let code = transformed_code("const enum E { X = 0 } const a = []; a[E.X] = 1;");
    assert!(code.contains("a[0] = 1"));
    assert!(!code.contains("var E"));

    let code = transformed_code("const enum E { X = 0 } const a = []; delete a[E.X];");
    assert!(code.contains("delete a[0]"));
    assert!(!code.contains("var E"));

    let code = transformed_code("const enum E { X = 0 } E.X = 1;");
    assert!(code.contains("E.X = 1"));

    let code = transformed_code("const enum E { X = 0 } delete E.X;");
    assert!(code.contains("delete E.X"));

    let code = transformed_code("const enum E { X = 0 } const a = []; a[E.X]++;");
    assert!(code.contains("a[0]++"));
}
