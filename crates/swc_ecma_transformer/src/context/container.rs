use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{ClassBody, ClassElement, SubRange};

use crate::context::TransformCtx;

#[derive(Clone, Copy)]
pub enum ContainerOwner {
    ClassBody(ClassBody),
}

#[derive(Clone, Copy)]
pub enum ContainerOwnerChild {
    ClassBody((ClassBody, ClassElement)),
}

#[derive(Default)]
pub struct Container {
    class_body: FxHashMap<ClassBody, Vec<ClassElement>>,
}

impl Container {
    pub fn is_empty(&self) -> bool {
        self.class_body.is_empty()
    }
}

impl<'c, 'a> TransformCtx<'c, 'a> {
    /// Read pending members, or the attached AST range if no edits are cached.
    /// Reading does not allocate or initialize a mutable container.
    pub fn container_class_body(
        &self,
        class_body: ClassBody,
    ) -> impl DoubleEndedIterator<Item = ClassElement> + '_ {
        let elements = self.state.container.class_body.get(&class_body);
        let body = if elements.is_some() {
            SubRange::empty()
        } else {
            class_body.body(self.ast)
        };
        elements
            .into_iter()
            .flatten()
            .copied()
            .chain(body.iter().map(|slot| self.ast.get_node_in_sub_range(slot)))
    }

    /// The current logical member list, including pending edits.
    pub fn container_class_body_mut(&mut self, class_body: ClassBody) -> &mut Vec<ClassElement> {
        self.state
            .container
            .class_body
            .entry(class_body)
            .or_insert_with(|| {
                class_body
                    .body(self.ast)
                    .iter()
                    .map(|slot| self.ast.get_node_in_sub_range(slot))
                    .collect()
            })
    }

    /// Attach a prepared list and release its cached handles. This only changes
    /// the attachment range, so it allocates no AST nodes or semantic identities.
    /// Callers must synchronize semantic changes when preparing member edits.
    pub fn container_flush(&mut self, owner: ContainerOwner) {
        match owner {
            ContainerOwner::ClassBody(class_body) => {
                let Some(elements) = self.state.container.class_body.remove(&class_body) else {
                    return;
                };
                let body = class_body.body(self.ast);
                if body.len() == elements.len()
                    && body.iter().zip(&elements).all(|(slot, element)| {
                        self.ast.get_node_in_sub_range(slot).node_id() == element.node_id()
                    })
                {
                    return;
                }
                let body = self.ast.add_sub_range(elements);
                class_body.set_body(self.ast, body);
            }
        }
    }
}
