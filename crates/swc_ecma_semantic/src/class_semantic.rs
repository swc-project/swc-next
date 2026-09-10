use swc_next_allocator::{
    Allocator, hash_map::HashMap as ArenaHashMap, hash_set::HashSet as ArenaHashSet,
    vec::Vec as ArenaVec,
};
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, AstUtf8, Class, MethodDefinition, MethodDefinitionKind, NodeId,
    NodeKindData, PropertyDefinition, PropertyKey,
};

use crate::{SemanticFeatureFlags, UsedNameState};

pub(crate) const CONSTRUCTOR_FEATURES: SemanticFeatureFlags =
    SemanticFeatureFlags::DERIVED_CONSTRUCTORS
        .union(SemanticFeatureFlags::CONSTRUCTOR_THIS)
        .union(SemanticFeatureFlags::CONSTRUCTOR_SUPER);

/// Semantic information collected for class transforms.
///
/// This data is intentionally separate from scopes and symbols: class
/// transforms may add private elements without introducing bindings in the
/// surrounding lexical scope.
pub struct ClassSemantic<'a> {
    allocator: &'a Allocator,
    features: SemanticFeatureFlags,
    classes: ArenaHashMap<'a, NodeId, ClassInfo<'a>>,
    static_block_layouts: ArenaHashMap<'a, NodeId, ClassStaticBlockLayout>,
    derived_constructors: ArenaHashSet<'a, NodeId>,
    arrows: ArenaHashMap<'a, NodeId, ClassArrowInfo>,
    static_block_arrows: ArenaHashMap<'a, NodeId, ArenaVec<'a, NodeId>>,
    static_blocks_with_lexical_super: ArenaHashSet<'a, NodeId>,
    comment_positions: ArenaHashSet<'a, u32>,
    blocks_with_comments: ArenaHashSet<'a, NodeId>,
    arrow_this_functions: ArenaHashMap<'a, NodeId, ArenaHashSet<'a, NodeId>>,
    super_calls: ArenaHashMap<'a, NodeId, NodeId>,
}

impl std::fmt::Debug for ClassSemantic<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ClassSemantic")
            .field("features", &self.features)
            .field("classes", &self.classes)
            .field("derived_constructors", &self.derived_constructors)
            .field("arrows", &self.arrows)
            .field("static_block_arrows", &self.static_block_arrows)
            .field(
                "static_blocks_with_lexical_super",
                &self.static_blocks_with_lexical_super,
            )
            .finish_non_exhaustive()
    }
}

/// Information associated with one class.
#[derive(Debug)]
pub struct ClassInfo<'a> {
    derived: bool,
    used_names: UsedNameState<'a>,
}

/// Class-member order needed to lower static blocks without looking ahead.
#[derive(Debug, Default)]
pub struct ClassStaticBlockLayout {
    pub has_static_blocks: bool,
    pub first_static_property: Option<PropertyDefinition>,
    pub last_static_property: Option<PropertyDefinition>,
    /// Nonempty or commented blocks following the final runtime static field.
    pub trailing_blocks: usize,
    /// Lexical super in trailing blocks that lower to a single expression.
    pub trailing_lexical_super: bool,
}

/// Lexical class environment inherited by an arrow expression.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClassArrowInfo {
    /// The property (or synthesized class environment) whose initializer evaluates this arrow.
    pub property_initializer: Option<NodeId>,
    pub lexical_super: bool,
    pub static_block_environment: bool,
}

