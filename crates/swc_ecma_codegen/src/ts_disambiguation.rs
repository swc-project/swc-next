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

use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};
use swc_next_ecma_ast::*;

use crate::{
    operator::{Binaryish, BinaryishOperator, ExprContext},
    util::{
        bare_arrow_parameter, choose_quote, new_expression_has_argument_list,
        pure_comment_annotation, simple_assignment_target_expression,
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

    fn set_known(&mut self) {
        self.insert(Self::KNOWN);
    }

    fn grouped(self) -> bool {
        self.contains(Self::GROUPED)
    }

    fn mark_grouped(&mut self) {
        self.insert(Self::GROUPED);
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
// Additional grammar entry positions share the existing three-byte transfer.
// They are not planning/cache-validity flags.
const TYPE_SUFFIX: u8 = 1 << 2;
const CONSTRUCTOR: u8 = 1 << 3;
const GRAMMAR_POSITIONS: u8 = BOTH | TYPE_SUFFIX | CONSTRUCTOR;

// A conservative relation between grammar entry positions before and after
// emitted tokens. In addition to arguments and generic parameters, composition
// distinguishes a suffix after a complete type and a constructor-type entry.
// `closes` records accepting prefixes; each remaining byte packs two columns.
// Boolean relation composition is associative, so every fragment uses `then`.
// Grammar positions are projected out before querying the protection cache.
#[derive(Clone, Copy, Default)]
struct TypeSyntax {
    closes: u8,
    argument: u8,
    parameter: u8,
}

impl TypeSyntax {
    fn carries(self) -> bool {
        self.argument & BOTH != 0 || self.parameter & BOTH != 0
    }

    const THROUGH: Self = Self {
        closes: 0,
        argument: ARGUMENT | (TYPE_SUFFIX << 4),
        parameter: PARAMETER | (CONSTRUCTOR << 4),
    };
    const NAME: Self = Self {
        closes: 0,
        argument: ARGUMENT | CONSTRUCTOR,
        parameter: PARAMETER,
    };
    const RECOVER: Self = Self {
        closes: 0,
        argument: BOTH,
        parameter: PARAMETER,
    };
    // A complete type that cannot continue a generic parameter declaration.
    const TYPE: Self = Self {
        closes: 0,
        argument: BOTH,
        parameter: 0,
    };
    const PARAMETER_PREFIX: Self = Self {
        closes: 0,
        argument: 0,
        parameter: PARAMETER,
    };
    const BARRIER: Self = Self {
        closes: 0,
        argument: 0,
        parameter: 0,
    };

    // Each byte stores two columns of the four-position transfer relation.
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
                self.argument >> 4
            } else {
                0
            })
            | (if continuation & CONSTRUCTOR != 0 {
                self.parameter >> 4
            } else {
                0
            })
    }

    fn then(self, next: Self) -> Self {
        // If neither relation reaches the contextual entry positions, their
        // composition only needs the two argument/parameter columns.
        if (self.argument | self.parameter | next.argument | next.parameter) >> 4 == 0 {
            return Self {
                closes: self.closes | self.transfer_entries(next.closes),
                argument: self.transfer_entries(next.argument),
                parameter: self.transfer_entries(next.parameter),
            };
        }

        self.then_contextual(next)
    }

    // Keep the full relation out of line so the two-column composition remains
    // small enough to inline at its consumers.
    #[inline(never)]
    fn then_contextual(self, next: Self) -> Self {
        Self {
            closes: self.closes | self.transfer(next.closes),
            argument: self.transfer(next.argument & GRAMMAR_POSITIONS)
                | (self.transfer(next.argument >> 4) << 4),
            parameter: self.transfer(next.parameter & GRAMMAR_POSITIONS)
                | (self.transfer(next.parameter >> 4) << 4),
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
        (self.closes | self.transfer(continuation)) & BOTH
    }
    fn close_mask(self) -> u8 {
        self.closes & BOTH
    }
    fn constructor(self) -> Self {
        Self {
            closes: if self.closes & CONSTRUCTOR != 0 {
                BOTH
            } else {
                0
            },
            argument: if self.argument & CONSTRUCTOR != 0 {
                BOTH
            } else {
                0
            },
            parameter: 0,
        }
    }
    fn parameter_default(self) -> Self {
        Self {
            closes: if self.close_mask() != 0 { PARAMETER } else { 0 },
            argument: 0,
            parameter: if self.transfer(ARGUMENT) & BOTH != 0 {
                PARAMETER
            } else {
                0
            },
        }
    }
    fn with_constructor(mut self, allowed: bool) -> Self {
        if allowed {
            self.argument |= CONSTRUCTOR;
        }
        self
    }
}

#[derive(Clone, Copy, Default)]
struct Fragment {
    state: State,
    syntax: TypeSyntax,
}

// External constraints travel down; finalized token/grammar summaries travel up.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Prefix {
    #[default]
    None,
    // An opening angle immediately precedes this fragment.
    Exposed,
    // Existing parentheses already shield a binary prefix; arrow token choices
    // still matter inside them during TypeScript's function-type recovery.
    Grouped,
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
    BinaryLeft(Binaryish, Context),
    Finish(Expr, Prefix, ExprData),
}

