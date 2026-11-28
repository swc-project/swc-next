use std::{
    cmp::{max, min},
    fmt,
    ops::{Deref, DerefMut, Range},
    sync::Arc,
};

/// Half-open UTF-8 byte offsets into the original input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    pub fn len(self) -> usize {
        self.end - self.start
    }
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
    pub fn text(self, source: &str) -> &str {
        &source[self.start..self.end]
    }
    pub(crate) fn bytes(self, source: &str) -> &[u8] {
        &source.as_bytes()[self.start..self.end]
    }
}

/// A view of one physical source line after stripping container prefixes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLine {
    /// Original physical line excluding its ending, including container markers and any initial BOM.
    pub raw: Span,
    pub content: Span,
    /// Original line terminator, or an empty span at EOF.
    pub ending: Span,
    /// Remaining spaces when a container boundary consumed only part of a tab.
    pub virtual_spaces: u8,
    pub column: usize,
    /// An enclosing container accepted this line as a lazy paragraph continuation.
    pub lazy: bool,
}

/// A contiguous view of physical lines or of the logical lines of one container.
#[derive(Clone)]
pub struct SourceLines(LineStorage);

#[derive(Clone)]
enum LineStorage {
    Empty,
    Single(SourceLine),
    Shared {
        rows: Arc<[SourceLine]>,
        range: Range<usize>,
    },
}

impl SourceLines {
    #[inline]
    pub fn as_slice(&self) -> &[SourceLine] {
        match &self.0 {
            LineStorage::Empty => &[],
            LineStorage::Single(line) => std::slice::from_ref(line),
            LineStorage::Shared { rows, range } => &rows[range.clone()],
        }
    }

    #[inline]
    pub(crate) fn slice(&self, range: Range<usize>) -> Self {
        match &self[range.clone()] {
            [] => return Self(LineStorage::Empty),
            [line] => return (*line).into(),
            _ => {}
        }
        match &self.0 {
            LineStorage::Shared {
                rows,
                range: parent,
            } => Self(LineStorage::Shared {
                rows: rows.clone(),
                range: parent.start + range.start..parent.start + range.end,
            }),
            _ => unreachable!("multiple lines share a table"),
        }
    }

    pub(crate) fn skip_first(&mut self) {
        *self = self.slice(1..self.len());
    }

    fn shared(rows: Arc<[SourceLine]>) -> Self {
        Self(LineStorage::Shared {
            range: 0..rows.len(),
            rows,
        })
    }
}

impl From<SourceLine> for SourceLines {
    fn from(line: SourceLine) -> Self {
        Self(LineStorage::Single(line))
    }
}

impl From<Vec<SourceLine>> for SourceLines {
    fn from(rows: Vec<SourceLine>) -> Self {
        match rows.as_slice() {
            [] => Self(LineStorage::Empty),
            [line] => (*line).into(),
            _ => Self::shared(rows.into()),
        }
    }
}

impl From<&[SourceLine]> for SourceLines {
    fn from(rows: &[SourceLine]) -> Self {
        match rows {
            [] => Self(LineStorage::Empty),
            [line] => (*line).into(),
            _ => Self::shared(rows.into()),
        }
    }
}

impl FromIterator<SourceLine> for SourceLines {
    fn from_iter<T: IntoIterator<Item = SourceLine>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let Some(first) = iter.next() else {
            return Self(LineStorage::Empty);
        };
        let Some(second) = iter.next() else {
            return first.into();
        };
        Self::shared([first, second].into_iter().chain(iter).collect())
    }
}

impl Deref for SourceLines {
    type Target = [SourceLine];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for SourceLines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.0 {
            LineStorage::Empty => &mut [],
            LineStorage::Single(line) => std::slice::from_mut(line),
            LineStorage::Shared { rows, range } => {
                if range.len() != rows.len() {
                    *rows = rows[range.clone()].into();
                    *range = 0..rows.len();
                }
                Arc::make_mut(rows)
            }
        }
    }
}

