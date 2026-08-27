use swc_next_ecma_ast::Span;
use swc_next_ecma_parser::{Token, TokenTag};

#[derive(Clone, Copy)]
pub(crate) struct TokenCursor<'source, 'tokens> {
    source: &'source str,
    tokens: &'tokens [Token],
}

impl<'source, 'tokens> TokenCursor<'source, 'tokens> {
    pub(crate) fn new(source: &'source str, tokens: &'tokens [Token]) -> Self {
        Self { source, tokens }
    }

    pub(crate) fn first_in_range(self, start: u32, end: u32) -> Option<Token> {
        let mut index = self.index_containing_or_after(start);
        while let Some(token) = self.tokens.get(index).copied() {
            if token.span.start >= end {
                return None;
            }
            if token.span.start >= start && token.span.end <= end {
                return Some(token);
            }
            index += 1;
        }
        None
    }

    pub(crate) fn last_in_range(self, start: u32, end: u32) -> Option<Token> {
        let mut index = self.index_starting_at_or_after(end);
        while index > 0 {
            index -= 1;
            let token = self.tokens[index];
            if token.span.end <= start {
                return None;
            }
            if token.span.start >= start && token.span.end <= end {
                return Some(token);
            }
        }
        None
    }

    pub(crate) fn previous_before(self, position: u32) -> Option<Token> {
        let index = self.index_containing_or_after(position);
        index
            .checked_sub(1)
            .and_then(|previous| self.tokens.get(previous).copied())
    }

    pub(crate) fn at_or_after(self, position: u32) -> Option<Token> {
        self.tokens
            .get(self.index_containing_or_after(position))
            .copied()
    }

    #[allow(dead_code)]
    pub(crate) fn find_forward(self, start: u32, end: u32, tag: TokenTag) -> Option<Token> {
        let mut index = self.index_containing_or_after(start);
        while let Some(token) = self.tokens.get(index).copied() {
            if token.span.start >= end {
                return None;
            }
            if token.span.start >= start && token.span.end <= end && token.tag == tag {
                return Some(token);
            }
            index += 1;
        }
        None
    }

    pub(crate) fn find_backward(self, start: u32, end: u32, tag: TokenTag) -> Option<Token> {
        let mut index = self.index_starting_at_or_after(end);
        while index > 0 {
            index -= 1;
            let token = self.tokens[index];
            if token.span.end <= start {
                return None;
            }
            if token.span.start >= start && token.span.end <= end && token.tag == tag {
                return Some(token);
            }
        }
        None
    }

    #[allow(dead_code)]
    pub(crate) fn text(self, token: Token) -> &'source str {
        token.text(self.source)
    }

    pub(crate) fn has_line_terminator(self, start: u32, end: u32) -> bool {
        self.source[start as usize..end as usize].contains(['\n', '\r', '\u{2028}', '\u{2029}'])
    }

    pub(crate) fn trailing_character_span(self, end: u32) -> Option<Span> {
        let prefix = self.source.get(..end as usize)?;
        let (start, _) = prefix.char_indices().next_back()?;
        Some(Span::new(start as u32, end))
    }

    pub(crate) fn leading_character_span(self, start: u32) -> Option<Span> {
        let character = self.source.get(start as usize..)?.chars().next()?;
        Some(Span::new(start, start + character.len_utf8() as u32))
    }

    fn index_containing_or_after(self, position: u32) -> usize {
        self.tokens
            .partition_point(|token| token.span.end <= position)
    }

    fn index_starting_at_or_after(self, position: u32) -> usize {
        self.tokens
            .partition_point(|token| token.span.start < position)
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Lang, SourceType};
    use swc_next_ecma_parser::{CommentMode, Options, Parser, TokenParserConfig};

    use super::*;

    #[test]
    fn range_lookup_skips_trivia_and_respects_direction() {
        let source = "import { A, /* keep */ type B, C } from 'm';";
        let allocator = Allocator::default();
        let parsed = Parser::init(
            &allocator,
            source,
            Options {
                source_type: SourceType::Module,
                lang: Lang::Ts,
                preserve_parens: true,
                comments: CommentMode::None,
            },
            TokenParserConfig,
        )
        .parse();
        let cursor = TokenCursor::new(source, parsed.tokens.as_slice());
        let forward = cursor
            .find_forward(0, source.len() as u32, TokenTag::Comma)
            .unwrap();
        let backward = cursor
            .find_backward(0, source.len() as u32, TokenTag::Comma)
            .unwrap();

        assert_eq!(forward.span.start, 10);
        assert_eq!(backward.span.start, 29);
    }

    #[test]
    fn range_endpoints_use_parser_token_boundaries() {
        let source = "type Result = Box<Foo>;";
        let allocator = Allocator::default();
        let parsed = Parser::init(
            &allocator,
            source,
            Options {
                source_type: SourceType::Module,
                lang: Lang::Ts,
                preserve_parens: true,
                comments: CommentMode::None,
            },
            TokenParserConfig,
        )
        .parse();
        let cursor = TokenCursor::new(source, parsed.tokens.as_slice());
        let first = cursor.first_in_range(5, 22).unwrap();
        let last = cursor.last_in_range(5, 22).unwrap();

        assert_eq!(cursor.text(first), "Result");
        assert_eq!(cursor.text(last), ">");
    }
}
