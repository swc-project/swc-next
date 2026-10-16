// TODO: Support inline helpers and provide swc-next's own helper library.

use swc_next_ecma_ast::{Ast, AstInternedUtf8, Stmt};

use crate::context::BoundIdentifier;

/// Prepared external helper imports, allocated on first use and attached once.
#[derive(Default)]
pub(crate) struct Helpers {
    require: Option<AstInternedUtf8>,
    set_function_name: Option<PreparedHelper>,
    to_property_key: Option<PreparedHelper>,
}

/// One generated helper name and its detached external declaration.
/// For example, `_swc_helpers` accompanies
/// `import * as _swc_helpers from "@swc/helpers";` until program injection.
pub(crate) struct PreparedHelper {
    pub(crate) bound_identifier: BoundIdentifier,
    pub(crate) statement: Stmt,
}

/// Selects the runtime package and callee shape for a supported helper.
#[derive(Clone, Copy)]
pub(crate) enum HelperKind {
    /// `_set_function_name(target, name)` from Babel's default export.
    SetFunctionName,
    /// `_swc_helpers._to_property_key(value)` from the SWC namespace.
    ToPropertyKey,
}

impl Helpers {
    /// Return the cached local binding name, or `None` before preparation or
    /// after draining the declaration. The name is the local identifier, such as
    /// `_swc_helpers2`, rather than a member expression or runtime package path.
    pub(crate) fn bound_identifier(&self, helper: HelperKind) -> Option<BoundIdentifier> {
        let entry = match helper {
            HelperKind::SetFunctionName => &self.set_function_name,
            HelperKind::ToPropertyKey => &self.to_property_key,
        };
        entry.as_ref().map(|entry| entry.bound_identifier)
    }

    /// Cache the callee spelling, without inventing a local `require` binding.
    /// Each call site registers a fresh reference in the program scope.
    pub(crate) fn require(&mut self, ast: &mut Ast) -> AstInternedUtf8 {
        *self
            .require
            .get_or_insert_with(|| ast.add_interned_utf8("require"))
    }

    /// Cache a newly prepared helper for reuse and later program injection.
    /// Each kind must be absent before insertion; repeated uses read `name`
    /// instead, so several calls share one external declaration:
    /// ```js
    /// import * as _swc_helpers from "@swc/helpers";
    /// _swc_helpers._to_property_key(a);
    /// _swc_helpers._to_property_key(b);
    /// ```
    pub(crate) fn insert(&mut self, helper: HelperKind, prepared: PreparedHelper) {
        let entry = match helper {
            HelperKind::SetFunctionName => &mut self.set_function_name,
            HelperKind::ToPropertyKey => &mut self.to_property_key,
        };
        debug_assert!(entry.is_none());
        *entry = Some(prepared);
    }

    /// Drain prepared declarations in stable order: ToPropertyKey first,
    /// then SetFunctionName, regardless of which was used first. An unused kind
    /// contributes `None`; the returned declarations still need attachment.
    ///
    /// When both helpers are used in a module, the order is:
    /// ```js
    /// import * as _swc_helpers from "@swc/helpers";
    /// import _set_function_name from "@babel/runtime/helpers/setFunctionName";
    /// ```
    pub(crate) fn take_statements(&mut self) -> [Option<Stmt>; 2] {
        // Keep import order stable regardless of the order of first use.
        [self.to_property_key.take(), self.set_function_name.take()]
            .map(|helper| helper.map(|helper| helper.statement))
    }

    /// Whether no prepared declarations remain in this cache. Used at transform
    /// completion to check that all helpers were handed to statement injection;
    /// the injector separately checks that its queue was flushed.
    pub(crate) fn is_empty(&self) -> bool {
        self.set_function_name.is_none() && self.to_property_key.is_none()
    }
}
