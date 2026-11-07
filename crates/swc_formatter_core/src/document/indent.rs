//! Persistent indentation queues used when root and dedent scopes are present.
use super::AlignText;
use crate::{IndentStyle, PrintOptions};

#[derive(Clone, Copy, Default)]
pub(super) struct State {
    queue: usize,
    root: usize,
}

#[derive(Clone, Copy, Default)]
struct Queue {
    previous: usize,
    prefix: usize,
    spaces: usize,
    aligns: usize,
    width: usize,
    non_whitespace: bool,
}

#[derive(Clone, Copy)]
enum Part<'a> {
    Empty,
    Spaces(usize),
    Tabs(usize),
    Text(&'a str),
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

    pub(super) fn dedent(&self, state: State) -> State {
        State {
            queue: self.queue[state.queue].previous,
            ..state
        }
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
        self.scratch.clear();
        let mut at = queue.prefix;
        while at != 0 {
            let node = self.prefix[at];
            self.scratch.push(node.part);
            at = node.previous;
        }
        for part in self.scratch.iter().rev() {
            match *part {
                Part::Empty => {}
                Part::Spaces(n) => output.extend(std::iter::repeat_n(' ', n)),
                Part::Tabs(n) => output.extend(std::iter::repeat_n('\t', n)),
                Part::Text(text) => output.push_str(text),
            }
        }
        output.extend(std::iter::repeat_n(' ', queue.spaces));
    }
}
