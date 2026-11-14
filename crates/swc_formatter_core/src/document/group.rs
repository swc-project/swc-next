//! One group contract with static dispatch over compact and source storage.
//!
//! Ordinary groups and cached flat measurements remain inline instructions.
//! Candidate groups share operations without a runtime wrapper or extra lowering.
//! Source fields and executable layouts have distinct traversal policies.

use rustc_hash::FxHashSet;
use swc_next_allocator::{Allocator, vec::ArenaVec};

use super::{
    Fragment, GroupId, GroupRequest, Instruction,
    layout::{self, Mode},
};

/// Anonymous Auto storage whose primary is its first candidate.
#[derive(Debug)]
pub(super) struct CompactGroup<'a> {
    /// Leading `expandedStates`; the first is also the Prettier group's `contents`.
    preferred: &'a [Fragment<'a>],
    /// Final `expandedStates` entry, used when no preferred layout fits.
    fallback: Fragment<'a>,
}

/// Storage retaining independent contents and original expanded states.
#[derive(Debug)]
pub(super) struct SourceGroup<'a> {
    /// Prettier `contents`, retained independently of `expandedStates[0]`.
    contents: Fragment<'a>,
    /// Original nonempty `expandedStates`, retained independently of `contents`.
    /// Execution uses `contents`, states between first and last, then the last state.
    /// A single state therefore remains the fallback instead of replacing `contents`.
    states: &'a [Fragment<'a>],
    /// Builder `shouldBreak`, stored as `break` on the Prettier Doc.
    request: GroupRequest,
    /// Prettier group `id`, remapped to a local ID; `None` is anonymous.
    id: Option<GroupId>,
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(std::mem::size_of::<CompactGroup<'_>>() <= 32);
    assert!(std::mem::size_of::<SourceGroup<'_>>() <= 40);
};

pub(super) struct GroupLayouts<'a> {
    /// Executable Prettier `contents`, kept independent of the original states.
    pub(super) primary: &'a Fragment<'a>,
    /// Executable intermediate `expandedStates`, after the primary and before fallback.
    pub(super) alternatives: &'a [Fragment<'a>],
    /// Final `expandedStates` entry, used when expanded or no preferred layout fits.
    pub(super) fallback: &'a Fragment<'a>,
}

impl<'a> GroupLayouts<'a> {
    pub(super) fn all(&self) -> impl DoubleEndedIterator<Item = Fragment<'a>> + '_ {
        std::iter::once(*self.primary)
            .chain(self.alternatives.iter().copied())
            .chain(std::iter::once(*self.fallback))
    }
}

#[inline(always)]
pub(super) fn compact<'a>(
    allocator: &'a Allocator,
    preferred: &'a [Fragment<'a>],
    fallback: Fragment<'a>,
) -> &'a CompactGroup<'a> {
    debug_assert!(!preferred.is_empty());
    allocator.alloc(CompactGroup {
        preferred,
        fallback,
    })
}

#[inline(always)]
pub(super) fn with_states<'a>(
    allocator: &'a Allocator,
    request: GroupRequest,
    id: Option<GroupId>,
    contents: Fragment<'a>,
    states: &'a [Fragment<'a>],
) -> &'a SourceGroup<'a> {
    debug_assert!(!states.is_empty());
    allocator.alloc(SourceGroup {
        contents,
        states,
        request,
        id,
    })
}

