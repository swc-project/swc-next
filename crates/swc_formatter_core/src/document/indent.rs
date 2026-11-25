//! Indentation emission and persistent queues for root and dedent scopes.
use super::AlignText;
use crate::{IndentStyle, PrintOptions};

pub(super) fn append_whitespace(output: &mut String, style: IndentStyle, count: usize) {
    let byte = match style {
        IndentStyle::Space => b' ',
        IndentStyle::Tab => b'\t',
    };
    let end = output
        .len()
        .checked_add(count)
        .expect("indentation exceeds output capacity");
    // SAFETY: Resizing upwards with ASCII whitespace preserves UTF-8.
    unsafe { output.as_mut_vec().resize(end, byte) };
}

#[derive(Clone, Copy, Default)]
pub(super) struct State {
    queue: usize,
    root: usize,
}

#[derive(Clone, Copy, Default)]
// Keep the compressed-history counter after the ordinary queue fields.
#[repr(C)]
struct Queue {
    previous: usize,
    prefix: usize,
    spaces: usize,
    aligns: usize,
    width: usize,
    non_whitespace: bool,
    // Intermediate indentation states omitted from the previous link.
    skipped_indents: u32,
}

#[derive(Clone, Copy)]
enum Part<'a> {
    Empty,
    Spaces(usize),
    Tabs(usize),
    Text(&'a str),
}

impl Part<'_> {
    fn emit(self, output: &mut String) {
        match self {
            Self::Empty => {}
            Self::Spaces(n) => append_whitespace(output, IndentStyle::Space, n),
            Self::Tabs(n) => append_whitespace(output, IndentStyle::Tab, n),
            Self::Text(text) => output.push_str(text),
        }
    }
}

#[derive(Clone, Copy)]
struct Prefix<'a> {
    previous: usize,
    part: Part<'a>,
}

pub(super) struct Rich<'a> {
    queue: Vec<Queue>,
    roots: Vec<State>,
    prefix: Vec<Prefix<'a>>,
    scratch: Vec<Part<'a>>,
    last_prefix: usize,
    prefix_text: String,
}

impl Default for Rich<'_> {
    fn default() -> Self {
        Self {
            queue: vec![Queue::default()],
            roots: vec![State::default()],
            prefix: vec![Prefix {
                previous: 0,
                part: Part::Empty,
            }],
            scratch: Vec::new(),
            last_prefix: 0,
            prefix_text: String::new(),
        }
    }
}

impl<'a> Rich<'a> {
    fn part(&mut self, previous: usize, part: Part<'a>) -> usize {
        let id = self.prefix.len();
        self.prefix.push(Prefix { previous, part });
        id
    }

    fn push(&mut self, state: State, mut queue: Queue) -> State {
        queue.previous = state.queue;
        queue.skipped_indents = 0;
        let id = self.queue.len();
        self.queue.push(queue);
        State { queue: id, ..state }
    }

    fn flush(&mut self, queue: &mut Queue, options: PrintOptions) {
        if queue.aligns == 0 {
            return;
        }
        let (part, width) = if options.indent_style == IndentStyle::Tab {
            (
                Part::Tabs(queue.aligns),
                queue.aligns * usize::from(options.indent_width.value()),
            )
        } else {
            (Part::Spaces(queue.spaces), queue.spaces)
        };
        queue.prefix = self.part(queue.prefix, part);
        queue.width = queue.width - queue.spaces + width;
        queue.spaces = 0;
        queue.aligns = 0;
    }

    pub(super) fn indent(&mut self, state: State, options: PrintOptions) -> State {
        let mut queue = self.queue[state.queue];
        self.flush(&mut queue, options);
        let width = usize::from(options.indent_width.value());
        let part = if options.indent_style == IndentStyle::Tab {
            Part::Tabs(1)
        } else {
            Part::Spaces(width)
        };
        queue.prefix = self.part(queue.prefix, part);
        queue.width += width;
        self.push(state, queue)
    }

