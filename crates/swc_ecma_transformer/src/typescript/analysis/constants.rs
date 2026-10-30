use rustc_hash::FxHashMap;

use swc_next_allocator::{
    CloneIn,
    wtf8::{Wtf8, Wtf8Buf},
};
use swc_next_ecma_ast::{
    Ast, AstUtf8, BinaryOperator, Expr, ExprData, MemberExpression, NodeId, NodeKindData,
    NumericLiteral, PropertyKeyData, SymbolId, TsEnumDeclaration, TsEnumMember, UnaryOperator,
    VariableDeclarator, Visit, VisitWith,
};
use swc_next_ecma_semantic::{NameResolution, Semantic, SymbolFlags};

/// A TypeScript enum member value known without evaluating runtime code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TsEnumConstantValue<'a> {
    Number { value: f64, raw: Option<AstUtf8> },
    String(&'a Wtf8),
}

/// Facts needed to emit a member whose exact value may be unknown.
#[derive(Debug, Clone, Copy, Default)]
pub(in crate::typescript) enum EnumMemberValue<'a> {
    Constant(TsEnumConstantValue<'a>),
    StringExpression,
    #[default]
    Unknown,
}

impl<'a> EnumMemberValue<'a> {
    pub(in crate::typescript) fn constant(self) -> Option<TsEnumConstantValue<'a>> {
        match self {
            Self::Constant(value) => Some(value),
            Self::StringExpression | Self::Unknown => None,
        }
    }

    pub(in crate::typescript) fn is_string(self) -> bool {
        matches!(
            self,
            Self::Constant(TsEnumConstantValue::String(_)) | Self::StringExpression
        )
    }
}

type EnumMemberValues<'a> = FxHashMap<SymbolId, EnumMemberValue<'a>>;

#[derive(Clone, Copy)]
pub(super) struct ConstBinding {
    pub syntax: VariableDeclarator,
    pub deferred: Option<NodeId>,
}

pub(super) type ConstBindings = FxHashMap<SymbolId, ConstBinding>;

#[derive(Clone, Copy)]
struct EvaluationLocation {
    node: NodeId,
    deferred: Option<NodeId>,
}

#[derive(Clone, Copy)]
struct EnumMemberDeclaration {
    syntax: TsEnumMember,
    owner: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnumEvaluation {
    Pending,
    Evaluating,
    Evaluated,
}

#[derive(Clone, Copy)]
enum ConstEvaluation<'a> {
    Visiting,
    Evaluated(EnumMemberValue<'a>),
}

#[derive(Clone, Copy)]
pub(super) struct EnumDeclaration {
    pub syntax: TsEnumDeclaration,
    pub auto_increment: bool,
    pub deferred: Option<NodeId>,
}

pub(super) fn evaluate<'a>(
    tree: &Ast<'a>,
    declarations: &[EnumDeclaration],
    const_bindings: &ConstBindings,
    semantic: &Semantic<'a>,
) -> EnumMemberValues<'a> {
    let mut enum_member_declarations = FxHashMap::default();
    for (owner, declaration) in declarations.iter().enumerate() {
        let syntax = declaration.syntax;
        for member_id in syntax.body(tree).members(tree).iter() {
            let member = tree.get_node_in_sub_range(member_id);
            if let Some(symbol) = semantic.symbol_of(member.node_id()) {
                enum_member_declarations.insert(
                    symbol,
                    EnumMemberDeclaration {
                        syntax: member,
                        owner,
                    },
                );
            }
        }
    }

    let mut computer = EnumValueComputer {
        tree,
        semantic,
        const_bindings,
        enum_member_declarations,
        declarations,
        enum_evaluations: vec![EnumEvaluation::Pending; declarations.len()],
        enum_values: FxHashMap::default(),
        const_evaluations: FxHashMap::default(),
        order: None,
    };

    for index in 0..declarations.len() {
        computer.compute_declaration(index);
    }

    computer.enum_values
}

struct EnumValueComputer<'visit, 'a> {
    tree: &'visit Ast<'a>,
    semantic: &'visit Semantic<'a>,
    const_bindings: &'visit ConstBindings,
    enum_member_declarations: FxHashMap<SymbolId, EnumMemberDeclaration>,
    declarations: &'visit [EnumDeclaration],
    enum_evaluations: Vec<EnumEvaluation>,
    enum_values: EnumMemberValues<'a>,
    const_evaluations: FxHashMap<SymbolId, ConstEvaluation<'a>>,
    order: Option<TraversalOrder<'visit, 'a>>,
}

