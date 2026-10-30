//! Automatic JSX runtime lowering for production and development builds.
use crate::used_names::UsedNameState;
use serde::Deserialize;
use swc_next_allocator::wtf8::Wtf8;
use swc_next_ecma_ast::*;
use swc_next_ecma_semantic::Semantic;

/// Options corresponding to TypeScript's `react-jsx` and `react-jsxdev` modes.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct JsxOptions {
    /// Emit `jsxDEV` calls with source locations instead of `jsx`/`jsxs`.
    pub development: bool,
    /// Runtime package, before `/jsx-runtime` or `/jsx-dev-runtime` is added.
    pub import_source: String,
    /// Source filename recorded in development metadata.
    pub filename: Option<String>,
}
impl Default for JsxOptions {
    fn default() -> Self {
        Self {
            development: false,
            import_source: "react".into(),
            filename: None,
        }
    }
}

/// Lower JSX after TypeScript erasure. Rebuild semantic indexes after mutation.
pub fn transform<'ast>(
    ast: &mut Ast<'ast>,
    semantic: &Semantic<'ast>,
    options: JsxOptions,
) -> Vec<Diagnostic<'static>> {
    let mut names = UsedNameState::new(ast.allocator());
    for entry in semantic.iter_symbols() {
        names.observe(ast.get_wtf8(entry.symbol.name).as_bytes());
    }
    for entry in semantic.iter_references() {
        names.observe(ast.get_utf8(entry.reference.name).as_bytes());
    }
    let mut lowerer = JsxLowerer {
        ast,
        names,
        options,
        imports: Vec::new(),
        diagnostics: Vec::new(),
    };
    let program = lowerer.ast.root_program();
    for (kind, value) in crate::jsx_pragma::pragmas(program, lowerer.ast) {
        if matches!(kind, crate::jsx_pragma::PragmaKind::ImportSource) {
            lowerer.options.import_source = value.into();
        }
        if matches!(kind, crate::jsx_pragma::PragmaKind::Runtime) && value != "automatic" {
            lowerer.diagnostics.push(Diagnostic::error(
                "The configured JSX transform requires the automatic runtime",
                DUMMY_SP,
            ));
        }
    }
    program.visit_mut_with(&mut lowerer);
    lowerer.prepend_imports(program);
    lowerer.diagnostics
}

struct JsxLowerer<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    names: UsedNameState<'ast>,
    options: JsxOptions,
    imports: Vec<(&'static str, AstUtf8, bool)>,
    diagnostics: Vec<Diagnostic<'static>>,
}