    pub(super) fn absolute(&mut self, state: State, width: usize, options: PrintOptions) -> State {
        let unit = usize::from(options.indent_width.value());
        let (levels, remainder) = match width.checked_div(unit) {
            Some(levels) => (levels, width % unit),
            None => (0, width),
        };
        let state = self.align(self.root(state), remainder);
        match levels {
            0 => return state,
            1 => return self.indent(state, options),
            _ => {}
        }
        let mut queue = self.queue[state.queue];
        self.flush(&mut queue, options);
        let part = match options.indent_style {
            IndentStyle::Space => Part::Spaces(levels * unit),
            IndentStyle::Tab => Part::Tabs(levels),
        };
        queue.prefix = self.part(queue.prefix, part);
        queue.width += levels * unit;
        // The base stays unflushed so exhausting the run restores precisely the
        // alignment state that existed before the first indentation level.
        let state = self.push(state, queue);
        let queue = &mut self.queue[state.queue];
        queue.skipped_indents = u32::try_from(levels - 1).expect("absolute indentation fits u32");
        state
    }

    pub(super) fn align(&mut self, state: State, width: usize) -> State {
        if width == 0 {
            return state;
        }
        let mut queue = self.queue[state.queue];
        queue.spaces += width;
        queue.aligns += 1;
        queue.width += width;
        self.push(state, queue)
    }

    pub(super) fn text(
        &mut self,
        state: State,
        text: &'a AlignText<'a>,
        options: PrintOptions,
    ) -> State {
        if text.source.is_empty() {
            return state;
        }
        let mut queue = self.queue[state.queue];
        self.flush(&mut queue, options);
        queue.prefix = self.part(queue.prefix, Part::Text(text.source));
        queue.width += text.width;
        queue.non_whitespace |= text.source.bytes().any(|b| !matches!(b, b' ' | b'\t'));
        self.push(state, queue)
    }

    #[inline]
    pub(super) fn dedent(&mut self, state: State, options: PrintOptions) -> State {
        if self.queue[state.queue].skipped_indents != 0 {
            return self.dedent_run(state, options);
        }
        State {
            queue: self.queue[state.queue].previous,
            ..state
        }
    }

    #[cold]
    #[inline(never)]
    fn dedent_run(&mut self, state: State, options: PrintOptions) -> State {
        let mut queue = self.queue[state.queue];
        let prefix = self.prefix[queue.prefix];
        let width = usize::from(options.indent_width.value());
        let part = match prefix.part {
            Part::Spaces(spaces) => Part::Spaces(spaces - width),
            Part::Tabs(tabs) => Part::Tabs(tabs - 1),
            _ => unreachable!("counted indentation has a whitespace prefix"),
        };
        queue.prefix = self.part(prefix.previous, part);
        queue.width -= width;
        queue.skipped_indents -= 1;
        let id = self.queue.len();
        self.queue.push(queue);
        State { queue: id, ..state }
    }

    pub(super) fn mark_root(&mut self, state: State) -> State {
        let root = self.roots.len();
        self.roots.push(state);
        State { root, ..state }
    }

    pub(super) fn root(&self, state: State) -> State {
        self.roots[state.root]
    }
    pub(super) fn width(&self, state: State) -> usize {
        self.queue[state.queue].width
    }
    pub(super) fn is_empty(&self, state: State) -> bool {
        let q = self.queue[state.queue];
        q.prefix == 0 && q.spaces == 0
    }
    pub(super) fn non_whitespace(&self, state: State) -> bool {
        self.queue[state.queue].non_whitespace
    }

    pub(super) fn emit(&mut self, state: State, output: &mut String) {
        let queue = self.queue[state.queue];
        let prefix = self.prefix[queue.prefix];
        if prefix.previous == 0 {
            prefix.part.emit(output);
        } else {
            // Prefix nodes are immutable. Lines at the same indentation share the spelling.
            if self.last_prefix != queue.prefix {
                self.prefix_text.clear();
                self.scratch.clear();
                let mut at = queue.prefix;
                while at != 0 {
                    let node = self.prefix[at];
                    self.scratch.push(node.part);
                    at = node.previous;
                }
                for part in self.scratch.iter().rev() {
                    part.emit(&mut self.prefix_text);
                }
                self.last_prefix = queue.prefix;
            }
            output.push_str(&self.prefix_text);
        }
        append_whitespace(output, IndentStyle::Space, queue.spaces);
    }
}
