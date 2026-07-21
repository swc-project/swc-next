use swc_experimental_allocator::{Allocator, vec::Vec};

pub struct SourceMap<'a> {
    pub version: u8,
    pub file: Option<&'a [u8]>,
    pub source_root: Option<&'a [u8]>,
    pub sources: &'a [&'a [u8]],
    pub sources_content: Option<&'a [Option<&'a [u8]>]>,
    pub names: &'a [&'a [u8]],
    pub mappings: &'a [u8],
}

#[derive(Clone, Copy, Default)]
pub struct Options<'a> {
    pub file: Option<&'a [u8]>,
    pub source_file_name: Option<&'a [u8]>,
    pub source_root: Option<&'a [u8]>,
    pub sources_content: Option<&'a [u8]>,
}

#[derive(Clone, Copy)]
pub struct Segment {
    pub gen_line: u32,
    pub gen_col: u32,
    pub orig_line: u32,
    pub orig_col: u32,
}

pub struct State<'a> {
    pub options: Options<'a>,
    pub out: Vec<'a, u8>,
    pub gen_line: u32,
    pub gen_col: u32,
    pub line_cursor: u32,
    pub pending: Segment,
    pub has_pending: bool,
    pub prev_gen_col: i32,
    pub prev_orig_line: i32,
    pub prev_orig_col: i32,
    pub enc_gen_line: i32,
    pub first_in_line: bool,
}

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub out_len: u32,
    pub gen_line: u32,
    pub gen_col: u32,
    pub pending: Segment,
    pub has_pending: bool,
    pub prev_gen_col: i32,
    pub prev_orig_line: i32,
    pub prev_orig_col: i32,
    pub enc_gen_line: i32,
    pub first_in_line: bool,
}

impl<'a> State<'a> {
    pub fn init(options: Options<'a>, allocator: &'a Allocator) -> Self {
        Self {
            options,
            out: Vec::new_in(allocator),
            gen_line: 0,
            gen_col: 0,
            line_cursor: 0,
            pending: Segment {
                gen_line: 0,
                gen_col: 0,
                orig_line: 0,
                orig_col: 0,
            },
            has_pending: false,
            prev_gen_col: 0,
            prev_orig_line: 0,
            prev_orig_col: 0,
            enc_gen_line: 0,
            first_in_line: true,
        }
    }

    pub fn advance(&mut self, bytes: &[u8]) {
        if Self::is_plain_ascii(bytes) {
            self.gen_col += bytes.len() as u32;
            return;
        }
        let mut i = 0;
        let mut col_inc = 0;
        let mut saw_break = false;
        while i < bytes.len() {
            let line_break_len = line_break_len(bytes, i);
            if line_break_len > 0 {
                self.gen_line += 1;
                col_inc = 0;
                saw_break = true;
                i += line_break_len;
                continue;
            }
            let byte = bytes[i];
            let len = if byte < 0x80 {
                1
            } else if (0xc2..=0xdf).contains(&byte) {
                2
            } else if (0xe0..=0xef).contains(&byte) {
                3
            } else if (0xf0..=0xf4).contains(&byte) {
                4
            } else {
                i += 1;
                continue;
            };
            col_inc += if len == 4 { 2 } else { 1 };
            i += len;
        }
        self.gen_col = if saw_break {
            col_inc
        } else {
            self.gen_col + col_inc
        };
    }

    #[inline]
    fn is_plain_ascii(bytes: &[u8]) -> bool {
        bytes
            .iter()
            .all(|&byte| byte < 0x80 && !matches!(byte, b'\n' | b'\r'))
    }

    pub fn record(&mut self, _allocator: &'a Allocator, orig_line: u32, orig_col: u32) {
        if self.has_pending
            && self.pending.gen_line == self.gen_line
            && self.pending.gen_col == self.gen_col
        {
            self.pending.orig_line = orig_line;
            self.pending.orig_col = orig_col;
            return;
        }
        if self.has_pending {
            self.flush(_allocator);
        }
        self.pending = Segment {
            gen_line: self.gen_line,
            gen_col: self.gen_col,
            orig_line,
            orig_col,
        };
        self.has_pending = true;
    }