/// A borrowed logical group, implemented only by the two arena storage shapes.
/// Static dispatch retains each shape's concrete candidate iterator and elides
/// constant Auto/anonymous metadata from the compact printing path.
pub(super) trait Group<'a>: Copy {
    const SOURCE_FIELDS: bool;

    fn element(self) -> Instruction<'a>;
    fn key(self) -> usize;
    fn request(self) -> GroupRequest;
    fn id(self) -> Option<GroupId>;
    fn primary_ref(self) -> &'a Fragment<'a>;
    fn fallback_ref(self) -> &'a Fragment<'a>;
    fn layouts(self) -> GroupLayouts<'a>;
    fn preferred(self) -> impl Iterator<Item = &'a Fragment<'a>>;
    /// Raw states for source storage, preferred candidates for compact storage.
    fn stored_states(self) -> &'a [Fragment<'a>];

    #[inline]
    fn primary(self) -> Fragment<'a> {
        *self.primary_ref()
    }

    #[inline]
    fn fallback(self) -> Fragment<'a> {
        *self.fallback_ref()
    }

    #[inline]
    fn has_source_fields(self) -> bool {
        Self::SOURCE_FIELDS
    }

    /// Recognize the compact contract without changing original source fields.
    #[inline]
    fn requires_analysis(self) -> bool {
        Self::SOURCE_FIELDS
            && (!matches!(self.request(), GroupRequest::Auto)
                || self.id().is_some()
                || !same(self.primary(), self.stored_states()[0]))
    }

    #[inline(always)]
    fn layout_flags(self) -> u8 {
        let layouts = self.layouts();
        layouts.alternatives.iter().fold(
            layouts.primary.summary.0 | layouts.fallback.summary.0,
            |flags, state| flags | state.summary.0,
        )
    }

    /// Structural traversal visits every retained field, including unused states.
    fn source_fragments(self) -> impl Iterator<Item = Fragment<'a>>;

    #[inline(always)]
    fn select(
        self,
        inherit_flat: bool,
        fits: impl FnMut(&Fragment<'a>) -> bool,
    ) -> (Fragment<'a>, Mode) {
        layout::select(
            self.request(),
            inherit_flat,
            || self.primary_ref(),
            self.fallback_ref(),
            self.preferred(),
            fits,
        )
    }

    fn map_sources(
        self,
        allocator: &'a Allocator,
        lookup: impl Fn(Fragment<'a>) -> Fragment<'a>,
        remap_id: impl FnOnce(GroupId) -> Option<GroupId>,
    ) -> Option<Instruction<'a>>;

    fn replace_primary(self, allocator: &'a Allocator, contents: Fragment<'a>) -> Instruction<'a>;
}

impl<'a> Group<'a> for &'a CompactGroup<'a> {
    const SOURCE_FIELDS: bool = false;
    #[inline]
    fn element(self) -> Instruction<'a> {
        Instruction::CompactGroup(self)
    }
    #[inline]
    fn key(self) -> usize {
        self as *const _ as usize
    }
    #[inline]
    fn request(self) -> GroupRequest {
        GroupRequest::Auto
    }
    #[inline]
    fn id(self) -> Option<GroupId> {
        None
    }
    #[inline]
    fn primary_ref(self) -> &'a Fragment<'a> {
        &self.preferred[0]
    }
    #[inline]
    fn primary(self) -> Fragment<'a> {
        self.preferred.first().copied().unwrap_or(self.fallback)
    }
    #[inline]
    fn fallback(self) -> Fragment<'a> {
        self.fallback
    }
    #[inline]
    fn fallback_ref(self) -> &'a Fragment<'a> {
        &self.fallback
    }
    #[inline]
    fn preferred(self) -> impl Iterator<Item = &'a Fragment<'a>> {
        self.preferred.iter()
    }
    #[inline]
    fn stored_states(self) -> &'a [Fragment<'a>] {
        self.preferred
    }
    fn layouts(self) -> GroupLayouts<'a> {
        GroupLayouts {
            primary: self.primary_ref(),
            alternatives: self.preferred.get(1..).unwrap_or_default(),
            fallback: &self.fallback,
        }
    }
    #[inline(always)]
    fn layout_flags(self) -> u8 {
        self.preferred
            .iter()
            .fold(self.fallback.summary.0, |flags, state| {
                flags | state.summary.0
            })
    }
    #[inline]
    fn source_fragments(self) -> impl Iterator<Item = Fragment<'a>> {
        std::iter::once(self.fallback).chain(self.preferred.iter().copied())
    }

    #[inline(always)]
    fn map_sources(
        self,
        allocator: &'a Allocator,
        lookup: impl Fn(Fragment<'a>) -> Fragment<'a>,
        _: impl FnOnce(GroupId) -> Option<GroupId>,
    ) -> Option<Instruction<'a>> {
        let fallback = lookup(self.fallback);
        if same(self.fallback, fallback)
            && self
                .preferred
                .iter()
                .copied()
                .all(|state| same(state, lookup(state)))
        {
            return None;
        }
        Some(
            compact(
                allocator,
                map_slice(allocator, self.preferred, lookup),
                fallback,
            )
            .element(),
        )
    }

    fn replace_primary(self, allocator: &'a Allocator, contents: Fragment<'a>) -> Instruction<'a> {
        let mut states = ArenaVec::with_capacity_in(self.preferred.len() + 1, allocator);
        states.extend_from_slice(self.preferred);
        states.push(self.fallback);
        with_states(
            allocator,
            GroupRequest::Auto,
            None,
            contents,
            states.into_bump_slice(),
        )
        .element()
    }
}

