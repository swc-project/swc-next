use super::*;

#[track_caller]
fn assert_import_target_access(source: &str, name: &[u8], read: bool) {
    assert_alias_reference_access(source, name, ReferenceSpace::Namespace, read);
}

#[track_caller]
fn assert_alias_reference_access(source: &str, name: &[u8], space: ReferenceSpace, read: bool) {
    let allocator = Allocator::new();
    let check = |options| {
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let target = symbol_named(&ast, &semantic, name);
        let reference = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name)
            .unwrap()
            .reference;
        assert_eq!(reference.symbol, Some(target));
        assert_eq!(reference.flags.space, space);
        assert_eq!(reference.flags.is_read(), read);
        assert!(!reference.flags.is_write());
        assert_eq!(semantic.lookup(reference.scope, name, space), Some(target));
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    });
}

#[test]
fn internal_aliases_resolve_across_chains() {
    use std::fmt::Write;

    let mut source = String::new();
    for index in 0..20_000 {
        writeln!(source, "import A{index} = A{};", index + 1).unwrap();
    }
    source.push_str("namespace A20000 { export interface T {} }\ntype Result = A0.T;");
    assert_alias_reference_access(&source, b"A0", ReferenceSpace::Namespace, false);

    let mut source = String::from("namespace N {\n");
    for index in 0..20_000 {
        writeln!(source, "export import A{index} = N.A{};", index + 1).unwrap();
    }
    source.push_str("export namespace A20000 { export interface T {} } }\nimport Root = N.A0;\ntype Result = Root.T;");
    assert_alias_reference_access(&source, b"Root", ReferenceSpace::Namespace, false);
}

#[test]
fn import_target_access() {
    assert_import_target_access(
        "namespace N { export const x = 1 } import A = N; export const result = A.x",
        b"N",
        true,
    );

    assert_import_target_access(
        "namespace N { export namespace Q { export const x = 1 } } import A = N.Q; export const result = A.x",
        b"N",
        true,
    );

    assert_import_target_access(
        "namespace N { export interface T {} } import A = N; export type Result = A.T",
        b"N",
        false,
    );

    assert_import_target_access(
        "namespace N { export const x = 1; export interface T {} } import A = N.T; export type Result = A",
        b"N",
        false,
    );

    assert_import_target_access(
        "namespace N { export interface T {} } import B = N; import A = B; export type Result = A.T",
        b"B",
        false,
    );

    assert_import_target_access(
        "declare namespace N { const x: number } declare namespace M { import A = N; const result: typeof A.x }",
        b"N",
        false,
    );
}

#[test]
fn alias_exports() {
    assert_alias_reference_access(
        "namespace N { export interface T {} } import A = N.T; export { A }",
        b"A",
        ReferenceSpace::Any,
        false,
    );

    assert_alias_reference_access(
        "namespace N { export const x = 1 } import A = N.x; export { A }",
        b"A",
        ReferenceSpace::Any,
        true,
    );

    assert_alias_reference_access(
        "import * as remote from './dep.js'; import A = remote; export { A }",
        b"A",
        ReferenceSpace::Any,
        true,
    );

    assert_import_target_access(
        "import * as remote from './dep.js'; namespace N { export import Alias = remote }",
        b"remote",
        true,
    );

    assert_import_target_access(
        "import * as remote from './dep.js'; namespace N { import B = remote; export import Alias = B }",
        b"remote",
        true,
    );
    assert_alias_reference_access(
        "import * as remote from './dep.js'; namespace N { export import Alias = remote } import A = N.Alias; export { A }",
        b"A",
        ReferenceSpace::Any,
        true,
    );

    assert_alias_reference_access(
        "namespace N { export interface T {} export import B = N.T } import A = N.B; export { A }",
        b"A",
        ReferenceSpace::Any,
        false,
    );
}

