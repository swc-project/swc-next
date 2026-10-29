//! Checked decoding of the versioned document transport.
use std::{collections::HashMap, fmt, num::NonZeroU32};
use swc_next_allocator::{Allocator, vec::ArenaVec};

use super::{
    Alternatives, BreakEffect, ConditionTarget, Conditional, Doc, FormatElement, Fragment,
    FragmentCapabilities, GroupId, GroupRequest, GroupWithStates, text::Text,
};

/// An invalid, unsupported, or resource-limited document buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentBufferError {
    /// The buffer does not describe a valid supported document.
    InvalidDocument,
    /// The document format version is not supported.
    UnsupportedVersion(u32),
    /// The input or decoding work exceeds a supported limit.
    LimitExceeded,
}

impl fmt::Display for DocumentBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocument => f.write_str("invalid document buffer"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported document version {version}")
            }
            Self::LimitExceeded => f.write_str("document decoding limit exceeded"),
        }
    }
}

impl std::error::Error for DocumentBufferError {}

type Result<T> = std::result::Result<T, DocumentBufferError>;

#[derive(Debug)]
enum Op {
    Text(u32, u32),
    Line(u8),
    Start(ScopeStart),
    End(u8),
    Ref(usize),
    Choices(Vec<usize>),
    Group {
        expanded: bool,
        id: u32,
        contents: usize,
        states: Option<Vec<usize>>,
    },
    Conditional(u32, usize, usize),
    Simple(u8),
}

#[derive(Debug)]
enum ScopeStart {
    Group { expanded: bool, id: u32 },
    Indent,
    Align(u32),
    IndentIfBreak(u32),
    Conditional { broken: bool, id: u32 },
    Fill,
    Entry,
    Suffix,
    AbsoluteIndent(u32),
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn byte(&mut self) -> Result<u8> {
        let b = *self
            .bytes
            .get(self.at)
            .ok_or(DocumentBufferError::InvalidDocument)?;
        self.at += 1;
        Ok(b)
    }

    fn word(&mut self) -> Result<u32> {
        let end = self
            .at
            .checked_add(4)
            .ok_or(DocumentBufferError::InvalidDocument)?;
        let bytes = self
            .bytes
            .get(self.at..end)
            .ok_or(DocumentBufferError::InvalidDocument)?;
        self.at = end;
        Ok(u32::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| DocumentBufferError::InvalidDocument)?,
        ))
    }

    fn take(&mut self, len: usize) -> Result<&[u8]> {
        let end = self
            .at
            .checked_add(len)
            .ok_or(DocumentBufferError::InvalidDocument)?;
        let slice = self
            .bytes
            .get(self.at..end)
            .ok_or(DocumentBufferError::InvalidDocument)?;
        self.at = end;
        Ok(slice)
    }
}

fn reference(r: &mut Reader<'_>, count: usize) -> Result<usize> {
    let id = r.word()? as usize;
    if id >= count {
        return Err(DocumentBufferError::InvalidDocument);
    }
    Ok(id)
}