/// Source spans survive AST edits, so declaration order and containment must
/// follow the current tree. Build this index only for symbol-dependent values.
struct TraversalOrder<'visit, 'a> {
    tree: &'visit Ast<'a>,
    ranges: Vec<std::ops::Range<usize>>,
    next: usize,
}

impl<'visit, 'a> TraversalOrder<'visit, 'a> {
    fn new(tree: &'visit Ast<'a>) -> Self {
        let mut order = Self {
            tree,
            ranges: vec![0..0; tree.node_count()],
            next: 0,
        };
        tree.root_program().visit_with(&mut order);
        order
    }
}

impl<'a> Visit<'a> for TraversalOrder<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.tree
    }

    fn enter_node(&mut self, node: NodeKindData) {
        self.ranges[node.node_id().index()].start = self.next;
        self.next += 1;
    }

    fn leave_node(&mut self, node: NodeKindData) {
        self.ranges[node.node_id().index()].end = self.next;
    }
}

impl<'a> EnumValueComputer<'_, 'a> {
    fn compute_declaration(&mut self, index: usize) {
        if self.enum_evaluations[index] != EnumEvaluation::Pending {
            return;
        }
        // Publish earlier members as we go, but do not reenter an enum whose
        // initializers are being evaluated. This also terminates dependency cycles.
        self.enum_evaluations[index] = EnumEvaluation::Evaluating;
        let declaration = self.declarations[index];
        let mut next = Some(number(0.0));
        for member_id in declaration.syntax.body(self.tree).members(self.tree).iter() {
            let member = self.tree.get_node_in_sub_range(member_id);
            let symbol = self.semantic.symbol_of(member.node_id());
            let location = EvaluationLocation {
                node: member.node_id(),
                deferred: declaration.deferred,
            };
            let value = if let Some(initializer) = member.initializer(self.tree) {
                self.compute(initializer, location)
            } else if declaration.auto_increment {
                next.map_or(EnumMemberValue::Unknown, EnumMemberValue::Constant)
            } else {
                EnumMemberValue::Unknown
            };
            if let Some(symbol) = symbol {
                if matches!(value, EnumMemberValue::Unknown) {
                    self.enum_values.remove(&symbol);
                } else {
                    self.enum_values.insert(symbol, value);
                }
            }
            next = value.constant().and_then(increment);
        }
        self.enum_evaluations[index] = EnumEvaluation::Evaluated;
    }

    fn compute(&mut self, expression: Expr, location: EvaluationLocation) -> EnumMemberValue<'a> {
        use EnumMemberValue::{Constant, StringExpression, Unknown};
        match self.tree.kind_data(expression.node_id()) {
            NodeKindData::StringLiteral(literal) => Constant(TsEnumConstantValue::String(
                self.tree.get_wtf8(literal.value(self.tree)),
            )),
            NodeKindData::NumericLiteral(literal) => Constant(self.number_literal(literal)),
            NodeKindData::IdentifierReference(identifier) => {
                let Some(reference) = self.semantic.reference_of(identifier.node_id()) else {
                    return Unknown;
                };
                match self.semantic.reference(reference).resolution() {
                    NameResolution::Resolved(symbol) => {
                        return self.compute_symbol(symbol, location);
                    }
                    NameResolution::Uncertain { .. } => return Unknown,
                    NameResolution::Unresolved => {}
                }
                match self.tree.get_utf8(identifier.name(self.tree)) {
                    "Infinity" => Constant(number(f64::INFINITY)),
                    "NaN" => Constant(number(f64::NAN)),
                    _ => Unknown,
                }
            }
            NodeKindData::ParenthesizedExpression(parenthesized) => {
                self.compute(parenthesized.expression(self.tree), location)
            }
            NodeKindData::ChainExpression(chain) => {
                self.compute(chain.expression(self.tree), location)
            }
            NodeKindData::UnaryExpression(unary) => {
                let Constant(TsEnumConstantValue::Number { value, .. }) =
                    self.compute(unary.argument(self.tree), location)
                else {
                    return Unknown;
                };
                let value = match unary.operator(self.tree) {
                    UnaryOperator::Positive => value,
                    UnaryOperator::Negate => -value,
                    UnaryOperator::BitwiseNot => f64::from(!to_int32(value)),
                    UnaryOperator::LogicalNot
                    | UnaryOperator::Typeof
                    | UnaryOperator::Void
                    | UnaryOperator::Delete => return Unknown,
                };
                Constant(number(value))
            }
            NodeKindData::BinaryExpression(binary) => {
                let left = self.compute(binary.left(self.tree), location);
                let operator = binary.operator(self.tree);
                // Only addition can preserve string knowledge after a failed fold.
                if left.constant().is_none() && operator != BinaryOperator::Add {
                    return Unknown;
                }
                let right = self.compute(binary.right(self.tree), location);
                if let (Some(left), Some(right)) = (left.constant(), right.constant()) {
                    return self
                        .compute_binary(left, right, operator)
                        .map_or(Unknown, Constant);
                }
                if operator == BinaryOperator::Add && (left.is_string() || right.is_string()) {
                    StringExpression
                } else {
                    Unknown
                }
            }
            NodeKindData::MemberExpression(member) => {
                let Some(symbol) = resolve_constant_member(self.tree, self.semantic, member) else {
                    return Unknown;
                };
                self.compute_symbol(symbol, location)
            }
            NodeKindData::TemplateLiteral(template) => self
                .compute_template(template, location)
                .map_or(StringExpression, Constant),
            // Type syntax deliberately terminates both constant evaluation and string
            // classification, matching TypeScript before lowering erases wrappers.
            _ => Unknown,
        }
    }

    fn compute_symbol(
        &mut self,
        symbol: SymbolId,
        location: EvaluationLocation,
    ) -> EnumMemberValue<'a> {
        // Semantic keeps an alias's binding identity separate from its terminal value.
        let symbol = self
            .semantic
            .ts_import_alias_target(symbol)
            .unwrap_or(symbol);
        if let Some(declaration) = self.enum_member_declarations.get(&symbol).copied() {
            if !self.is_declared_before(declaration.syntax.node_id(), location) {
                return EnumMemberValue::Unknown;
            }
            self.compute_declaration(declaration.owner);
            return self.enum_values.get(&symbol).copied().unwrap_or_default();
        }

        let Some(binding) = self.const_bindings.get(&symbol).copied() else {
            return EnumMemberValue::Unknown;
        };
        // Declaration order is use-site dependent. Check it before consulting the cache so
        // a forward reference cannot poison a later, valid use of the same const binding.
        if !self.is_declared_before(binding.syntax.node_id(), location) {
            return EnumMemberValue::Unknown;
        }
        if let Some(evaluation) = self.const_evaluations.get(&symbol).copied() {
            return match evaluation {
                ConstEvaluation::Visiting => EnumMemberValue::Unknown,
                ConstEvaluation::Evaluated(value) => value,
            };
        }

        let Some(initializer) = binding.syntax.init(self.tree) else {
            return EnumMemberValue::Unknown;
        };
        self.const_evaluations
            .insert(symbol, ConstEvaluation::Visiting);
        let location = EvaluationLocation {
            node: binding.syntax.node_id(),
            deferred: binding.deferred,
        };
        let value = self.compute(initializer, location);
        self.const_evaluations
            .insert(symbol, ConstEvaluation::Evaluated(value));
        value
    }

    fn is_declared_before(&mut self, declaration: NodeId, location: EvaluationLocation) -> bool {
        if declaration == location.node {
            return false;
        }
        let order = self
            .order
            .get_or_insert_with(|| TraversalOrder::new(self.tree));
        let declaration = &order.ranges[declaration.index()];
        if declaration.start <= order.ranges[location.node.index()].start {
            return true;
        }
        // A function or instance initializer may execute after an outer binding
        // is initialized. Bindings within that same deferred region still obey
        // source order. Immediately invoked functions do not create a region.
        location.deferred.is_some_and(|region| {
            let region = &order.ranges[region.index()];
            declaration.start < region.start || declaration.end > region.end
        })
    }

    fn number_literal(&self, literal: NumericLiteral) -> TsEnumConstantValue<'a> {
        TsEnumConstantValue::Number {
            value: literal.value(self.tree),
            raw: literal.raw(self.tree),
        }
    }

    fn compute_binary(
        &self,
        left: TsEnumConstantValue<'a>,
        right: TsEnumConstantValue<'a>,
        operator: BinaryOperator,
    ) -> Option<TsEnumConstantValue<'a>> {
        if operator == BinaryOperator::Add
            && (matches!(left, TsEnumConstantValue::String(_))
                || matches!(right, TsEnumConstantValue::String(_)))
        {
            let mut value = Wtf8Buf::new();
            push_to_js_string(&mut value, left);
            push_to_js_string(&mut value, right);
            return Some(TsEnumConstantValue::String(self.alloc_buffer(value)));
        }

        let TsEnumConstantValue::Number { value: left, .. } = left else {
            return None;
        };
        let TsEnumConstantValue::Number { value: right, .. } = right else {
            return None;
        };
        let value = match operator {
            BinaryOperator::Add => left + right,
            BinaryOperator::Subtract => left - right,
            BinaryOperator::Multiply => left * right,
            BinaryOperator::Divide => left / right,
            BinaryOperator::Modulo => left % right,
            BinaryOperator::Exponent => exponentiate(left, right),
            BinaryOperator::LeftShift => f64::from(to_int32(left).wrapping_shl(to_uint32(right))),
            BinaryOperator::RightShift => f64::from(to_int32(left).wrapping_shr(to_uint32(right))),
            BinaryOperator::UnsignedRightShift => {
                f64::from(to_uint32(left).wrapping_shr(to_uint32(right)))
            }
            BinaryOperator::BitwiseOr => f64::from(to_int32(left) | to_int32(right)),
            BinaryOperator::BitwiseXor => f64::from(to_int32(left) ^ to_int32(right)),
            BinaryOperator::BitwiseAnd => f64::from(to_int32(left) & to_int32(right)),
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::StrictEqual
            | BinaryOperator::StrictNotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::In
            | BinaryOperator::Instanceof => return None,
        };
        Some(number(value))
    }

    fn compute_template(
        &mut self,
        template: swc_next_ecma_ast::TemplateLiteral,
        location: EvaluationLocation,
    ) -> Option<TsEnumConstantValue<'a>> {
        let quasis = template.quasis(self.tree);
        let expressions = template.expressions(self.tree);
        let mut quasis = quasis.iter();
        let first = self.tree.get_node_in_sub_range(quasis.next()?);
        if first.is_cooked_undefined(self.tree) {
            return None;
        }
        let cooked = self.tree.get_wtf8(first.cooked(self.tree));
        if expressions.is_empty() {
            return Some(TsEnumConstantValue::String(cooked));
        }
        let mut value = Wtf8Buf::new();
        value.push_wtf8(cooked);
        for expression in expressions.iter() {
            let expression = self.tree.get_node_in_sub_range(expression);
            push_to_js_string(&mut value, self.compute(expression, location).constant()?);
            let quasi = self.tree.get_node_in_sub_range(quasis.next()?);
            if quasi.is_cooked_undefined(self.tree) {
                return None;
            }
            value.push_wtf8(self.tree.get_wtf8(quasi.cooked(self.tree)));
        }
        Some(TsEnumConstantValue::String(self.alloc_buffer(value)))
    }

    fn alloc_buffer(&self, value: Wtf8Buf) -> &'a Wtf8 {
        let text: &Wtf8 = &value;
        text.clone_in(self.tree.allocator())
    }
}

