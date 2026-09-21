use super::*;
use swc_next_ecma_ast::{AssignmentExpression, Ast, SymbolId, Visit, VisitWith};
use swc_next_ecma_semantic::{Semantic, SymbolFlags};

fn assert_parameter_assignment(code: &str) {
    with_output_semantic(code, |ast, semantic| {
        let parameter = semantic
            .iter_symbols()
            .find(|entry| {
                entry.symbol.flags.contains(SymbolFlags::PARAMETER)
                    && ast.get_wtf8(entry.symbol.name).as_bytes() == b"x"
            })
            .unwrap()
            .id;
        struct Assignments<'s, 'ast> {
            ast: &'s Ast<'ast>,
            semantic: &'s Semantic<'ast>,
            parameter: SymbolId,
            count: usize,
        }
        impl<'ast> Visit<'ast> for Assignments<'_, 'ast> {
            fn ast(&self) -> &Ast<'ast> {
                self.ast
            }
            fn visit_assignment_expression(&mut self, assignment: AssignmentExpression) {
                if let Some(reference) =
                    assignment.right(self.ast).as_identifier_reference(self.ast)
                    && self.ast.get_utf8(reference.name(self.ast)) == "x"
                {
                    let reference = self.semantic.reference_of(reference.node_id()).unwrap();
                    assert_eq!(
                        self.semantic.reference(reference).resolution().symbol(),
                        Some(self.parameter)
                    );
                    self.count += 1;
                }
                assignment.visit_children_with(self);
            }
        }
        let mut assignments = Assignments {
            ast,
            semantic,
            parameter,
            count: 0,
        };
        ast.root_program().visit_with(&mut assignments);
        assert_eq!(assignments.count, 1);
    });
}

#[test]
fn parameter_bindings() {
    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { { let x = 99; super(); } } }",
    );
    assert_parameter_assignment(&code);

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { { super(); let x = 99; } } }",
    );
    assert_parameter_assignment(&code);

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { ((x: number) => { super(); })(99); } }",
    );
    assert_parameter_assignment(&code);

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { try { throw 99; } catch (x) { super(); } } }",
    );
    assert_parameter_assignment(&code);

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { x = 2; super(); } }",
    );
    assert_parameter_assignment(&code);
    assert!(!code.contains("=>"));
}

#[test]
fn super_environments() {
    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { class Inner extends (super(), B) {} } }",
    );
    assert_parameter_assignment(&code);

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { class Inner { [(super(), 'm')]() {} } } }",
    );
    assert_parameter_assignment(&code);

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { super(); class Inner extends B { constructor() { super(); } } } }",
    );
    assert_parameter_assignment(&code);
    assert!(!code.contains("=>"));
}

#[test]
fn nested_parameter_initialization() {
    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { class Inner { [(super(), 'm')] = 0; } } }",
    );
    assert_parameter_assignment(&code);

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { const init = () => { const x = 99; super(); }; x = 2; init(); } }",
    );
    assert_parameter_assignment(&code);
}

#[test]
fn decorator_environments() {
    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { @dec(super()) class Inner {} } }",
    );
    assert!(code.contains("super(), _initProperties(), this"));
    assert!(code.contains("this.x = x"));

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { class Inner { @dec(super()) method() {} } } }",
    );
    assert!(code.contains("super(), _initProperties(), this"));
    assert!(code.contains("this.x = x"));

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { class Inner { @dec(super()) field = 0; } } }",
    );
    assert!(code.contains("super(), _initProperties(), this"));
    assert!(code.contains("this.x = x"));

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { class Inner { method(@dec(super()) arg: number) {} } } }",
    );
    assert!(code.contains("super(), _initProperties(), this"));
    assert!(code.contains("this.x = x"));

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { class Inner { method(@dec(super()) ...args: number[]) {} } } }",
    );
    assert!(code.contains("super(), _initProperties(), this"));
    assert!(code.contains("this.x = x"));

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { class Inner { constructor(@dec(super()) public arg: number) {} } } }",
    );
    assert!(code.contains("super(), _initProperties(), this"));
    assert!(code.contains("this.x = x"));
}

#[test]
fn decorator_bindings() {
    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { { let x = 99; @dec(super()) class Inner {} } } }",
    );
    assert_parameter_assignment(&code);

    let code = transformed_code(
        "class B {} class C extends B { constructor(public x: number) { @dec(super()) class Inner extends B { constructor() { super(); } field = () => this; static { const value = this; } } } }",
    );
    assert_parameter_assignment(&code);
    assert_eq!(code.matches("_initProperties()").count(), 1);
}
