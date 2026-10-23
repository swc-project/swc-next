//! Collects own-property facts for TypeScript alias target resolution.

use std::{borrow::Cow, hash::BuildHasher};

use hashbrown::{HashMap, HashSet};
use oxc_index::Idx;
use rustc_hash::FxBuildHasher;
use swc_next_ecma_ast::{
    Ast, AstWtf8, ClassType, MemberExpression, NodeId, NodeKindData, ScopeId, SymbolId,
    UnaryOperator,
};

use super::lookup::lookup_scope_name;
use super::{Range, SymbolFlags, SymbolTracker, TsScopeViewIndex};
use crate::{
    ecmascript::unwrap_parentheses,
    scope::{Scope, ScopeKind, ScopeTree},
};

/// The two containers TS7 checks when binding an expando assignment.
/// A container without locals (for example an object literal) is a barrier.
#[derive(Clone, Copy)]
pub(super) struct TsExpandoBindingSite {
    container: Option<ScopeId>,
    block: Option<ScopeId>,
}

/// Contributions of property declarations whose names do not require types.
/// Unknown names can add value meaning, never type or namespace meaning.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum PropertyValue {
    #[default]
    // No contribution from the collected own-property declarations.
    Absent,
    Present,
    Unknown,
}

#[derive(Default)]
pub(super) struct TsPropertyValues<'tree> {
    owners: HashMap<SymbolId, PropertyNames<'tree>, FxBuildHasher>,
}

#[derive(Default)]
struct PropertyNames<'tree> {
    known: HashSet<Cow<'tree, [u8]>, FxBuildHasher>,
    unknown: bool,
}

pub(super) struct TsMemberFactsLookup<'lookup, 'arena> {
    pub(super) tracker: &'lookup SymbolTracker<'arena>,
    pub(super) scopes: &'lookup [Scope],
    pub(super) scope_views: Option<&'lookup TsScopeViewIndex<'arena>>,
}

impl<'tree> TsPropertyValues<'tree> {
    pub(super) fn record(&mut self, owner: SymbolId, name: Option<Cow<'tree, [u8]>>) {
        let members = self.owners.entry(owner).or_default();
        if let Some(name) = name {
            members.known.insert(name);
        } else {
            members.unknown = true;
        }
    }

    pub(super) fn get(&self, owner: SymbolId, name: &[u8]) -> PropertyValue {
        let Some(members) = self.owners.get(&owner) else {
            return PropertyValue::Absent;
        };
        if members.known.contains(name) {
            PropertyValue::Present
        } else if members.unknown {
            PropertyValue::Unknown
        } else {
            PropertyValue::Absent
        }
    }
}

// Collect own exported property contributions, not inherited object properties.
// Only syntax accepted by TS7's early or late member binding contributes.
// Eligible names requiring types remain unknown, never proof of absence.
pub(super) fn collect_ts_property_values<'tree>(
    tree: &'tree Ast<'_>,
    scopes: &[Scope],
    assignments: &[(MemberExpression, TsExpandoBindingSite)],
    lookup: &TsMemberFactsLookup<'_, '_>,
) -> TsPropertyValues<'tree> {
    let mut values = TsPropertyValues::default();
    for scope in scopes {
        if scope.kind != ScopeKind::Class {
            continue;
        }
        let NodeKindData::Class(class) = tree.kind_data(scope.node) else {
            continue;
        };
        let owner = if let Some(id) = class.id(tree) {
            lookup.tracker.symbol_of_node(id.node_id())
        } else if class.class_type(tree) == ClassType::ClassDeclaration {
            lookup.tracker.default_export_symbol()
        } else {
            None
        };
        let Some(owner) = owner else {
            continue;
        };
        values.record(owner, Some(Cow::Borrowed(b"prototype")));
        for element in class.body(tree).body(tree).iter() {
            let element = tree.get_node_in_sub_range(element);
            let (key, computed) = match tree.kind_data(element.node_id()) {
                NodeKindData::PropertyDefinition(property) if property.r#static(tree) => {
                    (property.key(tree), property.computed(tree))
                }
                NodeKindData::MethodDefinition(method) if method.r#static(tree) => {
                    (method.key(tree), method.computed(tree))
                }
                NodeKindData::TsMethodDefinition(method) if method.r#static(tree) => {
                    (method.key(tree), method.computed(tree))
                }
                _ => continue,
            };
            // Parentheses are transparent to this model. TS7's class-name
            // binding is syntax-sensitive here; we intentionally use the
            // unparenthesized form for both class and assignment properties.
            let key = unwrap_parentheses(tree, key.node_id());
            let name = property_name(tree, key, computed);
            if let Some(name) = name {
                values.record(owner, Some(name));
            } else if computed && is_late_bindable_name(tree, key) {
                values.record(owner, None);
            }
        }
    }
    for &(member, site) in assignments {
        let object = member.object(tree).node_id();
        let receiver = lookup
            .expando_receiver(tree, object, site.block)
            .or_else(|| lookup.expando_receiver(tree, object, site.container));
        let Some(owner) = receiver.and_then(|symbol| lookup.function_symbol(symbol)) else {
            continue;
        };
        let property = member.property(tree).node_id();
        // As with class properties, normalize only transparent parentheses.
        let literal = unwrap_parentheses(tree, property);
        let name = property_name(tree, literal, member.computed(tree));
        if let Some(name) = name {
            // Like TS7 deferred expando binding, direct assignments do not
            // replace a non-assignment declaration with the same public name.
            if !lookup.has_declared_member(owner, &name) {
                values.record(owner, Some(name));
            }
        } else if member.computed(tree) && is_late_bindable_name(tree, property) {
            values.record(owner, None);
        }
    }
    values
}