pub(crate) struct GroupingPlan<'a> {
    ast: &'a Ast<'a>,
    fragments: ArenaVec<'a, Fragment>,
    minify: bool,
    comments: bool,
    line_breaks: ArenaVec<'a, (u64, u32)>,
    pending: ArenaVec<'a, Task>,
}

impl<'a> GroupingPlan<'a> {
    pub(crate) fn new(
        allocator: &'a Allocator,
        ast: &'a Ast<'a>,
        minify: bool,
        comments: bool,
    ) -> Self {
        Self {
            ast,
            minify,
            comments,
            fragments: ArenaVec::new_in(allocator),
            line_breaks: ArenaVec::new_in(allocator),
            pending: ArenaVec::new_in(allocator),
        }
    }

    pub(crate) fn bare_arrow_parameters(&self, arrow: ArrowFunctionExpression) -> bool {
        self.fragments
            .get(arrow.node_id().index())
            .is_some_and(|fragment| fragment.state.contains(State::BARE_ARROW))
    }

    fn needs_group(&self, expression: Expr) -> bool {
        self.fragments
            .get(expression.node_id().index())
            .is_some_and(|fragment| fragment.state.grouped())
    }

    // Planning may inspect flags before the expression has a summary.
    fn state(&self, expression: Expr) -> State {
        self.fragments[expression.node_id().index()].state
    }

