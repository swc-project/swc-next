pub mod parser;
pub mod semantic;
pub mod transform_remove_paren;

use std::{
    collections::HashMap,
    panic::{AssertUnwindSafe, catch_unwind},
};

use swc_core::{
    common::{BytePos, GLOBALS, Globals, Spanned},
    ecma::{
        ast as legacy_ast,
        parser::{
            EsSyntax as LegacyEsSyntax, Parser as LegacyParser, StringInput,
            Syntax as LegacySyntax, lexer::Lexer,
        },
        visit::{Visit as LegacyVisit, VisitWith as LegacyVisitWith},
    },
};
use swc_experimental_ecma_ast as experimental_ast;
use swc_experimental_ecma_ast_compat::AstCompat;
use swc_experimental_ecma_parser::Syntax;
use swc_experimental_ecma_semantic::resolver::resolver;

use crate::cases::{Case, IsModule};

pub type NodeKind = &'static str;
pub type NodeSpans = HashMap<NodeKind, Vec<NodeSpan>>;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NodeSpan {
    start: u32,
    end: u32,
}

pub enum LegacyParseResult {
    Succ(legacy_ast::Program),
    Fail,
    Panic,
    Ignore,
}

pub fn parse_legacy<C: Case>(case: &C) -> LegacyParseResult {
    GLOBALS.set(&Globals::new(), || parse_legacy_with_current_globals(case))
}

pub fn parse_legacy_with_current_globals<C: Case>(case: &C) -> LegacyParseResult {
    let input = StringInput::new(
        case.code(),
        BytePos(1),
        BytePos(case.code().len() as u32 + 1),
    );
    let lexer = Lexer::new(
        legacy_syntax(case.syntax()),
        Default::default(),
        input,
        None,
    );
    let mut parser = LegacyParser::new_from(lexer);

    let ret = match case.is_module() {
        IsModule::Script => catch_unwind(AssertUnwindSafe(|| {
            parser.parse_script().map(legacy_ast::Program::Script)
        })),
        IsModule::Module => catch_unwind(AssertUnwindSafe(|| {
            parser.parse_module().map(legacy_ast::Program::Module)
        })),
        IsModule::Unknown => catch_unwind(AssertUnwindSafe(|| parser.parse_program())),
        IsModule::Skip => {
            return LegacyParseResult::Ignore;
        }
    };

    match ret {
        Ok(Ok(root)) => {
            if parser.take_errors().is_empty() {
                LegacyParseResult::Succ(root)
            } else {
                LegacyParseResult::Fail
            }
        }
        Ok(Err(_)) => LegacyParseResult::Fail,
        Err(_) => LegacyParseResult::Panic,
    }
}

pub fn legacy_syntax(syntax: Syntax) -> LegacySyntax {
    match syntax {
        Syntax::Es(es) => LegacySyntax::Es(LegacyEsSyntax {
            jsx: es.jsx,
            fn_bind: es.fn_bind,
            decorators: es.decorators,
            decorators_before_export: es.decorators_before_export,
            export_default_from: es.export_default_from,
            import_attributes: es.import_attributes,
            allow_super_outside_method: es.allow_super_outside_method,
            allow_return_outside_function: es.allow_return_outside_function,
            auto_accessors: es.auto_accessors,
            explicit_resource_management: es.explicit_resource_management,
        }),
    }
}

pub fn collect_legacy_node_spans(program: &legacy_ast::Program) -> NodeSpans {
    let mut collector = LegacyNodeSpansCollector::default();
    LegacyVisitWith::visit_with(program, &mut collector);
    sort_node_spans(&mut collector.nodes);
    collector.nodes
}

pub fn compat_experimental_program<'a>(
    program: experimental_ast::Program<'a>,
) -> legacy_ast::Program {
    let semantic = resolver(&program);
    AstCompat::new(&semantic).compat_program(program)
}

