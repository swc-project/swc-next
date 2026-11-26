//! TypeScript comparison grouping from emitted syntax and balanced delimiters.
//!
//! A complete parenthesized value comparison is not a well-formed type. One
//! selected group therefore also rejects earlier trials in the same frame.
//! A close inside physical delimiters cannot close an outer type-argument
//! trial without structural recovery. Such recovery bugs are not a printing
//! requirement; independently valid TS must retain its value structure.
//!
//! Function and class expressions are separate islands. TypeScript 6 can recover
//! `function(...)` as a JSDoc type and commit with diagnostics, even in valid TS
//! expressions. Go rejects that interpretation; we do not reproduce this TS6 bug.
//! Explicit type arguments and type parameters are balanced type islands.
//! Their closing `>` cannot close an earlier speculative list.
//! A runtime `>>`/`>>>` can close the outer trial only after nested type lists
//! consumed its leading angles. A running upper bound on emitted `<` openings
//! rules that out when there are fewer openings than the operator's width.
//! Tracing starts at a comparison and stops once its delimiter/rejection region
//! has no pending candidate. Unrelated tokens do not allocate trace frames.
//!
//! Grammar summaries describe the emitted value/type interpretation. Parser
//! recovery bugs are not reproduced with compatibility groups.

mod follower;
mod value_syntax;

use swc_next_ecma_ast::Ast;

use self::follower::{needs_trace, printed_follower_can_commit};
use super::{GenerateResult, Options, Printer, Typed};

// Render once, select comparison groups in reverse delimiter order, then
// insert them into the existing output and replay adjusted source mappings.
pub(super) fn generate<'a>(ast: &'a Ast<'a>, options: Options) -> GenerateResult<'a> {
    if !needs_trace(ast, &options) {
        return Printer::<Typed>::init(ast, options).generate();
    }
    let mut printer = Printer::<State<'a>>::init(ast, options);
    printer.print_program(ast.root_program());
    if !printer.trace.has_records() {
        return printer.finish();
    }
    let mut state = std::mem::take(&mut printer.trace);
    state.complete();
    let state = state.plan(&printer.js);
    let with_map = printer.options.source_map_path.is_some();
    if state.has_groups {
        state.apply(&mut printer.js, &mut printer.builder, with_map);
    } else if with_map {
        state.replay_mappings(&printer.js, &mut printer.builder);
    }
    printer.finish()
}

#[derive(Clone, Copy)]
pub(super) struct Shape {
    pub wrapped: bool,
    pub rejects: bool,
}

#[derive(Clone, Copy)]
struct Fragment {
    start: usize,
    end: usize,
    begin: usize,
    angle: usize, // One-based opening ordinal.
    shape: Shape,
    left_grouped: bool,
    no_close: bool,
    external: bool,
    group: bool,
}

#[derive(Clone, Copy)]
enum Event {
    Begin(usize),
    End(usize),
    Operator(usize),
    Mapping(usize),
    Close(usize),
    TypeBreak,
    ValueStart,
    ValueEnd,
    TypeStart,
    TypeEnd,
    Syntax { byte: u8 },
}

struct Mapping<'a> {
    offset: usize,
    original: u32,
    name: &'a [u8],
}

#[derive(Clone, Copy)]
struct Close {
    offset: usize,
    eligible: usize, // Latest opening with enough angles before this close.
}

struct Prefix {
    pending: bool,
    byte: u8,
    angles: usize,
    offset: usize,
}

#[derive(Default)]
pub(super) struct State<'a> {
    events: Vec<Event>,
    fragments: Vec<Fragment>,
    binary_fragments: Vec<usize>,
    mappings: Vec<Mapping<'a>>,
    closes: Vec<Close>,
    prefixes: Vec<Prefix>,
    scopes: Vec<(usize, usize)>,
    active: bool,
    depth: usize,
    pending: usize,
    suspended: usize,
    open_fragments: usize,
    last_close: Option<usize>,
    last_group: Option<(usize, usize)>,
    angles: usize,
    has_groups: bool,
}

