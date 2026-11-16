use swc_next_ecma_ast::Stmt;

use crate::context::BoundIdentifier;
pub(crate) use swc_next_helpers::Helper as HelperKind;

/// Prepared external declarations, allocated on first use and attached once.
#[derive(Default)]
pub(crate) struct Helpers {
    entries: [Option<PreparedHelper>; HelperKind::ALL.len()],
}

pub(crate) struct PreparedHelper {
    pub(crate) bound_identifier: BoundIdentifier,
    pub(crate) statement: Stmt,
}

impl Helpers {
    pub(crate) fn bound_identifier(&self, helper: HelperKind) -> Option<BoundIdentifier> {
        self.entries[helper as usize]
            .as_ref()
            .map(|entry| entry.bound_identifier)
    }

    pub(crate) fn insert(&mut self, helper: HelperKind, prepared: PreparedHelper) {
        let entry = &mut self.entries[helper as usize];
        debug_assert!(entry.is_none());
        *entry = Some(prepared);
    }

    pub(crate) fn take_statements(&mut self) -> impl Iterator<Item = Stmt> + '_ {
        self.entries
            .iter_mut()
            .filter_map(|entry| entry.take().map(|helper| helper.statement))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.iter().all(Option::is_none)
    }
}