#[track_caller]
fn assert_namespace_shadowing(source: &str, name: &[u8]) {
    let allocator = Allocator::new();
    let check = |options| {
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let namespace = symbol_named(&ast, &semantic, b"N");
        let member = semantic.ts_namespace_member(namespace, name).unwrap();
        let outer = semantic
            .iter_symbols()
            .find(|entry| {
                ast.get_wtf8(entry.symbol.name).as_bytes() == name
                    && !entry.symbol.flags.contains(SymbolFlags::EXPORTED)
            })
            .unwrap()
            .id;
        assert_ne!(member, outer);
        assert_eq!(reference_symbol(&ast, &semantic, name), Some(member));
        for entry in semantic.iter_references() {
            let reference = entry.reference;
            let name = ast.get_utf8(reference.name).as_bytes();
            assert!(reference.flags.is_read_only());
            assert_eq!(
                semantic.lookup(reference.scope, name, reference.flags.space),
                reference.symbol
            );
        }
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    });
}

#[test]
fn import_value_merging() {
    assert_alias_reference_access(
        "import type { T as X } from './dep.js'; const X = 1; export const result = X;",
        b"X",
        ReferenceSpace::Value,
        true,
    );

    assert_alias_reference_access(
        "import type { T as X } from './dep.js'; const X = 1; namespace N {} import A = N; export const result = X;",
        b"X",
        ReferenceSpace::Value,
        true,
    );

    assert_alias_reference_access(
        "import type { T as X } from './dep.js'; const X = 1; export { X };",
        b"X",
        ReferenceSpace::Any,
        true,
    );

    assert_alias_reference_access(
        "import type { T as X } from './dep.js'; const X = 1; namespace N {} import A = N; export { X };",
        b"X",
        ReferenceSpace::Any,
        true,
    );
}

#[test]
fn namespace_shadowing() {
    assert_namespace_shadowing(
        "const x = 0; namespace N { export const x = 1 } namespace N { export const y = x }",
        b"x",
    );

    assert_namespace_shadowing(
        "import * as remote from './dep.js'; const Alias = { value: 'outer' }; namespace N { export import Alias = remote } namespace N { export const selected = Alias.value }",
        b"Alias",
    );

    assert_namespace_shadowing(
        "import * as remote from './dep.js'; const Alias = { value: 'outer' }; namespace N { import B = remote; export import Alias = B } namespace N { export const selected = Alias.value }",
        b"Alias",
    );

    assert_namespace_shadowing(
        "import * as remote from './dep.js'; namespace A { export interface T {} } const B = { value: 'outer' }; namespace N { import A = remote; export import B = A; } namespace N { export const result = B.value; }",
        b"B",
    );

    assert_namespace_shadowing(
        "import * as remote from './dep.js'; namespace A { export interface T {} } const B = { value: 'outer' }; namespace N { import A = remote.Inner; export import B = A; } namespace N { export const result = B.value; }",
        b"B",
    );
}

#[test]
fn member_alias_visibility() {
    let allocator = Allocator::new();
    let source = "export {}; const A = 0; type A = string; namespace Target { export interface T {} } enum E { A = 42, B = A } declare namespace E { export import A = Target } namespace E { const value: 0 = A; type ValueType = A; type Qualified = A.T }";
    let check = |options| {
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let outer = semantic
            .iter_symbols()
            .find(|entry| {
                ast.get_wtf8(entry.symbol.name).as_bytes() == b"A"
                    && entry.symbol.flags.contains(SymbolFlags::CONST_VAR)
            })
            .unwrap()
            .id;
        let owner = symbol_named(&ast, &semantic, b"E");
        let member = semantic.ts_enum_member(owner, b"A").unwrap();
        assert_ne!(outer, member);
        assert_eq!(semantic.ts_namespace_member(owner, b"A"), Some(member));
        assert_eq!(
            reference_symbols(&ast, &semantic, b"A"),
            [Some(member), Some(outer), Some(outer), Some(member)]
        );
        for entry in semantic.iter_references() {
            let reference = entry.reference;
            if ast.get_utf8(reference.name).as_bytes() == b"A" {
                assert_eq!(
                    semantic.lookup(reference.scope, b"A", reference.flags.space),
                    reference.symbol
                );
            }
        }
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    });
}

