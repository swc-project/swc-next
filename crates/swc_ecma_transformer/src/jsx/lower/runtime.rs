use super::super::{JsxOptions, JsxRuntime};
use super::{JsxEmitter, JsxLowerer, NamespaceProjection, build::*};
use crate::{syntax::NamespaceParameter, uid::LazyUidGenerator};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    Ast, AstUtf8, DUMMY_SP, Decl, Expr, NodeId, Program, SourceType, Stmt, SymbolId,
};
use swc_next_ecma_semantic::{ReferenceSpace, Semantic, scope::ScopeKind};

pub(crate) enum RuntimeBindings<'options> {
    Classic(ClassicBindings<'options>),
    AutomaticModule(AutomaticModuleBindings<'options>),
    AutomaticScript(AutomaticScriptBindings<'options>),
}

pub(crate) struct ClassicBindings<'options> {
    pragma: QualifiedName<'options>,
    pragma_frag: QualifiedName<'options>,
    runtime_references: FxHashMap<NodeId, (Option<SymbolId>, Option<SymbolId>)>,
}

pub(crate) struct AutomaticModuleBindings<'options> {
    source: AutomaticSource<'options>,
    jsx: Option<AstUtf8>,
    jsxs: Option<AstUtf8>,
    jsx_dev: Option<AstUtf8>,
    fragment: Option<AstUtf8>,
    create_element: Option<AstUtf8>,
    helper_imports: Vec<(&'static str, AstUtf8)>,
}

pub(crate) struct AutomaticScriptBindings<'options> {
    source: AutomaticSource<'options>,
    runtime_object: Option<AstUtf8>,
    react_object: Option<AstUtf8>,
}

struct AutomaticSource<'options> {
    import_source: &'options str,
    development: bool,
}

impl AutomaticSource<'_> {
    fn runtime_source(&self) -> String {
        let suffix = if self.development {
            "jsx-dev-runtime"
        } else {
            "jsx-runtime"
        };
        format!("{}/{suffix}", self.import_source)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum RuntimeHelper {
    Jsx,
    Jsxs,
    JsxDev,
    Fragment,
    CreateElement,
}

impl RuntimeHelper {
    fn imported_name(self) -> &'static str {
        match self {
            Self::Jsx => "jsx",
            Self::Jsxs => "jsxs",
            Self::JsxDev => "jsxDEV",
            Self::Fragment => "Fragment",
            Self::CreateElement => "createElement",
        }
    }

    fn local_base(self) -> &'static str {
        match self {
            Self::Jsx => "_jsx",
            Self::Jsxs => "_jsxs",
            Self::JsxDev => "_jsxDEV",
            Self::Fragment => "_Fragment",
            Self::CreateElement => "_createElement",
        }
    }
}

impl<'options> RuntimeBindings<'options> {
    pub(crate) fn new(options: &'options JsxOptions, source_type: SourceType) -> Self {
        if options.runtime == JsxRuntime::Classic {
            return Self::Classic(ClassicBindings {
                pragma: QualifiedName::parse(options.pragma.as_deref(), "React", "createElement"),
                pragma_frag: QualifiedName::parse(
                    options.pragma_frag.as_deref(),
                    "React",
                    "Fragment",
                ),
                runtime_references: FxHashMap::default(),
            });
        }
        let source = AutomaticSource {
            import_source: options
                .import_source
                .as_deref()
                .filter(|source| !source.is_empty())
                .unwrap_or("react"),
            development: options.development,
        };
        if source_type == SourceType::Module {
            Self::AutomaticModule(AutomaticModuleBindings {
                source,
                jsx: None,
                jsxs: None,
                jsx_dev: None,
                fragment: None,
                create_element: None,
                helper_imports: Vec::new(),
            })
        } else {
            Self::AutomaticScript(AutomaticScriptBindings {
                source,
                runtime_object: None,
                react_object: None,
            })
        }
    }

    pub(super) fn is_classic(&self) -> bool {
        matches!(self, Self::Classic(_))
    }

    /// Source analysis cannot see these names until JSX is lowered.
    pub(crate) fn implicit_roots(&self) -> [Option<&'options str>; 2] {
        match self {
            Self::Classic(bindings) => [bindings.pragma.root(), bindings.pragma_frag.root()],
            Self::AutomaticModule(_) | Self::AutomaticScript(_) => [None, None],
        }
    }