impl<'a> IntoIterator for &'a SourceLines {
    type Item = &'a SourceLine;
    type IntoIter = std::slice::Iter<'a, SourceLine>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl fmt::Debug for SourceLines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

pub(crate) fn lines(source: &str) -> Vec<SourceLine> {
    let bytes = source.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    while start < bytes.len() {
        let end = memchr::memchr2(b'\r', b'\n', &bytes[start..]).map_or(bytes.len(), |n| start + n);
        let next = match bytes.get(end) {
            Some(b'\r') if bytes.get(end + 1) == Some(&b'\n') => end + 2,
            Some(_) => end + 1,
            None => end,
        };
        let mut line = SourceLine::new(Span::new(start, end), Span::new(end, next));
        if start == 0 && source.starts_with('\u{feff}') {
            line.content.start = 3;
        }
        lines.push(line);
        start = next;
    }
    lines
}

/// The source already contains these logical lines joined by single line feeds.
pub(crate) fn contiguous_content(source: &str, lines: &[SourceLine]) -> Option<Span> {
    let (first, rest) = lines.split_first()?;
    let mut end = first.content.end;
    if first.virtual_spaces != 0 {
        return None;
    }
    for line in rest {
        if line.virtual_spaces != 0
            || end + 1 != line.content.start
            || source.as_bytes().get(end) != Some(&b'\n')
        {
            return None;
        }
        end = line.content.end;
    }
    Some(Span::new(first.content.start, end))
}

impl SourceLine {
    pub(crate) fn new(raw: Span, ending: Span) -> Self {
        Self {
            raw,
            content: raw,
            ending,
            virtual_spaces: 0,
            column: 0,
            lazy: false,
        }
    }

    pub fn physical(self) -> Span {
        Span::new(self.raw.start, self.ending.end)
    }
    /// Physical bytes preceding this logical line, including enclosing prefixes.
    /// A tab shared with the content is kept in `raw_content`.
    pub fn prefix(self) -> Span {
        Span::new(self.raw.start, self.start())
    }
    pub fn raw_content(self) -> Span {
        Span::new(self.start(), self.content.end)
    }
    pub fn append(&self, source: &str, output: &mut String) {
        output.extend(std::iter::repeat_n(' ', self.virtual_spaces as usize));
        output.push_str(self.content.text(source));
    }
    pub(crate) fn start(self) -> usize {
        self.content.start - usize::from(self.virtual_spaces != 0)
    }
    pub(crate) fn blank(self, source: &str) -> bool {
        self.first_content_byte(source).is_none()
    }
    pub(crate) fn first_content_byte(self, source: &str) -> Option<u8> {
        self.content
            .bytes(source)
            .iter()
            .copied()
            .find(|b| !matches!(b, b' ' | b'\t'))
    }
    /// Leading indentation in columns after removing enclosing container prefixes.
    pub fn indent(self, source: &str) -> usize {
        let mut col = self.column + usize::from(self.virtual_spaces);
        for b in self.content.bytes(source) {
            match b {
                b' ' => col += 1,
                b'\t' => col += 4 - col % 4,
                _ => break,
            }
        }
        col - self.column
    }
    /// Remove up to the requested number of indentation columns, even through a tab stop.
    pub(crate) fn strip(mut self, source: &str, mut columns: usize) -> Self {
        let virtual_used = min(usize::from(self.virtual_spaces), columns);
        self.virtual_spaces -= virtual_used as u8;
        self.column += virtual_used;
        columns -= virtual_used;
        while columns != 0 && self.content.start < self.content.end {
            let width = match source.as_bytes()[self.content.start] {
                b' ' => 1,
                b'\t' => 4 - self.column % 4,
                _ => break,
            };
            self.content.start += 1;
            let used = min(columns, width);
            self.column += used;
            columns -= used;
            self.virtual_spaces = (width - used) as u8;
        }
        self
    }
    /// Advance across an ASCII syntax marker with no preceding logical indentation.
    pub(crate) fn marker(mut self, bytes: usize) -> Self {
        debug_assert_eq!(self.virtual_spaces, 0);
        self.content.start += bytes;
        self.column += bytes;
        self
    }
    pub(crate) fn clipped(&self, span: Span) -> Option<Self> {
        if span.end <= self.start() || self.ending.end <= span.start {
            return None;
        }
        let mut line = *self;
        line.content.start = max(line.content.start, span.start).min(line.content.end);
        line.content.end = min(line.content.end, span.end).max(line.content.start);
        if span.start > self.start() {
            line.virtual_spaces = 0;
        }
        line.ending.start = max(line.ending.start, span.start).min(line.ending.end);
        line.ending.end = min(line.ending.end, span.end).max(line.ending.start);
        Some(line)
    }
}
