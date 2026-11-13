//! Compact, versioned document transport. No Rust pointers cross the bridge.
//!
//! Header: `SWCD`, version u32, fragment count u32, string-pool length u32.
//! Then (byte length u32, instructions) per fragment, followed by UTF-8 strings.
//! Integers are little endian. Scope ends identify their opening scope; all indent forms share one end tag.
use rustc_hash::FxHashMap;

use super::{ConditionTarget, Doc, Group, GroupId, Instruction};

pub(crate) fn encode<const PRETTIER: bool>(document: Doc<'_>) -> Vec<u8> {
    let mut encoder = Encoder {
        normalize_groups: document.root.has_source_groups(),
        ..Encoder::default()
    };
    encoder.reference(document.root.elements(), false);
    // Fragments form a DAG. A work list avoids recursion through shared scopes.
    let mut index = 0;
    while index < encoder.pending.len() {
        let (elements, flat, group_root) = encoder.pending[index];
        let instructions = encoder.instructions::<PRETTIER>(elements, flat, group_root);
        encoder.fragments.push(instructions);
        index += 1;
    }
    let mut output = b"SWCD".to_vec();
    word(&mut output, crate::DOCUMENT_FORMAT_VERSION as usize);
    word(&mut output, encoder.fragments.len());
    word(&mut output, encoder.strings.len());
    for fragment in encoder.fragments {
        word(&mut output, fragment.len());
        output.extend(fragment);
    }
    output.extend(encoder.strings);
    output
}

#[derive(Default)]
struct Encoder<'a> {
    fragments: Vec<Vec<u8>>,
    pending: Vec<(&'a [Instruction<'a>], bool, bool)>,
    normalize_groups: bool,
    fragment_ids: FxHashMap<(usize, usize, bool, bool), usize>,
    group_records: FxHashMap<usize, usize>,
    strings: Vec<u8>,
    string_ids: FxHashMap<&'a str, (usize, usize)>,
}

fn word(output: &mut Vec<u8>, value: usize) {
    let value = u32::try_from(value).expect("document transport exceeds u32 capacity");
    output.extend(value.to_le_bytes());
}

fn group(output: &mut Vec<u8>, id: Option<GroupId>) {
    word(output, id.map_or(0, |id| id.index() + 1));
}

