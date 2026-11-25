//! Decorator transforms ported from legacy SWC.

use swc_next_ecma_ast::*;

use crate::common::TraverseCtx;

mod legacy;
mod v2023_11;

/// Decorator proposal version to lower.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DecoratorVersion {
    /// Legacy (Stage 1 / TypeScript `experimentalDecorators`).
    Legacy,
    /// 2023-11 decorators proposal.
    #[default]
    V2023_11,
}

/// Decorator transform options.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DecoratorOptions {
    /// Decorator version to lower.
    pub version: DecoratorVersion,
    /// Emit `Reflect.metadata` design-type metadata (`emitDecoratorMetadata`).
    /// Applies to legacy decorators only.
    pub metadata: bool,
}

/// SWC: <https://github.com/swc-project/swc/blob/be15e4b756b61c64c00f6bb4d03d71aae1cd2d2f/crates/swc/src/config/mod.rs#L913>.
pub(crate) fn transform(ast: &mut Ast<'_>, ctx: &mut TraverseCtx<'_>, options: DecoratorOptions) {
    match options.version {
        DecoratorVersion::Legacy => legacy::transform(ast, ctx, options.metadata),
        DecoratorVersion::V2023_11 => v2023_11::transform(ast, ctx),
    }
}

struct DecoratorFinder<'a, 'ast> {
    ast: &'a Ast<'ast>,
    found: bool,
}

impl<'ast> Visit<'ast> for DecoratorFinder<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_decorator(&mut self, _: Decorator) {
        self.found = true;
    }
}

fn contains_decorator<'ast, N>(ast: &Ast<'ast>, node: N) -> bool
where
    for<'a> N: VisitWith<'ast, DecoratorFinder<'a, 'ast>>,
{
    let mut finder = DecoratorFinder { ast, found: false };
    node.visit_with(&mut finder);
    finder.found
}
