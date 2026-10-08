use swc_next_allocator::{
    Allocator, hash_map::HashMap as ArenaHashMap, hash_set::HashSet as ArenaHashSet,
};
use swc_next_ecma_ast::{
    Ast, AstUtf8, Class, NodeId, NodeKindData, PropertyDefinition, Visit, VisitWith,
};

use crate::used_names::UsedNameState;

use crate::TransformFeatureFlags;

/// Optional information owned and collected by the transformer.
///
/// This data is intentionally separate from scopes and symbols: class
/// transforms may add private elements without introducing bindings in the
/// surrounding lexical scope.
pub(crate) struct TransformAnalysis<'a> {
    allocator: &'a Allocator,
    features: TransformFeatureFlags,
    private_names: ArenaHashMap<'a, NodeId, UsedNameState<'a>>,
    static_block_layouts: ArenaHashMap<'a, NodeId, StaticBlockLayout>,
    comment_positions: ArenaHashSet<'a, u32>,
    blocks_with_comments: ArenaHashSet<'a, NodeId>,
}

/// Class-member order needed to lower static blocks without looking ahead.
#[derive(Debug, Default)]
pub(crate) struct StaticBlockLayout {
    pub has_static_blocks: bool,
    pub first_static_property: Option<PropertyDefinition>,
    pub last_static_property: Option<PropertyDefinition>,
    /// Nonempty or commented blocks following the final runtime static field.
    pub trailing_blocks: usize,
}

impl<'a> TransformAnalysis<'a> {
    fn new(ast: &Ast<'a>, features: TransformFeatureFlags) -> Self {
        let allocator = ast.allocator();
        let mut comment_positions = ArenaHashSet::new_in(allocator);
        if features.contains(TransformFeatureFlags::ES2022_STATIC_BLOCKS) {
            comment_positions.extend(ast.comments().iter().map(|comment| comment.attached_to));
        }
        Self {
            allocator,
            features,
            private_names: ArenaHashMap::new_in(allocator),
            static_block_layouts: ArenaHashMap::new_in(allocator),
            comment_positions,
            blocks_with_comments: ArenaHashSet::new_in(allocator),
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn assert_invariants(&self, ast: &Ast<'_>) {
        let mut visitor = Reachable {
            ast,
            nodes: vec![false; ast.node_count()],
        };
        ast.root_program().visit_with(&mut visitor);
        let reachable = visitor.nodes;
        for (class, layout) in &self.static_block_layouts {
            assert!(reachable[class.index()]);
            if let Some(property) = layout.first_static_property {
                assert!(reachable[property.node_id().index()]);
            }
            if let Some(property) = layout.last_static_property {
                assert!(reachable[property.node_id().index()]);
            }
        }
        for node in self.private_names.keys() {
            assert!(reachable[node.index()]);
            assert!(matches!(ast.kind_data(*node), NodeKindData::Class(_)));
        }
    }

    pub(crate) fn replace_static_block(&mut self, node: NodeId) {
        self.blocks_with_comments.remove(&node);
    }

    /// Original class-member order collected before lowering.
    pub(crate) fn static_block_layout(&self, class: NodeId) -> Option<&StaticBlockLayout> {
        self.static_block_layouts.get(&class)
    }

    /// Consume the original member layout before lowering changes the class.
    pub(crate) fn take_static_block_layout(&mut self, class: NodeId) -> Option<StaticBlockLayout> {
        self.static_block_layouts.remove(&class)
    }

    pub(crate) fn invalidate_static_block_layout(&mut self, class: NodeId) {
        self.static_block_layouts.remove(&class);
    }

    /// Allocates and reserves the next private name for `class`.
    ///
    /// The returned name does not include the leading `#`. Reserving and
    /// returning the name in one operation keeps generated class elements and
    /// transform state synchronized.
    pub(crate) fn next_private_name(&mut self, ast: &mut Ast<'_>, class: NodeId) -> AstUtf8 {
        debug_assert!(
            self.features
                .contains(TransformFeatureFlags::ES2022_STATIC_BLOCKS),
            "class private-name collection must be enabled before allocating names"
        );
        let names = self
            .private_names
            .get_mut(&class)
            .expect("class private names were collected");
        let name = names.next_name("_", 2);
        ast.add_utf8(name)
    }

    fn collect_class(&mut self, ast: &Ast, class: Class) {
        if self.collect_layout() {
            self.static_block_layouts
                .insert(class.node_id(), StaticBlockLayout::default());
        }
        let private_names = self
            .features
            .contains(TransformFeatureFlags::ES2022_STATIC_BLOCKS);
        if !private_names {
            return;
        }
        let mut names = UsedNameState::new(self.allocator);
        for element in class.body(ast).body(ast).iter() {
            let element = ast.get_node_in_sub_range(element);
            let key = match ast.kind_data(element.node_id()) {
                NodeKindData::PropertyDefinition(property) => property.key(ast),
                NodeKindData::MethodDefinition(method) => method.key(ast),
                NodeKindData::TsMethodDefinition(method) => method.key(ast),
                _ => continue,
            };
            if let Some(private) = key
                .as_expr(ast)
                .and_then(|key| key.as_private_identifier(ast))
            {
                names.observe(ast.get_utf8(private.name(ast)).as_bytes());
            }
        }
        self.private_names.insert(class.node_id(), names);
    }

    fn collect_layout(&self) -> bool {
        self.features.contains(
            TransformFeatureFlags::ES2022_STATIC_BLOCKS
                | TransformFeatureFlags::PROPOSAL_PREVENT_EXTENSIONS,
        )
    }

    pub(crate) fn static_block_has_comments(&self, block: NodeId) -> bool {
        self.blocks_with_comments.contains(&block)
    }

    fn record_static_block(&mut self, ast: &Ast<'_>, block: swc_next_ecma_ast::StaticBlock) {
        use swc_next_ecma_ast::GetSpan;
        let span = block.span(ast);
        if self.comment_positions.contains(&span.start)
            || self.comment_positions.contains(&span.end)
        {
            self.blocks_with_comments.insert(block.node_id());
        }
    }

    fn record_static_block_layout(&mut self, class: NodeId, ast: &Ast<'_>, element: NodeKindData) {
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
            }
            _ => {}
        }
    }
}