fn parse_fragment(bytes: &[u8], count: usize) -> Result<Vec<Op>> {
    let mut r = Reader { bytes, at: 0 };
    let mut ops = Vec::new();
    let mut stack = Vec::new();
    while r.at < bytes.len() {
        let tag = r.byte()?;
        let op = match tag {
            0 => Op::Text(r.word()?, r.word()?),
            2 => {
                let kind = r.byte()?;
                if !matches!(kind, 0..=6) {
                    return Err(DocumentBufferError::InvalidDocument);
                }
                Op::Line(kind)
            }
            3 | 8 => {
                let flag = r.byte()?;
                if flag > 1 {
                    return Err(DocumentBufferError::InvalidDocument);
                }
                let id = r.word()?;
                stack.push(tag);
                Op::Start(if tag == 3 {
                    ScopeStart::Group {
                        expanded: flag != 0,
                        id,
                    }
                } else {
                    ScopeStart::Conditional {
                        broken: flag != 0,
                        id,
                    }
                })
            }
            4 | 11 | 12 | 14 => {
                stack.push(tag);
                Op::Start(match tag {
                    4 => ScopeStart::Indent,
                    11 => ScopeStart::Fill,
                    12 => ScopeStart::Entry,
                    14 => ScopeStart::Suffix,
                    _ => unreachable!(),
                })
            }
            6 | 7 | 19 => {
                let value = r.word()?;
                if tag == 7 && value == 0 {
                    return Err(DocumentBufferError::InvalidDocument);
                }
                stack.push(tag);
                Op::Start(match tag {
                    6 => ScopeStart::Align(value),
                    7 => ScopeStart::IndentIfBreak(value),
                    19 => ScopeStart::AbsoluteIndent(value),
                    _ => unreachable!(),
                })
            }
            9 => Op::Ref(reference(&mut r, count)?),
            10 => {
                let n = r.word()? as usize;
                if n == 0 || n > (bytes.len() - r.at) / 4 {
                    return Err(DocumentBufferError::InvalidDocument);
                }
                let mut ids = Vec::with_capacity(n);
                for _ in 0..n {
                    ids.push(reference(&mut r, count)?);
                }
                Op::Choices(ids)
            }
            22 => {
                let flags = r.byte()?;
                if flags & !3 != 0 {
                    return Err(DocumentBufferError::InvalidDocument);
                }
                let id = r.word()?;
                let contents = reference(&mut r, count)?;
                let states = if flags & 2 != 0 {
                    let n = r.word()? as usize;
                    if n == 0 || n > (bytes.len() - r.at) / 4 {
                        return Err(DocumentBufferError::InvalidDocument);
                    }
                    let mut states = Vec::with_capacity(n);
                    for _ in 0..n {
                        states.push(reference(&mut r, count)?);
                    }
                    Some(states)
                } else {
                    None
                };
                Op::Group {
                    expanded: flags & 1 != 0,
                    id,
                    contents,
                    states,
                }
            }
            15 | 16 => Op::Simple(tag),
            17 => {
                let end = r.byte()?;
                let start = stack.pop().ok_or(DocumentBufferError::InvalidDocument)?;
                if start != end && !(end == 4 && matches!(start, 7 | 19)) {
                    return Err(DocumentBufferError::InvalidDocument);
                }
                Op::End(start)
            }
            21 => Op::Conditional(
                r.word()?,
                reference(&mut r, count)?,
                reference(&mut r, count)?,
            ),
            _ => return Err(DocumentBufferError::InvalidDocument),
        };
        ops.push(op);
    }
    if !stack.is_empty() {
        return Err(DocumentBufferError::InvalidDocument);
    }
    Ok(ops)
}

fn order(fragments: &[Vec<Op>]) -> Result<Vec<usize>> {
    let mut edges = Vec::with_capacity(fragments.len());
    for fragment in fragments {
        let mut children = Vec::new();
        for op in fragment {
            match op {
                Op::Ref(id) => children.push(*id),
                Op::Choices(ids) => children.extend_from_slice(ids),
                Op::Conditional(_, first, second) => children.extend([*first, *second]),
                Op::Group {
                    contents, states, ..
                } => {
                    children.push(*contents);
                    if let Some(states) = states {
                        children.extend_from_slice(states);
                    }
                }
                _ => {}
            }
        }
        edges.push(children);
    }
    let mut state = vec![0u8; fragments.len()];
    let mut result = Vec::with_capacity(fragments.len());
    for id in 0..fragments.len() {
        if state[id] != 0 {
            continue;
        }
        let mut stack = vec![(id, 0)];
        state[id] = 1;
        while let Some((id, next)) = stack.last_mut() {
            if *next == edges[*id].len() {
                state[*id] = 2;
                result.push(*id);
                stack.pop();
                continue;
            }
            let child = edges[*id][*next];
            *next += 1;
            match state[child] {
                0 => {
                    state[child] = 1;
                    stack.push((child, 0));
                }
                1 => return Err(DocumentBufferError::InvalidDocument),
                _ => {}
            }
        }
    }
    Ok(result)
}

struct Context<'a> {
    elements: ArenaVec<'a, FormatElement<'a>>,
    effect: BreakEffect,
    conditional_lines: bool,
    intrinsic_lines: bool,
    group_states: bool,
    requires_fill: bool,
    fill_depth: usize,
}

