//! RegExp transform options, independent of the ECMAScript edition groups.

/// SWC's `swc_ecma_compat_common::regexp::Config` feature switches.
///
/// A `true` flag selects lowering of that feature. All flags default to `false`.
/// These flags describe SWC's configuration surface, not a guarantee that every
/// pattern using a selected feature can be lowered.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RegExpOptions {
    /// Lower the `s` (dotAll) flag.
    pub dot_all_regex: bool,
    /// Handle the `d` (match indices) flag.
    pub has_indices: bool,
    /// Handle lookbehind assertions.
    pub lookbehind_assertion: bool,
    /// Lower named capture groups and references.
    pub named_capturing_groups_regex: bool,
    /// Lower duplicate capture-group names in different alternatives.
    pub duplicate_named_capturing_groups_regex: bool,
    /// Lower the `y` (sticky) flag.
    pub sticky_regex: bool,
    /// Lower Unicode property escapes (`\p{...}` and `\P{...}`).
    pub unicode_property_regex: bool,
    /// Lower the `u` (Unicode) flag.
    pub unicode_regex: bool,
    /// Lower the `v` (Unicode sets) flag.
    pub unicode_sets_regex: bool,
}