pub fn format_node_span_mismatch(
    title: &str,
    legacy_nodes: &NodeSpans,
    experimental_nodes: &NodeSpans,
) -> Option<String> {
    let mut mismatches = Vec::new();
    for kind in legacy_nodes.keys().chain(experimental_nodes.keys()) {
        let legacy_spans = legacy_nodes
            .get(kind)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let experimental_spans = experimental_nodes
            .get(kind)
            .map(Vec::as_slice)
            .unwrap_or_default();
        if legacy_spans != experimental_spans {
            mismatches.push(*kind);
        }
    }
    mismatches.sort_unstable();
    mismatches.dedup();

    if mismatches.is_empty() {
        return None;
    }

    let mut error = title.to_string();
    for kind in mismatches.iter().take(30) {
        let legacy_sample = span_sample(legacy_nodes.get(kind));
        let experimental_sample = span_sample(experimental_nodes.get(kind));
        error.push_str(&format!(
            "\n  {kind}: swc_core {legacy_sample}, swc_experimental {experimental_sample}"
        ));
    }
    if mismatches.len() > 30 {
        error.push_str(&format!(
            "\n  ... and {} more NodeKind span mismatches",
            mismatches.len() - 30
        ));
    }

    Some(error)
}

fn sort_node_spans(nodes: &mut NodeSpans) {
    for spans in nodes.values_mut() {
        spans.sort_unstable();
    }
}

fn span_sample(spans: Option<&Vec<NodeSpan>>) -> String {
    let Some(spans) = spans else {
        return "spans=[]".to_string();
    };

    let mut ret = String::from("spans=[");
    for (index, span) in spans.iter().take(8).enumerate() {
        if index > 0 {
            ret.push_str(", ");
        }
        ret.push_str(&format!("{}..{}", span.start, span.end));
    }
    if spans.len() > 8 {
        ret.push_str(&format!(", ... +{}", spans.len() - 8));
    }
    ret.push(']');
    ret
}

fn legacy_node_span<T: Spanned>(node: &T) -> NodeSpan {
    let span = node.span();
    NodeSpan {
        start: span.lo.0,
        end: span.hi.0,
    }
}

#[derive(Default)]
struct LegacyNodeSpansCollector {
    nodes: NodeSpans,
}

impl LegacyNodeSpansCollector {
    fn push(&mut self, kind: NodeKind, span: NodeSpan) {
        self.nodes.entry(kind).or_default().push(span);
    }
}

macro_rules! node_span_collector_methods {
    ($(($method:ident, $kind:literal, $legacy_ty:ident, $experimental_ty:ty)),* $(,)?) => {
        impl LegacyVisit for LegacyNodeSpansCollector {
            $(
                fn $method(&mut self, node: &legacy_ast::$legacy_ty) {
                    self.push($kind, legacy_node_span(node));
                    LegacyVisitWith::visit_children_with(node, self);
                }
            )*
        }
    };
}