    // Reading semantic facts requires completed dependency planning.
    fn summary(&self, expression: Expr) -> Fragment {
        let fragment = self.fragments[expression.node_id().index()];
        debug_assert!(
            fragment.state.known(),
            "summary requires planned dependencies"
        );
        fragment
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

    fn summarize(&mut self, expression: Expr) {
        self.initialize();
        if !self.state(expression).known() {
            self.pending.push(Task::Plan(expression, Context::NONE));
            self.run();
        }
    }

    fn run(&mut self) {
        while let Some(task) = self.pending.pop() {
            match task {
                Task::Plan(expression, context) => self.plan(expression, context),
                Task::BinaryLeft(binary, context) => self.plan_binary_left(binary, context),
                Task::Finish(expression, prefix, data) => {
                    self.finish(expression, data);
                    if prefix != Prefix::None
                        && let ExprData::ArrowFunctionExpression(node) = data
                        && self.fragments[expression.node_id().index()]
                            .syntax
                            .close_mask()
                            != 0
                        && let Some(identifier) = bare_arrow_parameter(self.ast, node, true)
                        && node.params(self.ast).span(self.ast).start
                            == identifier.span(self.ast).start
                    {
                        self.fragments[expression.node_id().index()]
                            .state
                            .insert(State::BARE_ARROW);
                        let first = self.leading_token(data);
                        self.fragments[expression.node_id().index()]
                            .state
                            .set_first(first);
                    }
                    self.fragments[expression.node_id().index()]
                        .state
                        .set_known();
                }
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
                    self.fragments[expression.node_id().index()]
                        .state
                        .protect(context.tail)
                };
        }
        if known && context.tail == 0 && context.prefix == Prefix::None {
            return;
        }
        let data = self.ast.expr_data(expression);
        self.pending
            .push(Task::Finish(expression, context.prefix, data));
        if let Some(binary) = Self::binary(data) {
            let operator = binary.operator(self.ast);
            let precedence = operator.precedence();
            let right_level = if operator.is_right_associative() {
                precedence
            } else {
                precedence.next()
            };
            let prefix = if matches!(
                operator,
                BinaryishOperator::Binary(BinaryOperator::LessThan | BinaryOperator::LeftShift)
            ) {
                Prefix::Exposed
            } else {
                Prefix::None
            };
            self.pending.push(Task::BinaryLeft(binary, context));
            if Self::operator_transfer(operator).carries() || context.tail != 0 {
                self.pending.push(Task::Plan(
                    binary.right(self.ast),
                    Context {
                        level: right_level,
                        tail: context.tail,
                        prefix,
                    },
                ));
            }
            return;
        }
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
            ExprData::NewExpression(node) => self.dependency(node.callee(ast)),
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
            ExprData::TsAsExpression(node) => self.dependency(node.expression(ast)),
            ExprData::TsSatisfiesExpression(node) => self.dependency(node.expression(ast)),
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
            ExprData::BinaryExpression(binary) => binary.operator(self.ast).precedence(),
            ExprData::LogicalExpression(binary) => binary.operator(self.ast).precedence(),
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
    // Keep this aligned with the paths followed by protect_tail.
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
            remaining.iter().rev().map(|id| {
                match ast.argument_data(ast.get_node_in_sub_range(id)) {
                    ArgumentData::Expr(e) => e,
                    ArgumentData::SpreadElement(s) => s.argument(ast),
                }
            }),
        );
        true
    }

    // A comma is also a type-argument separator. Fold every list with the same
    // rule, including sequence expressions and dynamic import options.
    pub(crate) fn list(
        &mut self,
        mut expressions: impl DoubleEndedIterator<Item = Expr>,
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
        let Some(first) = expressions.find(|&expression| self.can_expose_open(expression)) else {
            return;
        };
        self.plan_list_tail(first, level, std::iter::once(last).chain(expressions.rev()));
    }

    #[inline(never)]
    fn plan_list_tail(
        &mut self,
        first: Expr,
        level: Precedence,
        expressions: impl Iterator<Item = Expr>,
    ) {
        self.initialize();
        let mut continuation = 0;
        for expression in expressions {
            self.pending
                .push(Task::Plan(expression, Context::tail(level, continuation)));
            self.run();
            continuation = self.syntax(expression, level).before(continuation);
        }
        if continuation != 0 {
            self.pending
                .push(Task::Plan(first, Context::tail(level, continuation)));
            self.run();
        }
    }

    fn syntax(&self, expression: Expr, level: Precedence) -> TypeSyntax {
        let syntax = self.summary(expression).syntax;
        if self.grouped(expression, level) {
            // Parentheses can begin a function type. Recovery within that type
            // can still reach an angle in the contents, so grouping changes the
            // leading token but does not hide all speculative closing prefixes.
            TypeSyntax {
                closes: TypeSyntax::RECOVER.carry(syntax.close_mask())
                    | (syntax.closes & CONSTRUCTOR),
                ..TypeSyntax::TYPE
            }
            .with_constructor(syntax.argument & CONSTRUCTOR != 0)
        } else {
            syntax
        }
    }