#[track_caller]
fn assert_import_target(source: &str, name: Option<&[u8]>) {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let options = AnalyzeOptions {
            check_syntax,
            ..AnalyzeOptions::default()
        };
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let alias = semantic.binding(ScopeId::MODULE, b"A").unwrap();
        let target = semantic.ts_import_alias_target(alias);
        assert_eq!(
            target.map(|id| ast.get_wtf8(semantic.symbol(id).name).as_bytes()),
            name
        );
    }
}

#[test]
fn terminal_targets_follow_unique_aliases() {
    assert_import_target(
        "namespace N { export const enum E { X = 1 } } import A = N.E;",
        Some(b"E"),
    );

    assert_import_target(
        "namespace N { export const enum E { X = 1 } } import A = B; import B = N.E;",
        Some(b"E"),
    );

    assert_import_target(
        "namespace N { export interface T {} } import A = N.T;",
        Some(b"T"),
    );

    assert_import_target(
        "namespace N { export const enum E { X = 1 } } import A = N;",
        Some(b"N"),
    );

    assert_import_target("import * as N from 'pkg'; import A = N.E;", None);

    assert_import_target("import A = B; import B = A;", None);

    assert_import_target("class C { static x = 1; } import A = C.x;", None);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum InitializerReference {
    Absent,
    TypeOnly,
    Read,
}

#[track_caller]
fn assert_import_initializer(source: &str, expected: InitializerReference) {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let options = AnalyzeOptions {
            check_syntax,
            ..AnalyzeOptions::default()
        };
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let declaration = ast
            .root_program()
            .body(&ast)
            .iter()
            .find_map(|id| {
                let statement = ast.get_node_in_sub_range(id);
                let swc_next_ecma_ast::StmtData::Declaration(declaration) =
                    ast.stmt_data(statement)
                else {
                    return None;
                };
                let swc_next_ecma_ast::DeclData::TsImportEqualsDeclaration(declaration) =
                    ast.decl_data(declaration)
                else {
                    return None;
                };
                Some(declaration)
            })
            .unwrap();
        let reference = semantic.import_equals_reference(&ast, declaration);
        assert_eq!(
            reference.map_or(InitializerReference::Absent, |id| {
                if semantic.reference(id).flags.is_read() {
                    InitializerReference::Read
                } else {
                    InitializerReference::TypeOnly
                }
            }),
            expected
        );
    }
}

#[test]
fn initializer_references_distinguish_runtime_reads() {
    assert_import_initializer(
        "namespace N { export interface T {} } import A = N.T;",
        InitializerReference::TypeOnly,
    );

    assert_import_initializer(
        "namespace N { export const x = 1; } import A = N.x;",
        InitializerReference::Read,
    );

    assert_import_initializer(
        "import * as N from 'pkg'; import A = N.x;",
        InitializerReference::Read,
    );

    assert_import_initializer(
        "import type A = require('pkg');",
        InitializerReference::Absent,
    );

    assert_import_initializer(
        "namespace N { export const enum E { X = 1 } } import A = N.E;",
        InitializerReference::Read,
    );
}

#[track_caller]
fn assert_unrelated_alias_preserves_import(source: &str, declarations: &[Option<&str>]) {
    let ts_module = ts_module_options();
    assert_named_declaration_targets(source, ts_module, b"Item", declarations);
    let extended = format!(
        "{source} namespace Unrelated {{ export interface T {{}} }} import Unused = Unrelated;"
    );
    assert_named_declaration_targets(&extended, ts_module, b"Item", declarations);
}

#[test]
fn unrelated_aliases_preserve_external_import_lookup() {
    assert_unrelated_alias_preserves_import(
        "import type { Item } from './dep'; type A = Item;",
        &[Some("Item }")],
    );

    assert_unrelated_alias_preserves_import(
        "import { Item } from './dep'; type A = Item; const value = Item;",
        &[Some("Item }"), Some("Item }")],
    );

    assert_unrelated_alias_preserves_import(
        "import Item from './dep'; type A = typeof Item;",
        &[Some("Item from")],
    );

    assert_unrelated_alias_preserves_import(
        "import * as Item from './dep'; type A = typeof Item;",
        &[Some("Item from")],
    );

    assert_unrelated_alias_preserves_import(
        "import Item = require('./dep'); type A = typeof Item;",
        &[Some("Item = require")],
    );

    assert_uncertain_alias_targets(
        "import * as External from './dep'; import A = External.Item; type Result = A;",
        &["A = External.Item"],
    );
}

