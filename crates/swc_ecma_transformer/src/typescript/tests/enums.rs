use super::*;
use swc_next_ecma_ast::{Ast, FunctionBody, Program, Stmt, SubRange, VisitMut, VisitMutWith};

fn transformed_reversed_code(source: &str) -> String {
    struct ReverseBodies<'visit, 'ast>(&'visit mut Ast<'ast>);

    impl ReverseBodies<'_, '_> {
        fn reverse(&mut self, body: SubRange<Stmt>) -> SubRange<Stmt> {
            let mut statements: Vec<_> = body
                .iter()
                .map(|id| self.0.get_node_in_sub_range(id))
                .collect();
            statements.reverse();
            self.0.add_sub_range(statements)
        }
    }

    impl<'ast> VisitMut<'ast> for ReverseBodies<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
            self.0
        }

        fn visit_mut_program(&mut self, program: Program) -> Option<Program> {
            let body = self.reverse(program.body(self.0));
            program.set_body(self.0, body);
            program.visit_mut_children_with(self)
        }

        fn visit_mut_function_body(&mut self, function: FunctionBody) -> Option<FunctionBody> {
            let body = self.reverse(function.body(self.0));
            function.set_body(self.0, body);
            function.visit_mut_children_with(self)
        }
    }

    let allocator = Allocator::new();
    let options = ParserOptions {
        lang: Lang::Ts,
        ..ParserOptions::default()
    };
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    ast.root_program()
        .visit_mut_with(&mut ReverseBodies(&mut ast));
    Normalizer::normalize_runtime(ast.root_program(), &mut ast);
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    transform(
        ast.root_program(),
        &mut ast,
        &semantic,
        TypeScriptOptions::default(),
    );
    swc_next_ecma_codegen::print(&allocator, &ast, Default::default())
        .code
        .to_owned()
}

#[test]
fn enum_edited_order() {
    let code = transformed_reversed_code("const v=1; enum E { A=v }");
    assert!(code.contains("E[\"A\"] = v"));

    let code = transformed_reversed_code("enum E { A=v } const v=1;");
    assert!(code.contains("E[\"A\"] = 1"));

    let code = transformed_reversed_code("function f(){ const v=1; enum E { A=v } }");
    assert!(code.contains("E[\"A\"] = v"));

    let code = transformed_reversed_code("const v=1; function f(){ enum E { A=v } }");
    assert!(code.contains("E[\"A\"] = 1"));
}

