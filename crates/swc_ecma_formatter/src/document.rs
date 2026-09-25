//! Flat documents containing borrowed text and explicit line breaks.
//!
//! Construction owns a growable instruction buffer in the formatting arena.
//! Finishing seals an immutable slice; source text remains borrowed from its owner.

mod printer;
#[cfg(test)]
mod tests;
mod text;
mod writer;

pub(crate) use writer::Writer;

use crate::FormatOptions;
use text::Text;

#[derive(Debug, Clone, Copy)]
pub(crate) struct GroupId(usize);

/// Explicit layout requests stay distinct from preserved source layout.
#[derive(Debug, Clone, Copy)]
pub(crate) enum GroupRequest {
    Auto,
    Expanded,
    PreserveSource,
}

impl From<bool> for GroupRequest {
    fn from(expand: bool) -> Self {
        if expand { Self::Expanded } else { Self::Auto }
    }
}

/// Effects join within each complete scope. Source preferences remain effective
/// when printing, while required expansion also constrains canonical measurement.
#[derive(Debug, Clone, Copy, Default)]
struct BreakEffect(u8);

impl BreakEffect {
    const SOURCE: Self = Self(1);
    const REQUIRED: Self = Self(2);

    fn from_request(request: GroupRequest) -> Self {
        match request {
            GroupRequest::Auto => Self::default(),
            GroupRequest::Expanded => Self::REQUIRED,
            GroupRequest::PreserveSource => Self::SOURCE,
        }
    }

    fn join(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    fn preferred_breaks(self) -> bool {
        self.0 != 0
    }

    fn required(self) -> bool {
        self.0 & Self::REQUIRED.0 != 0
    }
}

#[derive(Debug, Clone, Copy)]
enum Measurement {
    Display,
    Canonical,
}

#[derive(Debug, Clone, Copy)]
enum SpaceKind {
    Deferred,
    /// One canonical gap reserves fit width before the next text commits it.
    Reserved,
}

#[derive(Debug, Clone, Copy)]
enum FormatElement<'a> {
    Text(Text<'a>),
    Space(SpaceKind),
    SoftLine,
    SoftLineIfBreak(GroupId),
    Line,
    HardLine,
    BreakParent,
    LiteralLine,
    IfBreak(Text<'a>),
    IfFlat(Text<'a>),
    IfCurrentBreak(Text<'a>),
    IfCurrentFlat(Text<'a>),
    LineSuffix(Text<'a>),
    StartGroup {
        effect: BreakEffect,
        id: Option<GroupId>,
    },
    EndGroup,
    FlatGroup(GroupId),
    StartIndent,
    StartAbsoluteIndent {
        width: usize,
    },
    StartIndentIfBreak(GroupId),
    EndIndent,
    StartAlign {
        width: usize,
    },
    EndAlign {
        width: usize,
    },
    StartFill,
    EndFill,
    /// Distance from this opening instruction to its matching EndEntry.
    StartEntry {
        end_offset: usize,
    },
    EndEntry,
    Fragment(Fragment<'a>),
    FlatFragment(FlatFragment<'a>),
    Alternatives(&'a Alternatives<'a>),
    CanonicalAlternatives(&'a Alternatives<'a>),
}

/// A complete immutable scope, owned by the current formatting call.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Fragment<'a> {
    elements: &'a [FormatElement<'a>],
    effect: BreakEffect,
    capabilities: FragmentCapabilities,
    conditional_lines: bool,
}

/// Both views retain the same complete scope; only the anonymous outer group differs.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RecordedGroup<'a> {
    grouped: Fragment<'a>,
}

impl<'a> RecordedGroup<'a> {
    pub(crate) fn grouped(self) -> Fragment<'a> {
        self.grouped
    }

    pub(crate) fn contents(self) -> Fragment<'a> {
        // Construction retains one complete anonymous Auto group. Its wrapper
        // inherits the contents' effects and capabilities without adding any.
        let elements = self.grouped.elements;
        let elements = &elements[1..elements.len() - 1];
        Fragment {
            elements,
            ..self.grouped
        }
    }
}

impl<'a> Fragment<'a> {
    pub(crate) fn preferred_breaks(self) -> bool {
        self.effect.preferred_breaks()
    }

    /// Measures this complete scope without changing its retained render view.
    pub(crate) fn canonical_candidate(self) -> CanonicalCandidate<'a> {
        CanonicalCandidate(self)
    }

    pub(crate) fn try_flat(self) -> Option<FlatFragment<'a>> {
        self.capabilities
            .permits_flat()
            .then_some(FlatFragment(self))
    }
}

/// A zero-copy measurement request, distinct from a printed Flat projection.
#[derive(Clone, Copy)]
pub(crate) struct CanonicalCandidate<'a>(Fragment<'a>);

