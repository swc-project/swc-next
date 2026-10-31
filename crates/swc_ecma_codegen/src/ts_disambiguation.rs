//! Compose emitted expression fragments before printing TypeScript boundaries.
//!
//! A fragment describes its emitted prefix and whether speculative type syntax
//! can cross it. One planning entry accepts the parent's precedence, prefix and
//! continuation constraints. A binary plans its right fragment first, derives
//! the left context from that finalized summary, then synthesizes its result.
//! Lists propagate continuation requirements from right to left. Grammar barriers
//! leave unrelated subtrees for their own printing boundaries.
//!
//! Existing printer parentheses are fixed output, never candidates for removal.
//! Only comparison expressions receive additional groups. Comma lists propagate
//! continuation requirements backwards, separately for type arguments and type
//! parameters (where an assignment can instead be a default type). The plan also
//! prevents newly parenthesizing a bare arrow parameter when doing so would turn
//! the emitted arrow into speculative function-type syntax.

use swc_next_ecma_ast::*;

use crate::{
    operator::{Binaryish, BinaryishOperator, ExprContext, PreparedBinary},
    util::{
        bare_arrow_parameter, binding_identifier_name, can_shorten_binding_property, choose_quote,
        new_expression_has_argument_list, pure_comment_annotation,
        simple_assignment_target_expression,
    },
};

#[derive(Clone, Copy, Default)]
enum LeadingToken {
    #[default]
    Expression,
    // `<`, `>`, `+`, and `-` reject type arguments even after a line break.
    Reject,
    // Parentheses, templates, regexp slashes and contextual operator tokens
    // admit speculative type arguments.
    Accept,
}

enum PrintedPrefix {
    Parenthesis,
    Child(Expr, Precedence, bool),
}

#[derive(Clone, Copy)]
enum TypeEntry {
    Parameters,
    Indexed,
    Type,
    Other,
}

// Planning state is separate from the compositional syntax summary.
bitflags::bitflags! {
    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    struct State: u8 {
        const PROTECTED_ARGUMENT = 1 << 0;
        const PROTECTED_PARAMETER = 1 << 1;
        const KNOWN = 1 << 2;
        const BARE_ARROW = 1 << 3;
        const GROUPED = 1 << 4;
        const REJECT = 1 << 5;
        const ACCEPT = 1 << 6;
        const FIRST = Self::REJECT.bits() | Self::ACCEPT.bits();
    }
}

impl State {
    fn known(self) -> bool {
        self.contains(Self::KNOWN)
    }

    fn grouped(self) -> bool {
        self.contains(Self::GROUPED)
    }

    // Return only requirements not already visited, keeping overlapping
    // boundary walks linear.
    fn protect(&mut self, requirement: u8) -> u8 {
        let requirement = Self::from_bits_retain(requirement);
        let pending = requirement.difference(*self);
        self.insert(requirement);
        pending.bits()
    }

    fn first(self) -> LeadingToken {
        match self.intersection(Self::FIRST) {
            Self::REJECT => LeadingToken::Reject,
            Self::ACCEPT => LeadingToken::Accept,
            _ => LeadingToken::Expression,
        }
    }

    fn set_first(&mut self, first: LeadingToken) {
        let first = match first {
            LeadingToken::Expression => Self::empty(),
            LeadingToken::Reject => Self::REJECT,
            LeadingToken::Accept => Self::ACCEPT,
        };
        self.remove(Self::FIRST);
        self.insert(first);
    }
}

// A single `<` opens type arguments; `<<` can start a generic function type
// inside them. Keep the two requirements separate when matching closing shifts.
const ARGUMENT: u8 = State::PROTECTED_ARGUMENT.bits();
const PARAMETER: u8 = State::PROTECTED_PARAMETER.bits();
const BOTH: u8 = ARGUMENT | PARAMETER;
// Grammar positions, not planning/cache-validity flags. A default is parsed as
// an ordinary type while the generic parameter and outer argument both remain
// open; recovery from a parameter name can instead close the former early.
const TYPE_SUFFIX: u8 = 1 << 2;
// A function/constructor type is expecting a parameter signature. Unlike a
// parenthesized type, its recovery can enter a nullable or nested return type.
const SIGNATURE: u8 = 1 << 3;
const DEFAULT_TYPE: u8 = 1 << 4;
const GRAMMAR_POSITIONS: u8 = BOTH | TYPE_SUFFIX | SIGNATURE | DEFAULT_TYPE;

// A conservative relation between grammar entry positions before and after
// emitted tokens. In addition to arguments and generic parameters, composition
// distinguishes a suffix after a complete type, function-signature entry, and
// ordinary type parsing inside a parameter default. `closes` records accepting
// prefixes; each remaining field is one column of the relation.
// Boolean relation composition is associative, so every fragment uses `then`.
// Grammar positions are projected out before querying the protection cache.
#[derive(Clone, Copy, Default)]
struct TypeSyntax {
    closes: u8,
    argument: u8,
    parameter: u8,
    suffix: u8,
    signature: u8,
    default_type: u8,
}

impl TypeSyntax {
    fn carries(self) -> bool {
        self.argument | self.parameter | self.suffix | self.signature | self.default_type != 0
    }

    const THROUGH: Self = Self {
        closes: 0,
        argument: ARGUMENT,
        parameter: PARAMETER,
        suffix: TYPE_SUFFIX,
        signature: SIGNATURE,
        default_type: DEFAULT_TYPE,
    };
    const NAME: Self = Self {
        closes: 0,
        argument: ARGUMENT | SIGNATURE,
        parameter: PARAMETER,
        suffix: TYPE_SUFFIX,
        signature: 0,
        default_type: DEFAULT_TYPE,
    };
    const RECOVER: Self = Self {
        closes: 0,
        argument: BOTH,
        parameter: PARAMETER,
        suffix: TYPE_SUFFIX,
        signature: 0,
        default_type: DEFAULT_TYPE,
    };
    // A complete type that cannot continue a generic parameter declaration.
    const TYPE: Self = Self {
        closes: 0,
        argument: BOTH,
        parameter: 0,
        suffix: TYPE_SUFFIX,
        signature: 0,
        default_type: DEFAULT_TYPE,
    };
    const PARAMETER_PREFIX: Self = Self {
        closes: 0,
        argument: 0,
        parameter: PARAMETER | DEFAULT_TYPE,
        suffix: 0,
        signature: 0,
        default_type: 0,
    };
    const BARRIER: Self = Self {
        closes: 0,
        argument: 0,
        parameter: 0,
        suffix: 0,
        signature: 0,
        default_type: 0,
    };

    // Composition uses all positions; protection queries project back to the
    // original argument/parameter requirements.
    fn transfer_entries(self, continuation: u8) -> u8 {
        (if continuation & ARGUMENT != 0 {
            self.argument & GRAMMAR_POSITIONS
        } else {
            0
        }) | (if continuation & PARAMETER != 0 {
            self.parameter & GRAMMAR_POSITIONS
        } else {
            0
        })
    }

    fn transfer(self, continuation: u8) -> u8 {
        self.transfer_entries(continuation)
            | (if continuation & TYPE_SUFFIX != 0 {
                self.suffix
            } else {
                0
            })
            | (if continuation & SIGNATURE != 0 {
                self.signature
            } else {
                0
            })
            | (if continuation & DEFAULT_TYPE != 0 {
                self.default_type
            } else {
                0
            })
    }

    fn then(self, next: Self) -> Self {
        Self {
            closes: self.closes | self.transfer(next.closes),
            argument: self.transfer(next.argument),
            parameter: self.transfer(next.parameter),
            suffix: self.transfer(next.suffix),
            signature: self.transfer(next.signature),
            default_type: self.transfer(next.default_type),
        }
    }
    fn carry(self, continuation: u8) -> u8 {
        (if continuation & ARGUMENT != 0 {
            self.argument & BOTH
        } else {
            0
        }) | (if continuation & PARAMETER != 0 {
            self.parameter & BOTH
        } else {
            0
        })
    }
    // A boundary only needs acceptance, not the composed outgoing columns.
    fn before(self, continuation: u8) -> u8 {
        self.closes | self.transfer(continuation)
    }
    fn close_mask(self) -> u8 {
        self.closes & BOTH
    }
    fn constructor(self) -> Self {
        Self {
            closes: if self.closes & SIGNATURE != 0 {
                BOTH
            } else {
                0
            },
            argument: if self.argument & SIGNATURE != 0 {
                BOTH
            } else {
                0
            },
            parameter: 0,
            ..Self::BARRIER
        }
    }
    fn parameter_default(self) -> Self {
        Self {
            closes: if self.closes & DEFAULT_TYPE != 0 {
                PARAMETER
            } else {
                0
            },
            argument: 0,
            parameter: if (self.default_type | self.parameter) & DEFAULT_TYPE != 0 {
                PARAMETER
            } else {
                0
            },
            ..Self::BARRIER
        }
    }
    // Explicit and generated groups have the same emitted grammar. Recovery
    // inside a function type can still reach accepting prefixes in the child.
    fn parenthesized(self) -> Self {
        Self {
            closes: Self::RECOVER.carry(self.close_mask())
                | (self.closes & (SIGNATURE | TYPE_SUFFIX | DEFAULT_TYPE)),
            // The closing `)` ends parameter-name continuation. Retain the
            // child's barriers and accepting prefixes rather than replacing
            // every grouped expression with an unconditional complete type.
            argument: Self::RECOVER.transfer(self.argument),
            suffix: Self::RECOVER.transfer(self.suffix),
            default_type: Self::RECOVER.transfer(self.default_type),
            ..Self::BARRIER
        }
        .with_signature(self.argument & SIGNATURE != 0)
    }

    fn with_signature(mut self, allowed: bool) -> Self {
        if allowed {
            self.argument |= SIGNATURE;
        }
        self
    }