impl<'a> Context<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self {
            elements: ArenaVec::new_in(allocator),
            effect: BreakEffect::default(),
            conditional_lines: false,
            intrinsic_lines: false,
            group_states: false,
            requires_fill: false,
            fill_depth: 0,
        }
    }

    fn append_fragment(&mut self, fragment: Fragment<'a>, requires_fill: bool) {
        self.group_states |= fragment.group_states();
        self.effect = self.effect.join(fragment.effect());
        self.conditional_lines |= fragment.conditional_lines();
        self.intrinsic_lines |= fragment.has_intrinsic_lines();
        self.requires_fill |= requires_fill && self.fill_depth == 0;
        self.elements.push(FormatElement::fragment(fragment));
    }

    fn finish(self) -> (Fragment<'a>, bool) {
        let capabilities = if self.effect.preferred_breaks() || self.conditional_lines {
            FragmentCapabilities::from_flat(
                self.elements.iter().all(|element| element.permits_flat()),
            )
        } else {
            FragmentCapabilities::classify(&self.elements)
        };
        (
            Fragment::new(
                self.elements.into_bump_slice(),
                self.effect,
                capabilities,
                self.conditional_lines,
                self.intrinsic_lines,
                self.group_states,
            ),
            self.requires_fill,
        )
    }
}

enum ScopeFrame<'a> {
    Group {
        start: usize,
        outer_effect: BreakEffect,
    },
    Indent,
    Align {
        width: u32,
    },
    IndentIfBreak {
        start: usize,
    },
    Conditional {
        broken: bool,
        id: u32,
        outer: Context<'a>,
    },
    Fill {
        outer_depth: usize,
    },
    Entry {
        start: usize,
    },
    Suffix {
        start: usize,
        outer_depth: usize,
        outer_requires_fill: bool,
    },
    AbsoluteIndent,
}

impl ScopeFrame<'_> {
    fn tag(&self) -> u8 {
        match self {
            Self::Group { .. } => 3,
            Self::Indent => 4,
            Self::Align { .. } => 6,
            Self::IndentIfBreak { .. } => 7,
            Self::Conditional { .. } => 8,
            Self::Fill { .. } => 11,
            Self::Entry { .. } => 12,
            Self::Suffix { .. } => 14,
            Self::AbsoluteIndent => 19,
        }
    }
}

fn group(ids: &mut HashMap<u32, GroupId>, id: u32) -> Result<Option<GroupId>> {
    if id == 0 {
        return Ok(None);
    }
    if let Some(id) = ids.get(&id) {
        return Ok(Some(*id));
    }
    let next = u32::try_from(ids.len())
        .map_err(|_| DocumentBufferError::InvalidDocument)?
        .checked_add(1)
        .ok_or(DocumentBufferError::InvalidDocument)?;
    let mapped = GroupId(NonZeroU32::new(next).ok_or(DocumentBufferError::InvalidDocument)?);
    ids.insert(id, mapped);
    Ok(Some(mapped))
}

fn target(ids: &mut HashMap<u32, GroupId>, id: u32) -> Result<ConditionTarget> {
    Ok(match group(ids, id)? {
        Some(id) => ConditionTarget::Group(id),
        None => ConditionTarget::Current,
    })
}

fn offset(n: usize) -> Result<u32> {
    u32::try_from(n).map_err(|_| DocumentBufferError::InvalidDocument)
}