impl<'a> TransformAnalysis<'a> {
    pub(crate) fn collect(ast: &Ast<'a>, features: TransformFeatureFlags) -> Self {
        let mut analysis = Self::new(ast, features);
        if features.contains(TransformFeatureFlags::ES2022_STATIC_BLOCKS) {
            let mut collector = Collector {
                ast,
                analysis: &mut analysis,
                path: Vec::with_capacity(64),
            };
            ast.root_program().visit_with(&mut collector);
        }
        analysis
    }
}

struct Collector<'v, 'a> {
    ast: &'v Ast<'a>,
    analysis: &'v mut TransformAnalysis<'a>,
    path: Vec<NodeId>,
}

impl<'a> Visit<'a> for Collector<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    fn enter_node(&mut self, node: NodeKindData) {
        match node {
            NodeKindData::Class(class) => self.analysis.collect_class(self.ast, class),
            NodeKindData::StaticBlock(block)
                if self
                    .analysis
                    .features
                    .contains(TransformFeatureFlags::ES2022_STATIC_BLOCKS) =>
            {
                self.analysis.record_static_block(self.ast, block)
            }
            _ => {}
        }
        self.path.push(node.node_id());
    }

    fn leave_node(&mut self, node: NodeKindData) {
        if self.analysis.collect_layout()
            && matches!(
                node,
                NodeKindData::PropertyDefinition(_) | NodeKindData::StaticBlock(_)
            )
        {
            let class = self
                .path
                .iter()
                .rev()
                .find_map(|&node| match self.ast.kind_data(node) {
                    NodeKindData::Class(class) => Some(class),
                    _ => None,
                });
            self.analysis.record_static_block_layout(
                class.expect("class element belongs to a class").node_id(),
                self.ast,
                node,
            );
        }
        debug_assert_eq!(self.path.pop(), Some(node.node_id()));
    }
}

#[cfg(test)]
#[path = "../tests/analysis.rs"]
mod tests;

#[cfg(debug_assertions)]
struct Reachable<'v, 'a> {
    ast: &'v Ast<'a>,
    nodes: Vec<bool>,
}

#[cfg(debug_assertions)]
impl<'a> Visit<'a> for Reachable<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }
    fn enter_node(&mut self, node: NodeKindData) {
        self.nodes[node.node_id().index()] = true;
    }
}
