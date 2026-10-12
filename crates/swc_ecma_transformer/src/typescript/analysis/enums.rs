use rustc_hash::{FxHashMap, FxHashSet};

use swc_next_ecma_ast::{
    Ast, BindingPatternData, Expr, MemberExpression, NodeId, NodeKindData, SymbolId,
    TsEnumDeclaration, UnaryOperator, VariableDeclaration, VariableKind,
};
use swc_next_ecma_semantic::{Semantic, SymbolFlags};

use super::super::TypeScriptOptions;
use super::constants::{
    ConstBinding, ConstBindings, EnumDeclaration, EnumMemberValue, TsEnumConstantValue, evaluate,
    resolve_constant_member,
};

pub(super) struct EnumCollector {
    collect_const_enum_uses: bool,
    remove_const_enums: bool,
    assignment_target: Option<NodeId>,
    enum_declarations: Vec<EnumDeclaration>,
    removal_candidates: Vec<ConstEnumRemovalCandidate>,
    const_bindings: ConstBindings,
    collect_const_bindings: bool,
    inline_candidates: Vec<InlineCandidate>,
}

impl EnumCollector {
    pub(super) fn new(ast: &Ast<'_>, semantic: &Semantic<'_>, options: &TypeScriptOptions) -> Self {
        let inline_const_enum_uses = !options.verbatim_module_syntax;
        let remove_const_enums = inline_const_enum_uses && !options.preserve_const_enums;
        let mut collect_const_bindings = false;
        let mut collect_const_enum_uses = false;
        for entry in semantic.iter_symbols() {
            let flags = entry.symbol.flags;
            if !collect_const_bindings && flags.contains(SymbolFlags::ENUM_MEMBER) {
                collect_const_bindings = semantic.declarations(entry.id).any(|declaration| {
                    let NodeKindData::TsEnumMember(member) = ast.kind_data(declaration.node) else {
                        return false;
                    };
                    member.initializer(ast).is_some_and(|initializer| {
                        may_evaluate_const_binding(ast, semantic, initializer)
                    })
                });
            }
            if inline_const_enum_uses && flags.contains(SymbolFlags::CONST_ENUM) {
                collect_const_enum_uses = true;
            }
        }
        Self {
            collect_const_enum_uses,
            remove_const_enums,
            assignment_target: None,
            enum_declarations: Vec::new(),
            removal_candidates: Vec::new(),
            const_bindings: FxHashMap::default(),
            collect_const_bindings,
            inline_candidates: Vec::new(),
        }
    }

    pub(super) fn set_assignment_target(
        &mut self,
        target: Option<NodeId>,
        ast: &Ast<'_>,
    ) -> Option<NodeId> {
        if !self.collect_const_enum_uses {
            return None;
        }
        let target = target.map(|mut node| {
            loop {
                node = match ast.kind_data(node) {
                    NodeKindData::ParenthesizedExpression(it) => it.expression(ast).node_id(),
                    NodeKindData::TsAsExpression(it) => it.expression(ast).node_id(),
                    NodeKindData::TsSatisfiesExpression(it) => it.expression(ast).node_id(),
                    NodeKindData::TsTypeAssertion(it) => it.expression(ast).node_id(),
                    NodeKindData::TsNonNullExpression(it) => it.expression(ast).node_id(),
                    _ => break,
                };
            }
            node
        });
        std::mem::replace(&mut self.assignment_target, target)
    }

    pub(super) fn record_enum_declaration(
        &mut self,
        declaration: TsEnumDeclaration,
        ambient: bool,
        deferred: Option<NodeId>,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) {
        let binding = declaration.id(ast);
        let Some(symbol) = semantic.symbol_of(binding.node_id()) else {
            return;
        };
        let owner = semantic.symbol(symbol);
        let flags = owner.flags;
        if !flags.intersects(SymbolFlags::REGULAR_ENUM | SymbolFlags::CONST_ENUM) {
            return;
        }
        self.enum_declarations.push(EnumDeclaration {
            syntax: declaration,
            deferred,
            auto_increment: declaration.is_const(ast) || !(ambient || declaration.declare(ast)),
        });
        if self.remove_const_enums
            && flags.contains(SymbolFlags::CONST_ENUM)
            && declaration.is_const(ast)
            && !declaration.declare(ast)
            && !flags.contains(SymbolFlags::EXPORTED)
        {
            self.removal_candidates.push(ConstEnumRemovalCandidate {
                symbol,
                syntax: declaration,
                primary: semantic.declaration(owner.declaration).node == binding.node_id(),
            });
        }
    }

