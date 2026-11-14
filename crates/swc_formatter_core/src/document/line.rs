//! Line semantics at construction; execution stores each case as a flat instruction.

/// The effective mode of a raw Prettier `line` Doc.
///
/// Import normalizes `hard` before `soft`, so an input with both flags selects
/// [`Self::Hard`]. These modes do not request expansion of enclosing groups.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineMode {
    /// `hard: false, soft: false`: a space when flat, a newline when broken.
    #[default]
    Normal,
    /// `hard: false, soft: true`: empty when flat, a newline when broken.
    Soft,
    /// `hard: true`: a newline in both modes, regardless of the input's `soft` flag.
    Hard,
}

/// A raw line for [`super::Writer::raw_line`], independent of group expansion.
///
/// The mode and whitespace policy describe six valid cases. The writer selects
/// one flat instruction immediately; neither field becomes a nested runtime tag.
/// Prettier's `hardline` and `literalline` additionally compose a `breakParent`;
/// use [`super::Writer::hard_line`] or [`super::Writer::literal_line`] for those.
#[derive(Debug, Clone, Copy, Default)]
pub struct LineOptions {
    /// Effective Prettier `hard` / `soft` flags after normalization.
    pub mode: LineMode,
    /// Prettier `literal`: on a newline, preserve trailing whitespace and resume
    /// at the marked root indentation instead of the current indentation.
    pub literal: bool,
}