impl<'a> Group<'a> for &'a SourceGroup<'a> {
    const SOURCE_FIELDS: bool = true;
    #[inline]
    fn element(self) -> Instruction<'a> {
        Instruction::SourceGroup(self)
    }
    #[inline]
    fn key(self) -> usize {
        self as *const _ as usize
    }
    #[inline]
    fn request(self) -> GroupRequest {
        self.request
    }
    #[inline]
    fn id(self) -> Option<GroupId> {
        self.id
    }
    #[inline]
    fn primary_ref(self) -> &'a Fragment<'a> {
        &self.contents
    }
    #[inline]
    fn primary(self) -> Fragment<'a> {
        self.contents
    }
    #[inline]
    fn fallback_ref(self) -> &'a Fragment<'a> {
        self.states.last().expect("group states are nonempty")
    }
    #[inline]
    fn stored_states(self) -> &'a [Fragment<'a>] {
        self.states
    }
    fn layouts(self) -> GroupLayouts<'a> {
        let (fallback, earlier) = self.states.split_last().expect("group states are nonempty");
        GroupLayouts {
            primary: &self.contents,
            // The raw first state executes only when it is the sole fallback.
            alternatives: earlier.get(1..).unwrap_or_default(),
            fallback,
        }
    }
    #[inline]
    fn preferred(self) -> impl Iterator<Item = &'a Fragment<'a>> {
        let layouts = self.layouts();
        std::iter::once(layouts.primary).chain(layouts.alternatives.iter())
    }
    #[inline]
    fn source_fragments(self) -> impl Iterator<Item = Fragment<'a>> {
        std::iter::once(self.contents).chain(self.states.iter().copied())
    }

    #[inline(always)]
    fn map_sources(
        self,
        allocator: &'a Allocator,
        lookup: impl Fn(Fragment<'a>) -> Fragment<'a>,
        remap_id: impl FnOnce(GroupId) -> Option<GroupId>,
    ) -> Option<Instruction<'a>> {
        let contents = lookup(self.contents);
        let id = self.id.and_then(remap_id);
        if id.is_none()
            && same(self.contents, contents)
            && self
                .states
                .iter()
                .copied()
                .all(|state| same(state, lookup(state)))
        {
            return None;
        }
        Some(
            with_states(
                allocator,
                self.request,
                id.or(self.id),
                contents,
                map_slice(allocator, self.states, lookup),
            )
            .element(),
        )
    }

    fn replace_primary(self, allocator: &'a Allocator, contents: Fragment<'a>) -> Instruction<'a> {
        with_states(allocator, self.request, self.id, contents, self.states).element()
    }
}

fn same(left: Fragment<'_>, right: Fragment<'_>) -> bool {
    std::ptr::eq(left.elements(), right.elements())
}

fn map_slice<'a>(
    allocator: &'a Allocator,
    source: &[Fragment<'a>],
    lookup: impl Fn(Fragment<'a>) -> Fragment<'a>,
) -> &'a [Fragment<'a>] {
    let mut result = ArenaVec::with_capacity_in(source.len(), allocator);
    result.extend(source.iter().copied().map(lookup));
    result.into_bump_slice()
}

/// Qualify only source-bearing scopes before allocating propagation facts.
/// Keeping this query out of the packed summary preserves the native instruction
/// layout. Shared fragments are visited once; unrelated fragments are skipped.
#[inline(never)]
pub(super) fn requires_analysis<'a>(root: Fragment<'a>) -> bool {
    let mut current = root.elements();
    let mut pending = Vec::new();
    let mut visited = FxHashSet::default();
    loop {
        let Some((&element, remaining)) = current.split_first() else {
            let Some(next) = pending.pop() else {
                return false;
            };
            current = next;
            continue;
        };
        let mut child = None;
        let mut visit = |fragment: Fragment<'a>| {
            if fragment.has_source_groups()
                && visited.insert(super::transform::key(fragment.elements()))
                && let Some(previous) = child.replace(fragment.elements())
            {
                pending.push(previous);
            }
        };
        match element {
            Instruction::Fragment { elements, summary }
            | Instruction::FlatFragment { elements, summary } => {
                visit(Fragment { elements, summary })
            }
            Instruction::IfBreak(conditional) | Instruction::FlatIfBreak(conditional) => {
                visit(conditional.broken);
                visit(conditional.flat);
            }
            Instruction::CompactGroup(group) => group.layouts().all().for_each(visit),
            Instruction::SourceGroup(group) => {
                if group.requires_analysis() {
                    return true;
                }
                group.layouts().all().for_each(visit);
            }
            _ => {}
        }
        current = match child {
            Some(child) => {
                if !remaining.is_empty() {
                    pending.push(remaining);
                }
                child
            }
            None => remaining,
        };
    }
}