impl TsMemberFactsLookup<'_, '_> {
    fn expando_receiver(
        &self,
        tree: &Ast,
        node: NodeId,
        scope: Option<ScopeId>,
    ) -> Option<SymbolId> {
        let node = unwrap_parentheses(tree, node);
        match tree.kind_data(node) {
            NodeKindData::IdentifierReference(identifier) => {
                self.container_binding(tree.get_utf8(identifier.name(tree)).as_bytes(), scope?)
            }
            NodeKindData::MemberExpression(member)
                if !member.computed(tree) && !member.optional(tree) =>
            {
                let receiver = self.expando_receiver(tree, member.object(tree).node_id(), scope)?;
                let owner = self.function_symbol(receiver)?;
                let name = text_member_name(tree, member.property(tree).node_id(), false)?;
                self.tracker
                    .ts_public_member(owner, tree.get_wtf8(name).as_bytes())?
                    .direct
            }
            _ => None,
        }
    }

    fn container_binding(&self, name: &[u8], scope: ScopeId) -> Option<SymbolId> {
        let hash = self.tracker.scope_maps[ScopeId::ROOT.index()]
            .hasher()
            .hash_one(name);
        // TS has one locals table for a function, including its body when
        // ECMAScript needs separate parameter and body environments.
        let body = self.scopes[scope.index()].function_body;
        let local = body
            .and_then(|body| lookup_scope_name(&self.tracker.scope_maps, body, name, hash))
            .or_else(|| lookup_scope_name(&self.tracker.scope_maps, scope, name, hash));
        if let Some(symbol) = local {
            let symbol = self
                .tracker
                .ts_exported_value(symbol, name)
                .unwrap_or(symbol);
            return Some(symbol);
        }
        let range = self
            .scope_views
            .map_or_else(Range::default, |views| views.ranges[scope.index()]);
        let start = range.start as usize;
        for view in &self.tracker.ts_member_views[start..start + range.len as usize] {
            if let Some(candidate) =
                view.candidates(&self.tracker.ts_member_environments, hash, name)
                && let Some(direct) = candidate.direct
            {
                return Some(direct.symbol);
            }
        }
        None
    }

    fn function_symbol(&self, symbol: SymbolId) -> Option<SymbolId> {
        // Const function initializers own a different property symbol. Their
        // expandos do not augment the variable's namespace export surface.
        self.tracker
            .symbol(symbol)
            .flags
            .contains(SymbolFlags::FUNCTION)
            .then_some(symbol)
    }

    fn has_declared_member(&self, owner: SymbolId, name: &[u8]) -> bool {
        self.tracker.ts_public_member(owner, name).is_some()
    }
}

pub(super) fn is_expando_assignment(tree: &Ast, mut member: MemberExpression) -> bool {
    loop {
        let object = unwrap_parentheses(tree, member.object(tree).node_id());
        if member.optional(tree) {
            return false;
        }
        match tree.kind_data(object) {
            NodeKindData::IdentifierReference(_) => return true,
            NodeKindData::MemberExpression(receiver) if !receiver.computed(tree) => {
                member = receiver
            }
            _ => return false,
        }
    }
}