impl<'a> State<'a> {
    #[inline]
    pub(super) fn is_active(&self) -> bool {
        self.active
    }

    #[inline]
    pub(super) fn has_pending_here(&self) -> bool {
        self.active && self.prefixes.last().is_some_and(|prefix| prefix.pending)
    }

    #[inline]
    pub(super) fn has_records(&self) -> bool {
        !self.fragments.is_empty()
    }

    pub(super) fn begin_binary(&mut self, shape: Shape, start: usize) {
        let id = self.begin(shape, start);
        self.binary_fragments.push(id);
    }

    pub(super) fn end_binary(&mut self, offset: usize) {
        let id = self.binary_fragments.pop().unwrap();
        self.end(id, offset);
    }

    pub(super) fn begin(&mut self, shape: Shape, start: usize) -> usize {
        if !self.active {
            self.active = true;
            self.depth = 0;
            self.prefixes.push(Prefix {
                pending: false,
                byte: 0,
                angles: 0,
                offset: 0,
            });
        }
        self.open_fragments += 1;
        let index = self.fragments.len();
        self.fragments.push(Fragment {
            start,
            end: start,
            begin: self.events.len(),
            angle: 0,
            shape,
            left_grouped: false,
            no_close: false,
            external: false,
            group: false,
        });
        self.events.push(Event::Begin(index));
        index
    }

    pub(super) fn end(&mut self, begin: usize, offset: usize) {
        let fragment = &mut self.fragments[begin];
        fragment.end = offset;
        fragment.no_close = self.last_close.is_none_or(|close| close < fragment.start);
        self.events.push(Event::End(begin));
        self.open_fragments -= 1;
        self.maybe_end_region();
    }

    pub(super) fn operator(&mut self, openings: usize, left_end: usize) {
        let begin = *self.binary_fragments.last().unwrap();
        // Read actual emitted grouping, including precedence and cast groups.
        // A delimited type head cannot take another type-argument list.
        self.fragments[begin].left_grouped =
            self.last_group == Some((self.fragments[begin].start, left_end));
        self.fragments[begin].angle = self.angles - openings + 1;
        self.events.push(Event::Operator(begin));
        let prefix = self.prefixes.last_mut().unwrap();
        if !prefix.pending {
            prefix.pending = true;
            self.pending += 1;
        }
    }

    pub(super) fn mapping(&mut self, offset: usize, original: u32, name: &'a [u8]) {
        let index = self.mappings.len();
        self.mappings.push(Mapping {
            offset,
            original,
            name,
        });
        self.events.push(Event::Mapping(index));
    }

    fn clear_prefix(&mut self) {
        let prefix = self.prefixes.last_mut().unwrap();
        if prefix.pending {
            self.pending -= 1;
            prefix.pending = false;
        }
    }

    #[inline]
    pub(super) fn type_break(&mut self) {
        if self.active {
            self.active_type_break();
        }
    }

    #[inline(never)]
    fn active_type_break(&mut self) {
        self.events.push(Event::TypeBreak);
        self.clear_prefix();
        self.maybe_end_region();
    }

    #[inline]
    pub(super) fn begin_value(&mut self) -> bool {
        if !self.active {
            return false;
        }
        self.begin_active_value()
    }

    #[inline(never)]
    fn begin_active_value(&mut self) -> bool {
        self.events.push(Event::TypeBreak);
        self.clear_prefix();
        self.maybe_end_region();
        if !self.active {
            return false;
        }
        self.begin_scope();
        self.events.push(Event::ValueStart);
        true
    }

    #[inline]
    pub(super) fn begin_type(&mut self) -> bool {
        if !self.active {
            return false;
        }
        self.begin_active_type();
        true
    }

    #[inline(never)]
    fn begin_active_type(&mut self) {
        self.begin_scope();
        self.events.push(Event::TypeStart);
    }

    fn begin_scope(&mut self) {
        self.scopes.push((self.pending, self.angles));
        self.suspended += self.pending;
        self.pending = 0;
        self.angles = 0;
        self.prefixes.push(Prefix {
            pending: false,
            byte: 0,
            angles: 0,
            offset: 0,
        });
    }

