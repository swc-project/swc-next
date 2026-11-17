use swc_next_ecma_ast::Stmt;

use crate::context::BoundIdentifier;
pub(crate) use swc_next_helpers::Helper as HelperKind;

/// One external helper namespace, allocated on first use and attached once.
#[derive(Default)]
pub(crate) struct Helpers {
    namespace: Option<PreparedHelper>,
}

pub(crate) struct PreparedHelper {
    pub(crate) bound_identifier: BoundIdentifier,
    pub(crate) statement: Stmt,
}

impl Helpers {
    pub(crate) fn bound_identifier(&self) -> Option<BoundIdentifier> {
        self.namespace.as_ref().map(|entry| entry.bound_identifier)
    }

    pub(crate) fn insert(&mut self, prepared: PreparedHelper) {
        let entry = &mut self.namespace;
        debug_assert!(entry.is_none());
        *entry = Some(prepared);
    }

    pub(crate) fn take_statement(&mut self) -> Option<Stmt> {
        self.namespace.take().map(|helper| helper.statement)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.namespace.is_none()
    }
}