#[derive(Clone, Copy)]
enum MemberResolution {
    Constant,
    Inline,
}

// Constant evaluation follows TypeScript's entity-name grammar: dot access can
// resolve a namespace constant, while element access only resolves enum members.
pub(super) fn resolve_constant_member(
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    member: MemberExpression,
) -> Option<SymbolId> {
    let (_, owner) = namespace_object(
        member.object(ast),
        ast,
        semantic,
        MemberResolution::Constant,
    )?;
    let name = static_member_name(ast, member)?;
    match ast.property_key_data(member.property(ast)) {
        PropertyKeyData::IdentifierName(_) => namespace_value_symbol(semantic, owner, name),
        PropertyKeyData::Expr(_) => semantic.ts_enum_member(owner, name),
    }
}

pub(super) fn resolve_enum_member(
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    member: MemberExpression,
) -> Option<(NodeId, SymbolId, SymbolId)> {
    if member.optional(ast) {
        return None;
    }
    let (reference, owner) =
        namespace_object(member.object(ast), ast, semantic, MemberResolution::Inline)?;
    let name = static_member_name(ast, member)?;
    let symbol = semantic.ts_enum_member(owner, name)?;
    Some((reference, owner, symbol))
}

fn static_member_name<'a>(ast: &'a Ast<'_>, member: MemberExpression) -> Option<&'a [u8]> {
    let expression = match ast.property_key_data(member.property(ast)) {
        PropertyKeyData::IdentifierName(identifier) => {
            return Some(ast.get_utf8(identifier.name(ast)).as_bytes());
        }
        PropertyKeyData::Expr(expression) => expression,
    };
    match ast.expr_data(expression) {
        ExprData::StringLiteral(literal) => Some(ast.get_wtf8(literal.value(ast)).as_bytes()),
        ExprData::TemplateLiteral(template)
            if template.expressions(ast).is_empty() && template.quasis(ast).len() == 1 =>
        {
            let quasi = ast.get_node_in_sub_range(template.quasis(ast).iter().next()?);
            (!quasi.is_cooked_undefined(ast)).then(|| ast.get_wtf8(quasi.cooked(ast)).as_bytes())
        }
        _ => None,
    }
}

