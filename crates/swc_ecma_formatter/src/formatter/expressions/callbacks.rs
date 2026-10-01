//! Callback argument selection and immutable layouts over source-owned slots.
use super::arguments::ArgumentHead;
use super::arrays::{ArrayArgumentArity, RecordedArray, RecordedArrayArgument};
use super::arrows::{ArrowBodyLayout, PreparedArrowBody, RecordedArrow};
use super::objects::RecordedObject;
use super::{ExpressionContext, ExpressionLayout, ExpressionPosition, Formatter};
use crate::formatter::comments::ExpressionTail;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::parameters::CallbackParameterShape;
use crate::formatter::suppression::ContentMode;
use crate::{
    FormatError, TrailingCommas,
    document::{FlatFragment, Fragment, Writer},
};
use swc_next_ecma_ast::{
    Argument, ArgumentData, ArrayExpression, ArrowFunctionExpression, Expr, ExprData, Function,
    GetSpan, ObjectExpression, ObjectPropertyKindData, PropertyKeyData, SimpleAssignmentTargetData,
    Span, SubRange, UnaryOperator,
};
#[derive(Clone, Copy)]
enum CallbackKind {
    Arrow(ArrowFunctionExpression),
    Function(Function),
}

#[derive(Clone, Copy)]
struct Callback {
    expression: Expr,
    kind: CallbackKind,
}

pub(super) struct SelectedCallback {
    argument: Argument,
    callback: Callback,
    prepared_body: Option<PreparedArrowBody>,
}
pub(super) struct SelectedObject {
    argument: Argument,
    expression: Expr,
    object: ObjectExpression,
}

pub(super) struct SelectedArray {
    argument: Argument,
    expression: Expr,
    array: ArrayExpression,
    arity: ArrayArgumentArity,
}

pub(super) enum SelectedArgument {
    Callback(SelectedCallback),
    FinalObject(SelectedObject),
    FinalArray(SelectedArray),
}

#[derive(Clone, Copy)]
enum GroupingPolicy {
    Callback,
    FinalObject,
    FinalArray,
}

