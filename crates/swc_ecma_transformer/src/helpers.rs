use std::collections::BTreeMap;

use swc_next_ecma_ast::{ReferenceId, Stmt};

use crate::context::BoundIdentifier;
pub(crate) use swc_next_helpers::Helper as HelperKind;

/// How runtime helpers are provided to transformed code.
///
/// Inlining reports a diagnostic when a helper's free name conflicts with a
/// binding whose original name is observable through dynamic lookup or the
/// script's global environment, including lexical bindings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HelperMode {
    /// Import helpers from `@swc-next/helpers` in modules and inline them in scripts and CommonJS.
    #[default]
    External,
    /// Parse the packaged helper sources and inject their declarations and dependencies.
    Inline,
}

/// Prepared helper declarations, allocated on first use and attached once.
pub(crate) struct Helpers<'a> {
    pub(crate) mode: HelperMode,
    namespace: Option<PreparedHelper>,
    entries: [Option<PreparedHelper>; HelperKind::ALL.len()],
    pub(crate) preparing: [bool; HelperKind::ALL.len()],
    pub(crate) globals: BTreeMap<&'a str, Vec<ReferenceId>>,
}

pub(crate) struct PreparedHelper {
    pub(crate) bound_identifier: BoundIdentifier,
    pub(crate) statement: Stmt,
}

impl Helpers<'_> {
    pub(crate) fn new(mode: HelperMode) -> Self {
        Self {
            mode,
            namespace: None,
            entries: Default::default(),
            preparing: Default::default(),
            globals: BTreeMap::new(),
        }
    }

    pub(crate) fn bound_identifier(&self) -> Option<BoundIdentifier> {
        self.namespace
            .as_ref()
            .map(|helper| helper.bound_identifier)
    }

    pub(crate) fn insert(&mut self, prepared: PreparedHelper) {
        debug_assert!(self.namespace.is_none());
        self.namespace = Some(prepared);
    }

    pub(crate) fn inline_binding(&self, helper: HelperKind) -> Option<BoundIdentifier> {
        self.entries[helper as usize]
            .as_ref()
            .map(|entry| entry.bound_identifier)
    }

    pub(crate) fn insert_inline(&mut self, helper: HelperKind, prepared: PreparedHelper) {
        let entry = &mut self.entries[helper as usize];
        debug_assert!(entry.is_none());
        *entry = Some(prepared);
    }

    pub(crate) fn take_statements(&mut self) -> impl Iterator<Item = Stmt> + '_ {
        self.namespace
            .take()
            .into_iter()
            .chain(self.entries.iter_mut().filter_map(Option::take))
            .map(|helper| helper.statement)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.namespace.is_none()
            && self.entries.iter().all(Option::is_none)
            && self.globals.is_empty()
    }
}
