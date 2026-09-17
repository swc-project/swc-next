use crate::{
    Accessibility, AssignmentOperator, Ast, AstUtf8, AstWtf8, BinaryOperator, ClassType,
    FormalParameterKind, FunctionType, ImportOrExportKind, ImportPhase, LogicalOperator,
    MethodDefinitionKind, PropertyKind, SourceType, Span, TsMappedTypeModifier,
    TsMethodSignatureKind, TsModuleDeclarationKind, TsTypeOperatorKind, UnaryOperator,
    UpdateOperator, VariableKind, Visit, VisitMut, VisitMutWith, VisitWith,
};

/// Shared AST and mutable user state passed to every [`crate::Traverse`] hook.
///
/// AST mutation requires [`TraverseMutCtx`]:
pub struct TraverseCtx<'traverse, 'ast, State = ()> {
    pub state: State,
    pub ast: &'traverse Ast<'ast>,
}

impl<'traverse, 'ast, State> TraverseCtx<'traverse, 'ast, State> {
    #[inline]
    pub fn new(ast: &'traverse Ast<'ast>, state: State) -> Self {
        Self { state, ast }
    }
}

/// Mutable AST state passed to every [`crate::TraverseMut`] hook.
pub struct TraverseMutCtx<'traverse, 'ast, State = ()> {
    pub state: State,
    pub ast: &'traverse mut Ast<'ast>,
}

impl<'traverse, 'ast, State> TraverseMutCtx<'traverse, 'ast, State> {
    #[inline]
    pub fn new(ast: &'traverse mut Ast<'ast>, state: State) -> Self {
        Self { state, ast }
    }
}

macro_rules! dummy_visit_mut_impl {
    ($ty:ty) => {
        impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for $ty {
            #[inline]
            fn visit_with(self, _visitor: &mut V) {}

            #[inline]
            fn visit_children_with(self, _visitor: &mut V) {}
        }

        impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for $ty {
            #[inline]
            fn visit_mut_with(self, _visitor: &mut V) -> Option<Self> {
                None
            }

            #[inline]
            fn visit_mut_children_with(self, _visitor: &mut V) -> Option<Self> {
                None
            }
        }
    };
}

dummy_visit_mut_impl!(bool);
dummy_visit_mut_impl!(f64);
dummy_visit_mut_impl!(Span);
dummy_visit_mut_impl!(AstUtf8);
dummy_visit_mut_impl!(AstWtf8);
dummy_visit_mut_impl!(ImportPhase);
dummy_visit_mut_impl!(SourceType);
dummy_visit_mut_impl!(MethodDefinitionKind);
dummy_visit_mut_impl!(Accessibility);
dummy_visit_mut_impl!(ClassType);
dummy_visit_mut_impl!(BinaryOperator);
dummy_visit_mut_impl!(LogicalOperator);
dummy_visit_mut_impl!(UnaryOperator);
dummy_visit_mut_impl!(UpdateOperator);
dummy_visit_mut_impl!(AssignmentOperator);
dummy_visit_mut_impl!(VariableKind);
dummy_visit_mut_impl!(FunctionType);
dummy_visit_mut_impl!(FormalParameterKind);
dummy_visit_mut_impl!(PropertyKind);
dummy_visit_mut_impl!(ImportOrExportKind);
dummy_visit_mut_impl!(TsTypeOperatorKind);
dummy_visit_mut_impl!(TsMethodSignatureKind);
dummy_visit_mut_impl!(TsModuleDeclarationKind);
dummy_visit_mut_impl!(TsMappedTypeModifier);