    fn plan_binary_left(&mut self, binary: Binaryish, context: Context) {
        let expression = match binary {
            Binaryish::Binary(node) => Expr::BinaryExpression(node),
            Binaryish::Logical(node) => Expr::LogicalExpression(node),
        };
        let operator = binary.operator(self.ast);
        let precedence = operator.precedence();
        let left_level = if operator.is_right_associative() {
            precedence.next()
        } else {
            precedence
        };
        let right_level = if operator.is_right_associative() {
            precedence
        } else {
            precedence.next()
        };
        let mut tail = 0;
        let mut prefix = Prefix::None;
        if context.tail != 0 {
            let requirement = self
                .syntax(binary.right(self.ast), right_level)
                .carry(context.tail);
            let opening = match operator {
                BinaryishOperator::Binary(BinaryOperator::LessThan) => ARGUMENT,
                BinaryishOperator::Binary(BinaryOperator::LeftShift) => PARAMETER,
                _ => 0,
            };
            if opening & requirement != 0 {
                self.fragments[expression.node_id().index()]
                    .state
                    .mark_grouped();
            } else {
                tail = Self::operator_transfer(operator).carry(requirement);
            }
        }
        if context.prefix == Prefix::Exposed && !self.grouped(expression, context.level) {
            if Self::closing(operator)
                && self.can_follow(binary.left(self.ast), binary.right(self.ast), right_level)
            {
                self.fragments[expression.node_id().index()]
                    .state
                    .mark_grouped();
            } else {
                prefix = Prefix::Exposed;
            }
        }
        if Self::closing(operator)
            || Self::type_operator(operator)
            || operator == BinaryishOperator::Binary(BinaryOperator::LessThan)
        {
            tail |= self
                .operator_syntax(binary.left(self.ast), binary.right(self.ast), operator)
                .before(self.syntax(binary.right(self.ast), right_level).closes);
        }
        self.pending.push(Task::Plan(
            binary.left(self.ast),
            Context {
                level: left_level,
                tail,
                prefix,
            },
        ));
    }

    // Plan this boundary and return its additional TypeScript grouping decision.
    pub(crate) fn plan_binary(&mut self, binary: Binaryish, operator: BinaryishOperator) -> bool {
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
        if matches!(
            operator,
            BinaryishOperator::Binary(BinaryOperator::LessThan | BinaryOperator::LeftShift)
        ) && matches!(
            self.ast.expr_data(binary.right(self.ast)),
            ExprData::IdentifierReference(_)
                | ExprData::NumericLiteral(_)
                | ExprData::StringLiteral(_)
                | ExprData::BooleanLiteral(_)
                | ExprData::NullLiteral(_)
                | ExprData::BigIntLiteral(_)
                | ExprData::RegExpLiteral(_)
        ) {
            return false;
        }
        self.summarize(expression);
        self.needs_group(expression)
    }

    fn operator_syntax(
        &mut self,
        left: Expr,
        right: Expr,
        operator: BinaryishOperator,
    ) -> TypeSyntax {
        let mut syntax = Self::operator_transfer(operator);
        if Self::closing(operator) && self.can_follow(left, right, operator.precedence().next()) {
            syntax.closes = if operator == BinaryishOperator::Binary(BinaryOperator::GreaterThan) {
                ARGUMENT
            } else {
                PARAMETER
            };
        }
        syntax
    }

    fn operator_transfer(operator: BinaryishOperator) -> TypeSyntax {
        if Self::closing(operator) {
            // A close discharges a function's parameters, leaving an outer
            // type-argument requirement for the continuation to finish.
            TypeSyntax {
                closes: 0,
                argument: PARAMETER,
                parameter: PARAMETER,
            }
        } else if Self::type_operator(operator) {
            TypeSyntax::RECOVER
        } else if operator == BinaryishOperator::Binary(BinaryOperator::LessThan) {
            TypeSyntax {
                closes: 0,
                argument: ARGUMENT,
                parameter: ARGUMENT,
            }
        } else if operator == BinaryishOperator::Binary(BinaryOperator::LeftShift) {
            TypeSyntax::THROUGH
        } else {
            TypeSyntax::BARRIER
        }
    }