enum SelectionOutcome {
    Callback,
    FinalObject { has_forced: bool },
    FinalArray { has_forced: bool },
    Ordinary,
}
enum RecordedValue<'a> {
    Ordinary(Fragment<'a>),
    FinalObject(RecordedObject<'a>),
    FinalArray(RecordedArray<'a>),
    Callback {
        signature: Option<GroupedSignature<'a>>,
        body: CallbackBody<'a>,
        normal: Fragment<'a>,
        layout: ExpressionLayout,
    },
}

#[derive(Clone, Copy)]
enum CallbackBody<'a> {
    /// Keep the existing body group or source hug layout.
    Retained(Fragment<'a>),
    /// The expression body and the actual terminal argument boundary share a group.
    Terminal(Fragment<'a>),
}

struct RecordedArgument<'a> {
    prefix: Fragment<'a>,
    value: RecordedValue<'a>,
    suffix: Fragment<'a>,
}

#[derive(Clone, Copy)]
enum GroupedSignature<'a> {
    Independent(Fragment<'a>),
    Flat(FlatFragment<'a>),
}

enum GroupedValue<'a> {
    Ordinary(FlatFragment<'a>),
    FinalObject(RecordedObject<'a>),
    FinalArray(RecordedArray<'a>),
    Callback {
        signature: GroupedSignature<'a>,
        body: CallbackBody<'a>,
        suffix: Fragment<'a>,
    },
}

struct GroupedArgument<'a> {
    prefix: FlatFragment<'a>,
    value: GroupedValue<'a>,
    suffix: FlatFragment<'a>,
}

impl<'a> RecordedArgument<'a> {
    fn try_group(&self, policy: GroupingPolicy) -> Option<GroupedArgument<'a>> {
        let prefix = grouping_view(self.prefix, policy)?;
        let suffix = grouping_view(self.suffix, policy)?;
        let value = match self.value {
            RecordedValue::Ordinary(value) => {
                // A neighboring value retains its own source layout preference.
                if value.preferred_breaks() {
                    return None;
                }
                GroupedValue::Ordinary(grouping_view(value, policy)?)
            }
            RecordedValue::FinalObject(value) => GroupedValue::FinalObject(value),
            RecordedValue::FinalArray(value) => GroupedValue::FinalArray(value),
            RecordedValue::Callback {
                signature,
                body,
                normal,
                layout,
            } => {
                if layout == ExpressionLayout::Flat
                    && let Some(normal) = normal.try_flat()
                {
                    GroupedValue::Ordinary(normal)
                } else {
                    GroupedValue::Callback {
                        signature: signature?,
                        body,
                        suffix: self.suffix,
                    }
                }
            }
        };
        Some(GroupedArgument {
            prefix,
            value,
            suffix,
        })
    }
}

fn grouping_view(fragment: Fragment<'_>, policy: GroupingPolicy) -> Option<FlatFragment<'_>> {
    if matches!(
        policy,
        GroupingPolicy::FinalObject | GroupingPolicy::FinalArray
    ) && fragment.preferred_breaks()
    {
        return None;
    }
    fragment.try_flat()
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    fn callback(&self, argument: Argument) -> Option<Callback> {
        self.callback_data(self.ast.argument_data(argument))
    }

    fn callback_data(&self, data: ArgumentData) -> Option<Callback> {
        let ArgumentData::Expr(expression) = data else {
            return None;
        };
        let content = self.expression_content(expression);
        if let Some(boundary) = content.parentheses.map(|removed| removed.source)
            && (!self
                .comments
                .in_range(Span::new(boundary.outer.start, boundary.inner.start))
                .is_empty()
                || !self
                    .comments
                    .in_range(Span::new(boundary.inner.end, boundary.outer.end))
                    .is_empty())
        {
            return None;
        }
        let kind = match content.data {
            ExprData::ArrowFunctionExpression(arrow) => CallbackKind::Arrow(arrow),
            ExprData::Function(function) => CallbackKind::Function(function),
            _ => return None,
        };
        Some(Callback {
            expression: content.expression,
            kind,
        })
    }

    pub(super) fn select_argument_layout(
        &self,
        arguments: SubRange<Argument>,
        allow_final_collection: bool,
    ) -> Option<SelectedArgument> {
        let ast = self.ast;
        let mut items = arguments.iter().map(|id| ast.get_node_in_sub_range(id));
        let argument = items.next_back()?;
        let data = ast.argument_data(argument);
        let ArgumentData::Expr(expression) = data else {
            return None;
        };
        let callback = match ast.expr_data(expression) {
            ExprData::ArrowFunctionExpression(arrow) => Callback {
                expression,
                kind: CallbackKind::Arrow(arrow),
            },
            ExprData::Function(function) => Callback {
                expression,
                kind: CallbackKind::Function(function),
            },
            ExprData::ParenthesizedExpression(_) => self.callback_data(data)?,
            ExprData::ObjectExpression(object) if allow_final_collection => {
                if let Some(previous) = items.next_back()
                    && matches!(ast.argument_data(previous), ArgumentData::Expr(expression)
                        if matches!(ast.expr_data(expression), ExprData::ObjectExpression(_)))
                {
                    return None;
                }
                let span = object.span(ast);
                if object.properties(ast).is_empty()
                    && self
                        .comments
                        .in_range(Span::new(span.start + 1, span.end - 1))
                        .is_empty()
                {
                    return None;
                }
                return Some(SelectedArgument::FinalObject(SelectedObject {
                    argument,
                    expression,
                    object,
                }));
            }
            ExprData::ArrayExpression(array) if allow_final_collection => {
                if let Some(previous) = items.next_back()
                    && let ArgumentData::Expr(expression) = ast.argument_data(previous)
                {
                    let previous = ast.expr_data(expression);
                    if matches!(previous, ExprData::ArrayExpression(_))
                        || arguments.len() == 2
                            && matches!(previous, ExprData::ArrowFunctionExpression(_))
                    {
                        return None;
                    }
                }
                let span = array.span(ast);
                if array.elements(ast).is_empty()
                    && self
                        .comments
                        .in_range(Span::new(span.start + 1, span.end - 1))
                        .is_empty()
                {
                    return None;
                }
                return Some(SelectedArgument::FinalArray(SelectedArray {
                    argument,
                    expression,
                    array,
                    arity: if arguments.len() == 1 {
                        ArrayArgumentArity::Single
                    } else {
                        ArrayArgumentArity::Multiple
                    },
                }));
            }
            _ => return None,
        };
        if let Some(previous) = items.next_back()
            && let Some(previous) = self.callback(previous)
            && std::mem::discriminant(&previous.kind) == std::mem::discriminant(&callback.kind)
        {
            return None;
        }
        let prepared_body = match callback.kind {
            CallbackKind::Function(_) => None,
            CallbackKind::Arrow(arrow) => Some(self.callback_can_expand(arrow)?),
        };
        Some(SelectedArgument::Callback(SelectedCallback {
            argument,
            callback,
            prepared_body,
        }))
    }
    fn callback_can_expand(&self, arrow: ArrowFunctionExpression) -> Option<PreparedArrowBody> {
        let root = self.prepare_arrow_body(arrow.body(self.ast));
        let mut child = None;
        loop {
            let body = child.as_ref().unwrap_or(&root);
            let content = match body {
                PreparedArrowBody::FunctionBody(_) => return Some(root),
                PreparedArrowBody::Expression { content, .. } => content,
            };
            match content.data {
                ExprData::ObjectExpression(_) | ExprData::ArrayExpression(_) => return Some(root),
                ExprData::ArrowFunctionExpression(arrow) => {
                    child = Some(self.prepare_arrow_body(arrow.body(self.ast)));
                }
                ExprData::ConditionalExpression(_) | ExprData::CallExpression(_) => {
                    return child.is_none().then_some(root);
                }
                ExprData::ChainExpression(_)
                    if matches!(
                        self.chain_element_data(content.data),
                        ExprData::CallExpression(_)
                    ) =>
                {
                    return child.is_none().then_some(root);
                }
                _ => return None,
            }
        }
    }

    fn record_callback(
        &mut self,
        selected: &mut SelectedCallback,
        sole_argument: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(RecordedValue<'ast>, Option<ExpressionTail<'ast>>), FormatError> {
        let callback = selected.callback;
        let layout = self.expression_layout(callback.expression);
        let (signature, body, normal, tail, independent) = match callback.kind {
            CallbackKind::Arrow(arrow) => {
                let source = self.source_arrow(arrow)?;
                let context = ExpressionContext::at(ExpressionPosition::Nested)
                    .with_arrow_layout(super::arrows::ArrowLayout::Argument);
                let recorded = match self.record_arrow(
                    source,
                    selected.prepared_body.take(),
                    context,
                    writer,
                )? {
                    RecordedArrow::Verbatim(value) => {
                        let (value, ()) = writer.record_with_value(|writer| {
                            layout.write(writer, |writer| writer.write_fragment(value));
                            Ok::<_, FormatError>(())
                        })?;
                        return Ok((RecordedValue::Ordinary(value), None));
                    }
                    RecordedArrow::Formatted(recorded) => recorded,
                };
                (
                    recorded.signature,
                    if recorded.body_layout == ArrowBodyLayout::Expression {
                        CallbackBody::Terminal(recorded.body.contents())
                    } else {
                        CallbackBody::Retained(if recorded.body_layout == ArrowBodyLayout::Hug {
                            recorded.body.contents()
                        } else {
                            recorded.body.grouped()
                        })
                    },
                    recorded.normal,
                    recorded.tail,
                    recorded.independent_signature,
                )
            }
            CallbackKind::Function(function) => {
                let source = self.source_function_signature(function)?;
                if source.enclosing_mode() == ContentMode::Verbatim {
                    let (value, ()) = writer.record_with_value(|writer| {
                        layout.write(writer, |writer| {
                            self.format_verbatim(function.span(self.ast), writer)
                        })
                    })?;
                    return Ok((RecordedValue::Ordinary(value), None));
                }
                let independent = !source.has_parameters();
                let mut shape = (sole_argument && layout == ExpressionLayout::Normal)
                    .then_some(CallbackParameterShape::Identifiers);
                let (signature, body) = writer.record_with_value(|writer| {
                    self.format_function_head_with_shape(function, source, shape.as_mut(), writer)
                })?;
                let (body, ()) = writer
                    .record_with_value(|writer| self.format_function_body_boundary(body, writer))?;
                let (normal, ()) = writer.record_with_value(|writer| {
                    writer.write_fragment(signature);
                    writer.write_fragment(body);
                    Ok::<_, FormatError>(())
                })?;
                (
                    signature,
                    CallbackBody::Retained(body),
                    normal,
                    None,
                    independent || shape == Some(CallbackParameterShape::Other),
                )
            }
        };
        let (normal, ()) = writer.record_with_value(|writer| {
            layout.write(writer, |writer| writer.write_fragment(normal));
            Ok::<_, FormatError>(())
        })?;
        let signature = if independent {
            Some(GroupedSignature::Independent(signature))
        } else {
            signature.try_flat().map(GroupedSignature::Flat)
        };
        Ok((
            RecordedValue::Callback {
                signature,
                body,
                normal,
                layout,
            },
            tail,
        ))
    }

    fn simple_callback_neighbor(&self, expression: Expr, depth: u8) -> bool {
        if depth == 0 {
            return false;
        }
        let data = self.expression_content(expression).data;
        self.simple_callback_neighbor_data(data, depth)
    }

    pub(super) fn simple_call_argument(&self, argument: Argument, depth: u8) -> bool {
        let ArgumentData::Expr(expression) = self.ast.argument_data(argument) else {
            return false;
        };
        self.simple_callback_neighbor(expression, depth)
    }

    fn simple_callback_neighbor_data(&self, mut data: ExprData, depth: u8) -> bool {
        if depth == 0 {
            return false;
        }
        let ast = self.ast;
        let child_depth = depth - 1;
        loop {
            let next = match data {
                ExprData::IdentifierReference(_)
                | ExprData::ThisExpression(_)
                | ExprData::StringLiteral(_)
                | ExprData::NumericLiteral(_)
                | ExprData::BooleanLiteral(_)
                | ExprData::NullLiteral(_)
                | ExprData::BigIntLiteral(_) => return true,
                ExprData::RegExpLiteral(literal) => {
                    return crate::text_width::width(ast.get_utf8(literal.pattern(ast))) <= 5;
                }
                ExprData::ChainExpression(chain) => chain.expression(ast),
                ExprData::UnaryExpression(unary)
                    if matches!(
                        unary.operator(ast),
                        UnaryOperator::LogicalNot
                            | UnaryOperator::Negate
                            | UnaryOperator::Positive
                            | UnaryOperator::BitwiseNot
                    ) =>
                {
                    unary.argument(ast)
                }
                ExprData::UpdateExpression(update) => {
                    match ast.simple_assignment_target_data(update.argument(ast)) {
                        SimpleAssignmentTargetData::IdentifierReference(_) => return true,
                        SimpleAssignmentTargetData::MemberExpression(member) => {
                            Expr::MemberExpression(member)
                        }
                        _ => return false,
                    }
                }
                ExprData::MemberExpression(member) => {
                    if let PropertyKeyData::Expr(property) =
                        ast.property_key_data(member.property(ast))
                        && !self.simple_callback_neighbor(property, depth)
                    {
                        return false;
                    }
                    member.object(ast)
                }
                ExprData::CallExpression(call) => {
                    let arguments = call.arguments(ast);
                    if arguments.len() > usize::from(depth)
                        || !arguments.iter().all(|id| {
                            self.simple_call_argument(ast.get_node_in_sub_range(id), child_depth)
                        })
                    {
                        return false;
                    }
                    call.callee(ast)
                }
                ExprData::NewExpression(new) => {
                    let arguments = new.arguments(ast);
                    if arguments.len() > usize::from(depth)
                        || !arguments.iter().all(|id| {
                            self.simple_call_argument(ast.get_node_in_sub_range(id), child_depth)
                        })
                    {
                        return false;
                    }
                    new.callee(ast)
                }
                ExprData::ArrayExpression(array) => {
                    return array.elements(ast).iter().all(|id| {
                        ast.get_node_in_sub_range(id)
                            .is_none_or(|argument| self.simple_call_argument(argument, child_depth))
                    });
                }
                ExprData::ObjectExpression(object) => {
                    return object.properties(ast).iter().all(|id| {
                        let property = ast.get_node_in_sub_range(id);
                        let ObjectPropertyKindData::ObjectProperty(property) =
                            ast.object_property_kind_data(property)
                        else {
                            return false;
                        };
                        !property.computed(ast)
                            && !property.method(ast)
                            && (property.shorthand(ast)
                                || self.simple_callback_neighbor(property.value(ast), child_depth))
                    });
                }
                _ => return false,
            };
            data = self.expression_content(next).data;
        }
    }

    pub(super) fn format_selected_arguments(
        &mut self,
        arguments: SubRange<Argument>,
        content: Span,
        head: ArgumentHead,
        mut selected: SelectedArgument,
        mut missing_function: Option<&mut bool>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let (selected_argument, mut outcome) = match &selected {
            SelectedArgument::Callback(selected) => (selected.argument, SelectionOutcome::Callback),
            SelectedArgument::FinalObject(selected) => (
                selected.argument,
                SelectionOutcome::FinalObject { has_forced: false },
            ),
            SelectedArgument::FinalArray(selected) => (
                selected.argument,
                SelectionOutcome::FinalArray { has_forced: false },
            ),
        };
        let (opening, ()) = writer.record_with_value(|writer| {
            self.opening_line_constraint(content).write(
                writer,
                FlatSeparator::Empty,
                Writer::soft_line,
            );
            Ok::<_, FormatError>(())
        })?;
        let mut recorded = Vec::with_capacity(arguments.len());
        let sole_argument = arguments.len() == 1;
        let mut arguments = arguments
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .peekable();
        let mut previous = None;
        while let Some(argument) = arguments.next() {
            let (prefix, mut item) = writer.record_with_value(|writer| {
                self.prepare_argument_slot(
                    argument,
                    arguments.peek().copied(),
                    previous,
                    content,
                    self.options.trailing_commas == TrailingCommas::All,
                    writer,
                )
            })?;
            if previous.is_none() && head.first_mode == ContentMode::Verbatim {
                item.set_leading_mode(head.first_mode);
            }
            let (value, tail) = match &mut selected {
                SelectedArgument::Callback(selected)
                    if argument == selected.argument
                        && item.mode == ContentMode::Formatted
                        && !self.argument_has_boundary_comments(&item) =>
                {
                    if let Some(function) = missing_function.take() {
                        *function = true;
                    }
                    let (mut value, tail) =
                        self.record_callback(selected, sole_argument, writer)?;
                    if tail.as_ref().is_some_and(|tail| !tail.comments.is_empty())
                        && let RecordedValue::Callback { normal, .. } = value
                    {
                        value = RecordedValue::Ordinary(normal);
                    }
                    if matches!(value, RecordedValue::Ordinary(_)) {
                        outcome = SelectionOutcome::Ordinary;
                    }
                    (value, tail)
                }
                SelectedArgument::FinalObject(selected)
                    if argument == selected.argument
                        && item.mode == ContentMode::Formatted
                        && !self.argument_has_boundary_comments(&item) =>
                {
                    let object =
                        self.record_object_argument(selected.expression, selected.object, writer)?;
                    outcome = SelectionOutcome::FinalObject {
                        has_forced: object.forced.is_some(),
                    };
                    (RecordedValue::FinalObject(object), None)
                }
                SelectedArgument::FinalArray(selected)
                    if argument == selected.argument
                        && item.mode == ContentMode::Formatted
                        && !self.argument_has_boundary_comments(&item) =>
                {
                    match self.record_array_argument(
                        selected.expression,
                        selected.array,
                        selected.arity,
                        writer,
                    )? {
                        RecordedArrayArgument::Selected(array) => {
                            outcome = SelectionOutcome::FinalArray {
                                has_forced: array.forced.is_some(),
                            };
                            (RecordedValue::FinalArray(array), None)
                        }
                        RecordedArrayArgument::Ordinary(value) => {
                            outcome = SelectionOutcome::Ordinary;
                            (RecordedValue::Ordinary(value), None)
                        }
                    }
                }
                _ => {
                    if argument == selected_argument {
                        outcome = SelectionOutcome::Ordinary;
                    }
                    let (value, tail) = writer.record_with_value(|writer| {
                        self.format_argument_with_function_fact(
                            argument,
                            item.mode,
                            &mut missing_function,
                            writer,
                        )
                    })?;
                    (RecordedValue::Ordinary(value), tail)
                }
            };
            let (suffix, boundary) = writer
                .record_with_value(|writer| self.finish_delimited_item(item, tail, writer))?;
            previous = Some(boundary);
            recorded.push(RecordedArgument {
                prefix,
                value,
                suffix,
            });
        }
        let (closing, ()) = writer.record_with_value(|writer| {
            let end = previous.map_or(content.start, |boundary| boundary.end);
            let end = self.format_argument_dangling(Span::new(end, content.end), writer)?;
            self.line_constraint(end, content.end).write(
                writer,
                FlatSeparator::Empty,
                Writer::soft_line,
            );
            Ok::<_, FormatError>(())
        })?;
        // A rejected callback expansion owns a broken argument frame, even when
        // its neighbor only requested source layout. Chain measurement must see
        // that complete frame instead of flattening the neighbor in isolation.
        let expand_fallback = matches!(outcome, SelectionOutcome::Callback)
            && recorded.iter().any(|item| match item.value {
                RecordedValue::Ordinary(value) => value.preferred_breaks(),
                RecordedValue::Callback {
                    signature: None,
                    layout: ExpressionLayout::Normal,
                    ..
                } => true,
                _ => false,
            });
        let (fallback, ()) = writer.record_with_value(|writer| {
            writer.group(expand_fallback, |writer| {
                writer.text("(");
                writer.indent(|writer| {
                    writer.write_fragment(opening);
                    for item in &recorded {
                        writer.write_fragment(item.prefix);
                        match item.value {
                            RecordedValue::Ordinary(value) => {
                                writer.write_fragment(value);
                            }
                            RecordedValue::FinalObject(object) => {
                                writer.write_fragment(object.normal)
                            }
                            RecordedValue::FinalArray(array) => writer.write_fragment(array.normal),
                            RecordedValue::Callback { normal, .. } => writer.write_fragment(normal),
                        }
                        writer.write_fragment(item.suffix);
                    }
                });
                writer.write_fragment(closing);
                writer.text(")");
            });
            Ok::<_, FormatError>(())
        })?;
        let policy = match outcome {
            SelectionOutcome::Callback => GroupingPolicy::Callback,
            SelectionOutcome::FinalObject { .. } => GroupingPolicy::FinalObject,
            SelectionOutcome::FinalArray { .. } => GroupingPolicy::FinalArray,
            SelectionOutcome::Ordinary => {
                writer.write_fragment(fallback);
                return Ok(None);
            }
        };
        if let Some(grouped_items) = recorded
            .iter()
            .map(|item| item.try_group(policy))
            .collect::<Option<Vec<_>>>()
            && let Some(opening) = grouping_view(opening, policy)
            && let Some(flat_closing) = grouping_view(closing, policy)
        {
            let (grouped, ()) = writer.record_with_value(|writer| {
                write_grouped_arguments(
                    opening,
                    &grouped_items,
                    closing,
                    flat_closing,
                    false,
                    writer,
                );
                Ok::<_, FormatError>(())
            })?;
            if matches!(
                outcome,
                SelectionOutcome::FinalObject { has_forced: true }
                    | SelectionOutcome::FinalArray { has_forced: true }
            ) || matches!(outcome, SelectionOutcome::Callback)
                && grouped_items
                    .iter()
                    .any(|item| matches!(item.value, GroupedValue::Callback { .. }))
            {
                let (expanded, ()) = writer.record_with_value(|writer| {
                    write_grouped_arguments(
                        opening,
                        &grouped_items,
                        closing,
                        flat_closing,
                        true,
                        writer,
                    );
                    Ok::<_, FormatError>(())
                })?;
                writer.write_alternatives(fallback, &[grouped, expanded], fallback);
            } else {
                writer.write_alternatives(fallback, &[grouped], fallback);
            }
        } else {
            writer.write_fragment(fallback);
        }
        Ok(None)
    }
}

fn write_grouped_arguments<'a>(
    opening: FlatFragment<'a>,
    items: &[GroupedArgument<'a>],
    closing: Fragment<'a>,
    flat_closing: FlatFragment<'a>,
    expand_body: bool,
    writer: &mut Writer<'a>,
) {
    writer.text("(");
    writer.write_flat_fragment(opening);
    for item in items {
        writer.write_flat_fragment(item.prefix);
        match item.value {
            GroupedValue::Ordinary(value) => writer.write_flat_fragment(value),
            GroupedValue::FinalObject(object) => {
                let value = if expand_body {
                    object.forced.unwrap_or(object.normal)
                } else {
                    object.normal
                };
                writer.write_fragment(value);
            }
            GroupedValue::FinalArray(array) => {
                let value = if expand_body {
                    array.forced.unwrap_or(array.normal)
                } else {
                    array.normal
                };
                writer.write_fragment(value);
            }
            GroupedValue::Callback {
                signature,
                body: CallbackBody::Terminal(body),
                suffix,
            } => {
                match signature {
                    GroupedSignature::Independent(signature) => writer.write_fragment(signature),
                    GroupedSignature::Flat(signature) => writer.write_flat_fragment(signature),
                }
                writer.group(expand_body, |writer| {
                    writer.write_fragment(body);
                    writer.write_fragment(suffix);
                    writer.write_fragment(closing);
                });
                writer.text(")");
                return;
            }
            GroupedValue::Callback {
                signature,
                body: CallbackBody::Retained(body),
                ..
            } => match signature {
                GroupedSignature::Independent(signature) => {
                    writer.group(expand_body, |writer| {
                        writer.write_fragment(signature);
                        writer.write_fragment(body);
                    });
                }
                GroupedSignature::Flat(signature) => {
                    writer.write_flat_fragment(signature);
                    writer.group(expand_body, |writer| writer.write_fragment(body));
                }
            },
        }
        writer.write_flat_fragment(item.suffix);
    }
    writer.write_flat_fragment(flat_closing);
    writer.text(")");
}
