//! Project retained type members onto the declaration surface.

use swc_next_ecma_ast::{
    Ast, Program, SubRange, TsMethodSignatureKind, TsTypeElement, TsTypeElementData, VisitMut,
    VisitMutWith,
};

use super::Transformer;

impl<'ast> Transformer<'_, 'ast> {
    pub(super) fn transform_type_surfaces(&mut self, program: Program, ast: &mut Ast<'ast>) {
        let mut visitor = TypeSurfaceVisitor {
            transformer: self,
            ast,
        };
        let _ = program.visit_mut_with(&mut visitor);
    }
}

struct TypeSurfaceVisitor<'transform, 'ast, 'semantic> {
    transformer: &'transform mut Transformer<'semantic, 'ast>,
    ast: &'transform mut Ast<'ast>,
}

impl<'ast, 'semantic> VisitMut<'ast> for TypeSurfaceVisitor<'_, 'ast, 'semantic> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_ts_type_elements(
        &mut self,
        elements: SubRange<TsTypeElement>,
    ) -> Option<SubRange<TsTypeElement>> {
        self.transformer
            .transform_type_accessor_elements(elements, self.ast);
        self.transformer
            .validate_computed_type_elements(elements, self.ast);
        for element in elements.iter() {
            let element = self.ast.get_node_in_sub_range(element);
            if let TsTypeElementData::TsMethodSignature(signature) =
                self.ast.ts_type_element_data(element)
                && signature.kind(self.ast) == TsMethodSignatureKind::Method
                && !self
                    .transformer
                    .validated_type_methods
                    .contains(&signature.node_id())
            {
                self.transformer
                    .validate_type_method_signature(signature, self.ast);
            }
        }

        let _ = elements.visit_mut_children_with(self);
        Some(elements)
    }
}