impl<'ast> JsxLowerer<'_, 'ast> {
    fn helper(&mut self, exported: &'static str, root: bool) -> Expr {
        let name = if let Some((_, name, _)) = self
            .imports
            .iter()
            .find(|(name, _, from_root)| *name == exported && *from_root == root)
        {
            *name
        } else {
            let name = self.names.next_name(&format!("_{exported}"), 2);
            let name = self.ast.add_utf8(name);
            self.imports.push((exported, name, root));
            name
        };
        self.ast.expr_identifier_reference(DUMMY_SP, name)
    }
    fn string(&mut self, span: Span, text: &str) -> Expr {
        let value = self.ast.add_wtf8(Wtf8::from_str(text));
        let raw = self.ast.empty_utf8();
        self.ast.expr_string_literal(span, value, raw)
    }
    fn undefined(&mut self) -> Expr {
        let zero = self.ast.expr_numeric_literal(DUMMY_SP, 0.0, None);
        self.ast
            .expr_unary_expression(DUMMY_SP, zero, UnaryOperator::Void)
    }
    fn property(&mut self, name: &str, value: Expr) -> ObjectPropertyKind {
        let key = PropertyKey::Expr(self.string(DUMMY_SP, name));
        // A JSX prop named __proto__ is an own data property, never a prototype setter.
        self.ast.object_property_kind_object_property(
            DUMMY_SP,
            key,
            value,
            PropertyKind::Init,
            false,
            false,
            name == "__proto__",
        )
    }
    fn object(&mut self, properties: Vec<ObjectPropertyKind>) -> Expr {
        let properties = self.ast.add_sub_range(properties);
        self.ast.expr_object_expression(DUMMY_SP, properties)
    }
    fn expression(&mut self, expression: Expr) -> Expr {
        expression.visit_mut_with(self).unwrap_or(expression)
    }
    fn element_name(&mut self, name: JsxElementName) -> Expr {
        match self.ast.jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(id) => {
                let name = id.name(self.ast);
                let text = self.ast.get_utf8(name);
                if text == "this" {
                    self.ast.expr_this_expression(id.span(self.ast))
                } else if text.starts_with(|c: char| c.is_ascii_lowercase()) || text.contains('-') {
                    self.string(id.span(self.ast), text)
                } else {
                    self.ast.expr_identifier_reference(id.span(self.ast), name)
                }
            }
            JsxElementNameData::JsxMemberExpression(member) => self.member_name(member),
            JsxElementNameData::JsxNamespacedName(name) => {
                self.diagnostics.push(Diagnostic::error(
                    "JSX namespace tags are unsupported by the automatic runtime",
                    name.span(self.ast),
                ));
                let text = self.namespaced(name);
                self.string(name.span(self.ast), &text)
            }
        }
    }
    fn namespaced(&self, name: JsxNamespacedName) -> String {
        format!(
            "{}:{}",
            self.ast.get_utf8(name.namespace(self.ast).name(self.ast)),
            self.ast.get_utf8(name.name(self.ast).name(self.ast))
        )
    }
    fn member_name(&mut self, member: JsxMemberExpression) -> Expr {
        let object = match self
            .ast
            .jsx_member_expression_object_data(member.object(self.ast))
        {
            JsxMemberExpressionObjectData::JsxIdentifier(id) => {
                let name = id.name(self.ast);
                if self.ast.get_utf8(name) == "this" {
                    self.ast.expr_this_expression(id.span(self.ast))
                } else {
                    self.ast.expr_identifier_reference(id.span(self.ast), name)
                }
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => self.member_name(member),
        };
        let property = member.property(self.ast);
        let key = self
            .ast
            .property_key_identifier_name(property.span(self.ast), property.name(self.ast));
        self.ast
            .expr_member_expression(member.span(self.ast), object, key, false, false)
    }
    fn attribute_value(&mut self, value: Option<JsxAttributeValue>) -> Expr {
        match value.map(|value| self.ast.jsx_attribute_value_data(value)) {
            None => self.ast.expr_boolean_literal(DUMMY_SP, true),
            Some(JsxAttributeValueData::StringLiteral(literal)) => {
                let span = literal.span(self.ast);
                let raw = &self.ast.source()[span.start as usize + 1..span.end as usize - 1];
                let text = raw.replace(['\r', '\n', '\t'], " ");
                self.string(span, &html_escape::decode_html_entities(&text))
            }
            Some(JsxAttributeValueData::JsxExpressionContainer(container)) => {
                if let Some(expression) = container.expression(self.ast).as_expr(self.ast) {
                    self.expression(expression)
                } else {
                    self.undefined()
                }
            }
            Some(JsxAttributeValueData::JsxElement(element)) => self.element(element),
            Some(JsxAttributeValueData::JsxFragment(fragment)) => self.fragment(fragment),
        }
    }
    fn children(&mut self, children: SubRange<JsxChild>) -> Vec<Expr> {
        let mut result = Vec::new();
        for id in children.iter() {
            let child = self.ast.get_node_in_sub_range(id);
            let value = match self.ast.jsx_child_data(child) {
                JsxChildData::JsxText(text) => {
                    let text_value = clean_text(self.ast.get_utf8(text.value(self.ast)));
                    if text_value.is_empty() {
                        continue;
                    }
                    self.string(
                        text.span(self.ast),
                        &html_escape::decode_html_entities(&text_value),
                    )
                }
                JsxChildData::JsxExpressionContainer(container) => {
                    let Some(expression) = container.expression(self.ast).as_expr(self.ast) else {
                        continue;
                    };
                    self.expression(expression)
                }
                JsxChildData::JsxElement(element) => self.element(element),
                JsxChildData::JsxFragment(fragment) => self.fragment(fragment),
                JsxChildData::JsxSpreadChild(child) => {
                    self.diagnostics.push(Diagnostic::error(
                        "JSX spread children are unsupported by the automatic runtime",
                        child.span(self.ast),
                    ));
                    continue;
                }
            };
            result.push(value);
        }
        result
    }
    fn element(&mut self, element: JsxElement) -> Expr {
        let opening = element.opening_element(self.ast);
        let tag = self.element_name(opening.name(self.ast));
        let attributes = opening.attributes(self.ast);
        let mut spread_seen = false;
        let mut key_count = 0;
        let fallback = attributes.iter().any(|id| {
            let attribute = self.ast.get_node_in_sub_range(id);
            match self.ast.jsx_attribute_item_data(attribute) {
                JsxAttributeItemData::JsxSpreadAttribute(_) => {
                    spread_seen = true;
                    false
                }
                JsxAttributeItemData::JsxAttribute(attribute) => {
                    let is_key = attribute
                        .name(self.ast)
                        .as_jsx_identifier(self.ast)
                        .is_some_and(|id| self.ast.get_utf8(id.name(self.ast)) == "key");
                    key_count += usize::from(is_key);
                    is_key && (spread_seen || key_count > 1)
                }
            }
        });
        let mut properties = Vec::new();
        let mut key = None;
        for id in attributes.iter() {
            let attribute = self.ast.get_node_in_sub_range(id);
            match self.ast.jsx_attribute_item_data(attribute) {
                JsxAttributeItemData::JsxSpreadAttribute(spread) => {
                    let argument = self.expression(spread.argument(self.ast));
                    properties.push(
                        self.ast
                            .object_property_kind_spread_element(spread.span(self.ast), argument),
                    );
                }
                JsxAttributeItemData::JsxAttribute(attribute) => {
                    let name = match self.ast.jsx_attribute_name_data(attribute.name(self.ast)) {
                        JsxAttributeNameData::JsxIdentifier(id) => {
                            self.ast.get_utf8(id.name(self.ast)).to_owned()
                        }
                        JsxAttributeNameData::JsxNamespacedName(name) => self.namespaced(name),
                    };
                    let value = self.attribute_value(attribute.value(self.ast));
                    if name == "key" && !fallback {
                        key = Some(value);
                    } else {
                        properties.push(self.property(&name, value));
                    }
                }
            }
        }
        let children = self.children(element.children(self.ast));
        self.call(
            element.span(self.ast),
            tag,
            properties,
            key,
            children,
            fallback,
        )
    }
    fn fragment(&mut self, fragment: JsxFragment) -> Expr {
        let tag = self.helper("Fragment", false);
        let children = self.children(fragment.children(self.ast));
        self.call(
            fragment.span(self.ast),
            tag,
            Vec::new(),
            None,
            children,
            false,
        )
    }
    fn call(
        &mut self,
        span: Span,
        tag: Expr,
        mut properties: Vec<ObjectPropertyKind>,
        key: Option<Expr>,
        children: Vec<Expr>,
        fallback: bool,
    ) -> Expr {
        let count = children.len();
        let mut arguments = vec![Argument::Expr(tag)];
        if !fallback && count > 0 {
            let children = if count == 1 {
                children[0]
            } else {
                let elements = self
                    .ast
                    .add_sub_range(children.iter().map(|child| Some(Argument::Expr(*child))));
                self.ast.expr_array_expression(span, elements)
            };
            properties.push(self.property("children", children));
        }
        let props = self.object(properties);
        arguments.push(Argument::Expr(props));
        let callee = if fallback {
            arguments.extend(children.into_iter().map(Argument::Expr));
            self.helper("createElement", true)
        } else {
            let key = key.unwrap_or_else(|| self.undefined());
            arguments.push(Argument::Expr(key));
            if self.options.development {
                let static_children = self.ast.expr_boolean_literal(span, count > 1);
                arguments.push(Argument::Expr(static_children));
                let source = self.source_location(span);
                arguments.push(Argument::Expr(source));
                let self_argument = self.undefined();
                arguments.push(Argument::Expr(self_argument));
                self.helper("jsxDEV", false)
            } else {
                self.helper(if count > 1 { "jsxs" } else { "jsx" }, false)
            }
        };
        let arguments = self.ast.add_sub_range(arguments);
        self.ast
            .expr_call_expression(span, callee, None, arguments, false)
    }
    fn source_location(&mut self, span: Span) -> Expr {
        let prefix = &self.ast.source()[..span.start as usize];
        let mut line = 1;
        let mut column = 1;
        let mut previous_cr = false;
        for ch in prefix.chars() {
            match ch {
                '\r' => {
                    line += 1;
                    column = 1;
                }
                '\n' if previous_cr => {}
                '\n' | '\u{2028}' | '\u{2029}' => {
                    line += 1;
                    column = 1;
                }
                _ => column += ch.len_utf16(),
            }
            previous_cr = ch == '\r';
        }
        let filename = self
            .options
            .filename
            .as_deref()
            .unwrap_or("<anon>")
            .to_owned();
        let filename = self.string(DUMMY_SP, &filename);
        let line = self.ast.expr_numeric_literal(DUMMY_SP, line as f64, None);
        let column = self.ast.expr_numeric_literal(DUMMY_SP, column as f64, None);
        let properties = vec![
            self.property("fileName", filename),
            self.property("lineNumber", line),
            self.property("columnNumber", column),
        ];
        self.object(properties)
    }
    fn prepend_imports(&mut self, program: Program) {
        let mut prefix = Vec::new();
        for (exported, local, root) in self.imports.clone() {
            let path = if root {
                self.options.import_source.clone()
            } else {
                format!(
                    "{}/{}",
                    self.options.import_source,
                    if self.options.development {
                        "jsx-dev-runtime"
                    } else {
                        "jsx-runtime"
                    }
                )
            };
            let imported_name = self.ast.add_utf8(exported);
            if self.ast.source_type() == SourceType::Module {
                let imported = self.ast.identifier_name(DUMMY_SP, imported_name);
                let local = self.ast.binding_identifier(DUMMY_SP, local);
                let specifier = self.ast.import_declaration_specifier_import_specifier(
                    DUMMY_SP,
                    ModuleExportName::IdentifierName(imported),
                    local,
                    ImportOrExportKind::Value,
                );
                let specifiers = self.ast.add_sub_range([specifier]);
                let value = self.ast.add_wtf8(Wtf8::from_str(&path));
                let raw = self.ast.empty_utf8();
                let source = self.ast.string_literal(DUMMY_SP, value, raw);
                prefix.push(self.ast.stmt_import_declaration(
                    DUMMY_SP,
                    specifiers,
                    source,
                    SubRange::empty(),
                    None,
                    ImportOrExportKind::Value,
                ));
            } else {
                let require_name = self.ast.add_utf8("require");
                let require = self.ast.expr_identifier_reference(DUMMY_SP, require_name);
                let path = self.string(DUMMY_SP, &path);
                let arguments = self.ast.add_sub_range([Argument::Expr(path)]);
                let call = self
                    .ast
                    .expr_call_expression(DUMMY_SP, require, None, arguments, false);
                let property = self
                    .ast
                    .property_key_identifier_name(DUMMY_SP, imported_name);
                let value = self
                    .ast
                    .expr_member_expression(DUMMY_SP, call, property, false, false);
                let binding = self.ast.binding_identifier(DUMMY_SP, local);
                let declarator = self.ast.variable_declarator(
                    DUMMY_SP,
                    BindingPattern::BindingIdentifier(binding),
                    None,
                    Some(value),
                    false,
                );
                let declarators = self.ast.add_sub_range([declarator]);
                let declaration = self.ast.variable_declaration(
                    DUMMY_SP,
                    VariableKind::Const,
                    declarators,
                    false,
                );
                prefix.push(Stmt::Declaration(Decl::VariableDeclaration(declaration)));
            }
        }
        prefix.extend(
            program
                .body(self.ast)
                .iter()
                .map(|id| self.ast.get_node_in_sub_range(id)),
        );
        let body = self.ast.add_sub_range(prefix);
        program.set_body(self.ast, body);
    }
}
impl<'ast> VisitMut<'ast> for JsxLowerer<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        match self.ast.expr_data(expression) {
            ExprData::JsxElement(element) => Some(self.element(element)),
            ExprData::JsxFragment(fragment) => Some(self.fragment(fragment)),
            _ => expression.visit_mut_children_with(self),
        }
    }
}