    // Substitute one grammar entry without changing the other incoming rows.
    fn with_entry(self, target: u8, syntax: Self, source: u8) -> Self {
        let replace =
            |column: u8, from: u8| (column & !target) | if from & source != 0 { target } else { 0 };
        Self {
            closes: replace(self.closes, syntax.closes),
            argument: replace(self.argument, syntax.argument),
            parameter: replace(self.parameter, syntax.parameter),
            suffix: replace(self.suffix, syntax.suffix),
            signature: replace(self.signature, syntax.signature),
            default_type: replace(self.default_type, syntax.default_type),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Summary {
    state: State,
    syntax: TypeSyntax,
}

// Complete expressions never leave a function-signature entry pending. Store the
// four remaining columns and accepting set in 25 bits, beside the existing
// seven planning/token flags. The directly indexed cache remains four bytes.
#[derive(Clone, Copy, Default)]
struct Fragment(u32);

impl Fragment {
    fn state(self) -> State {
        State::from_bits_retain((self.0 & 0x7f) as u8)
    }

    fn set_state(&mut self, state: State) {
        self.0 = self.0 & !0x7f | u32::from(state.bits());
    }

    fn syntax(self) -> TypeSyntax {
        let bits = self.0 >> 7;
        TypeSyntax {
            closes: (bits & 31) as u8,
            argument: ((bits >> 5) & 31) as u8,
            parameter: ((bits >> 10) & 31) as u8,
            suffix: ((bits >> 15) & 31) as u8,
            default_type: ((bits >> 20) & 31) as u8,
            signature: 0,
        }
    }

    fn finish(&mut self, first: LeadingToken, syntax: TypeSyntax) {
        debug_assert_eq!(syntax.signature, 0);
        let mut state = self.state();
        state.set_first(first);
        let bits = u32::from(syntax.closes)
            | u32::from(syntax.argument) << 5
            | u32::from(syntax.parameter) << 10
            | u32::from(syntax.suffix) << 15
            | u32::from(syntax.default_type) << 20;
        self.0 = u32::from(state.bits()) | bits << 7;
    }

    fn insert(&mut self, state: State) {
        self.0 |= u32::from(state.bits());
    }

    fn set_known(&mut self) {
        self.insert(State::KNOWN);
    }

    fn mark_grouped(&mut self) {
        self.insert(State::GROUPED);
    }

    fn protect(&mut self, requirement: u8) -> u8 {
        let mut state = self.state();
        let pending = state.protect(requirement);
        self.set_state(state);
        pending
    }
}

// External constraints travel down; finalized token/grammar summaries travel up.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Prefix {
    #[default]
    None,
    // The immediately preceding opener determines the grammar entry.
    Argument,
    Parameter,
    // Existing parentheses already shield a binary prefix; arrow token choices
    // still matter inside them during TypeScript's function-type recovery.
    Grouped,
}

impl Prefix {
    fn opening(self) -> u8 {
        match self {
            Self::Argument => ARGUMENT,
            Self::Parameter => PARAMETER,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Context {
    // The base printer's precedence requirement is never weakened.
    level: Precedence,
    // Speculative argument/parameter parses accepted by the following fragment.
    tail: u8,
    prefix: Prefix,
}

impl Context {
    const NONE: Self = Self {
        level: Precedence::Lowest,
        tail: 0,
        prefix: Prefix::None,
    };
    fn tail(level: Precedence, tail: u8) -> Self {
        Self {
            level,
            tail,
            ..Self::NONE
        }
    }
}

// Continuations encode dependency order, not separate protection passes.
enum Task {
    Plan(Expr, Context),
    // None checks an independent printing boundary without publishing a summary.
    BinaryLeft(PreparedBinary, Option<Context>),
    // The right fragment is final before the left context is derived.
    FinishBinary(PreparedBinary, u8),
    Finish(Expr, Prefix, ExprData),
}

// Keep the source member until its initializer or emitted binding head is
// needed. Candidate scans only inspect initializers; the reverse grammar fold
// also reads names, annotations and delimiters.
// Missing initializers still participate: names can continue type parameters,
// whereas `:` and holes stop them. Heads do not shield their own initializers.
#[derive(Clone, Copy)]
enum ListMember {
    Expression(Expr),
    Barrier(Option<Expr>),
    Binding(BindingPattern),
    Declaration(VariableDeclarator),
    Parameter(FormalParameterItem),
    Property(BindingProperty),
    Enum(TsEnumMember),
}

impl ListMember {
    fn argument(ast: &Ast<'_>, argument: Option<Argument>) -> Self {
        match argument.map(|argument| ast.argument_data(argument)) {
            Some(ArgumentData::Expr(expression)) => Self::Expression(expression),
            Some(ArgumentData::SpreadElement(spread)) => Self::Barrier(Some(spread.argument(ast))),
            None => Self::Barrier(None),
        }
    }

    fn binding_parts(ast: &Ast<'_>, binding: BindingPattern) -> (BindingPattern, Option<Expr>) {
        match ast.binding_pattern_data(binding) {
            BindingPatternData::AssignmentPattern(node) => (node.left(ast), Some(node.right(ast))),
            _ => (binding, None),
        }
    }

    #[inline]
    fn expression(self, ast: &Ast<'_>) -> Option<Expr> {
        match self {
            Self::Expression(expression) => Some(expression),
            Self::Barrier(expression) => expression,
            Self::Binding(binding) => Self::binding_parts(ast, binding).1,
            Self::Declaration(declaration) => declaration.init(ast),
            Self::Parameter(item) => {
                let parameter = match ast.formal_parameter_item_data(item) {
                    FormalParameterItemData::FormalParameter(parameter) => parameter,
                    FormalParameterItemData::TsParameterProperty(property) => {
                        property.parameter(ast)
                    }
                };
                parameter.initializer(ast)
            }
            Self::Property(property) => Self::binding_parts(ast, property.value(ast)).1,
            Self::Enum(member) => member.initializer(ast),
        }
    }

    fn binding_head(ast: &Ast<'_>, binding: BindingPattern) -> TypeSyntax {
        if binding_identifier_name(ast, binding).is_some() {
            TypeSyntax::NAME
        } else {
            TypeSyntax::BARRIER
        }
    }

    fn syntax(self, plan: &GroupingPlan<'_>, level: Precedence) -> TypeSyntax {
        let ast = plan.ast;
        let head = match self {
            Self::Expression(expression) => return plan.syntax(expression, level),
            Self::Barrier(_) => return TypeSyntax::BARRIER,
            Self::Binding(binding) => Self::binding_head(ast, Self::binding_parts(ast, binding).0),
            Self::Declaration(declaration) => {
                if declaration.definite(ast) || declaration.type_annotation(ast).is_some() {
                    TypeSyntax::BARRIER
                } else {
                    Self::binding_head(ast, declaration.id(ast))
                }
            }
            Self::Parameter(item) => {
                let (parameter, decorated) = match ast.formal_parameter_item_data(item) {
                    FormalParameterItemData::FormalParameter(parameter) => (parameter, false),
                    FormalParameterItemData::TsParameterProperty(property) => (
                        property.parameter(ast),
                        !property.decorators(ast).is_empty(),
                    ),
                };
                if decorated
                    || !parameter.decorators(ast).is_empty()
                    || parameter.optional(ast)
                    || parameter.type_annotation(ast).is_some()
                {
                    TypeSyntax::BARRIER
                } else {
                    Self::binding_head(ast, parameter.pattern(ast).into())
                }
            }
            Self::Property(property) => {
                if !property.shorthand(ast)
                    && !can_shorten_binding_property(ast, property, plan.minify)
                {
                    TypeSyntax::BARRIER
                } else {
                    Self::binding_head(ast, Self::binding_parts(ast, property.value(ast)).0)
                }
            }
            Self::Enum(member) => {
                if !member.computed(ast)
                    && matches!(
                        ast.ts_enum_member_name_data(member.id(ast)),
                        TsEnumMemberNameData::IdentifierName(_)
                    )
                {
                    TypeSyntax::NAME
                } else {
                    TypeSyntax::BARRIER
                }
            }
        };
        self.expression(ast).map_or(head, |expression| {
            head.then(plan.syntax(expression, level).parameter_default())
        })
    }
}

pub(crate) struct GroupingPlan<'a> {
    ast: &'a Ast<'a>,
    fragments: Vec<Fragment>,
    minify: bool,
    comments: bool,
    line_breaks: Vec<(u64, u32)>,
    pending: Vec<Task>,
}

impl<'a> GroupingPlan<'a> {
    pub(crate) fn new(ast: &'a Ast<'a>, minify: bool, comments: bool) -> Self {
        Self {
            ast,
            minify,
            comments,
            fragments: Vec::new(),
            line_breaks: Vec::new(),
            pending: Vec::new(),
        }
    }

    pub(crate) fn bare_arrow_parameters(&self, arrow: ArrowFunctionExpression) -> bool {
        self.fragments
            .get(arrow.node_id().index())
            .is_some_and(|fragment| fragment.state().contains(State::BARE_ARROW))
    }

    fn needs_group(&self, expression: Expr) -> bool {
        self.fragments
            .get(expression.node_id().index())
            .is_some_and(|fragment| fragment.state().grouped())
    }

    // Planning may inspect flags before the expression has a summary.
    fn state(&self, expression: Expr) -> State {
        self.fragments[expression.node_id().index()].state()
    }

    // Reading semantic facts requires completed dependency planning.
    fn summary(&self, expression: Expr) -> Summary {
        let fragment = self.fragments[expression.node_id().index()];
        debug_assert!(
            fragment.state().known(),
            "summary requires planned dependencies"
        );
        Summary {
            state: fragment.state(),
            syntax: fragment.syntax(),
        }
    }

    fn binary(data: ExprData) -> Option<Binaryish> {
        match data {
            ExprData::BinaryExpression(binary) => Some(Binaryish::Binary(binary)),
            ExprData::LogicalExpression(binary) => Some(Binaryish::Logical(binary)),
            _ => None,
        }
    }

    fn initialize(&mut self) {
        if !self.fragments.is_empty() {
            return;
        }
        self.fragments
            .resize(self.ast.node_count(), Fragment::default());
    }

    fn index_line_breaks(&mut self) {
        if !self.line_breaks.is_empty() {
            return;
        }
        // A rank index answers comment-gap queries in constant time. Building
        // it is linear in source size; no printer cursor enters cached facts.
        self.line_breaks
            .resize(self.ast.source().len() / 64 + 1, (0, 0));
        for comment in self.ast.comments() {
            let value = self
                .ast
                .get_utf8(comment.value(self.ast.source().as_bytes()));
            let pure = pure_comment_annotation(comment.kind, value.as_bytes()).is_some();
            if !pure
                && (comment.kind == CommentKind::Line
                    || value.contains(['\n', '\r', '\u{2028}', '\u{2029}']))
            {
                let offset = comment.span.end.saturating_sub(1) as usize;
                self.line_breaks[offset / 64].0 |= 1 << (offset % 64);
            }
        }
        let mut count = 0;
        for (bits, rank) in &mut self.line_breaks {
            *rank = count;
            count += bits.count_ones();
        }
    }

    fn line_break_rank(&self, offset: u32) -> u32 {
        let offset = (offset as usize).min(self.ast.source().len());
        let Some(&(bits, rank)) = self.line_breaks.get(offset / 64) else {
            return 0;
        };
        rank + (bits & ((1u64 << (offset % 64)) - 1)).count_ones()
    }

    fn run(&mut self) {
        self.run_since(0);
    }

    fn run_since(&mut self, depth: usize) {
        while self.pending.len() > depth {
            let task = self.pending.pop().unwrap();
            match task {
                Task::Plan(expression, context) => self.plan(expression, context),
                Task::BinaryLeft(binary, context) => self.plan_binary_left(binary, context),
                Task::FinishBinary(binary, closes) => {
                    let (first, syntax) = self.binary_summary(binary, closes);
                    self.record_summary(binary.expression(), first, syntax);
                }
                Task::Finish(expression, prefix, data) => self.finish(expression, prefix, data),
            }
        }
    }

    fn dependency(&mut self, expression: Expr) {
        self.pending.push(Task::Plan(expression, Context::NONE));
    }

    fn plan(&mut self, expression: Expr, mut context: Context) {
        let known = self.state(expression).known();
        if context.tail != 0 {
            context.tail =
                if self.grouped(expression, context.level) || !self.can_expose_open(expression) {
                    0
                } else {
                    self.fragments[expression.node_id().index()].protect(context.tail & BOTH)
                        | (context.tail & !BOTH)
                };
        }
        if known && context.tail == 0 && context.prefix == Prefix::None {
            return;
        }
        let data = self.ast.expr_data(expression);
        if let Some(syntax) = Self::terminal_syntax(data) {
            let first = self.leading_token(data);
            self.record_summary(expression, first, syntax);
            return;
        }
        if let Some(binary) = Self::binary(data) {
            let binary =
                PreparedBinary::new(self.ast, binary, binary.operator(self.ast), self.minify);
            self.plan_binary_right(binary, Some(context));
            return;
        }
        self.pending
            .push(Task::Finish(expression, context.prefix, data));
        let ast = self.ast;
        match data {
            ExprData::ParenthesizedExpression(node) => self.pending.push(Task::Plan(
                node.expression(ast),
                Context {
                    prefix: if context.prefix == Prefix::None {
                        Prefix::None
                    } else {
                        Prefix::Grouped
                    },
                    ..Context::NONE
                },
            )),
            ExprData::AssignmentExpression(node) => {
                if node.operator(ast) == AssignmentOperator::Assign || context.tail != 0 {
                    self.pending.push(Task::Plan(
                        node.right(ast),
                        Context::tail(Precedence::Assignment, context.tail),
                    ));
                }
            }
            ExprData::AwaitExpression(node) => self.dependency(node.argument(ast)),
            ExprData::TsTypeAssertion(node) => self.dependency(node.expression(ast)),
            ExprData::ImportExpression(node) => {
                self.dependency(node.source(ast));
                if let Some(options) = node.options(ast) {
                    self.dependency(options);
                }
            }
            ExprData::UnaryExpression(node)
                if matches!(
                    node.operator(ast),
                    UnaryOperator::Typeof
                        | UnaryOperator::LogicalNot
                        | UnaryOperator::Void
                        | UnaryOperator::Delete
                ) =>
            {
                self.dependency(node.argument(ast));
            }
            ExprData::NewExpression(node) => {
                self.dependency(node.callee(ast));
                for id in node.arguments(ast).iter() {
                    let argument = match ast.argument_data(ast.get_node_in_sub_range(id)) {
                        ArgumentData::Expr(e) => e,
                        ArgumentData::SpreadElement(s) => s.argument(ast),
                    };
                    self.dependency(argument);
                }
            }
            ExprData::YieldExpression(node) => {
                if let Some(argument) = node.argument(ast) {
                    self.pending.push(Task::Plan(
                        argument,
                        Context::tail(Precedence::Yield, context.tail),
                    ));
                }
            }
            ExprData::CallExpression(node) => {
                self.dependency(node.callee(ast));
                for id in node.arguments(ast).iter() {
                    let argument = match ast.argument_data(ast.get_node_in_sub_range(id)) {
                        ArgumentData::Expr(e) => e,
                        ArgumentData::SpreadElement(s) => s.argument(ast),
                    };
                    self.dependency(argument);
                }
            }
            ExprData::TsInstantiationExpression(node) => self.dependency(node.expression(ast)),
            ExprData::UpdateExpression(node) if !node.prefix(ast) => {
                self.dependency(simple_assignment_target_expression(ast, node.argument(ast)))
            }
            ExprData::MemberExpression(node) => {
                self.dependency(node.object(ast));
                if node.computed(ast)
                    && let PropertyKeyData::Expr(key) = ast.property_key_data(node.property(ast))
                {
                    self.dependency(key);
                }
            }
            ExprData::ArrayExpression(node) => {
                for id in node.elements(ast).iter() {
                    if let Some(argument) = ast.get_node_in_sub_range(id) {
                        let expression = match ast.argument_data(argument) {
                            ArgumentData::Expr(e) => e,
                            ArgumentData::SpreadElement(s) => s.argument(ast),
                        };
                        self.dependency(expression);
                    }
                }
            }
            ExprData::ObjectExpression(node) => {
                for id in node.properties(ast).iter() {
                    match ast.object_property_kind_data(ast.get_node_in_sub_range(id)) {
                        ObjectPropertyKindData::SpreadElement(s) => {
                            self.dependency(s.argument(ast))
                        }
                        ObjectPropertyKindData::ObjectProperty(p) => {
                            self.dependency(p.value(ast));
                            if p.computed(ast)
                                && let PropertyKeyData::Expr(key) =
                                    ast.property_key_data(p.key(ast))
                            {
                                self.dependency(key);
                            }
                        }
                    }
                }
            }
            ExprData::ChainExpression(node) => self.dependency(node.expression(ast)),
            ExprData::TaggedTemplateExpression(node) => {
                self.dependency(node.tag(ast));
                self.dependency(Expr::TemplateLiteral(node.quasi(ast)));
            }
            ExprData::TemplateLiteral(node) => {
                for id in node.expressions(ast).iter() {
                    self.dependency(ast.get_node_in_sub_range(id));
                }
            }
            ExprData::JsxElement(_) | ExprData::JsxFragment(_) => {
                for child in jsx_expressions(ast, expression) {
                    self.dependency(child);
                }
            }
            ExprData::TsNonNullExpression(node) => self.dependency(node.expression(ast)),
            ExprData::TsAsExpression(node) => self.pending.push(Task::Plan(
                node.expression(ast),
                Context::tail(
                    Precedence::Compare,
                    self.type_parameter_suffix(node.type_annotation(ast))
                        .transfer(context.tail),
                ),
            )),
            ExprData::TsSatisfiesExpression(node) => self.pending.push(Task::Plan(
                node.expression(ast),
                Context::tail(
                    Precedence::Compare,
                    self.type_parameter_suffix(node.type_annotation(ast))
                        .transfer(context.tail),
                ),
            )),
            ExprData::ConditionalExpression(node) => {
                self.dependency(node.test(ast));
                self.dependency(node.consequent(ast));
                self.pending.push(Task::Plan(
                    node.alternate(ast),
                    Context::tail(Precedence::Yield, context.tail),
                ));
            }
            ExprData::ArrowFunctionExpression(node) => {
                if let ArrowFunctionBodyData::Expr(body) =
                    ast.arrow_function_body_data(node.body(ast))
                {
                    self.pending.push(Task::Plan(
                        body,
                        Context::tail(Precedence::Assignment, context.tail),
                    ));
                }
            }
            ExprData::SequenceExpression(node) => {
                for id in node.expressions(ast).iter() {
                    self.pending.push(Task::Plan(
                        ast.get_node_in_sub_range(id),
                        Context::tail(self.sequence_level(), context.tail),
                    ));
                }
            }
            _ => {}
        }
    }

    fn precedence(&self, expression: Expr) -> Precedence {
        match self.ast.expr_data(expression) {
            ExprData::BinaryExpression(binary) => Binaryish::Binary(binary).printed_precedence(
                self.ast,
                BinaryishOperator::Binary(binary.operator(self.ast)),
                self.minify,
            ),
            ExprData::LogicalExpression(binary) => Binaryish::Logical(binary).printed_precedence(
                self.ast,
                BinaryishOperator::Logical(binary.operator(self.ast)),
                self.minify,
            ),
            ExprData::AssignmentExpression(_) | ExprData::ArrowFunctionExpression(_) => {
                Precedence::Assignment
            }
            ExprData::ConditionalExpression(_) => Precedence::Conditional,
            ExprData::SequenceExpression(_) => Precedence::Lowest,
            ExprData::YieldExpression(_) => Precedence::Assignment,
            ExprData::UnaryExpression(_)
            | ExprData::UpdateExpression(_)
            | ExprData::AwaitExpression(_)
            | ExprData::TsTypeAssertion(_) => Precedence::Prefix,
            ExprData::TsAsExpression(_) | ExprData::TsSatisfiesExpression(_) => Precedence::Compare,
            ExprData::NewExpression(_) | ExprData::ChainExpression(_) => Precedence::New,
            _ => Precedence::Call,
        }
    }

    fn grouped(&self, expression: Expr, level: Precedence) -> bool {
        self.state(expression).grouped() || level > self.precedence(expression)
    }

    // A summary is interpreted in its parent's precedence context. Both base
    // printer groups and additional planned groups change the emitted prefix.
    fn first(&self, expression: Expr, level: Precedence) -> LeadingToken {
        if self.grouped(expression, level) {
            LeadingToken::Accept
        } else {
            self.summary(expression).state.first()
        }
    }

    fn first_with_instantiation_grouping(
        &self,
        expression: Expr,
        level: Precedence,
        no_instantiation: bool,
    ) -> LeadingToken {
        if no_instantiation
            && matches!(
                self.ast.expr_data(expression),
                ExprData::TsInstantiationExpression(_)
            )
        {
            LeadingToken::Accept
        } else {
            self.first(expression, level)
        }
    }

    fn can_follow(&mut self, left: Expr, expression: Expr, level: Precedence) -> bool {
        match self.first(expression, level) {
            LeadingToken::Accept => true,
            LeadingToken::Reject => false,
            LeadingToken::Expression => {
                let start = left.span(self.ast).end;
                let end = expression.span(self.ast).start;
                // Whitespace, closing angles, and removed grouping delimiters
                // cannot emit a line break. Everything else may be comment
                // text (including HTML comments), so consult the rank index.
                // Binary operand gaps are disjoint: this scan remains linear.
                if !self.comments
                    || !self
                        .ast
                        .source()
                        .as_bytes()
                        .get(start as usize..end as usize)
                        .is_some_and(|gap| {
                            gap.iter().any(|byte| {
                                !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'(' | b')')
                            })
                        })
                {
                    return false;
                }
                self.index_line_breaks();
                self.line_break_rank(end) > self.line_break_rank(start)
            }
        }
    }

    fn closing(operator: BinaryishOperator) -> bool {
        matches!(
            operator,
            BinaryishOperator::Binary(
                BinaryOperator::GreaterThan
                    | BinaryOperator::RightShift
                    | BinaryOperator::UnsignedRightShift
            )
        )
    }

    fn type_operator(operator: BinaryishOperator) -> bool {
        matches!(
            operator,
            BinaryishOperator::Binary(BinaryOperator::BitwiseOr | BinaryOperator::BitwiseAnd)
        )
    }

    // Only these roots can expose an opening angle through their printed tail.
    // Keep this aligned with the tail propagation in `plan`.
    fn can_expose_open(&self, expression: Expr) -> bool {
        matches!(
            self.ast.expr_data(expression),
            ExprData::BinaryExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::AssignmentExpression(_)
                | ExprData::ConditionalExpression(_)
                | ExprData::ArrowFunctionExpression(_)
                | ExprData::YieldExpression(_)
                | ExprData::SequenceExpression(_)
                | ExprData::TsAsExpression(_)
                | ExprData::TsSatisfiesExpression(_)
        )
    }

    // Barrier roots can be emitted immediately. Return true once the remaining
    // comma boundaries have been planned, or no continuation remains.
    pub(crate) fn try_arguments(&mut self, first: Expr, remaining: SubRange<Argument>) -> bool {
        if remaining.is_empty() {
            return true;
        }
        if !self.can_expose_open(first) {
            return false;
        }
        let ast = self.ast;
        self.plan_list_tail(
            first,
            Precedence::Assignment,
            remaining
                .iter()
                .rev()
                .map(|id| ListMember::argument(ast, Some(ast.get_node_in_sub_range(id)))),
        );
        true
    }

    fn arguments(&mut self, arguments: SubRange<Argument>) {
        let ast = self.ast;
        self.members(
            arguments
                .iter()
                .map(|id| ListMember::argument(ast, Some(ast.get_node_in_sub_range(id)))),
            Precedence::Assignment,
        );
    }

    pub(crate) fn declarators(&mut self, declarations: SubRange<VariableDeclarator>) {
        let ast = self.ast;
        self.members(
            declarations
                .iter()
                .map(|id| ListMember::Declaration(ast.get_node_in_sub_range(id))),
            Precedence::Assignment,
        );
    }

    pub(crate) fn parameters(&mut self, parameters: FormalParameters) {
        let ast = self.ast;
        self.members(
            parameters
                .items(ast)
                .iter()
                .map(|id| ListMember::Parameter(ast.get_node_in_sub_range(id))),
            Precedence::Assignment,
        );
    }

    pub(crate) fn array_pattern(&mut self, pattern: ArrayPattern) {
        let ast = self.ast;
        self.members(
            pattern.elements(ast).iter().map(|id| {
                ast.get_node_in_sub_range(id)
                    .map_or(ListMember::Barrier(None), ListMember::Binding)
            }),
            Precedence::Assignment,
        );
    }

    pub(crate) fn array(&mut self, elements: SubRange<Option<Argument>>) {
        let ast = self.ast;
        self.members(
            elements
                .iter()
                .map(|id| ListMember::argument(ast, ast.get_node_in_sub_range(id))),
            Precedence::Assignment,
        );
    }

    pub(crate) fn object_pattern(&mut self, pattern: ObjectPattern) {
        let ast = self.ast;
        self.members(
            pattern
                .properties(ast)
                .iter()
                .map(|id| ListMember::Property(ast.get_node_in_sub_range(id))),
            Precedence::Assignment,
        );
    }

    pub(crate) fn enum_members(&mut self, members: SubRange<TsEnumMember>) {
        let ast = self.ast;
        self.members(
            members
                .iter()
                .map(|id| ListMember::Enum(ast.get_node_in_sub_range(id))),
            Precedence::Assignment,
        );
    }

    // A comma is also a type-argument separator. Fold every list with the same
    // rule, including sequence expressions and dynamic import options.
    pub(crate) fn list(
        &mut self,
        expressions: impl DoubleEndedIterator<Item = Expr>,
        level: Precedence,
    ) {
        self.members(expressions.map(ListMember::Expression), level);
    }

    fn members(
        &mut self,
        mut expressions: impl DoubleEndedIterator<Item = ListMember>,
        level: Precedence,
    ) {
        if expressions.size_hint().1.is_some_and(|len| len < 2) {
            return;
        }
        // The last expression cannot open type arguments across a comma.
        let Some(last) = expressions.next_back() else {
            return;
        };
        // Nothing before the first possible opener needs continuation facts.
        let Some(first) = expressions.find_map(|member| {
            member
                .expression(self.ast)
                .filter(|&expression| self.can_expose_open(expression))
        }) else {
            return;
        };
        self.plan_list_tail(first, level, std::iter::once(last).chain(expressions.rev()));
    }

    #[inline(never)]
    fn plan_list_tail(
        &mut self,
        first: Expr,
        level: Precedence,
        expressions: impl Iterator<Item = ListMember>,
    ) {
        self.initialize();
        let mut continuation = 0;
        for member in expressions {
            if let Some(expression) = member.expression(self.ast) {
                let depth = self.pending.len();
                self.pending
                    .push(Task::Plan(expression, Context::tail(level, continuation)));
                self.run_since(depth);
            }
            continuation = member.syntax(self, level).before(continuation);
        }
        if continuation != 0 {
            let depth = self.pending.len();
            self.pending
                .push(Task::Plan(first, Context::tail(level, continuation)));
            self.run_since(depth);
        }
    }

    fn syntax(&self, expression: Expr, level: Precedence) -> TypeSyntax {
        let summary = self.summary(expression);
        let syntax = summary.syntax;
        if self.grouped(expression, level) {
            syntax.parenthesized()
        } else if summary.state.contains(State::BARE_ARROW)
            && let ExprData::ArrowFunctionExpression(node) = self.ast.expr_data(expression)
            && !node.r#async(self.ast)
        {
            // Without the enclosing `(`, a bare parameter ends the default
            // type before `=>`; recovery reads the body as a return type.
            syntax.with_entry(DEFAULT_TYPE, syntax, ARGUMENT)
        } else {
            syntax
        }
    }

    // After an identifier-like type name, `[` starts indexed access. At `(`,
    // recovery closes the generic parameters and enters function parameters;
    // otherwise it resumes the generic parameter list at the operand.
    fn after_type_name(&self, argument: Expr, level: Precedence, result: TypeSyntax) -> TypeSyntax {
        let (entry, residual, level) = self.type_entry(argument, level);
        let source = match entry {
            TypeEntry::Parameters => ARGUMENT,
            TypeEntry::Indexed => DEFAULT_TYPE,
            TypeEntry::Type => ARGUMENT,
            TypeEntry::Other => PARAMETER,
        };
        result.with_entry(DEFAULT_TYPE, self.syntax(residual, level), source)
    }

    // Only accepting prefixes are consumed, so fold the continuation backwards
    // without composing outgoing relation columns.
    fn argument_closes(&self, arguments: SubRange<Argument>) -> u8 {
        let ast = self.ast;
        arguments.iter().rev().fold(0, |closes, id| {
            let argument = match ast.argument_data(ast.get_node_in_sub_range(id)) {
                ArgumentData::Expr(expression) => expression,
                ArgumentData::SpreadElement(spread) => spread.argument(ast),
            };
            self.syntax(argument, Precedence::Assignment).before(closes)
        })
    }

    // `typeof` requires an entity name before it admits type suffixes. A
    // missing name recovers into a function return type. Reserved `new` is an
    // entity-name token here, so its remaining callee/arguments use the same
    // name/parameter recovery as a call rather than constructor-type recovery.
    fn type_query(&self, argument: Expr, syntax: TypeSyntax) -> TypeSyntax {
        let ast = self.ast;
        let (entry, residual, _) = self.type_entry(argument, Precedence::Prefix);
        if residual != argument {
            return self.after_type_name(argument, Precedence::Prefix, TypeSyntax::RECOVER);
        }
        if matches!(entry, TypeEntry::Parameters | TypeEntry::Type) {
            return TypeSyntax::RECOVER.with_entry(DEFAULT_TYPE, syntax, ARGUMENT);
        }
        if let ExprData::NewExpression(node) = ast.expr_data(argument) {
            let callee = node.callee(ast);
            let syntax = self.syntax(callee, Precedence::New);
            if matches!(
                self.type_entry(callee, Precedence::New).0,
                TypeEntry::Parameters
            ) {
                let continues = self.summary(callee).syntax.argument & SIGNATURE != 0;
                let arguments = self.argument_closes(node.arguments(ast));
                return TypeSyntax::RECOVER.with_entry(
                    DEFAULT_TYPE,
                    TypeSyntax {
                        closes: syntax.closes | if continues { arguments & ARGUMENT } else { 0 },
                        argument: if continues { ARGUMENT } else { 0 },
                        ..TypeSyntax::BARRIER
                    },
                    ARGUMENT,
                );
            }
            let syntax =
                if new_expression_has_argument_list(ast, node, self.minify, Precedence::Lowest) {
                    let arguments = self.argument_closes(node.arguments(ast));
                    syntax.then(TypeSyntax {
                        closes: if arguments & ARGUMENT != 0 {
                            PARAMETER
                        } else {
                            0
                        },
                        argument: PARAMETER,
                        ..TypeSyntax::BARRIER
                    })
                } else {
                    syntax
                };
            return TypeSyntax::RECOVER.with_entry(DEFAULT_TYPE, syntax, PARAMETER);
        }
        TypeSyntax::RECOVER.with_entry(DEFAULT_TYPE, syntax, DEFAULT_TYPE)
    }

    // An independent printer entry needs only boundary constraints. A parent
    // consuming this fragment also requests its complete syntax summary.
    fn plan_binary_right(&mut self, binary: PreparedBinary, summary_context: Option<Context>) {
        let operator = binary.operator;
        let right_level = binary.right_level;
        let prefix = match operator {
            BinaryishOperator::Binary(BinaryOperator::LessThan) => Prefix::Argument,
            BinaryishOperator::Binary(BinaryOperator::LeftShift) => Prefix::Parameter,
            _ => Prefix::None,
        };
        self.pending.push(Task::BinaryLeft(binary, summary_context));
        let context = summary_context.unwrap_or(Context::NONE);
        if Self::closing(operator)
            || Self::operator_transfer(operator).carries()
            || (context.tail != 0
                && self.can_expose_open(binary.right(self.ast))
                && !self.grouped(binary.right(self.ast), right_level))
        {
            self.pending.push(Task::Plan(
                binary.right(self.ast),
                Context {
                    level: right_level,
                    tail: context.tail,
                    prefix,
                },
            ));
        }
    }

    fn plan_binary_left(&mut self, binary: PreparedBinary, summary_context: Option<Context>) {
        let summarize = summary_context.is_some();
        let context = summary_context.unwrap_or(Context::NONE);
        let expression = binary.expression();
        let operator = binary.operator;
        let left = binary.left(self.ast);
        let right = binary.right(self.ast);
        let left_level = binary.left_level;
        let right_level = binary.right_level;
        let mut tail = 0;
        let mut prefix = Prefix::None;
        let closes = self.closing_prefix(left, right, operator, right_level);
        let syntax = self.operator_syntax(left, operator, closes);
        let carries = syntax.carries();
        let boundary = Self::closing(operator)
            || Self::type_operator(operator)
            || operator == BinaryishOperator::Binary(BinaryOperator::LessThan);
        // A barrier only contributes its already-known accepting prefix.
        // Otherwise both consumers share this finalized right-hand summary.
        let right_syntax = if carries && (context.tail != 0 || boundary) {
            self.syntax(right, right_level)
        } else {
            TypeSyntax::BARRIER
        };
        if context.tail != 0 && carries {
            let requirement = right_syntax.transfer(context.tail);
            let opening = match operator {
                BinaryishOperator::Binary(BinaryOperator::LessThan) => ARGUMENT,
                BinaryishOperator::Binary(BinaryOperator::LeftShift) => PARAMETER,
                _ => 0,
            };
            if opening & requirement != 0 {
                self.fragments[expression.node_id().index()].mark_grouped();
            } else {
                tail = syntax.transfer(requirement);
            }
        }
        if context.prefix.opening() != 0 && !self.grouped(expression, context.level) {
            if syntax.closes & context.prefix.opening() != 0 {
                self.fragments[expression.node_id().index()].mark_grouped();
            } else {
                prefix = context.prefix;
            }
        }
        if boundary {
            tail |= syntax.before(right_syntax.closes);
        }
        if summarize {
            self.pending.push(Task::FinishBinary(binary, closes));
        }
        if summarize || tail != 0 || prefix != Prefix::None {
            self.pending.push(Task::Plan(
                left,
                Context {
                    level: left_level,
                    tail,
                    prefix,
                },
            ));
        }
    }

    // Plan this boundary and return its additional TypeScript grouping decision.
    pub(crate) fn plan_binary(&mut self, prepared: PreparedBinary) -> bool {
        let binary = prepared.binary;
        let operator = prepared.operator;
        if !Self::closing(operator)
            && !Self::type_operator(operator)
            && !matches!(
                operator,
                BinaryishOperator::Binary(BinaryOperator::LessThan | BinaryOperator::LeftShift)
            )
        {
            return false;
        }
        let expression = match binary {
            Binaryish::Binary(node) => Expr::BinaryExpression(node),
            Binaryish::Logical(node) => Expr::LogicalExpression(node),
        };
        if self.needs_group(expression) {
            return true;
        }
        // A standalone boundary needs planning only when both sides can
        // participate. Parents that consume a fragment still plan its complete
        // dependencies; these gates only avoid work for independent expressions.
        // Base printer groups already isolate opening angles in the left operand.
        if (Self::closing(operator) || Self::type_operator(operator))
            && (!self.can_expose_open(binary.left(self.ast))
                || operator.precedence() > self.precedence(binary.left(self.ast)))
        {
            return false;
        }
        if !Self::closing(operator)
            && Self::terminal_syntax(self.ast.expr_data(binary.right(self.ast)))
                .is_some_and(|syntax| syntax.closes == 0)
        {
            return false;
        }
        self.initialize();
        if !self.state(expression).known() {
            self.plan_binary_right(prepared, None);
            self.run();
        }
        self.needs_group(expression)
    }

    // The right fragment's leading token and source gap are final before
    // planning the left operand. Carry this acceptance mask to synthesis.
    fn closing_prefix(
        &mut self,
        left: Expr,
        right: Expr,
        operator: BinaryishOperator,
        right_level: Precedence,
    ) -> u8 {
        if Self::closing(operator) && self.can_follow(left, right, right_level) {
            match operator {
                BinaryishOperator::Binary(BinaryOperator::GreaterThan) => ARGUMENT,
                BinaryishOperator::Binary(BinaryOperator::RightShift) => PARAMETER | DEFAULT_TYPE,
                _ => TYPE_SUFFIX,
            }
        } else {
            0
        }
    }

    fn operator_syntax(&self, left: Expr, operator: BinaryishOperator, closes: u8) -> TypeSyntax {
        let mut syntax = Self::operator_transfer(operator);
        if operator == BinaryishOperator::Binary(BinaryOperator::LessThan) {
            // Before the left dependency is finalized, retain both possible
            // suffix productions. The completed summary can use its actual
            // emitted tail, including any groups planned in that dependency.
            let suffix = if !self.state(left).known() && self.can_expose_open(left) {
                PARAMETER | TYPE_SUFFIX
            } else {
                self.angle_suffix(left, Precedence::Compare)
            };
            syntax.suffix &= !DEFAULT_TYPE;
            if suffix & TYPE_SUFFIX != 0 {
                syntax.suffix |= DEFAULT_TYPE;
            }
            if suffix & PARAMETER != 0 {
                syntax.parameter |= DEFAULT_TYPE;
            }
        }
        syntax.closes = closes;
        syntax
    }

    // A `<` stays within a trailing type reference/operator operand, but a
    // complete type ends the current default and enters recovered parameters.
    // Follow the printed tail rather than the JavaScript parent node: a chain
    // such as `a < b < c` still ends in the type name `c` during speculation.
    fn angle_suffix(&self, mut expression: Expr, mut level: Precedence) -> u8 {
        let ast = self.ast;
        loop {
            if self.grouped(expression, level) {
                return PARAMETER;
            }
            let data = ast.expr_data(expression);
            if let Some(binary) = Self::binary(data) {
                let operator = binary.operator(ast);
                level = binary.right_precedence(ast, operator);
                expression = binary.right(ast);
                continue;
            }
            return match data {
                ExprData::IdentifierReference(name) => {
                    if matches!(
                        ast.get_utf8(name.name(ast)),
                        "any"
                            | "unknown"
                            | "string"
                            | "number"
                            | "bigint"
                            | "symbol"
                            | "boolean"
                            | "undefined"
                            | "never"
                            | "object"
                            | "infer"
                    ) {
                        PARAMETER
                    } else {
                        TYPE_SUFFIX
                    }
                }
                ExprData::MemberExpression(member)
                    if !member.computed(ast) && !member.optional(ast) =>
                {
                    if self.entity_name(member.object(ast), false) {
                        TYPE_SUFFIX
                    } else {
                        PARAMETER | TYPE_SUFFIX
                    }
                }
                ExprData::UnaryExpression(node) if node.operator(ast) == UnaryOperator::Typeof => {
                    if self.entity_name(node.argument(ast), true) {
                        TYPE_SUFFIX
                    } else {
                        PARAMETER
                    }
                }
                ExprData::UnaryExpression(node)
                    if node.operator(ast) == UnaryOperator::LogicalNot =>
                {
                    expression = node.argument(ast);
                    level = Precedence::Prefix;
                    continue;
                }
                ExprData::TsTypeAssertion(node) => {
                    expression = node.expression(ast);
                    level = Precedence::Prefix;
                    continue;
                }
                ExprData::NewExpression(node) => {
                    if new_expression_has_argument_list(ast, node, self.minify, Precedence::Lowest)
                    {
                        PARAMETER
                    } else {
                        TYPE_SUFFIX
                    }
                }
                ExprData::ImportExpression(_) => TYPE_SUFFIX,
                ExprData::TsAsExpression(_) | ExprData::TsSatisfiesExpression(_) => {
                    PARAMETER | TYPE_SUFFIX
                }
                _ => PARAMETER,
            };
        }
    }

    fn entity_name(&self, mut expression: Expr, allow_this: bool) -> bool {
        loop {
            if self.grouped(expression, Precedence::Call) {
                return false;
            }
            match self.ast.expr_data(expression) {
                ExprData::IdentifierReference(_) => return true,
                ExprData::ThisExpression(_) => return allow_this,
                ExprData::MemberExpression(member)
                    if !member.computed(self.ast) && !member.optional(self.ast) =>
                {
                    expression = member.object(self.ast);
                }
                _ => return false,
            }
        }
    }

    fn operator_transfer(operator: BinaryishOperator) -> TypeSyntax {
        if Self::closing(operator) {
            // A close discharges a function's parameters, leaving an outer
            // type-argument requirement for the continuation to finish.
            TypeSyntax {
                closes: 0,
                argument: match operator {
                    BinaryishOperator::Binary(BinaryOperator::GreaterThan) => {
                        PARAMETER | DEFAULT_TYPE
                    }
                    BinaryishOperator::Binary(BinaryOperator::RightShift) => TYPE_SUFFIX,
                    _ => 0,
                },
                // A generic function's `>` is followed by its parameter
                // signature. This is the same entry used after `new`.
                signature: if operator == BinaryishOperator::Binary(BinaryOperator::GreaterThan) {
                    PARAMETER | DEFAULT_TYPE
                } else {
                    0
                },
                default_type: if operator == BinaryishOperator::Binary(BinaryOperator::GreaterThan)
                {
                    TYPE_SUFFIX
                } else {
                    0
                },
                ..TypeSyntax::BARRIER
            }
        } else if Self::type_operator(operator) {
            TypeSyntax {
                parameter: 0,
                ..TypeSyntax::RECOVER
            }
        } else if operator == BinaryishOperator::Binary(BinaryOperator::LessThan) {
            TypeSyntax {
                closes: 0,
                argument: ARGUMENT,
                parameter: ARGUMENT | PARAMETER,
                default_type: ARGUMENT,
                suffix: DEFAULT_TYPE | TYPE_SUFFIX,
                ..TypeSyntax::BARRIER
            }
        } else if operator == BinaryishOperator::Binary(BinaryOperator::LeftShift) {
            TypeSyntax {
                default_type: 0,
                // Nested generic parameters may recover into a complete type
                // before a later suffix, just like an ordinary type argument.
                suffix: BOTH | TYPE_SUFFIX | DEFAULT_TYPE,
                ..TypeSyntax::THROUGH
            }
        } else if operator == BinaryishOperator::Logical(LogicalOperator::NullishCoalescing) {
            // `??` ends a default type, then is rescanned as nullable prefixes
            // in the recovered function return type.
            TypeSyntax {
                argument: PARAMETER | DEFAULT_TYPE,
                ..TypeSyntax::BARRIER
            }
        } else {
            TypeSyntax::BARRIER
        }
    }

    fn finish(&mut self, expression: Expr, prefix: Prefix, data: ExprData) {
        // A parent consumes the final emitted fragments, including groups
        // required by a child's own comma boundaries.
        if !self.state(expression).known() {
            let ast = self.ast;
            match data {
                ExprData::CallExpression(node) => self.arguments(node.arguments(ast)),
                ExprData::NewExpression(node) => self.arguments(node.arguments(ast)),
                ExprData::ArrayExpression(node) => self.array(node.elements(ast)),
                ExprData::SequenceExpression(node) => self.list(
                    node.expressions(ast)
                        .iter()
                        .map(|id| ast.get_node_in_sub_range(id)),
                    self.sequence_level(),
                ),
                ExprData::ImportExpression(node) => self.list(
                    std::iter::once(node.source(ast)).chain(node.options(ast)),
                    Precedence::Assignment,
                ),
                _ => {}
            }
        }
        if prefix != Prefix::None
            && let ExprData::ArrowFunctionExpression(node) = data
            && let ArrowFunctionBodyData::Expr(body) =
                self.ast.arrow_function_body_data(node.body(self.ast))
            && self.syntax(body, Precedence::Assignment).close_mask() != 0
            && let Some(identifier) = bare_arrow_parameter(self.ast, node, true)
            && node.params(self.ast).span(self.ast).start == identifier.span(self.ast).start
        {
            self.fragments[expression.node_id().index()].insert(State::BARE_ARROW);
        }
        if let ExprData::ArrowFunctionExpression(node) = data
            && !node.r#async(self.ast)
            && bare_arrow_parameter(
                self.ast,
                node,
                self.minify || self.bare_arrow_parameters(node),
            )
            .is_some()
        {
            self.fragments[expression.node_id().index()].insert(State::BARE_ARROW);
        }
        let first = self.leading_token(data);
        let syntax = self.type_syntax(expression, data);
        self.record_summary(expression, first, syntax);
    }

    fn record_summary(&mut self, expression: Expr, first: LeadingToken, syntax: TypeSyntax) {
        let fragment = &mut self.fragments[expression.node_id().index()];
        fragment.finish(first, syntax);
        fragment.set_known();
    }

    // Mirror the printer's token choices, independently of type grammar.
    fn leading_token(&self, data: ExprData) -> LeadingToken {
        let ast = self.ast;
        if let Some(prefix) = self.printed_prefix(data) {
            return match prefix {
                PrintedPrefix::Parenthesis => LeadingToken::Accept,
                PrintedPrefix::Child(expression, level, no_instantiation) => {
                    self.first_with_instantiation_grouping(expression, level, no_instantiation)
                }
            };
        }
        match data {
            ExprData::ParenthesizedExpression(_)
            | ExprData::TemplateLiteral(_)
            | ExprData::RegExpLiteral(_) => LeadingToken::Accept,
            ExprData::IdentifierReference(node) => {
                // These identifier spellings are also relational operator tokens
                // after a closing angle, where TypeScript favors type arguments.
                if matches!(ast.get_utf8(node.name(ast)), "as" | "satisfies") {
                    LeadingToken::Accept
                } else {
                    LeadingToken::Expression
                }
            }
            ExprData::ArrowFunctionExpression(node) => {
                if node.r#async(ast) {
                    LeadingToken::Expression
                } else if node.type_parameters(ast).is_some() {
                    LeadingToken::Reject
                } else if bare_arrow_parameter(
                    ast,
                    node,
                    self.minify || self.bare_arrow_parameters(node),
                )
                .is_some()
                {
                    LeadingToken::Expression
                } else {
                    LeadingToken::Accept
                }
            }
            ExprData::StringLiteral(node) => {
                if choose_quote(ast.get_wtf8(node.value(ast)), self.minify) == b'`' {
                    LeadingToken::Accept
                } else {
                    LeadingToken::Expression
                }
            }
            ExprData::UnaryExpression(node) => {
                if matches!(
                    node.operator(ast),
                    UnaryOperator::Positive | UnaryOperator::Negate
                ) {
                    LeadingToken::Reject
                } else {
                    LeadingToken::Expression
                }
            }
            ExprData::TsTypeAssertion(_) | ExprData::JsxElement(_) | ExprData::JsxFragment(_) => {
                LeadingToken::Reject
            }
            _ => LeadingToken::Expression,
        }
    }

    // A prefix path is printer policy: wrappers emit their child's first token
    // unless precedence, instantiation syntax, or `let[` requires a group.
    fn printed_prefix(&self, data: ExprData) -> Option<PrintedPrefix> {
        let ast = self.ast;
        if let Some(binary) = Self::binary(data) {
            let operator = binary.operator(ast);
            return Some(PrintedPrefix::Child(
                binary.left(ast),
                binary.left_precedence(ast, operator, self.minify),
                operator
                    .following_context()
                    .contains(ExprContext::NO_INSTANTIATION),
            ));
        }
        let (expression, level, no_instantiation) = match data {
            ExprData::TsInstantiationExpression(node) => {
                (node.expression(ast), Precedence::Call, true)
            }
            ExprData::UpdateExpression(node) if !node.prefix(ast) => (
                simple_assignment_target_expression(ast, node.argument(ast)),
                Precedence::Postfix,
                false,
            ),
            ExprData::CallExpression(node) => {
                let callee = node.callee(ast);
                (
                    callee,
                    if matches!(ast.expr_data(callee), ExprData::NewExpression(_)) {
                        Precedence::New
                    } else {
                        Precedence::Call
                    },
                    !node.optional(ast),
                )
            }
            ExprData::MemberExpression(node) => {
                let object = node.object(ast);
                if node.computed(ast)
                    && matches!(ast.expr_data(object), ExprData::IdentifierReference(id) if ast.get_utf8(id.name(ast)) == "let")
                {
                    return Some(PrintedPrefix::Parenthesis);
                }
                (
                    object,
                    if matches!(ast.expr_data(object), ExprData::NewExpression(_)) {
                        Precedence::New
                    } else {
                        Precedence::Call
                    },
                    !node.optional(ast) || !node.computed(ast),
                )
            }
            ExprData::ChainExpression(node) => (node.expression(ast), Precedence::Lowest, false),
            ExprData::TaggedTemplateExpression(node) => {
                let tag = node.tag(ast);
                (
                    tag,
                    if node.type_arguments(ast).is_some()
                        || matches!(ast.expr_data(tag), ExprData::ChainExpression(_))
                    {
                        Precedence::Call
                    } else {
                        Precedence::Postfix
                    },
                    true,
                )
            }
            ExprData::TsNonNullExpression(node) => {
                let operand = node.expression(ast);
                (
                    operand,
                    if matches!(ast.expr_data(operand), ExprData::ChainExpression(_)) {
                        Precedence::Call
                    } else {
                        Precedence::Postfix
                    },
                    true,
                )
            }
            ExprData::TsAsExpression(node) => (node.expression(ast), Precedence::Compare, false),
            ExprData::TsSatisfiesExpression(node) => {
                (node.expression(ast), Precedence::Compare, false)
            }
            ExprData::ConditionalExpression(node) => {
                (node.test(ast), Precedence::Conditional.next(), false)
            }
            ExprData::SequenceExpression(node) => (
                node.expressions(ast).get_node(ast, 0)?,
                self.sequence_level(),
                false,
            ),
            _ => return None,
        };
        Some(PrintedPrefix::Child(expression, level, no_instantiation))
    }

    // Only keyword/type-prefix recovery needs this finer distinction. Walk
    // transparent prefixes locally instead of adding another cached token flag.
    fn type_entry(
        &self,
        mut expression: Expr,
        mut level: Precedence,
    ) -> (TypeEntry, Expr, Precedence) {
        while !self.grouped(expression, level)
            && let ExprData::UnaryExpression(node) = self.ast.expr_data(expression)
            && node.operator(self.ast) == UnaryOperator::LogicalNot
        {
            expression = node.argument(self.ast);
            level = Precedence::Prefix;
        }
        let residual = (expression, level);
        let entry = loop {
            if self.grouped(expression, level) {
                break TypeEntry::Parameters;
            }
            let data = self.ast.expr_data(expression);
            if let Some(prefix) = self.printed_prefix(data) {
                match prefix {
                    PrintedPrefix::Parenthesis => break TypeEntry::Parameters,
                    PrintedPrefix::Child(child, child_level, no_instantiation) => {
                        if no_instantiation
                            && matches!(
                                self.ast.expr_data(child),
                                ExprData::TsInstantiationExpression(_)
                            )
                        {
                            break TypeEntry::Parameters;
                        }
                        expression = child;
                        level = child_level;
                        continue;
                    }
                }
            }
            break match data {
                ExprData::ArrayExpression(_) => TypeEntry::Indexed,
                ExprData::NumericLiteral(_)
                | ExprData::BigIntLiteral(_)
                | ExprData::StringLiteral(_)
                | ExprData::TemplateLiteral(_) => TypeEntry::Type,
                ExprData::UnaryExpression(node)
                    if node.operator(self.ast) == UnaryOperator::Negate
                        && matches!(
                            self.ast.expr_data(node.argument(self.ast)),
                            ExprData::NumericLiteral(_) | ExprData::BigIntLiteral(_)
                        ) =>
                {
                    TypeEntry::Type
                }
                ExprData::ParenthesizedExpression(_) => TypeEntry::Parameters,
                ExprData::ArrowFunctionExpression(node)
                    if !node.r#async(self.ast)
                        && node.type_parameters(self.ast).is_none()
                        && bare_arrow_parameter(
                            self.ast,
                            node,
                            self.minify || self.bare_arrow_parameters(node),
                        )
                        .is_none() =>
                {
                    TypeEntry::Parameters
                }
                _ => TypeEntry::Other,
            };
        };
        (entry, residual.0, residual.1)
    }

    fn sequence_level(&self) -> Precedence {
        if self.minify {
            Precedence::Lowest
        } else {
            Precedence::Assignment
        }
    }

    // After a recovered parameter name, identifier-like type tokens can be
    // more parameter names. Other type productions end that list and recover
    // into a function return type; a conditional's `? :` stops that recovery.
    fn type_parameter_suffix(&self, mut annotation: TsType) -> TypeSyntax {
        let ast = self.ast;
        loop {
            let names = match ast.ts_type_data(annotation) {
                TsTypeData::TsTypeReference(node) => {
                    node.type_arguments(ast).is_none()
                        && matches!(
                            ast.ts_type_name_data(node.type_name(ast)),
                            TsTypeNameData::IdentifierReference(_)
                        )
                }
                TsTypeData::TsTypeOperator(node) => {
                    annotation = node.type_annotation(ast);
                    continue;
                }
                TsTypeData::TsAnyKeyword(_)
                | TsTypeData::TsUnknownKeyword(_)
                | TsTypeData::TsNeverKeyword(_)
                | TsTypeData::TsUndefinedKeyword(_)
                | TsTypeData::TsStringKeyword(_)
                | TsTypeData::TsNumberKeyword(_)
                | TsTypeData::TsBigIntKeyword(_)
                | TsTypeData::TsBooleanKeyword(_)
                | TsTypeData::TsSymbolKeyword(_)
                | TsTypeData::TsObjectKeyword(_)
                | TsTypeData::TsIntrinsicKeyword(_) => true,
                TsTypeData::TsConditionalType(_) => return TypeSyntax::BARRIER,
                _ => false,
            };
            return if names {
                TypeSyntax {
                    parameter: PARAMETER | DEFAULT_TYPE,
                    suffix: TYPE_SUFFIX,
                    ..TypeSyntax::BARRIER
                }
            } else {
                TypeSyntax {
                    argument: PARAMETER | DEFAULT_TYPE,
                    parameter: TYPE_SUFFIX,
                    suffix: TYPE_SUFFIX,
                    ..TypeSyntax::BARRIER
                }
            };
        }
    }

    fn assertion_syntax(&self, expression: Expr, annotation: TsType) -> TypeSyntax {
        let operand = self.syntax(expression, Precedence::Compare);
        operand
            .then(self.type_parameter_suffix(annotation))
            // Within a function/constructor signature these contextual names
            // can be additional recovered parameters, rather than operators.
            .with_signature(operand.argument & SIGNATURE != 0)
    }

    // Synthesize both facts from the same printed child boundaries after all
    // dependencies are finalized. Base grouping levels come from the task;
    // planned groups and grammar transfers are read at this final stage.
    fn binary_summary(&self, binary: PreparedBinary, closes: u8) -> (LeadingToken, TypeSyntax) {
        let operator = binary.operator;
        let left = binary.left(self.ast);
        let right = binary.right(self.ast);
        let left_level = binary.left_level;
        let right_level = binary.right_level;
        let first = self.first_with_instantiation_grouping(
            left,
            left_level,
            operator
                .following_context()
                .contains(ExprContext::NO_INSTANTIATION),
        );
        let operator_syntax = self.operator_syntax(left, operator, closes);
        // Once the operator cuts off speculative parsing, the right operand
        // cannot contribute to this fragment's transfer.
        let left_syntax = self.syntax(left, left_level);
        let syntax = if operator_syntax.carries() {
            let right = self.syntax(right, right_level);
            let mut syntax = left_syntax.then(operator_syntax).then(right);
            // In a function parameter signature, `??` ends the parameter list
            // and is rescanned as nullable return-type prefixes. An ordinary
            // type entry instead stops at the same token.
            if operator == BinaryishOperator::Logical(LogicalOperator::NullishCoalescing)
                && left_syntax.argument & SIGNATURE != 0
                && right.closes & ARGUMENT != 0
            {
                syntax.closes |= SIGNATURE;
            }
            syntax
        } else {
            TypeSyntax {
                closes: left_syntax.before(operator_syntax.closes),
                ..TypeSyntax::BARRIER
            }
        };
        (first, syntax)
    }

    // Terminals have complete grammar facts without child planning. The same
    // facts rule out independent boundaries and directly finish worklist leaves.
    fn terminal_syntax(data: ExprData) -> Option<TypeSyntax> {
        Some(match data {
            ExprData::IdentifierReference(_) => TypeSyntax::NAME,
            ExprData::NumericLiteral(_)
            | ExprData::StringLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::BigIntLiteral(_) => TypeSyntax::TYPE,
            ExprData::RegExpLiteral(_) => TypeSyntax::BARRIER,
            ExprData::Function(_) | ExprData::Class(_) => TypeSyntax {
                default_type: 0,
                suffix: 0,
                ..TypeSyntax::RECOVER
            },
            ExprData::ThisExpression(_)
            | ExprData::Super(_)
            | ExprData::MetaProperty(_)
            | ExprData::PrivateIdentifier(_) => TypeSyntax::RECOVER,
            _ => return None,
        })
    }

    // TypeScript's parseType/parseTypeParameter productions also perform error
    // recovery during speculative parsing. Keep these grammar transfers apart
    // from emitted-token classification and from the grouping worklist.
    fn type_syntax(&self, expression: Expr, data: ExprData) -> TypeSyntax {
        let ast = self.ast;
        match data {
            ExprData::ParenthesizedExpression(node) => self
                .syntax(node.expression(ast), Precedence::Lowest)
                .parenthesized(),
            ExprData::AssignmentExpression(node) => {
                if node.operator(ast) == AssignmentOperator::Assign {
                    // `=` is a parameter default, never part of a type argument.
                    self.syntax(node.right(ast), Precedence::Assignment)
                        .parameter_default()
                        .with_signature(true)
                } else {
                    TypeSyntax::BARRIER
                }
            }
            ExprData::CallExpression(node) => {
                // Recovering the missing generic-parameter `>` can make a call's
                // arguments function-type parameters. That path starts at `<<`.
                let arguments = self.argument_closes(node.arguments(ast));
                let callee = self.syntax(node.callee(ast), Precedence::Call);
                TypeSyntax {
                    closes: callee.closes
                        | if !node.optional(ast) && arguments & ARGUMENT != 0 {
                            PARAMETER
                        } else {
                            0
                        }
                        | if !node.optional(ast) && arguments & ARGUMENT != 0 {
                            callee.default_type
                        } else {
                            0
                        },
                    argument: if node.optional(ast) {
                        0
                    } else {
                        PARAMETER | DEFAULT_TYPE | TYPE_SUFFIX
                    },
                    ..TypeSyntax::BARRIER
                }
            }
            ExprData::MemberExpression(node) => {
                let object = self.syntax(node.object(ast), Precedence::Call);
                let qualified = match ast.expr_data(node.object(ast)) {
                    ExprData::IdentifierReference(_) | ExprData::ThisExpression(_) => true,
                    ExprData::MemberExpression(member) => {
                        !member.computed(ast) && !member.optional(ast)
                    }
                    _ => false,
                };
                let index = if node.computed(ast)
                    && let PropertyKeyData::Expr(key) = ast.property_key_data(node.property(ast))
                {
                    self.syntax(key, Precedence::Lowest).closes
                } else {
                    0
                };
                TypeSyntax {
                    closes: object.closes
                        | if !node.optional(ast) && index & ARGUMENT != 0 {
                            BOTH
                        } else {
                            0
                        }
                        | if node.optional(ast) {
                            0
                        } else {
                            object.transfer(index & (DEFAULT_TYPE | TYPE_SUFFIX))
                        },
                    default_type: if node.optional(ast) || (!node.computed(ast) && !qualified) {
                        0
                    } else {
                        object.default_type
                    },
                    suffix: if node.optional(ast) || (!node.computed(ast) && !qualified) {
                        0
                    } else {
                        object.suffix
                    },
                    argument: if node.optional(ast) {
                        0
                    } else {
                        TypeSyntax::RECOVER.argument
                    },
                    parameter: if node.computed(ast) || node.optional(ast) {
                        0
                    } else {
                        object.parameter
                    },
                    ..TypeSyntax::RECOVER
                }
                .with_signature(!node.optional(ast) && object.argument & SIGNATURE != 0)
            }
            // Tuple and object types parse their contents before demanding the
            // closing bracket/brace. Missing delimiters do not end speculation.
            ExprData::ArrayExpression(node) => {
                let mut syntax = TypeSyntax {
                    argument: TypeSyntax::TYPE.argument | TYPE_SUFFIX | SIGNATURE,
                    ..TypeSyntax::TYPE
                };
                for id in node.elements(ast).iter() {
                    if let Some(argument) = ast.get_node_in_sub_range(id) {
                        let expression = match ast.argument_data(argument) {
                            ArgumentData::Expr(e) => e,
                            ArgumentData::SpreadElement(s) => s.argument(ast),
                        };
                        syntax = syntax.then(self.syntax(expression, Precedence::Assignment));
                    }
                }
                syntax
            }
            ExprData::ObjectExpression(node) => {
                let mut closes = 0;
                let mut type_closes = 0;
                for id in node.properties(ast).iter() {
                    match ast.object_property_kind_data(ast.get_node_in_sub_range(id)) {
                        ObjectPropertyKindData::SpreadElement(s) => {
                            closes |= self.syntax(s.argument(ast), Precedence::Assignment).closes
                        }
                        ObjectPropertyKindData::ObjectProperty(p) => {
                            let value = self.syntax(p.value(ast), Precedence::Assignment).closes;
                            closes |= value;
                            type_closes |= value;
                            if p.computed(ast)
                                && let PropertyKeyData::Expr(key) =
                                    ast.property_key_data(p.key(ast))
                            {
                                closes |= self.syntax(key, Precedence::Lowest).closes;
                            }
                        }
                    }
                }
                TypeSyntax {
                    closes: TypeSyntax::TYPE.carry(closes)
                        | (type_closes & (TYPE_SUFFIX | DEFAULT_TYPE)),
                    ..TypeSyntax::TYPE
                }
                .with_signature(true)
            }
            ExprData::ChainExpression(node) => {
                self.syntax(node.expression(ast), Precedence::Lowest)
            }
            ExprData::TsNonNullExpression(node) => TypeSyntax {
                closes: self
                    .syntax(node.expression(ast), Precedence::Postfix)
                    .closes,
                ..TypeSyntax::RECOVER
            },
            // A contextual assertion ends an ordinary type argument. Generic
            // parameter recovery can instead read `as`/`satisfies` as another
            // parameter name and continue into the printed type annotation.
            ExprData::TsAsExpression(node) => {
                self.assertion_syntax(node.expression(ast), node.type_annotation(ast))
            }
            ExprData::TsSatisfiesExpression(node) => {
                self.assertion_syntax(node.expression(ast), node.type_annotation(ast))
            }
            ExprData::TsTypeAssertion(node) => TypeSyntax {
                closes: self.syntax(node.expression(ast), Precedence::Prefix).closes
                    & (DEFAULT_TYPE | TYPE_SUFFIX),
                ..TypeSyntax::RECOVER
            },
            ExprData::ImportExpression(node) => {
                let source = self.syntax(node.source(ast), Precedence::Assignment);
                TypeSyntax {
                    closes: TypeSyntax::TYPE.carry(source.closes)
                        | (source.closes & (DEFAULT_TYPE | TYPE_SUFFIX)),
                    ..TypeSyntax::TYPE
                }
            }
            ExprData::ConditionalExpression(node) => {
                let test = self.syntax(node.test(ast), Precedence::Conditional.next());
                let consequent = self.syntax(node.consequent(ast), Precedence::Assignment);
                let alternate = self.syntax(node.alternate(ast), Precedence::Assignment);
                // A default type stops before `?`. Recovery closes the generic
                // parameters, then reads `?` as a nullable function return type.
                TypeSyntax {
                    closes: test.closes
                        | if consequent.closes & ARGUMENT != 0 {
                            BOTH | DEFAULT_TYPE
                        } else {
                            0
                        }
                        | (alternate.closes & ARGUMENT),
                    ..TypeSyntax::BARRIER
                }
                .with_signature(true)
            }
            ExprData::UnaryExpression(node) => {
                let argument = node.argument(ast);
                if !matches!(
                    node.operator(ast),
                    UnaryOperator::Typeof
                        | UnaryOperator::LogicalNot
                        | UnaryOperator::Void
                        | UnaryOperator::Delete
                ) {
                    return TypeSyntax {
                        default_type: if node.operator(ast) == UnaryOperator::Negate
                            && matches!(
                                ast.expr_data(argument),
                                ExprData::NumericLiteral(_) | ExprData::BigIntLiteral(_)
                            ) {
                            DEFAULT_TYPE
                        } else {
                            0
                        },
                        ..TypeSyntax::RECOVER
                    };
                }
                let syntax = self.syntax(argument, Precedence::Prefix);
                if node.operator(ast) == UnaryOperator::LogicalNot {
                    // Prefix `!` recursively parses a non-nullable type.
                    return TypeSyntax {
                        parameter: 0,
                        ..TypeSyntax::RECOVER
                    }
                    .then(syntax);
                }
                if node.operator(ast) != UnaryOperator::Typeof {
                    return self.after_type_name(argument, Precedence::Prefix, TypeSyntax::RECOVER);
                }
                self.type_query(argument, syntax)
            }
            ExprData::ArrowFunctionExpression(node) => {
                let body = match ast.arrow_function_body_data(node.body(ast)) {
                    ArrowFunctionBodyData::Expr(body) => self.syntax(body, Precedence::Assignment),
                    _ => TypeSyntax::BARRIER,
                };
                let closes = TypeSyntax::TYPE.carry(body.closes);
                if node.r#async(ast) {
                    // `async` is a type/parameter name, not a function-type
                    // modifier. Only parameter-list recovery reaches the body,
                    // after discharging the inner generic-parameter angle.
                    let mut syntax = TypeSyntax {
                        argument: PARAMETER | DEFAULT_TYPE,
                        ..TypeSyntax::BARRIER
                    }
                    .then(body);
                    if syntax.close_mask() != 0 {
                        syntax.closes |= SIGNATURE;
                    }
                    return syntax;
                }
                // Recovery inside constructor parameters may enter the arrow's
                // return expression before rejecting the remaining signature.
                TypeSyntax {
                    closes: closes
                        | if closes != 0 { SIGNATURE } else { 0 }
                        | (body.closes & DEFAULT_TYPE),
                    ..TypeSyntax::TYPE
                }
            }
            ExprData::SequenceExpression(node) => {
                node.expressions(ast)
                    .iter()
                    .fold(TypeSyntax::THROUGH, |syntax, id| {
                        syntax
                            .then(self.syntax(ast.get_node_in_sub_range(id), self.sequence_level()))
                    })
            }
            ExprData::TemplateLiteral(node) => {
                node.expressions(ast)
                    .iter()
                    .fold(TypeSyntax::TYPE, |syntax, id| {
                        syntax.then(self.syntax(ast.get_node_in_sub_range(id), Precedence::Lowest))
                    })
            }
            ExprData::TaggedTemplateExpression(node) => {
                let tag = self.syntax(node.tag(ast), Precedence::Postfix);
                let template = self.summary(Expr::TemplateLiteral(node.quasi(ast))).syntax;
                TypeSyntax {
                    closes: tag.closes
                        | if template.closes & ARGUMENT != 0 {
                            PARAMETER
                        } else {
                            0
                        }
                        | if template.closes & ARGUMENT != 0 {
                            tag.default_type
                        } else {
                            0
                        },
                    argument: PARAMETER | DEFAULT_TYPE,
                    ..TypeSyntax::BARRIER
                }
            }
            ExprData::JsxElement(_) | ExprData::JsxFragment(_) => {
                // In type syntax the initial `<` starts another parameter list.
                // A close reached inside an attribute must discharge that list
                // as well as the outer type argument, hence the PARAMETER path.
                let closes = jsx_expressions(ast, expression).fold(0, |closes, child| {
                    closes | self.syntax(child, Precedence::Lowest).closes
                });
                TypeSyntax {
                    closes: if closes & PARAMETER != 0 { BOTH } else { 0 }
                        | if closes & TYPE_SUFFIX != 0 {
                            DEFAULT_TYPE
                        } else {
                            0
                        },
                    default_type: 0,
                    ..TypeSyntax::RECOVER
                }
            }
            ExprData::YieldExpression(node) => {
                if let Some(argument) = node.argument(ast) {
                    if node.delegate(ast) {
                        // After a recovered parameter name, `*` is a complete
                        // return type. Only indexed-access suffixes can extend
                        // it into the operand; a following name or `(` stops it.
                        let (entry, residual, level) = self.type_entry(argument, Precedence::Yield);
                        return if matches!(entry, TypeEntry::Indexed) {
                            TypeSyntax::BARRIER.with_entry(
                                PARAMETER | DEFAULT_TYPE | SIGNATURE,
                                self.syntax(residual, level),
                                ARGUMENT,
                            )
                        } else {
                            TypeSyntax::BARRIER
                        };
                    }
                    // The keyword can be a parameter name; a following operand
                    // can start a function return type after missing delimiters.
                    let operand = self.syntax(argument, Precedence::Yield);
                    self.after_type_name(
                        argument,
                        Precedence::Yield,
                        TypeSyntax::PARAMETER_PREFIX.then(operand),
                    )
                } else {
                    TypeSyntax::NAME
                }
            }
            ExprData::AwaitExpression(node) => {
                let argument = node.argument(ast);
                let syntax = self.syntax(argument, Precedence::Prefix);
                let result = if self.grouped(argument, Precedence::Prefix)
                    || matches!(
                        ast.expr_data(argument),
                        ExprData::ParenthesizedExpression(_)
                    ) {
                    let mut constructor = syntax.constructor();
                    constructor.argument &= PARAMETER;
                    constructor.closes &= PARAMETER;
                    constructor
                } else {
                    TypeSyntax::PARAMETER_PREFIX.then(syntax)
                };
                self.after_type_name(argument, Precedence::Prefix, result)
            }
            ExprData::NewExpression(node) => {
                let callee = self.syntax(node.callee(ast), Precedence::New);
                let mut syntax = callee.constructor();
                let parameters = self.grouped(node.callee(ast), Precedence::New)
                    || matches!(
                        ast.expr_data(node.callee(ast)),
                        ExprData::ParenthesizedExpression(_)
                    );
                let arguments =
                    new_expression_has_argument_list(ast, node, self.minify, Precedence::Lowest);
                if !parameters && arguments {
                    syntax = syntax.then(TypeSyntax::BARRIER);
                    syntax.argument |= DEFAULT_TYPE | TYPE_SUFFIX;
                } else {
                    syntax.default_type = DEFAULT_TYPE;
                    syntax.suffix = TYPE_SUFFIX;
                }
                syntax.closes |= callee.closes & (DEFAULT_TYPE | TYPE_SUFFIX);
                for id in node.arguments(ast).iter() {
                    let argument = match ast.argument_data(ast.get_node_in_sub_range(id)) {
                        ArgumentData::Expr(e) => e,
                        ArgumentData::SpreadElement(s) => s.argument(ast),
                    };
                    let closes = self.syntax(argument, Precedence::Assignment).closes;
                    if closes & if parameters { DEFAULT_TYPE } else { ARGUMENT } != 0 {
                        syntax.closes |= DEFAULT_TYPE;
                    }
                }
                syntax
            }
            ExprData::UpdateExpression(_) => TypeSyntax::BARRIER,
            ExprData::TsInstantiationExpression(node) => {
                let mut syntax = self.syntax(node.expression(ast), Precedence::Call);
                syntax.closes |= SIGNATURE;
                syntax.default_type = DEFAULT_TYPE;
                syntax.suffix = TYPE_SUFFIX;
                syntax
            }
            _ => Self::terminal_syntax(data).unwrap_or(TypeSyntax::RECOVER),
        }
    }
}

// JSX has expression children in two distinct lists. Expose those dependencies
// once for both planning and synthesis, without allocating an intermediate list.
fn jsx_expressions<'a>(ast: &'a Ast<'a>, expression: Expr) -> impl Iterator<Item = Expr> + 'a {
    let (attributes, children) = match ast.expr_data(expression) {
        ExprData::JsxElement(node) => (
            node.opening_element(ast).attributes(ast),
            node.children(ast),
        ),
        ExprData::JsxFragment(node) => (SubRange::empty(), node.children(ast)),
        _ => unreachable!("JSX dependency traversal requires a JSX expression"),
    };
    let attributes = attributes.iter().filter_map(move |id| {
        match ast.jsx_attribute_item_data(ast.get_node_in_sub_range(id)) {
            JsxAttributeItemData::JsxSpreadAttribute(node) => Some(node.argument(ast)),
            JsxAttributeItemData::JsxAttribute(node) => {
                match ast.jsx_attribute_value_data(node.value(ast)?) {
                    JsxAttributeValueData::JsxExpressionContainer(node) => jsx_container(ast, node),
                    JsxAttributeValueData::JsxElement(node) => Some(Expr::JsxElement(node)),
                    JsxAttributeValueData::JsxFragment(node) => Some(Expr::JsxFragment(node)),
                    JsxAttributeValueData::StringLiteral(_) => None,
                }
            }
        }
    });
    let children = children.iter().filter_map(move |id| {
        match ast.jsx_child_data(ast.get_node_in_sub_range(id)) {
            JsxChildData::JsxExpressionContainer(node) => jsx_container(ast, node),
            JsxChildData::JsxSpreadChild(node) => Some(node.expression(ast)),
            JsxChildData::JsxElement(node) => Some(Expr::JsxElement(node)),
            JsxChildData::JsxFragment(node) => Some(Expr::JsxFragment(node)),
            JsxChildData::JsxText(_) => None,
        }
    });
    attributes.chain(children)
}

fn jsx_container(ast: &Ast<'_>, node: JsxExpressionContainer) -> Option<Expr> {
    match ast.jsx_expression_data(node.expression(ast)) {
        JsxExpressionData::Expr(expression) => Some(expression),
        JsxExpressionData::JsxEmptyExpression(_) => None,
    }
}
