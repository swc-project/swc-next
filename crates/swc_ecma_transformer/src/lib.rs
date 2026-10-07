use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, BlockStatement, CallExpression, Class, ClassBody, Function,
    FunctionBody, IdentifierReference, MetaProperty, NodeId, NodeKindData, ObjectProperty, Program,
    RegExpLiteral, StaticBlock, SwitchCase, ThisExpression, TraverseMut, TraverseMutCtx,
    TraverseMutWith, TsModuleBlock,
};
use swc_next_ecma_semantic::{Semantic, SemanticFeatureFlags};

use crate::{assumptions::Assumptions, state::TransformerState};

pub mod assumptions;
mod context;
pub use context::{GeneratedBinding, Mutate, Mutation, StatementList};
pub mod es2015;
pub mod es2022;
mod helpers;
pub mod normalize;
pub mod proposal;
mod state;

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
    pub(crate) struct TransformFeatureFlags: u8 {
        const ARROW_FUNCTIONS = 1 << 0;
        const SHORTHAND_PROPERTIES = 1 << 1;
        const STICKY_REGEX = 1 << 2;
        const ES2022_STATIC_BLOCKS = 1 << 3;
        const PROPOSAL_PREVENT_EXTENSIONS = 1 << 4;
    }
}

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub es2015: es2015::ES2015Options,
    pub es2022: es2022::ES2022Options,
    pub proposal: proposal::ProposalOptions,
}

impl TransformerOptions {
    fn transform_features(&self) -> TransformFeatureFlags {
        let mut features = TransformFeatureFlags::empty();
        features.set(
            TransformFeatureFlags::ARROW_FUNCTIONS,
            self.es2015.arrow_functions.is_some(),
        );
        features.set(
            TransformFeatureFlags::SHORTHAND_PROPERTIES,
            self.es2015.shorthand_properties,
        );
        features.set(
            TransformFeatureFlags::STICKY_REGEX,
            self.es2015.sticky_regex,
        );
        features.set(
            TransformFeatureFlags::ES2022_STATIC_BLOCKS,
            self.es2022.static_blocks,
        );
        features.set(
            TransformFeatureFlags::PROPOSAL_PREVENT_EXTENSIONS,
            self.proposal.prevent_extensions,
        );
        features
    }

    /// Semantic information that must be collected before this transform runs.
    pub fn semantic_features(&self) -> SemanticFeatureFlags {
        let mut features = SemanticFeatureFlags::empty();
        if self.es2022.static_blocks {
            features |= SemanticFeatureFlags::CLASS_PRIVATE_NAMES
                | SemanticFeatureFlags::STATIC_BLOCK_COMMENTS;
            if self.proposal.prevent_extensions {
                features |= SemanticFeatureFlags::STATIC_BLOCK_LAYOUT;
            }
        }
        if self.es2015.arrow_functions.is_some() {
            features |= SemanticFeatureFlags::ARROW_ENVIRONMENTS
                | SemanticFeatureFlags::LEXICAL_SUPER
                | SemanticFeatureFlags::DERIVED_CONSTRUCTORS
                | SemanticFeatureFlags::CONSTRUCTOR_THIS
                | SemanticFeatureFlags::CONSTRUCTOR_SUPER;
        }
        features
    }
}

pub struct Transformer {
    x0_es2015: es2015::ES2015,
    x0_es2022: es2022::ES2022,
}

impl Transformer {
    pub fn transform<'ast>(
        ast: &mut Ast<'ast>,
        semantic: &mut Semantic<'ast>,
        options: TransformerOptions,
    ) {
        let required_features = options.semantic_features();
        let features = options.transform_features();
        assert!(
            semantic.features().contains(required_features),
            "semantic analysis did not collect all features required by the transformer"
        );
        let program = ast.root_program();

        let mut transformer = Self {
            x0_es2015: es2015::ES2015::new(options.es2015, features),
            x0_es2022: es2022::ES2022::new(features),
        };

        let state = TransformerState::new(semantic, options.assumptions);
        let mut ctx = TraverseMutCtx::new(ast, state);
        ctx.mutate().normalize();
        program.traverse_mut_with(&mut transformer, &mut ctx);
        ctx.mutate().finish();
        debug_assert!(transformer.x0_es2022.is_empty());
    }
}