#[derive(Debug, Clone, Copy)]
enum FragmentCapabilities {
    General,
    FlatEligible,
    Neutral,
}

impl FragmentCapabilities {
    fn from_flat(flat: bool) -> Self {
        if flat {
            Self::FlatEligible
        } else {
            Self::General
        }
    }

    fn permits_flat(self) -> bool {
        !matches!(self, Self::General)
    }

    /// Fresh Writer records contain complete, balanced scopes. Once neutrality
    /// is ruled out, the same iterator resumes the existing flat-only scan.
    fn classify(elements: &[FormatElement<'_>]) -> Self {
        let mut elements = elements.iter().copied();
        while let Some(element) = elements.next() {
            let flat = match element {
                FormatElement::Text(text) => {
                    if !text.is_single_line() {
                        return Self::General;
                    }
                    continue;
                }
                FormatElement::Space(_)
                | FormatElement::StartGroup {
                    effect: BreakEffect(0),
                    id: None,
                }
                | FormatElement::EndGroup => continue,
                FormatElement::Fragment(fragment) => match fragment.capabilities {
                    Self::Neutral => continue,
                    Self::General => return Self::General,
                    Self::FlatEligible => true,
                },
                FormatElement::FlatFragment(fragment)
                    if matches!(fragment.0.capabilities, Self::Neutral) =>
                {
                    continue;
                }
                FormatElement::FlatFragment(_) => true,
                element => element.permits_flat(),
            };
            return Self::from_flat(flat && elements.all(FormatElement::permits_flat));
        }
        Self::Neutral
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FlatFragment<'a>(Fragment<'a>);

#[derive(Debug)]
struct Alternatives<'a> {
    preferred: &'a [Fragment<'a>],
    fallback: Fragment<'a>,
}

impl<'a> Alternatives<'a> {
    /// Flat views and their eligibility always follow this exact path.
    fn first(&self) -> Fragment<'a> {
        self.preferred.first().copied().unwrap_or(self.fallback)
    }
}

impl FormatElement<'_> {
    fn permits_flat(self) -> bool {
        match self {
            Self::HardLine | Self::LiteralLine | Self::LineSuffix(_) | Self::BreakParent => false,
            Self::Text(text) | Self::IfCurrentFlat(text) => text.is_single_line(),
            Self::Fragment(fragment) => fragment.capabilities.permits_flat(),
            Self::Alternatives(alternatives) | Self::CanonicalAlternatives(alternatives) => {
                alternatives.first().capabilities.permits_flat()
            }
            _ => true,
        }
    }

    /// Applies a flat view without changing any retained instructions.
    fn flattened(self) -> Option<Self> {
        match self {
            Self::Line => Some(Self::Space(SpaceKind::Deferred)),
            Self::IfFlat(text) | Self::IfCurrentFlat(text) => Some(Self::Text(text)),
            Self::StartGroup { id: Some(id), .. } => Some(Self::FlatGroup(id)),
            Self::Fragment(fragment) => Some(Self::FlatFragment(FlatFragment(fragment))),
            Self::Alternatives(alternatives) | Self::CanonicalAlternatives(alternatives) => {
                Some(Self::FlatFragment(FlatFragment(alternatives.first())))
            }
            element @ (Self::Text(_)
            | Self::Space(_)
            | Self::FlatGroup(_)
            | Self::FlatFragment(_)) => Some(element),
            _ => None,
        }
    }
}

pub(crate) struct Doc<'a> {
    elements: &'a [FormatElement<'a>],
    group_count: usize,
    conditional_lines: bool,
}

impl Doc<'_> {
    pub(crate) fn print(self, options: FormatOptions) -> String {
        printer::print(
            self.elements,
            self.group_count,
            self.conditional_lines,
            options,
        )
    }
}