// These assertions describe lookup results, certainty, uses, and access;
// alias caches and graph traversal can change without changing the contract.
#[track_caller]
fn assert_import_root_access(source: &str, name: &[u8], read: bool) {
    let allocator = Allocator::new();
    let checked = AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    };
    let (ast, semantic) = analyze_with_options(&allocator, source, ts_module_options(), checked);
    let reference = semantic
        .iter_references()
        .find(|entry| {
            ast.get_utf8(entry.reference.name).as_bytes() == name
                && entry.reference.flags.space == ReferenceSpace::Namespace
        })
        .unwrap()
        .reference;
    assert_eq!(reference.flags.is_read(), read);
    assert!(!reference.flags.is_write());
}

#[test]
fn import_export_links() {
    let ts = ts_module_options();

    let source = "namespace Outer { export function F() {} namespace F { export interface I {} } type A = string; const A = 1; namespace Test { import A = F; export const value: typeof A = A; export let text: A; } }";
    assert_named_declaration_targets(
        source,
        ts,
        b"A",
        &[Some("A = F"), Some("A = F"), Some("A = string")],
    );
    assert_import_root_access(source, b"F", true);

    let source = "namespace Outer { export class C {} namespace C { export interface I {} } type A = string; const A = 1; namespace Test { import A = C; export let text: A; export const value = A; } }";
    assert_named_declaration_targets(source, ts, b"A", &[Some("A = C"), Some("A = C")]);

    let source = "namespace Outer { export function F() {} namespace F { export interface I {} } namespace Test { import A = F.I; export let item: A; } }";
    assert_named_declaration_targets(source, ts, b"A", &[Some("A = F.I")]);
    assert_import_root_access(source, b"F", false);

    let source = "namespace Outer { export function F() {} } namespace Outer { namespace F { export interface I {} } type A = string; const A = 1; namespace Test { import A = F; export const value: typeof A = A; export let item: A.I; } }";
    assert_named_declaration_targets(
        source,
        ts,
        b"A",
        &[Some("A = string"), Some("A = string"), Some("A = F")],
    );
    assert_import_root_access(source, b"F", false);

    let source = "namespace Outer { export function F() {} namespace F { export interface I {} } namespace Test { import A = F; const A = 1; export const value = A; } }";
    assert_named_declaration_targets(source, ts, b"A", &[Some("A = F")]);
}

#[test]
fn import_member_fallback() {
    let ts = ts_module_options();

    let source = "namespace N { export class C { static x = 1; } } import Owner = N.C; namespace Owner { export interface Item {} } type A = string; namespace Test { import A = Owner.x; export let text: A; export const value = A; }";
    assert_named_declaration_targets(source, ts, b"A", &[Some("A = string"), Some("A = Owner.x")]);
    assert_import_root_access(source, b"Owner", true);

    let source = "namespace Owner { export interface Item {} } import Owner = N.C; namespace N { export class C { static x = 1; } } import Mid = Owner; type A = string; namespace Test { import A = Mid.x; export let text: A; export const value = A; }";
    assert_named_declaration_targets(source, ts, b"A", &[Some("A = string"), Some("A = Mid.x")]);

    let source = "namespace N { export class C { static x = 1; } } import Owner = N.C; namespace Owner { export interface x {} } const A = 1; namespace Test { import A = Owner.x; export let text: A; export const value = A; }";
    assert_named_declaration_targets(source, ts, b"A", &[Some("A = Owner.x"), Some("A = 1")]);
    assert_import_root_access(source, b"Owner", false);
}

