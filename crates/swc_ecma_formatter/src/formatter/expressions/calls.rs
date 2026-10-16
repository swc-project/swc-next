//! Callees and source-owned argument delimiters.
use super::AccessKind;
use super::arguments::{ArgumentGroup, CallArgumentFrame, SourceArguments};
use super::parentheses::ExpressionContent;
use super::{
    CallArgumentOwner, ExpressionContext, ExpressionPosition, Formatter, JsxCallbackContext,
    MemberLayout, TailOwner,
};
use crate::formatter::comments::ExpressionTail;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::ArgumentData;
use swc_next_ecma_ast::{
    Argument, ArrowFunctionBodyData, ArrowFunctionExpression, CallExpression, Expr, ExprData,
    GetSpan, PropertyKeyData, Span, SubRange,
};
enum TestCallee {
    It,
    Describe,
    Test,
    TestDescribe,
    ParallelOrSerial,
    Terminal,
}

enum ModuleCallee {
    Define,
    Require,
    Resolve,
    ImportMeta,
    Terminal,
}

pub(super) struct CallCallee {
    pub(super) expression: Expr,
    pub(super) mode: ContentMode,
    pub(super) access: AccessKind,
    pub(super) curried_call: Option<CallExpression>,
}
impl CallCallee {
    pub(super) fn context(&self, context: ExpressionContext) -> ExpressionContext {
        let mut callee = ExpressionContext::at(ExpressionPosition::CallCallee {
            access: self.access,
            curried_call: self.curried_call,
        })
        .with_parent_head(context)
        .with_tail_owner(TailOwner::ParentOperation)
        .with_content_mode(self.mode);
        callee.member_chain_layout = context.member_chain_layout;
        callee.conditional_envelope = context.conditional_envelope.chain_head();
        callee
    }
}
impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn call_span_is_source_clear(&self, span: Span) -> bool {
        if self.diagnostic_range_is_sensitive(span) {
            return false;
        }
        let comments = self.ast.comments();
        let first = comments.partition_point(|comment| comment.span.end <= span.start);
        comments
            .get(first)
            .is_none_or(|comment| comment.span.start >= span.end)
    }

    fn call_member_callee_is_compact(&self, mut expression: Expr) -> bool {
        if !self.call_span_is_source_clear(expression.span(self.ast)) {
            return false;
        }
        loop {
            expression = match self.expression_content(expression).data {
                ExprData::MemberExpression(member) => {
                    let Ok(head) = self.prepare_member_head(member) else {
                        return false;
                    };
                    if !matches!(self.source_clear_member_has_chain_barrier(&head), Ok(false)) {
                        return false;
                    }
                    head.object
                }
                ExprData::TsNonNullExpression(node) => node.expression(self.ast),
                ExprData::ChainExpression(node) => node.expression(self.ast),
                data => {
                    return super::chains::atomic_chain_head(data)
                        || matches!(data, ExprData::Super(_));
                }
            };
        }
    }

    pub(super) fn source_call_callee(
        &self,
        call: CallExpression,
        context: ExpressionContext,
    ) -> Result<CallCallee, FormatError> {
        // Only suppression can change the callee's content mode.
        if self.has_suppression {
            return Ok(self.source_call(call, context)?.callee_head());
        }
        let ast = self.ast;
        let expression = call.callee(ast);
        let access = if call.optional(ast) {
            AccessKind::Optional
        } else {
            AccessKind::Ordinary
        };
        let mode = ContentMode::Formatted;
        let curried_call = self.curried_call_callee(expression, call.arguments(ast), mode);
        Ok(CallCallee {
            expression,
            access,
            mode,
            curried_call,
        })
    }
    /// The RHS spine owner has verified this call's suffix window.
    /// Suppression still uses the complete source argument owner.
    pub(super) fn rhs_call_callee(
        &self,
        call: CallExpression,
        context: ExpressionContext,
    ) -> Option<CallCallee> {
        let ast = self.ast;
        if self.has_suppression || call.type_arguments(ast).is_some() {
            let source = self.source_call(call, context).ok()?;
            if source.callee_mode != ContentMode::Formatted || source.has_outside_prefix() {
                return None;
            }
            return Some(source.callee_head());
        }
        let callee = self.source_call_callee(call, context).ok()?;
        self.source_clear_argument_opening(
            call.arguments(ast),
            callee.expression,
            call.span(ast).end,
            callee.access,
        )
        .then_some(callee)
    }

    pub(super) fn format_call(
        &mut self,
        call: CallExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let arguments = self.source_call(call, context)?;
        let mut callee_context = arguments.callee_context(context);
        let member_callee = arguments.callee_mode == ContentMode::Formatted
            && !arguments.has_fixed_argument_frame()
            && matches!(
                self.expression_content(arguments.callee).data,
                ExprData::MemberExpression(_)
            )
            && self.expression_needs_parentheses(
                self.ast.expr_data(arguments.callee),
                callee_context.position,
                arguments.callee_mode,
            ) != Some(true)
            && !arguments.has_outside_prefix();
        if member_callee && self.call_member_callee_is_compact(arguments.callee) {
            callee_context.member_layout = MemberLayout::Tight;
            callee_context.member_chain_layout = super::MemberChainLayout::Local;
        }
        let group = arguments.curried_callee.map(|_| arguments.group);
        let write = |writer: &mut Writer<'ast>| {
            let tail = self.format_expression(arguments.callee, callee_context, writer)?;
            self.format_source_arguments(arguments, tail, writer)
        };
        if member_callee {
            // The complete member call owns receiver frames even when its
            // receiver is not eligible for the tight, atomic-callee path.
            writer.group(false, write)
        } else {
            match group {
                Some(group) => group.write(false, writer, write),
                None => write(writer),
            }
        }
    }

    pub(super) fn source_call(
        &self,
        call: CallExpression,
        context: ExpressionContext,
    ) -> Result<SourceArguments<'ast>, FormatError> {
        let ast = self.ast;
        let callee = call.callee(ast);
        let call_arguments = call.arguments(ast);
        let access = if call.optional(ast) {
            AccessKind::Optional
        } else {
            AccessKind::Ordinary
        };
        let source = if let Some(node) = call.type_arguments(ast) {
            let types = self.prepare_expression_type_arguments(callee, node, access)?;
            self.source_arguments_after_types(call_arguments, callee, call.span(ast).end, types)
        } else {
            self.source_arguments(call_arguments, callee, call.span(ast).end, access)
        };
        let mut arguments =
            source.ok_or_else(|| self.unsupported("call opening boundary", call))?;
        arguments.allow_composition = !matches!(context.position, ExpressionPosition::Decorator);
        if let Some(frame) = self.test_call_frame(call, &arguments) {
            arguments.frame = frame;
        } else if matches!(context.call_argument_owner,
            CallArgumentOwner::AngularTestWrapper(owner) if owner == call)
            && !call.optional(ast)
            && let Some(frame) = self.angular_wrapper_frame(&arguments)
        {
            arguments.frame = frame;
        }
        arguments.jsx_call = match context.jsx_callback {
            JsxCallbackContext::ContainerCall(owner) if owner == call => Some(call),
            _ => None,
        };
        arguments.group = if matches!(context.position,
            ExpressionPosition::CallCallee { curried_call: Some(curried), .. } if curried == call)
        {
            ArgumentGroup::ParentCall
        } else {
            ArgumentGroup::Local
        };
        if !arguments.has_fixed_argument_frame()
            && let Some(frame) = self.module_call_frame(call, context, &arguments)
        {
            arguments.frame = frame;
        }
        if matches!(arguments.frame, CallArgumentFrame::Ordinary)
            && matches!(context.call_argument_owner,
                CallArgumentOwner::DecoratedFunction(owner) if owner == call)
            && arguments.callee_mode == ContentMode::Formatted
            && let Some(callback) = self.decorated_function_callback(call)
        {
            arguments.frame = CallArgumentFrame::DecoratedFunction(callback);
        }
        arguments.curried_callee =
            self.curried_call_callee(callee, call_arguments, arguments.callee_mode);
        Ok(arguments)
    }
    pub(in crate::formatter) fn call_callee_content_data(&self, callee: Expr) -> ExprData {
        match self.ast.expr_data(callee) {
            ExprData::ParenthesizedExpression(_) => self.expression_content(callee).data,
            data => data,
        }
    }

    pub(super) fn is_test_setup_callee(&self, expression: Expr) -> bool {
        let ast = self.ast;
        matches!(self.call_callee_content_data(expression), ExprData::IdentifierReference(identifier)
            if matches!(ast.get_utf8(identifier.name(ast)), "beforeEach" | "beforeAll" | "afterEach" | "afterAll"))
    }

    pub(super) fn is_test_callee(&self, expression: Expr) -> bool {
        self.test_callee(expression).is_some()
    }

    fn test_callee(&self, expression: Expr) -> Option<TestCallee> {
        let ast = self.ast;
        match self.call_callee_content_data(expression) {
            ExprData::IdentifierReference(identifier) => match ast.get_utf8(identifier.name(ast)) {
                "it" => Some(TestCallee::It),
                "describe" => Some(TestCallee::Describe),
                "test" => Some(TestCallee::Test),
                "skip" | "xit" | "xdescribe" | "xtest" | "fit" | "fdescribe" | "ftest" => {
                    Some(TestCallee::Terminal)
                }
                _ => None,
            },
            ExprData::MemberExpression(member)
                if !member.computed(ast) && !member.optional(ast) =>
            {
                let PropertyKeyData::IdentifierName(property) =
                    ast.property_key_data(member.property(ast))
                else {
                    return None;
                };
                let receiver = self.test_callee(member.object(ast))?;
                match (receiver, ast.get_utf8(property.name(ast))) {
                    (
                        TestCallee::It
                        | TestCallee::Describe
                        | TestCallee::Test
                        | TestCallee::TestDescribe,
                        "only" | "skip",
                    )
                    | (TestCallee::Test, "fixme" | "step")
                    | (TestCallee::TestDescribe, "fixme")
                    | (TestCallee::ParallelOrSerial, "only") => Some(TestCallee::Terminal),
                    (TestCallee::Test, "describe") => Some(TestCallee::TestDescribe),
                    (TestCallee::TestDescribe, "parallel" | "serial") => {
                        Some(TestCallee::ParallelOrSerial)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn module_call_frame(
        &self,
        call: CallExpression,
        context: ExpressionContext,
        list: &SourceArguments<'ast>,
    ) -> Option<CallArgumentFrame> {
        let ast = self.ast;
        if call.optional(ast) || list.callee_mode != ContentMode::Formatted {
            return None;
        }
        let module = self.module_callee(list.callee)?;
        if matches!(module, ModuleCallee::ImportMeta)
            || matches!(module, ModuleCallee::Define)
                && !matches!(context.position, ExpressionPosition::Statement)
        {
            return None;
        }
        let arguments = list.arguments();
        let mut items = arguments.iter().map(|id| ast.get_node_in_sub_range(id));
        let first = items.next()?;
        let data = match ast.argument_data(first) {
            ArgumentData::Expr(expression) => Some(self.expression_content(expression).data),
            ArgumentData::SpreadElement(_) => None,
        };
        let definition = matches!(module, ModuleCallee::Define);
        let shape = if definition {
            arguments.len() == 1
                || arguments.len() == 2 && matches!(data, Some(ExprData::ArrayExpression(_)))
                || arguments.len() == 3 && matches!(data, Some(ExprData::StringLiteral(_)))
                    && items.next().is_some_and(|argument| matches!(ast.argument_data(argument), ArgumentData::Expr(expression)
                        if matches!(self.expression_content(expression).data, ExprData::ArrayExpression(_))))
        } else {
            matches!(module, ModuleCallee::Require) && arguments.len() > 1
                || arguments.len() == 1 && matches!(data, Some(ExprData::StringLiteral(_)))
        };
        shape.then(|| self.module_argument_frame(list, definition))
    }

    pub(in crate::formatter) fn is_require_callee(&self, expression: Expr) -> bool {
        matches!(self.module_callee(expression), Some(ModuleCallee::Require))
    }

    fn module_callee(&self, expression: Expr) -> Option<ModuleCallee> {
        let ast = self.ast;
        match self.call_callee_content_data(expression) {
            ExprData::IdentifierReference(identifier) => match ast.get_utf8(identifier.name(ast)) {
                "require" => Some(ModuleCallee::Require),
                "define" => Some(ModuleCallee::Define),
                _ => None,
            },
            ExprData::MetaProperty(meta)
                if ast.get_utf8(meta.meta(ast).name(ast)) == "import"
                    && ast.get_utf8(meta.property(ast).name(ast)) == "meta" =>
            {
                Some(ModuleCallee::ImportMeta)
            }
            ExprData::MemberExpression(member)
                if !member.computed(ast) && !member.optional(ast) =>
            {
                let PropertyKeyData::IdentifierName(property) =
                    ast.property_key_data(member.property(ast))
                else {
                    return None;
                };
                let property = ast.get_utf8(property.name(ast));
                if !matches!(property, "resolve" | "paths") {
                    return None;
                }
                match (self.module_callee(member.object(ast))?, property) {
                    (ModuleCallee::Require, "resolve") => Some(ModuleCallee::Resolve),
                    (ModuleCallee::ImportMeta, "resolve") | (ModuleCallee::Resolve, "paths") => {
                        Some(ModuleCallee::Terminal)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(in crate::formatter) fn decorated_function_call(
        &self,
        expression: Expr,
    ) -> Option<CallExpression> {
        self.decorated_function_call_content(&self.expression_content(expression))
    }

    pub(in crate::formatter) fn decorated_function_call_content(
        &self,
        content: &ExpressionContent,
    ) -> Option<CallExpression> {
        let ExprData::CallExpression(call) = content.data else {
            return None;
        };
        self.decorated_function_callback(call)?;
        Some(call)
    }

    fn decorated_function_callback(&self, call: CallExpression) -> Option<ArrowFunctionExpression> {
        let ast = self.ast;
        let arguments = call.arguments(ast);
        if call.optional(ast) || arguments.len() != 1 {
            return None;
        }
        let argument = ast.get_node_in_sub_range(arguments.first()?);
        let ArgumentData::Expr(expression) = ast.argument_data(argument) else {
            return None;
        };
        let ExprData::ArrowFunctionExpression(arrow) = self.expression_content(expression).data
        else {
            return None;
        };
        if !matches!(
            ast.arrow_function_body_data(arrow.body(ast)),
            ArrowFunctionBodyData::FunctionBody(_)
        ) {
            return None;
        }
        let ExprData::CallExpression(factory) = self.call_callee_content_data(call.callee(ast))
        else {
            return None;
        };
        if factory.optional(ast) {
            return None;
        }
        match self.call_callee_content_data(factory.callee(ast)) {
            ExprData::IdentifierReference(_) => Some(arrow),
            ExprData::MemberExpression(member)
                if !member.computed(ast)
                    && !member.optional(ast)
                    && matches!(
                        self.call_callee_content_data(member.object(ast)),
                        ExprData::IdentifierReference(_)
                    )
                    && matches!(
                        ast.property_key_data(member.property(ast)),
                        PropertyKeyData::IdentifierName(_)
                    ) =>
            {
                Some(arrow)
            }
            _ => None,
        }
    }

    fn curried_call_callee(
        &self,
        callee: Expr,
        arguments: SubRange<Argument>,
        mode: ContentMode,
    ) -> Option<CallExpression> {
        if arguments.is_empty() || mode != ContentMode::Formatted {
            return None;
        }
        match self.expression_content(callee).data {
            ExprData::CallExpression(child)
                if child.arguments(self.ast).len() > arguments.len() =>
            {
                Some(child)
            }
            _ => None,
        }
    }
}
