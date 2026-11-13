//! Checked decoding of the versioned document transport.
use std::{collections::HashMap, fmt, num::NonZeroU32};
use swc_next_allocator::{Allocator, vec::ArenaVec};

use super::{
    AlignText, BreakEffect, ConditionTarget, Doc, Fragment, Group, GroupId, GroupRequest, IfBreak,
    Instruction, LabelId, RichIndentKind, facts::Facts, text::Text, wire,
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
    End,
    Ref(usize),
    Choices(Vec<usize>),
    Group {
        expanded: bool,
        id: u32,
        contents: usize,
        states: Option<Vec<usize>>,
    },
    IfBreak(u32, usize, usize),
    Simple(u8),
}

#[derive(Debug)]
enum ScopeStart {
    Group { expanded: bool, id: u32 },
    Indent,
    Align(u32),
    IndentIfBreak { id: u32, negate: bool },
    IfBreak { broken: bool, id: u32 },
    Fill,
    FillPart,
    Suffix,
    AbsoluteIndent(u32),
    AlignText(u32, u32),
    RichIndent(RichIndentKind),
    Label(LabelId),
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
                if !matches!(kind, 0..=8) {
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
                    ScopeStart::IfBreak {
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
                    12 => ScopeStart::FillPart,
                    14 => ScopeStart::Suffix,
                    _ => unreachable!(),
                })
            }
            7 => {
                let flags = r.byte()?;
                if flags & !1 != 0 {
                    return Err(DocumentBufferError::InvalidDocument);
                }
                let id = r.word()?;
                stack.push(tag);
                Op::Start(ScopeStart::IndentIfBreak {
                    id,
                    negate: flags != 0,
                })
            }
            6 | 19 => {
                let value = r.word()?;
                stack.push(tag);
                Op::Start(match tag {
                    6 => ScopeStart::Align(value),
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
            23 => {
                let start = r.word()?;
                let len = r.word()?;
                stack.push(tag);
                Op::Start(ScopeStart::AlignText(start, len))
            }
            24..=26 => {
                stack.push(tag);
                Op::Start(ScopeStart::RichIndent(
                    wire::rich_indent_kind(tag).ok_or(DocumentBufferError::InvalidDocument)?,
                ))
            }
            15 | 16 | 27 | 28 => Op::Simple(tag),
            29 => {
                stack.push(tag);
                Op::Start(ScopeStart::Label(LabelId(r.word()?)))
            }
            17 => {
                let end = r.byte()?;
                let start = stack.pop().ok_or(DocumentBufferError::InvalidDocument)?;
                if start != end && !(end == 4 && matches!(start, 7 | 19)) {
                    return Err(DocumentBufferError::InvalidDocument);
                }
                Op::End
            }
            21 => Op::IfBreak(
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
                Op::IfBreak(_, first, second) => children.extend([*first, *second]),
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

#[derive(Clone, Copy, Default)]
enum FillPosition {
    // The common valid-owner case uses the zero discriminant.
    Body,
    #[default]
    Outside,
    Part,
}

struct Context<'a> {
    elements: ArenaVec<'a, Instruction<'a>>,
    facts: Facts,
    requires_fill: bool,
    fill: FillPosition,
}

impl<'a> Context<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self {
            elements: ArenaVec::new_in(allocator),
            facts: Facts::default(),
            requires_fill: false,
            fill: FillPosition::Outside,
        }
    }

    #[inline(always)]
    fn push(&mut self, element: Instruction<'a>) {
        self.facts.observe(element);
        self.elements.push(element);
    }

    // Balanced child fragments can require an enclosing Fill body, but cannot
    // borrow an owner whose current part is already open.
    fn require_fill_body(&mut self, required: bool) -> Result<()> {
        if required {
            match self.fill {
                FillPosition::Outside => self.requires_fill = true,
                FillPosition::Body => {}
                FillPosition::Part => return Err(DocumentBufferError::InvalidDocument),
            }
        }
        Ok(())
    }

    fn append_fragment(&mut self, fragment: Fragment<'a>, requires_fill: bool) -> Result<()> {
        self.require_fill_body(requires_fill)?;
        self.push(Instruction::fragment(fragment));
        Ok(())
    }

    fn finish(self) -> (Fragment<'a>, bool) {
        let summary = self.facts.finish(&self.elements);
        (
            Fragment::new(self.elements.into_bump_slice(), summary),
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
    IfBreak {
        broken: bool,
        id: u32,
        outer: Context<'a>,
    },
    Fill {
        outer: FillPosition,
    },
    FillPart {
        start: usize,
        outer: FillPosition,
    },
    Suffix {
        start: usize,
        outer_fill: FillPosition,
        outer_requires_fill: bool,
    },
    AbsoluteIndent,
    RichIndent(RichIndentKind),
    Label,
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
) -> Result<()> {
    context.require_fill_body(broken.1 || flat.1)?;
    let conditional = allocator.alloc(IfBreak {
        target,
        broken: broken.0,
        flat: flat.0,
    });
    context.push(Instruction::IfBreak(conditional));
    Ok(())
}

struct StringCache<'a> {
    texts: HashMap<(u32, u32), Text<'a>>,
    prefixes: HashMap<(u32, u32), &'a AlignText<'a>>,
    budget: usize,
}

fn build<'a>(
    allocator: &'a Allocator,
    ops: &[Op],
    pool: &'a [u8],
    built: &[Option<(Fragment<'a>, bool)>],
    ids: &mut HashMap<u32, GroupId>,
    strings: &mut StringCache<'a>,
) -> Result<(Fragment<'a>, bool)> {
    let StringCache {
        texts,
        prefixes,
        budget: text_budget,
    } = strings;
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
                context.push(Instruction::text(text));
            }
            Op::Line(kind) => {
                let element = match kind {
                    0 => Instruction::NormalLine,
                    1 => Instruction::SoftLine,
                    2 => Instruction::HardLine,
                    3 => Instruction::HardLineOnly,
                    4 => Instruction::LiteralLine,
                    5 => Instruction::LiteralLineAtCurrentIndent,
                    6 => Instruction::LiteralLineOnly,
                    7 => Instruction::NormalLiteralLine,
                    8 => Instruction::SoftLiteralLine,
                    _ => return Err(DocumentBufferError::InvalidDocument),
                };
                context.push(element);
            }
            Op::Start(scope) => {
                let start = context.elements.len();
                let frame = match *scope {
                    ScopeStart::Label(id) => {
                        context.push(Instruction::StartLabel(id));
                        ScopeFrame::Label
                    }
                    ScopeStart::Group { expanded, id } => {
                        let outer_effect = context.facts.begin_group();
                        context.push(Instruction::StartGroup {
                            request: GroupRequest::from(expanded),
                            effect: context.facts.effect(),
                            id: group(ids, id)?,
                            end_offset: 0,
                        });
                        ScopeFrame::Group {
                            start,
                            outer_effect,
                        }
                    }
                    ScopeStart::Indent => {
                        context.push(Instruction::StartIndent);
                        ScopeFrame::Indent
                    }
                    ScopeStart::Align(width) => {
                        if width != 0 {
                            context.push(Instruction::StartAlign { width });
                        }
                        ScopeFrame::Align { width }
                    }
                    ScopeStart::IndentIfBreak { id, negate } => {
                        context.push(Instruction::StartIndentIfBreak {
                            target: target(ids, id)?,
                            negate,
                            end_offset: 0,
                        });
                        ScopeFrame::IndentIfBreak { start }
                    }
                    ScopeStart::IfBreak { broken, id } => {
                        let outer = std::mem::replace(&mut context, Context::new(allocator));
                        ScopeFrame::IfBreak { broken, id, outer }
                    }
                    ScopeStart::Fill => {
                        let outer = std::mem::replace(&mut context.fill, FillPosition::Body);
                        context.push(Instruction::StartFill);
                        ScopeFrame::Fill { outer }
                    }
                    ScopeStart::FillPart => {
                        context.require_fill_body(true)?;
                        let outer = std::mem::replace(&mut context.fill, FillPosition::Part);
                        context.push(Instruction::StartFillPart { end_offset: 0 });
                        ScopeFrame::FillPart { start, outer }
                    }
                    ScopeStart::Suffix => {
                        let outer_fill =
                            std::mem::replace(&mut context.fill, FillPosition::Outside);
                        let outer_requires_fill = context.requires_fill;
                        context.requires_fill = false;
                        context.push(Instruction::StartLineSuffix { end_offset: 0 });
                        ScopeFrame::Suffix {
                            start,
                            outer_fill,
                            outer_requires_fill,
                        }
                    }
                    ScopeStart::AlignText(start, len) => {
                        let key = (start, len);
                        let text = if let Some(text) = prefixes.get(&key) {
                            *text
                        } else {
                            let end = (start as usize)
                                .checked_add(len as usize)
                                .ok_or(DocumentBufferError::InvalidDocument)?;
                            let bytes = pool
                                .get(start as usize..end)
                                .ok_or(DocumentBufferError::InvalidDocument)?;
                            *text_budget = text_budget
                                .checked_sub(len as usize)
                                .ok_or(DocumentBufferError::LimitExceeded)?;
                            let source = std::str::from_utf8(bytes)
                                .map_err(|_| DocumentBufferError::InvalidDocument)?;
                            let text = &*allocator.alloc(AlignText {
                                source,
                                width: crate::measure_text_width(source),
                            });
                            prefixes.insert(key, text);
                            text
                        };
                        context.push(Instruction::StartAlignText(text));
                        ScopeFrame::RichIndent(RichIndentKind::AlignText)
                    }
                    ScopeStart::RichIndent(kind) => {
                        context.push(match kind {
                            RichIndentKind::Dedent => Instruction::StartDedent,
                            RichIndentKind::MarkRoot => Instruction::StartMarkRoot,
                            RichIndentKind::DedentToRoot => Instruction::StartDedentToRoot,
                            RichIndentKind::AlignText => {
                                return Err(DocumentBufferError::InvalidDocument);
                            }
                        });
                        ScopeFrame::RichIndent(kind)
                    }
                    ScopeStart::AbsoluteIndent(width) => {
                        context.push(Instruction::StartAbsoluteIndent { width });
                        ScopeFrame::AbsoluteIndent
                    }
                };
                frames.push(frame);
            }
            Op::End => {
                // Parsing has already checked matching wire scope tags.
                // Construction only needs the semantic frame it is closing.
                let frame = frames.pop().ok_or(DocumentBufferError::InvalidDocument)?;
                match frame {
                    ScopeFrame::Label => context.push(Instruction::EndLabel),
                    ScopeFrame::Group {
                        start,
                        outer_effect,
                    } => {
                        let distance = offset(context.elements.len() - start)?;
                        let Instruction::StartGroup {
                            request,
                            effect: stored,
                            end_offset,
                            ..
                        } = &mut context.elements[start]
                        else {
                            return Err(DocumentBufferError::InvalidDocument);
                        };
                        let effect = context.facts.end_group(*request, outer_effect);
                        *stored = effect;
                        *end_offset = distance;
                        context.push(Instruction::EndGroup);
                    }
                    ScopeFrame::RichIndent(kind) => {
                        context.push(Instruction::EndRichIndent(kind));
                    }
                    ScopeFrame::Indent | ScopeFrame::AbsoluteIndent => {
                        context.push(Instruction::EndIndent);
                    }
                    ScopeFrame::Align { width } => {
                        if width != 0 {
                            context.push(Instruction::EndAlign { width });
                        }
                    }
                    ScopeFrame::IndentIfBreak { start } => {
                        let distance = offset(context.elements.len() - start)?;
                        let Instruction::StartIndentIfBreak { end_offset, .. } =
                            &mut context.elements[start]
                        else {
                            return Err(DocumentBufferError::InvalidDocument);
                        };
                        *end_offset = distance;
                        context.push(Instruction::EndIndent);
                    }
                    ScopeFrame::IfBreak { broken, id, outer } => {
                        let fragment = context.finish();
                        context = outer;
                        let condition = target(ids, id)?;
                        let (broken, flat) = if broken {
                            (fragment, empty)
                        } else {
                            (empty, fragment)
                        };
                        make_conditional(allocator, &mut context, condition, broken, flat)?;
                    }
                    ScopeFrame::Fill { outer } => {
                        context.fill = outer;
                        context.push(Instruction::EndFill);
                    }
                    ScopeFrame::FillPart { start, outer } => {
                        context.fill = outer;
                        let distance = offset(context.elements.len() - start)?;
                        let Instruction::StartFillPart { end_offset } =
                            &mut context.elements[start]
                        else {
                            return Err(DocumentBufferError::InvalidDocument);
                        };
                        *end_offset = distance;
                        context.push(Instruction::EndFillPart);
                    }
                    ScopeFrame::Suffix {
                        start,
                        outer_fill,
                        outer_requires_fill,
                    } => {
                        if context.requires_fill {
                            return Err(DocumentBufferError::InvalidDocument);
                        }
                        context.requires_fill = outer_requires_fill;
                        context.fill = outer_fill;
                        let distance = offset(context.elements.len() - start)?;
                        let Instruction::StartLineSuffix { end_offset } =
                            &mut context.elements[start]
                        else {
                            return Err(DocumentBufferError::InvalidDocument);
                        };
                        *end_offset = distance;
                        context.push(Instruction::EndLineSuffix);
                    }
                }
            }
            Op::Ref(id) => {
                let (fragment, requires_fill) =
                    built[*id].ok_or(DocumentBufferError::InvalidDocument)?;
                context.append_fragment(fragment, requires_fill)?;
            }
            Op::IfBreak(id, broken, flat) => {
                let broken = built[*broken].ok_or(DocumentBufferError::InvalidDocument)?;
                let flat = built[*flat].ok_or(DocumentBufferError::InvalidDocument)?;
                let condition = target(ids, *id)?;
                make_conditional(allocator, &mut context, condition, broken, flat)?;
            }
            Op::Choices(ids) => {
                let mut preferred = ArenaVec::new_in(allocator);
                for id in &ids[..ids.len() - 1] {
                    let (fragment, requires_fill) =
                        built[*id].ok_or(DocumentBufferError::InvalidDocument)?;
                    context.require_fill_body(requires_fill)?;
                    preferred.push(fragment);
                }
                let (fallback, requires_fill) = built
                    [*ids.last().ok_or(DocumentBufferError::InvalidDocument)?]
                .ok_or(DocumentBufferError::InvalidDocument)?;
                context.require_fill_body(requires_fill)?;
                if preferred.is_empty() {
                    preferred.push(fallback);
                }
                let group = super::group::compact(allocator, preferred.into_bump_slice(), fallback);
                context.push(group.element());
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
                context.require_fill_body(requires_fill)?;
                if let Some(states) = states {
                    let mut fragments = ArenaVec::with_capacity_in(states.len(), allocator);
                    for state in states {
                        let (fragment, requires_fill) =
                            built[*state].ok_or(DocumentBufferError::InvalidDocument)?;
                        context.require_fill_body(requires_fill)?;
                        fragments.push(fragment);
                    }
                    let group = super::group::with_states(
                        allocator,
                        request,
                        id,
                        contents,
                        fragments.into_bump_slice(),
                    );
                    context.push(group.element());
                } else {
                    let effect = BreakEffect::from_request(request).join(contents.effect());
                    let outer = context.facts.begin_group();
                    context.push(Instruction::StartGroup {
                        request,
                        effect,
                        id,
                        end_offset: 2,
                    });
                    context.push(Instruction::fragment(contents));
                    context.push(Instruction::EndGroup);
                    context.facts.end_group(request, outer);
                }
            }
            Op::Simple(15) => context.push(Instruction::LineSuffixBoundary),
            Op::Simple(27) => context.push(Instruction::Trim),
            Op::Simple(28) => context.push(Instruction::Cursor),
            Op::Simple(16) => {
                context.push(Instruction::BreakParent);
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
    let mut strings = StringCache {
        texts: HashMap::default(),
        prefixes: HashMap::default(),
        budget: bytes
            .len()
            .checked_mul(8)
            .ok_or(DocumentBufferError::LimitExceeded)?,
    };
    for id in order {
        built[id] = Some(build(
            allocator,
            &fragments[id],
            pool,
            &built,
            &mut ids,
            &mut strings,
        )?);
    }
    let (root, requires_fill) = built[0].ok_or(DocumentBufferError::InvalidDocument)?;
    if requires_fill {
        return Err(DocumentBufferError::InvalidDocument);
    }
    let group_count = u32::try_from(ids.len()).map_err(|_| DocumentBufferError::InvalidDocument)?;
    Ok(Doc {
        allocator,
        root,
        group_count,
    })
}