    fn finish(&mut self, expression: Expr, data: ExprData) {
        if let Some(binary) = Self::binary(data) {
            let operator = binary.operator(self.ast);
            let left = binary.left(self.ast);
            let right = binary.right(self.ast);
            let precedence = operator.precedence();
            let left_level = if operator.is_right_associative() {
                precedence.next()
            } else {
                precedence
            };
            let right_level = if operator.is_right_associative() {
                precedence
            } else {
                precedence.next()
            };
            let first = self.first_with_instantiation_grouping(
                left,
                left_level,
                operator
                    .following_context()
                    .contains(ExprContext::NO_INSTANTIATION),
            );
            let operator_syntax = self.operator_syntax(left, right, operator);
            // Once the operator cuts off speculative parsing, the right
            // operand cannot contribute to this fragment's transfer.
            let mut syntax = self.syntax(left, left_level).then(operator_syntax);
            if operator_syntax.carries() {
                syntax = syntax.then(self.syntax(right, right_level));
            }
            let fragment = &mut self.fragments[expression.node_id().index()];
            fragment.state.set_first(first);
            fragment.syntax = syntax;
            return;
        }
        let first = self.leading_token(data);
        let syntax = self.type_syntax(expression, data);
        let fragment = &mut self.fragments[expression.node_id().index()];
        fragment.state.set_first(first);
        fragment.syntax = syntax;
    }