    fn reference<'ast>(
        &mut self,
        helper: RuntimeHelper,
        ast: &mut Ast<'ast>,
        uid: &mut LazyUidGenerator<'_, 'ast>,
        namespace: &NamespaceProjection,
    ) -> Expr {
        match self {
            Self::Classic(bindings) => match helper {
                RuntimeHelper::Fragment => bindings
                    .pragma_frag
                    .create_expression(ast, namespace.fragment),
                _ => bindings.pragma.create_expression(ast, namespace.factory),
            },
            Self::AutomaticModule(bindings) => {
                let name = bindings.helper(helper, ast, uid);
                ast.expr_identifier_reference(DUMMY_SP, name)
            }
            Self::AutomaticScript(bindings) => {
                let object = bindings.object(helper, ast, uid);
                static_member_with_name(ast, object, helper.imported_name())
            }
        }
    }

    pub(super) fn runtime_dependencies(&mut self, semantic: &Semantic<'_>) -> FxHashSet<SymbolId> {
        match self {
            Self::Classic(bindings) => bindings.runtime_dependencies(semantic),
            Self::AutomaticModule(_) | Self::AutomaticScript(_) => FxHashSet::default(),
        }
    }

    pub(super) fn project_namespace(
        &self,
        expression: Expr,
        ast: &Ast<'_>,
        namespace_parameter: &NamespaceParameter<'_>,
    ) -> NamespaceProjection {
        let Self::Classic(bindings) = self else {
            return NamespaceProjection::default();
        };
        let Some(&(factory, fragment)) = bindings.runtime_references.get(&expression.node_id())
        else {
            return NamespaceProjection::default();
        };
        NamespaceProjection {
            factory: factory
                .zip(bindings.pragma.root())
                .and_then(|(symbol, name)| namespace_parameter(ast, name.as_bytes(), symbol)),
            fragment: fragment
                .zip(bindings.pragma_frag.root())
                .and_then(|(symbol, name)| namespace_parameter(ast, name.as_bytes(), symbol)),
        }
    }

    fn statements(&self, ast: &mut Ast<'_>) -> Vec<Stmt> {
        let mut statements = Vec::new();
        match self {
            Self::Classic(_) => {}
            Self::AutomaticModule(bindings) => {
                if !bindings.helper_imports.is_empty() {
                    let source = bindings.source.runtime_source();
                    statements.push(import_statement(
                        ast,
                        &source,
                        bindings.helper_imports.iter().copied(),
                    ));
                }
                if let Some(local) = bindings.create_element {
                    statements.push(import_statement(
                        ast,
                        bindings.source.import_source,
                        [("createElement", local)],
                    ));
                }
            }
            Self::AutomaticScript(bindings) => {
                if let Some(local) = bindings.react_object {
                    statements.push(require_statement(ast, bindings.source.import_source, local));
                }
                if let Some(local) = bindings.runtime_object {
                    let source = bindings.source.runtime_source();
                    statements.push(require_statement(ast, &source, local));
                }
            }
        }
        statements
    }
}

impl ClassicBindings<'_> {
    fn runtime_dependencies(&mut self, semantic: &Semantic<'_>) -> FxHashSet<SymbolId> {
        let mut dependencies = FxHashSet::default();
        if semantic.jsx_usages().is_empty() {
            return dependencies;
        }
        let has_namespaces = semantic
            .iter_scopes()
            .any(|entry| entry.scope.kind == ScopeKind::TsModule);
        for (scope, usages) in semantic.jsx_usage_groups() {
            let factory = self
                .pragma
                .root()
                .and_then(|root| semantic.lookup(scope, root.as_bytes(), ReferenceSpace::Value));
            let fragment = self
                .pragma_frag
                .root()
                .and_then(|root| semantic.lookup(scope, root.as_bytes(), ReferenceSpace::Value));
            if let Some(symbol) = factory {
                dependencies.insert(symbol);
            }
            if has_namespaces && (factory.is_some() || fragment.is_some()) {
                for usage in usages {
                    self.runtime_references
                        .insert(usage.node, (factory, fragment));
                }
            }
            if usages.iter().any(|usage| usage.is_fragment)
                && let Some(symbol) = fragment
            {
                dependencies.insert(symbol);
            }
        }
        dependencies
    }
}

impl AutomaticModuleBindings<'_> {
    fn helper<'ast>(
        &mut self,
        helper: RuntimeHelper,
        ast: &mut Ast<'ast>,
        uid: &mut LazyUidGenerator<'_, 'ast>,
    ) -> AstUtf8 {
        let slot = match helper {
            RuntimeHelper::Jsx => &mut self.jsx,
            RuntimeHelper::Jsxs => &mut self.jsxs,
            RuntimeHelper::JsxDev => &mut self.jsx_dev,
            RuntimeHelper::Fragment => &mut self.fragment,
            RuntimeHelper::CreateElement => &mut self.create_element,
        };
        if let Some(name) = *slot {
            return name;
        }
        let name = uid.fresh(helper.local_base(), ast);
        *slot = Some(name);
        if helper != RuntimeHelper::CreateElement {
            self.helper_imports.push((helper.imported_name(), name));
        }
        name
    }
}

impl AutomaticScriptBindings<'_> {
    fn object<'ast>(
        &mut self,
        helper: RuntimeHelper,
        ast: &mut Ast<'ast>,
        uid: &mut LazyUidGenerator<'_, 'ast>,
    ) -> AstUtf8 {
        let (slot, base) = if helper == RuntimeHelper::CreateElement {
            (&mut self.react_object, "_react")
        } else {
            let base = if self.source.development {
                "_reactJsxDevRuntime"
            } else {
                "_reactJsxRuntime"
            };
            (&mut self.runtime_object, base)
        };
        if let Some(name) = *slot {
            return name;
        }
        let name = uid.fresh(base, ast);
        *slot = Some(name);
        name
    }
}

impl<'ast, const TRACK_SELF: bool> JsxEmitter<'_, '_, 'ast, TRACK_SELF> {
    pub(super) fn runtime_helper(&mut self, helper: RuntimeHelper, ast: &mut Ast<'ast>) -> Expr {
        self.state
            .bindings
            .reference(helper, ast, &mut self.state.uid_generator, &self.namespace)
    }
}

impl<'ast, const TRACK_SELF: bool> JsxLowerer<'_, 'ast, TRACK_SELF> {
    pub(super) fn insert_program_bindings(&mut self, program: Program, ast: &mut Ast<'ast>) {
        if self.bindings.is_classic() && self.filename.is_none() {
            return;
        }
        let mut bindings = self.bindings.statements(ast);
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