    pub(super) fn end_value(&mut self) {
        self.end_scope();
        self.events.push(Event::ValueEnd);
        self.maybe_end_region();
    }

    pub(super) fn end_type(&mut self) {
        self.end_scope();
        self.events.push(Event::TypeEnd);
        self.maybe_end_region();
    }

    fn end_scope(&mut self) {
        self.prefixes.pop().unwrap();
        let (pending, angles) = self.scopes.pop().unwrap();
        self.pending = pending;
        self.angles = angles;
        self.suspended -= pending;
    }

    #[inline(always)]
    pub(super) fn syntax(&mut self, text: &[u8], offset: usize) {
        if !self.active {
            return;
        }
        if matches!(text, b">>" | b">>>") && self.angles < text.len() {
            return;
        }
        if matches!(text, b">>" | b">>>") {
            self.close(offset + text.len() - 1, self.angles - text.len() + 1);
            return;
        }
        for (index, &byte) in text.iter().enumerate() {
            if byte == b'<' {
                self.angles += 1;
            }
            if matches!(byte, b'(' | b'[' | b'{' | b')' | b']' | b'}' | b';')
                || byte == b'>'
                    && (index == 0 || text[index - 1] != b'=')
                    && text.get(index + 1) != Some(&b'=')
            {
                self.syntax_token(byte, offset + index);
            }
        }
    }

    #[inline(always)]
    fn syntax_token(&mut self, byte: u8, offset: usize) {
        if self.active {
            if byte == b'>' {
                self.close(offset, self.angles);
            } else {
                self.active_token(byte, offset);
            }
        }
    }

    fn close(&mut self, offset: usize, eligible: usize) {
        self.last_close = Some(offset);
        if self.pending == 0 {
            return;
        }
        let index = self.closes.len();
        self.closes.push(Close { offset, eligible });
        self.events.push(Event::Close(index));
        self.maybe_end_region();
    }

    fn active_token(&mut self, byte: u8, offset: usize) {
        match byte {
            b'(' | b'[' | b'{' => {
                self.depth += 1;
                self.prefixes.push(Prefix {
                    pending: false,
                    byte,
                    angles: self.angles,
                    offset,
                });
            }
            b')' | b']' | b'}' => {
                if self.depth == 0 {
                    debug_assert_eq!(self.open_fragments, 0);
                    self.end_region();
                    return;
                }
                self.depth -= 1;
                let prefix = self.prefixes.pop().unwrap();
                self.angles = prefix.angles;
                self.last_group = Some((prefix.offset, offset + 1));
                if prefix.pending {
                    self.pending -= 1;
                }
            }
            b';' => self.clear_prefix(),
            _ => unreachable!(),
        }
        self.events.push(Event::Syntax { byte });
        self.maybe_end_region();
    }

    fn maybe_end_region(&mut self) {
        if self.open_fragments == 0 && self.pending == 0 && self.suspended == 0 {
            self.end_region();
        }
    }

    fn end_region(&mut self) {
        debug_assert!(self.scopes.is_empty());
        // The region began in a frame with no earlier candidate. Balance only
        // frames opened in this region; physical ancestors belong to no trace.
        for prefix in self.prefixes.drain(1..).rev() {
            let byte = match prefix.byte {
                b'(' => b')',
                b'[' => b']',
                b'{' => b'}',
                _ => unreachable!(),
            };
            self.events.push(Event::Syntax { byte });
        }
        self.events.push(Event::TypeBreak);
        self.prefixes.clear();
        self.active = false;
        self.pending = 0;
        self.angles = 0;
        self.last_group = None;
    }

    fn complete(&mut self) {
        if self.active {
            self.end_region();
        }
    }

    fn select(&mut self, index: usize) {
        let fragment = &mut self.fragments[index];
        if fragment.group {
            return;
        }
        self.has_groups = true;
        fragment.group = true;
    }