node_span_collector_methods! {
    (visit_program, "Program", Program, experimental_ast::Program<'a>),
    (visit_module, "Module", Module, experimental_ast::Module<'a>),
    (visit_script, "Script", Script, experimental_ast::Script<'a>),
    (visit_module_item, "ModuleItem", ModuleItem, experimental_ast::ModuleItem<'a>),
    (visit_module_decl, "ModuleDecl", ModuleDecl, experimental_ast::ModuleDecl<'a>),
    (visit_import_decl, "ImportDecl", ImportDecl, experimental_ast::ImportDecl<'a>),
    (visit_import_specifier, "ImportSpecifier", ImportSpecifier, experimental_ast::ImportSpecifier<'a>),
    (visit_import_named_specifier, "ImportNamedSpecifier", ImportNamedSpecifier, experimental_ast::ImportNamedSpecifier<'a>),
    (visit_import_default_specifier, "ImportDefaultSpecifier", ImportDefaultSpecifier, experimental_ast::ImportDefaultSpecifier<'a>),
    (visit_import_star_as_specifier, "ImportStarAsSpecifier", ImportStarAsSpecifier, experimental_ast::ImportStarAsSpecifier<'a>),
    (visit_export_decl, "ExportDecl", ExportDecl, experimental_ast::ExportDecl<'a>),
    (visit_named_export, "NamedExport", NamedExport, experimental_ast::NamedExport<'a>),
    (visit_export_specifier, "ExportSpecifier", ExportSpecifier, experimental_ast::ExportSpecifier<'a>),
    (visit_export_namespace_specifier, "ExportNamespaceSpecifier", ExportNamespaceSpecifier, experimental_ast::ExportNamespaceSpecifier<'a>),
    (visit_module_export_name, "ModuleExportName", ModuleExportName, experimental_ast::ModuleExportName<'a>),
    (visit_export_default_specifier, "ExportDefaultSpecifier", ExportDefaultSpecifier, experimental_ast::ExportDefaultSpecifier<'a>),
    (visit_export_named_specifier, "ExportNamedSpecifier", ExportNamedSpecifier, experimental_ast::ExportNamedSpecifier<'a>),
    (visit_export_default_decl, "ExportDefaultDecl", ExportDefaultDecl, experimental_ast::ExportDefaultDecl<'a>),
    (visit_default_decl, "DefaultDecl", DefaultDecl, experimental_ast::DefaultDecl<'a>),
    (visit_export_default_expr, "ExportDefaultExpr", ExportDefaultExpr, experimental_ast::ExportDefaultExpr<'a>),
    (visit_export_all, "ExportAll", ExportAll, experimental_ast::ExportAll<'a>),
    (visit_block_stmt, "BlockStmt", BlockStmt, experimental_ast::BlockStmt<'a>),
    (visit_stmt, "Stmt", Stmt, experimental_ast::Stmt<'a>),
    (visit_expr_stmt, "ExprStmt", ExprStmt, experimental_ast::ExprStmt<'a>),
    (visit_empty_stmt, "EmptyStmt", EmptyStmt, experimental_ast::EmptyStmt),
    (visit_debugger_stmt, "DebuggerStmt", DebuggerStmt, experimental_ast::DebuggerStmt),
    (visit_with_stmt, "WithStmt", WithStmt, experimental_ast::WithStmt<'a>),
    (visit_return_stmt, "ReturnStmt", ReturnStmt, experimental_ast::ReturnStmt<'a>),
    (visit_labeled_stmt, "LabeledStmt", LabeledStmt, experimental_ast::LabeledStmt<'a>),
    (visit_break_stmt, "BreakStmt", BreakStmt, experimental_ast::BreakStmt<'a>),
    (visit_continue_stmt, "ContinueStmt", ContinueStmt, experimental_ast::ContinueStmt<'a>),
    (visit_if_stmt, "IfStmt", IfStmt, experimental_ast::IfStmt<'a>),
    (visit_switch_stmt, "SwitchStmt", SwitchStmt, experimental_ast::SwitchStmt<'a>),
    (visit_throw_stmt, "ThrowStmt", ThrowStmt, experimental_ast::ThrowStmt<'a>),
    (visit_try_stmt, "TryStmt", TryStmt, experimental_ast::TryStmt<'a>),
    (visit_while_stmt, "WhileStmt", WhileStmt, experimental_ast::WhileStmt<'a>),
    (visit_do_while_stmt, "DoWhileStmt", DoWhileStmt, experimental_ast::DoWhileStmt<'a>),
    (visit_for_stmt, "ForStmt", ForStmt, experimental_ast::ForStmt<'a>),
    (visit_for_in_stmt, "ForInStmt", ForInStmt, experimental_ast::ForInStmt<'a>),
    (visit_for_of_stmt, "ForOfStmt", ForOfStmt, experimental_ast::ForOfStmt<'a>),
    (visit_switch_case, "SwitchCase", SwitchCase, experimental_ast::SwitchCase<'a>),
    (visit_catch_clause, "CatchClause", CatchClause, experimental_ast::CatchClause<'a>),
    (visit_for_head, "ForHead", ForHead, experimental_ast::ForHead<'a>),
    (visit_var_decl_or_expr, "VarDeclOrExpr", VarDeclOrExpr, experimental_ast::VarDeclOrExpr<'a>),
    (visit_decl, "Decl", Decl, experimental_ast::Decl<'a>),
    (visit_fn_decl, "FnDecl", FnDecl, experimental_ast::FnDecl<'a>),
    (visit_class_decl, "ClassDecl", ClassDecl, experimental_ast::ClassDecl<'a>),
    (visit_var_decl, "VarDecl", VarDecl, experimental_ast::VarDecl<'a>),
    (visit_var_declarator, "VarDeclarator", VarDeclarator, experimental_ast::VarDeclarator<'a>),
    (visit_using_decl, "UsingDecl", UsingDecl, experimental_ast::UsingDecl<'a>),
    (visit_expr, "Expr", Expr, experimental_ast::Expr<'a>),
    (visit_this_expr, "ThisExpr", ThisExpr, experimental_ast::ThisExpr),
    (visit_array_lit, "ArrayLit", ArrayLit, experimental_ast::ArrayLit<'a>),
    (visit_object_lit, "ObjectLit", ObjectLit, experimental_ast::ObjectLit<'a>),
    (visit_prop_or_spread, "PropOrSpread", PropOrSpread, experimental_ast::PropOrSpread<'a>),
    (visit_spread_element, "SpreadElement", SpreadElement, experimental_ast::SpreadElement<'a>),
    (visit_unary_expr, "UnaryExpr", UnaryExpr, experimental_ast::UnaryExpr<'a>),
    (visit_update_expr, "UpdateExpr", UpdateExpr, experimental_ast::UpdateExpr<'a>),
    (visit_bin_expr, "BinExpr", BinExpr, experimental_ast::BinExpr<'a>),
    (visit_fn_expr, "FnExpr", FnExpr, experimental_ast::FnExpr<'a>),
    (visit_class_expr, "ClassExpr", ClassExpr, experimental_ast::ClassExpr<'a>),
    (visit_assign_expr, "AssignExpr", AssignExpr, experimental_ast::AssignExpr<'a>),
    (visit_member_expr, "MemberExpr", MemberExpr, experimental_ast::MemberExpr<'a>),
    (visit_member_prop, "MemberProp", MemberProp, experimental_ast::MemberProp<'a>),
    (visit_super_prop_expr, "SuperPropExpr", SuperPropExpr, experimental_ast::SuperPropExpr<'a>),
    (visit_super_prop, "SuperProp", SuperProp, experimental_ast::SuperProp<'a>),
    (visit_cond_expr, "CondExpr", CondExpr, experimental_ast::CondExpr<'a>),
    (visit_call_expr, "CallExpr", CallExpr, experimental_ast::CallExpr<'a>),
    (visit_new_expr, "NewExpr", NewExpr, experimental_ast::NewExpr<'a>),
    (visit_seq_expr, "SeqExpr", SeqExpr, experimental_ast::SeqExpr<'a>),
    (visit_arrow_expr, "ArrowExpr", ArrowExpr, experimental_ast::ArrowExpr<'a>),
    (visit_yield_expr, "YieldExpr", YieldExpr, experimental_ast::YieldExpr<'a>),
    (visit_meta_prop_expr, "MetaPropExpr", MetaPropExpr, experimental_ast::MetaPropExpr),
    (visit_await_expr, "AwaitExpr", AwaitExpr, experimental_ast::AwaitExpr<'a>),
    (visit_tpl, "Tpl", Tpl, experimental_ast::Tpl<'a>),
    (visit_tagged_tpl, "TaggedTpl", TaggedTpl, experimental_ast::TaggedTpl<'a>),
    (visit_tpl_element, "TplElement", TplElement, experimental_ast::TplElement<'a>),
    (visit_paren_expr, "ParenExpr", ParenExpr, experimental_ast::ParenExpr<'a>),
    (visit_callee, "Callee", Callee, experimental_ast::Callee<'a>),
    (visit_super, "Super", Super, experimental_ast::Super),
    (visit_import, "Import", Import, experimental_ast::Import),
    (visit_expr_or_spread, "ExprOrSpread", ExprOrSpread, experimental_ast::ExprOrSpread<'a>),
    (visit_block_stmt_or_expr, "BlockStmtOrExpr", BlockStmtOrExpr, experimental_ast::BlockStmtOrExpr<'a>),
    (visit_assign_target, "AssignTarget", AssignTarget, experimental_ast::AssignTarget<'a>),
    (visit_assign_target_pat, "AssignTargetPat", AssignTargetPat, experimental_ast::AssignTargetPat<'a>),
    (visit_simple_assign_target, "SimpleAssignTarget", SimpleAssignTarget, experimental_ast::SimpleAssignTarget<'a>),
    (visit_opt_chain_expr, "OptChainExpr", OptChainExpr, experimental_ast::OptChainExpr<'a>),
    (visit_opt_chain_base, "OptChainBase", OptChainBase, experimental_ast::OptChainBase<'a>),
    (visit_opt_call, "OptCall", OptCall, experimental_ast::OptCall<'a>),
    (visit_invalid, "Invalid", Invalid, experimental_ast::Invalid),
    (visit_function, "Function", Function, experimental_ast::Function<'a>),
    (visit_param, "Param", Param, experimental_ast::Param<'a>),
    (visit_param_or_ts_param_prop, "ParamOrTsParamProp", ParamOrTsParamProp, experimental_ast::ParamOrTsParamProp<'a>),
    (visit_class, "Class", Class, experimental_ast::Class<'a>),
    (visit_class_member, "ClassMember", ClassMember, experimental_ast::ClassMember<'a>),
    (visit_class_prop, "ClassProp", ClassProp, experimental_ast::ClassProp<'a>),
    (visit_private_prop, "PrivateProp", PrivateProp, experimental_ast::PrivateProp<'a>),
    (visit_class_method, "ClassMethod", ClassMethod, experimental_ast::ClassMethod<'a>),
    (visit_private_method, "PrivateMethod", PrivateMethod, experimental_ast::PrivateMethod<'a>),
    (visit_constructor, "Constructor", Constructor, experimental_ast::Constructor<'a>),
    (visit_decorator, "Decorator", Decorator, experimental_ast::Decorator<'a>),
    (visit_static_block, "StaticBlock", StaticBlock, experimental_ast::StaticBlock<'a>),
    (visit_key, "Key", Key, experimental_ast::Key<'a>),
    (visit_auto_accessor, "AutoAccessor", AutoAccessor, experimental_ast::AutoAccessor<'a>),
    (visit_prop, "Prop", Prop, experimental_ast::Prop<'a>),
    (visit_key_value_prop, "KeyValueProp", KeyValueProp, experimental_ast::KeyValueProp<'a>),
    (visit_assign_prop, "AssignProp", AssignProp, experimental_ast::AssignProp<'a>),
    (visit_getter_prop, "GetterProp", GetterProp, experimental_ast::GetterProp<'a>),
    (visit_setter_prop, "SetterProp", SetterProp, experimental_ast::SetterProp<'a>),
    (visit_method_prop, "MethodProp", MethodProp, experimental_ast::MethodProp<'a>),
    (visit_prop_name, "PropName", PropName, experimental_ast::PropName<'a>),
    (visit_computed_prop_name, "ComputedPropName", ComputedPropName, experimental_ast::ComputedPropName<'a>),
    (visit_pat, "Pat", Pat, experimental_ast::Pat<'a>),
    (visit_array_pat, "ArrayPat", ArrayPat, experimental_ast::ArrayPat<'a>),
    (visit_object_pat, "ObjectPat", ObjectPat, experimental_ast::ObjectPat<'a>),
    (visit_assign_pat, "AssignPat", AssignPat, experimental_ast::AssignPat<'a>),
    (visit_rest_pat, "RestPat", RestPat, experimental_ast::RestPat<'a>),
    (visit_object_pat_prop, "ObjectPatProp", ObjectPatProp, experimental_ast::ObjectPatProp<'a>),
    (visit_key_value_pat_prop, "KeyValuePatProp", KeyValuePatProp, experimental_ast::KeyValuePatProp<'a>),
    (visit_assign_pat_prop, "AssignPatProp", AssignPatProp, experimental_ast::AssignPatProp<'a>),
    (visit_ident, "Ident", Ident, experimental_ast::Ident<'a>),
    (visit_ident_name, "IdentName", IdentName, experimental_ast::IdentName<'a>),
    (visit_private_name, "PrivateName", PrivateName, experimental_ast::PrivateName<'a>),
    (visit_binding_ident, "BindingIdent", BindingIdent, experimental_ast::BindingIdent<'a>),
    (visit_lit, "Lit", Lit, experimental_ast::Lit<'a>),
    (visit_str, "Str", Str, experimental_ast::Str<'a>),
    (visit_bool, "Bool", Bool, experimental_ast::Bool),
    (visit_null, "Null", Null, experimental_ast::Null),
    (visit_number, "Number", Number, experimental_ast::Number<'a>),
    (visit_big_int, "BigInt", BigInt, experimental_ast::BigInt<'a>),
    (visit_regex, "Regex", Regex, experimental_ast::Regex<'a>),
    (visit_jsx_object, "JSXObject", JSXObject, experimental_ast::JSXObject<'a>),
    (visit_jsx_member_expr, "JSXMemberExpr", JSXMemberExpr, experimental_ast::JSXMemberExpr<'a>),
    (visit_jsx_namespaced_name, "JSXNamespacedName", JSXNamespacedName, experimental_ast::JSXNamespacedName<'a>),
    (visit_jsx_empty_expr, "JSXEmptyExpr", JSXEmptyExpr, experimental_ast::JSXEmptyExpr),
    (visit_jsx_expr_container, "JSXExprContainer", JSXExprContainer, experimental_ast::JSXExprContainer<'a>),
    (visit_jsx_expr, "JSXExpr", JSXExpr, experimental_ast::JSXExpr<'a>),
    (visit_jsx_spread_child, "JSXSpreadChild", JSXSpreadChild, experimental_ast::JSXSpreadChild<'a>),
    (visit_jsx_element_name, "JSXElementName", JSXElementName, experimental_ast::JSXElementName<'a>),
    (visit_jsx_opening_element, "JSXOpeningElement", JSXOpeningElement, experimental_ast::JSXOpeningElement<'a>),
    (visit_jsx_attr_or_spread, "JSXAttrOrSpread", JSXAttrOrSpread, experimental_ast::JSXAttrOrSpread<'a>),
    (visit_jsx_closing_element, "JSXClosingElement", JSXClosingElement, experimental_ast::JSXClosingElement<'a>),
    (visit_jsx_attr, "JSXAttr", JSXAttr, experimental_ast::JSXAttr<'a>),
    (visit_jsx_attr_name, "JSXAttrName", JSXAttrName, experimental_ast::JSXAttrName<'a>),
    (visit_jsx_attr_value, "JSXAttrValue", JSXAttrValue, experimental_ast::JSXAttrValue<'a>),
    (visit_jsx_text, "JSXText", JSXText, experimental_ast::JSXText<'a>),
    (visit_jsx_element, "JSXElement", JSXElement, experimental_ast::JSXElement<'a>),
    (visit_jsx_element_child, "JSXElementChild", JSXElementChild, experimental_ast::JSXElementChild<'a>),
    (visit_jsx_fragment, "JSXFragment", JSXFragment, experimental_ast::JSXFragment<'a>),
    (visit_jsx_opening_fragment, "JSXOpeningFragment", JSXOpeningFragment, experimental_ast::JSXOpeningFragment),
    (visit_jsx_closing_fragment, "JSXClosingFragment", JSXClosingFragment, experimental_ast::JSXClosingFragment),
}