    fn flush(&mut self, _allocator: &'a Allocator) {
        let segment = self.pending;
        let gen_line = segment.gen_line as i32;
        let line_gap = (gen_line - self.enc_gen_line) as usize;
        let needed = line_gap + 2 + 3 * 7;
        self.out
            .reserve(needed.saturating_sub(self.out.capacity() - self.out.len()));

        if gen_line > self.enc_gen_line {
            self.out.resize(self.out.len() + line_gap, b';');
            self.enc_gen_line = gen_line;
            self.prev_gen_col = 0;
            self.first_in_line = true;
        }
        if !self.first_in_line {
            self.out.push(b',');
        }
        self.first_in_line = false;

        let gen_col = segment.gen_col as i32;
        write_vlq(&mut self.out, gen_col - self.prev_gen_col);
        self.prev_gen_col = gen_col;
        self.out.push(b'A');
        let orig_line = segment.orig_line as i32;
        write_vlq(&mut self.out, orig_line - self.prev_orig_line);
        self.prev_orig_line = orig_line;
        let orig_col = segment.orig_col as i32;
        write_vlq(&mut self.out, orig_col - self.prev_orig_col);
        self.prev_orig_col = orig_col;
    }

    pub fn last_mapping(&self) -> Option<Segment> {
        self.has_pending.then_some(self.pending)
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            out_len: self.out.len() as u32,
            gen_line: self.gen_line,
            gen_col: self.gen_col,
            pending: self.pending,
            has_pending: self.has_pending,
            prev_gen_col: self.prev_gen_col,
            prev_orig_line: self.prev_orig_line,
            prev_orig_col: self.prev_orig_col,
            enc_gen_line: self.enc_gen_line,
            first_in_line: self.first_in_line,
        }
    }

    pub fn restore(&mut self, snapshot: Snapshot) {
        assert!(snapshot.out_len as usize <= self.out.len());
        self.out.truncate(snapshot.out_len as usize);
        self.gen_line = snapshot.gen_line;
        self.gen_col = snapshot.gen_col;
        self.pending = snapshot.pending;
        self.has_pending = snapshot.has_pending;
        self.prev_gen_col = snapshot.prev_gen_col;
        self.prev_orig_line = snapshot.prev_orig_line;
        self.prev_orig_col = snapshot.prev_orig_col;
        self.enc_gen_line = snapshot.enc_gen_line;
        self.first_in_line = snapshot.first_in_line;
    }

    pub fn build(&mut self, allocator: &'a Allocator) -> SourceMap<'a> {
        if self.has_pending {
            self.flush(allocator);
            self.has_pending = false;
        }
        let mappings = std::mem::replace(&mut self.out, Vec::new_in(allocator)).into_bump_slice();
        let mut sources = Vec::with_capacity_in(1, allocator);
        sources.push(self.options.source_file_name.unwrap_or(b""));
        let sources = sources.into_bump_slice();
        let sources_content = self.options.sources_content.map(|content| {
            let mut values = Vec::with_capacity_in(1, allocator);
            values.push(Some(content));
            values.into_bump_slice() as &'a [Option<&'a [u8]>]
        });
        SourceMap {
            version: 3,
            file: self.options.file,
            source_root: self.options.source_root,
            sources,
            sources_content,
            names: &[],
            mappings,
        }
    }
}

fn line_break_len(bytes: &[u8], index: usize) -> usize {
    match bytes[index] {
        b'\n' => 1,
        b'\r' if bytes.get(index + 1) == Some(&b'\n') => 2,
        b'\r' => 1,
        0xe2 if matches!(
            bytes.get(index + 1..index + 3),
            Some([0x80, 0xa8] | [0x80, 0xa9])
        ) =>
        {
            3
        }
        _ => 0,
    }
}

const VLQ_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[inline]
fn write_vlq(out: &mut Vec<'_, u8>, value: i32) {
    let mut bits = if value < 0 {
        ((-value) as u32) << 1 | 1
    } else {
        (value as u32) << 1
    };
    loop {
        let digit = (bits & 0x1f) as u8;
        bits >>= 5;
        out.push(
            VLQ_CHARS[if bits != 0 {
                (digit | 0x20) as usize
            } else {
                digit as usize
            }],
        );
        if bits == 0 {
            return;
        }
    }
}