fn namespace_object(
    expression: Expr,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    resolution: MemberResolution,
) -> Option<(NodeId, SymbolId)> {
    let (reference, symbol) = match ast.expr_data(expression) {
        ExprData::IdentifierReference(identifier) => {
            let reference = semantic.reference_of(identifier.node_id())?;
            let mut targets = semantic.reference_value_targets(ast, reference);
            let symbol = targets.next()?.symbol()?;
            if targets.next().is_some() {
                return None;
            }
            (identifier.node_id(), symbol)
        }
        ExprData::ParenthesizedExpression(parenthesized) => {
            return namespace_object(parenthesized.expression(ast), ast, semantic, resolution);
        }
        ExprData::MemberExpression(member) => {
            let name = match resolution {
                MemberResolution::Constant => {
                    let PropertyKeyData::IdentifierName(identifier) =
                        ast.property_key_data(member.property(ast))
                    else {
                        return None;
                    };
                    ast.get_utf8(identifier.name(ast)).as_bytes()
                }
                MemberResolution::Inline => {
                    if member.optional(ast) {
                        return None;
                    }
                    static_member_name(ast, member)?
                }
            };
            let (reference, owner) =
                namespace_object(member.object(ast), ast, semantic, resolution)?;
            let symbol = namespace_value_symbol(semantic, owner, name)?;
            (reference, symbol)
        }
        _ => return None,
    };
    if semantic
        .symbol(symbol)
        .flags
        .intersects(SymbolFlags::ANY_IMPORT)
    {
        semantic
            .ts_import_alias_target(symbol)
            .map(|target| (reference, target))
    } else {
        Some((reference, symbol))
    }
}