impl TsExpandoBindingSite {
    pub(super) fn new(
        tree: &Ast,
        scopes: &ScopeTree<'_>,
        current_scope: ScopeId,
        ancestors: impl Iterator<Item = NodeId>,
    ) -> Self {
        let mut block_node = None;
        for node in ancestors {
            let kind = tree.kind_data(node);
            let container = matches!(
                kind,
                NodeKindData::Program(_)
                    | NodeKindData::Function(_)
                    | NodeKindData::ArrowFunctionExpression(_)
                    | NodeKindData::Class(_)
                    | NodeKindData::StaticBlock(_)
                    | NodeKindData::ObjectExpression(_)
                    | NodeKindData::TsModuleBlock(_)
                    | NodeKindData::TsEnumDeclaration(_)
                    | NodeKindData::TsInterfaceDeclaration(_)
                    | NodeKindData::TsTypeLiteral(_)
                    | NodeKindData::TsTypeAliasDeclaration(_)
                    | NodeKindData::TsMappedType(_)
                    | NodeKindData::TsIndexSignature(_)
                    | NodeKindData::TsFunction(_)
                    | NodeKindData::TsFunctionType(_)
                    | NodeKindData::TsConstructorType(_)
                    | NodeKindData::TsMethodSignature(_)
                    | NodeKindData::TsCallSignatureDeclaration(_)
                    | NodeKindData::TsConstructSignatureDeclaration(_)
                    | NodeKindData::JsxOpeningElement(_)
            );
            if container {
                let find_scope = |owner| {
                    let mut current = Some(current_scope);
                    while let Some(id) = current {
                        let scope = &scopes.list[id.index()];
                        if scope.node == owner && !scope.kind.is_ts_lookup_view() {
                            return Some(id);
                        }
                        current = scope.parent;
                    }
                    None
                };
                let container = find_scope(node);
                let block = block_node.and_then(find_scope).or(container);
                return Self { container, block };
            }
            if block_node.is_none()
                && matches!(
                    kind,
                    NodeKindData::BlockStatement(_)
                        | NodeKindData::CatchClause(_)
                        | NodeKindData::ForStatement(_)
                        | NodeKindData::ForInStatement(_)
                        | NodeKindData::ForOfStatement(_)
                        | NodeKindData::SwitchStatement(_)
                )
            {
                block_node = Some(node);
            }
        }
        Self {
            container: None,
            block: None,
        }
    }
}

// Property facts own normalized names when the AST only stores a numeric value.
// A known numeric key must neither disappear nor make unrelated names uncertain.
fn property_name<'tree>(
    tree: &'tree Ast<'_>,
    node: NodeId,
    computed: bool,
) -> Option<Cow<'tree, [u8]>> {
    if let Some(name) = text_member_name(tree, node, computed) {
        return Some(Cow::Borrowed(tree.get_wtf8(name).as_bytes()));
    }
    let (number, prefix) = match tree.kind_data(node) {
        NodeKindData::NumericLiteral(number) => (number, ""),
        NodeKindData::UnaryExpression(unary) if computed => {
            let prefix = match unary.operator(tree) {
                UnaryOperator::Positive => "+",
                UnaryOperator::Negate => "-",
                _ => return None,
            };
            let argument = unwrap_parentheses(tree, unary.argument(tree).node_id());
            let NodeKindData::NumericLiteral(number) = tree.kind_data(argument) else {
                return None;
            };
            (number, prefix)
        }
        _ => return None,
    };
    let mut buffer = zmij_ecma::Buffer::new();
    let number = buffer.format(number.value(tree));
    // TS7 binds signed literal syntax using the token sign plus the normalized
    // numeric token, rather than evaluating the whole computed expression.
    let mut name = Vec::with_capacity(prefix.len() + number.len());
    name.extend_from_slice(prefix.as_bytes());
    name.extend_from_slice(number.as_bytes());
    Some(Cow::Owned(name))
}

pub(super) fn text_member_name(tree: &Ast<'_>, node: NodeId, computed: bool) -> Option<AstWtf8> {
    match tree.kind_data(node) {
        NodeKindData::IdentifierName(identifier) if !computed => Some(identifier.name(tree).into()),
        NodeKindData::StringLiteral(literal) => Some(literal.value(tree)),
        NodeKindData::TemplateLiteral(template)
            if template.expressions(tree).is_empty() && template.quasis(tree).len() == 1 =>
        {
            let quasi_id = template.quasis(tree).iter().next()?;
            let quasi = tree.get_node_in_sub_range(quasi_id);
            (!quasi.is_cooked_undefined(tree)).then(|| quasi.cooked(tree))
        }
        // Expressions requiring type analysis cannot supply a known name here.
        // Private names do not contribute to the public property surface.
        _ => None,
    }
}

fn is_late_bindable_name(tree: &Ast<'_>, mut node: NodeId) -> bool {
    // Mirrors TS7 isLateBindableAST -> IsEntityNameExpression: identifiers
    // and dotted identifier paths qualify; their types remain outside this pass.
    loop {
        node = unwrap_parentheses(tree, node);
        match tree.kind_data(node) {
            NodeKindData::IdentifierReference(_) => return true,
            NodeKindData::MemberExpression(member) if !member.computed(tree) => {
                node = member.object(tree).node_id();
            }
            NodeKindData::ChainExpression(chain) => {
                node = chain.expression(tree).node_id();
            }
            _ => return false,
        }
    }
}