#[test]
fn enum_container_bindings() {
    let code =
        transformed_code("enum E { A = 1, B = (() => { const E = { A: 42 }; return A; })() }");
    assert!(code.contains("function(E2)"));
    assert!(code.contains("return E2.A;"));
    assert!(code.contains("E2[E2[\"B\"]"));
    assert!(code.contains("E || (E = {})"));

    let code =
        transformed_code("namespace N { export enum E { A = 1, B = ((E: number) => A)(42) } }");
    assert!(code.contains("function(E2)"));
    assert!(code.contains("=> E2.A"));
    assert!(code.contains("E = N.E || (N.E = {})"));
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
fn enum_namespace_constants() {
    let code = transformed_code("namespace N { export const x=42; } enum E { A=N.x, B }");
    assert!(code.contains("E[\"A\"] = 42"));
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code(
        "namespace N { export const x=40; export const y=x+2; } enum E { A=N.y, B }",
    );
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code(
        "namespace N { export namespace M { export const x=42; } } import Alias=N.M; enum E { A=Alias.x, B }",
    );
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code("namespace N { export const x:number=42; } enum E { A=N.x }");
    assert!(code.contains("E[\"A\"] = N.x"));

    let code = transformed_code("namespace N { export const x=42; } enum E { A=N[\"x\"] }");
    assert!(code.contains("E[\"A\"] = N[\"x\"]"));
}

#[test]
fn enum_alias_constants() {
    let code =
        transformed_code("namespace N { export const x=42; } import X=N.x; enum E { A=X, B }");
    assert!(code.contains("E[\"A\"] = 42"));
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code(
        "namespace N { export const x=42; export import y=N.x; export import z=N.y; } enum E { A=N.z, B }",
    );
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code(
        "namespace N { export const x=42; } import X=N.x; const y=X; enum E { A=y, B }",
    );
    assert!(code.contains("E[\"B\"] = 43"));

    let code = transformed_code("enum E { A=42 } import X=E.A; enum F { A=X, B }");
    assert!(code.contains("F[\"B\"] = 43"));

    let code =
        transformed_code("namespace N { export const x:number=42; } import X=N.x; enum E { A=X }");
    assert!(code.contains("E[\"A\"] = X"));

    let code = transformed_code("import X=N.x; enum E { A=X } namespace N { export const x=42; }");
    assert!(code.contains("E[\"A\"] = X"));
}

#[test]
fn enum_optional_constants() {
    let code = transformed_code("enum E { A=42 } enum F { A=E?.A, B }");
    assert!(code.contains("F[\"A\"] = 42"));
    assert!(code.contains("F[\"B\"] = 43"));

    let code = transformed_code("enum E { A=42 } enum F { A=E?.[\"A\"], B }");
    assert!(code.contains("F[\"B\"] = 43"));

    let code = transformed_code("namespace N { export enum E { A=42 } } enum F { A=N?.E.A, B }");
    assert!(code.contains("F[\"B\"] = 43"));

    let code = transformed_code("namespace N { export const x=42; } enum E { A=N?.x, B }");
    assert!(code.contains("E[\"B\"] = 43"));

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
    let source = "namespace N { export const enum E { X = 1 } } export import A = N.E;";
    let code = transformed_code(source);
    assert!(code.contains("export var A = N.E"));
    assert!(code.contains("function(N)"));

    let preserve = TypeScriptOptions {
        preserve_const_enums: true,
        ..TypeScriptOptions::default()
    };
    let code = transformed_code_with_options(source, preserve.clone());
    assert!(code.contains("export var A = N.E"));

    let verbatim = TypeScriptOptions {
        verbatim_module_syntax: true,
        ..TypeScriptOptions::default()
    };
    let code = transformed_code_with_options(source, verbatim.clone());
    assert!(code.contains("export var A = N.E"));

    let code = transformed_code(
        "namespace N { export const enum E { X = 1 } } import A = N.E; export const x = A.X;",
    );
    assert!(code.contains("export const x = 1"));
    assert!(!code.contains("var A = N.E"));

    let code =
        transformed_code("namespace N { export const enum E { X = 1 } } export const x = N.E.X;");
    assert!(code.contains("export const x = 1"));
    assert!(code.contains("function(N)"));

    let code =
        transformed_code("namespace N { export const enum E { X = 1 } } export import A = N;");
    assert!(code.contains("export var A = N"));

    let code = transformed_code("namespace N { export enum E { X = 1 } } export import A = N.E;");
    assert!(code.contains("export var A = N.E"));
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
    let source =
        "declare namespace N { const enum E { X = 1 } } import A = N.E; export const x = A.X;";
    let code = transformed_code(source);
    assert!(code.contains("export const x = 1"));
    assert!(!code.contains("N.E"));

    let code = transformed_code_with_options(source, preserve.clone());
    assert!(code.contains("export const x = 1"));
    assert!(!code.contains("N.E"));

    let code = transformed_code(
        "namespace N { export const enum E { X = 1 } } import A = N.E; import B = A; export const x = B.X;",
    );
    assert!(code.contains("export const x = 1"));
    assert!(!code.contains("var A"));
    assert!(!code.contains("var B"));
    let code = transformed_code(
        "declare namespace N { const enum E { X = 1 } } export const x = N[\"E\"].X;",
    );
    assert!(code.contains("export const x = 1"));
    assert!(!code.contains("N["));

    let code = transformed_code(
        "namespace N { export const enum E { X = 1 } } import A = N.E; const enum F { Y = A.X + 1 } export const x = F.Y;",
    );
    assert!(code.contains("export const x = 2"));
    assert!(!code.contains("function(F)"));
    assert!(!code.contains("var A"));
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

    let source = "namespace N { const enum E { A = 1 }\nconst enum E { B = 2 }\nexport const value = E.A + E.B; } console.log(N.value);";
    let code = transformed_code(source);
    assert!(code.contains("N.value = 1 + 2"));
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

    let source =
        "const enum E { A = 1 }\nnamespace E { export const B = 2; } console.log(E.A, E.B);";
    let code = transformed_code(source);
    assert!(code.contains("E.B = 2"));
    assert!(code.contains("E[\"A\"] = 1"));

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
fn const_enum_namespace_aliases() {
    let code = transformed_code("namespace N { const enum E { X = 1 } } export import A = N;");
    assert!(code.contains("function(N)"));
    assert!(code.contains("export var A = N"));
    assert!(!code.contains("function(E)"));

    let code = transformed_code(
        "namespace N { export namespace M { const enum E { X = 1 } } } export import A = N.M;",
    );
    assert!(code.contains("N.M"));
    assert!(code.contains("function(M)"));
    assert!(!code.contains("function(E)"));

    let code = transformed_code(
        "namespace N.M { const enum E { X = 1 } } import Root = N; export import A = Root.M;",
    );
    assert!(code.contains("function(M)"));
    assert!(code.contains("var Root = N"));
    assert!(!code.contains("function(E)"));
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