impl<'a> Encoder<'a> {
    fn reference(&mut self, elements: &'a [Instruction<'a>], flat: bool) -> usize {
        self.reference_kind(elements, flat, false)
    }

    fn reference_kind(
        &mut self,
        elements: &'a [Instruction<'a>],
        flat: bool,
        group_root: bool,
    ) -> usize {
        let key = (elements.as_ptr() as usize, elements.len(), flat, group_root);
        let next_id = self.pending.len();
        *self.fragment_ids.entry(key).or_insert_with(|| {
            self.pending.push((elements, flat, group_root));
            next_id
        })
    }

    fn string(&mut self, text: &'a str) -> (usize, usize) {
        *self.string_ids.entry(text).or_insert_with(|| {
            let start = self.strings.len();
            self.strings.extend(text.as_bytes());
            (start, text.len())
        })
    }

    fn text(&mut self, output: &mut Vec<u8>, text: &'a str) {
        let (start, length) = self.string(text);
        output.push(0);
        word(output, start);
        word(output, length);
    }

    fn instructions<const PRETTIER: bool>(
        &mut self,
        elements: &'a [Instruction<'a>],
        flat: bool,
        group_root: bool,
    ) -> Vec<u8> {
        let mut output = Vec::new();
        let mut index = 0;
        while let Some(&element) = elements.get(index) {
            index += 1;
            if self.normalize_groups
                && !flat
                && !(group_root && index == 1)
                && let Instruction::StartGroup { end_offset, .. }
                | Instruction::StartGroupWithFlatWidth { end_offset, .. } = element
            {
                let end = index + end_offset as usize;
                output.push(9);
                word(
                    &mut output,
                    self.reference_kind(&elements[index - 1..end], false, true),
                );
                index = end;
                continue;
            }
            if self.normalize_groups
                && !flat
                && !(group_root && index == 1)
                && let Instruction::SourceGroup(group) = element
            {
                let key = group.key();
                let next = self.pending.len();
                let reference = *self.group_records.entry(key).or_insert_with(|| {
                    self.pending
                        .push((&elements[index - 1..index], false, true));
                    next
                });
                output.push(9);
                word(&mut output, reference);
                continue;
            }
            // A flat projection retains the named condition but projects only
            // this complete indent body. Its scope end is consumed here.
            if let Instruction::StartIndentIfBreak {
                target,
                negate,
                end_offset,
            } = element
            {
                if flat && matches!(target, ConditionTarget::Current) {
                    continue;
                }
                let end = index
                    .checked_add(end_offset as usize)
                    .expect("document scope offset exceeds usize");
                output.extend([7, u8::from(negate)]);
                group(
                    &mut output,
                    match target {
                        ConditionTarget::Current => None,
                        ConditionTarget::Group(id) => Some(id),
                    },
                );
                output.push(9);
                word(&mut output, self.reference(&elements[index..end - 1], flat));
                output.extend([17, 4]);
                index = end;
                continue;
            }
            let element = if flat {
                let Some(element) = element.flattened() else {
                    continue;
                };
                element
            } else {
                element
            };
            match element {
                Instruction::Text { source, .. } => self.text(&mut output, source.as_str()),
                Instruction::NormalLine => output.extend([2, 0]),
                Instruction::SoftLine => output.extend([2, 1]),
                Instruction::NormalLiteralLine => output.extend([2, 7]),
                Instruction::SoftLiteralLine => output.extend([2, 8]),
                Instruction::HardLine => output.extend([2, 2]),
                Instruction::HardLineOnly => output.extend([2, 3]),
                Instruction::LiteralLine => output.extend([2, 4]),
                Instruction::LiteralLineOnly => output.extend([2, 6]),
                Instruction::LiteralLineAtCurrentIndent => output.extend([2, 5]),
                Instruction::Trim => output.push(27),
                Instruction::Cursor => output.push(28),
                Instruction::StartLabel(id) => {
                    output.push(29);
                    word(&mut output, id.0 as usize);
                }
                Instruction::EndLabel => output.extend([17, 29]),
                Instruction::SoftLineIfGroupBreaks(id) => {
                    output.extend([8, 1]);
                    group(&mut output, Some(id));
                    output.extend([2, 1, 17, 8]);
                }
                Instruction::TextIfBreak { source, .. }
                | Instruction::TextIfFlat { source, .. } => {
                    output.extend([
                        8,
                        u8::from(matches!(element, Instruction::TextIfBreak { .. })),
                    ]);
                    group(&mut output, None);
                    self.text(&mut output, source.as_str());
                    output.extend([17, 8]);
                }
                Instruction::IfBreak(conditional) | Instruction::FlatIfBreak(conditional) => {
                    output.push(21);
                    group(
                        &mut output,
                        match conditional.target {
                            ConditionTarget::Current => None,
                            ConditionTarget::Group(id) => Some(id),
                        },
                    );
                    let flat = matches!(element, Instruction::FlatIfBreak(_));
                    word(
                        &mut output,
                        self.reference(conditional.broken.elements(), flat),
                    );
                    word(
                        &mut output,
                        self.reference(conditional.flat.elements(), flat),
                    );
                }
                Instruction::LineSuffixBoundary => output.push(15),
                Instruction::StartLineSuffix { .. } => output.push(14),
                Instruction::EndLineSuffix => output.extend([17, 14]),
                Instruction::StartGroup { request, id, .. } => {
                    output.extend([
                        3,
                        u8::from(matches!(request, super::GroupRequest::Expanded)),
                    ]);
                    group(&mut output, id);
                }
                Instruction::StartGroupWithFlatWidth { .. } => {
                    output.extend([3, 0]);
                    group(&mut output, None);
                }
                Instruction::EndGroup => output.extend([17, 3]),
                Instruction::StartIndent => output.push(4),
                Instruction::StartAbsoluteIndent { width } => {
                    output.push(19);
                    word(&mut output, width as usize);
                }
                Instruction::StartIndentIfBreak { .. } => {
                    unreachable!("named indent scopes are encoded before projection")
                }
                Instruction::EndIndent => output.extend([17, 4]),
                Instruction::StartAlignText(prefix) => {
                    let (start, length) = self.string(prefix.source);
                    output.push(23);
                    word(&mut output, start);
                    word(&mut output, length);
                }
                Instruction::StartDedent => output.push(24),
                Instruction::StartMarkRoot => output.push(25),
                Instruction::StartDedentToRoot => output.push(26),
                Instruction::EndRichIndent(kind) => {
                    output.extend([17, super::wire::rich_indent_tag(kind)])
                }
                Instruction::StartAlign { width } => {
                    output.push(6);
                    word(&mut output, width as usize);
                }
                Instruction::EndAlign { .. } => output.extend([17, 6]),
                Instruction::StartFill => output.push(11),
                Instruction::EndFill => output.extend([17, 11]),
                Instruction::StartFillPart { .. } => output.push(12),
                Instruction::EndFillPart => output.extend([17, 12]),
                Instruction::Fragment { elements, .. } => {
                    output.push(9);
                    word(&mut output, self.reference(elements.get(), false));
                }
                Instruction::FlatFragment { elements, .. } => {
                    output.push(9);
                    word(&mut output, self.reference(elements.get(), true));
                }
                Instruction::CompactGroup(data) => {
                    let group = data;
                    output.push(10);
                    word(&mut output, group.stored_states().len() + 1);
                    for fragment in group.stored_states() {
                        word(&mut output, self.reference(fragment.elements(), false));
                    }
                    word(
                        &mut output,
                        self.reference(group.fallback().elements(), false),
                    );
                }
                Instruction::SourceGroup(data) => {
                    let group_data = data;
                    output.extend([
                        22,
                        2 | u8::from(matches!(
                            group_data.request(),
                            super::GroupRequest::Expanded
                        )),
                    ]);
                    group(&mut output, group_data.id());
                    word(
                        &mut output,
                        self.reference(group_data.primary().elements(), false),
                    );
                    if PRETTIER {
                        // Prettier's propagation visits expandedStates instead
                        // of contents. Put the primary first so every executable
                        // layout is visible, including independently rewritten contents.
                        let layouts = group_data.layouts();
                        word(&mut output, layouts.alternatives.len() + 2);
                        for state in layouts.all() {
                            word(&mut output, self.reference(state.elements(), false));
                        }
                    } else {
                        word(&mut output, group_data.stored_states().len());
                        for state in group_data.stored_states() {
                            word(&mut output, self.reference(state.elements(), false));
                        }
                    }
                }
                Instruction::BreakParent => output.push(16),
            }
        }
        output
    }
}