    pub(super) fn record_variable_declaration(
        &mut self,
        declaration: VariableDeclaration,
        deferred: Option<NodeId>,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) {
        if !self.collect_const_bindings || declaration.kind(ast) != VariableKind::Const {
            return;
        }

        for declarator_id in declaration.declarators(ast).iter() {
            let declarator = ast.get_node_in_sub_range(declarator_id);
            let BindingPatternData::BindingIdentifier(identifier) =
                ast.binding_pattern_data(declarator.id(ast))
            else {
                continue;
            };
            if declarator.type_annotation(ast).is_some() {
                continue;
            }
            if declarator.init(ast).is_none() {
                continue;
            }
            let Some(symbol) = semantic.symbol_of(identifier.node_id()) else {
                continue;
            };
            if semantic.symbol(symbol).flags.contains(SymbolFlags::AMBIENT) {
                continue;
            }
            self.const_bindings.entry(symbol).or_insert(ConstBinding {
                syntax: declarator,
                deferred,
            });
        }
    }

    pub(super) fn record_member_expression(
        &mut self,
        member: MemberExpression,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) {
        let Some((reference, member_symbol)) = self.enum_member(member, ast, semantic) else {
            return;
        };
        self.inline_candidates.push(InlineCandidate {
            expression: member.node_id(),
            root_reference: reference,
            member: member_symbol,
        });
    }

    fn enum_member(
        &self,
        member: MemberExpression,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) -> Option<(NodeId, SymbolId)> {
        if !self.collect_const_enum_uses
            || self.assignment_target == Some(member.node_id())
            || member.optional(ast)
        {
            return None;
        }
        let (reference, owner, member_symbol) =
            super::constants::resolve_enum_member(ast, semantic, member)?;
        if !semantic
            .symbol(owner)
            .flags
            .contains(SymbolFlags::CONST_ENUM)
        {
            return None;
        }
        Some((reference, member_symbol))
    }
}

struct InlineCandidate {
    expression: NodeId,
    root_reference: NodeId,
    member: SymbolId,
}

struct ConstEnumRemovalCandidate {
    symbol: SymbolId,
    syntax: TsEnumDeclaration,
    primary: bool,
}

#[derive(Default)]
pub(super) struct EnumAnalysis<'ast> {
    member_values: FxHashMap<SymbolId, EnumMemberValue<'ast>>,
    removable: FxHashSet<SymbolId>,
    inline_uses: FxHashMap<NodeId, TsEnumConstantValue<'ast>>,
}

impl<'ast> EnumAnalysis<'ast> {
    pub(super) fn finish(
        ast: &Ast<'ast>,
        semantic: &Semantic<'ast>,
        collector: &EnumCollector,
    ) -> (Self, FxHashSet<NodeId>) {
        let member_values = evaluate(
            ast,
            &collector.enum_declarations,
            &collector.const_bindings,
            semantic,
        );
        if !collector.collect_const_enum_uses {
            return (
                Self {
                    member_values,
                    ..Self::default()
                },
                FxHashSet::default(),
            );
        }

        let capacity = collector.inline_candidates.len();
        let mut safe_references = FxHashSet::with_capacity_and_hasher(capacity, Default::default());
        let mut inline_uses = FxHashMap::with_capacity_and_hasher(capacity, Default::default());
        for &InlineCandidate {
            expression,
            root_reference: reference,
            member,
        } in &collector.inline_candidates
        {
            if let Some(value) = member_values
                .get(&member)
                .and_then(|value| value.constant())
            {
                inline_uses.insert(expression, value);
                safe_references.insert(reference);
            }
        }

        let mut removable = FxHashSet::default();
        for candidate in &collector.removal_candidates {
            let symbol = candidate.symbol;
            if candidate.primary {
                if Self::can_remove_const_enum(
                    candidate,
                    ast,
                    semantic,
                    &member_values,
                    &safe_references,
                ) {
                    removable.insert(symbol);
                }
                continue;
            }
            // Candidates are in declaration order. Later bodies may reject the
            // whole group, but only its primary declaration can make it eligible.
            if removable.contains(&symbol)
                && !Self::all_members_constant(candidate.syntax, ast, semantic, &member_values)
            {
                removable.remove(&symbol);
            }
        }

        (
            Self {
                member_values,
                removable,
                inline_uses,
            },
            safe_references,
        )
    }