impl<'a> ClassSemantic<'a> {
    pub(crate) fn new(ast: &Ast<'a>, features: SemanticFeatureFlags) -> Self {
        let allocator = ast.allocator();
        let mut comment_positions = ArenaHashSet::new_in(allocator);
        if features.intersects(
            SemanticFeatureFlags::STATIC_BLOCK_COMMENTS | SemanticFeatureFlags::STATIC_BLOCK_LAYOUT,
        ) {
            comment_positions.extend(ast.comments().iter().map(|comment| comment.attached_to));
        }
        Self {
            allocator,
            features: features & SemanticFeatureFlags::CLASS,
            classes: ArenaHashMap::new_in(allocator),
            static_block_layouts: ArenaHashMap::new_in(allocator),
            derived_constructors: ArenaHashSet::new_in(allocator),
            arrows: ArenaHashMap::new_in(allocator),
            static_block_arrows: ArenaHashMap::new_in(allocator),
            static_blocks_with_lexical_super: ArenaHashSet::new_in(allocator),
            comment_positions,
            blocks_with_comments: ArenaHashSet::new_in(allocator),
            arrow_this_functions: ArenaHashMap::new_in(allocator),
            super_calls: ArenaHashMap::new_in(allocator),
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn assert_invariants(&self, ast: &Ast<'_>, reachable: &[bool]) {
        for (class, layout) in &self.static_block_layouts {
            assert!(reachable[class.index()]);
            if let Some(property) = layout.first_static_property {
                assert!(reachable[property.node_id().index()]);
            }
            if let Some(property) = layout.last_static_property {
                assert!(reachable[property.node_id().index()]);
            }
        }
        for node in self.classes.keys() {
            assert!(reachable[node.index()]);
            assert!(matches!(ast.kind_data(*node), NodeKindData::Class(_)));
        }
        for node in &self.derived_constructors {
            assert!(reachable[node.index()]);
            assert!(matches!(ast.kind_data(*node), NodeKindData::Function(_)));
        }
        for node in self.arrows.keys() {
            assert!(reachable[node.index()]);
            assert!(matches!(
                ast.kind_data(*node),
                NodeKindData::ArrowFunctionExpression(_)
            ));
        }
        for (node, arrows) in &self.static_block_arrows {
            assert!(reachable[node.index()]);
            assert!(matches!(ast.kind_data(*node), NodeKindData::StaticBlock(_)));
            for arrow in arrows {
                assert!(reachable[arrow.index()]);
            }
        }
    }

    pub(crate) fn replace_static_block(&mut self, node: NodeId) {
        self.static_block_arrows.remove(&node);
        self.static_blocks_with_lexical_super.remove(&node);
        self.blocks_with_comments.remove(&node);
    }

    pub(crate) fn replace_property(&mut self, old: NodeId, new: NodeId) {
        for info in self.arrows.values_mut() {
            if info.property_initializer == Some(old) {
                info.property_initializer = Some(new);
            }
        }
    }

    /// Order information collected during the ordinary semantic traversal.
    pub fn static_block_layout(&self, class: NodeId) -> Option<&ClassStaticBlockLayout> {
        self.static_block_layouts.get(&class)
    }

    /// Consume the original member layout before lowering changes the class.
    pub fn take_static_block_layout(&mut self, class: NodeId) -> Option<ClassStaticBlockLayout> {
        self.static_block_layouts.remove(&class)
    }

    pub(crate) fn invalidate_static_block_layout(&mut self, class: NodeId) {
        self.static_block_layouts.remove(&class);
    }

    pub(crate) fn replace_this(&mut self, node: NodeId, ancestors: impl Iterator<Item = NodeId>) {
        for owner in ancestors {
            if let Some(uses) = self.arrow_this_functions.get_mut(&owner) {
                uses.remove(&node);
                if uses.is_empty() {
                    self.arrow_this_functions.remove(&owner);
                }
                break;
            }
        }
    }

    pub(crate) fn remove_arrow(&mut self, arrow: NodeId, ancestors: impl Iterator<Item = NodeId>) {
        self.arrows.remove(&arrow);
        for owner in ancestors {
            if let Some(arrows) = self.static_block_arrows.get_mut(&owner) {
                arrows.retain(|node| *node != arrow);
                break;
            }
        }
    }

    /// Optional per-class information collected by semantic analysis.
    pub fn features(&self) -> SemanticFeatureFlags {
        self.features
    }

    /// Returns the information collected for `class`.
    ///
    /// This returns `None` when collection was disabled or the node is not a
    /// class from the analyzed tree.
    pub fn get(&self, class: NodeId) -> Option<&ClassInfo<'a>> {
        self.classes.get(&class)
    }

    /// Returns whether `function` is the constructor of a derived class.
    pub fn is_derived_constructor(&self, function: NodeId) -> bool {
        self.derived_constructors.contains(&function)
    }

    /// Returns the initializer environment in which an arrow is evaluated.
    pub fn property_initializer_owner(&self, arrow: NodeId) -> Option<NodeId> {
        self.arrows
            .get(&arrow)
            .and_then(|info| info.property_initializer)
    }

    /// Returns whether an arrow contains a lexical `super` reference.
    pub fn arrow_preserves_lexical_super(&self, arrow: NodeId) -> bool {
        self.arrows
            .get(&arrow)
            .is_some_and(|info| info.lexical_super)
    }

    /// Returns whether `new.target` is undefined in an arrow synthesized for a
    /// static block.
    pub fn is_static_block_environment_arrow(&self, arrow: NodeId) -> bool {
        self.arrows
            .get(&arrow)
            .is_some_and(|info| info.static_block_environment)
    }

    /// Records the evaluation environment of a newly constructed arrow.
    pub fn record_arrow_environment(&mut self, arrow: NodeId, mut environment: ClassArrowInfo) {
        if !self.features.intersects(
            SemanticFeatureFlags::ARROW_ENVIRONMENTS | SemanticFeatureFlags::LEXICAL_SUPER,
        ) {
            return;
        }
        if !self
            .features
            .contains(SemanticFeatureFlags::ARROW_ENVIRONMENTS)
        {
            environment.property_initializer = None;
            environment.static_block_environment = false;
        }
        environment.lexical_super &= self.features.contains(SemanticFeatureFlags::LEXICAL_SUPER);
        let previous = self.arrows.insert(arrow, environment);
        debug_assert!(previous.is_none());
    }

    /// Preserves the evaluation environment of arrows when their scope is inlined.
    pub fn inherit_scope_environment(&mut self, block: NodeId, environment: ClassArrowInfo) {
        if !self
            .features
            .contains(SemanticFeatureFlags::ARROW_ENVIRONMENTS)
        {
            return;
        }
        let Some(arrows) = self.static_block_arrows.get(&block) else {
            return;
        };
        for arrow in arrows {
            let info = self.arrows.entry(*arrow).or_default();
            info.property_initializer = info
                .property_initializer
                .or(environment.property_initializer);
            info.lexical_super |= environment.lexical_super
                && self.features.contains(SemanticFeatureFlags::LEXICAL_SUPER);
            info.static_block_environment |= environment.static_block_environment;
        }
    }

    /// Returns whether a static block contains a `super` reference that a
    /// synthesized arrow must preserve.
    pub fn static_block_preserves_lexical_super(&self, block: NodeId) -> bool {
        self.static_blocks_with_lexical_super.contains(&block)
    }

    /// Allocates and reserves the next private name for `class`.
    ///
    /// The returned name does not include the leading `#`. Reserving and
    /// returning the name in one operation keeps generated class elements and
    /// semantic state synchronized.
    pub fn next_private_name(&mut self, ast: &mut Ast<'_>, class: NodeId) -> AstUtf8 {
        debug_assert!(
            self.features
                .contains(SemanticFeatureFlags::CLASS_PRIVATE_NAMES),
            "class private-name collection must be enabled before allocating names"
        );
        let info = self
            .classes
            .get_mut(&class)
            .expect("class semantic entry must exist for an analyzed class");
        let name = info.used_names.next_name("_", 2);
        ast.add_utf8(name)
    }

    pub(crate) fn enter_class(&mut self, ast: &Ast, class: Class) {
        if self
            .features
            .contains(SemanticFeatureFlags::STATIC_BLOCK_LAYOUT)
        {
            self.static_block_layouts
                .insert(class.node_id(), ClassStaticBlockLayout::default());
        }
        if !self
            .features
            .intersects(SemanticFeatureFlags::CLASS_PRIVATE_NAMES.union(CONSTRUCTOR_FEATURES))
        {
            return;
        }
        let class_id = class.node_id();
        let previous = self.classes.insert(
            class_id,
            ClassInfo {
                derived: self.features.intersects(CONSTRUCTOR_FEATURES)
                    && class.super_class(ast).is_some(),
                used_names: UsedNameState::new(self.allocator),
            },
        );
        debug_assert!(previous.is_none());
    }

    pub(crate) fn record_method_definition(
        &mut self,
        class: NodeId,
        ast: &Ast,
        method: MethodDefinition,
    ) {
        if !self.features.intersects(CONSTRUCTOR_FEATURES)
            || method.kind(ast) != MethodDefinitionKind::Constructor
        {
            return;
        }
        let info = self
            .classes
            .get(&class)
            .expect("current class semantic entry must exist");
        if info.derived {
            self.derived_constructors
                .insert(method.value(ast).node_id());
        }
    }

    pub(crate) fn record_arrow<I>(
        &mut self,
        ast: &Ast,
        arrow: ArrowFunctionExpression,
        ancestors: I,
    ) where
        I: IntoIterator<Item = NodeId>,
    {
        if !self
            .features
            .contains(SemanticFeatureFlags::ARROW_ENVIRONMENTS)
        {
            return;
        }

        let arrow_id = arrow.node_id();
        let mut child = arrow_id;
        for ancestor in ancestors {
            match ast.kind_data(ancestor) {
                NodeKindData::ArrowFunctionExpression(_) | NodeKindData::Function(_) => return,
                NodeKindData::PropertyDefinition(property) => {
                    if property
                        .value(ast)
                        .is_some_and(|value| value.node_id() == child)
                    {
                        self.arrows
                            .entry(arrow_id)
                            .or_default()
                            .property_initializer = Some(property.node_id());
                        return;
                    }
                    child = ancestor;
                }
                NodeKindData::StaticBlock(block) => {
                    self.static_block_arrows
                        .entry(block.node_id())
                        .or_insert_with(|| ArenaVec::new_in(self.allocator))
                        .push(arrow_id);
                    return;
                }
                _ => child = ancestor,
            }
        }
    }

    pub(crate) fn record_super<I>(&mut self, ast: &Ast, ancestors: I)
    where
        I: IntoIterator<Item = NodeId>,
    {
        if !self.features.contains(SemanticFeatureFlags::LEXICAL_SUPER) {
            return;
        }

        let mut child = None;
        for ancestor in ancestors {
            match ast.kind_data(ancestor) {
                NodeKindData::PropertyDefinition(property)
                    if property.value(ast).map(|value| value.node_id()) == child =>
                {
                    return;
                }
                NodeKindData::ArrowFunctionExpression(arrow) => {
                    let info = self.arrows.entry(arrow.node_id()).or_default();
                    info.lexical_super = true;
                    if info.property_initializer.is_some() {
                        return;
                    }
                }
                NodeKindData::Function(_) => return,
                NodeKindData::StaticBlock(block) => {
                    self.static_blocks_with_lexical_super
                        .insert(block.node_id());
                    return;
                }
                _ => {}
            }
            child = Some(ancestor);
        }
    }

    pub fn static_block_has_comments(&self, block: NodeId) -> bool {
        self.blocks_with_comments.contains(&block)
    }

    pub(crate) fn record_static_block(
        &mut self,
        ast: &Ast<'_>,
        block: swc_next_ecma_ast::StaticBlock,
    ) {
        use swc_next_ecma_ast::GetSpan;
        let span = block.span(ast);
        if self.comment_positions.contains(&span.start)
            || self.comment_positions.contains(&span.end)
        {
            self.blocks_with_comments.insert(block.node_id());
        }
    }

    pub(crate) fn record_static_block_layout(
        &mut self,
        class: NodeId,
        ast: &Ast<'_>,
        element: NodeKindData,
    ) {
        use swc_next_ecma_ast::GetSpan;
        let layout = self
            .static_block_layouts
            .get_mut(&class)
            .expect("class layout exists");
        match element {
            NodeKindData::PropertyDefinition(property)
                if property.r#static(ast)
                    && !property.declare(ast)
                    && !property.r#abstract(ast) =>
            {
                layout.first_static_property.get_or_insert(property);
                layout.last_static_property = Some(property);
                layout.trailing_blocks = 0;
                layout.trailing_lexical_super = false;
            }
            NodeKindData::StaticBlock(block) => {
                layout.has_static_blocks = true;
                let span = block.span(ast);
                let statements = block.body(ast);
                if statements.is_empty()
                    && !self.comment_positions.contains(&span.start)
                    && !self.comment_positions.contains(&span.end)
                {
                    return;
                }
                layout.trailing_blocks += 1;
                layout.trailing_lexical_super |= statements.len() == 1
                    && statements
                        .get_node(ast, 0)
                        .unwrap()
                        .as_expression_statement(ast)
                        .is_some()
                    && self
                        .static_blocks_with_lexical_super
                        .contains(&block.node_id());
            }
            _ => {}
        }
    }

    pub fn constructor_needs_this_capture(&self, function: NodeId) -> bool {
        self.is_derived_constructor(function) && self.arrow_this_functions.contains_key(&function)
    }

    pub fn super_call_constructor(&self, call: NodeId) -> Option<NodeId> {
        self.super_calls
            .get(&call)
            .copied()
            .filter(|owner| self.is_derived_constructor(*owner))
    }

    /// Class keys/heritage inherit their surrounding environment; field values
    /// and static blocks introduce an independent this/super environment.
    pub(crate) fn record_lexical_this(
        &mut self,
        ast: &Ast<'_>,
        node: NodeId,
        ancestors: impl Iterator<Item = NodeId>,
        super_call: bool,
    ) {
        let mut child = node;
        let mut in_arrow = false;
        for ancestor in ancestors {
            match ast.kind_data(ancestor) {
                NodeKindData::ArrowFunctionExpression(_) => in_arrow = true,
                NodeKindData::PropertyDefinition(property)
                    if property
                        .value(ast)
                        .is_some_and(|value| value.node_id() == child) =>
                {
                    return;
                }
                NodeKindData::StaticBlock(_) => return,
                NodeKindData::Function(function) => {
                    if !self.is_derived_constructor(function.node_id()) {
                        return;
                    }
                    if super_call {
                        self.super_calls.insert(node, function.node_id());
                    } else if in_arrow {
                        self.arrow_this_functions
                            .entry(function.node_id())
                            .or_insert_with(|| ArenaHashSet::new_in(self.allocator))
                            .insert(node);
                    }
                    return;
                }
                _ => {}
            }
            child = ancestor;
        }
    }

    pub(crate) fn record_private_key(&mut self, class: NodeId, ast: &Ast, key: PropertyKey) {
        if !self
            .features
            .contains(SemanticFeatureFlags::CLASS_PRIVATE_NAMES)
        {
            return;
        }
        let Some(private) = key
            .as_expr(ast)
            .and_then(|key| key.as_private_identifier(ast))
        else {
            return;
        };
        let name = ast.get_utf8(private.name(ast)).as_bytes();
        let info = self
            .classes
            .get_mut(&class)
            .expect("current class semantic entry must exist");
        info.used_names.observe(name);
    }
}

impl ClassInfo<'_> {
    /// Returns whether the class has a heritage expression.
    pub fn is_derived(&self) -> bool {
        self.derived
    }

    /// Returns whether a private name, without its leading `#`, is used.
    pub fn has_name(&self, name: &[u8]) -> bool {
        self.used_names.contains(name)
    }

    /// Iterates the used private names, without their leading `#`.
    pub fn used_names(&self) -> impl Iterator<Item = &[u8]> {
        self.used_names.names()
    }
}