fn make_conditional<'a>(
    allocator: &'a Allocator,
    context: &mut Context<'a>,
    target: ConditionTarget,
    broken: (Fragment<'a>, bool),
    flat: (Fragment<'a>, bool),
) {
    context.group_states |= broken.0.group_states() || flat.0.group_states();
    context.effect = context.effect.join(broken.0.effect()).join(flat.0.effect());
    context.conditional_lines |= broken.0.conditional_lines() || flat.0.conditional_lines();
    context.intrinsic_lines |= broken.0.has_intrinsic_lines() || flat.0.has_intrinsic_lines();
    context.requires_fill |= (broken.1 || flat.1) && context.fill_depth == 0;
    let conditional = allocator.alloc(Conditional {
        target,
        broken: broken.0,
        flat: flat.0,
    });
    context
        .elements
        .push(FormatElement::Conditional(conditional));
}

fn build<'a>(
    allocator: &'a Allocator,
    ops: &[Op],
    pool: &'a [u8],
    built: &[Option<(Fragment<'a>, bool)>],
    ids: &mut HashMap<u32, GroupId>,
    texts: &mut HashMap<(u32, u32), Text<'a>>,
    text_budget: &mut usize,
) -> Result<(Fragment<'a>, bool)> {
    let mut context = Context::new(allocator);
    let mut frames: Vec<ScopeFrame<'a>> = Vec::new();
    let empty = Context::new(allocator).finish();
    for op in ops {
        match op {
            Op::Text(start, len) => {
                let key = (*start, *len);
                let text = if let Some(text) = texts.get(&key) {
                    *text
                } else {
                    let start = *start as usize;
                    let end = start
                        .checked_add(*len as usize)
                        .ok_or(DocumentBufferError::InvalidDocument)?;
                    let source_bytes = pool
                        .get(start..end)
                        .ok_or(DocumentBufferError::InvalidDocument)?;
                    // Each distinct range is measured once. Charge its byte length
                    // before UTF-8 validation or width computation.
                    *text_budget = text_budget
                        .checked_sub(*len as usize)
                        .ok_or(DocumentBufferError::LimitExceeded)?;
                    let source = std::str::from_utf8(source_bytes)
                        .map_err(|_| DocumentBufferError::InvalidDocument)?;
                    let text = Text::new(source);
                    texts.insert(key, text);
                    text
                };
                context.intrinsic_lines |= !text.is_single_line();
                context.elements.push(FormatElement::text(text));
            }
            Op::Line(kind) => {
                let element = match kind {
                    0 => FormatElement::Line,
                    1 => FormatElement::SoftLine,
                    2 => FormatElement::HardLine,
                    3 => FormatElement::HardLineWithoutBreakParent,
                    4 => FormatElement::LiteralLine,
                    5 => FormatElement::LiteralLineWithIndent,
                    6 => FormatElement::LiteralLineWithoutBreakParent,
                    _ => return Err(DocumentBufferError::InvalidDocument),
                };
                if matches!(kind, 2 | 4 | 5) {
                    context.effect = context.effect.join(BreakEffect::REQUIRED);
                }
                if matches!(kind, 2..=6) {
                    context.intrinsic_lines = true;
                }
                context.elements.push(element);
            }
            Op::Start(scope) => {
                let start = context.elements.len();
                let frame = match *scope {
                    ScopeStart::Group { expanded, id } => {
                        let outer_effect = context.effect;
                        context.effect = BreakEffect::default();
                        context.elements.push(FormatElement::StartGroup {
                            request: GroupRequest::from(expanded),
                            effect: context.effect,
                            id: group(ids, id)?,
                            end_offset: 0,
                        });
                        ScopeFrame::Group {
                            start,
                            outer_effect,
                        }
                    }
                    ScopeStart::Indent => {
                        context.elements.push(FormatElement::StartIndent);
                        ScopeFrame::Indent
                    }
                    ScopeStart::Align(width) => {
                        context.elements.push(FormatElement::StartAlign { width });
                        ScopeFrame::Align { width }
                    }
                    ScopeStart::IndentIfBreak(id) => {
                        context.elements.push(FormatElement::StartIndentIfBreak {
                            id: group(ids, id)?.ok_or(DocumentBufferError::InvalidDocument)?,
                            end_offset: 0,
                        });
                        ScopeFrame::IndentIfBreak { start }
                    }
                    ScopeStart::Conditional { broken, id } => {
                        let outer = std::mem::replace(&mut context, Context::new(allocator));
                        ScopeFrame::Conditional { broken, id, outer }
                    }
                    ScopeStart::Fill => {
                        let outer_depth = context.fill_depth;
                        context.fill_depth += 1;
                        context.elements.push(FormatElement::StartFill);
                        ScopeFrame::Fill { outer_depth }
                    }
                    ScopeStart::Entry => {
                        context.requires_fill |= context.fill_depth == 0;
                        context
                            .elements
                            .push(FormatElement::StartEntry { end_offset: 0 });
                        ScopeFrame::Entry { start }
                    }
                    ScopeStart::Suffix => {
                        let outer_depth = context.fill_depth;
                        let outer_requires_fill = context.requires_fill;
                        context.intrinsic_lines = true;
                        context.fill_depth = 0;
                        context.requires_fill = false;
                        context
                            .elements
                            .push(FormatElement::StartLineSuffix { end_offset: 0 });
                        ScopeFrame::Suffix {
                            start,
                            outer_depth,
                            outer_requires_fill,
                        }
                    }
                    ScopeStart::AbsoluteIndent(width) => {
                        context
                            .elements
                            .push(FormatElement::StartAbsoluteIndent { width });
                        ScopeFrame::AbsoluteIndent
                    }
                };
                frames.push(frame);
            }
            Op::End(tag) => {
                let frame = frames.pop().ok_or(DocumentBufferError::InvalidDocument)?;
                if frame.tag() != *tag {
                    return Err(DocumentBufferError::InvalidDocument);
                }
                match frame {
                    ScopeFrame::Group {
                        start,
                        outer_effect,
                    } => {
                        let distance = offset(context.elements.len() - start)?;
                        let FormatElement::StartGroup {
                            request,
                            effect: stored,
                            end_offset,
                            ..
                        } = &mut context.elements[start]
                        else {
                            return Err(DocumentBufferError::InvalidDocument);
                        };
                        let effect = BreakEffect::from_request(*request).join(context.effect);
                        *stored = effect;
                        *end_offset = distance;
                        context.effect = outer_effect.join(effect);
                        context.elements.push(FormatElement::EndGroup);
                    }
                    ScopeFrame::Indent | ScopeFrame::AbsoluteIndent => {
                        context.elements.push(FormatElement::EndIndent);
                    }
                    ScopeFrame::Align { width } => {
                        context.elements.push(FormatElement::EndAlign { width });
                    }
                    ScopeFrame::IndentIfBreak { start } => {
                        let distance = offset(context.elements.len() - start)?;
                        let FormatElement::StartIndentIfBreak { end_offset, .. } =
                            &mut context.elements[start]
                        else {
                            return Err(DocumentBufferError::InvalidDocument);
                        };
                        *end_offset = distance;
                        context.elements.push(FormatElement::EndIndent);
                    }
                    ScopeFrame::Conditional { broken, id, outer } => {
                        let fragment = context.finish();
                        context = outer;
                        let condition = target(ids, id)?;
                        let (broken, flat) = if broken {
                            (fragment, empty)
                        } else {
                            (empty, fragment)
                        };
                        make_conditional(allocator, &mut context, condition, broken, flat);
                    }
                    ScopeFrame::Fill { outer_depth } => {
                        context.fill_depth = outer_depth;
                        context.elements.push(FormatElement::EndFill);
                    }
                    ScopeFrame::Entry { start } => {
                        let distance = offset(context.elements.len() - start)?;
                        let FormatElement::StartEntry { end_offset } = &mut context.elements[start]
                        else {
                            return Err(DocumentBufferError::InvalidDocument);
                        };
                        *end_offset = distance;
                        context.elements.push(FormatElement::EndEntry);
                    }
                    ScopeFrame::Suffix {
                        start,
                        outer_depth,
                        outer_requires_fill,
                    } => {
                        if context.requires_fill {
                            return Err(DocumentBufferError::InvalidDocument);
                        }
                        context.requires_fill = outer_requires_fill;
                        context.fill_depth = outer_depth;
                        let distance = offset(context.elements.len() - start)?;
                        let FormatElement::StartLineSuffix { end_offset } =
                            &mut context.elements[start]
                        else {
                            return Err(DocumentBufferError::InvalidDocument);
                        };
                        *end_offset = distance;
                        context.elements.push(FormatElement::EndLineSuffix);
                    }
                }
            }
            Op::Ref(id) => {
                let (fragment, requires_fill) =
                    built[*id].ok_or(DocumentBufferError::InvalidDocument)?;
                context.append_fragment(fragment, requires_fill);
            }
            Op::Conditional(id, broken, flat) => {
                let broken = built[*broken].ok_or(DocumentBufferError::InvalidDocument)?;
                let flat = built[*flat].ok_or(DocumentBufferError::InvalidDocument)?;
                let condition = target(ids, *id)?;
                make_conditional(allocator, &mut context, condition, broken, flat);
            }
            Op::Choices(ids) => {
                let mut preferred = ArenaVec::new_in(allocator);
                for id in &ids[..ids.len() - 1] {
                    let (fragment, requires_fill) =
                        built[*id].ok_or(DocumentBufferError::InvalidDocument)?;
                    context.group_states |= fragment.group_states();
                    context.conditional_lines |= fragment.conditional_lines();
                    context.requires_fill |= requires_fill && context.fill_depth == 0;
                    preferred.push(fragment);
                }
                let (fallback, requires_fill) = built
                    [*ids.last().ok_or(DocumentBufferError::InvalidDocument)?]
                .ok_or(DocumentBufferError::InvalidDocument)?;
                context.group_states |= fallback.group_states();
                context.conditional_lines |= fallback.conditional_lines();
                context.requires_fill |= requires_fill && context.fill_depth == 0;
                if preferred.is_empty() {
                    preferred.push(fallback);
                }
                context.intrinsic_lines |= preferred[0].has_intrinsic_lines();
                let alternatives = allocator.alloc(Alternatives {
                    preferred: preferred.into_bump_slice(),
                    fallback,
                });
                context
                    .elements
                    .push(FormatElement::Alternatives(alternatives));
            }
            Op::Group {
                expanded,
                id,
                contents,
                states,
            } => {
                let (contents, requires_fill) =
                    built[*contents].ok_or(DocumentBufferError::InvalidDocument)?;
                let request = GroupRequest::from(*expanded);
                let id = group(ids, *id)?;
                context.requires_fill |= requires_fill && context.fill_depth == 0;
                context.group_states |= contents.group_states();
                context.conditional_lines |= contents.conditional_lines();
                context.intrinsic_lines |= contents.has_intrinsic_lines();
                if let Some(states) = states {
                    let mut fragments = ArenaVec::with_capacity_in(states.len(), allocator);
                    for state in states {
                        let (fragment, requires_fill) =
                            built[*state].ok_or(DocumentBufferError::InvalidDocument)?;
                        context.requires_fill |= requires_fill && context.fill_depth == 0;
                        context.conditional_lines |= fragment.conditional_lines();
                        fragments.push(fragment);
                    }
                    context.group_states = true;
                    context.effect = context.effect.join(BreakEffect::from_request(request));
                    let group = allocator.alloc(GroupWithStates {
                        contents,
                        states: fragments.into_bump_slice(),
                        request,
                        id,
                    });
                    context.elements.push(FormatElement::GroupWithStates(group));
                } else {
                    let effect = BreakEffect::from_request(request).join(contents.effect());
                    context.effect = context.effect.join(effect);
                    context.elements.push(FormatElement::StartGroup {
                        request,
                        effect,
                        id,
                        end_offset: 2,
                    });
                    context.elements.push(FormatElement::fragment(contents));
                    context.elements.push(FormatElement::EndGroup);
                }
            }
            Op::Simple(15) => context.elements.push(FormatElement::LineSuffixBoundary),
            Op::Simple(16) => {
                context.effect = context.effect.join(BreakEffect::REQUIRED);
                context.elements.push(FormatElement::BreakParent);
            }
            _ => return Err(DocumentBufferError::InvalidDocument),
        }
    }
    if !frames.is_empty() {
        return Err(DocumentBufferError::InvalidDocument);
    }
    Ok(context.finish())
}

pub(super) fn decode<'a>(allocator: &'a Allocator, bytes: &'a [u8]) -> Result<Doc<'a>> {
    if bytes.len() > u32::MAX as usize {
        return Err(DocumentBufferError::LimitExceeded);
    }
    let mut reader = Reader { bytes, at: 0 };
    if reader.take(4)? != b"SWCD" {
        return Err(DocumentBufferError::InvalidDocument);
    }
    let version = reader.word()?;
    if version != crate::DOCUMENT_FORMAT_VERSION {
        return Err(DocumentBufferError::UnsupportedVersion(version));
    }
    let count = reader.word()? as usize;
    let pool_len = reader.word()? as usize;
    if count == 0 || count > (bytes.len() - reader.at) / 4 {
        return Err(DocumentBufferError::InvalidDocument);
    }
    let mut fragments = Vec::with_capacity(count);
    for _ in 0..count {
        let len = reader.word()? as usize;
        fragments.push(parse_fragment(reader.take(len)?, count)?);
    }
    if pool_len != bytes.len() - reader.at {
        return Err(DocumentBufferError::InvalidDocument);
    }
    let pool = &bytes[reader.at..];
    let order = order(&fragments)?;
    let mut built = vec![None; count];
    let mut ids = HashMap::default();
    let mut texts = HashMap::default();
    let mut text_budget = bytes
        .len()
        .checked_mul(8)
        .ok_or(DocumentBufferError::LimitExceeded)?;
    for id in order {
        built[id] = Some(build(
            allocator,
            &fragments[id],
            pool,
            &built,
            &mut ids,
            &mut texts,
            &mut text_budget,
        )?);
    }
    let (root, requires_fill) = built[0].ok_or(DocumentBufferError::InvalidDocument)?;
    if requires_fill {
        return Err(DocumentBufferError::InvalidDocument);
    }
    let group_count = u32::try_from(ids.len()).map_err(|_| DocumentBufferError::InvalidDocument)?;
    Ok(Doc {
        allocator,
        elements: root.elements(),
        group_count,
        group_states: root.group_states(),
    })
}
