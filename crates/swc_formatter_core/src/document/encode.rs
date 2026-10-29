//! Compact, versioned document transport. No Rust pointers cross the bridge.
//!
//! Header: `SWCD`, version u32, fragment count u32, string-pool length u32.
//! Then (byte length u32, instructions) per fragment, followed by UTF-8 strings.
//! Integers are little endian. Scope ends identify their opening scope; all indent forms share one end tag.
use rustc_hash::FxHashMap;

use super::{ConditionTarget, Doc, FormatElement, GroupId};

pub(crate) fn encode(document: Doc<'_>) -> Vec<u8> {
    let mut encoder = Encoder::default();
    encoder.reference(document.elements, false);
    // Fragments form a DAG. A work list avoids recursion through shared scopes.
    let mut index = 0;
    while index < encoder.pending.len() {
        let (elements, flat) = encoder.pending[index];
        let instructions = encoder.instructions(elements, flat);
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
    pending: Vec<(&'a [FormatElement<'a>], bool)>,
    fragment_ids: FxHashMap<(usize, usize, bool), usize>,
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
    fn reference(&mut self, elements: &'a [FormatElement<'a>], flat: bool) -> usize {
        let key = (elements.as_ptr() as usize, elements.len(), flat);
        let next_id = self.pending.len();
        *self.fragment_ids.entry(key).or_insert_with(|| {
            self.pending.push((elements, flat));
            next_id
        })
    }

    fn text(&mut self, output: &mut Vec<u8>, text: &'a str) {
        let (start, length) = *self.string_ids.entry(text).or_insert_with(|| {
            let start = self.strings.len();
            self.strings.extend(text.as_bytes());
            (start, text.len())
        });
        output.push(0);
        word(output, start);
        word(output, length);
    }

    fn instructions(&mut self, elements: &'a [FormatElement<'a>], flat: bool) -> Vec<u8> {
        let mut output = Vec::new();
        let mut index = 0;
        while let Some(&element) = elements.get(index) {
            index += 1;
            // A flat projection retains the named condition but projects only
            // this complete indent body. Its scope end is consumed here.
            if let FormatElement::StartIndentIfBreak { id, end_offset } = element {
                let end = index
                    .checked_add(end_offset as usize)
                    .expect("document scope offset exceeds usize");
                output.push(7);
                group(&mut output, Some(id));
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
                FormatElement::Text { source, .. } => self.text(&mut output, source.as_str()),
                FormatElement::Line => output.extend([2, 0]),
                FormatElement::SoftLine => output.extend([2, 1]),
                FormatElement::HardLine => output.extend([2, 2]),
                FormatElement::HardLineWithoutBreakParent => output.extend([2, 3]),
                FormatElement::LiteralLine => output.extend([2, 4]),
                FormatElement::LiteralLineWithoutBreakParent => output.extend([2, 6]),
                FormatElement::LiteralLineWithIndent => output.extend([2, 5]),
                FormatElement::SoftLineIfBreak(id) => {
                    output.extend([8, 1]);
                    group(&mut output, Some(id));
                    output.extend([2, 1, 17, 8]);
                }
                FormatElement::IfBreak { source, .. } | FormatElement::IfFlat { source, .. } => {
                    output.extend([
                        8,
                        u8::from(matches!(element, FormatElement::IfBreak { .. })),
                    ]);
                    group(&mut output, None);
                    self.text(&mut output, source.as_str());
                    output.extend([17, 8]);
                }
                FormatElement::Conditional(conditional)
                | FormatElement::FlatConditional(conditional) => {
                    output.push(21);
                    group(
                        &mut output,
                        match conditional.target {
                            ConditionTarget::Current => None,
                            ConditionTarget::Group(id) => Some(id),
                        },
                    );
                    let flat = matches!(element, FormatElement::FlatConditional(_));
                    word(
                        &mut output,
                        self.reference(conditional.broken.elements(), flat),
                    );
                    word(
                        &mut output,
                        self.reference(conditional.flat.elements(), flat),
                    );
                }
                FormatElement::LineSuffixBoundary => output.push(15),
                FormatElement::StartLineSuffix { .. } => output.push(14),
                FormatElement::EndLineSuffix => output.extend([17, 14]),
                FormatElement::StartGroup { request, id, .. } => {
                    output.extend([
                        3,
                        u8::from(matches!(request, super::GroupRequest::Expanded)),
                    ]);
                    group(&mut output, id);
                }
                FormatElement::StartGroupWithFlatWidth { .. } => {
                    output.extend([3, 0]);
                    group(&mut output, None);
                }
                FormatElement::EndGroup => output.extend([17, 3]),
                FormatElement::StartIndent => output.push(4),
                FormatElement::StartAbsoluteIndent { width } => {
                    output.push(19);
                    word(&mut output, width as usize);
                }
                FormatElement::StartIndentIfBreak { .. } => {
                    unreachable!("named indent scopes are encoded before projection")
                }
                FormatElement::EndIndent => output.extend([17, 4]),
                FormatElement::StartAlign { width } => {
                    output.push(6);
                    word(&mut output, width as usize);
                }
                FormatElement::EndAlign { .. } => output.extend([17, 6]),
                FormatElement::StartFill => output.push(11),
                FormatElement::EndFill => output.extend([17, 11]),
                FormatElement::StartEntry { .. } => output.push(12),
                FormatElement::EndEntry => output.extend([17, 12]),
                FormatElement::Fragment { elements, .. } => {
                    output.push(9);
                    word(&mut output, self.reference(elements.get(), false));
                }
                FormatElement::FlatFragment { elements, .. } => {
                    output.push(9);
                    word(&mut output, self.reference(elements.get(), true));
                }
                FormatElement::Alternatives(alternatives) => {
                    output.push(10);
                    word(&mut output, alternatives.preferred.len() + 1);
                    for fragment in alternatives.preferred {
                        word(&mut output, self.reference(fragment.elements(), false));
                    }
                    word(
                        &mut output,
                        self.reference(alternatives.fallback.elements(), false),
                    );
                }
                FormatElement::BreakParent => output.push(16),
            }
        }
        output
    }
}