fn clean_text(text: &str) -> String {
    let text = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\t', " ");
    let lines = text.split('\n').collect::<Vec<_>>();
    let last_nonempty = lines
        .iter()
        .rposition(|line| !line.trim_matches(' ').is_empty())
        .unwrap_or(0);
    let mut result = String::new();
    for (index, line) in lines.iter().enumerate() {
        let line = if index > 0 {
            line.trim_start_matches(' ')
        } else {
            line
        };
        let line = if index + 1 < lines.len() {
            line.trim_end_matches(' ')
        } else {
            line
        };
        if line.is_empty() {
            continue;
        }
        result.push_str(line);
        if index < last_nonempty {
            result.push(' ');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Transformer, TransformerOptions, normalize::Normalizer};
    use swc_next_allocator::Allocator;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};

    fn output(source: &str, options: JsxOptions, source_type: SourceType) -> String {
        let allocator = Allocator::new();
        let parsed = Parser::init(
            &allocator,
            source,
            ParserOptions {
                lang: Lang::Tsx,
                source_type,
                ..Default::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let mut ast = parsed.ast;
        Normalizer::normalize_runtime(ast.root_program(), &mut ast);
        let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
        let result = Transformer::transform(
            &mut ast,
            semantic,
            TransformerOptions {
                typescript: Some(Default::default()),
                jsx: Some(options),
                ..Default::default()
            },
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let code = swc_next_ecma_codegen::print(&ast, Default::default()).code;
        let parsed = Parser::init(
            &allocator,
            &code,
            ParserOptions {
                source_type,
                ..Default::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{code}\n{:?}",
            parsed.diagnostics
        );
        let analyzed = analyze(
            &parsed.ast,
            AnalyzeOptions {
                check_syntax: SyntaxChecks::all(),
            },
        );
        assert!(
            analyzed.diagnostics.is_empty(),
            "{code}\n{:?}",
            analyzed.diagnostics
        );
        code
    }

    #[test]
    fn automatic_runtime() {
        let code = output(
            "const view = <div disabled title='a &amp; b'>text</div>;",
            JsxOptions::default(),
            SourceType::Module,
        );
        assert!(code.contains("react/jsx-runtime"));
        assert!(code.contains("_jsx(\"div\""));
        assert!(code.contains("\"disabled\": true"));
        assert!(code.contains("a & b"));
        assert!(code.contains("\"children\": \"text\""));

        let code = output(
            "const view = <><Item /><UI.Item /></>;",
            JsxOptions::default(),
            SourceType::Module,
        );
        assert!(code.contains("Fragment"));
        assert!(code.contains("jsxs"));
        assert!(code.contains("UI.Item"));
    }

    #[test]
    fn props_and_keys() {
        let code = output(
            "const view = <Item key={id} {...props} />;",
            JsxOptions::default(),
            SourceType::Module,
        );
        assert!(code.contains("...props"));
        assert!(!code.contains("createElement"));
        assert!(code.contains("id)"));

        let code = output(
            "const view = <Item {...props} key={id} />;",
            JsxOptions::default(),
            SourceType::Module,
        );
        assert!(code.contains("createElement"));
        assert!(code.contains("\"key\": id"));
        assert!(!code.contains("jsx-runtime"));

        let code = output(
            "const view = <Item __proto__={value} />;",
            JsxOptions::default(),
            SourceType::Module,
        );
        assert!(code.contains("[\"__proto__\"]: value"));
    }

    #[test]
    fn development_runtime() {
        let options = JsxOptions {
            development: true,
            filename: Some("view.tsx".into()),
            ..Default::default()
        };
        let code = output(
            "'😀';\r\n  const view = <Item />;",
            options,
            SourceType::Module,
        );
        assert!(code.contains("react/jsx-dev-runtime"));
        assert!(code.contains("jsxDEV"));
        assert!(code.contains("\"fileName\": \"view.tsx\""));
        assert!(code.contains("\"lineNumber\": 2"));
        assert!(code.contains("\"columnNumber\": 16"));
    }

    #[test]
    fn runtime_bindings() {
        let code = output(
            "const _jsx = 1; const view = <Item />;",
            JsxOptions::default(),
            SourceType::Module,
        );
        assert!(code.contains("jsx as _jsx2"));
        assert!(code.contains("_jsx2(Item"));

        let code = output(
            "const view = <Item />;",
            JsxOptions {
                import_source: "preact".into(),
                ..Default::default()
            },
            SourceType::CommonJs,
        );
        assert!(code.contains("require(\"preact/jsx-runtime\").jsx"));
        assert!(!code.contains("import "));

        let code = output(
            "/** @jsxImportSource preact */ const view = <Item />;",
            JsxOptions::default(),
            SourceType::Module,
        );
        assert!(code.contains("preact/jsx-runtime"));
    }

    #[test]
    fn jsx_text() {
        let code = output(
            "const view = <div>\n  hello\n  world &amp; friends\n</div>;",
            JsxOptions::default(),
            SourceType::Module,
        );
        assert!(code.contains("hello world & friends"));

        let code = output(
            "const view = <div>{/* comment */}{value}</div>;",
            JsxOptions::default(),
            SourceType::Module,
        );
        assert!(code.contains("\"children\": /* comment */ value"));
    }
}