    pub(super) fn is_removable(&self, symbol: SymbolId) -> bool {
        self.removable.contains(&symbol)
    }

    pub(super) fn inline_use(&self, node: NodeId) -> Option<TsEnumConstantValue<'ast>> {
        self.inline_uses.get(&node).copied()
    }

    pub(super) fn member_value(&self, member: SymbolId) -> EnumMemberValue<'ast> {
        self.member_values.get(&member).copied().unwrap_or_default()
    }

    fn can_remove_const_enum(
        candidate: &ConstEnumRemovalCandidate,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
        member_values: &FxHashMap<SymbolId, EnumMemberValue<'_>>,
        safe_references: &FxHashSet<NodeId>,
    ) -> bool {
        let symbol = candidate.symbol;
        // Lowering erases every declaration of a removable symbol, so reject
        // mixed declarations before validating the individual enum bodies.
        if !semantic.declarations(symbol).all(|declaration| {
            declaration.flags.contains(SymbolFlags::CONST_ENUM)
                && !declaration.flags.contains(SymbolFlags::AMBIENT)
        }) {
            return false;
        }
        if !Self::all_members_constant(candidate.syntax, ast, semantic, member_values) {
            return false;
        }
        semantic
            .uses(symbol)
            .iter()
            .filter_map(|&reference_id| {
                let reference = semantic.reference(reference_id);
                (reference.flags.is_read() || reference.flags.is_write()).then_some(reference.node)
            })
            .all(|node| safe_references.contains(&node))
    }

    fn all_members_constant(
        declaration: TsEnumDeclaration,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
        member_values: &FxHashMap<SymbolId, EnumMemberValue<'_>>,
    ) -> bool {
        declaration.body(ast).members(ast).iter().all(|member_id| {
            let member = ast.get_node_in_sub_range(member_id);
            let Some(symbol) = semantic.symbol_of(member.node_id()) else {
                return false;
            };
            // Values are keyed by symbol; conflicting member declarations must
            // not let a later constant hide an earlier unknown initializer.
            semantic.declarations(symbol).len() == 1
                && member_values
                    .get(&symbol)
                    .is_some_and(|value| value.constant().is_some())
        })
    }
}

// Check only expression forms supported by constant evaluation. A direct const
// dependency enables the index for its transitive dependencies too, allowing the
// existing collection walk to retain declarator handles without a second AST walk.
fn may_evaluate_const_binding(ast: &Ast<'_>, semantic: &Semantic<'_>, expression: Expr) -> bool {
    let symbol = match ast.kind_data(expression.node_id()) {
        NodeKindData::IdentifierReference(identifier) => semantic
            .reference_of(identifier.node_id())
            .and_then(|reference| semantic.reference(reference).resolution().symbol()),
        NodeKindData::MemberExpression(member) => resolve_constant_member(ast, semantic, member),
        NodeKindData::ParenthesizedExpression(parenthesized) => {
            return may_evaluate_const_binding(ast, semantic, parenthesized.expression(ast));
        }
        NodeKindData::ChainExpression(chain) => {
            return may_evaluate_const_binding(ast, semantic, chain.expression(ast));
        }
        NodeKindData::UnaryExpression(unary) => {
            return matches!(
                unary.operator(ast),
                UnaryOperator::Positive | UnaryOperator::Negate | UnaryOperator::BitwiseNot
            ) && may_evaluate_const_binding(ast, semantic, unary.argument(ast));
        }
        NodeKindData::BinaryExpression(binary) => {
            return may_evaluate_const_binding(ast, semantic, binary.left(ast))
                || may_evaluate_const_binding(ast, semantic, binary.right(ast));
        }
        NodeKindData::TemplateLiteral(template) => {
            return template.expressions(ast).iter().any(|id| {
                may_evaluate_const_binding(ast, semantic, ast.get_node_in_sub_range(id))
            });
        }
        _ => return false,
    };
    let Some(symbol) = symbol else {
        return false;
    };
    let symbol = semantic.ts_import_alias_target(symbol).unwrap_or(symbol);
    let flags = semantic.symbol(symbol).flags;
    flags.contains(SymbolFlags::CONST_VAR) && !flags.contains(SymbolFlags::AMBIENT)
}