    fn plan(mut self, code: &[u8]) -> Self {
        #[derive(Default)]
        struct Frame {
            suffix: usize,
        }
        let mut stack = vec![Frame::default()];
        let mut index = self.events.len();
        while index > 0 {
            index -= 1;
            match self.events[index] {
                Event::End(id) => {
                    let fragment = self.fragments[id];
                    let Shape {
                        wrapped, rejects, ..
                    } = fragment.shape;
                    let frame = stack.last_mut().unwrap();
                    let angle = fragment.angle;
                    let risk = frame.suffix >= angle;
                    if wrapped || rejects {
                        frame.suffix = 0;
                    }
                    self.fragments[id].external = risk && !wrapped && !rejects;
                    if !fragment.no_close {
                        continue;
                    }
                    if self.fragments[id].external {
                        self.select(id);
                        frame.suffix = 0;
                    }
                    if fragment.left_grouped {
                        frame.suffix = 0;
                    }
                    index = fragment.begin;
                }
                Event::Operator(id) => {
                    if self.fragments[id].external
                        && stack.last().unwrap().suffix >= self.fragments[id].angle
                    {
                        self.select(id);
                        stack.last_mut().unwrap().suffix = 0;
                    }
                    if self.fragments[id].left_grouped {
                        stack.last_mut().unwrap().suffix = 0;
                    }
                }
                Event::Begin(id) if self.fragments[id].group => {
                    stack.last_mut().unwrap().suffix = 0;
                }
                Event::ValueEnd => stack.push(Frame::default()),
                Event::ValueStart => {
                    stack.pop().unwrap();
                    stack.last_mut().unwrap().suffix = 0;
                }
                Event::TypeEnd => stack.push(Frame::default()),
                Event::TypeStart => {
                    stack.pop().unwrap();
                }
                Event::Syntax {
                    byte: b')' | b']' | b'}',
                    ..
                } => {
                    stack.push(Frame::default());
                }
                Event::Syntax {
                    byte: b'(' | b'[' | b'{',
                    ..
                } => {
                    stack.pop().unwrap();
                }
                Event::Syntax { byte: b';', .. } | Event::TypeBreak => {
                    stack.last_mut().unwrap().suffix = 0;
                }
                Event::Close(id) => {
                    let close = self.closes[id];
                    if printed_follower_can_commit(code, close.offset + 1) {
                        let frame = stack.last_mut().unwrap();
                        frame.suffix = frame.suffix.max(close.eligible);
                    }
                }
                _ => {}
            }
        }
        self
    }

    fn apply(
        mut self,
        code: &mut Vec<u8>,
        builder: &mut crate::sourcemap::ChunkBuilder<'a>,
        with_map: bool,
    ) {
        let mut edits: Vec<(usize, u8)> = Vec::new();
        for event in &self.events {
            match *event {
                Event::Begin(id) => {
                    let fragment = self.fragments[id];
                    if fragment.group && !fragment.shape.wrapped {
                        edits.push((fragment.start, b'('));
                    }
                }
                Event::End(id) => {
                    let fragment = self.fragments[id];
                    if fragment.group && !fragment.shape.wrapped {
                        edits.push((fragment.end, b')'));
                    }
                }
                Event::Mapping(id) => self.mappings[id].offset += edits.len(),
                _ => {}
            }
        }
        debug_assert!(edits.windows(2).all(|pair| pair[0].0 <= pair[1].0));
        let mut read = code.len();
        let mut write = read + edits.len();
        code.resize(write, 0);
        for &(position, byte) in edits.iter().rev() {
            let length = read - position;
            write -= length;
            code.copy_within(position..read, write);
            write -= 1;
            code[write] = byte;
            read = position;
        }
        debug_assert_eq!(read, write);
        if with_map {
            self.replay_mappings(code, builder);
        }
    }

    fn replay_mappings(self, code: &[u8], builder: &mut crate::sourcemap::ChunkBuilder<'a>) {
        for mapping in self.mappings {
            builder.add_source_mapping(mapping.original, mapping.name, &code[..mapping.offset]);
        }
    }
}