#[test]
fn anonymous_class_import_members() {
    let ts = ts_module_options();

    let source = "export default interface I {} export default class { static x = 1; } export namespace Owner { export interface Item {} } export { I as Owner }; type A = string; namespace Test { import A = Owner.x; export let text: A; export const value = A; }";
    assert_named_declaration_targets(source, ts, b"A", &[Some("A = string"), Some("A = Owner.x")]);
    assert_import_root_access(source, b"Owner", true);

    let source = "export default interface I {} export default class {} export namespace Owner { export interface Item {} } export { I as Owner }; type A = string; namespace Test { import A = Owner.prototype; export let text: A; export const value = A; }";
    assert_named_declaration_targets(
        source,
        ts,
        b"A",
        &[Some("A = string"), Some("A = Owner.prototype")],
    );
}

#[track_caller]
fn assert_qualified_alias_bindings(source: &str, alias_is_value: bool) {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
        let alias = semantic
            .iter_symbols()
            .find(|entry| {
                ast.get_wtf8(entry.symbol.name).as_bytes() == b"A"
                    && entry.symbol.flags.contains(SymbolFlags::IMPORT)
            })
            .unwrap()
            .id;
        let mut seen = [false; 2];
        for entry in semantic.iter_references() {
            let reference = entry.reference;
            let name = ast.get_utf8(reference.name).as_bytes();
            if name == b"A" {
                let value = reference.flags.space != ReferenceSpace::Type;
                seen[usize::from(value)] = true;
                let expected = if value == alias_is_value {
                    alias
                } else {
                    outer
                };
                assert_eq!(reference.symbol, Some(expected));
                assert_eq!(reference.resolution(), NameResolution::Resolved(expected));
                assert_eq!(
                    semantic.reference_candidates(entry.id).collect::<Vec<_>>(),
                    [expected]
                );
                assert!(semantic.uses(expected).contains(&entry.id));
                assert_eq!(
                    semantic.lookup(reference.scope, name, reference.flags.space),
                    Some(expected)
                );
            }
            if name == b"B" {
                let binding = semantic.binding(ScopeId::MODULE, b"B").unwrap();
                assert_eq!(reference.symbol, Some(binding));
                assert_eq!(
                    semantic.lookup(reference.scope, name, reference.flags.space),
                    Some(binding)
                );
            }
        }
        assert!(seen.into_iter().all(|seen| seen));
        let root = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"N")
            .unwrap()
            .reference;
        assert_eq!(root.flags.is_read(), alias_is_value);
    }
}

#[test]
fn qualified_export_fallback() {
    assert_qualified_alias_bindings(
        "export {}; declare namespace N { class Original { static x: number; } export namespace X { interface T { own: true; } } export { Original as X }; } import Mid = N.X; declare namespace Holder { export { Mid as Y }; } type A = number; namespace Use { import A = Holder.Y.x; export const typed: A = 1; export const runtime = A; export type Q = typeof A; } import B = Holder.Y.T; const b: B = { own: true };\n",
        true,
    );

    assert_qualified_alias_bindings(
        "export {}; declare namespace N { class Original { static x: number; } export namespace X { interface T { own: true; } } export { Original as X }; } import Mid = N.X; type A = number; namespace Use { import A = Mid.x; export const typed: A = 1; export const runtime = A; export type Q = typeof A; } import B = Mid.T; const b: B = { own: true };\n",
        true,
    );

    assert_qualified_alias_bindings(
        "export {}; declare namespace N { class Original { static x: number; } export namespace X { interface T { own: true; } } export { Original as X }; } type A = number; namespace Use { import A = N.X.x; export const typed: A = 1; export const runtime = A; export type Q = typeof A; } import B = N.X.T; const b: B = { own: true };\n",
        true,
    );

    assert_qualified_alias_bindings(
        "export {}; declare namespace N { class Original { static x: number; } export namespace X { interface T { own: true; } } export { Original as X }; export { X as Y }; } type A = number; namespace Use { import A = N.Y.x; export const typed: A = 1; export const runtime = A; export type Q = typeof A; } import B = N.Y.T; const b: B = { own: true };\n",
        true,
    );

    assert_qualified_alias_bindings(
        "export {}; declare namespace N { export namespace X { interface T { own: true; } } } declare namespace N { class Original { static x: number; } export { Original as X }; } type A = number; namespace Use { import A = N.X.x; export const typed: A = 1; export const runtime = A; export type Q = typeof A; } import B = N.X.T; const b: B = { own: true };\n",
        true,
    );

    assert_qualified_alias_bindings(
        "export {}; declare namespace N { class Original { static x: number; } export namespace X { interface x { own: true; } } export { Original as X }; } const A = 1; namespace Use { import A = N.X.x; export const typed: A = { own: true }; export const runtime = A; }\n",
        false,
    );
}