    // Mirror the printer's token choices, independently of type grammar.
    fn leading_token(&self, data: ExprData) -> LeadingToken {
        let ast = self.ast;
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
            ExprData::TsInstantiationExpression(node) => {
                self.first_with_instantiation_grouping(node.expression(ast), Precedence::Call, true)
            }
            ExprData::UpdateExpression(node) if !node.prefix(ast) => self.first(
                simple_assignment_target_expression(ast, node.argument(ast)),
                Precedence::Postfix,
            ),
            ExprData::CallExpression(node) => {
                let callee = node.callee(ast);
                let level = if matches!(ast.expr_data(callee), ExprData::NewExpression(_)) {
                    Precedence::New
                } else {
                    Precedence::Call
                };
                self.first_with_instantiation_grouping(callee, level, !node.optional(ast))
            }
            ExprData::MemberExpression(node) => {
                let object = node.object(ast);
                let level = if matches!(ast.expr_data(object), ExprData::NewExpression(_)) {
                    Precedence::New
                } else {
                    Precedence::Call
                };
                let grouped_let = node.computed(ast)
                    && matches!(ast.expr_data(object), ExprData::IdentifierReference(id) if ast.get_utf8(id.name(ast)) == "let");
                if grouped_let {
                    LeadingToken::Accept
                } else {
                    self.first_with_instantiation_grouping(
                        object,
                        level,
                        !node.optional(ast) || !node.computed(ast),
                    )
                }
            }
            ExprData::ChainExpression(node) => self.first(node.expression(ast), Precedence::Lowest),
            ExprData::TaggedTemplateExpression(node) => {
                let tag = node.tag(ast);
                let level = if node.type_arguments(ast).is_some()
                    || matches!(ast.expr_data(tag), ExprData::ChainExpression(_))
                {
                    Precedence::Call
                } else {
                    Precedence::Postfix
                };
                self.first_with_instantiation_grouping(tag, level, true)
            }
            ExprData::TsNonNullExpression(node) => {
                let operand = node.expression(ast);
                let level = if matches!(ast.expr_data(operand), ExprData::ChainExpression(_)) {
                    Precedence::Call
                } else {
                    Precedence::Postfix
                };
                self.first_with_instantiation_grouping(operand, level, true)
            }
            ExprData::TsAsExpression(node) => self.first(node.expression(ast), Precedence::Compare),
            ExprData::TsSatisfiesExpression(node) => {
                self.first(node.expression(ast), Precedence::Compare)
            }
            ExprData::ConditionalExpression(node) => {
                self.first(node.test(ast), Precedence::Conditional.next())
            }
            ExprData::SequenceExpression(node) => node
                .expressions(ast)
                .get_node(ast, 0)
                .map_or(LeadingToken::Expression, |first| {
                    self.first(first, self.sequence_level())
                }),
            _ => LeadingToken::Expression,
        }
    }

    fn sequence_level(&self) -> Precedence {
        if self.minify {
            Precedence::Lowest
        } else {
            Precedence::Assignment
        }
    }

    // TypeScript's parseType/parseTypeParameter productions also perform error
    // recovery during speculative parsing. Keep these grammar transfers apart
    // from emitted-token classification and from the grouping worklist.
    fn type_syntax(&self, expression: Expr, data: ExprData) -> TypeSyntax {
        let ast = self.ast;
        match data {
            ExprData::ParenthesizedExpression(node) => {
                TypeSyntax::RECOVER.then(self.syntax(node.expression(ast), Precedence::Lowest))
            }
            ExprData::AssignmentExpression(node) => {
                if node.operator(ast) == AssignmentOperator::Assign {
                    // `=` is a parameter default, never part of a type argument.
                    self.syntax(node.right(ast), Precedence::Assignment)
                        .parameter_default()
                        .with_constructor(true)
                } else {
                    TypeSyntax::BARRIER
                }
            }
            ExprData::CallExpression(node) => {
                // Recovering the missing generic-parameter `>` can make a call's
                // arguments function-type parameters. That path starts at `<<`.
                let mut arguments = TypeSyntax::THROUGH;
                for id in node.arguments(ast).iter() {
                    let argument = match ast.argument_data(ast.get_node_in_sub_range(id)) {
                        ArgumentData::Expr(e) => e,
                        ArgumentData::SpreadElement(s) => s.argument(ast),
                    };
                    arguments = arguments.then(self.syntax(argument, Precedence::Assignment));
                }
                TypeSyntax {
                    closes: self.syntax(node.callee(ast), Precedence::Call).closes
                        | if arguments.closes != 0 { PARAMETER } else { 0 },
                    argument: PARAMETER,
                    parameter: PARAMETER,
                }
            }
            ExprData::MemberExpression(node) => {
                let object = self.syntax(node.object(ast), Precedence::Call);
                let index = if node.computed(ast)
                    && let PropertyKeyData::Expr(key) = ast.property_key_data(node.property(ast))
                {
                    self.syntax(key, Precedence::Lowest).closes
                } else {
                    0
                };
                TypeSyntax {
                    closes: object.closes | TypeSyntax::RECOVER.carry(index),
                    ..TypeSyntax::RECOVER
                }
                .with_constructor(object.argument & CONSTRUCTOR != 0)
            }
            // Tuple and object types parse their contents before demanding the
            // closing bracket/brace. Missing delimiters do not end speculation.
            ExprData::ArrayExpression(node) => {
                let mut syntax = TypeSyntax::TYPE;
                for id in node.elements(ast).iter() {
                    if let Some(argument) = ast.get_node_in_sub_range(id) {
                        let expression = match ast.argument_data(argument) {
                            ArgumentData::Expr(e) => e,
                            ArgumentData::SpreadElement(s) => s.argument(ast),
                        };
                        syntax = syntax.then(self.syntax(expression, Precedence::Assignment));
                    }
                }
                syntax.argument |= TYPE_SUFFIX | CONSTRUCTOR;
                syntax
            }
            ExprData::ObjectExpression(node) => {
                let mut closes = 0;
                for id in node.properties(ast).iter() {
                    match ast.object_property_kind_data(ast.get_node_in_sub_range(id)) {
                        ObjectPropertyKindData::SpreadElement(s) => {
                            closes |= self.syntax(s.argument(ast), Precedence::Assignment).closes
                        }
                        ObjectPropertyKindData::ObjectProperty(p) => {
                            closes |= self.syntax(p.value(ast), Precedence::Assignment).closes;
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
                    closes: TypeSyntax::TYPE.carry(closes),
                    ..TypeSyntax::TYPE
                }
                .with_constructor(true)
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
            ExprData::TsAsExpression(node) => {
                self.syntax(node.expression(ast), Precedence::Compare)
            }
            ExprData::TsSatisfiesExpression(node) => {
                self.syntax(node.expression(ast), Precedence::Compare)
            }
            ExprData::ConditionalExpression(node) => TypeSyntax {
                closes: TypeSyntax::RECOVER.carry(
                    self.syntax(node.test(ast), Precedence::Conditional.next())
                        .closes
                        | self
                            .syntax(node.consequent(ast), Precedence::Assignment)
                            .closes
                        | self
                            .syntax(node.alternate(ast), Precedence::Assignment)
                            .closes,
                ),
                ..TypeSyntax::RECOVER
            }
            .with_constructor(true),
            ExprData::ArrowFunctionExpression(node) => {
                let closes = match ast.arrow_function_body_data(node.body(ast)) {
                    ArrowFunctionBodyData::Expr(body) => {
                        TypeSyntax::RECOVER.carry(self.syntax(body, Precedence::Assignment).closes)
                    }
                    _ => 0,
                };
                // Recovery inside constructor parameters may enter the arrow's
                // return expression before rejecting the remaining signature.
                TypeSyntax {
                    closes: closes | if closes != 0 { CONSTRUCTOR } else { 0 },
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
                    .fold(TypeSyntax::RECOVER, |syntax, id| {
                        syntax.then(self.syntax(ast.get_node_in_sub_range(id), Precedence::Lowest))
                    })
            }
            ExprData::TaggedTemplateExpression(node) => TypeSyntax {
                closes: self.syntax(node.tag(ast), Precedence::Postfix).closes
                    | if self
                        .summary(Expr::TemplateLiteral(node.quasi(ast)))
                        .syntax
                        .closes
                        != 0
                    {
                        PARAMETER
                    } else {
                        0
                    },
                argument: PARAMETER,
                parameter: PARAMETER,
            },
            ExprData::JsxElement(_) | ExprData::JsxFragment(_) => {
                // In type syntax the initial `<` starts another parameter list.
                // A close reached inside an attribute must discharge that list
                // as well as the outer type argument, hence the PARAMETER path.
                let closes = jsx_expressions(ast, expression).fold(0, |closes, child| {
                    closes | self.syntax(child, Precedence::Lowest).closes
                });
                TypeSyntax {
                    closes: if closes & PARAMETER != 0 { BOTH } else { 0 },
                    ..TypeSyntax::RECOVER
                }
            }
            ExprData::YieldExpression(node) => {
                if let Some(argument) = node.argument(ast) {
                    // The keyword can be a parameter name; a following operand
                    // can start a function return type after missing delimiters.
                    let prefix = if node.delegate(ast) {
                        TypeSyntax {
                            closes: 0,
                            argument: PARAMETER << 4,
                            parameter: 0,
                        }
                    } else {
                        TypeSyntax::PARAMETER_PREFIX
                    };
                    prefix.then(self.syntax(argument, Precedence::Yield))
                } else {
                    TypeSyntax::NAME
                }
            }
            ExprData::AwaitExpression(node) => {
                let argument = node.argument(ast);
                let syntax = self.syntax(argument, Precedence::Prefix);
                if self.grouped(argument, Precedence::Prefix)
                    || matches!(
                        ast.expr_data(argument),
                        ExprData::ParenthesizedExpression(_)
                    )
                {
                    let mut constructor = syntax.constructor();
                    constructor.argument &= PARAMETER;
                    constructor.closes &= PARAMETER;
                    constructor
                } else {
                    TypeSyntax::PARAMETER_PREFIX.then(syntax)
                }
            }
            ExprData::NewExpression(node) => {
                let callee = self.syntax(node.callee(ast), Precedence::New);
                let syntax = callee.constructor();
                let parameters = self.grouped(node.callee(ast), Precedence::New)
                    || matches!(
                        ast.expr_data(node.callee(ast)),
                        ExprData::ParenthesizedExpression(_)
                    );
                if !parameters
                    && new_expression_has_argument_list(ast, node, self.minify, Precedence::Lowest)
                {
                    syntax.then(TypeSyntax::BARRIER)
                } else {
                    syntax
                }
            }
            ExprData::UpdateExpression(_) => TypeSyntax::BARRIER,
            ExprData::TsInstantiationExpression(node) => {
                let mut syntax = self.syntax(node.expression(ast), Precedence::Call);
                syntax.closes |= CONSTRUCTOR;
                syntax
            }
            ExprData::NumericLiteral(_)
            | ExprData::StringLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::BigIntLiteral(_) => TypeSyntax::TYPE,
            ExprData::IdentifierReference(_) => TypeSyntax::NAME,
            _ => TypeSyntax::RECOVER,
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