impl<'s, 'a> TraverseMut<'a, TransformerState<'s, 'a>> for Transformer {
    fn enter_node(
        &mut self,
        node: NodeId,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        ctx.state.path.push(node);
    }

    fn exit_node(
        &mut self,
        node: NodeId,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        debug_assert_eq!(ctx.state.path.pop(), Some(node));
    }

    fn enter_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        ctx.enter_statement_container(node.node_id());
        ctx.enter_var_scope(node.node_id());
    }

    fn enter_class_body(
        &mut self,
        node: ClassBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2022.enter_class_body(node, ctx);
    }

    fn exit_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2015.exit_call_expression(node, ctx);
    }

    fn exit_class(
        &mut self,
        node: Class,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2022.exit_class(node, ctx);
    }

    fn exit_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        ctx.inject_helpers(node);
        self.x0_es2015.exit_program(node, ctx);
        ctx.flush_statements(node.node_id());
        ctx.exit_var_scope(node.node_id());
        ctx.exit_statement_container(node.node_id());
    }

    fn enter_function(
        &mut self,
        node: Function,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2015.enter_function(node, ctx);
        let body = node.body(ctx.ast()).node_id();
        ctx.enter_var_scope(body);
    }

    fn exit_function(
        &mut self,
        node: Function,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2015.exit_function(node, ctx);
        let body = node.body(ctx.ast()).node_id();
        ctx.flush_statements(body);
        ctx.exit_var_scope(body);
    }

    fn enter_function_body(
        &mut self,
        node: FunctionBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        ctx.enter_statement_container(node.node_id());
    }

    fn exit_function_body(
        &mut self,
        node: FunctionBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        // Ordinary functions enqueue lexical captures on exit_function.
        // Arrow bodies can flush now, before their owner is converted.
        if !ctx
            .parent()
            .is_some_and(|parent| matches!(ctx.ast().kind_data(parent), NodeKindData::Function(_)))
        {
            ctx.flush_statements(node.node_id());
        }
        ctx.exit_statement_container(node.node_id());
    }

    fn enter_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        ctx.enter_statement_container(node.node_id());
    }

    fn exit_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        ctx.flush_statements(node.node_id());
        ctx.exit_statement_container(node.node_id());
    }

    fn enter_static_block(
        &mut self,
        node: StaticBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2015.enter_static_block();
        ctx.enter_statement_container(node.node_id());
        ctx.enter_var_scope(node.node_id());
    }

    fn exit_static_block(
        &mut self,
        node: StaticBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2015.exit_static_block(node, ctx);
        ctx.flush_statements(node.node_id());
        ctx.exit_var_scope(node.node_id());
        ctx.exit_statement_container(node.node_id());
    }

    fn enter_switch_case(
        &mut self,
        node: SwitchCase,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        ctx.enter_statement_container(node.node_id());
    }

    fn exit_switch_case(
        &mut self,
        node: SwitchCase,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        ctx.flush_statements(node.node_id());
        ctx.exit_statement_container(node.node_id());
    }

    fn enter_ts_module_block(
        &mut self,
        node: TsModuleBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        ctx.enter_statement_container(node.node_id());
        ctx.enter_var_scope(node.node_id());
    }

    fn exit_ts_module_block(
        &mut self,
        node: TsModuleBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        ctx.flush_statements(node.node_id());
        ctx.exit_var_scope(node.node_id());
        ctx.exit_statement_container(node.node_id());
    }

    fn enter_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        let property_initializer_owner = ctx
            .semantic()
            .class()
            .property_initializer_owner(node.node_id());
        self.x0_es2015
            .enter_arrow_function_expression(node, property_initializer_owner, ctx);
        let container = node
            .body(ctx.ast())
            .as_function_body(ctx.ast())
            .map_or(node.node_id(), |body| body.node_id());
        ctx.enter_var_scope(container);
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        let container = node
            .body(ctx.ast())
            .as_function_body(ctx.ast())
            .map_or(node.node_id(), |body| body.node_id());
        ctx.flush_arrow_statements(node);
        ctx.exit_var_scope(container);
        self.x0_es2015.exit_arrow_function_expression(node, ctx);
    }

    fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2015.exit_object_property(node, ctx);
    }

    fn exit_reg_exp_literal(
        &mut self,
        node: RegExpLiteral,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2015.exit_reg_exp_literal(node, ctx);
    }

    fn exit_this_expression(
        &mut self,
        node: ThisExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2015.exit_this_expression(node, ctx);
    }

    fn enter_identifier_reference(
        &mut self,
        node: IdentifierReference,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2015.enter_identifier_reference(node, ctx);
    }

    fn exit_meta_property(
        &mut self,
        node: MetaProperty,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let ctx = &mut ctx.mutate();
        self.x0_es2015.exit_meta_property(node, ctx);
    }
}

#[cfg(test)]
#[path = "../tests/lib.rs"]
mod tests;
