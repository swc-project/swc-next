use super::super::JsxRuntime;
use super::{JsxLowerer, build::*};
use swc_next_ecma_ast::{Ast, AstUtf8, DUMMY_SP, Decl, Expr, Program, SourceType, Stmt};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum RuntimeHelper {
    Jsx,
    Jsxs,
    JsxDev,
    Fragment,
}

impl RuntimeHelper {
    fn imported_name(self) -> &'static str {
        match self {
            Self::Jsx => "jsx",
            Self::Jsxs => "jsxs",
            Self::JsxDev => "jsxDEV",
            Self::Fragment => "Fragment",
        }
    }
}

impl<'ast> JsxLowerer<'_, 'ast> {
    pub(super) fn runtime_helper(&mut self, helper: RuntimeHelper, ast: &mut Ast<'ast>) -> Expr {
        if ast.source_type() == SourceType::Module {
            let name = self.ensure_module_helper(helper, ast);
            return ast.expr_identifier_reference(DUMMY_SP, name);
        }
        let object = self.ensure_runtime_object(ast);
        let property = helper.imported_name();
        static_member_with_name(ast, object, property)
    }

    pub(super) fn create_element_helper(&mut self, ast: &mut Ast<'ast>) -> Expr {
        if ast.source_type() == SourceType::Module {
            let name = if let Some(name) = self.create_element {
                name
            } else {
                let name = self.uid_generator.fresh("_createElement", ast);
                self.create_element = Some(name);
                name
            };
            return ast.expr_identifier_reference(DUMMY_SP, name);
        }
        let object = self.ensure_react_object(ast);
        static_member_with_name(ast, object, "createElement")
    }

    fn ensure_module_helper(&mut self, helper: RuntimeHelper, ast: &mut Ast<'ast>) -> AstUtf8 {
        let current = match helper {
            RuntimeHelper::Jsx => self.jsx,
            RuntimeHelper::Jsxs => self.jsxs,
            RuntimeHelper::JsxDev => self.jsx_dev,
            RuntimeHelper::Fragment => self.fragment,
        };
        if let Some(current) = current {
            return current;
        }
        let base = match helper {
            RuntimeHelper::Jsx => "_jsx",
            RuntimeHelper::Jsxs => "_jsxs",
            RuntimeHelper::JsxDev => "_jsxDEV",
            RuntimeHelper::Fragment => "_Fragment",
        };
        let name = self.uid_generator.fresh(base, ast);
        match helper {
            RuntimeHelper::Jsx => self.jsx = Some(name),
            RuntimeHelper::Jsxs => self.jsxs = Some(name),
            RuntimeHelper::JsxDev => self.jsx_dev = Some(name),
            RuntimeHelper::Fragment => self.fragment = Some(name),
        }
        self.helper_imports.push((helper.imported_name(), name));
        name
    }

    fn ensure_runtime_object(&mut self, ast: &mut Ast<'ast>) -> AstUtf8 {
        if let Some(name) = self.runtime_object {
            return name;
        }
        let base = if self.options.development {
            "_reactJsxDevRuntime"
        } else {
            "_reactJsxRuntime"
        };
        let name = self.uid_generator.fresh(base, ast);
        self.runtime_object = Some(name);
        name
    }

    fn ensure_react_object(&mut self, ast: &mut Ast<'ast>) -> AstUtf8 {
        if let Some(name) = self.react_object {
            return name;
        }
        let name = self.uid_generator.fresh("_react", ast);
        self.react_object = Some(name);
        name
    }

    fn runtime_bindings(&self, ast: &mut Ast<'ast>) -> Vec<Stmt> {
        let mut bindings = Vec::new();
        if self.options.runtime != JsxRuntime::Automatic {
            return bindings;
        }
        if ast.source_type() == SourceType::Module {
            if !self.helper_imports.is_empty() {
                let runtime_source = self.runtime_source();
                bindings.push(import_statement(
                    ast,
                    &runtime_source,
                    self.helper_imports.iter().copied(),
                ));
            }
            if let Some(local) = self.create_element {
                bindings.push(import_statement(
                    ast,
                    self.import_source,
                    [("createElement", local)],
                ));
            }
        } else {
            if let Some(local) = self.react_object {
                bindings.push(require_statement(ast, self.import_source, local));
            }
            if let Some(local) = self.runtime_object {
                let runtime_source = self.runtime_source();
                bindings.push(require_statement(ast, &runtime_source, local));
            }
        }
        bindings
    }

    pub(super) fn insert_program_bindings(&mut self, program: Program, ast: &mut Ast<'ast>) {
        if self.options.runtime == JsxRuntime::Classic && self.filename.is_none() {
            return;
        }
        let mut bindings = self.runtime_bindings(ast);
        let filename = self.filename.map(|filename| {
            let source_path = string_expression(ast, DUMMY_SP, self.source_path);
            let declaration = variable_declaration(ast, filename, source_path);
            Stmt::Declaration(Decl::VariableDeclaration(declaration))
        });
        if bindings.is_empty() && filename.is_none() {
            return;
        }

        let existing = program.body(ast);
        let is_module = ast.source_type() == SourceType::Module;
        let insertion = if is_module && !bindings.is_empty() {
            existing
                .iter()
                .enumerate()
                .filter_map(|(index, id)| {
                    let statement = ast.get_node_in_sub_range(id);
                    if statement.is_import_declaration(ast) {
                        Some(index + 1)
                    } else if Some(statement) == self.runtime_import_anchor {
                        Some(index)
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(0)
        } else {
            0
        };
        let mut before = existing;
        let after = before.split_off(insertion);
        let mut body =
            Vec::with_capacity(existing.len() + bindings.len() + usize::from(filename.is_some()));
        if is_module {
            body.extend(filename);
            body.extend(before.iter().map(|id| ast.get_node_in_sub_range(id)));
            body.append(&mut bindings);
        } else {
            body.append(&mut bindings);
            body.extend(filename);
        }
        body.extend(after.iter().map(|id| ast.get_node_in_sub_range(id)));
        let body = ast.add_sub_range(body);
        program.set_body(ast, body);
    }

    fn runtime_source(&self) -> String {
        let suffix = if self.options.development {
            "jsx-dev-runtime"
        } else {
            "jsx-runtime"
        };
        format!("{}/{suffix}", self.import_source)
    }

    pub(super) fn source_object(&mut self, offset: u32, ast: &mut Ast<'ast>) -> Expr {
        let filename = if let Some(filename) = self.filename {
            filename
        } else {
            let filename = self.uid_generator.fresh("_jsxFileName", ast);
            self.filename = Some(filename);
            filename
        };
        let (line, column) = self.source_locations.line_column(ast, offset);
        let file_name = ast.expr_identifier_reference(DUMMY_SP, filename);
        let line_number = ast.expr_numeric_literal(DUMMY_SP, f64::from(line), None);
        let column_number = ast.expr_numeric_literal(DUMMY_SP, f64::from(column), None);
        let file_name = object_property(ast, "fileName", file_name, DUMMY_SP);
        let line_number = object_property(ast, "lineNumber", line_number, DUMMY_SP);
        let column_number = object_property(ast, "columnNumber", column_number, DUMMY_SP);
        let properties = ast.add_sub_range([file_name, line_number, column_number]);
        ast.expr_object_expression(DUMMY_SP, properties)
    }
}