#[track_caller]
fn assert_namespace_owner_binding(source: &str, declaration: &str, import_root: &[u8]) {
    let allocator = Allocator::new();
    let expected_start = source.find(declaration).unwrap() as u32;
    let check = |options| {
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let target = semantic
            .iter_declarations()
            .find(|entry| ast.span(entry.declaration.node).start == expected_start)
            .unwrap()
            .declaration
            .symbol;
        let mut references = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name) == "A");
        let entry = references.next().unwrap();
        assert!(references.next().is_none());
        assert_eq!(
            entry.reference.resolution(),
            NameResolution::Resolved(target)
        );
        assert_eq!(
            semantic.lookup(entry.reference.scope, b"A", entry.reference.flags.space),
            Some(target)
        );
        assert_eq!(
            semantic.reference_candidates(entry.id).collect::<Vec<_>>(),
            [target]
        );
        for symbol in semantic
            .iter_symbols()
            .filter(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == b"A")
        {
            let expected: &[swc_next_ecma_ast::ReferenceId] = if symbol.id == target {
                &[entry.id]
            } else {
                &[]
            };
            assert_eq!(semantic.uses(symbol.id), expected);
        }
        let root = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == import_root)
            .unwrap();
        assert!(!root.reference.flags.is_read());
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    });
}

#[test]
fn qualified_namespace_owners() {
    assert_namespace_owner_binding(
        "declare namespace N { namespace Source { export interface x {} } export class X { static x: number; } export { Source as X }; } type A = number; namespace Use { import A = N.X.x; const value: A = {}; }",
        "A = N.X.x",
        b"N",
    );

    assert_namespace_owner_binding(
        "declare namespace N { namespace Source { export interface x {} } export class X { static x: number; } export { Source as X }; } const A = 0; namespace Use { import A = N.X.x; const value = A; }",
        "A = 0",
        b"N",
    );

    assert_namespace_owner_binding(
        "declare namespace N { namespace Source { export interface x {} } export class X { static x: number; } export { Source as X }; } type A = number; namespace Use { import Owner = N.X; import A = Owner.x; const value: A = {}; }",
        "A = Owner.x",
        b"Owner",
    );

    assert_namespace_owner_binding(
        "declare namespace N { namespace Source { export interface x {} } export class X { static x: number; } export { Source as X }; export { X as Y }; } type A = number; namespace Use { import A = N.Y.x; const value: A = {}; }",
        "A = N.Y.x",
        b"N",
    );
}

fn import_binding_flags(source: &str) -> SymbolFlags {
    let allocator = Allocator::new();
    let (_, semantic) = analyze_source(&allocator, source, ts_module_options());
    let alias = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    semantic.symbol(alias).flags
}

#[test]
fn import_binding_forms() {
    let flags = import_binding_flags("import A from 'dep';");
    assert!(!flags.contains(SymbolFlags::IMPORT_EQUALS));

    let flags = import_binding_flags("import type A from 'dep';");
    assert!(!flags.contains(SymbolFlags::IMPORT_EQUALS));

    let flags = import_binding_flags("import A = Unknown.Member;");
    assert!(flags.contains(SymbolFlags::IMPORT_EQUALS));

    let flags = import_binding_flags("import A = B; import B = A;");
    assert!(flags.contains(SymbolFlags::IMPORT_EQUALS));

    let flags = import_binding_flags("import A = require('dep');");
    assert!(flags.contains(SymbolFlags::IMPORT_EQUALS | SymbolFlags::IMPORT));

    let flags = import_binding_flags("import type A = require('dep');");
    assert!(flags.contains(SymbolFlags::IMPORT_EQUALS | SymbolFlags::TYPE_IMPORT));
}