fn namespace_value_symbol(
    semantic: &Semantic<'_>,
    owner: SymbolId,
    name: &[u8],
) -> Option<SymbolId> {
    let mut targets = semantic.ts_namespace_value_targets(owner, name);
    let symbol = targets.next()?.symbol()?;
    targets
        .all(|target| target.symbol() == Some(symbol))
        .then_some(symbol)
}

fn number(value: f64) -> TsEnumConstantValue<'static> {
    TsEnumConstantValue::Number { value, raw: None }
}

fn increment(value: TsEnumConstantValue<'_>) -> Option<TsEnumConstantValue<'static>> {
    match value {
        TsEnumConstantValue::Number { value, .. } => Some(number(value + 1.0)),
        TsEnumConstantValue::String(_) => None,
    }
}

fn to_uint32(value: f64) -> u32 {
    if !value.is_finite() {
        return 0;
    }
    value.trunc().rem_euclid(4_294_967_296.0) as u32
}

fn to_int32(value: f64) -> i32 {
    to_uint32(value) as i32
}

fn exponentiate(left: f64, right: f64) -> f64 {
    if right.is_nan() || (left.abs() == 1.0 && right.is_infinite()) {
        f64::NAN
    } else {
        left.powf(right)
    }
}

fn push_to_js_string(output: &mut Wtf8Buf, value: TsEnumConstantValue<'_>) {
    match value {
        TsEnumConstantValue::String(value) => output.push_wtf8(value),
        TsEnumConstantValue::Number { value, .. } => {
            let mut buffer = zmij_ecma::Buffer::new();
            output.push_str(buffer.format(value));
        }
    }
}
